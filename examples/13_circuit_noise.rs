use qnect::{builder::NoiseModel, create};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Qnect: Noise Model Demonstration ===\n");
    println!("This example shows how noise affects quantum circuits");
    println!("by comparing ideal vs noisy Bell state preparation.\n");

    // Test 1: Bell State Fidelity vs Noise
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("1. BELL STATE FIDELITY vs NOISE LEVEL");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let noise_levels = vec![0.0, 0.001, 0.01, 0.05, 0.1];
    let shots = 1000;

    for noise in noise_levels {
        let mut perfect_bell = 0;

        for _ in 0..shots {
            let mut q = create()
                .with_qubits(2)
                .with_noise(NoiseModel {
                    depolarizing_rate: noise,
                    measurement_error: 0.0, // Isolate gate noise
                })
                .build()?;

            // Create Bell state
            q.h(0).await?;
            q.cnot(0, 1).await?;

            // Measure
            let m0 = q.measure(0).await?;
            let m1 = q.measure(1).await?;

            // Check if we got |00⟩ or |11⟩ (perfect Bell correlation)
            if m0 == m1 {
                perfect_bell += 1;
            }
        }

        let fidelity = perfect_bell as f64 / shots as f64;
        println!(
            "Gate noise: {:5.1}% → Bell state fidelity: {:.3} (ideal: 1.000)",
            noise * 100.0,
            fidelity
        );
    }

    // Test 2: Measurement Error Effects
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("2. MEASUREMENT ERROR EFFECTS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let measurement_errors = vec![0.0, 0.01, 0.05, 0.1];

    for m_error in measurement_errors {
        let mut zeros = 0;

        for _ in 0..shots {
            let mut q = create()
                .with_qubits(1)
                .with_noise(NoiseModel {
                    depolarizing_rate: 0.0, // No gate noise
                    measurement_error: m_error,
                })
                .build()?;

            // Prepare |0⟩ (do nothing)
            // Measure - should always be 0 without error
            if q.measure(0).await? == 0 {
                zeros += 1;
            }
        }

        let accuracy = zeros as f64 / shots as f64;
        println!(
            "Measurement error: {:5.1}% → P(measure 0|prep 0): {:.3}",
            m_error * 100.0,
            accuracy
        );
    }

    // Test 3: GHZ State Degradation
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("3. GHZ STATE DEGRADATION WITH CIRCUIT DEPTH");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let qubit_counts = vec![2, 3, 4, 5];
    let noise_rate = 0.01; // 1% per gate

    for n in qubit_counts {
        let mut ghz_correlations = 0;

        for _ in 0..shots {
            let mut q = create()
                .with_qubits(n)
                .with_noise(NoiseModel {
                    depolarizing_rate: noise_rate,
                    measurement_error: 0.0,
                })
                .build()?;

            // Create GHZ state
            q.h(0).await?;
            for i in 1..n {
                q.cnot(0, i).await?;
            }

            // Measure all qubits
            let mut measurements = Vec::new();
            for i in 0..n {
                measurements.push(q.measure(i).await?);
            }

            // Check if all 0s or all 1s
            let all_same = measurements.iter().all(|&m| m == measurements[0]);
            if all_same {
                ghz_correlations += 1;
            }
        }

        let ghz_fidelity = ghz_correlations as f64 / shots as f64;
        println!(
            "{}-qubit GHZ (depth {}): fidelity = {:.3}",
            n,
            n, // Circuit depth grows with qubit count
            ghz_fidelity
        );
    }

    // Test 4: Error Accumulation in Deep Circuits
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("4. ERROR ACCUMULATION IN DEEP CIRCUITS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let depths = vec![1, 10, 50, 100];
    let noise_rate = 0.001; // 0.1% per gate

    for depth in depths {
        let mut correct = 0;

        for _ in 0..shots {
            let mut q = create()
                .with_qubits(1)
                .with_noise(NoiseModel {
                    depolarizing_rate: noise_rate,
                    measurement_error: 0.0,
                })
                .build()?;

            // Apply pairs of X gates (should cancel out)
            for _ in 0..depth {
                q.x(0).await?;
                q.x(0).await?;
            }

            // Should measure |0⟩ if no errors
            if q.measure(0).await? == 0 {
                correct += 1;
            }
        }

        let success_rate = correct as f64 / shots as f64;
        let expected = (1.0 - 2.0 * noise_rate).powi(2 * depth); // Rough approximation

        println!(
            "Circuit depth {:3}: success rate = {:.3} (expected ≈ {:.3})",
            2 * depth, // Total gate count
            success_rate,
            expected
        );
    }

    // Test 5: Noise Comparison - Ideal vs Noisy
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("5. QUANTUM ALGORITHM COMPARISON: IDEAL vs NOISY");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Simple quantum algorithm: Create superposition and measure statistics
    let mut ideal_stats = HashMap::new();
    let mut noisy_stats = HashMap::new();

    for _ in 0..shots {
        // Ideal circuit
        let mut q_ideal = create().with_qubits(3).build()?;

        // Create equal superposition
        q_ideal.h(0).await?;
        q_ideal.h(1).await?;
        q_ideal.h(2).await?;

        let outcome = format!(
            "{}{}{}",
            q_ideal.measure(0).await?,
            q_ideal.measure(1).await?,
            q_ideal.measure(2).await?
        );
        *ideal_stats.entry(outcome).or_insert(0) += 1;

        // Noisy circuit
        let mut q_noisy = create()
            .with_qubits(3)
            .with_noise(NoiseModel {
                depolarizing_rate: 0.02,
                measurement_error: 0.01,
            })
            .build()?;

        q_noisy.h(0).await?;
        q_noisy.h(1).await?;
        q_noisy.h(2).await?;

        let outcome = format!(
            "{}{}{}",
            q_noisy.measure(0).await?,
            q_noisy.measure(1).await?,
            q_noisy.measure(2).await?
        );
        *noisy_stats.entry(outcome).or_insert(0) += 1;
    }

    println!("\nIdeal 3-qubit superposition distribution:");
    for i in 0..8 {
        let bitstring = format!("{:03b}", i);
        let count = ideal_stats.get(&bitstring).unwrap_or(&0);
        let prob = *count as f64 / shots as f64;
        println!("  |{}⟩: {:.3} (expect: 0.125)", bitstring, prob);
    }

    println!("\nNoisy circuit (2% gate error, 1% measurement error):");
    for i in 0..8 {
        let bitstring = format!("{:03b}", i);
        let count = noisy_stats.get(&bitstring).unwrap_or(&0);
        let prob = *count as f64 / shots as f64;
        println!(
            "  |{}⟩: {:.3} (deviation: {:+.3})",
            bitstring,
            prob,
            prob - 0.125
        );
    }

    // Calculate total variation distance
    let mut tvd = 0.0;
    for i in 0..8 {
        let bitstring = format!("{:03b}", i);
        let ideal = ideal_stats.get(&bitstring).unwrap_or(&0);
        let noisy = noisy_stats.get(&bitstring).unwrap_or(&0);
        tvd += (*ideal as f64 - *noisy as f64).abs() / shots as f64;
    }
    tvd /= 2.0;

    println!("\nTotal variation distance: {:.3}", tvd);
    println!("(0 = identical distributions, 1 = completely different)");

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("CONCLUSION:");
    println!("• Small noise rates significantly impact quantum algorithms");
    println!("• Errors accumulate with circuit depth");
    println!("• Measurement errors can be as impactful as gate errors");
    println!("• Real quantum computers need error correction!");
    println!("═══════════════════════════════════════════════════════════════");

    Ok(())
}
