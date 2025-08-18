use qnect::network::network::QuantumNetwork;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Qnect: Quantum Network Demo ---\n");

    let mut network = QuantumNetwork::new();

    // Create a simple network
    println!("1. Setting up quantum network:");
    network.add_node("Alice", 2);
    network.add_node("Bob", 2);
    network.add_node("Charlie", 1);
    println!("   ✓ Added 3 nodes with 5 total qubits");

    // Create EPR pairs
    println!("\n2. Creating entanglement:");
    match network.create_epr_pair("Alice", "Bob") {
        Ok((a, b)) => {
            println!(
                "   ✓ Created EPR pair between Alice (qubit {}) and Bob (qubit {})",
                a, b
            );
            println!("   ✓ Entangled: {}", network.are_entangled(a, b));
        }
        Err(e) => println!("   ✗ Error: {:?}", e),
    }

    // Try to measure Bob's qubit from Alice (should fail)
    println!("\n3. Testing qubit ownership:");
    match network.measure("Alice", 2) {
        // Qubit 2 belongs to Bob
        Ok(_) => println!("   ✗ Security breach! Alice accessed Bob's qubit"),
        Err(_) => println!("   ✓ Security working: Alice cannot measure Bob's qubit"),
    }

    println!("\n📝 This demonstrates the current QuantumNetwork API.");
    println!("   Future versions will integrate with the QuantumSystem backend.");

    Ok(())
}
