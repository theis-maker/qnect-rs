use crate::error::{QnectError, Result};
use crate::quantum::system::QuantumSystem;
use crate::utils::qasm_parser::QasmOperation;
use crate::{
    backend::{
        backend::QuantumBackend,
        mock_qnpu::MockQnpuBackend,
        network_backend::{NetworkBackend, NetworkTopology},
        noisy::NoisyBackend,
        stabilizer::StabilizerBackend,
        state_vector::StateVectorBackend,
    },
    utils::qasm_parser::parse_qasm,
};

/// Fluent API for building quantum systems
pub struct QuantumSystemBuilder {
    backend_type: BackendType,
    qubit_count: Option<usize>,
    topology: Option<NetworkTopology>,
    noise_model: Option<NoiseModel>,
}

#[derive(Debug, Clone)]
pub enum BackendType {
    StateVector,
    Network,
    Stabilizer,
    MockQnpu { endpoint: String, node_id: String },
    TensorNetwork, // Future
}

#[derive(Debug, Clone)]
pub struct NoiseModel {
    pub depolarizing_rate: f64, // Probability of error per gate
    pub measurement_error: f64, // Probability of measurement flip
}

impl QuantumSystemBuilder {
    pub fn new() -> Self {
        QuantumSystemBuilder {
            backend_type: BackendType::StateVector,
            qubit_count: None,
            topology: None,
            noise_model: None,
        }
    }

    pub fn with_noise(mut self, noise_model: NoiseModel) -> Self {
        self.noise_model = Some(noise_model);
        self
    }

    pub fn with_qubits(mut self, n: usize) -> Self {
        self.qubit_count = Some(n);
        self
    }

    pub fn with_backend(mut self, backend: BackendType) -> Self {
        self.backend_type = backend;
        self
    }

    pub fn with_topology(mut self, topology: NetworkTopology) -> Self {
        self.topology = Some(topology);
        self
    }

    /// Create a quantum system builder from a QASM string
    pub fn from_qasm(mut self, qasm: &str) -> Result<Self> {
        let program = parse_qasm(qasm)?;

        // Modify the existing builder instead of creating a new one
        self.qubit_count = Some(program.num_qubits);
        Ok(self)
    }

    pub fn build(self) -> Result<QuantumSystem<Box<dyn QuantumBackend>>> {
        let mut backend: Box<dyn QuantumBackend> = match self.backend_type {
            BackendType::StateVector => {
                Box::new(StateVectorBackend::new(self.qubit_count.unwrap_or(2)))
            }
            BackendType::Network => Box::new(NetworkBackend::new(
                self.topology.unwrap_or(NetworkTopology::AllToAll),
            )),
            BackendType::Stabilizer => {
                Box::new(StabilizerBackend::new(self.qubit_count.unwrap_or(2)))
            }
            BackendType::MockQnpu { endpoint, node_id } => {
                let backend = MockQnpuBackend::new(
                    endpoint.clone(),
                    node_id.clone(),
                    self.qubit_count.unwrap_or(2),
                );
                Box::new(backend) as Box<dyn QuantumBackend>
            }
            _ => {
                return Err(QnectError::backend_not_implemented(format!(
                    "{:?}",
                    self.backend_type
                )));
            }
        };

        // Wrap in noise if requested
        if let Some(noise) = self.noise_model {
            backend = Box::new(NoisyBackend::new(
                backend,
                noise.depolarizing_rate,
                noise.measurement_error,
            ));
        }

        Ok(QuantumSystem::new(backend))
    }
}

impl Default for QuantumSystemBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Execute a parsed QASM program on a quantum system
pub async fn execute_qasm<B: QuantumBackend>(
    system: &mut QuantumSystem<B>,
    qasm: &str,
) -> Result<Vec<u8>> {
    let program = parse_qasm(qasm)?;
    let mut measurements = vec![0u8; program.num_bits];

    for op in program.operations {
        match op {
            QasmOperation::H(q) => system.h(q).await?,
            QasmOperation::X(q) => system.x(q).await?,
            QasmOperation::Y(q) => system.y(q).await?,
            QasmOperation::Z(q) => system.z(q).await?,
            QasmOperation::S(q) => system.s(q).await?,
            QasmOperation::SDag(q) => system.s_dag(q).await?,
            QasmOperation::T(q) => system.t(q).await?,
            QasmOperation::TDag(q) => system.t_dag(q).await?,
            QasmOperation::Rx(q, angle) => system.rx(q, angle).await?,
            QasmOperation::Ry(q, angle) => system.ry(q, angle).await?,
            QasmOperation::Rz(q, angle) => system.rz(q, angle).await?,
            QasmOperation::CX(q1, q2) => system.cnot(q1, q2).await?,
            QasmOperation::CY(q1, q2) => system.cy(q1, q2).await?,
            QasmOperation::CZ(q1, q2) => system.cz(q1, q2).await?,
            QasmOperation::Swap(q1, q2) => system.swap(q1, q2).await?,
            QasmOperation::Measure(qubit, bit) => {
                measurements[bit] = system.measure(qubit).await?;
            }
        }
    }

    Ok(measurements)
}
