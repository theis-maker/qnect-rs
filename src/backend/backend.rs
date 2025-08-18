use crate::error::Result;
use async_trait::async_trait;

/// Core quantum operations that any backend must support
#[async_trait]
pub trait QuantumBackend: Send + Sync {
    /// Apply a single-qubit gate
    async fn apply_single_gate(&mut self, qubit: usize, gate: Gate1) -> Result<()>;

    /// Apply a two-qubit gate
    async fn apply_two_gate(&mut self, q1: usize, q2: usize, gate: Gate2) -> Result<()>;

    /// Measure a qubit, collapsing its state
    async fn measure(&mut self, qubit: usize) -> Result<u8>;

    /// Create entanglement between two qubits
    async fn create_entanglement(&mut self, q1: usize, q2: usize) -> Result<()>;

    /// Get number of qubits
    fn qubit_count(&self) -> usize;
}

// Implement QuantumBackend for Box<dyn QuantumBackend>
#[async_trait]
impl QuantumBackend for Box<dyn QuantumBackend> {
    async fn apply_single_gate(&mut self, qubit: usize, gate: Gate1) -> Result<()> {
        (**self).apply_single_gate(qubit, gate).await
    }

    async fn apply_two_gate(&mut self, q1: usize, q2: usize, gate: Gate2) -> Result<()> {
        (**self).apply_two_gate(q1, q2, gate).await
    }

    async fn measure(&mut self, qubit: usize) -> Result<u8> {
        (**self).measure(qubit).await
    }

    async fn create_entanglement(&mut self, q1: usize, q2: usize) -> Result<()> {
        (**self).create_entanglement(q1, q2).await
    }

    fn qubit_count(&self) -> usize {
        (**self).qubit_count()
    }
}

/// Single-qubit gates
#[derive(Debug, Clone, Copy)]
pub enum Gate1 {
    H,
    X,
    Y,
    Z,
    S,
    T,
    Rx(f64),
    Ry(f64),
    Rz(f64),
}

/// Two-qubit gates
#[derive(Debug, Clone, Copy)]
pub enum Gate2 {
    CNOT,
    CZ,
    SWAP,
    CY,
}
