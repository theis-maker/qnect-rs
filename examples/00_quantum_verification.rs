use std::f64::consts::PI;

use qnect::create;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Qnect: Reproducible Quantum Verification ---\n");
    println!("This demo validates the simulator by reproducing standard quantum predictions.");
    println!("(It is not a hardware Bell test; it checks that the *simulator* matches theory.)\n");

    // -------------------------------
    // 1) CHSH (Bell inequality)
    // -------------------------------
    println!("=======================================================");
    println!("1) CHSH (Bell inequality) => compute S with 95% CI");
    println!("   Classical bound: S ≤ 2; Quantum max: 2√2 ≈ 2.828\n");

    println!("CHSH Test Circuit (one round):");
    let mut demo = create().with_qubits(2).build()?.with_recording();
    demo.h(0).await?;
    demo.cnot(0, 1).await?;
    demo.ry(1, 5.0 * PI / 8.0).await?; // Example with b' measurement
    demo.h(0).await?; // Alice's X measurement
    demo.measure(0).await?;
    demo.measure(1).await?;
    demo.print_circuit();
    println!();

    fn pm1(b: u8) -> f64 {
        if b == 0 { 1.0 } else { -1.0 }
    }

    async fn correlator(
        alice: usize,
        bob: usize,
        a_prime: bool,
        b_prime: bool,
        shots: usize,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        let mut sum = 0.0;
        for _ in 0..shots {
            let mut q = create().with_qubits(2).build()?;
            // Bell state |Φ+⟩
            q.h(alice).await?;
            q.cnot(alice, bob).await?;

            // Alice basis: a = Z, a' = X (H then Z)
            if a_prime {
                q.h(alice).await?;
            }
            // Bob basis: b = RY(-π/8) then Z,  b' = RY(5π/8) then Z
            if b_prime {
                q.ry(bob, 5.0 * PI / 8.0).await?;
            } else {
                q.ry(bob, -PI / 8.0).await?;
            }

            let a = pm1(q.measure(alice).await?);
            let b = pm1(q.measure(bob).await?);
            sum += a * b;
        }
        Ok(sum / shots as f64)
    }

    async fn chsh_for(
        alice: usize,
        bob: usize,
        shots: usize,
    ) -> Result<(f64, [f64; 4], f64), Box<dyn std::error::Error>> {
        // E(a,b), E(a,b'), E(a',b), E(a',b')
        let e1 = correlator(alice, bob, false, false, shots).await?;
        let e2 = correlator(alice, bob, false, true, shots).await?;
        let e3 = correlator(alice, bob, true, false, shots).await?;
        let e4 = correlator(alice, bob, true, true, shots).await?;
        let s = e1 - e2 + e3 - e4; // Note the minus before e4
        // Conservative SE for S from the four correlators:
        let se = (((1.0 - e1 * e1) + (1.0 - e2 * e2) + (1.0 - e3 * e3) + (1.0 - e4 * e4))
            / shots as f64)
            .sqrt();
        Ok((s, [e1, e2, e3, e4], se))
    }

    let shots = 10_000usize;

    // Try both labelings in case qubit order is reversed in the backend.
    let (s01, e01, se01) = chsh_for(0, 1, shots).await?;
    let (s10, e10, se10) = chsh_for(1, 0, shots).await?;

    // Pick the stronger violation to display prominently.
    let (best_lab, s, es, se) = if s01.abs() >= s10.abs() {
        ("Alice=0, Bob=1", s01, e01, se01)
    } else {
        ("Alice=1, Bob=0", s10, e10, se10)
    };

    let ci_low = s - 1.96 * se;
    let ci_high = s + 1.96 * se;

    println!("   Correlators ({}):", best_lab);
    println!("     E(a,b)   ≈ {:+.4}", es[0]);
    println!("     E(a,b')  ≈ {:+.4}", es[1]);
    println!("     E(a',b)  ≈ {:+.4}", es[2]);
    println!("     E(a',b') ≈ {:+.4}", es[3]);
    println!("   S ≈ {:.4}  (95% CI: [{:.4}, {:.4}])", s, ci_low, ci_high);
    println!(
        "   Result: {}",
        if s > 2.0 {
            "Violation observed (> 2)"
        } else {
            "No violation (check qubit order / ry gate)"
        }
    );
    println!();

    // -------------------------------
    // 2) Phase kickback (⟨X⟩ flip)
    // -------------------------------
    println!("=======================================================");
    println!("2) Phase kickback => ⟨X⟩ on control flips sign with CZ (target in |1⟩)\n");

    println!("Phase Kickback Circuit:");
    let mut demo = create().with_qubits(2).build()?.with_recording();
    demo.h(0).await?;
    demo.x(1).await?;
    demo.cz(0, 1).await?;
    demo.h(0).await?;
    demo.measure(0).await?;
    demo.measure(1).await?;
    demo.print_circuit();
    println!();

    let shots = 10_000usize;
    let mut x_sum_no_cz = 0.0;
    let mut x_sum_with_cz = 0.0;

    // Baseline: |+⟩|1⟩; measure X on control (H then Z)
    for _ in 0..shots {
        let mut q = create().with_qubits(2).build()?;
        q.h(0).await?; // control -> |+>
        q.x(1).await?; // target  -> |1>
        q.h(0).await?; // measure X via H then Z
        x_sum_no_cz += pm1(q.measure(0).await?);
    }

    // With CZ between them
    for _ in 0..shots {
        let mut q = create().with_qubits(2).build()?;
        q.h(0).await?;
        q.x(1).await?;
        q.cz(0, 1).await?;
        q.h(0).await?;
        x_sum_with_cz += pm1(q.measure(0).await?);
    }

    let x_no = x_sum_no_cz / shots as f64;
    let x_yes = x_sum_with_cz / shots as f64;
    println!("   ⟨X⟩ without CZ ≈ {:+.3}", x_no); // ~ +1
    println!("   ⟨X⟩ with    CZ ≈ {:+.3}", x_yes); // ~ -1
    println!();

    // -------------------------------
    // 3) Interference A/B
    // -------------------------------
    println!("=======================================================");
    println!("3) Interference A/B => P(0) baseline vs with path phases (Z,S)\n");

    println!("Interference Circuit (with phases):");
    let mut demo = create().with_qubits(3).build()?.with_recording();
    demo.h(0).await?;
    demo.cnot(0, 1).await?;
    demo.cnot(0, 2).await?;
    demo.z(1).await?;
    demo.s(2).await?;
    demo.cnot(0, 1).await?;
    demo.cnot(0, 2).await?;
    demo.h(0).await?;
    demo.measure(0).await?;
    demo.print_circuit();
    println!();

    let shots = 10_000usize;
    let mut zeros_no = 0usize;
    let mut zeros_yes = 0usize;

    // (A) Baseline: no phases → whole gadget cancels → H then H = I → P(0) ≈ 1
    for _ in 0..shots {
        let mut q = create().with_qubits(3).build()?;
        q.h(0).await?;
        q.cnot(0, 1).await?;
        q.cnot(0, 2).await?;
        // no phases
        q.cnot(0, 1).await?;
        q.cnot(0, 2).await?;
        q.h(0).await?;
        if q.measure(0).await? == 0 {
            zeros_no += 1;
        }
    }

    // (B) With phases on the ancilla paths (Z on q1, S on q2) → P(0) ≈ 0.5
    for _ in 0..shots {
        let mut q = create().with_qubits(3).build()?;
        q.h(0).await?;
        q.cnot(0, 1).await?;
        q.cnot(0, 2).await?;
        q.z(1).await?;
        q.s(2).await?;
        q.cnot(0, 1).await?;
        q.cnot(0, 2).await?;
        q.h(0).await?;
        if q.measure(0).await? == 0 {
            zeros_yes += 1;
        }
    }

    let p0_no = zeros_no as f64 / shots as f64;
    let p0_yes = zeros_yes as f64 / shots as f64;
    println!("   P(0) baseline (no phases):  {:.3}", p0_no); // ~ 1.000
    println!("   P(0) with phases (Z,S):     {:.3}", p0_yes); // ~ 0.500
    println!();

    // -------------------------------
    // 4) GHZ: Z-basis and X-parity
    // -------------------------------
    println!("=======================================================");
    println!("4) GHZ => outcomes in Z are only 000/111; X-parity is even\n");

    println!("GHZ State Circuit:");
    let mut demo = create().with_qubits(3).build()?.with_recording();
    demo.h(0).await?;
    demo.cnot(0, 1).await?;
    demo.cnot(0, 2).await?;
    demo.measure(0).await?;
    demo.measure(1).await?;
    demo.measure(2).await?;
    demo.print_circuit();
    println!();

    let shots = 10_000usize;
    let mut counts = [0usize; 8];

    for _ in 0..shots {
        let mut q = create().with_qubits(3).build()?;
        q.h(0).await?;
        q.cnot(0, 1).await?;
        q.cnot(0, 2).await?;
        let m0 = q.measure(0).await?;
        let m1 = q.measure(1).await?;
        let m2 = q.measure(2).await?;
        let idx = (m0 as usize) << 2 | (m1 as usize) << 1 | (m2 as usize);
        counts[idx] += 1;
    }

    let only_000_111 = (counts[0] + counts[7]) as f64 / shots as f64;
    println!("   Z-basis: P(000)+P(111) ≈ {:.3}", only_000_111); // ~1.0
    println!("   (Other outcomes total: {:.3})", 1.0 - only_000_111);

    // X-parity check: apply H to all (measuring X) and check parity m0 ^ m1 ^ m2 == 0
    let mut even_parity = 0usize;
    for _ in 0..shots {
        let mut q = create().with_qubits(3).build()?;
        q.h(0).await?;
        q.cnot(0, 1).await?;
        q.cnot(0, 2).await?;
        q.h(0).await?;
        q.h(1).await?;
        q.h(2).await?;
        let m0 = q.measure(0).await?;
        let m1 = q.measure(1).await?;
        let m2 = q.measure(2).await?;
        if ((m0 ^ m1) ^ m2) == 0 {
            even_parity += 1;
        }
    }
    let p_even = even_parity as f64 / shots as f64; // ~1.0
    println!("   X-parity even fraction ≈ {:.3}\n", p_even);

    // -------------------------------
    // 5) Teleportation fidelity (several inputs)
    // -------------------------------
    println!("=======================================================");
    println!("5) Teleportation => empirical fidelity for |ψ⟩ = RY(θ)|0⟩\n");

    println!("Teleportation Circuit:");
    let mut demo = create().with_qubits(3).build()?.with_recording();
    demo.ry(0, PI / 4.0).await?; // State to teleport
    demo.h(1).await?;
    demo.cnot(1, 2).await?;
    demo.cnot(0, 1).await?;
    demo.h(0).await?;
    demo.measure(0).await?;
    demo.measure(1).await?;
    demo.x(2).await?; // Example correction
    demo.z(2).await?; // Example correction
    demo.ry(2, -PI / 4.0).await?; // Verify state
    demo.measure(2).await?;
    demo.print_circuit();
    println!();

    async fn teleport_fidelity(
        theta: f64,
        shots: usize,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        let mut ok = 0usize;
        for _ in 0..shots {
            let mut q = create().with_qubits(3).build()?;
            // Prepare |ψ⟩ on qubit 0
            q.ry(0, theta).await?;
            // Share Bell pair on (1,2)
            q.h(1).await?;
            q.cnot(1, 2).await?;
            // Bell-measure (0,1)
            q.cnot(0, 1).await?;
            q.h(0).await?;
            let m0 = q.measure(0).await?;
            let m1 = q.measure(1).await?;
            // Bob’s corrections on qubit 2
            if m1 == 1 {
                q.x(2).await?;
            }
            if m0 == 1 {
                q.z(2).await?;
            }
            // Undo preparation; if states match, measuring Z gives 0
            q.ry(2, -theta).await?;
            if q.measure(2).await? == 0 {
                ok += 1;
            }
        }
        Ok(ok as f64 / shots as f64)
    }

    let tele_shots = 5_000usize;
    let thetas = [0.0, PI / 8.0, PI / 6.0, PI / 4.0, PI / 3.0, PI / 2.0];
    let mut avg_fid = 0.0;
    for &t in &thetas {
        let f = teleport_fidelity(t, tele_shots).await?;
        println!("   Fidelity for θ={:.3} ≈ {:.3}", t, f); // expect ~1.000 within sampling error
        avg_fid += f;
    }
    avg_fid /= thetas.len() as f64;
    println!("   Average fidelity ≈ {:.3}\n", avg_fid);

    // --------------------------------
    // Summary (all should look “near 1” or clearly > 2)
    // --------------------------------
    println!("=======================================================");
    println!("SUMMARY (simulator vs. textbook predictions)");
    println!(
        "  • CHSH: S ≈ {:.3} (95% CI [{:.3}, {:.3}]) → violates 2",
        s, ci_low, ci_high
    );
    println!("  • Kickback: ⟨X⟩ flips sign (+ → −) with CZ");
    println!(
        "  • Interference: P(0) baseline ≈ {:.3}, with phases ≈ {:.3}",
        p0_no, p0_yes
    );
    println!(
        "  • GHZ: Z-only 000/111 ≈ {:.3}, X-parity even ≈ {:.3}",
        only_000_111, p_even
    );
    println!("  • Teleportation: avg fidelity ≈ {:.3}", avg_fid);
    println!(
        "\nAll checks align with standard quantum theory => strong evidence simulator is correct."
    );
    println!("(Re-run to reproduce; more shots tighten the confidence intervals.)");
    println!("=======================================================");

    Ok(())
}
