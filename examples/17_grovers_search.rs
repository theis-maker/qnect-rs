use qnect::backend::backend::QuantumBackend;
use qnect::create;
use qnect::system::QuantumSystem;
use std::f64::consts::PI;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Qnect: Grover's Search Algorithm ===\n");
    println!("Grover's algorithm finds a marked item in an unsorted database");
    println!("with O(√N) operations instead of classical O(N).\n");

    // Example 1: Search in 4 items (2 qubits)
    println!("Example 1: Search for item 3 in a 4-item database");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    search_for_item(4, 3).await?;

    // Example 2: Search in 8 items (3 qubits)
    println!("\nExample 2: Search for item 5 in an 8-item database");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    search_for_item(8, 5).await?;

    // Example 3: Search in 16 items (4 qubits)
    println!("\nExample 3: Search for item 10 in a 16-item database");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    search_for_item(16, 10).await?;

    // Example 4: Multiple marked items
    println!("\nExample 4: Search with multiple marked items");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    search_multiple_marked(8, vec![2, 5]).await?;

    // Example 5: Visual demonstration with circuit
    println!("\nExample 5: Visual circuit for 2-qubit Grover");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    visual_grover_demo().await?;

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("Grover's Algorithm Summary:");
    println!("✓ Quadratic speedup over classical search");
    println!("✓ Optimal after ~π√N/4 iterations");
    println!("✓ Works with multiple marked items");
    println!("✓ Demonstrates quantum amplitude amplification");
    println!("═══════════════════════════════════════════════════════════════");

    Ok(())
}

/// Perform Grover's search for a single marked item
async fn search_for_item(
    n_items: usize,
    marked_item: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let n_qubits = (n_items as f64).log2().ceil() as usize;

    // For 4 logical qubits, allocate one extra ancilla
    let (total_qubits, ancilla) = if n_qubits == 4 {
        (n_qubits + 1, Some(n_qubits)) // ancilla at index 4
    } else {
        (n_qubits, None)
    };

    let mut q = create().with_qubits(total_qubits).build()?;

    println!("Database size: {} items ({} qubits)", n_items, n_qubits);
    if ancilla.is_some() {
        println!("Using 1 ancilla qubit for clean multi-controlled gates");
    }
    println!("Marked item: {}", marked_item);

    // Initialize superposition on logical qubits only
    for i in 0..n_qubits {
        q.h(i).await?;
    }

    // Calculate optimal iterations
    let iterations = calculate_iterations(n_items, 1);
    println!("Grover iterations: {}", iterations);

    // Perform Grover iterations
    for _ in 0..iterations {
        apply_oracle(&mut q, n_qubits, marked_item, ancilla).await?;
        apply_diffusion(&mut q, n_qubits, ancilla).await?;
    }

    // Measure logical qubits only (not the ancilla)
    let mut measurements = Vec::new();
    for i in 0..n_qubits {
        measurements.push(q.measure(i).await?);
    }

    let result = bits_to_number(&measurements);
    println!(
        "Measurement result: {} (binary: {:0width$b})",
        result,
        result,
        width = n_qubits
    );

    if result == marked_item {
        println!("✓ SUCCESS! Found the marked item!");
    } else {
        println!("✗ Found {} instead of {}", result, marked_item);
    }

    // Run multiple times to show probability
    println!("\nRunning 100 times to show success probability:");
    let mut success_count = 0;

    for _ in 0..100 {
        let mut q = create().with_qubits(total_qubits).build()?;

        // Initialize logical qubits
        for i in 0..n_qubits {
            q.h(i).await?;
        }

        // Grover iterations
        for _ in 0..iterations {
            apply_oracle(&mut q, n_qubits, marked_item, ancilla).await?;
            apply_diffusion(&mut q, n_qubits, ancilla).await?;
        }

        // Measure logical qubits only
        let mut measurements = Vec::new();
        for i in 0..n_qubits {
            measurements.push(q.measure(i).await?);
        }

        if bits_to_number(&measurements) == marked_item {
            success_count += 1;
        }
    }

    println!("Success rate: {}%", success_count);

    Ok(())
}

/// Apply the oracle that marks the target item
async fn apply_oracle(
    q: &mut QuantumSystem<Box<dyn QuantumBackend>>,
    n_qubits: usize,
    marked: usize,
    ancilla: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Apply X gates to qubits that are 0 in the binary representation
    for i in 0..n_qubits {
        if (marked >> i) & 1 == 0 {
            q.x(i).await?;
        }
    }

    apply_multi_controlled_z(q, n_qubits, ancilla).await?;

    // Undo the X gates
    for i in 0..n_qubits {
        if (marked >> i) & 1 == 0 {
            q.x(i).await?;
        }
    }

    Ok(())
}

/// Apply the diffusion operator (inversion about average)
async fn apply_diffusion(
    q: &mut QuantumSystem<Box<dyn QuantumBackend>>,
    n_qubits: usize,
    ancilla: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Apply H gates
    for i in 0..n_qubits {
        q.h(i).await?;
    }

    // Apply X gates
    for i in 0..n_qubits {
        q.x(i).await?;
    }

    // Multi-controlled Z
    apply_multi_controlled_z(q, n_qubits, ancilla).await?;

    // Undo X gates
    for i in 0..n_qubits {
        q.x(i).await?;
    }

    // Undo H gates
    for i in 0..n_qubits {
        q.h(i).await?;
    }

    Ok(())
}

/// Calculate optimal number of Grover iterations
fn calculate_iterations(n_items: usize, n_marked: usize) -> usize {
    let theta = (n_marked as f64 / n_items as f64).sqrt().asin();
    let iterations = (PI / (4.0 * theta)).round() as usize;
    iterations.max(1)
}

/// Convert bit array to number
fn bits_to_number(bits: &[u8]) -> usize {
    bits.iter()
        .enumerate()
        .map(|(i, &bit)| (bit as usize) << i)
        .sum()
}

/// Apply multi-controlled Z gate with proper decomposition
async fn apply_multi_controlled_z(
    q: &mut QuantumSystem<Box<dyn QuantumBackend>>,
    n_qubits: usize,
    ancilla: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    match n_qubits {
        2 => {
            q.cz(0, 1).await?;
        }
        3 => {
            // CCZ via H-CCX-H
            q.h(2).await?;
            q.ccx(0, 1, 2).await?;
            q.h(2).await?;
        }
        4 => {
            // CCCZ using ancilla
            let anc = ancilla.ok_or("CCCZ for 4 qubits requires an ancilla")?;

            // Compute ancilla = q0 AND q1
            q.ccx(0, 1, anc).await?;

            // Apply CCZ(ancilla, q2, q3)
            q.h(3).await?;
            q.ccx(anc, 2, 3).await?;
            q.h(3).await?;

            // Uncompute ancilla
            q.ccx(0, 1, anc).await?;
        }
        _ => return Err("Multi-controlled Z for >4 qubits not implemented".into()),
    }
    Ok(())
}

/// Search for multiple marked items
async fn search_multiple_marked(
    n_items: usize,
    marked_items: Vec<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let n_qubits = (n_items as f64).log2().ceil() as usize;
    let (total_qubits, ancilla) = if n_qubits == 4 {
        (n_qubits + 1, Some(n_qubits))
    } else {
        (n_qubits, None)
    };

    println!("Database size: {} items ({} qubits)", n_items, n_qubits);
    println!("Marked items: {:?}", marked_items);

    let iterations = calculate_iterations(n_items, marked_items.len());
    println!("Grover iterations: {}", iterations);

    let mut success_counts = vec![0; n_items];

    for _ in 0..100 {
        let mut q = create().with_qubits(total_qubits).build()?;

        // Initialize superposition
        for i in 0..n_qubits {
            q.h(i).await?;
        }

        // Grover iterations
        for _ in 0..iterations {
            // Oracle for multiple marked items
            for &marked in &marked_items {
                apply_oracle(&mut q, n_qubits, marked, ancilla).await?;
            }
            apply_diffusion(&mut q, n_qubits, ancilla).await?;
        }

        // Measure logical qubits only
        let mut measurements = Vec::new();
        for i in 0..n_qubits {
            measurements.push(q.measure(i).await?);
        }

        let result = bits_to_number(&measurements);
        if result < n_items {
            success_counts[result] += 1;
        }
    }

    println!("\nMeasurement distribution (100 runs):");
    for (item, count) in success_counts.iter().enumerate() {
        if *count > 0 {
            println!(
                "Item {}: {} times{}",
                item,
                count,
                if marked_items.contains(&item) {
                    " ✓ (marked)"
                } else {
                    ""
                }
            );
        }
    }

    Ok(())
}

/// Visual demonstration with circuit output
async fn visual_grover_demo() -> Result<(), Box<dyn std::error::Error>> {
    let mut q = create().with_qubits(2).build()?.with_recording();

    println!("Grover's algorithm circuit for 2 qubits, marked item = 3:");

    // Initialization
    q.h(0).await?;
    q.h(1).await?;

    // One Grover iteration
    // Oracle for |11⟩
    q.cz(0, 1).await?;

    // Diffusion
    q.h(0).await?;
    q.h(1).await?;
    q.x(0).await?;
    q.x(1).await?;
    q.cz(0, 1).await?;
    q.x(0).await?;
    q.x(1).await?;
    q.h(0).await?;
    q.h(1).await?;

    // Measure
    q.measure(0).await?;
    q.measure(1).await?;

    println!();
    q.print_circuit();

    Ok(())
}
