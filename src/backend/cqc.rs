// use crate::builder::BackendType;
// use crate::network::cqc::cqc::*;
// use crate::network::network::QuantumNetwork;
// use crate::state::Gate1Q;
// use std::collections::HashMap;
// use std::mem;
// use std::sync::Arc;
// use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use tokio::net::{TcpListener, TcpStream};
// use tokio::sync::Mutex;

// // Import CQC types from the module
// use super::*;

// pub struct CQCBackend {
//     network: Arc<Mutex<QuantumNetwork>>,
//     nodes: Arc<Mutex<HashMap<String, CQCNode>>>,
//     port: u16,
// }

// pub struct CQCNode {
//     pub app_id: u16,
//     pub node_name: String,
//     pub allocated_qubits: HashMap<u16, usize>, // CQC qubit_id -> network qubit_id
//     pub next_qubit_id: u16,
// }

// impl CQCBackend {
//     pub async fn new(port: u16) -> Result<Self, Box<dyn std::error::Error>> {
//         let network = QuantumNetwork::new_distributed();

//         Ok(CQCBackend {
//             network: Arc::new(Mutex::new(network)),
//             nodes: Arc::new(Mutex::new(HashMap::new())),
//             port,
//         })
//     }

//     pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
//         let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
//         info!("CQC Backend listening on port {}", self.port);

//         loop {
//             let (socket, addr) = listener.accept().await?;
//             info!("New CQC connection from {}", addr);

//             let network = self.network.clone();
//             let nodes = self.nodes.clone();

//             tokio::spawn(async move {
//                 if let Err(e) = handle_client(socket, network, nodes).await {
//                     error!("Client error: {}", e);
//                 }
//             });
//         }
//     }
// }

// async fn handle_client(
//     mut socket: TcpStream,
//     network: Arc<Mutex<QuantumNetwork>>,
//     nodes: Arc<Mutex<HashMap<String, CQCNode>>>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let mut buffer = vec![0u8; 1024];

//     loop {
//         // Read CQC Header
//         let n = socket.read(&mut buffer[..8]).await?;
//         if n == 0 {
//             break; // Connection closed
//         }

//         let header = CQCHeader::from_bytes(&buffer[..8])?;

//         // Extract fields to avoid packed struct alignment issues
//         let msg_type = header.msg_type;
//         let app_id = header.app_id;
//         let length = header.length;

//         info!(
//             "Received CQC message: type={}, app_id={}, length={}",
//             msg_type, app_id, length
//         );

//         // Read rest of message
//         if length > 0 {
//             socket.read_exact(&mut buffer[..length as usize]).await?;
//         }

//         // Process message
//         let response =
//             process_cqc_message(header, &buffer[..length as usize], &network, &nodes).await?;

//         // Send response
//         for resp_bytes in response {
//             socket.write_all(&resp_bytes).await?;
//         }
//     }

//     Ok(())
// }

// async fn process_cqc_message(
//     header: CQCHeader,
//     payload: &[u8],
//     network: &Arc<Mutex<QuantumNetwork>>,
//     nodes: &Arc<Mutex<HashMap<String, CQCNode>>>,
// ) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
//     let mut responses = Vec::new();

//     // Extract header fields to avoid alignment issues
//     let msg_type = header.msg_type;
//     let app_id = header.app_id;

//     match msg_type {
//         x if x == CQCType::Hello as u8 => {
//             // Respond with HELLO
//             let resp_header = CQCHeader::new(CQCType::Hello, app_id, 0);
//             responses.push(resp_header.to_bytes());
//         }

//         x if x == CQCType::Command as u8 => {
//             // Process command sequence
//             let mut offset = 0;

//             while offset < payload.len() {
//                 let cmd_header = CQCCmdHeader::from_bytes(&payload[offset..])?;
//                 offset += std::mem::size_of::<CQCCmdHeader>();

//                 // Extract fields to avoid alignment issues
//                 let qubit_id = cmd_header.qubit_id;
//                 let instr = cmd_header.instr;
//                 let options = cmd_header.options;

//                 match instr {
//                     x if x == CQCCmd::New as u8 => {
//                         // Allocate new qubit
//                         let mut net = network.lock().await;
//                         let mut nodes_map = nodes.lock().await;

//                         // Get or create node for this app
//                         let node_name = format!("app_{}", app_id);
//                         if !nodes_map.contains_key(&node_name) {
//                             net.add_distributed_node(&node_name, 100, BackendType::Stabilizer)?;
//                             nodes_map.insert(node_name.clone(), CQCNode {
//                                 app_id,
//                                 node_name: node_name.clone(),
//                                 allocated_qubits: HashMap::new(),
//                                 next_qubit_id: 0,
//                             });
//                         }

//                         let node = nodes_map.get_mut(&node_name).unwrap();
//                         let network_qubit = net.allocate_local_qubit(&node_name)?;
//                         let cqc_qubit_id = node.next_qubit_id;
//                         node.next_qubit_id += 1;
//                         node.allocated_qubits.insert(cqc_qubit_id, network_qubit);

//                         // Send NEW_OK response
//                         let resp_header = CQCHeader::new(CQCType::NewOk, app_id, 2);
//                         let qubit_header = CQCXtraQubitHeader {
//                             qubit_id: cqc_qubit_id,
//                         };

//                         responses.push(resp_header.to_bytes());
//                         responses.push(qubit_header.to_bytes());

//                         if options & CQC_OPT_NOTIFY != 0 {
//                             let done_header = CQCHeader::new(CQCType::Done, app_id, 0);
//                             responses.push(done_header.to_bytes());
//                         }
//                     }

//                     x if x == CQCCmd::H as u8 => {
//                         // Apply Hadamard
//                         let mut net = network.lock().await;
//                         let nodes_map = nodes.lock().await;
//                         let node_name = format!("app_{}", app_id);
//                         let node = nodes_map.get(&node_name).ok_or("Node not found")?;
//                         let network_qubit = node
//                             .allocated_qubits
//                             .get(&qubit_id)
//                             .ok_or("Qubit not found")?;

//                         net.apply_local_gate(&node_name, *network_qubit, Gate1Q::H)?;

//                         if options & CQC_OPT_NOTIFY != 0 {
//                             let done_header = CQCHeader::new(CQCType::Done, app_id, 0);
//                             responses.push(done_header.to_bytes());
//                         }
//                     }

//                     x if x == CQCCmd::Measure as u8 => {
//                         // Measure qubit
//                         let mut net = network.lock().await;
//                         let nodes_map = nodes.lock().await;
//                         let node_name = format!("app_{}", app_id);
//                         let node = nodes_map.get(&node_name).ok_or("Node not found")?;
//                         let network_qubit = node
//                             .allocated_qubits
//                             .get(&qubit_id)
//                             .ok_or("Qubit not found")?;

//                         let outcome = net.measure(&node_name, *network_qubit)?;

//                         // Send MEASOUT response
//                         let resp_header = CQCHeader::new(CQCType::MeasOut, app_id, 1);
//                         let meas_header = CQCMeasOutHeader { meas_out: outcome };

//                         responses.push(resp_header.to_bytes());
//                         responses.push(meas_header.to_bytes());

//                         if options & CQC_OPT_NOTIFY != 0 {
//                             let done_header = CQCHeader::new(CQCType::Done, app_id, 0);
//                             responses.push(done_header.to_bytes());
//                         }
//                     }

//                     x if x == CQCCmd::X as u8 => {
//                         // Apply X gate
//                         let mut net = network.lock().await;
//                         let nodes_map = nodes.lock().await;
//                         let node_name = format!("app_{}", app_id);
//                         let node = nodes_map.get(&node_name).ok_or("Node not found")?;
//                         let network_qubit = node
//                             .allocated_qubits
//                             .get(&qubit_id)
//                             .ok_or("Qubit not found")?;

//                         net.apply_local_gate(&node_name, *network_qubit, Gate1Q::X)?;

//                         if options & CQC_OPT_NOTIFY != 0 {
//                             let done_header = CQCHeader::new(CQCType::Done, app_id, 0);
//                             responses.push(done_header.to_bytes());
//                         }
//                     }

//                     x if x == CQCCmd::Z as u8 => {
//                         // Apply Z gate
//                         let mut net = network.lock().await;
//                         let nodes_map = nodes.lock().await;
//                         let node_name = format!("app_{}", app_id);
//                         let node = nodes_map.get(&node_name).ok_or("Node not found")?;
//                         let network_qubit = node
//                             .allocated_qubits
//                             .get(&qubit_id)
//                             .ok_or("Qubit not found")?;

//                         net.apply_local_gate(&node_name, *network_qubit, Gate1Q::Z)?;

//                         if options & CQC_OPT_NOTIFY != 0 {
//                             let done_header = CQCHeader::new(CQCType::Done, app_id, 0);
//                             responses.push(done_header.to_bytes());
//                         }
//                     }

//                     x if x == CQCCmd::Y as u8 => {
//                         // Apply Y gate
//                         let mut net = network.lock().await;
//                         let nodes_map = nodes.lock().await;
//                         let node_name = format!("app_{}", app_id);
//                         let node = nodes_map.get(&node_name).ok_or("Node not found")?;
//                         let network_qubit = node
//                             .allocated_qubits
//                             .get(&qubit_id)
//                             .ok_or("Qubit not found")?;

//                         net.apply_local_gate(&node_name, *network_qubit, Gate1Q::Y)?;

//                         if options & CQC_OPT_NOTIFY != 0 {
//                             let done_header = CQCHeader::new(CQCType::Done, app_id, 0);
//                             responses.push(done_header.to_bytes());
//                         }
//                     }

//                     x if x == CQCCmd::T as u8 => {
//                         // Apply T gate
//                         let mut net = network.lock().await;
//                         let nodes_map = nodes.lock().await;
//                         let node_name = format!("app_{}", app_id);
//                         let node = nodes_map.get(&node_name).ok_or("Node not found")?;
//                         let network_qubit = node
//                             .allocated_qubits
//                             .get(&qubit_id)
//                             .ok_or("Qubit not found")?;

//                         net.apply_local_gate(&node_name, *network_qubit, Gate1Q::T)?;

//                         if options & CQC_OPT_NOTIFY != 0 {
//                             let done_header = CQCHeader::new(CQCType::Done, app_id, 0);
//                             responses.push(done_header.to_bytes());
//                         }
//                     }

//                     x if x == CQCCmd::RotX as u8
//                         || x == CQCCmd::RotY as u8
//                         || x == CQCCmd::RotZ as u8 =>
//                     {
//                         // Rotation gates need extra header
//                         if offset + std::mem::size_of::<CQCRotationHeader>() > payload.len() {
//                             return Err("Missing rotation header".into());
//                         }

//                         let rot_header = CQCRotationHeader::from_bytes(&payload[offset..])?;
//                         offset += std::mem::size_of::<CQCRotationHeader>();

//                         let angle = (rot_header.step as f64) * std::f64::consts::PI / 128.0;

//                         let mut net = network.lock().await;
//                         let nodes_map = nodes.lock().await;
//                         let node_name = format!("app_{}", app_id);
//                         let node = nodes_map.get(&node_name).ok_or("Node not found")?;
//                         let network_qubit = node
//                             .allocated_qubits
//                             .get(&qubit_id)
//                             .ok_or("Qubit not found")?;

//                         let gate = match instr {
//                             x if x == CQCCmd::RotX as u8 => Gate1Q::Rx(angle),
//                             x if x == CQCCmd::RotY as u8 => Gate1Q::Ry(angle),
//                             x if x == CQCCmd::RotZ as u8 => Gate1Q::Rz(angle),
//                             _ => unreachable!(),
//                         };

//                         net.apply_local_gate(&node_name, *network_qubit, gate)?;

//                         if options & CQC_OPT_NOTIFY != 0 {
//                             let done_header = CQCHeader::new(CQCType::Done, app_id, 0);
//                             responses.push(done_header.to_bytes());
//                         }
//                     }

//                     // Add more commands as needed...
//                     _ => {
//                         error!("Unimplemented command: {}", instr);
//                         let err_header = CQCHeader::new(CQCType::ErrUnsupp, app_id, 0);
//                         responses.push(err_header.to_bytes());
//                     }
//                 }
//             }
//         }

//         _ => {
//             error!("Unknown message type: {}", msg_type);
//             let err_header = CQCHeader::new(CQCType::ErrGeneral, app_id, 0);
//             responses.push(err_header.to_bytes());
//         }
//     }

//     Ok(responses)
// }
