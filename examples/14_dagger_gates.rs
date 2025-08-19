use qnect::create;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Qnect: Dagger Gates Demo ===\n");

    // Test 1: S and S† should cancel
    println!("Test 1: S and S† cancellation");
    let mut q = create().with_qubits(1).build()?;

    // Prepare |+⟩ state
    q.h(0).await?;

    // Apply S then S†
    q.s(0).await?;
    q.s_dag(0).await?;

    // Should be back to |+⟩
    q.h(0).await?;
    let result = q.measure(0).await?;
    println!("After S·S†: measured |{}⟩ (should always be |0⟩)", result);

    // Test 2: T and T† should cancel
    println!("\nTest 2: T and T† cancellation");
    let mut q = create().with_qubits(1).build()?;

    // Prepare |+⟩ state
    q.h(0).await?;

    // Apply T then T†
    q.t(0).await?;
    q.t_dag(0).await?;

    // Should be back to |+⟩
    q.h(0).await?;
    let result = q.measure(0).await?;
    println!("After T·T†: measured |{}⟩ (should always be |0⟩)", result);

    // Test 3: Visualize in circuit
    println!("\nTest 3: Circuit visualization");
    let mut q = create().with_qubits(2).build()?.with_recording();

    q.h(0).await?;
    q.s(0).await?;
    q.s_dag(0).await?;
    q.t(1).await?;
    q.t_dag(1).await?;
    q.cnot(0, 1).await?;

    q.print_circuit();

    // Test 4: Phase relationships
    println!("\nTest 4: Phase relationships");
    println!("S²  = Z  (applying S twice gives Z)");
    println!("S†² = Z  (applying S† twice gives Z)");
    println!("T⁴  = Z  (applying T four times gives Z)");
    println!("T†⁴ = Z  (applying T† four times gives Z)");

    Ok(())
}
