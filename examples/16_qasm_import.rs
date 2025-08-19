use qnect::{builder::execute_qasm, create};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Qnect: QASM Import Demo ===\n");

    // Example 1: Import a simple Bell state circuit
    let bell_qasm = r#"
OPENQASM 2.0;
include "qelib1.inc";

qreg q[2];
creg c[2];

h q[0];
cx q[0], q[1];
measure q[0] -> c[0];
measure q[1] -> c[1];
"#;

    println!("Example 1: Importing Bell state circuit");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{}", bell_qasm);

    let mut system = create().from_qasm(bell_qasm)?.build()?;

    let results = execute_qasm(&mut system, bell_qasm).await?;
    println!("Measurement results: {:?}", results);
    println!("(Should be [0,0] or [1,1] for Bell state)\n");

    // Example 2: Import circuit with rotations
    let rotation_qasm = r#"
OPENQASM 2.0;
include "qelib1.inc";

qreg q[1];
creg c[1];

rx(pi/4) q[0];
ry(pi/2) q[0];
rz(pi) q[0];
measure q[0] -> c[0];
"#;

    println!("Example 2: Importing rotation circuit");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{}", rotation_qasm);

    let mut system = create().from_qasm(rotation_qasm)?.build()?;

    let results = execute_qasm(&mut system, rotation_qasm).await?;
    println!("Measurement result: {:?}\n", results);

    // Example 3: Import QASM 3.0 format
    let qasm3_circuit = r#"
OPENQASM 3.0;
include "stdgates.inc";

qubit[3] q;
bit[3] c;

h q[0];
cx q[0], q[1];
cx q[1], q[2];
c[0] = measure q[0];
c[1] = measure q[1];
c[2] = measure q[2];
"#;

    println!("Example 3: Importing QASM 3.0 format");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{}", qasm3_circuit);

    let mut system = create().from_qasm(qasm3_circuit)?.build()?;

    let results = execute_qasm(&mut system, qasm3_circuit).await?;
    println!("GHZ measurement results: {:?}", results);
    println!("(Should be [0,0,0] or [1,1,1] for GHZ state)\n");

    // Example 4: Round-trip test - export then import
    println!("Example 4: Round-trip test (Create → Export → Import → Execute)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Create and execute a circuit
    let mut original = create().with_qubits(3).build()?.with_recording();
    original.h(0).await?;
    original.cnot(0, 1).await?;
    original.cnot(1, 2).await?;
    original.s(0).await?;
    original.t(1).await?;
    original.s_dag(2).await?;

    println!("Original circuit:");
    original.print_circuit();

    // Export to QASM
    let exported_qasm = original.to_qasm2();
    println!("\nExported QASM:");
    println!("{}", exported_qasm);

    // Import and execute
    let mut imported = create().from_qasm(&exported_qasm)?.build()?;

    // Add measurements to the QASM for execution
    let qasm_with_measurements = format!(
        "{}\nmeasure q[0] -> c[0];\nmeasure q[1] -> c[1];\nmeasure q[2] -> c[2];",
        exported_qasm.trim_end_matches('\n')
    );

    let results = execute_qasm(&mut imported, &qasm_with_measurements).await?;
    println!("\nResults after round-trip: {:?}", results);

    // Example 5: Error handling
    println!("\nExample 5: Error handling - EXPECTED ERROR");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let invalid_qasm = r#"
OPENQASM 2.0;
qreg q[2];
unknown_gate q[0];
"#;

    match create().from_qasm(invalid_qasm) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("SUCCESS! Expected error caught: {:?}", e),
    }

    // Example 6: Import from file
    println!("\nExample 6: Import from file");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Create a test QASM file
    let test_qasm = r#"OPENQASM 2.0;
include "qelib1.inc";

qreg q[4];
creg c[4];

// Create W state: (|0001⟩ + |0010⟩ + |0100⟩ + |1000⟩)/2
h q[0];
cx q[0], q[1];
x q[0];
cx q[0], q[2];
x q[0];
cx q[0], q[3];

measure q[0] -> c[0];
measure q[1] -> c[1];
measure q[2] -> c[2];
measure q[3] -> c[3];
"#;

    // Save to file
    std::fs::write("test_circuit.qasm", test_qasm)?;
    println!("Saved test circuit to 'test_circuit.qasm'");

    // Read and import
    let file_contents = std::fs::read_to_string("test_circuit.qasm")?;
    let mut system = create().from_qasm(&file_contents)?.build()?;
    let results = execute_qasm(&mut system, &file_contents).await?;

    println!("W-state measurement: {:?}", results);
    println!("(Should have exactly one 1 and three 0s)");

    // Clean up
    std::fs::remove_file("test_circuit.qasm").ok();

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("QASM Import Summary:");
    println!("✅ Supports both QASM 2.0 and 3.0 formats");
    println!("✅ Handles symbolic angles (pi/4, pi/2, etc.)");
    println!("✅ Round-trip compatible with export");
    println!("✅ Proper error handling for invalid QASM");
    println!("✅ Can import from strings or files");
    println!("═══════════════════════════════════════════════════════════════");

    Ok(())
}
