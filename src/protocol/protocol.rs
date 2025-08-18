use crate::{
    network::network::{NetworkError, QuantumNetwork},
    state::{Gate1Q, Gate2Q},
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
    ) -> Result<(), NetworkError> {
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
    ) -> Result<Vec<usize>, NetworkError> {
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
    use super::*;

    #[test]
    fn test_quantum_network() {
        let mut network = QuantumNetwork::new();

        // Add nodes
        network.add_node("alice", 2);
        network.add_node("bob", 2);

        // Create EPR pair
        let (a, b) = network.create_epr_pair("alice", "bob").unwrap();

        // Verify entanglement
        assert!(network.are_entangled(a, b));

        // Measure at Alice
        let _m1 = network.measure("alice", a).unwrap();

        // No longer entangled after measurement
        assert!(!network.are_entangled(a, b));
    }

    #[tokio::test]
    async fn test_ghz_distribution() {
        let mut network = QuantumNetwork::new();

        // Three-party network
        network.add_node("alice", 1);
        network.add_node("bob", 1);
        network.add_node("charlie", 1);

        // Create GHZ state
        let qubits = QuantumProtocol::distribute_ghz(&mut network, vec!["alice", "bob", "charlie"])
            .await
            .unwrap();

        // All should be entangled
        assert!(network.are_entangled(qubits[0], qubits[1]));
        assert!(network.are_entangled(qubits[1], qubits[2]));
    }
}
