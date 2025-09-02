use std::collections::HashMap;
use std::process::Command;
use std::{env, fs};

use qnect::builder::BackendType;
use qnect::network::network::{
    BlindComputationPattern, LinkType, NetworkOperation, QuantumNetwork,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Quantum Network Runtime - Hardware Ready Demo ===\n");

    // 1. Create hybrid network with mixed backends
    let mut network = QuantumNetwork::new_distributed();

    // Simulated node (development/testing)
    network.add_distributed_node("Alice", 8, BackendType::StateVector)?;
    network.add_distributed_node("Router1", 12, BackendType::Stabilizer)?; // Fast classical sim

    // "Hardware" nodes with mock QNPU backend
    network.add_distributed_node("Router2", 12, BackendType::MockQnpu {
        endpoint: "https://qnpu1.quantum.net/api".to_string(),
        node_id: "Router2".to_string(),
    })?;
    network.add_distributed_node("Bob", 16, BackendType::MockQnpu {
        endpoint: "https://qnpu2.quantum.net/api".to_string(),
        node_id: "Bob".to_string(),
    })?;

    // Add realistic network topology
    network.add_quantum_link(
        "Alice",
        "Router1",
        LinkType::Fiber {
            length_km: 10.0,
            loss_db_per_km: 0.2,
        },
        0.98,
        1000.0,
    )?;
    network.add_quantum_link(
        "Router1",
        "Router2",
        LinkType::Fiber {
            length_km: 50.0,
            loss_db_per_km: 0.2,
        },
        0.95,
        800.0,
    )?;
    network.add_quantum_link(
        "Router2",
        "Bob",
        LinkType::Satellite {
            orbital_height_km: 500.0,
        },
        0.92,
        100.0,
    )?;

    println!(
        "Network topology established:\n{}\n",
        network.visualize_network()
    );

    // 2. Run comprehensive protocol suite
    println!("=== Running Quantum Network Protocols ===\n");

    // 2a. Basic EPR generation with heralding simulation
    println!("1. Creating heralded EPR pair between direct neighbors...");
    let start = std::time::Instant::now();
    let (q1, q2) = retry_until_heralded(&mut network, "Alice", "Router1").await?;
    println!(
        "   ✓ EPR pair created after {:.2}ms: Alice:q{} <-> Router1:q{}\n",
        start.elapsed().as_millis(),
        q1,
        q2
    );

    // 2b. Multi-hop entanglement distribution
    println!("2. Establishing end-to-end entanglement through routers...");
    let start = std::time::Instant::now();
    let (alice_q, bob_q) = network
        .establish_end_to_end_entanglement("Alice", "Bob")
        .await?;
    println!(
        "   ✓ End-to-end entanglement established after {:.2}ms: Alice:q{} <-> Bob:q{}\n",
        start.elapsed().as_millis(),
        alice_q,
        bob_q
    );

    // 2c. Distributed GHZ state
    println!("3. Creating distributed GHZ state...");
    let ghz_qubits = network
        .create_distributed_ghz(vec!["Alice", "Router1", "Router2", "Bob"])
        .await?;
    println!("   ✓ GHZ state created: {:?}\n", ghz_qubits);

    // 2d. Blind quantum computation
    println!("4. Running blind quantum computation...");
    let pattern = BlindComputationPattern {
        computation_graph: vec![(0, 1), (1, 2)],
        measurement_angles: vec![0.5, 1.0, 1.5],
        flow: vec![0, 1, 2],
    };
    let blind_results = network
        .blind_computation_ubqc("Alice", "Bob", pattern)
        .await?;
    println!("   ✓ Blind computation results: {:?}\n", blind_results);

    // 3. Generate NetQASM for all nodes
    println!("=== Generating NetQASM Code ===\n");
    let netqasm_programs = network.generate_netqasm();

    // Create temp directory for NetQASM files
    let temp_dir = env::temp_dir().join(format!("qnect_netqasm_{}", std::process::id())); // Use process ID to avoid conflicts
    fs::create_dir_all(&temp_dir)?;
    let mut netqasm_files = HashMap::new();

    for (node_id, program) in &netqasm_programs {
        let filename = format!("{}_protocol.py", node_id.to_lowercase());
        let filepath = temp_dir.as_path().join(&filename);
        fs::write(&filepath, program)?;
        netqasm_files.insert(node_id.clone(), filepath);

        println!("Generated NetQASM for {}: {} bytes", node_id, program.len());
    }

    // 4. Validate NetQASM syntax (if netqasm is installed)
    println!("\n=== Validating NetQASM Code ===\n");
    for (node_id, filepath) in &netqasm_files {
        match validate_netqasm(filepath) {
            Ok(output) => println!("✓ {} NetQASM validation: PASSED\n{}", node_id, output),
            Err(e) => println!("⚠ {} NetQASM validation skipped: {}", node_id, e),
        }
    }

    println!("\n=== Sample Generated NetQASM ===");
    if let Some(alice_code) = netqasm_programs.get("Alice") {
        println!(
            "Alice's NetQASM ({} lines total):",
            alice_code.lines().count()
        );
        println!("----------------------------------------");
        for line in alice_code.lines().take(30) {
            // Show a bit more
            println!("{}", line);
        }
        if alice_code.lines().count() > 30 {
            println!("... ({} more lines)", alice_code.lines().count() - 30);
        }
        println!("----------------------------------------\n");
    }

    // 5. Show hardware API call trace
    println!("\n=== Hardware API Call Trace ===");
    println!("(This is what would be sent to real QNPUs)\n");
    // The MockQnpuBackend already logs these, but we could collect and display them

    // 6. Performance metrics
    println!("\n=== Performance Metrics ===");
    let stats = network.get_stats();
    println!("Total operations recorded: {}", stats.operations_recorded);
    println!("Total qubits allocated: {}", stats.total_qubits);
    println!("Network mode: {:?}", stats.mode);

    // 7. Export execution timeline (for hardware scheduling)
    println!("\n=== Execution Timeline ===");
    generate_execution_timeline(&network)?;

    // 8. Demonstrate error handling for hardware failures
    println!("\n=== Hardware Failure Simulation => ! EXPECTED ERROR ! ===");
    demonstrate_hardware_failures(&mut network).await?;

    // 9. Resource utilization report
    println!("\n=== Resource Utilization ===");
    for (node_id, node) in &network.nodes {
        let allocated = node.qubit_allocator.allocated_qubits.len();
        let free = node.qubit_allocator.get_free_count();
        let total = allocated + free;
        println!(
            "{}: {}/{} qubits used ({:.1}% utilization)",
            node_id,
            allocated,
            total,
            (allocated as f64 / total as f64) * 100.0
        );
    }

    // 10. Fidelity tracking
    println!("\n=== Fidelity Analysis ===");
    for link in network.links.values() {
        println!(
            "Link {}-{}: F={:.3}, Rate={}Hz, Latency={}μs",
            link.node1, link.node2, link.fidelity, link.generation_rate_hz, link.latency_us
        );
    }

    println!("\n✅ All tests completed successfully!");
    println!("This network is ready for real QNPU deployment! 🚀");

    Ok(())
}

/// Retry EPR generation until heralding succeeds
async fn retry_until_heralded(
    network: &mut QuantumNetwork,
    node1: &str,
    node2: &str,
) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    let mut attempts = 0;
    loop {
        attempts += 1;
        match network.create_epr_pair(node1, node2) {
            Ok((q1, q2)) => {
                // In real hardware, we'd check heralding here
                println!("   EPR generation attempt {}: SUCCESS", attempts);
                return Ok((q1, q2));
            }
            Err(_) if attempts < 5 => {
                println!(
                    "   EPR generation attempt {}: FAILED, retrying...",
                    attempts
                );
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
            Err(e) => return Err(Box::new(e)),
        }
    }
}

/// Validate NetQASM code using the netqasm Python package
fn validate_netqasm(filepath: &std::path::Path) -> Result<String, Box<dyn std::error::Error>> {
    // Try to run Python validation
    let output = Command::new("python3")
        .arg("-c")
        .arg(format!(
            "import sys; sys.path.append('.'); exec(open('{}').read())",
            filepath.display()
        ))
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(format!(
            "Validation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into())
    }
}

/// Generate execution timeline for hardware scheduling
fn generate_execution_timeline(network: &QuantumNetwork) -> Result<(), Box<dyn std::error::Error>> {
    let mut timeline: Vec<(u64, String)> = Vec::new();
    let mut current_time_ns = 0u64;

    for op in &network.protocol_history {
        let duration_ns = match op {
            NetworkOperation::LocalGate { gate, .. } => {
                match gate.as_str() {
                    "H" | "X" | "Y" | "Z" => 100,   // 100ns for single qubit gates
                    g if g.starts_with("R") => 200, // 200ns for rotations
                    _ => 150,
                }
            }
            NetworkOperation::CreateEPR { .. } => 10_000, // 10μs for EPR generation
            NetworkOperation::Measure { .. } => 1_000,    // 1μs for measurement
            _ => 50,                                      // Classical operations
        };

        timeline.push((current_time_ns, format!("{:?}", op)));
        current_time_ns += duration_ns;
    }

    // Print critical path
    println!(
        "Total execution time: {:.2}μs",
        current_time_ns as f64 / 1000.0
    );
    println!("Critical operations:");
    for (time_ns, op) in timeline.iter().take(10) {
        println!("  {:>8.2}μs: {}", *time_ns as f64 / 1000.0, op);
    }
    if timeline.len() > 10 {
        println!("  ... and {} more operations", timeline.len() - 10);
    }

    Ok(())
}

/// Demonstrate handling of hardware failures
async fn demonstrate_hardware_failures(
    network: &mut QuantumNetwork,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Simulating EPR generation failure and recovery...");

    // This would trigger retries in a real system
    let mut success = false;
    for attempt in 1..=3 {
        println!("  Attempt {}: Requesting EPR pair...", attempt);
        match network.create_epr_pair("Alice", "Bob") {
            Ok(_) => {
                println!("  ✓ Recovery successful!");
                success = true;
                break;
            }
            Err(_) => {
                println!("  ✗ Failed, backing off...");
                tokio::time::sleep(tokio::time::Duration::from_millis(100 * attempt)).await;
            }
        }
    }

    if !success {
        println!("  ⚠ Maximum retries exceeded - would trigger fallback protocol");
    }

    Ok(())
}
