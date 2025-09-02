use crate::builder::QuantumSystemBuilder;

pub mod algorithms;
pub mod backend;
pub mod builder;
pub mod error;
pub mod network;
pub mod physics;
pub mod protocol;
pub mod quantum;

pub mod types;
pub mod utils;

/// Creates a quantum system builder.
///
/// # Example
/// ```
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut q = qnect::create()
///     .with_qubits(2)
///     .build()?;
///     Ok(())
/// }
/// ```
pub fn create() -> QuantumSystemBuilder {
    QuantumSystemBuilder::new()
}
