use qnect::{create, error::QnectError};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Qnect: Clean Error Handling ---\n");

    let mut q = create().with_qubits(3).build()?;

    // Good operation
    match q.h(0).await {
        Ok(_) => println!("✓ H gate on qubit 0: Success"),
        Err(e) => println!("✗ Error: {}", e),
    }

    // Out of range error
    match q.cnot(1, 5).await {
        Ok(_) => println!("This shouldn't happen!"),
        Err(QnectError::QubitOutOfRange { qubit, max }) => {
            println!(
                "✓ Caught error: Qubit {} out of range (max: {})",
                qubit, max
            )
        }
        Err(e) => println!("✗ Unexpected error: {}", e),
    }

    // Same qubit error
    match q.cnot(2, 2).await {
        Ok(_) => println!("This shouldn't happen!"),
        Err(QnectError::InvalidGate { reason }) => {
            println!("✓ Caught error: {}", reason)
        }
        Err(e) => println!("✗ Unexpected error: {}", e),
    }

    Ok(())
}
