use qnect::{
    builder::BackendType,
    network::{
        builder::{NetworkBuilder, Topology},
        network::LinkType,
        node_types::RoutingStrategy,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Quantum Network Topologies Demo\n");

    // Test 1: Star Topology - Quantum Data Center
    test_star_topology().await?;

    // Test 2: Hierarchical - Metropolitan Quantum Network
    test_hierarchical_topology().await?;

    // Test 3: Mesh - Research Testbed
    test_mesh_topology().await?;

    // Test 4: Ring - Quantum Token Ring
    test_ring_topology().await?;

    // Test 5: Line - Quantum Repeater Chain
    test_line_topology().await?;

    Ok(())
}

async fn test_star_topology() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══ STAR TOPOLOGY: Quantum Data Center ═══\n");

    let mut network = NetworkBuilder::new()
        .add_hub_with_strategy("QDC-Hub", 50, RoutingStrategy::HighestFidelity)
        .with_topology(Topology::Star {
            hub_name: "QDC-Hub".to_string(),
            hub_capacity: 50,
        })
        .add_endpoint("Compute-1", 20)
        .add_endpoint("Compute-2", 20)
        .add_endpoint("Storage-1", 10)
        .add_endpoint("Storage-2", 10)
        .with_link_type(LinkType::Fiber {
            length_km: 0.05, // 50 meters within data center
            loss_db_per_km: 0.1,
        })
        .build()?;

    println!("\n📊 Topology Visualization:");
    println!("      Compute-1");
    println!("           \\");
    println!("   Storage-1-QDC-Hub-Storage-2");
    println!("           /");
    println!("      Compute-2\n");

    // Test: Create EPR between compute nodes through hub
    let (q1, q2) = network
        .create_epr_pair_through_hub("Compute-1", "Compute-2", "QDC-Hub")
        .await?;

    println!(
        "✅ Entanglement distributed: Compute-1:q{} <-> Compute-2:q{}",
        q1, q2
    );
    println!("   This happened through teleportation via QDC-Hub!\n");

    Ok(())
}

async fn test_hierarchical_topology() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══ HIERARCHICAL: Metropolitan Quantum Network ═══\n");

    let network = NetworkBuilder::new()
        .add_hub_with_strategy("Metro-Core", 200, RoutingStrategy::ShortestPath)
        .add_hub("District-North", 50)
        .add_hub("District-South", 50)
        .with_topology(Topology::Hierarchical {
            central_hub: "Metro-Core".to_string(),
            regional_hubs: vec!["District-North".to_string(), "District-South".to_string()],
        })
        .add_endpoint("Bank-A", 5)
        .add_endpoint("Hospital-B", 5)
        .add_endpoint("University-C", 8)
        .add_endpoint("Lab-D", 8)
        .with_link_type(LinkType::Fiber {
            length_km: 5.0,
            loss_db_per_km: 0.2,
        })
        .build()?;

    println!("\n📊 Topology Visualization:");
    println!("         Metro-Core");
    println!("          /      \\");
    println!("   District-North  District-South");
    println!("      /    \\          /    \\");
    println!("  Bank-A  Hospital-B  University-C  Lab-D\n");

    // Test: Find path between nodes in different districts
    let path = network.find_shortest_path("Bank-A", "University-C");
    if let Some(p) = path {
        println!("✅ Path found: {}", p.join(" → "));
    }
    println!();

    Ok(())
}

async fn test_mesh_topology() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══ MESH TOPOLOGY: Quantum Research Testbed ═══\n");

    let mut network = NetworkBuilder::new()
        .with_topology(Topology::Mesh {
            link_fidelity: 0.99,
        })
        .add_endpoint_with_backend("Lab-MIT", 10, BackendType::StateVector)
        .add_endpoint_with_backend("Lab-IBM", 10, BackendType::StateVector)
        .add_endpoint_with_backend("Lab-Google", 10, BackendType::StateVector)
        .add_endpoint_with_backend("Lab-QuTech", 10, BackendType::StateVector)
        .with_link_type(LinkType::Satellite {
            orbital_height_km: 500.0,
        })
        .build()?;

    println!("\n📊 Topology Visualization:");
    println!("     Lab-MIT ← → Lab-IBM");
    println!("         ×   ×");
    println!("   Lab-Google ← → Lab-QuTech");
    println!("   (Everyone connected to everyone)\n");

    // Test: Direct EPR between any two nodes
    let (q1, q2) = network.create_epr_pair("Lab-MIT", "Lab-QuTech")?;
    println!("✅ Direct EPR pair: Lab-MIT:q{} <-> Lab-QuTech:q{}", q1, q2);
    println!("   No hub needed in mesh topology!\n");

    Ok(())
}

async fn test_ring_topology() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══ RING TOPOLOGY: Quantum Token Ring ═══\n");

    let network = NetworkBuilder::new()
        .with_topology(Topology::Ring)
        .add_endpoint("Node-1", 4)
        .add_endpoint("Node-2", 4)
        .add_endpoint("Node-3", 4)
        .add_endpoint("Node-4", 4)
        .with_link_type(LinkType::Fiber {
            length_km: 2.0,
            loss_db_per_km: 0.15,
        })
        .with_fidelity(0.97)
        .build()?;

    println!("\n📊 Topology Visualization:");
    println!("     Node-1 — Node-2");
    println!("       |        |");
    println!("     Node-4 — Node-3\n");

    // Test: Find path around ring
    let path = network.find_shortest_path("Node-1", "Node-3");
    if let Some(p) = path {
        println!("✅ Shortest path: {}", p.join(" → "));
    }
    println!();

    Ok(())
}

async fn test_line_topology() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══ LINE TOPOLOGY: Quantum Repeater Chain ═══\n");

    let mut network = NetworkBuilder::new()
        .with_topology(Topology::Line)
        .add_endpoint("Alice", 5)
        .add_endpoint("Repeater-1", 4)
        .add_endpoint("Repeater-2", 4)
        .add_endpoint("Repeater-3", 4)
        .add_endpoint("Bob", 5)
        .with_link_type(LinkType::Fiber {
            length_km: 50.0, // Long distance
            loss_db_per_km: 0.2,
        })
        .with_fidelity(0.92)
        .build()?;

    println!("\n📊 Topology Visualization:");
    println!("  Alice — Rep1 — Rep2 — Rep3 — Bob");
    println!("  <--50km--><--50km--><--50km--><--50km-->\n");

    // Test: End-to-end entanglement through repeaters
    let (q1, q2) = network
        .establish_end_to_end_entanglement("Alice", "Bob")
        .await?;
    println!("✅ End-to-end entanglement: Alice:q{} <-> Bob:q{}", q1, q2);
    println!("   Established through 3 repeater swaps!");

    // Calculate total distance and fidelity
    let path = network.find_shortest_path("Alice", "Bob").unwrap();
    let fidelity = network.calculate_path_fidelity(&path)?;
    println!(
        "   Total distance: 200km, Final fidelity: {:.3}\n",
        fidelity
    );

    Ok(())
}
