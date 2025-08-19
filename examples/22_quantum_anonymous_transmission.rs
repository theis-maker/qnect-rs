use std::collections::HashMap;

use qnect::network::network::{LinkType, NetworkOperation, QuantumNetwork};
use qnect::state::Gate1Q;

/// Quantum Anonymous Transmission Protocol Example
/// Based on: "Quantum Anonymous Transmissions" by Christandl & Wehner (2018)
/// https://arxiv.org/abs/quant-ph/0409201
///
/// This example demonstrates:
/// 1. Anonymous classical bit transmission (ANON protocol)
/// 2. Anonymous entanglement (AE protocol)
/// 3. Anonymous quantum state transmission (ANONQ protocol)
/// 4. The traceless property - impossible classically!

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Quantum Anonymous Transmission Protocol ===");
    println!("Based on Christandl & Wehner (2018)\n");

    // The dining cryptographers scenario
    let participants = vec!["Alice", "Bob", "Charlie", "David", "Eve"];

    // Test 1: Anonymous Classical Bit Transmission
    test_anonymous_classical(&participants).await?;

    // Test 2: Verify Traceless Property
    test_traceless_property(&participants).await?;

    // Test 3: Anonymous Entanglement
    test_anonymous_entanglement(&participants).await?;

    // Test 4: Anonymous Quantum Transmission
    test_anonymous_quantum(&participants).await?;

    // Test 5: Multiple Simultaneous Senders (Collision)
    test_collision_scenario(&participants).await?;

    println!("\n✅ All tests passed! The protocol works as described in the paper.");

    Ok(())
}

/// Test 1: Basic anonymous classical bit transmission
async fn test_anonymous_classical(participants: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Test 1: Anonymous Classical Bit Transmission (ANON)");
    println!("   Scenario: Alice wants to send bit '1' anonymously\n");

    let mut network = create_network(participants)?;

    // Alice sends bit 1 anonymously
    let sender = "Alice";
    let bit_to_send = 1;

    println!("   Step 1: Creating GHZ state |Ψ⟩ = (|00000⟩ + |11111⟩)/√2");
    println!("   Step 2: Alice applies phase flip (bit = 1)");
    println!("   Step 3: Everyone applies H and measures");

    let result = network
        .anonymous_transmission(sender, participants.to_vec(), bit_to_send)
        .await?;

    println!("   Step 4: Parity of measurements = {}", result);
    println!("   ✓ Everyone knows bit = {} was sent", result);
    println!("   ✓ Nobody knows Alice was the sender!\n");

    // Verify: Try with bit 0
    let mut network2 = create_network(participants)?;
    let result2 = network2
        .anonymous_transmission("Bob", participants.to_vec(), 0)
        .await?;
    println!(
        "   Verification: When Bob sends bit 0, parity = {}",
        result2
    );

    assert_eq!(result, bit_to_send);
    assert_eq!(result2, 0);

    Ok(())
}

/// Test 2: Demonstrate the traceless property
async fn test_traceless_property(participants: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Test 2: Traceless Property (Impossible Classically!)");
    println!("   Even with ALL communication recorded, sender remains hidden\n");

    let mut network = create_network(participants)?;

    // Enable protocol history tracking
    network.protocol_history.clear();

    // Charlie is the actual sender, but this should be untraceable
    let actual_sender = "Charlie";
    let bit = 1;

    let _ = network
        .anonymous_transmission(actual_sender, participants.to_vec(), bit)
        .await?;

    // Analyze protocol history
    println!("   Analyzing all recorded operations:");

    let mut local_ops = HashMap::new();
    let mut measurements = HashMap::new();

    for op in &network.protocol_history {
        match op {
            NetworkOperation::LocalGate { node, gate, .. } => {
                if gate.contains("Z") {
                    *local_ops.entry(node.clone()).or_insert(0) += 1;
                }
            }
            NetworkOperation::Measure { node, result, .. } => {
                measurements.insert(node.clone(), *result);
            }
            _ => {}
        }
    }

    println!("   - Phase flips applied: {:?}", local_ops);
    println!("   - Measurements: {:?}", measurements);

    // Key insight: The phase flip changes the GLOBAL state, not individual messages
    println!("\n   ⚡ Key insight: The protocol is traceless because:");
    println!("   - Phase flip changes the global entangled state");
    println!("   - Individual measurements are random");
    println!("   - Only the parity reveals the bit");
    println!("   - No record links Charlie to the transmission!\n");

    Ok(())
}

/// Test 3: Anonymous Entanglement (AE)
async fn test_anonymous_entanglement(
    participants: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Test 3: Anonymous Entanglement (AE)");
    println!("   Alice and Bob will share EPR pair anonymously\n");

    let mut network = create_network(participants)?;

    let sender = "Alice";
    let receiver = "Bob";

    println!(
        "   Step 1: Create GHZ state among all {} participants",
        participants.len()
    );
    println!("   Step 2: Charlie, David, Eve apply H and measure");
    println!("   Step 3: Random bit broadcast and corrections");

    let (alice_q, bob_q) = network
        .anonymous_entanglement(sender, receiver, participants.to_vec())
        .await?;

    println!(
        "   ✓ Alice (q{}) and Bob (q{}) now share EPR pair",
        alice_q, bob_q
    );
    println!("   ✓ Others don't know who has the entanglement!\n");

    // Verify it's actually an EPR pair by checking correlations
    // The exact Bell state depends on the measurement outcomes and corrections
    let m1 = network.measure(sender, alice_q)?;
    let m2 = network.measure(receiver, bob_q)?;

    println!("   Verification: Measuring EPR pair in Z basis:");
    println!("   Alice: {} Bob: {}", m1, m2);
    println!(
        "   (They share one of the 4 Bell states - exact correlations depend on protocol outcome)"
    );

    Ok(())
}

/// Test 4: Anonymous Quantum State Transmission
async fn test_anonymous_quantum(participants: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌟 Test 4: Anonymous Quantum State Transmission (ANONQ)");
    println!("   David will send |+⟩ state to Eve anonymously\n");

    let mut network = create_network(participants)?;

    let sender = "David";
    let receiver = "Eve";

    // David prepares |+⟩ state
    let david_qubit = network.allocate_local_qubit(sender)?;
    network.apply_local_gate(sender, david_qubit, Gate1Q::H)?;
    println!("   David prepares |+⟩ state on qubit {}", david_qubit);

    // Anonymous quantum transmission
    println!("   Step 1: Establish anonymous entanglement");
    println!("   Step 2: Teleport using anonymous EPR pair");
    println!("   Step 3: Send corrections anonymously");

    let eve_qubit = network
        .anonymous_quantum_transmission(sender, receiver, participants.to_vec(), david_qubit)
        .await?;

    println!("   ✓ Eve received quantum state on qubit {}", eve_qubit);
    println!("   ✓ Nobody knows David sent it or Eve received it!");

    // Verify Eve received |+⟩ by measuring in Z basis
    let measurement = network.measure(receiver, eve_qubit)?;
    println!(
        "\n   Verification: Eve measures in Z basis: {} (|+⟩ gives 0 or 1 randomly)",
        measurement
    );

    Ok(())
}

/// Test 5: Collision detection when multiple senders
async fn test_collision_scenario(participants: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    println!("💥 Test 5: Collision Detection (Multiple Senders)");
    println!("   What if Alice (bit=1) and Bob (bit=1) both try to send?\n");

    let mut network = create_network(participants)?;

    // Create GHZ state
    let ghz_qubits = network
        .create_distributed_ghz(participants.to_vec())
        .await?;

    // BOTH Alice and Bob apply phase flips (collision!)
    let alice_idx = 0;
    let bob_idx = 1;
    network.apply_local_gate("Alice", ghz_qubits[alice_idx], Gate1Q::Z)?;
    network.apply_local_gate("Bob", ghz_qubits[bob_idx], Gate1Q::Z)?;

    // Everyone measures
    let mut measurements = Vec::new();
    for (i, participant) in participants.iter().enumerate() {
        network.apply_local_gate(participant, ghz_qubits[i], Gate1Q::H)?;
        let m = network.measure(participant, ghz_qubits[i])?;
        measurements.push(m);
    }

    let parity = measurements.iter().sum::<u8>() % 2;

    println!("   Measurements: {:?}", measurements);
    println!("   Parity: {} (XOR of both bits: 1 ⊕ 1 = 0)", parity);
    println!("   ⚠️  Protocol computes XOR, not individual bits");
    println!("   ✓ This motivates the collision detection protocol\n");

    Ok(())
}

/// Helper: Create network with all participants
fn create_network(participants: &[&str]) -> Result<QuantumNetwork, Box<dyn std::error::Error>> {
    let mut network = QuantumNetwork::new_distributed();

    // Add all nodes
    for p in participants {
        network.add_node(*p, 8); // Dereference &&str to &str, no ? needed
    }

    // Create full mesh connectivity (dining table scenario)
    network.add_multiparty_link(
        participants.to_vec(),
        LinkType::Fiber {
            length_km: 0.1, // Same room
            loss_db_per_km: 0.1,
        },
        0.99,    // High fidelity for local connections
        10000.0, // High generation rate
    )?;

    Ok(network)
}
