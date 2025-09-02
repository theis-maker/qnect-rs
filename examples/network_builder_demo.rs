use qnect::network::{
    builder::{NetworkBuilder, Topology},
    network::LinkType,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏗️ Network Builder Demo\n");
    println!("Building quantum networks with fluent API...\n");

    // Example 1: Star topology data center
    println!("Example 1: Quantum Data Center (Star Topology)");

    let mut datacenter = NetworkBuilder::new()
        .with_topology(Topology::Star {
            hub_name: "DataCenter-Hub".to_string(),
            hub_capacity: 100,
        })
        .add_hub("DataCenter-Hub", 100)
        .add_endpoint("Server-1", 10)
        .add_endpoint("Server-2", 10)
        .add_endpoint("Server-3", 10)
        .with_link_type(LinkType::Fiber {
            length_km: 0.1,
            loss_db_per_km: 0.1,
        })
        .build()?;

    println!("\nData center built! Testing entanglement...");
    let (q1, q2) = datacenter
        .create_epr_pair_through_hub("Server-1", "Server-2", "DataCenter-Hub")
        .await?;
    println!(
        "✅ EPR pair created: Server-1:q{} <-> Server-2:q{}\n",
        q1, q2
    );

    // Example 2: Ring network
    println!("\nExample 2: Ring Network");

    let _ring = NetworkBuilder::new()
        .with_topology(Topology::Ring)
        .add_endpoint("Node-A", 5)
        .add_endpoint("Node-B", 5)
        .add_endpoint("Node-C", 5)
        .add_endpoint("Node-D", 5)
        .with_fidelity(0.95)
        .build()?;

    println!("\nRing network built!\n");

    // Example 3: Mesh network for anonymous protocols
    println!("Example 3: Full Mesh for Anonymous Transmission");

    let _mesh = NetworkBuilder::new()
        .with_topology(Topology::Mesh {
            link_fidelity: 0.99,
        })
        .add_endpoint("Alice", 8)
        .add_endpoint("Bob", 8)
        .add_endpoint("Charlie", 8)
        .build()?;

    println!("\nMesh network built! Perfect for anonymous protocols.\n");

    println!("🎉 NetworkBuilder makes complex topologies simple!");

    Ok(())
}
