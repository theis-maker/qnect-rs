use qnect::create;
use std::f64::consts::PI;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Qnect: Complete Gate Showcase ---\n");
    println!("Demonstrating all available quantum gates in Qnect\n");

    // Single-qubit gates
    println!("🔲 SINGLE-QUBIT GATES:");
    println!("─────────────────────\n");

    // Pauli gates
    println!("1. Pauli Gates (X, Y, Z):");
    let mut q = create().with_qubits(1).build()?;

    println!("   X gate (NOT): |0⟩ → |1⟩");
    q.x(0).await?;
    let m = q.measure(0).await?;
    println!("   After X on |0⟩: measured {}\n", m);

    let mut q = create().with_qubits(1).build()?;
    println!("   Y gate: |0⟩ → i|1⟩");
    q.y(0).await?;
    println!("   (Adds complex phase)\n");

    let mut q = create().with_qubits(1).build()?;
    println!("   Z gate: |1⟩ → -|1⟩");
    q.x(0).await?; // Prepare |1⟩
    q.z(0).await?;
    println!("   (Flips phase of |1⟩)\n");

    // Hadamard
    println!("2. Hadamard Gate (H):");
    let mut q = create().with_qubits(1).build()?;
    q.h(0).await?;
    println!("   Creates superposition: |0⟩ → (|0⟩ + |1⟩)/√2");
    let m = q.measure(0).await?;
    println!("   Measured: {} (50/50 chance)\n", m);

    // Phase gates
    println!("3. Phase Gates (S, T):");
    let mut q = create().with_qubits(1).build()?;
    q.h(0).await?;
    q.s(0).await?;
    println!("   S gate: adds π/2 phase to |1⟩");

    q.t(0).await?;
    println!("   T gate: adds π/4 phase to |1⟩\n");

    // Rotation gates
    println!("4. Rotation Gates (Rx, Ry, Rz):");

    // Rx
    let mut q = create().with_qubits(1).build()?;
    q.rx(0, PI / 2.0).await?;
    println!("   Rx(π/2): rotates around X-axis");
    let m = q.measure(0).await?;
    println!("   Result: {} (creates superposition)\n", m);

    // Ry - most useful for creating arbitrary states
    let mut q = create().with_qubits(1).build()?;
    let angle = 2.0 * (0.3_f64).asin(); // For 30% |0⟩, 70% |1⟩
    q.ry(0, angle).await?;
    println!("   Ry(θ): rotates around Y-axis");
    println!("   Can create any real superposition\n");

    // Rz
    let mut q = create().with_qubits(1).build()?;
    q.h(0).await?;
    q.rz(0, PI / 4.0).await?;
    println!("   Rz(π/4): rotates around Z-axis");
    println!("   Adds relative phase between |0⟩ and |1⟩\n");

    // Two-qubit gates
    println!("🔲🔲 TWO-QUBIT GATES:");
    println!("────────────────────\n");

    // CNOT
    println!("5. CNOT Gate (Controlled-X):");
    let mut q = create().with_qubits(2).build()?;
    q.x(0).await?; // Control = 1
    q.cnot(0, 1).await?;
    let (m0, m1) = (q.measure(0).await?, q.measure(1).await?);
    println!("   CNOT flips target when control=1");
    println!("   Control: {}, Target: {} (both 1)\n", m0, m1);

    // CZ
    println!("6. CZ Gate (Controlled-Z):");
    let mut q = create().with_qubits(2).build()?;
    q.h(0).await?;
    q.h(1).await?;
    q.cz(0, 1).await?;
    println!("   CZ adds phase when both qubits are |1⟩");
    println!("   Creates different entanglement than CNOT\n");

    // SWAP
    println!("7. SWAP Gate:");
    let mut q = create().with_qubits(2).build()?;
    q.x(0).await?; // First qubit = 1
    println!("   Before SWAP: |10⟩");
    q.swap(0, 1).await?;
    let (m0, m1) = (q.measure(0).await?, q.measure(1).await?);
    println!("   After SWAP: |{}{}⟩ (swapped!)\n", m0, m1);

    // CY
    println!("8. CY Gate (Controlled-Y):");
    let mut q = create().with_qubits(2).build()?;
    q.x(0).await?;
    q.cy(0, 1).await?;
    println!("   Like CNOT but with additional phase\n");

    // Composite operations
    println!("🎯 COMPOSITE OPERATIONS:");
    println!("───────────────────────\n");

    println!("9. Bell State Creation:");
    let mut q = create().with_qubits(2).build()?;
    q.create_bell_pair(0, 1).await?;
    let (m0, m1) = (q.measure(0).await?, q.measure(1).await?);
    println!("   |Φ+⟩ = (|00⟩ + |11⟩)/√2");
    println!("   Measured: |{}{}⟩ (always correlated)\n", m0, m1);

    // Gate sequence example
    println!("10. Example Gate Sequence:");
    let mut q = create().with_qubits(2).build()?;
    println!("   Creating a complex state:");
    q.ry(0, PI / 3.0).await?;
    println!("   → Ry(π/3) on qubit 0");
    q.h(1).await?;
    println!("   → H on qubit 1");
    q.cnot(0, 1).await?;
    println!("   → CNOT(0,1)");
    q.s(0).await?;
    println!("   → S on qubit 0");
    q.t(1).await?;
    println!("   → T on qubit 1");

    let (m0, m1) = (q.measure(0).await?, q.measure(1).await?);
    println!("   Final measurement: |{}{}⟩\n", m0, m1);

    println!("✨ That's all the gates in Qnect!");
    println!("   Combine them to build any quantum algorithm!");

    Ok(())
}
