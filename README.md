# Qnect 🔗

[![Crates.io](https://img.shields.io/crates/v/qnect.svg)](https://crates.io/crates/qnect)
[![Docs.rs](https://docs.rs/qnect/badge.svg)](https://docs.rs/qnect)
[![License](https://img.shields.io/crates/l/qnect)](./LICENSE)

**Quantum computing in Rust: from Bell pairs to distributed quantum networks.**

**No config files. No magic. Just quantum.**

Build quantum circuits, simulate massive systems, and create quantum networks - all with one powerful framework:

- 🧮 **Quantum circuits** (2 to 5000+ qubits)
- 🌐 **Distributed quantum networks** (multi-hop entanglement, teleportation)
- 🔒 **Research-grade protocols** (anonymous transmission, blind quantum computing)
- ⚙️ **Hardware-ready integration** (NetQASM, MockQnpu, QASM import/export)

```rust
use qnect::{create, builder::BackendType};
use qnect::network::{QuantumNetwork, NetworkBuilder, Topology};

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

// ...or use quantum hubs for routing!
let mut network = NetworkBuilder::new()
    .with_topology(Topology::Star { hub_name: "Hub".into(), hub_capacity: 100 })
    .add_hub("Hub", 100)
    .add_endpoint("Alice", 10)
    .add_endpoint("Bob", 10)
    .build()?;

// Distribute entanglement through the hub
let (q1, q2) = network
    .create_epr_pair_through_hub("Alice", "Bob", "Hub")
    .await?;
```

## Features

- 🚀 **Fast** - 100,000 gates/second in pure Rust
- 📈 **Scalable** - Same API from 2 to 5000+ qubits
- 🌐 **Networked** - Multi-hop entanglement, hub-based routing, NetQASM generation
- 🔬 **Research-grade** - Anonymous protocols, blind computation, verified physics
- 🎯 **Zero config** - No setup files, just `cargo run`
- 📊 **Realistic noise** - Depolarizing and measurement errors for accurate simulation

## Installation

```toml
[dependencies]
qnect = "0.3.0"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

### Your First Bell Pair (The Quantum Hello World!)

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

### Quantum Networks with Hub Routing

```rust
use qnect::network::{NetworkBuilder, Topology, LinkType};

// Build a quantum data center in 3 lines
let mut network = NetworkBuilder::new()
    .with_topology(Topology::Star {
        hub_name: "DataCenter".into(),
        hub_capacity: 100
    })
    .add_hub("DataCenter", 100)
    .add_endpoint("Server1", 10)
    .add_endpoint("Server2", 10)
    .build()?;

// Servers can share entanglement through the hub
let (q1, q2) = network
    .create_epr_pair_through_hub("Server1", "Server2", "DataCenter")
    .await?;
```

### Quantum Repeaters for Long-Distance Networks

```rust
use qnect::network::QuantumNetwork;
use qnect::network::LinkType;
use qnect::builder::BackendType;

// Build transcontinental quantum network with repeaters
let mut network = QuantumNetwork::new_distributed();

// Add cities as endpoints
network.add_distributed_node("London", 10, BackendType::Stabilizer)?;
network.add_distributed_node("Paris", 10, BackendType::Stabilizer)?;
network.add_distributed_node("Berlin", 10, BackendType::Stabilizer)?;

// Connect with quantum repeaters for long distances
network.add_quantum_link("London", "Paris",
    LinkType::Fiber { length_km: 350.0, loss_db_per_km: 0.2 },
    0.85, 100.0)?;
network.add_quantum_link("Paris", "Berlin",
    LinkType::Fiber { length_km: 850.0, loss_db_per_km: 0.2 },
    0.85, 100.0)?;

// Establish end-to-end entanglement through repeater
let (q1, q2) = network
    .establish_end_to_end_entanglement("London", "Berlin")
    .await?;
// Entanglement swapping at Paris enables London ↔ Berlin connection!
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
```

## 🌍 Real-World Simulation Example: Quantum-Secured Chat

Build a quantum chat application with QKD (Quantum Key Distribution):

```rust
// Terminal 1: Start quantum service
let service = QuantumService::new().await?;
service.run(6666).await?;

// Terminal 2: Run Paris repeater
let repeater = QuantumChatNode::new("Paris", NodeType::Repeater,
    (48.8566, 2.3522)).await?;
repeater.listen(7001).await?;

// Terminal 3: Alice in London
let mut alice = QuantumChatNode::new("Alice", NodeType::Endpoint,
    (51.5074, -0.1278)).await?;
alice.connect_to("Paris", "127.0.0.1:7001").await?;
let key = alice.run_bb84_alice("Bob", 128).await?;  // QKD protocol!
alice.run_chat("Bob", key).await?;  // Quantum-secured chat

// Terminal 4: Bob in Berlin
let mut bob = QuantumChatNode::new("Bob", NodeType::Endpoint,
    (52.5200, 13.4050)).await?;
bob.connect_to("Paris", "127.0.0.1:7001").await?;
let key = bob.run_bb84_bob("Alice").await?;
bob.run_chat("Alice", key).await?;
```

This creates a **quantum network** where:
- 🔐 Keys are distributed using BB84 quantum protocol
- 🌉 Paris acts as quantum repeater between London and Berlin
- 💬 Chat is encrypted with quantum-generated keys
- 🛡️ Eavesdropping is detected by quantum mechanics

### Anonymous Quantum Communication

Based on [Christandl & Wehner 2004](https://arxiv.org/abs/quant-ph/0409201) - impossible classically!

```rust
use qnect::network::{QuantumNetwork, LinkType};

// Create network with all participants
let participants = vec!["Alice", "Bob", "Charlie", "David", "Eve"];
let mut network = QuantumNetwork::new_distributed();

// Add all nodes (works with either mode)
for p in &participants {
    network.add_node(p, 8);
}

// Create full mesh connectivity
network.add_multiparty_link(
    participants.to_vec(),
    LinkType::Fiber { length_km: 0.1, loss_db_per_km: 0.1 },
    0.99, 10000.0
)?;

// Anonymous bit transmission
let parity = network
    .anonymous_transmission("Alice", participants.to_vec(), 1)
    .await?;
// Everyone knows bit 1 was sent, but not that Alice sent it!

// Anonymous entanglement
let (q1, q2) = network
    .anonymous_entanglement("Alice", "Bob", participants.to_vec())
    .await?;
// Alice and Bob share entanglement, untraceable by others
```

### Blind Quantum Computing (UBQC)

```rust
// Client delegates computation without revealing data
let pattern = BlindComputationPattern {
    computation_graph: vec![(0, 1), (1, 2)],
    measurement_angles: vec![0.5, 1.0, 1.5],  // Encrypted
    flow: vec![0, 1, 2],
};

let results = network
    .blind_computation_ubqc("Client", "Server", pattern)
    .await?;
// Server performed computation but learned nothing!
```

## Network Topologies

Build complex quantum networks with one line:

```rust
// Star - Quantum Data Centers
NetworkBuilder::new().with_topology(Topology::Star { hub_name: "DC".into(), hub_capacity: 100 })

// Ring - Metropolitan Networks
NetworkBuilder::new().with_topology(Topology::Ring)

// Mesh - Research Testbeds
NetworkBuilder::new().with_topology(Topology::Mesh { link_fidelity: 0.99 })

// Hierarchical - Internet Scale
NetworkBuilder::new().with_topology(Topology::Hierarchical {
    central_hub: "Core".into(),
    regional_hubs: vec!["NA".into(), "EU".into(), "APAC".into()],
})
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
- `Noisy` - Wraps any backend with realistic errors
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
cargo run --example quantum_hub_demo        # Hub-based routing
cargo run --example network_builder_demo    # Build topologies
cargo run --example quantum_topologies      # All 5 topology types
cargo run --example quantum_qkd_chat        # Secure chat with QKD
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
- Hub-based routing
- NetQASM generation
- Realistic noise models

**For Production:**
- Type-safe Rust
- Async/await design
- Resource tracking
- Hardware-ready output

## Roadmap

✅ **Released (v0.3.0)**
- Quantum simulators (state vector, stabilizer)
- Realistic noise models
- Quantum networks with routing
- **Hub-based quantum routing** (New!)
- **NetworkBuilder API** (New!)
- **5 topology templates** (New!)
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

## License

MIT OR Apache-2.0

---

*In quantum computing, the hardest part shouldn't be the framework.*
