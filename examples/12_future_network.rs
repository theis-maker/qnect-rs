use qnect::{backend::network_backend::NetworkTopology, builder::BackendType, create};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Qnect: Distributed Quantum Computing (Future) ---\n");

    println!("This demonstrates how quantum networks will work once implemented.\n");

    // Create a quantum network with specific topology
    let network = create()
        .with_backend(BackendType::Network)
        .with_topology(NetworkTopology::Mesh {
            connections: vec![
                ("Alice".to_string(), vec![
                    "Bob".to_string(),
                    "Charlie".to_string(),
                ]),
                ("Bob".to_string(), vec![
                    "Alice".to_string(),
                    "Charlie".to_string(),
                ]),
                ("Charlie".to_string(), vec![
                    "Alice".to_string(),
                    "Bob".to_string(),
                ]),
            ]
            .into_iter()
            .collect(),
        })
        .with_qubits(6) // 2 per node
        .build();

    match network {
        Ok(_) => {
            println!("✅ Network backend created successfully!");
            println!("\nWhen fully implemented, you'll be able to:");
            println!("  - Distribute qubits across physical locations");
            println!("  - Generate entanglement between remote nodes");
            println!("  - Run distributed quantum algorithms");
            println!("  - Handle network latency and errors automatically");

            println!("\nExample of future API:");
            println!("  // Alice prepares a state");
            println!("  network.h(0).await?;  // Alice's qubit 0");
            println!();
            println!("  // Create entanglement between Alice and Bob");
            println!("  network.create_remote_entanglement(\"Alice\", \"Bob\").await?;");
            println!();
            println!("  // Run distributed algorithm");
            println!("  network.distributed_phase_estimation(...).await?;");

            println!("\nThe same Qnect API will work for:");
            println!("  • Local simulation (today)");
            println!("  • Distributed simulation (coming soon)");
            println!("  • Real quantum networks (future)");
        }
        Err(e) => {
            println!("Network backend not yet implemented.");
            println!("Error: {}", e);
            println!("\nFor now, use StateVector backend for local simulation.");
        }
    }

    println!("\n🌐 The future of quantum computing is distributed!");
    println!("   Imagine quantum computers in different cities working together,");
    println!("   sharing entanglement through quantum channels, solving problems");
    println!("   too large for any single quantum computer.");

    Ok(())
}
