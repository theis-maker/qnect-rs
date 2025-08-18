# Qnect 🔗

Quantum computing using Rust: build, simulate, and verify quantum circuits.

**No config files. No magic. Just quantum.**

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

## Features

- 🚀 **Fast** - Pure Rust performance, no Python overhead
- 🔒 **Type-safe** - If it compiles, it runs
- 📈 **Scalable** - Same API works for 2 or 200 qubits
- 🌐 **Network-ready** - Built for distributed quantum computing
- 📊 **Visual Circuits** - See what you're building with ASCII diagrams
- 🎯 **Zero config** - No JSON files, no hidden mappings
- 🛠️ **Extensible** - Swap backends without changing code

## Installation

```toml
[dependencies]
qnect = "0.1"
tokio = { version = "1", features = ["full"] }
rand = "0.8"  # For quantum algorithms

# Requires Rust 1.70+ (for stable async traits)
```

## Quick Start

### Create a Bell State

```rust
let mut q = create().with_qubits(2).build()?;

q.h(0).await?;
q.cnot(0, 1).await?;

// Measurements are perfectly correlated
let m0 = q.measure(0).await?;
let m1 = q.measure(1).await?;
assert_eq!(m0, m1);  // Always both 0 or both 1
```

### Visualize Your Circuits

```rust
let mut q = create().with_qubits(3).build()?.with_recording();

q.h(0).await?;
q.cnot(0, 1).await?;
q.cnot(0, 2).await?;
q.print_circuit();

// Output:
// q0: ┤H├─┤●├─┤●├─
// q1: ─────┤⊕├─┤│├─
// q2: ─────────┤⊕├─
```

### Scale to N-qubit GHZ States

```rust
// The SAME code pattern works for any number of qubits
let n = 10;
let mut q = create().with_qubits(n).build()?;

q.h(0).await?;
for i in 1..n {
    q.cnot(0, i).await?;
}
// Created |0000000000⟩ + |1111111111⟩
```

### Quantum Teleportation

```rust
let mut q = create().with_qubits(3).build()?;

// Alice prepares qubit 0 in state |ψ⟩
q.ry(0, std::f64::consts::PI / 3.0).await?;

// Alice and Bob share entanglement
q.create_bell_pair(1, 2).await?;

// Teleportation protocol
q.cnot(0, 1).await?;
q.h(0).await?;
let (m0, m1) = (q.measure(0).await?, q.measure(1).await?);

// Bob's corrections
if m1 == 1 { q.x(2).await?; }
if m0 == 1 { q.z(2).await?; }

// Qubit 2 now contains |ψ⟩!
```

### Quantum Key Distribution (BB84)

```rust
use rand::Rng;

// Create information-theoretically secure keys
let mut rng = rand::thread_rng();
let mut shared_key = Vec::new();

for _ in 0..100 {
    let mut q = create().with_qubits(1).build()?;

    // Alice: prepare random bit in random basis
    let bit = rng.gen_range(0..2);
    let basis = rng.gen_bool(0.5);

    if bit == 1 { q.x(0).await?; }
    if basis { q.h(0).await?; }

    // Bob: measure in random basis
    let bob_basis = rng.gen_bool(0.5);
    if bob_basis { q.h(0).await?; }
    let result = q.measure(0).await?;

    // Share bases (not bits!) over classical channel
    if basis == bob_basis {
        shared_key.push(result);
    }
}
```

## Clean Architecture

### Core Components

#### 🎯 **QuantumSystem**
The main API - clean, minimal, and async:
```rust
pub struct QuantumSystem<B: QuantumBackend> {
    backend: B,
}

impl<B: QuantumBackend> QuantumSystem<B> {
    pub async fn h(&mut self, qubit: usize) -> Result<()>
    pub async fn cnot(&mut self, control: usize, target: usize) -> Result<()>
    pub async fn measure(&mut self, qubit: usize) -> Result<u8>
    // ... other gates
}
```

#### 🔌 **QuantumBackend Trait**
Swap backends without changing your code:
```rust
#[async_trait]
pub trait QuantumBackend: Send + Sync {
    async fn apply_single_gate(&mut self, qubit: usize, gate: Gate1) -> Result<()>;
    async fn apply_two_gate(&mut self, q1: usize, q2: usize, gate: Gate2) -> Result<()>;
    async fn measure(&mut self, qubit: usize) -> Result<u8>;
    async fn create_entanglement(&mut self, q1: usize, q2: usize) -> Result<()>;
    fn qubit_count(&self) -> usize;
}
```

#### 📊 **StateVectorBackend**
Exact quantum simulation with pure Rust:
- No external dependencies
- Optimized gate operations
- Scales to ~30 qubits

#### 🌐 **QuantumNetwork**
Distributed quantum computing made simple:
```rust
let mut network = QuantumNetwork::new();
network.add_node("Alice", 2);
network.add_node("Bob", 2);
let (a, b) = network.create_epr_pair("Alice", "Bob")?;
```

### Clean Error Handling

```rust
match q.cnot(0, 10).await {
    Err(QnectError::QubitOutOfRange { qubit, max }) => {
        println!("Qubit {} doesn't exist (max: {})", qubit, max);
    }
    Ok(_) => println!("Gate applied successfully"),
}
```

## Complete Gate Set

All the gates you need, with consistent naming:

**Single-qubit gates:**
- `h()` - Hadamard
- `x()`, `y()`, `z()` - Pauli gates
- `s()`, `t()` - Phase gates
- `rx()`, `ry()`, `rz()` - Rotation gates

**Two-qubit gates:**
- `cnot()` - Controlled-NOT
- `cz()` - Controlled-Z
- `swap()` - SWAP gate
- `cy()` - Controlled-Y

## Examples

See Qnect in action:

```bash
# List all available examples
cargo run --example list

# Correctness verification
cargo run --example 00_quantum_verification

# Basic quantum
cargo run --example 01_quantum_interference
cargo run --example 03_simple_bell_pair

# Algorithms
cargo run --example 04_quantum_teleportation
cargo run --example 07_quantum_walk

# Cryptography
cargo run --example 09_qkd_bb84
cargo run --example 10_qkd_with_eve

# All gates reference
cargo run --example 11_gate_showcase
```

### 🔬 Correctness Verification

Qnect implements quantum mechanics correctly. Run our comprehensive verification:

```bash
cargo run --example 00_quantum_verification
```

This runs five fundamental quantum tests with visual circuit diagrams:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
1) CHSH (Bell inequality) — compute S with 95% CI
   Classical bound: S ≤ 2; Quantum max: 2√2 ≈ 2.828

CHSH Test Circuit:
q0: ┤H├─┤●├────┤H├┤M├────
q1: ───┤⊕├─┤Ry├──────┤M├

   S ≈ 2.619 (95% CI [2.591, 2.646]) → Violation observed!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

✅ **Bell Inequality Violation** - Reproduces quantum predictions beyond classical limits
✅ **Phase Kickback** - Demonstrates quantum information flow
✅ **Interference Patterns** - Shows wave-particle duality
✅ **GHZ Correlations** - Verifies multi-qubit entanglement
✅ **Teleportation Fidelity** - Confirms quantum state transfer

## Why Qnect?

### Built for Clarity - The Qnect way

Qnect prioritizes simplicity and correctness:

```rust
// Everything you need in one line
let mut q = create().with_qubits(5).build()?;

// Clear, explicit operations
q.h(0).await?;
q.cnot(0, 1).await?;

// Type safety catches errors at compile time
// q.cnot(0, 10).await?;  // Won't compile - saves debugging time!
```

### Design Principles

- **Explicit over implicit** - You see exactly what's happening
- **Type-safe by default** - Rust's compiler is your friend
- **Minimal API surface** - One clear way to do things
- **Zero configuration** - Start coding quantum circuits immediately
- **Async-first** - Ready for distributed quantum computing

### Focus on Fundamentals

Qnect helps you understand quantum computing by:
- Providing clear visual circuit representations
- Including comprehensive correctness tests
- Offering working examples of real quantum algorithms

## Performance

- **State Vector Backend**: Exact simulation up to ~30 qubits on modern hardware
- **Memory**: ~16GB RAM for 30 qubits (2^30 complex amplitudes)
- **Speed**: Optimized gate operations, multi-threaded where possible
- **Coming Soon**: Stabilizer backend for 1000+ qubit Clifford circuits

Pure Rust performance with no Python overhead.

## Roadmap

### ✅ Completed
- State vector simulator with all standard gates
- Async/await API
- Backend trait system
- Circuit visualization
- Quantum networking
- BB84 Quantum Key Distribution
- Comprehensive correctness tests

### 🚧 In Progress
- Stabilizer backend (Clifford circuits to 1000s of qubits)
- OpenQASM import/export
- Noise models

### 🔮 Future
- Tensor network backend
- Hardware backends (IBMQ, IonQ)
- Distributed quantum operations
- Circuit optimization

## Contributing

We welcome contributions! Areas we're excited about:
- 🧮 **Backend implementations** - Stabilizer, tensor networks
- 🔧 **Quantum algorithms** - Grover, Shor, VQE, QAOA
- 📊 **Benchmarks** - Show us where we can improve
- 📖 **Documentation** - Help others learn quantum + Rust

## Testing

```bash
# Run all tests
cargo test

# Run verification suite
cargo run --example 00_quantum_verification
```

## License

MIT OR Apache-2.0 (your choice)

---

*Remember: In quantum computing, the hardest part shouldn't be the framework.*

**Qnect** - Where quantum meets Rust. 🦀⚛️
