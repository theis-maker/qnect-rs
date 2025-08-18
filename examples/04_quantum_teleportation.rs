use qnect::{backend::backend::QuantumBackend, create, system::QuantumSystem};

async fn teleport_qubit(
    q: &mut QuantumSystem<Box<dyn QuantumBackend>>,
    state_qubit: usize,
    alice_qubit: usize,
    bob_qubit: usize,
) -> Result<(u8, u8), Box<dyn std::error::Error>> {
    // Create entanglement between Alice and Bob
    q.create_bell_pair(alice_qubit, bob_qubit).await?;

    // Alice performs Bell measurement
    q.cnot(state_qubit, alice_qubit).await?;
    q.h(state_qubit).await?;

    let m1 = q.measure(state_qubit).await?;
    let m2 = q.measure(alice_qubit).await?;

    // Bob applies corrections
    if m2 == 1 {
        q.x(bob_qubit).await?;
    }
    if m1 == 1 {
        q.z(bob_qubit).await?;
    }

    Ok((m1, m2))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Qnect: Quantum Teleportation ---\n");

    let mut q = create().with_qubits(3).build()?;

    // Prepare an interesting state to teleport
    println!("Preparing state |+⟩ to teleport...");
    q.h(0).await?;

    // Teleport from qubit 0 to qubit 2
    let (m1, m2) = teleport_qubit(&mut q, 0, 1, 2).await?;

    println!("Alice measured: {}{}", m1, m2);
    println!("Bob's qubit now contains the teleported state!");

    Ok(())
}
