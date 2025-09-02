use std::fmt;

pub type Result<T> = std::result::Result<T, QnectError>;

#[derive(Debug, Clone)]
pub enum QnectError {
    // Quantum circuit errors
    QubitOutOfRange {
        qubit: usize,
        max: usize,
    },
    InvalidGate {
        reason: String,
    },
    MeasurementError {
        qubit: usize,
    },
    EntanglementError {
        q1: usize,
        q2: usize,
        reason: String,
    },

    // Resource errors
    InsufficientQubits,
    NoFreeQubits,
    QubitNotOwned {
        node: String,
        qubit: usize,
    },
    QubitNotAllocated {
        qubit: usize,
    },

    // Network errors
    NodeNotFound {
        node: String,
    },
    NoConnection {
        from: String,
        to: String,
    },
    ConnectionClosed,
    SendFailed,
    EmptyMessage,
    InsufficientNodes,

    // Physical constraints
    FidelityTooLow {
        value: f64,
        minimum: f64,
    },
    InvalidFidelity {
        value: f64,
    },
    InvalidGenerationRate {
        value: f64,
    },

    // Operation errors
    InvalidOperation {
        operation: String,
        reason: String,
    },
    GateApplicationFailed {
        gate: String,
        reason: String,
    },
    MeasurementFailed,
    NoLocalSystem {
        node: String,
    },

    // Backend errors
    BackendNotImplemented {
        backend: String,
    },

    // Feature not implemented
    NotImplemented {
        feature: String,
    },

    // Generic network error for migration
    NetworkError {
        message: String,
    },
}

impl fmt::Display for QnectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::QubitOutOfRange { qubit, max } => {
                write!(f, "Qubit {} out of range (max: {})", qubit, max)
            }
            Self::InvalidGate { reason } => {
                write!(f, "Invalid gate operation: {}", reason)
            }
            Self::MeasurementError { qubit } => {
                write!(f, "Failed to measure qubit {}", qubit)
            }
            Self::EntanglementError { q1, q2, reason } => {
                write!(f, "Cannot entangle qubits {} and {}: {}", q1, q2, reason)
            }
            Self::InsufficientQubits => {
                write!(f, "Insufficient qubits available")
            }
            Self::NoFreeQubits => {
                write!(f, "No free qubits available")
            }
            Self::QubitNotOwned { node, qubit } => {
                write!(f, "Node {} does not own qubit {}", node, qubit)
            }
            Self::QubitNotAllocated { qubit } => {
                write!(f, "Qubit {} not allocated", qubit)
            }
            Self::NodeNotFound { node } => {
                write!(f, "Node '{}' not found in network", node)
            }
            Self::NoConnection { from, to } => {
                write!(f, "No connection between '{}' and '{}'", from, to)
            }
            Self::ConnectionClosed => {
                write!(f, "Connection closed")
            }
            Self::SendFailed => {
                write!(f, "Failed to send message")
            }
            Self::EmptyMessage => {
                write!(f, "Empty message received")
            }
            Self::InsufficientNodes => {
                write!(f, "Insufficient nodes for operation")
            }
            Self::FidelityTooLow { value, minimum } => {
                write!(f, "Fidelity {:.3} below minimum {:.3}", value, minimum)
            }
            Self::InvalidFidelity { value } => {
                write!(f, "Invalid fidelity {:.3} (must be 0.0-1.0)", value)
            }
            Self::InvalidGenerationRate { value } => {
                write!(
                    f,
                    "Invalid generation rate {} (must be non-negative)",
                    value
                )
            }
            Self::InvalidOperation { operation, reason } => {
                write!(f, "Invalid operation '{}': {}", operation, reason)
            }
            Self::GateApplicationFailed { gate, reason } => {
                write!(f, "Failed to apply gate '{}': {}", gate, reason)
            }
            Self::MeasurementFailed => {
                write!(f, "Measurement failed")
            }
            Self::NoLocalSystem { node } => {
                write!(f, "Node '{}' has no local quantum system", node)
            }
            Self::BackendNotImplemented { backend } => {
                write!(f, "Backend '{}' not implemented", backend)
            }
            Self::NotImplemented { feature } => {
                write!(f, "{} not yet implemented", feature)
            }
            Self::NetworkError { message } => {
                write!(f, "Network error: {}", message)
            }
        }
    }
}

impl std::error::Error for QnectError {}

// Convenience constructors
impl QnectError {
    pub fn qubit_out_of_range(qubit: usize, max: usize) -> Self {
        Self::QubitOutOfRange { qubit, max }
    }

    pub fn node_not_found(node: impl Into<String>) -> Self {
        Self::NodeNotFound { node: node.into() }
    }

    pub fn no_connection(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self::NoConnection {
            from: from.into(),
            to: to.into(),
        }
    }

    pub fn invalid_operation(operation: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidOperation {
            operation: operation.into(),
            reason: reason.into(),
        }
    }

    pub fn invalid_gate(reason: impl Into<String>) -> Self {
        Self::InvalidGate {
            reason: reason.into(),
        }
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self::NetworkError {
            message: message.into(),
        }
    }

    pub fn backend_not_implemented(backend: impl Into<String>) -> Self {
        Self::BackendNotImplemented {
            backend: backend.into(),
        }
    }

    pub fn not_implemented(feature: impl Into<String>) -> Self {
        Self::NotImplemented {
            feature: feature.into(),
        }
    }
}

// // Complete mapping from NetworkError to QnectError
// impl From<crate::network::network::NetworkError> for QnectError {
//     fn from(err: crate::network::network::NetworkError) -> Self {
//         use crate::network::network::NetworkError;

//         match err {
//             NetworkError::NodeNotFound => Self::NodeNotFound {
//                 node: "unknown".into(),
//             },
//             NetworkError::NoFreeQubits => Self::NoFreeQubits,
//             NetworkError::QubitNotOwned => Self::QubitNotOwned {
//                 node: "unknown".into(),
//                 qubit: 0,
//             },
//             NetworkError::InvalidOperation => Self::InvalidOperation {
//                 operation: "unknown".into(),
//                 reason: "invalid operation".into(),
//             },
//             NetworkError::NoConnection => Self::NoConnection {
//                 from: "unknown".into(),
//                 to: "unknown".into(),
//             },
//             NetworkError::FidelityTooLow => Self::FidelityTooLow {
//                 value: 0.0,
//                 minimum: 0.5,
//             },
//             NetworkError::SendFailed => Self::SendFailed,
//             NetworkError::ConnectionClosed => Self::ConnectionClosed,
//             NetworkError::EmptyMessage => Self::EmptyMessage,
//             NetworkError::InsufficientNodes => Self::InsufficientNodes,
//             NetworkError::InvalidFidelity => Self::InvalidFidelity { value: -1.0 },
//             NetworkError::InvalidGenerationRate => Self::InvalidGenerationRate { value: -1.0 },
//             NetworkError::QubitNotAllocated => Self::QubitNotAllocated { qubit: 0 },
//             NetworkError::GateApplicationFailed => Self::GateApplicationFailed {
//                 gate: "unknown".into(),
//                 reason: "application failed".into(),
//             },
//             NetworkError::MeasurementFailed => Self::MeasurementFailed,
//             NetworkError::NoLocalSystem => Self::NoLocalSystem {
//                 node: "unknown".into(),
//             },
//             NetworkError::DistributedGHZNotImplemented => Self::NotImplemented {
//                 feature: "Distributed GHZ".into(),
//             },
//         }
//     }
// }
