use std::f64::consts::PI;

use qnect::builder::BackendType;
use qnect::network::network::{BlindComputationPattern, LinkType, QuantumNetwork};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut network = QuantumNetwork::new_distributed();

    // Create quantum internet topology
    network.add_distributed_node("ClientA", 8, BackendType::StateVector)?;
    network.add_distributed_node("Router1", 12, BackendType::StateVector)?;
    network.add_distributed_node("Router2", 12, BackendType::StateVector)?;
    network.add_distributed_node("ServerB", 16, BackendType::StateVector)?;

    // Physical links
    network.add_quantum_link(
        "ClientA",
        "Router1",
        LinkType::Fiber {
            length_km: 50.0,
            loss_db_per_km: 0.2,
        },
        0.95,
        1000.0,
    )?;
    network.add_quantum_link(
        "Router1",
        "Router2",
        LinkType::Fiber {
            length_km: 100.0,
            loss_db_per_km: 0.2,
        },
        0.90,
        800.0,
    )?;
    network.add_quantum_link(
        "Router2",
        "ServerB",
        LinkType::Fiber {
            length_km: 50.0,
            loss_db_per_km: 0.2,
        },
        0.95,
        1000.0,
    )?;

    println!("{}", network.visualize_network());

    // Layer 1: Direct entanglement
    let (q1, q2) = network.create_epr_pair("ClientA", "Router1")?;
    println!(
        "Created EPR pair between ClientA (q{}) and Router1 (q{})",
        q1, q2
    );

    // Layer 2: Multi-hop entanglement
    let (end1, end2) = network
        .establish_end_to_end_entanglement("ClientA", "ServerB")
        .await?;
    println!(
        "Established end-to-end entanglement: ClientA (q{}) <-> ServerB (q{})",
        end1, end2
    );

    // Layer 3: Distributed GHZ
    let ghz_qubits = network
        .create_distributed_ghz(vec!["ClientA", "Router1", "ServerB"])
        .await?;
    println!(
        "Created distributed GHZ state across network: {:?}",
        ghz_qubits
    );

    // Layer 4: Universal Blind Quantum Computation
    let pattern = BlindComputationPattern {
        // Simple 2-qubit cluster state for demo
        computation_graph: vec![(0, 1)],
        measurement_angles: vec![PI / 4.0, PI / 3.0],
        flow: vec![0, 1], // Measurement order
    };

    let results = network
        .blind_computation_ubqc("ClientA", "ServerB", pattern)
        .await?;
    println!("Blind computation results: {:?}", results);

    // Generate NetQASM
    let programs = network.generate_netqasm();
    for (node, program) in programs {
        println!("\n=== NetQASM for {} ===\n{}", node, program);
    }

    Ok(())
}
