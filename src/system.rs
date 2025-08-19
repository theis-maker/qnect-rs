use crate::{
    backend::backend::{Gate1, Gate2, QuantumBackend},
    circuit_viz::{CircuitRecorder, Operation},
    error::Result,
};

/// High-level quantum system API that stays stable
#[derive(Debug, Clone, Copy)]
pub struct QuantumSystem<B: QuantumBackend> {
    backend: B,
}

impl<B: QuantumBackend> QuantumSystem<B> {
    pub fn new(backend: B) -> Self {
        QuantumSystem { backend }
    }

    /// Apply Hadamard gate
    pub async fn h(&mut self, qubit: usize) -> Result<()> {
        self.backend.apply_single_gate(qubit, Gate1::H).await
    }

    /// Apply Pauli-X gate
    pub async fn x(&mut self, qubit: usize) -> Result<()> {
        self.backend.apply_single_gate(qubit, Gate1::X).await
    }

    /// Apply Pauli-Y gate
    pub async fn y(&mut self, qubit: usize) -> Result<()> {
        self.backend.apply_single_gate(qubit, Gate1::Y).await
    }

    /// Apply Pauli-Z gate
    pub async fn z(&mut self, qubit: usize) -> Result<()> {
        self.backend.apply_single_gate(qubit, Gate1::Z).await
    }

    /// Apply S gate (phase)
    pub async fn s(&mut self, qubit: usize) -> Result<()> {
        self.backend.apply_single_gate(qubit, Gate1::S).await
    }

    /// Apply T gate (π/8)
    pub async fn t(&mut self, qubit: usize) -> Result<()> {
        self.backend.apply_single_gate(qubit, Gate1::T).await
    }

    /// Apply S† gate (S dagger)
    pub async fn s_dag(&mut self, qubit: usize) -> Result<()> {
        self.backend.apply_single_gate(qubit, Gate1::SDag).await
    }

    /// Apply T† gate (T dagger)
    pub async fn t_dag(&mut self, qubit: usize) -> Result<()> {
        self.backend.apply_single_gate(qubit, Gate1::TDag).await
    }

    /// Apply rotation around X axis
    pub async fn rx(&mut self, qubit: usize, angle: f64) -> Result<()> {
        self.backend
            .apply_single_gate(qubit, Gate1::Rx(angle))
            .await
    }

    /// Apply rotation around Y axis
    pub async fn ry(&mut self, qubit: usize, angle: f64) -> Result<()> {
        self.backend
            .apply_single_gate(qubit, Gate1::Ry(angle))
            .await
    }

    /// Apply rotation around Z axis
    pub async fn rz(&mut self, qubit: usize, angle: f64) -> Result<()> {
        self.backend
            .apply_single_gate(qubit, Gate1::Rz(angle))
            .await
    }

    /// Apply CNOT gate
    pub async fn cnot(&mut self, control: usize, target: usize) -> Result<()> {
        self.backend
            .apply_two_gate(control, target, Gate2::CNOT)
            .await
    }

    /// Apply CZ gate
    pub async fn cz(&mut self, control: usize, target: usize) -> Result<()> {
        self.backend
            .apply_two_gate(control, target, Gate2::CZ)
            .await
    }

    /// Apply SWAP gate
    pub async fn swap(&mut self, q1: usize, q2: usize) -> Result<()> {
        self.backend.apply_two_gate(q1, q2, Gate2::SWAP).await
    }

    /// Apply CY gate
    pub async fn cy(&mut self, control: usize, target: usize) -> Result<()> {
        self.backend
            .apply_two_gate(control, target, Gate2::CY)
            .await
    }

    /// Measure a qubit
    pub async fn measure(&mut self, qubit: usize) -> Result<u8> {
        self.backend.measure(qubit).await
    }

    /// Create Bell pair
    pub async fn create_bell_pair(&mut self, q1: usize, q2: usize) -> Result<()> {
        self.backend.create_entanglement(q1, q2).await
    }

    /// Apply Toffoli (CCX) gate - controlled-controlled-X
    /// Flips the target qubit if both control qubits are |1⟩
    pub async fn ccx(&mut self, control1: usize, control2: usize, target: usize) -> Result<()> {
        // Check if backend supports non-Clifford gates
        // For now, only allow CCX in state vector backend
        // Later we could add a capability query method to backends

        // Decomposition of Toffoli using standard gates
        self.h(target).await?;
        self.cnot(control2, target).await?;
        self.t_dag(target).await?;
        self.cnot(control1, target).await?;
        self.t(target).await?;
        self.cnot(control2, target).await?;
        self.t_dag(target).await?;
        self.cnot(control1, target).await?;
        self.t(control2).await?;
        self.t(target).await?;
        self.cnot(control1, control2).await?;
        self.h(target).await?;
        self.t(control1).await?;
        self.t_dag(control2).await?;
        self.cnot(control1, control2).await?;
        Ok(())
    }

    /// Get number of qubits
    pub fn qubit_count(&self) -> usize {
        self.backend.qubit_count()
    }

    /// Convert to a recording system that tracks operations for visualization
    pub fn with_recording(self) -> RecordingQuantumSystem<B> {
        RecordingQuantumSystem::new(self)
    }
}

/// A wrapper that records operations for visualization
pub struct RecordingQuantumSystem<B: QuantumBackend> {
    system: QuantumSystem<B>,
    recorder: CircuitRecorder,
}

impl<B: QuantumBackend> RecordingQuantumSystem<B> {
    pub fn new(system: QuantumSystem<B>) -> Self {
        let n_qubits = system.qubit_count();
        RecordingQuantumSystem {
            system,
            recorder: CircuitRecorder::new(n_qubits),
        }
    }

    /// Apply Hadamard gate
    pub async fn h(&mut self, qubit: usize) -> Result<()> {
        self.recorder.record_single(qubit, Gate1::H);
        self.system.h(qubit).await
    }

    /// Apply Pauli-X gate
    pub async fn x(&mut self, qubit: usize) -> Result<()> {
        self.recorder.record_single(qubit, Gate1::X);
        self.system.x(qubit).await
    }

    /// Apply S† gate (S dagger)
    pub async fn s_dag(&mut self, qubit: usize) -> Result<()> {
        self.recorder.record_single(qubit, Gate1::SDag);
        self.system.s_dag(qubit).await
    }

    /// Apply T† gate (T dagger)
    pub async fn t_dag(&mut self, qubit: usize) -> Result<()> {
        self.recorder.record_single(qubit, Gate1::TDag);
        self.system.t_dag(qubit).await
    }

    /// Apply Pauli-Y gate
    pub async fn y(&mut self, qubit: usize) -> Result<()> {
        self.recorder.record_single(qubit, Gate1::Y);
        self.system.y(qubit).await
    }

    /// Apply Pauli-Z gate
    pub async fn z(&mut self, qubit: usize) -> Result<()> {
        self.recorder.record_single(qubit, Gate1::Z);
        self.system.z(qubit).await
    }

    /// Apply S gate (phase)
    pub async fn s(&mut self, qubit: usize) -> Result<()> {
        self.recorder.record_single(qubit, Gate1::S);
        self.system.s(qubit).await
    }

    /// Apply T gate (π/8)
    pub async fn t(&mut self, qubit: usize) -> Result<()> {
        self.recorder.record_single(qubit, Gate1::T);
        self.system.t(qubit).await
    }

    /// Apply rotation around X axis
    pub async fn rx(&mut self, qubit: usize, angle: f64) -> Result<()> {
        self.recorder.record_single(qubit, Gate1::Rx(angle));
        self.system.rx(qubit, angle).await
    }

    /// Apply rotation around Y axis
    pub async fn ry(&mut self, qubit: usize, angle: f64) -> Result<()> {
        self.recorder.record_single(qubit, Gate1::Ry(angle));
        self.system.ry(qubit, angle).await
    }

    /// Apply rotation around Z axis
    pub async fn rz(&mut self, qubit: usize, angle: f64) -> Result<()> {
        self.recorder.record_single(qubit, Gate1::Rz(angle));
        self.system.rz(qubit, angle).await
    }

    /// Apply CNOT gate
    pub async fn cnot(&mut self, control: usize, target: usize) -> Result<()> {
        self.recorder.record_two(control, target, Gate2::CNOT);
        self.system.cnot(control, target).await
    }

    /// Apply CZ gate
    pub async fn cz(&mut self, control: usize, target: usize) -> Result<()> {
        self.recorder.record_two(control, target, Gate2::CZ);
        self.system.cz(control, target).await
    }

    /// Apply SWAP gate
    pub async fn swap(&mut self, q1: usize, q2: usize) -> Result<()> {
        self.recorder.record_two(q1, q2, Gate2::SWAP);
        self.system.swap(q1, q2).await
    }

    /// Apply CY gate
    pub async fn cy(&mut self, control: usize, target: usize) -> Result<()> {
        self.recorder.record_two(control, target, Gate2::CY);
        self.system.cy(control, target).await
    }

    /// Measure a qubit
    pub async fn measure(&mut self, qubit: usize) -> Result<u8> {
        self.recorder.record_measure(qubit);
        self.system.measure(qubit).await
    }

    /// Create Bell pair
    pub async fn create_bell_pair(&mut self, q1: usize, q2: usize) -> Result<()> {
        self.recorder.record_single(q1, Gate1::H);
        self.recorder.record_two(q1, q2, Gate2::CNOT);
        self.system.create_bell_pair(q1, q2).await
    }

    /// Apply Toffoli (CCX) gate
    pub async fn ccx(&mut self, control1: usize, control2: usize, target: usize) -> Result<()> {
        // For now, just record the decomposed gates
        self.system.ccx(control1, control2, target).await
    }

    /// Get number of qubits
    pub fn qubit_count(&self) -> usize {
        self.system.qubit_count()
    }

    /// Print the circuit to stdout
    pub fn print_circuit(&self) {
        println!("{}", self.recorder.to_ascii());
    }

    /// Get the circuit as a string
    pub fn circuit_string(&self) -> String {
        self.recorder.to_ascii()
    }
    pub fn to_qasm(&self) -> String {
        let mut qasm = String::new();

        // Header
        qasm.push_str("OPENQASM 3.0;\n");
        qasm.push_str("// Bit ordering: q[0] is the least significant bit\n");
        qasm.push_str("// Measurement results read as: c[n-1]...c[1]c[0]\n\n");

        // Include standard library for gates
        qasm.push_str("include \"stdgates.inc\";\n\n");

        // Declare quantum and classical registers
        qasm.push_str(&format!("qubit[{}] q;\n", self.system.qubit_count()));
        qasm.push_str(&format!("bit[{}] c;\n\n", self.system.qubit_count()));

        // Convert operations
        for op in &self.recorder.operations {
            match op {
                Operation::Single(qubit, gate) => {
                    let gate_str = match gate {
                        Gate1::H => format!("h q[{}];", qubit),
                        Gate1::X => format!("x q[{}];", qubit),
                        Gate1::Y => format!("y q[{}];", qubit),
                        Gate1::Z => format!("z q[{}];", qubit),
                        Gate1::S => format!("s q[{}];", qubit),
                        Gate1::SDag => format!("sdg q[{}];", qubit),
                        Gate1::T => format!("t q[{}];", qubit),
                        Gate1::TDag => format!("tdg q[{}];", qubit),
                        Gate1::Rx(angle) => {
                            format!("rx({}) q[{}];", Self::format_angle(*angle), qubit)
                        }
                        Gate1::Ry(angle) => {
                            format!("ry({}) q[{}];", Self::format_angle(*angle), qubit)
                        }
                        Gate1::Rz(angle) => {
                            format!("rz({}) q[{}];", Self::format_angle(*angle), qubit)
                        }
                    };
                    qasm.push_str(&gate_str);
                    qasm.push('\n');
                }
                Operation::Two(q1, q2, gate) => {
                    let gate_str = match gate {
                        Gate2::CNOT => format!("cx q[{}], q[{}];", q1, q2),
                        Gate2::CY => format!("cy q[{}], q[{}];", q1, q2),
                        Gate2::CZ => format!("cz q[{}], q[{}];", q1, q2),
                        Gate2::SWAP => format!("swap q[{}], q[{}];", q1, q2),
                    };
                    qasm.push_str(&gate_str);
                    qasm.push('\n');
                }
                Operation::Measure(qubit) => {
                    qasm.push_str(&format!("c[{}] = measure q[{}];\n", qubit, qubit));
                }
            }
        }

        qasm
    }

    fn format_angle(radians: f64) -> String {
        use std::f64::consts::PI;

        // Common π fractions and their string representations
        const FRACTIONS: &[(f64, &str)] = &[
            (0.0, "0"),
            (0.25, "pi/4"),
            (0.5, "pi/2"),
            (0.75, "3*pi/4"),
            (1.0, "pi"),
            (1.25, "5*pi/4"),
            (1.5, "3*pi/2"),
            (1.75, "7*pi/4"),
            (2.0, "2*pi"),
            (0.333333333, "pi/3"),
            (0.666666667, "2*pi/3"),
            (0.166666667, "pi/6"),
            (0.833333333, "5*pi/6"),
        ];

        // Normalize angle to [0, 2π)
        let normalized = radians.rem_euclid(2.0 * PI);
        let fraction = normalized / PI;

        // Check if it's close to a common fraction
        const EPSILON: f64 = 1e-9;
        for &(frac, repr) in FRACTIONS {
            if (fraction - frac).abs() < EPSILON {
                return repr.to_string();
            }
        }

        // For negative common angles
        let neg_fraction = (radians / PI).rem_euclid(2.0);
        if neg_fraction > 1.0 {
            for &(frac, repr) in FRACTIONS {
                if ((2.0 - neg_fraction) - frac).abs() < EPSILON {
                    return format!("-{}", repr);
                }
            }
        }

        // Fall back to numeric representation
        format!("{:.15}", radians)
    }

    /// Export to OpenQASM 2.0 format (for compatibility with older tools)
    pub fn to_qasm2(&self) -> String {
        let mut qasm = String::new();

        // Header
        qasm.push_str("OPENQASM 2.0;\n");
        qasm.push_str("include \"qelib1.inc\";\n\n");

        // Registers
        qasm.push_str(&format!("qreg q[{}];\n", self.system.qubit_count()));
        qasm.push_str(&format!("creg c[{}];\n\n", self.system.qubit_count()));

        // Convert operations
        for op in &self.recorder.operations {
            match op {
                Operation::Single(qubit, gate) => {
                    let gate_str = match gate {
                        Gate1::H => format!("h q[{}];", qubit),
                        Gate1::X => format!("x q[{}];", qubit),
                        Gate1::Y => format!("y q[{}];", qubit),
                        Gate1::Z => format!("z q[{}];", qubit),
                        Gate1::S => format!("s q[{}];", qubit),
                        Gate1::SDag => format!("sdg q[{}];", qubit),
                        Gate1::T => format!("t q[{}];", qubit),
                        Gate1::TDag => format!("tdg q[{}];", qubit),
                        Gate1::Rx(angle) => {
                            format!("rx({}) q[{}];", Self::format_angle(*angle), qubit)
                        }
                        Gate1::Ry(angle) => {
                            format!("ry({}) q[{}];", Self::format_angle(*angle), qubit)
                        }
                        Gate1::Rz(angle) => {
                            format!("rz({}) q[{}];", Self::format_angle(*angle), qubit)
                        }
                    };
                    qasm.push_str(&gate_str);
                    qasm.push('\n');
                }
                Operation::Two(q1, q2, gate) => {
                    let gate_str = match gate {
                        Gate2::CNOT => format!("cx q[{}],q[{}];", q1, q2),
                        Gate2::CY => format!("cy q[{}],q[{}];", q1, q2),
                        Gate2::CZ => format!("cz q[{}],q[{}];", q1, q2),
                        Gate2::SWAP => format!("swap q[{}],q[{}];", q1, q2),
                    };
                    qasm.push_str(&gate_str);
                    qasm.push('\n');
                }
                Operation::Measure(qubit) => {
                    qasm.push_str(&format!("measure q[{}] -> c[{}];\n", qubit, qubit));
                }
            }
        }

        qasm
    }
}
