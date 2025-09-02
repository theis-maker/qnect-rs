#[cfg(test)]
mod example_tests {
    use std::process::{Command, Stdio};
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    fn run_example_with_timeout(name: &str, timeout_secs: u64) -> Result<(), String> {
        println!("Running example: {}", name);

        let (tx, rx) = mpsc::channel();
        let example_name = name.to_string();

        thread::spawn(move || {
            let output = Command::new("cargo")
                .args(["run", "--example", &example_name])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output();

            tx.send(output).ok();
        });

        match rx.recv_timeout(Duration::from_secs(timeout_secs)) {
            Ok(Ok(output)) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(format!("Example {} failed:\n{}", name, stderr));
                }
                println!("✓ {} completed successfully\n", name);
                Ok(())
            }
            Ok(Err(e)) => Err(format!("Failed to execute {}: {}", name, e)),
            Err(_) => Err(format!(
                "Example {} timed out after {}s",
                name, timeout_secs
            )),
        }
    }

    #[test]
    fn test_all_examples() {
        let examples = vec![
            // Original examples
            ("00_quantum_verification", 30),
            ("01_quantum_interference", 5),
            ("02_ghz_scaling", 30),
            ("03_simple_bell_pair", 5),
            ("04_quantum_teleportation", 5),
            ("05_backend_comparison", 5),
            ("06_error_handling", 5),
            ("07_quantum_walk", 5),
            ("08_quantum_network", 5),
            ("09_qkd_bb84", 5),
            ("10_qkd_with_eve", 5),
            ("11_gate_showcase", 5),
            ("12_future_network", 5),
            ("13_circuit_noise", 5),
            ("14_dagger_gates", 5),
            ("15_qasm_export", 5),
            ("16_qasm_import", 5),
            ("17_grovers_search", 10),
            ("18_stabilizer_demo", 30),
            ("19_quantum_network_protocols", 10),
            ("20_quantum_internet_stack", 10),
            ("21_qnpu_test", 5),
            ("22_quantum_anonymous_transmission", 10),
            // New hub and builder examples
            ("quantum_hub_demo", 10),
            ("network_builder_demo", 10),
            ("quantum_topologies", 15),
            ("quantum_qkd_chat", 10),
        ];

        let mut failed = Vec::new();

        for (example, timeout) in &examples {
            if let Err(e) = run_example_with_timeout(example, *timeout) {
                failed.push((example, e));
            }
        }

        if !failed.is_empty() {
            eprintln!("\n❌ Failed examples:");
            for (name, error) in &failed {
                eprintln!("  - {}: {}", name, error);
            }
            panic!("{} examples failed!", failed.len());
        } else {
            println!("\n✅ All {} examples passed!", examples.len());
        }
    }
}
