use qnect::create;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Qnect: Simple Bell Pair ---\n");

    // Look how clean this is!
    let mut q = create().with_qubits(2).build()?;

    // Create Bell state
    q.h(0).await?;
    q.cnot(0, 1).await?;

    // Measure
    let m0 = q.measure(0).await?;
    let m1 = q.measure(1).await?;

    println!("Measured: |{}{}⟩", m0, m1);
    println!(
        "Correlation: {}",
        if m0 == m1 {
            "✓ Perfect!"
        } else {
            "✗ Error!"
        }
    );

    Ok(())
}
