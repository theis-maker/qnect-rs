use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetworkMessage {
    NodeJoin {
        name: String,
        node_type: NodeType,
        location: (f64, f64),
    },
    BB84Start {
        rounds: usize,
    },
    BB84Measure {
        round: usize,
        qubit_id: usize,
    },
    BB84Basis {
        round: usize,
        basis: bool,
        result: u8,
    },
    BB84KeyBits {
        count: usize,
    },
    BB84Use {
        round: usize,
    },
    BB84EndKey,
    EncryptedMessage {
        data: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum NodeType {
    Endpoint,
    Repeater,
}
