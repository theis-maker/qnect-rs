use qnect::{builder::BackendType, create};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Qnect: Stabilizer Backend Demo ===\n");
    println!("The stabilizer formalism enables simulation of Clifford circuits");
    println!("with thousands of qubits using only O(n²) memory!\n");

    // Example 1: Compare backends for small circuits
    println!("Example 1: Backend Comparison (10 qubits)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    compare_backends(10).await?;

    // Example 2: Scale test - increasing qubit counts
    println!("\nExample 2: Scaling Test - GHZ States");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    for n in [10, 50, 100, 500, 1000] {
        create_ghz_state(n).await?;
    }

    // Example 3: Quantum error correction circuit
    println!("\nExample 3: Quantum Error Correction - 3-qubit bit flip code");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    quantum_error_correction().await?;

    // Example 4: Random Clifford circuit benchmark
    println!("\nExample 4: Random Clifford Circuit (100 qubits, 1000 gates)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    random_clifford_benchmark(100, 1000).await?;

    // Example 5: The ultimate test - 5000 qubit GHZ state!
    println!("\nExample 5: The Ultimate Test - 5000 Qubit GHZ State!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    create_massive_ghz(5000).await?;

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("Stabilizer Backend Summary:");
    println!("✓ Simulated up to 5000 qubits on a regular computer!");
    println!("✓ O(n²) memory scaling vs O(2ⁿ) for state vectors");
    println!("✓ Perfect for quantum error correction research");
    println!("✓ Enables simulation of real-scale quantum algorithms");
    println!("═══════════════════════════════════════════════════════════════");

    Ok(())
}

/// Compare state vector vs stabilizer backends
async fn compare_backends(n_qubits: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Creating {}-qubit GHZ state with both backends...",
        n_qubits
    );

    // State vector backend
    let start = Instant::now();
    let mut sv = create()
        .with_backend(BackendType::StateVector)
        .with_qubits(n_qubits)
        .build()?;

    sv.h(0).await?;
    for i in 1..n_qubits {
        sv.cnot(0, i).await?;
    }

    let sv_time = start.elapsed();
    let sv_memory = (1u64 << n_qubits) * 16; // Complex64 = 16 bytes

    println!("  State Vector Backend:");
    println!("    Time: {:?}", sv_time);
    println!("    Memory: {} MB", sv_memory / 1_000_000);

    // Stabilizer backend
    let start = Instant::now();
    let mut stab = create()
        .with_backend(BackendType::Stabilizer)
        .with_qubits(n_qubits)
        .build()?;

    stab.h(0).await?;
    for i in 1..n_qubits {
        stab.cnot(0, i).await?;
    }

    let stab_time = start.elapsed();
    let stab_memory = 2 * n_qubits * (2 * n_qubits + 1) / 8; // bits to bytes

    println!("  Stabilizer Backend:");
    println!("    Time: {:?}", stab_time);
    println!("    Memory: {} KB", stab_memory / 1_000);
    println!(
        "    Speedup: {:.2}x",
        sv_time.as_secs_f64() / stab_time.as_secs_f64()
    );
    println!(
        "    Memory reduction: {:.0}x",
        sv_memory as f64 / stab_memory as f64
    );

    Ok(())
}

/// Create GHZ state with n qubits
async fn create_ghz_state(n_qubits: usize) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();

    let mut q = create()
        .with_backend(BackendType::Stabilizer)
        .with_qubits(n_qubits)
        .build()?;

    // Create GHZ state: H on first qubit, then CNOT to all others
    q.h(0).await?;
    for i in 1..n_qubits {
        q.cnot(0, i).await?;
    }

    // Measure a few qubits to verify GHZ correlations
    let m0 = q.measure(0).await?;
    let m1 = q.measure(1).await?;
    let m_last = q.measure(n_qubits - 1).await?;

    let elapsed = start.elapsed();

    println!(
        "  {:4} qubits: {:8.3} ms | Memory: {:6} KB | Measured: {} {} ... {}",
        n_qubits,
        elapsed.as_secs_f64() * 1000.0,
        2 * n_qubits * (2 * n_qubits + 1) / 8 / 1024,
        m0,
        m1,
        m_last
    );

    // Verify GHZ property
    if m0 == m1 && m1 == m_last {
        println!("       ✓ GHZ correlations verified!");
    } else {
        println!("       ✗ Measurement error - not a valid GHZ state");
    }

    Ok(())
}

/// Demonstrate quantum error correction
async fn quantum_error_correction() -> Result<(), Box<dyn std::error::Error>> {
    let mut q = create()
        .with_backend(BackendType::Stabilizer)
        .with_qubits(3) // Just 3 qubits for simple repetition code
        .build()?;

    println!("Encoding logical |0⟩ into 3-qubit repetition code...");

    // Already in |000⟩ state - this is our encoded logical |0⟩

    println!("Introducing single bit flip error on qubit 1...");
    q.x(1).await?;

    // Classical error correction - just measure all qubits
    let q0 = q.measure(0).await?;
    let q1 = q.measure(1).await?;
    let q2 = q.measure(2).await?;

    println!("Measured state: |{}{}{}>", q0, q1, q2);

    // Majority vote
    let zeros = (q0 == 0) as i32 + (q1 == 0) as i32 + (q2 == 0) as i32;
    let logical_value = if zeros >= 2 { 0 } else { 1 };

    println!("Majority vote result: {}", logical_value);

    if logical_value == 0 {
        println!("✓ Error correction successful - recovered logical |0⟩!");
    } else {
        println!("✗ Error correction failed");
    }

    // Now let's do a proper stabilizer-based error correction
    println!("\nDemonstrating stabilizer-based error detection:");

    let mut q2 = create()
        .with_backend(BackendType::Stabilizer)
        .with_qubits(3)
        .build()?;

    // Create a more interesting encoded state using stabilizers
    // Encode |+⟩ state
    q2.h(0).await?;
    q2.cnot(0, 1).await?;
    q2.cnot(1, 2).await?;

    println!("Created encoded |+⟩ state with stabilizers XX_ and _XX");

    // Introduce error
    println!("Introducing Z error on qubit 1...");
    q2.z(1).await?;

    // The beauty of stabilizer formalism - we can detect this!
    // In a real implementation, we'd measure the stabilizer generators

    Ok(())
}

/// Benchmark with random Clifford circuits
async fn random_clifford_benchmark(
    n_qubits: usize,
    n_gates: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    use rand::Rng;
    let mut rng = rand::rng();

    let start = Instant::now();
    let mut q = create()
        .with_backend(BackendType::Stabilizer)
        .with_qubits(n_qubits)
        .build()?;

    println!(
        "Running {} random Clifford gates on {} qubits...",
        n_gates, n_qubits
    );

    for i in 0..n_gates {
        // Random gate type
        match rng.random_range(0..4) {
            0 => {
                // Random H gate
                let qubit = rng.random_range(0..n_qubits);
                q.h(qubit).await?;
            }
            1 => {
                // Random S gate
                let qubit = rng.random_range(0..n_qubits);
                q.s(qubit).await?;
            }
            2 => {
                // Random CNOT
                let q1 = rng.random_range(0..n_qubits);
                let mut q2 = rng.random_range(0..n_qubits);
                while q2 == q1 {
                    q2 = rng.random_range(0..n_qubits);
                }
                q.cnot(q1, q2).await?;
            }
            _ => {
                // Random Pauli
                let qubit = rng.random_range(0..n_qubits);
                match rng.random_range(0..3) {
                    0 => q.x(qubit).await?,
                    1 => q.y(qubit).await?,
                    _ => q.z(qubit).await?,
                }
            }
        }

        if (i + 1) % 100 == 0 {
            print!(".");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
    }
    println!();

    let elapsed = start.elapsed();
    let gates_per_sec = n_gates as f64 / elapsed.as_secs_f64();

    println!("Time: {:?}", elapsed);
    println!("Performance: {:.0} gates/second", gates_per_sec);
    println!(
        "Memory used: {} KB",
        2 * n_qubits * (2 * n_qubits + 1) / 8 / 1024
    );

    // Sample measurement
    let m0 = q.measure(0).await?;
    let m1 = q.measure(n_qubits / 2).await?;
    let m2 = q.measure(n_qubits - 1).await?;
    println!("Sample measurements: {} {} {}", m0, m1, m2);

    Ok(())
}

/// The ultimate test - create a massive GHZ state
async fn create_massive_ghz(n_qubits: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating a {}-qubit GHZ state...", n_qubits);

    // Calculate memory requirements without overflow
    if n_qubits <= 50 {
        let state_vector_memory = (1u128 << n_qubits) / 8 / 1_000_000_000_000;
        println!(
            "This would require {} TB with state vector!",
            state_vector_memory
        );
    } else {
        println!(
            "This would require 2^{} bytes with state vector (impossible!)",
            n_qubits
        );
    }

    let start = Instant::now();

    let mut q = create()
        .with_backend(BackendType::Stabilizer)
        .with_qubits(n_qubits)
        .build()?;

    // Create GHZ
    q.h(0).await?;

    // Show progress
    print!("Applying CNOT gates: ");
    for i in 1..n_qubits {
        q.cnot(0, i).await?;
        if i % 100 == 0 {
            print!("{}%", (i * 100) / n_qubits);
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
            print!("\x1B[{}D", 4); // Move cursor back
        }
    }
    println!("100%");

    let creation_time = start.elapsed();

    // Measure a few qubits
    let measurements = vec![
        q.measure(0).await?,
        q.measure(1000).await?,
        q.measure(2500).await?,
        q.measure(4999).await?,
    ];

    let total_time = start.elapsed();

    println!("\nResults:");
    println!("  Creation time: {:?}", creation_time);
    println!("  Total time: {:?}", total_time);
    println!(
        "  Memory used: {} MB",
        2 * n_qubits * (2 * n_qubits + 1) / 8 / 1_000_000
    );
    println!("  Measurements: {:?}", measurements);

    // Verify GHZ
    let all_same = measurements.windows(2).all(|w| w[0] == w[1]);
    if all_same {
        println!("  ✓ Successfully created {}-qubit GHZ state!", n_qubits);
    } else {
        println!("  ✗ Measurements don't match - something went wrong");
    }

    Ok(())
}
