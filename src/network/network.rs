use crate::entanglement::EntanglementTracker;
use crate::state::{Gate1Q, Gate2Q, QuantumState};
use std::collections::HashMap;

/// Represents a quantum network node
#[derive(Debug)]
pub struct QuantumNode {
    /// Node identifier
    pub id: String,
    /// Qubits owned by this node
    pub qubits: Vec<usize>,
    /// Local quantum memory/registers
    pub local_state: Option<QuantumState>,
}

impl QuantumNode {
    pub fn new(id: impl Into<String>) -> Self {
        QuantumNode {
            id: id.into(),
            qubits: Vec::new(),
            local_state: None,
        }
    }

    /// Allocate qubits to this node
    pub fn allocate_qubits(&mut self, start_idx: usize, count: usize) {
        self.qubits.extend(start_idx..start_idx + count);
    }
}

/// A quantum network with multiple nodes
pub struct QuantumNetwork {
    /// Global quantum state
    pub state: QuantumState,
    /// Network nodes
    pub nodes: HashMap<String, QuantumNode>,
    /// Tracks entanglement
    pub entanglement: EntanglementTracker,
    /// Total qubits in the network
    pub total_qubits: usize,
    /// Maps qubit index to node ID
    pub qubit_ownership: HashMap<usize, String>,
}

impl QuantumNetwork {
    /// Create a new quantum network
    pub fn new() -> Self {
        QuantumNetwork {
            state: QuantumState::zeros(0),
            nodes: HashMap::new(),
            entanglement: EntanglementTracker::new(0),
            total_qubits: 0,
            qubit_ownership: HashMap::new(),
        }
    }

    /// Add a node with a specified number of qubits
    pub fn add_node(&mut self, id: impl Into<String>, n_qubits: usize) -> &mut QuantumNode {
        let id = id.into();
        let start_idx = self.total_qubits;

        // Expand global state
        let new_total = self.total_qubits + n_qubits;
        self.state = QuantumState::zeros(new_total);
        self.entanglement = EntanglementTracker::new(new_total);

        // Create node
        let mut node = QuantumNode::new(id.clone());
        node.allocate_qubits(start_idx, n_qubits);

        // Track ownership
        for i in start_idx..start_idx + n_qubits {
            self.qubit_ownership.insert(i, id.clone());
        }

        self.total_qubits = new_total;
        self.nodes.insert(id.clone(), node);
        self.nodes.get_mut(&id).unwrap()
    }

    /// Create an EPR pair between two nodes
    pub fn create_epr_pair(
        &mut self,
        node1: &str,
        node2: &str,
    ) -> Result<(usize, usize), NetworkError> {
        // Get free qubits from each node
        let q1 = self.get_free_qubit(node1)?;
        let q2 = self.get_free_qubit(node2)?;

        // Create Bell pair
        self.state.apply_single_qubit_gate(q1, Gate1Q::H);
        self.state.apply_two_qubit_gate(q1, q2, Gate2Q::CNOT);

        // Track entanglement
        self.entanglement.entangle(q1, q2);

        Ok((q1, q2))
    }

    /// Get a free qubit from a node (simplified - just returns first)
    pub fn get_free_qubit(&self, node_id: &str) -> Result<usize, NetworkError> {
        self.nodes
            .get(node_id)
            .and_then(|n| n.qubits.first())
            .copied()
            .ok_or(NetworkError::NoFreeQubits)
    }

    /// Apply a local gate on a node's qubit
    pub fn apply_local_gate(
        &mut self,
        node_id: &str,
        qubit_idx: usize,
        gate: Gate1Q,
    ) -> Result<(), NetworkError> {
        // Verify ownership
        if self.qubit_ownership.get(&qubit_idx) != Some(&node_id.to_string()) {
            return Err(NetworkError::QubitNotOwned);
        }

        self.state.apply_single_qubit_gate(qubit_idx, gate);
        Ok(())
    }

    /// Measure a qubit at a node
    pub fn measure(&mut self, node_id: &str, qubit_idx: usize) -> Result<u8, NetworkError> {
        // Verify ownership
        if self.qubit_ownership.get(&qubit_idx) != Some(&node_id.to_string()) {
            return Err(NetworkError::QubitNotOwned);
        }

        let result = self.state.measure(qubit_idx);
        self.entanglement.measure(qubit_idx);
        Ok(result)
    }

    /// Check if two qubits are entangled
    pub fn are_entangled(&self, q1: usize, q2: usize) -> bool {
        self.entanglement.are_entangled(q1, q2)
    }
}

#[derive(Debug)]
pub enum NetworkError {
    NodeNotFound,
    NoFreeQubits,
    QubitNotOwned,
}
