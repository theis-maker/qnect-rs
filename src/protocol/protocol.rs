use crate::{
    error::Result,
    network::network::QuantumNetwork,
    quantum::state::{Gate1Q, Gate2Q},
};

/// High-level quantum network protocols
pub struct QuantumProtocol;

impl QuantumProtocol {
    /// Teleport a qubit from one node to another using pre-shared entanglement
    pub async fn teleport(
        network: &mut QuantumNetwork,
        from_node: &str,
        to_node: &str,
        _qubit_to_teleport: usize,
    ) -> Result<()> {
        // Create EPR pair
        let (_alice_epr, _bob_epr) = network.create_epr_pair(from_node, to_node)?;

        // Alice performs Bell measurement
        // (In real implementation, this would be Bell basis measurement)
        // For now, simplified CNOT + H + measure

        // ... teleportation protocol ...

        Ok(())
    }

    /// Distribute entanglement across multiple nodes (GHZ state)
    pub async fn distribute_ghz(
        network: &mut QuantumNetwork,
        nodes: Vec<&str>,
    ) -> Result<Vec<usize>> {
        if nodes.len() < 2 {
            return Ok(vec![]);
        }

        // Get one qubit from each node
        let mut qubits = Vec::new();
        for node in &nodes {
            qubits.push(network.get_free_qubit(node)?);
        }

        // Create GHZ state
        // H on first qubit
        network.state.apply_single_qubit_gate(qubits[0], Gate1Q::H);

        // CNOT cascade
        for i in 1..qubits.len() {
            network
                .state
                .apply_two_qubit_gate(qubits[0], qubits[i], Gate2Q::CNOT);
        }

        // Track entanglement
        for i in 1..qubits.len() {
            network.entanglement.entangle(qubits[0], qubits[i]);
        }

        Ok(qubits)
    }
}

// Example usage showing the clean API
#[cfg(test)]
mod tests {
    use crate::{builder::BackendType, network::network::LinkType};

    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_quantum_network() {
        let mut network = QuantumNetwork::new_distributed();
        network
            .add_distributed_node("Alice", 2, BackendType::StateVector)
            .unwrap();
        network
            .add_distributed_node("Bob", 2, BackendType::StateVector)
            .unwrap();

        network
            .add_quantum_link(
                "Alice",
                "Bob",
                LinkType::Fiber {
                    length_km: 10.0,
                    loss_db_per_km: 0.2,
                },
                0.98,
                1000.0,
            )
            .unwrap();

        let (q1, q2) = network.create_epr_pair("Alice", "Bob").unwrap();

        // Check nonlocal Bell pair registration instead of entanglement_registry
        assert!(network.is_bell_qubit("Alice", q1).is_some());
        assert!(network.is_bell_qubit("Bob", q2).is_some());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_ghz_distribution() {
        let mut network = QuantumNetwork::new_distributed();
        network
            .add_distributed_node("Alice", 4, BackendType::StateVector) // Increased from 2 to 4
            .unwrap();
        network
            .add_distributed_node("Bob", 4, BackendType::StateVector) // Increased from 2 to 4
            .unwrap();
        network
            .add_distributed_node("Charlie", 4, BackendType::StateVector) // Increased from 2 to 4
            .unwrap();

        // Create full mesh for GHZ
        network
            .add_quantum_link(
                "Alice",
                "Bob",
                LinkType::Fiber {
                    length_km: 10.0,
                    loss_db_per_km: 0.2,
                },
                0.98,
                1000.0,
            )
            .unwrap();
        network
            .add_quantum_link(
                "Alice",
                "Charlie",
                LinkType::Fiber {
                    length_km: 10.0,
                    loss_db_per_km: 0.2,
                },
                0.98,
                1000.0,
            )
            .unwrap();
        network
            .add_quantum_link(
                "Bob",
                "Charlie",
                LinkType::Fiber {
                    length_km: 10.0,
                    loss_db_per_km: 0.2,
                },
                0.98,
                1000.0,
            )
            .unwrap();

        let ghz_qubits = network
            .create_distributed_ghz(vec!["Alice", "Bob", "Charlie"])
            .await
            .unwrap();
        assert_eq!(ghz_qubits.len(), 3);
    }
}
