use qnect::create;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Qnect: GHZ State Scaling Demo ---\n");

    // Here we can see how it scales from 3 to N qubits with same code
    for n in [3, 5, 10, 15, 20] {
        let start = Instant::now();

        // Sample measurements
        let mut all_zeros = 0;
        let mut all_ones = 0;

        for _ in 0..20 {
            let mut q = create().with_qubits(n).build()?;

            // Create GHZ state - same code regardless of size!
            q.h(0).await?;
            for i in 1..n {
                q.cnot(0, i).await?;
            }

            // Measure all qubits sequentially
            let mut measurements = Vec::new();
            for i in 0..n {
                measurements.push(q.measure(i).await?);
            }

            if measurements.iter().all(|&m| m == 0) {
                all_zeros += 1;
            } else if measurements.iter().all(|&m| m == 1) {
                all_ones += 1;
            }
        }

        println!(
            "{}-qubit GHZ: |0...0⟩: {}%, |1...1⟩: {}% ({}ms)",
            n,
            all_zeros,
            all_ones,
            start.elapsed().as_millis()
        );
    }

    Ok(())
}
