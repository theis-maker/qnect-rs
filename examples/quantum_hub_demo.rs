use qnect::{
    builder::BackendType,
    network::{
        network::{LinkType, QuantumNetwork},
        node_types::RoutingStrategy,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌟 Quantum Hub Network Demo\n");
    println!("Building a star topology with central hub...\n");

    // Create network
    let mut network = QuantumNetwork::new_distributed();

    // Add central hub
    network.add_hub_with_config(
        "Central-Hub",
        (0.0, 0.0),
        50, // capacity
        RoutingStrategy::ShortestPath,
    )?;

    // Add endpoints
    println!("Adding endpoints...");
    network.add_distributed_node("Alice", 10, BackendType::Stabilizer)?;
    network.add_distributed_node("Bob", 10, BackendType::Stabilizer)?;
    network.add_distributed_node("Charlie", 10, BackendType::Stabilizer)?;
    network.add_distributed_node("Diana", 10, BackendType::Stabilizer)?;

    // Connect everyone to the hub
    println!("\nConnecting nodes to hub...");
    network.connect_to_hub("Alice", "Central-Hub", LinkType::Fiber {
        length_km: 5.0,
        loss_db_per_km: 0.85,
    })?;
    network.connect_to_hub("Bob", "Central-Hub", LinkType::Fiber {
        length_km: 7.0,
        loss_db_per_km: 0.85,
    })?;
    network.connect_to_hub("Charlie", "Central-Hub", LinkType::Fiber {
        length_km: 4.0,
        loss_db_per_km: 0.85,
    })?;
    network.connect_to_hub("Diana", "Central-Hub", LinkType::Fiber {
        length_km: 6.0,
        loss_db_per_km: 0.85,
    })?;

    // Show network topology
    println!("\n📊 Network Topology:");
    println!("         Alice");
    println!("           |");
    println!("    Bob -- Hub -- Charlie");
    println!("           |");
    println!("         Diana");

    // Test routing through hub
    println!("\n🔄 Testing quantum routing through hub...\n");

    // Alice wants to send to Bob (through hub)
    let path = network
        .route_through_hub("Alice", "Bob", "Central-Hub")
        .await?;
    println!("Route found: {}", path.join(" → "));

    // Create EPR pair through hub
    println!("\n🔗 Creating EPR pairs through hub...");
    let (q1, q2) = network
        .create_epr_pair_through_hub("Alice", "Bob", "Central-Hub")
        .await?;

    println!("EPR pair created: Alice:q{} <--> Bob:q{}", q1, q2);

    // Show that everyone can communicate through the hub
    println!("\n✅ All nodes can communicate through the central hub!");
    println!("This topology is perfect for:");
    println!("  • Metropolitan quantum networks");
    println!("  • Quantum data centers");
    println!("  • Corporate quantum networks");

    Ok(())
}
