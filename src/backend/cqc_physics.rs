// use crate::backend::cqc::CQCNode;
// use crate::builder::BackendType;
// use crate::network::cqc::cqc::*;
// use crate::network::network::QuantumNetwork;
// use crate::physics::{EPRSource, NoiseModel, QuantumChannel, QubitMemory};
// use crate::state::Gate1Q;
// use std::collections::HashMap;
// use std::sync::Arc;
// use std::sync::atomic::{AtomicU64, Ordering};
// use std::time::{Duration, Instant};
// use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use tokio::net::{TcpListener, TcpStream};
// use tokio::sync::Mutex;

// /// Production CQC Backend with realistic physics
// #[derive(Clone)]
// pub struct ProductionCQCBackend {
//     network: Arc<Mutex<QuantumNetwork>>,
//     nodes: Arc<Mutex<HashMap<String, CQCNode>>>,
//     port: u16,

//     // Physics simulation
//     noise_models: Arc<HashMap<String, NoiseModel>>,
//     quantum_memories: Arc<Mutex<HashMap<(String, usize), QubitMemory>>>,
//     channels: Arc<HashMap<(String, String), QuantumChannel>>,
//     epr_sources: Arc<HashMap<String, EPRSource>>,

//     // Statistics
//     total_gates_applied: Arc<AtomicU64>,
//     total_errors: Arc<AtomicU64>,
//     total_epr_attempts: Arc<AtomicU64>,
//     successful_epr: Arc<AtomicU64>,
// }

// impl ProductionCQCBackend {
//     pub async fn new_with_physics(port: u16) -> Result<Self, Box<dyn std::error::Error>> {
//         let network = QuantumNetwork::new_distributed();

//         // Setup realistic noise models
//         let mut noise_models = HashMap::new();
//         noise_models.insert("singapore".to_string(), NoiseModel::realistic_nv_center());
//         noise_models.insert("amsterdam".to_string(), NoiseModel::realistic_ion_trap());
//         noise_models.insert("newyork".to_string(), NoiseModel::realistic_nv_center());

//         // Setup quantum channels
//         let mut channels = HashMap::new();
//         channels.insert(
//             ("singapore".to_string(), "amsterdam".to_string()),
//             QuantumChannel::realistic_fiber(10_000.0),
//         );
//         channels.insert(
//             ("amsterdam".to_string(), "newyork".to_string()),
//             QuantumChannel::realistic_fiber(6_000.0),
//         );

//         // Setup EPR sources
//         let mut epr_sources = HashMap::new();
//         epr_sources.insert("singapore".to_string(), EPRSource::realistic_nv_center());
//         epr_sources.insert("amsterdam".to_string(), EPRSource::realistic_spdc());
//         epr_sources.insert("newyork".to_string(), EPRSource::realistic_nv_center());

//         Ok(ProductionCQCBackend {
//             network: Arc::new(Mutex::new(network)),
//             nodes: Arc::new(Mutex::new(HashMap::new())),
//             port,
//             noise_models: Arc::new(noise_models),
//             quantum_memories: Arc::new(Mutex::new(HashMap::new())),
//             channels: Arc::new(channels),
//             epr_sources: Arc::new(epr_sources),
//             total_gates_applied: Arc::new(AtomicU64::new(0)),
//             total_errors: Arc::new(AtomicU64::new(0)),
//             total_epr_attempts: Arc::new(AtomicU64::new(0)),
//             successful_epr: Arc::new(AtomicU64::new(0)),
//         })
//     }

//     /// Apply a gate with realistic noise simulation
//     pub async fn apply_gate_with_noise(
//         &self,
//         node_name: &str,
//         qubit: usize,
//         gate: Gate1Q,
//         is_two_qubit: bool,
//     ) -> Result<(), Box<dyn std::error::Error>> {
//         self.total_gates_applied.fetch_add(1, Ordering::Relaxed);

//         // Get noise model for this node
//         let noise = self
//             .noise_models
//             .get(node_name)
//             .ok_or_else(|| format!("No noise model for node {}", node_name))?;

//         // Check if gate error occurs
//         if noise.apply_gate_noise(is_two_qubit) {
//             self.total_errors.fetch_add(1, Ordering::Relaxed);

//             // Apply random Pauli error
//             let error_gate = match rand::random::<u8>() % 3 {
//                 0 => Gate1Q::X,
//                 1 => Gate1Q::Y,
//                 _ => Gate1Q::Z,
//             };

//             // Apply error gate
//             let mut net = self.network.lock().await;
//             net.apply_local_gate(node_name, qubit, error_gate)?;
//         }

//         // Apply the intended gate
//         let mut net = self.network.lock().await;
//         net.apply_local_gate(node_name, qubit, gate)?;

//         // Update memory timestamps and check decoherence
//         let mut memories = self.quantum_memories.lock().await;
//         if let Some(memory) = memories.get_mut(&(node_name.to_string(), qubit)) {
//             memory.last_operation_time = Instant::now();

//             // Check if qubit has decohered
//             if memory.is_decohered(0.5) {
//                 return Err("Qubit decohered below threshold".into());
//             }

//             // Apply decoherence to the state
//             let elapsed = memory.allocation_time.elapsed();
//             let t = elapsed.as_secs_f64();

//             // T2 dephasing
//             let dephasing_factor = (-t * noise.dephasing_rate).exp();
//             memory.initial_fidelity *= dephasing_factor;
//         }

//         Ok(())
//     }

//     /// Measure a qubit with realistic noise
//     pub async fn measure_with_noise(
//         &self,
//         node_name: &str,
//         qubit: usize,
//     ) -> Result<u8, Box<dyn std::error::Error>> {
//         // Get noise model
//         let noise = self
//             .noise_models
//             .get(node_name)
//             .ok_or_else(|| format!("No noise model for node {}", node_name))?;

//         // Perform measurement
//         let mut net = self.network.lock().await;
//         let mut outcome = net.measure(node_name, qubit)?;

//         // Apply measurement noise
//         outcome = noise.apply_measurement_noise(outcome);

//         // Remove from memory tracking
//         let mut memories = self.quantum_memories.lock().await;
//         memories.remove(&(node_name.to_string(), qubit));

//         Ok(outcome)
//     }

//     /// Create EPR pair with realistic physics
//     pub async fn create_realistic_epr(
//         &self,
//         node1: &str,
//         node2: &str,
//     ) -> Result<(usize, usize, f64), Box<dyn std::error::Error>> {
//         self.total_epr_attempts.fetch_add(1, Ordering::Relaxed);

//         // Get EPR source (use node1's source)
//         let source = self
//             .epr_sources
//             .get(node1)
//             .ok_or_else(|| format!("No EPR source at node {}", node1))?;

//         // Get channel between nodes
//         let channel = self
//             .channels
//             .get(&(node1.to_string(), node2.to_string()))
//             .or_else(|| self.channels.get(&(node2.to_string(), node1.to_string())))
//             .ok_or("No quantum channel between nodes")?;

//         // Simulate generation time
//         tokio::time::sleep(source.generation_time()).await;

//         // Check if photons survive transmission
//         let photon1_arrives = channel.transmit_photon();
//         let photon2_arrives = channel.transmit_photon();

//         if !photon1_arrives || !photon2_arrives {
//             return Err("Photon loss in channel".into());
//         }

//         // Calculate resulting fidelity
//         let raw_fidelity = source.raw_fidelity;
//         let channel_fidelity = channel.output_fidelity(raw_fidelity);

//         // Create EPR pair in network
//         let mut net = self.network.lock().await;
//         let (q1, q2) = net.create_epr_pair(node1, node2)?;

//         // Store in quantum memory with decoherence tracking
//         let mut memories = self.quantum_memories.lock().await;

//         let coherence_time = Duration::from_millis(100); // 100ms for NV centers
//         memories.insert(
//             (node1.to_string(), q1),
//             QubitMemory::new(q1, coherence_time),
//         );
//         memories.insert(
//             (node2.to_string(), q2),
//             QubitMemory::new(q2, coherence_time),
//         );

//         self.successful_epr.fetch_add(1, Ordering::Relaxed);

//         Ok((q1, q2, channel_fidelity))
//     }

//     /// Allocate a qubit with memory tracking
//     pub async fn allocate_qubit(
//         &self,
//         node_name: &str,
//     ) -> Result<usize, Box<dyn std::error::Error>> {
//         let mut net = self.network.lock().await;
//         let qubit = net.allocate_local_qubit(node_name)?;

//         // Track in quantum memory
//         let mut memories = self.quantum_memories.lock().await;
//         let coherence_time = Duration::from_millis(100);
//         memories.insert(
//             (node_name.to_string(), qubit),
//             QubitMemory::new(qubit, coherence_time),
//         );

//         Ok(qubit)
//     }

//     pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
//         let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
//         info!("Production CQC Backend listening on port {}", self.port);

//         // Clone self for the spawned task
//         let backend_clone = self.clone();

//         // Spawn statistics monitor
//         tokio::spawn(async move {
//             loop {
//                 tokio::time::sleep(Duration::from_secs(30)).await;
//                 backend_clone.print_physics_stats().await;
//             }
//         });

//         loop {
//             let (socket, addr) = listener.accept().await?;
//             info!("New CQC connection from {}", addr);

//             let network = self.network.clone();
//             let nodes = self.nodes.clone();
//             let memories = self.quantum_memories.clone();
//             let noise_models = self.noise_models.clone();
//             let channels = self.channels.clone();
//             let gate_counter = self.total_gates_applied.clone();
//             let error_counter = self.total_errors.clone();

//             tokio::spawn(async move {
//                 if let Err(e) = handle_cqc_client(
//                     socket,
//                     network,
//                     nodes,
//                     memories,
//                     noise_models,
//                     channels,
//                     gate_counter,
//                     error_counter,
//                 )
//                 .await
//                 {
//                     einfo!("Client error: {}", e);
//                 }
//             });
//         }
//     }

//     pub async fn print_physics_stats(&self) {
//         let gates = self.total_gates_applied.load(Ordering::Relaxed);
//         let errors = self.total_errors.load(Ordering::Relaxed);
//         let epr_attempts = self.total_epr_attempts.load(Ordering::Relaxed);
//         let epr_success = self.successful_epr.load(Ordering::Relaxed);

//         info!("\n📊 Physics Simulation Statistics:");
//         info!("├─ Gate operations: {}", gates);
//         info!(
//             "├─ Gate errors: {} ({:.2}%)",
//             errors,
//             if gates > 0 {
//                 errors as f64 / gates as f64 * 100.0
//             } else {
//                 0.0
//             }
//         );
//         info!("├─ EPR attempts: {}", epr_attempts);
//         info!(
//             "├─ EPR success: {} ({:.2}%)",
//             epr_success,
//             if epr_attempts > 0 {
//                 epr_success as f64 / epr_attempts as f64 * 100.0
//             } else {
//                 0.0
//             }
//         );

//         // Check memory states
//         let memories = self.quantum_memories.lock().await;
//         let active = memories
//             .iter()
//             .filter(|(_, m)| !m.is_decohered(0.5))
//             .count();
//         let decohered = memories.len() - active;

//         info!("├─ Active qubits: {}", active);
//         info!("└─ Decohered qubits: {}", decohered);
//     }
// }

// async fn handle_cqc_client(
//     mut socket: TcpStream,
//     network: Arc<Mutex<QuantumNetwork>>,
//     nodes: Arc<Mutex<HashMap<String, CQCNode>>>,
//     memories: Arc<Mutex<HashMap<(String, usize), QubitMemory>>>,
//     noise_models: Arc<HashMap<String, NoiseModel>>,
//     channels: Arc<HashMap<(String, String), QuantumChannel>>,
//     gate_counter: Arc<AtomicU64>,
//     error_counter: Arc<AtomicU64>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let mut buffer = vec![0u8; 4096];

//     loop {
//         let n = socket.read(&mut buffer[..8]).await?;
//         if n == 0 {
//             break;
//         }

//         let header = CQCHeader::from_bytes(&buffer[..8])?;
//         let length = header.length;

//         if length > 0 {
//             socket.read_exact(&mut buffer[..length as usize]).await?;
//         }

//         // Process with physics - for now, use regular CQC processing
//         let response = process_cqc_message_with_physics(
//             header,
//             &buffer[..length as usize],
//             &network,
//             &nodes,
//             &memories,
//             &noise_models,
//             gate_counter.clone(),
//             error_counter.clone(),
//         )
//         .await?;

//         // Write responses
//         for resp_bytes in response {
//             socket.write_all(&resp_bytes).await?;
//         }
//     }

//     Ok(())
// }

// async fn process_cqc_message_with_physics(
//     header: CQCHeader,
//     payload: &[u8],
//     network: &Arc<Mutex<QuantumNetwork>>,
//     nodes: &Arc<Mutex<HashMap<String, CQCNode>>>,
//     memories: &Arc<Mutex<HashMap<(String, usize), QubitMemory>>>,
//     noise_models: &Arc<HashMap<String, NoiseModel>>,
//     gate_counter: Arc<AtomicU64>,
//     error_counter: Arc<AtomicU64>,
// ) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
//     let mut responses = Vec::new();
//     let msg_type = header.msg_type;
//     let app_id = header.app_id;

//     match msg_type {
//         x if x == CQCType::Hello as u8 => {
//             // Respond with HELLO
//             let resp_header = CQCHeader::new(CQCType::Hello, app_id, 0);
//             responses.push(resp_header.to_bytes());
//         }

//         x if x == CQCType::Command as u8 => {
//             // Process command with physics
//             let mut offset = 0;

//             while offset < payload.len() {
//                 let cmd_header = CQCCmdHeader::from_bytes(&payload[offset..])?;
//                 offset += std::mem::size_of::<CQCCmdHeader>();

//                 let qubit_id = cmd_header.qubit_id;
//                 let instr = cmd_header.instr;

//                 match instr {
//                     x if x == CQCCmd::New as u8 => {
//                         // Allocate new qubit with memory tracking
//                         let mut net = network.lock().await;
//                         let mut nodes_map = nodes.lock().await;

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

//                         // Track in quantum memory
//                         let mut mems = memories.lock().await;
//                         mems.insert(
//                             (node_name, network_qubit),
//                             QubitMemory::new(network_qubit, Duration::from_millis(100)),
//                         );

//                         // Send NEW_OK response
//                         let resp_header = CQCHeader::new(CQCType::NewOk, app_id, 2);
//                         let qubit_header = CQCXtraQubitHeader {
//                             qubit_id: cqc_qubit_id,
//                         };

//                         responses.push(resp_header.to_bytes());
//                         responses.push(qubit_header.to_bytes());
//                     }

//                     x if x == CQCCmd::H as u8 => {
//                         // Apply Hadamard with noise
//                         gate_counter.fetch_add(1, Ordering::Relaxed);

//                         let mut net = network.lock().await;
//                         let nodes_map = nodes.lock().await;
//                         let node_name = format!("app_{}", app_id);

//                         if let Some(node) = nodes_map.get(&node_name) {
//                             if let Some(&network_qubit) = node.allocated_qubits.get(&qubit_id) {
//                                 // Check for gate error
//                                 if let Some(noise) = noise_models.get(&node_name) {
//                                     if noise.apply_gate_noise(false) {
//                                         error_counter.fetch_add(1, Ordering::Relaxed);
//                                         // Apply random error
//                                         net.apply_local_gate(&node_name, network_qubit, Gate1Q::X)?;
//                                     }
//                                 }

//                                 // Apply intended gate
//                                 net.apply_local_gate(&node_name, network_qubit, Gate1Q::H)?;

//                                 // Update memory timestamp
//                                 let mut mems = memories.lock().await;
//                                 if let Some(mem) = mems.get_mut(&(node_name.clone(), network_qubit))
//                                 {
//                                     mem.last_operation_time = Instant::now();
//                                 }
//                             }
//                         }
//                     }

//                     x if x == CQCCmd::Measure as u8 => {
//                         // Measure with noise
//                         let mut net = network.lock().await;
//                         let nodes_map = nodes.lock().await;
//                         let node_name = format!("app_{}", app_id);

//                         if let Some(node) = nodes_map.get(&node_name) {
//                             if let Some(&network_qubit) = node.allocated_qubits.get(&qubit_id) {
//                                 let mut outcome = net.measure(&node_name, network_qubit)?;

//                                 // Apply measurement noise
//                                 if let Some(noise) = noise_models.get(&node_name) {
//                                     outcome = noise.apply_measurement_noise(outcome);
//                                 }

//                                 // Send MEASOUT response
//                                 let resp_header = CQCHeader::new(CQCType::MeasOut, app_id, 1);
//                                 let meas_header = CQCMeasOutHeader { meas_out: outcome };

//                                 responses.push(resp_header.to_bytes());
//                                 responses.push(meas_header.to_bytes());
//                             }
//                         }
//                     }

//                     _ => {
//                         // Other commands not yet implemented with physics
//                         einfo!("Command {} not implemented with physics", instr);
//                     }
//                 }
//             }
//         }

//         _ => {
//             einfo!("Unknown message type: {}", msg_type);
//             let err_header = CQCHeader::new(CQCType::ErrGeneral, app_id, 0);
//             responses.push(err_header.to_bytes());
//         }
//     }

//     Ok(responses)
// }

// // Add this struct if not already defined
// #[repr(C, packed)]
// struct CQCMeasOutHeader {
//     meas_out: u8,
// }

// impl CQCMeasOutHeader {
//     fn to_bytes(&self) -> Vec<u8> {
//         vec![self.meas_out]
//     }
// }
