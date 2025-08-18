use std::fmt;

// Result type alias for cleaner signatures
pub type Result<T> = std::result::Result<T, QnectError>;

#[derive(Debug, Clone)]
pub enum QnectError {
    QubitOutOfRange {
        qubit: usize,
        max: usize,
    },
    InvalidGate {
        reason: String,
    },
    NetworkError {
        message: String,
    },
    BackendNotImplemented {
        backend: String,
    },
    MeasurementError {
        qubit: usize,
    },
    EntanglementError {
        q1: usize,
        q2: usize,
        reason: String,
    },
    InvalidOperation {
        operation: String,
        reason: String,
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
            Self::NetworkError { message } => {
                write!(f, "Network error: {}", message)
            }
            Self::BackendNotImplemented { backend } => {
                write!(f, "Backend '{}' not implemented yet", backend)
            }
            Self::MeasurementError { qubit } => {
                write!(f, "Failed to measure qubit {}", qubit)
            }
            Self::EntanglementError { q1, q2, reason } => {
                write!(f, "Cannot entangle qubits {} and {}: {}", q1, q2, reason)
            }
            Self::InvalidOperation { operation, reason } => {
                write!(f, "Invalid operation '{}': {}", operation, reason)
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
}
