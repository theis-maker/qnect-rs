use crate::{
    backend::backend::{Gate1, Gate2, QuantumBackend},
    error::{QnectError, Result},
};
use async_trait::async_trait;
use std::collections::HashMap;

/// Network-aware quantum backend (placeholder for future implementation)
///
/// This backend will enable distributed quantum computing across multiple nodes.
/// Currently returns `BackendNotImplemented` errors for all operations.
pub struct NetworkBackend {
    _topology: NetworkTopology,
    _local_backends: HashMap<String, Box<dyn QuantumBackend>>,
    _routing: RoutingProtocol,
    total_qubits: usize,
}

#[derive(Debug, Clone)]
pub enum NetworkTopology {
    AllToAll,
    Linear,
    Star {
        center: String,
    },
    Mesh {
        connections: HashMap<String, Vec<String>>,
    },
}

#[derive(Debug, Clone)]
pub enum RoutingProtocol {
    Direct,
    EntanglementSwapping,
    Teleportation,
}

impl NetworkBackend {
    pub fn new(topology: NetworkTopology) -> Self {
        NetworkBackend {
            _topology: topology,
            _local_backends: HashMap::new(),
            _routing: RoutingProtocol::Direct,
            total_qubits: 0,
        }
    }
}

#[async_trait]
impl QuantumBackend for NetworkBackend {
    async fn apply_single_gate(&mut self, _qubit: usize, _gate: Gate1) -> Result<()> {
        // TODO: Route to correct node and apply
        Err(QnectError::backend_not_implemented("NetworkBackend"))
    }

    async fn apply_two_gate(&mut self, _q1: usize, _q2: usize, _gate: Gate2) -> Result<()> {
        // TODO: Check if qubits are on same node or need distributed operation
        Err(QnectError::backend_not_implemented("NetworkBackend"))
    }

    async fn measure(&mut self, _qubit: usize) -> Result<u8> {
        // TODO: Route to correct node
        Err(QnectError::backend_not_implemented("NetworkBackend"))
    }

    async fn create_entanglement(&mut self, _q1: usize, _q2: usize) -> Result<()> {
        // TODO: Distributed entanglement generation
        Err(QnectError::backend_not_implemented("NetworkBackend"))
    }

    fn qubit_count(&self) -> usize {
        self.total_qubits
    }
}
