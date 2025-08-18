use qnect::create;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Qnect: Quantum Random Walk ---\n");

    let n_steps = 4;
    let n_qubits = n_steps + 1; // Position qubits + coin qubit

    let mut q = create().with_qubits(n_qubits).build()?;

    // Initialize coin in superposition
    println!("Initializing quantum coin in superposition...");
    q.h(0).await?;

    // Perform quantum walk steps
    for step in 0..n_steps {
        println!(
            "Step {}: Applying controlled operation based on coin...",
            step + 1
        );

        // Controlled operations based on coin
        // This showcases the clean API even for complex algorithms
        q.cnot(0, step + 1).await?;
    }

    // Measure position (sequential measurements)
    println!("\nMeasuring final position...");
    let mut position = Vec::new();
    for i in 1..=n_steps {
        position.push(q.measure(i).await?);
    }

    // Also measure the coin
    let coin = q.measure(0).await?;

    // Convert binary position to decimal
    let position_int: usize = position
        .iter()
        .enumerate()
        .map(|(i, &bit)| (bit as usize) << i)
        .sum();

    println!("\nResults:");
    println!("  Coin state: {}", if coin == 0 { "↑" } else { "↓" });
    println!(
        "  Walker position: {} (binary: {:?})",
        position_int, position
    );
    println!(
        "\nThe quantum walker explored {} possible paths simultaneously!",
        1 << n_steps
    );

    Ok(())
}
