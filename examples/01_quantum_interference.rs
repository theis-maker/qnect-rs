use qnect::create;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Qnect: Quantum Interference Demo ---\n");
    println!("Demonstrating the heart of quantum computing: interference!\n");

    // Single qubit interference (the simplest quantum algorithm)
    println!("1. Basic Interference Pattern:");
    let mut q = create().with_qubits(1).build()?;

    // Create superposition
    q.h(0).await?;
    println!("   After H: qubit is in |+⟩ = (|0⟩ + |1⟩)/√2");

    // Apply phase
    q.z(0).await?;
    println!("   After Z: qubit is in |−⟩ = (|0⟩ - |1⟩)/√2");

    // Interfere
    q.h(0).await?;
    println!("   After second H: interference causes cancellation");

    let result = q.measure(0).await?;
    println!(
        "   Measured: |{}⟩ (always 1 due to destructive interference!)\n",
        result
    );

    // Two-qubit interference (basis of many algorithms)
    println!("2. Two-Qubit Interference:");
    let mut q = create().with_qubits(2).build()?;

    // Create entanglement
    q.h(0).await?;
    q.cnot(0, 1).await?;
    println!("   Created |Φ+⟩ = (|00⟩ + |11⟩)/√2");

    // Apply phases
    q.z(0).await?;
    println!("   After Z on qubit 0: (|00⟩ - |11⟩)/√2");

    // More operations to show interference
    q.h(0).await?;
    q.h(1).await?;

    let (m0, m1) = (q.measure(0).await?, q.measure(1).await?);
    println!("   Measured: |{}{}⟩", m0, m1);
    println!("   (Results depend on quantum interference patterns)\n");

    // Demonstrate phase kickback (used in many algorithms)
    println!("3. Phase Kickback (Key to Quantum Algorithms):");
    let mut q = create().with_qubits(2).build()?;

    // Prepare control in superposition, target in |1⟩
    q.h(0).await?;
    q.x(1).await?;

    // Controlled-Z gate causes phase kickback
    q.cz(0, 1).await?;
    println!("   CZ gate 'kicks back' phase to control qubit");

    q.h(0).await?;
    let control = q.measure(0).await?;
    println!(
        "   Control measured: |{}⟩ (phase affected measurement)",
        control
    );

    println!("\n💡 Key Insight: Quantum interference is what makes quantum");
    println!("   algorithms powerful. It's not just superposition - it's");
    println!("   the ability to interfere amplitudes constructively and");
    println!("   destructively that enables quantum speedup!");

    Ok(())
}
