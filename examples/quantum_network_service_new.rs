use qnect::builder::BackendType;
use qnect::error::QnectError;
use qnect::network::network::{LinkType, QuantumNetwork};
use qnect::network::repeater::{EntanglementSwapper, QuantumRepeaterNode};
use qnect::quantum::state::Gate1Q;

use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

struct QuantumService {
    network: Arc<Mutex<QuantumNetwork>>,
    swapper: Arc<Mutex<EntanglementSwapper>>,
}

impl QuantumService {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create the full network topology
        let mut net = QuantumNetwork::new_distributed();

        // Add all nodes
        net.add_distributed_node("Alice", 256, BackendType::Stabilizer)?;
        net.add_distributed_node("Repeater", 64, BackendType::Stabilizer)?;
        net.add_distributed_node("Bob", 256, BackendType::Stabilizer)?;

        // Add quantum links
        net.add_quantum_link(
            "Alice",
            "Repeater",
            LinkType::Fiber {
                length_km: 500.0,
                loss_db_per_km: 0.2,
            },
            0.85,
            100.0,
        )?;

        net.add_quantum_link(
            "Repeater",
            "Bob",
            LinkType::Fiber {
                length_km: 500.0,
                loss_db_per_km: 0.2,
            },
            0.85,
            100.0,
        )?;

        let network_arc = Arc::new(Mutex::new(net));
        let swapper = Arc::new(Mutex::new(EntanglementSwapper::new(network_arc.clone())));

        // Add the repeater node to the swapper
        {
            let mut swap = swapper.lock().await;
            swap.add_repeater(QuantumRepeaterNode::new(
                "Repeater".to_string(),
                (48.8566, 2.3522), // Paris coordinates
                32,                // Memory slots
            ));
        }

        Ok(QuantumService {
            network: network_arc,
            swapper,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║            QUANTUM NETWORK SERVICE - ULTIMATE EDITION          ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!("\nInitializing quantum infrastructure...\n");

    let service = QuantumService::new().await?;

    // Print network topology
    {
        let net = service.network.lock().await;
        println!("📊 Network Topology:");
        println!("├─ Alice (London): 256 qubits");
        println!("├─ Repeater (Paris): 64 qubits + 32 memory slots");
        println!("└─ Bob (Berlin): 256 qubits");
        println!("\n🔗 Quantum Links:");
        println!("├─ Alice <--500km fiber--> Repeater (85% fidelity)");
        println!("└─ Repeater <--500km fiber--> Bob (85% fidelity)");

        // Show nonlocal resource tracking
        println!("\n📦 Nonlocal Resources:");
        println!("├─ Bell pairs: {}", net.nonlocal.bells.len());
        println!("└─ GHZ states: {}", net.nonlocal.ghzs.len());
    }

    println!("\n✅ Quantum network initialized");
    println!("🔊 Listening on port 6666...\n");

    let listener = TcpListener::bind("127.0.0.1:6666").await?;
    let service = Arc::new(service);

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("[CONNECT] Client connected: {}", addr);

        let service_clone = service.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, service_clone).await {
                eprintln!("[ERROR] Client handler: {}", e);
            }
        });
    }
}

async fn handle_client(
    socket: TcpStream,
    service: Arc<QuantumService>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while reader.read_line(&mut line).await? > 0 {
        let parts: Vec<&str> = line.trim().split(':').collect();

        let response = match parts[0] {
            "ALLOCATE" => {
                let node = parts[1];
                let mut net = service.network.lock().await;
                match net.allocate_local_qubit(node) {
                    Ok(q) => {
                        println!("[ALLOCATE] {} allocated qubit {}", node, q);
                        format!("OK:{}\n", q)
                    }
                    Err(e) => format!("ERR:{:?}\n", e),
                }
            }

            "GATE" => {
                let node = parts[1];
                let qubit: usize = parts[2].parse()?;
                let gate = parts[3];

                let mut net = service.network.lock().await;
                let result = match gate {
                    "H" => net.apply_local_gate(node, qubit, Gate1Q::H),
                    "X" => net.apply_local_gate(node, qubit, Gate1Q::X),
                    "Y" => net.apply_local_gate(node, qubit, Gate1Q::Y),
                    "Z" => net.apply_local_gate(node, qubit, Gate1Q::Z),
                    "S" => net.apply_local_gate(node, qubit, Gate1Q::S),
                    "T" => net.apply_local_gate(node, qubit, Gate1Q::T),
                    _ => Err(QnectError::invalid_operation("operation", "invalid")),
                };

                match result {
                    Ok(_) => {
                        println!("[GATE] {} applied {} to qubit {}", node, gate, qubit);
                        "OK\n".to_string()
                    }
                    Err(e) => format!("ERR:{:?}\n", e),
                }
            }

            "MEASURE" => {
                let node = parts[1];
                let qubit: usize = parts[2].parse()?;

                let mut net = service.network.lock().await;
                match net.measure(node, qubit) {
                    Ok(result) => {
                        println!("[MEASURE] {} measured qubit {} = {}", node, qubit, result);
                        format!("OK:{}\n", result)
                    }
                    Err(e) => format!("ERR:{:?}\n", e),
                }
            }

            "EPR" => {
                let node1 = parts[1];
                let node2 = parts[2];

                let mut net = service.network.lock().await;
                match net.create_epr_pair(node1, node2) {
                    Ok((q1, q2)) => {
                        println!(
                            "[EPR] Created Bell pair: {}:q{} <-> {}:q{}",
                            node1, q1, node2, q2
                        );

                        // Show it's tracked in NonlocalStore
                        if let Some(bell_id) = net.is_bell_qubit(node1, q1) {
                            println!("      └─> Tracked with Bell ID: {}", bell_id);
                        }

                        format!("OK:{}:{}\n", q1, q2)
                    }
                    Err(e) => format!("ERR:{:?}\n", e),
                }
            }

            "TELEPORT" => {
                let from = parts[1];
                let to = parts[2];
                let qubit: usize = parts[3].parse()?;

                let mut net = service.network.lock().await;

                // Check if direct connection or need multi-hop
                let path = net.find_shortest_path(from, to).ok_or("No path found")?;

                println!("[TELEPORT] Path: {:?}", path);

                let result = if path.len() == 2 {
                    // Direct teleportation
                    net.quantum_teleportation(from, to, qubit).await
                } else {
                    // Multi-hop teleportation through repeaters
                    println!("           Using multi-hop teleportation...");

                    // First establish end-to-end entanglement
                    match net.establish_end_to_end_entanglement(from, to).await {
                        Ok((src_epr, dst_epr)) => {
                            println!(
                                "           E2E entanglement: {}:q{} <-> {}:q{}",
                                from, src_epr, to, dst_epr
                            );

                            // Then teleport using the established entanglement
                            // (simplified - in real implementation would use the E2E pair)
                            net.quantum_teleportation(from, to, qubit).await
                        }
                        Err(e) => Err(e),
                    }
                };

                match result {
                    Ok(new_q) => {
                        println!(
                            "[TELEPORT] {} teleported q{} to {} (new q{})",
                            from, qubit, to, new_q
                        );
                        format!("OK:{}\n", new_q)
                    }
                    Err(e) => format!("ERR:{:?}\n", e),
                }
            }

            "SWAP" => {
                // Entanglement swapping at repeater
                let repeater = parts[1];
                let left_q: usize = parts[2].parse()?;
                let right_q: usize = parts[3].parse()?;

                let mut net = service.network.lock().await;

                // Apply Bell measurement at repeater
                match net.nodes.get_mut(repeater) {
                    Some(node) => {
                        if let Some(system) = &mut node.local_system {
                            // Bell measurement
                            let _ = tokio::task::block_in_place(|| {
                                tokio::runtime::Handle::current().block_on(async {
                                    system.cnot(left_q, right_q).await?;
                                    system.h(left_q).await
                                })
                            });

                            let m1 = net.measure(repeater, left_q)?;
                            let m2 = net.measure(repeater, right_q)?;

                            println!("[SWAP] {} performed swap: m1={}, m2={}", repeater, m1, m2);
                            format!("OK:{}:{}\n", m1, m2)
                        } else {
                            "ERR:No quantum system\n".to_string()
                        }
                    }
                    None => "ERR:Repeater not found\n".to_string(),
                }
            }

            "PATH" => {
                // Find path between nodes
                let from = parts[1];
                let to = parts[2];

                let net = service.network.lock().await;
                match net.find_shortest_path(from, to) {
                    Some(path) => {
                        println!("[PATH] {} -> {}: {:?}", from, to, path);
                        format!("OK:{}\n", path.join(","))
                    }
                    None => "ERR:No path\n".to_string(),
                }
            }

            "STATS" => {
                // Network statistics
                let net = service.network.lock().await;
                let stats = net.get_stats();

                println!("[STATS] Network statistics requested");
                format!(
                    "OK:nodes={},qubits={},links={},mode={:?}\n",
                    stats.total_nodes, stats.total_qubits, stats.total_links, stats.mode
                )
            }

            "VISUALIZE" => {
                // Network visualization
                let net = service.network.lock().await;
                let viz = net.visualize_network();
                println!("{}", viz);
                "OK:Visualization printed to console\n".to_string()
            }

            _ => "ERR:Unknown command\n".to_string(),
        };

        writer.write_all(response.as_bytes()).await?;
        writer.flush().await?;
        line.clear();
    }

    println!("[DISCONNECT] Client disconnected");
    Ok(())
}
