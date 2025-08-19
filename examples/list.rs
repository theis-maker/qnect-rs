fn main() {
    println!("Available Qnect examples:\n");

    println!("== Quantum Basics ==");
    println!("  00_quantum_verification       - Comprehensive correctness tests");
    println!("  01_quantum_interference       - Wave-particle duality demo");
    println!("  02_ghz_scaling                - Large GHZ state generation");
    println!("  03_simple_bell_pair           - Basic entanglement");
    println!("  04_quantum_teleportation      - Quantum state transfer");
    println!("  05_backend_comparison         - Compare backends");
    println!("  06_error_handling             - Graceful error recovery");
    println!("  07_quantum_walk               - Quantum random walk");
    println!("  11_gate_showcase              - All available gates demo");
    println!("  14_dagger_gates               - Inverse gate examples");
    println!();

    println!("== Quantum Algorithms ==");
    println!("  09_qkd_bb84                   - Quantum key distribution (BB84)");
    println!("  10_qkd_with_eve               - QKD with eavesdropper");
    println!("  17_grovers_search             - Grover's algorithm");
    println!();

    println!("== Noise & QASM ==");
    println!("  13_circuit_noise              - Noisy circuit simulation");
    println!("  15_qasm_export                - Export circuits to QASM");
    println!("  16_qasm_import                - Import circuits from QASM");
    println!("  18_stabilizer_demo            - 5000-qubit stabilizer simulation");
    println!();

    println!("== Quantum Networking ==");
    println!("  08_quantum_network            - Simple network setup");
    println!("  12_future_network             - Experimental protocols");
    println!("  19_quantum_network_protocols  - Advanced network protocols");
    println!("  20_quantum_internet_stack     - Internet stack demo");
    println!("  22_quantum_anonymous_transmission - Anonymous quantum communication");
    println!();

    println!("== Production & Hardware ==");
    println!("  21_qnpu_test                  - Mock QNPU backend test");
    println!();

    println!("Run with: cargo run --example <name>");
}
