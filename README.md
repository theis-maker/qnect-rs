# Qnect 🔗

Qnect is a minimal, fast, quantum-native circuit toolkit in Rust.

## Example

```rust
use qnect::circuit::Circuit;

let mut circuit = Circuit::new(2);
circuit.h(0);
circuit.cx(0, 1);
circuit.measure_all();
