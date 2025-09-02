use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum QuantumMessage {
    // Node discovery
    NodeAnnounce {
        node_id: String,
        location: String,
        qubit_count: usize,
        capabilities: Vec<String>,
    },

    // Entanglement management
    RequestEntanglement {
        request_id: Uuid,
        from: String,
        to: String,
        num_pairs: usize,
    },

    EntanglementReady {
        request_id: Uuid,
        pairs: Vec<(usize, usize)>, // (local_qubit, remote_qubit)
    },

    // Teleportation
    TeleportationCorrections {
        target_node: String,
        target_qubit: usize,
        m1: u8,
        m2: u8,
    },

    // Generic classical data
    ClassicalData {
        from: String,
        to: String,
        data: Vec<u8>,
    },
}
