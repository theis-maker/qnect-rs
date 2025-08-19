use qnect::create;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Qnect: OpenQASM Export Demo ===\n");

    // Create a circuit with recording
    let mut q = create().with_qubits(3).build()?.with_recording();

    // Build a quantum circuit
    println!("Building quantum circuit...\n");

    // Bell pair on qubits 0,1
    q.h(0).await?;
    q.cnot(0, 1).await?;

    // GHZ extension to qubit 2
    q.cnot(1, 2).await?;

    // Some single qubit gates
    q.s(0).await?;
    q.t(1).await?;

    // Rotation gates
    q.rx(2, std::f64::consts::PI / 4.0).await?;
    q.ry(0, std::f64::consts::PI / 2.0).await?;

    // Measure all
    q.measure(0).await?;
    q.measure(1).await?;
    q.measure(2).await?;

    // Export to QASM
    println!("Circuit visualization:");
    q.print_circuit();

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("OpenQASM 3.0 Export:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{}", q.to_qasm());

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("OpenQASM 2.0 Export (for older tools):");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{}", q.to_qasm2());

    // Save to file example
    std::fs::write("circuit.qasm", q.to_qasm())?;
    println!("Circuit saved to circuit.qasm");

    Ok(())
}
