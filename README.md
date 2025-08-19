# Qnect 🔗

[![Crates.io](https://img.shields.io/crates/v/qnect.svg)](https://crates.io/crates/qnect)
[![Docs.rs](https://docs.rs/qnect/badge.svg)](https://docs.rs/qnect)
[![License](https://img.shields.io/crates/l/qnect)](./LICENSE)

**Quantum computing in Rust: from Bell pairs to distributed quantum networks.**

**No config files. No magic. Just quantum.**

Build quantum circuits, simulate massive systems, and create quantum networks - all with one powerful framework.


```rust
use qnect::{create, builder::BackendType};
use qnect::network::QuantumNetwork;

// Start simple...
let mut q = create().with_qubits(2).build()?;
q.h(0).await?;
q.cnot(0, 1).await?;

// ...or scale to 5000 qubits?
let mut q = create()
    .with_backend(BackendType::Stabilizer)
    .with_qubits(5000)
    .build()?;

// ...or build quantum networks!
let mut network = QuantumNetwork::new_distributed();
network.add_node("Alice", 4);
network.add_node("Bob", 4);
```

## Features

- 🚀 **Fast** - 100,000 gates/second in pure Rust
- 📈 **Scalable** - Same API from 2 to 5000+ qubits
- 🌐 **Networked** - Multi-hop entanglement, NetQASM generation
- 🔬 **Research-grade** - Anonymous protocols, blind computation, verified physics
- 🎯 **Zero config** - No setup files, just `cargo run`
- 📊 **Realistic noise** - Depolarizing and measurement errors for accurate simulation

## Installation

```toml
[dependencies]
qnect = "0.2.0"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

### Your First Bell Pair (The Quantum = Hello World!)

```rust
use qnect::create;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut q = create().with_qubits(2).build()?;
    q.h(0).await?;        // Superposition
    q.cnot(0, 1).await?;  // Entanglement
    let (m0, m1) = (q.measure(0).await?, q.measure(1).await?);
    println!("Bell pair: |{}{}⟩", m0, m1);  // Always 00 or 11
    Ok(())
}
```

### Try It Now

```bash
# See quantum mechanics in action
cargo run --example 03_simple_bell_pair
cargo run --example 00_quantum_verification  # Verify Bell inequalities!
```

## Key Examples

### Quantum Teleportation

```rust
use qnect::create;
use std::f64::consts::PI;

let mut q = create().with_qubits(3).build()?;

// Alice's qubit to teleport
q.ry(0, PI/3.0).await?;

// Shared Bell pair
q.h(1).await?;
q.cnot(1, 2).await?;

// Teleportation protocol
q.cnot(0, 1).await?;
q.h(0).await?;
let (m0, m1) = (q.measure(0).await?, q.measure(1).await?);

// Bob's corrections
if m1 == 1 { q.x(2).await?; }
if m0 == 1 { q.z(2).await?; }
// State transferred to qubit 2!
```

### Scale to 5000 Qubits

```rust
use qnect::{create, builder::BackendType};

// Switch backends, same API
let mut q = create()
    .with_backend(BackendType::Stabilizer)
    .with_qubits(5000)
    .build()?;

// Create massive GHZ state
q.h(0).await?;
for i in 1..5000 {
    q.cnot(0, i).await?;
}
// |00000...⟩ + |11111...⟩ with only 12MB RAM!
```

### Realistic Noise Modeling

```rust
use qnect::{create, builder::{BackendType, NoiseModel}};

// Add realistic quantum errors
let mut q = create()
    .with_backend(BackendType::StateVector)
    .with_qubits(5)
    .with_noise(NoiseModel {
        depolarizing_rate: 0.01, // 1% gate error
        measurement_error: 0.001  // 0.1% measurement error
    })
    .build()?;

// Noise model includes:
// - Depolarizing noise after gates (random Pauli errors)
// - Measurement bit-flip errors
// - Two-qubit correlated errors (2x error rate)
// - Proper measurement caching
```

### Quantum Networks

```rust
use qnect::network::{QuantumNetwork, LinkType};

let mut network = QuantumNetwork::new_distributed();

// Add quantum nodes
network.add_node("Alice", 4);
network.add_node("Repeater", 4);
network.add_node("Bob", 4);

// Connect with realistic links
network.add_quantum_link("Alice", "Repeater",
    LinkType::Fiber { length_km: 50.0, loss_db_per_km: 0.2 },
    0.95, 1000.0)?;

// Multi-hop entanglement
let (q1, q2) = network
    .establish_end_to_end_entanglement("Alice", "Bob")
    .await?;
```

## Advanced Protocols

### Anonymous Quantum Communication

Based on [Christandl & Wehner 2004](https://arxiv.org/abs/quant-ph/0409201) - impossible classically!

```rust
use qnect::network::{QuantumNetwork, LinkType};

// Create network with all participants
let participants = vec!["Alice", "Bob", "Charlie", "David", "Eve"];
let mut network = QuantumNetwork::new_distributed();

// Add all nodes
for p in &participants {
    network.add_node(p, 8);
}

// Create full mesh connectivity
network.add_multiparty_link(
    participants.clone(),
    LinkType::Fiber { length_km: 0.1, loss_db_per_km: 0.1 },
    0.99, 10000.0
)?;

// Anonymous bit transmission
let parity = network
    .anonymous_transmission("Alice", participants.clone(), 1)
    .await?;
// Everyone knows bit 1 was sent, but not that Alice sent it!

// Anonymous entanglement
let (q1, q2) = network
    .anonymous_entanglement("Alice", "Bob", participants.clone())
    .await?;
// Alice and Bob share entanglement, untraceable by others
```

### Blind Quantum Computing (UBQC)

```rust
use qnect::network::{QuantumNetwork, BlindComputationPattern};

// Client delegates computation without revealing data
let pattern = BlindComputationPattern {
    computation_graph: vec![(0, 1), (1, 2)],
    measurement_angles: vec![0.5, 1.0, 1.5],  // Encrypted
    flow: vec![0, 1, 2],
};

let mut network = QuantumNetwork::new_distributed();
network.add_node("Client", 4);
network.add_node("Server", 8);

let results = network
    .blind_computation_ubqc("Client", "Server", pattern)
    .await?;
// Server performed computation but learned nothing!
```

## Architecture

```rust
// Unified backend trait - same for local and distributed
#[async_trait]
pub trait QuantumBackend: Send + Sync {
    async fn apply_single_gate(&mut self, qubit: usize, gate: Gate1) -> Result<()>;
    async fn apply_two_gate(&mut self, q1: usize, q2: usize, gate: Gate2) -> Result<()>;
    async fn measure(&mut self, qubit: usize) -> Result<u8>;
}
```

**Available Backends:**
- `StateVector` - Exact simulation (≤30 qubits)
- `Stabilizer` - Clifford circuits (5000+ qubits, O(n²) memory)
- `MockQnpu` - Hardware API testing
- `Noisy` - Wraps any backend with "realistic" (it's a start) errors
- Future: IBMQ, IonQ, QuTech

## All Examples

```bash
# 👋 Start Here
cargo run --example 03_simple_bell_pair     # Your first quantum program
cargo run --example 00_quantum_verification  # See Bell inequality violation

# 🎓 Learn Quantum
cargo run --example 04_quantum_teleportation
cargo run --example 17_grovers_search       # Grover's algorithm
cargo run --example 18_stabilizer_demo      # 5000 qubits!
cargo run --example 15_noise_models         # Realistic quantum noise

# 🌐 Quantum Networks
cargo run --example 19_quantum_network      # Network basics
cargo run --example 20_network_protocols    # Teleportation, GHZ, blind computation
cargo run --example 22_quantum_anonymous_transmission  # Anonymous protocols

# 🏭 Production Ready
cargo run --example 21_hardware_ready       # NetQASM generation demo
cargo run --example 16_qasm_import          # OpenQASM 2.0/3.0 support
```

## Performance

| Backend | Max Qubits | Memory | Speed | Use Case |
|---------|------------|---------|--------|----------|
| StateVector | ~30 | O(2ⁿ) | Exact | Research, small circuits |
| **Stabilizer** | **5000+** | **O(n²)** | **135k gates/sec** | **Error correction, large circuits** |
| Noisy | Same as wrapped | Same as wrapped | ~5% overhead | Error modeling, noisy simulation |
| MockQnpu | Hardware | Minimal | Network-limited | Hardware testing |

## Why Qnect?

**For Beginners:**
- Start with 5 lines of code
- All the gates you need
- Clear error messages
- Visual circuit diagrams
- No configuration needed

**For Researchers:**
- Anonymous quantum protocols
- Blind computation (UBQC)
- Distributed GHZ states
- NetQASM generation
- Realistic noise models

**For Production:**
- Type-safe Rust
- Async/await design
- Resource tracking
- Hardware-ready output

## Roadmap

✅ **Released**
- Quantum simulators (state vector, stabilizer)
- Realistic noise models (depolarizing, measurement errors)
- Quantum networks with routing
- Anonymous protocols
- NetQASM SDK generation
- OpenQASM import and export

🚧 **In Progress**
- Tensor network backend
- Real hardware adapters
- Python bindings

🔮 **Planned**
- GPU acceleration
- Quantum error correction
- Advanced routing protocols

## Contributing

We welcome contributions! Key areas:
- 🧮 New backends (tensor networks, GPU)
- 🔧 Algorithms (Shor, VQE, QAOA)
- 🌐 Network protocols
- 📖 Documentation

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT OR Apache-2.0

---

*In quantum computing, the hardest part shouldn't be the framework.*
