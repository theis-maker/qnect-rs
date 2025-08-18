use qnect::create;
use rand::Rng;

/// BB84 Quantum Key Distribution Protocol
/// Alice and Bob establish a shared secret key using quantum mechanics
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Qnect: BB84 Quantum Key Distribution ---\n");
    println!("Alice and Bob want to establish a shared secret key.");
    println!(
        "Eve might be listening on the classical channel, but quantum mechanics protects them!\n"
    );

    // Parameters
    let key_length = 16; // Desired final key length
    let num_qubits = key_length * 4; // Send extra for privacy amplification

    // Alice's random bit string and basis choices
    let mut rng = rand::rng();
    let alice_bits: Vec<u8> = (0..num_qubits).map(|_| rng.random_range(0..2)).collect();
    let alice_bases: Vec<bool> = (0..num_qubits).map(|_| rng.random_bool(0.5)).collect();

    println!("Step 1: Alice prepares {} qubits", num_qubits);
    println!("  Alice's bits:  {:?}...", &alice_bits[..8]);
    println!(
        "  Alice's bases: {:?}... (true=X, false=Z)\n",
        &alice_bases[..8]
    );

    // Bob's random basis choices
    let bob_bases: Vec<bool> = (0..num_qubits).map(|_| rng.random_bool(0.5)).collect();

    // Quantum transmission phase
    let mut bob_measurements = Vec::new();

    for i in 0..num_qubits {
        let mut q = create().with_qubits(1).build()?;

        // Alice prepares qubit based on her bit and basis
        if alice_bits[i] == 1 {
            q.x(0).await?; // Prepare |1⟩
        }
        if alice_bases[i] {
            q.h(0).await?; // Switch to X basis
        }

        // Simulate quantum channel transmission
        // (In real QKD, the qubit would travel through fiber or free space)

        // Bob measures in his chosen basis
        if bob_bases[i] {
            q.h(0).await?; // Measure in X basis
        }
        let measurement = q.measure(0).await?;
        bob_measurements.push(measurement);
    }

    println!("Step 2: Quantum transmission complete");
    println!("  Bob's measurements: {:?}...\n", &bob_measurements[..8]);

    // Classical communication phase - basis reconciliation
    println!("Step 3: Basis reconciliation (classical channel)");
    let mut sifted_key_alice = Vec::new();
    let mut sifted_key_bob = Vec::new();

    for i in 0..num_qubits {
        if alice_bases[i] == bob_bases[i] {
            sifted_key_alice.push(alice_bits[i]);
            sifted_key_bob.push(bob_measurements[i]);
        }
    }

    println!(
        "  Matching bases: {} out of {}",
        sifted_key_alice.len(),
        num_qubits
    );
    println!("  Sifted key length: {}\n", sifted_key_alice.len());

    // Error estimation (sacrifice some bits to check for eavesdropping)
    let check_size = sifted_key_alice.len() / 4;
    let mut errors = 0;

    println!("Step 4: Error rate estimation");
    for i in 0..check_size {
        if sifted_key_alice[i] != sifted_key_bob[i] {
            errors += 1;
        }
    }

    let error_rate = errors as f64 / check_size as f64;
    println!("  Checked {} bits, found {} errors", check_size, errors);
    println!("  Error rate: {:.1}%", error_rate * 100.0);

    if error_rate > 0.11 {
        println!("\n❌ ERROR RATE TOO HIGH! Possible eavesdropper detected!");
        println!("   Aborting key generation for security.");
        return Ok(());
    }

    println!("  ✓ Error rate acceptable - no eavesdropper detected\n");

    // Final key (excluding checked bits)
    let final_key_alice: Vec<u8> = sifted_key_alice[check_size..check_size + key_length].to_vec();
    let final_key_bob: Vec<u8> = sifted_key_bob[check_size..check_size + key_length].to_vec();

    println!("Step 5: Final shared secret key");
    println!("  Alice's key: {:?}", final_key_alice);
    println!("  Bob's key:   {:?}", final_key_bob);
    println!(
        "  Match: {}\n",
        if final_key_alice == final_key_bob {
            "✓ Yes!"
        } else {
            "✗ No"
        }
    );

    // Convert to hex for display
    let key_hex: String = final_key_alice
        .iter()
        .map(|&b| format!("{}", b))
        .collect::<String>()
        .chars()
        .collect::<Vec<_>>()
        .chunks(4)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ");

    println!("🔐 Shared Secret Key (in binary blocks): {}", key_hex);
    println!("\nThis key is guaranteed secure by the laws of quantum mechanics!");

    Ok(())
}
