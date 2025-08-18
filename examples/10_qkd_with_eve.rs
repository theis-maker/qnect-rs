use qnect::create;
use rand::Rng;

/// BB84 with Eve (eavesdropper) attempting to intercept
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Qnect: BB84 with Eavesdropper (Eve) ---\n");
    println!("Eve will try to intercept the quantum communication...\n");

    let num_qubits = 20;
    let mut rng = rand::rng();

    // Alice's preparation
    let alice_bits: Vec<u8> = (0..num_qubits).map(|_| rng.random_range(0..2)).collect();
    let alice_bases: Vec<bool> = (0..num_qubits).map(|_| rng.random_bool(0.5)).collect();

    // Bob's measurement bases
    let bob_bases: Vec<bool> = (0..num_qubits).map(|_| rng.random_bool(0.5)).collect();

    // Eve's interception strategy
    let eve_active = true;
    let eve_bases: Vec<bool> = (0..num_qubits).map(|_| rng.random_bool(0.5)).collect();

    println!("👤 Alice prepares {} qubits", num_qubits);
    println!(
        "🦹 Eve is {} the channel",
        if eve_active { "intercepting" } else { "not on" }
    );
    println!("👤 Bob awaits the qubits\n");

    let mut bob_measurements = Vec::new();
    let mut eve_measurements = Vec::new();

    for i in 0..num_qubits {
        let mut q = create().with_qubits(1).build()?;

        // Alice prepares
        if alice_bits[i] == 1 {
            q.x(0).await?;
        }
        if alice_bases[i] {
            q.h(0).await?;
        }

        // Eve intercepts!
        if eve_active {
            if eve_bases[i] {
                q.h(0).await?; // Eve measures in X basis
            }
            let eve_measurement = q.measure(0).await?;
            eve_measurements.push(eve_measurement);

            // Eve re-prepares what she measured (but this disturbs the state!)
            let mut q_new = create().with_qubits(1).build()?;
            if eve_measurement == 1 {
                q_new.x(0).await?;
            }
            if eve_bases[i] {
                q_new.h(0).await?;
            }
            q = q_new; // Send Eve's qubit to Bob
        }

        // Bob measures
        if bob_bases[i] {
            q.h(0).await?;
        }
        bob_measurements.push(q.measure(0).await?);
    }

    // Basis reconciliation
    let mut matching_indices = Vec::new();
    for i in 0..num_qubits {
        if alice_bases[i] == bob_bases[i] {
            matching_indices.push(i);
        }
    }

    // Check error rate on matching bases
    let mut errors = 0;
    for &i in &matching_indices {
        if alice_bits[i] != bob_measurements[i] {
            errors += 1;
        }
    }

    let error_rate = errors as f64 / matching_indices.len() as f64;

    println!("📊 Results:");
    println!(
        "  Matching bases: {}/{}",
        matching_indices.len(),
        num_qubits
    );
    println!("  Errors in matching bases: {}", errors);
    println!("  Error rate: {:.1}%", error_rate * 100.0);

    if eve_active {
        println!("\n🚨 EVE'S IMPACT:");
        println!("  Expected error rate with Eve: ~25%");
        println!("  Observed error rate: {:.1}%", error_rate * 100.0);

        if error_rate > 0.11 {
            println!("\n❌ EAVESDROPPER DETECTED!");
            println!("  The high error rate reveals Eve's presence.");
            println!("  Alice and Bob abort the protocol!");
        }
    } else {
        println!("\n✅ No eavesdropper - error rate is from noise only");
    }

    println!("\n💡 Key insight: Eve cannot measure without disturbing the qubits!");
    println!("   This is guaranteed by the quantum no-cloning theorem.");

    Ok(())
}
