use crate::{
    backend::backend::{Gate1, Gate2, QuantumBackend},
    circuit_viz::CircuitRecorder,
    error::Result,
};

/// High-level quantum system API that stays stable
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
}
