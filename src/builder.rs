use crate::backend::{
    backend::QuantumBackend,
    network_backend::{NetworkBackend, NetworkTopology},
    state_vector::StateVectorBackend,
};
use crate::error::{QnectError, Result};
use crate::system::QuantumSystem;

/// Fluent API for building quantum systems
pub struct QuantumSystemBuilder {
    backend_type: BackendType,
    qubit_count: Option<usize>,
    topology: Option<NetworkTopology>,
}

#[derive(Debug, Clone)]
pub enum BackendType {
    StateVector,
    Network,
    Stabilizer,    // Future
    TensorNetwork, // Future
}

impl QuantumSystemBuilder {
    pub fn new() -> Self {
        QuantumSystemBuilder {
            backend_type: BackendType::StateVector,
            qubit_count: None,
            topology: None,
        }
    }

    pub fn with_qubits(mut self, n: usize) -> Self {
        self.qubit_count = Some(n);
        self
    }

    pub fn with_backend(mut self, backend: BackendType) -> Self {
        self.backend_type = backend;
        self
    }

    pub fn with_topology(mut self, topology: NetworkTopology) -> Self {
        self.topology = Some(topology);
        self
    }

    pub fn build(self) -> Result<QuantumSystem<Box<dyn QuantumBackend>>> {
        let backend: Box<dyn QuantumBackend> = match self.backend_type {
            BackendType::StateVector => {
                Box::new(StateVectorBackend::new(self.qubit_count.unwrap_or(2)))
            }
            BackendType::Network => Box::new(NetworkBackend::new(
                self.topology.unwrap_or(NetworkTopology::AllToAll),
            )),
            _ => {
                return Err(QnectError::backend_not_implemented(format!(
                    "{:?}",
                    self.backend_type
                )));
            }
        };

        Ok(QuantumSystem::new(backend))
    }
}

impl Default for QuantumSystemBuilder {
    fn default() -> Self {
        Self::new()
    }
}
