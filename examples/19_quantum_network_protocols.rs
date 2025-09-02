use qnect::{network::network::QuantumNetwork, quantum::state::Gate1Q};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Qnect: Quantum Network Protocols Demo ===\n");

    // Example 1: Basic Network Setup and EPR Pairs
    println!("Example 1: Creating EPR Pairs Between Nodes");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    basic_epr_demo().await?;

    // Example 2: Classical Communication
    println!("\nExample 2: Classical Communication Between Nodes");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    classical_comm_demo().await?;

    // Example 3: Quantum Teleportation
    println!("\nExample 3: Quantum Teleportation Protocol");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    teleportation_demo().await?;

    // Example 4: Distributed GHZ
    println!("\nExample 4: Distributed GHZ State");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    distributed_ghz_demo().await?;

    Ok(())
}

async fn basic_epr_demo() -> Result<(), Box<dyn std::error::Error>> {
    let mut network = QuantumNetwork::new();

    // Add nodes
    network.add_node("Alice", 2);
    network.add_node("Bob", 2);

    // Create EPR pair
    let (q1, q2) = network.create_epr_pair("Alice", "Bob")?;
    println!("Created EPR pair: Alice qubit {} <-> Bob qubit {}", q1, q2);

    // Verify entanglement
    if network.are_entangled(q1, q2) {
        println!("✓ Qubits are entangled!");
    }

    // Measure both qubits
    let m1 = network.measure("Alice", q1)?;
    let m2 = network.measure("Bob", q2)?;

    println!("Alice measured: {}", m1);
    println!("Bob measured: {}", m2);

    if m1 == m2 {
        println!("✓ Measurements are correlated!");
    }

    Ok(())
}

async fn classical_comm_demo() -> Result<(), Box<dyn std::error::Error>> {
    let mut network = QuantumNetwork::new();
    network.add_node("Alice", 2);
    network.add_node("Bob", 2);

    // Alice prepares a state
    let alice_qubit = network.nodes.get("Alice").unwrap().qubits[0];
    network.apply_local_gate("Alice", alice_qubit, Gate1Q::H)?;

    // Alice measures her qubit
    let alice_result = network.measure("Alice", alice_qubit)?;
    println!("Alice measured: {}", alice_result);

    // Send result to Bob
    let alice_node = network.nodes.get("Alice").unwrap();
    alice_node.send_bit("Bob", alice_result).await?;

    // Bob receives
    let bob_node = network.nodes.get_mut("Bob").unwrap();
    let received = bob_node.recv_bit("Alice").await?;
    println!("Bob received: {}", received);

    Ok(())
}

async fn teleportation_demo() -> Result<(), Box<dyn std::error::Error>> {
    let mut network = QuantumNetwork::new();
    network.add_node("Alice", 2);
    network.add_node("Bob", 1);

    // Alice prepares a state on qubit 0
    network.apply_local_gate("Alice", 0, Gate1Q::H)?;
    println!("Alice prepared |+⟩ state to teleport");

    // Run teleportation protocol
    let bob_qubit = network.quantum_teleportation("Alice", "Bob", 0).await?;

    println!(
        "Teleportation complete! Bob's qubit {} now has the state",
        bob_qubit
    );

    Ok(())
}

async fn distributed_ghz_demo() -> Result<(), Box<dyn std::error::Error>> {
    let mut network = QuantumNetwork::new();
    network.add_node("Alice", 2);
    network.add_node("Bob", 2);
    network.add_node("Charlie", 2);

    let ghz_qubits = network
        .create_distributed_ghz(vec!["Alice", "Bob", "Charlie"])
        .await?;

    println!("Created distributed GHZ state across:");
    println!("  Alice: qubit {}", ghz_qubits[0]);
    println!("  Bob: qubit {}", ghz_qubits[1]);
    println!("  Charlie: qubit {}", ghz_qubits[2]);

    // Measure all qubits
    let m1 = network.measure("Alice", ghz_qubits[0])?;
    let m2 = network.measure("Bob", ghz_qubits[1])?;
    let m3 = network.measure("Charlie", ghz_qubits[2])?;

    println!("\nMeasurements: {} {} {}", m1, m2, m3);

    if m1 == m2 && m2 == m3 {
        println!("✓ GHZ correlations verified!");
    }

    Ok(())
}
