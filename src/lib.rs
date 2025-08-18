use crate::builder::QuantumSystemBuilder;

pub mod backend;
pub mod builder;
pub mod circuit_viz;
pub mod entanglement;
pub mod error;
pub mod network;
pub mod protocol;
pub mod state;
pub mod system;
pub mod types;

// Convenience function
pub fn create() -> QuantumSystemBuilder {
    QuantumSystemBuilder::new()
}
