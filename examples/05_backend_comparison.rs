use std::time::Instant;

use qnect::{builder::BackendType, create};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Qnect: Backend Comparison ---\n");

    // Test Bell State
    println!("Bell State Algorithm:");

    // StateVector backend
    let start = Instant::now();
    let mut q = create()
        .with_qubits(2)
        .with_backend(BackendType::StateVector)
        .build()?;

    q.h(0).await?;
    q.cnot(0, 1).await?;

    println!("  StateVector: {:?}", start.elapsed());

    // Test GHZ-5
    println!("\nGHZ-5 Algorithm:");

    let start = Instant::now();
    let mut q = create()
        .with_qubits(5)
        .with_backend(BackendType::StateVector)
        .build()?;

    q.h(0).await?;
    for i in 1..5 {
        q.cnot(0, i).await?;
    }

    println!("  StateVector: {:?}", start.elapsed());

    // Future backends would be tested here
    println!("\nFuture backends:");
    println!("  TensorNetwork: (coming soon)");
    println!("  Stabilizer: (coming soon)");
    println!("  Hardware: (coming soon)");

    Ok(())
}
