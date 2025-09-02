use num_complex::Complex64;
use std::f64::consts::PI;

/// Represents the quantum state of N qubits
/// State vector has 2^n amplitudes for n qubits
#[derive(Debug, Clone)]
pub struct QuantumState {
    /// Number of qubits
    pub n_qubits: usize,
    /// State vector: amplitudes[i] is amplitude of |i⟩ in binary
    /// For 2 qubits: amplitudes[0]=|00⟩, [1]=|01⟩, [2]=|10⟩, [3]=|11⟩
    pub amplitudes: Vec<Complex64>,
}

impl QuantumState {
    /// Create a new state with all qubits in |0⟩
    pub fn zeros(n_qubits: usize) -> Self {
        let size = 1 << n_qubits; // 2^n
        let mut amplitudes = vec![Complex64::new(0.0, 0.0); size];
        amplitudes[0] = Complex64::new(1.0, 0.0); // |00...0⟩

        QuantumState {
            n_qubits,
            amplitudes,
        }
    }

    /// Create a specific computational basis state
    /// E.g., computational_basis(3, 0b101) creates |101⟩
    pub fn computational_basis(n_qubits: usize, state: usize) -> Self {
        let size = 1 << n_qubits;
        assert!(
            state < size,
            "State {} too large for {} qubits",
            state,
            n_qubits
        );

        let mut amplitudes = vec![Complex64::new(0.0, 0.0); size];
        amplitudes[state] = Complex64::new(1.0, 0.0);

        QuantumState {
            n_qubits,
            amplitudes,
        }
    }

    /// Apply a single-qubit gate to qubit at given index
    pub fn apply_single_qubit_gate(&mut self, qubit: usize, gate: Gate1Q) {
        assert!(qubit < self.n_qubits, "Qubit index out of range");

        // For each computational basis state
        for state in 0..(1 << self.n_qubits) {
            // Check if we need to apply the gate
            // We apply to pairs of states that differ only in the target qubit
            if state & (1 << qubit) == 0 {
                // This state has qubit in |0⟩
                let state_1 = state | (1 << qubit); // Same state but qubit in |1⟩

                // Get current amplitudes
                let a0 = self.amplitudes[state];
                let a1 = self.amplitudes[state_1];

                // Apply gate matrix
                let (new_a0, new_a1) = match gate {
                    Gate1Q::X => (a1, a0),
                    Gate1Q::Y => (-Complex64::i() * a1, Complex64::i() * a0),
                    Gate1Q::Z => (a0, -a1),
                    Gate1Q::H => {
                        let sqrt2 = 2.0_f64.sqrt();
                        ((a0 + a1) / sqrt2, (a0 - a1) / sqrt2)
                    }
                    Gate1Q::S => (a0, Complex64::i() * a1),
                    Gate1Q::T => (a0, Complex64::from_polar(1.0, PI / 4.0) * a1),
                    Gate1Q::Rx(theta) => {
                        let cos = Complex64::new((theta / 2.0).cos(), 0.0);
                        let sin = Complex64::new(0.0, -(theta / 2.0).sin());
                        (cos * a0 + sin * a1, sin * a0 + cos * a1)
                    }
                    Gate1Q::Ry(theta) => {
                        let cos = (theta / 2.0).cos();
                        let sin = (theta / 2.0).sin();
                        (
                            Complex64::new(cos, 0.0) * a0 + Complex64::new(-sin, 0.0) * a1,
                            Complex64::new(sin, 0.0) * a0 + Complex64::new(cos, 0.0) * a1,
                        )
                    }
                    Gate1Q::Rz(theta) => (
                        a0 * Complex64::from_polar(1.0, -theta / 2.0),
                        a1 * Complex64::from_polar(1.0, theta / 2.0),
                    ),
                };

                self.amplitudes[state] = new_a0;
                self.amplitudes[state_1] = new_a1;
            }
        }
    }

    /// Apply a two-qubit gate
    pub fn apply_two_qubit_gate(&mut self, qubit1: usize, qubit2: usize, gate: Gate2Q) {
        assert!(qubit1 < self.n_qubits && qubit2 < self.n_qubits);
        assert!(
            qubit1 != qubit2,
            "Cannot apply two-qubit gate to same qubit"
        );

        // Iterate through all computational basis states
        for state in 0..(1 << self.n_qubits) {
            // Only process states where both qubits are |0⟩
            if (state & (1 << qubit1)) == 0 && (state & (1 << qubit2)) == 0 {
                // Generate all 4 states for these two qubits
                let s00 = state;
                let s01 = state | (1 << qubit2);
                let s10 = state | (1 << qubit1);
                let s11 = state | (1 << qubit1) | (1 << qubit2);

                // Get current amplitudes
                let a00 = self.amplitudes[s00];
                let a01 = self.amplitudes[s01];
                let a10 = self.amplitudes[s10];
                let a11 = self.amplitudes[s11];

                // Apply gate
                let (new_a00, new_a01, new_a10, new_a11) = match gate {
                    Gate2Q::CNOT => (a00, a01, a11, a10), // |10⟩ ↔ |11⟩
                    Gate2Q::CZ => (a00, a01, a10, -a11),  // |11⟩ → -|11⟩
                    Gate2Q::SWAP => (a00, a10, a01, a11), // |01⟩ ↔ |10⟩
                    Gate2Q::CY => (a00, a01, -Complex64::i() * a11, Complex64::i() * a10),
                };

                self.amplitudes[s00] = new_a00;
                self.amplitudes[s01] = new_a01;
                self.amplitudes[s10] = new_a10;
                self.amplitudes[s11] = new_a11;
            }
        }
    }

    /// Measure a qubit, returning 0 or 1 and collapsing the state
    pub fn measure(&mut self, qubit: usize) -> u8 {
        assert!(qubit < self.n_qubits);

        // Calculate probability of measuring |0⟩
        let mut prob_0 = 0.0;
        for state in 0..(1 << self.n_qubits) {
            if (state & (1 << qubit)) == 0 {
                prob_0 += self.amplitudes[state].norm_sqr();
            }
        }

        // Collapse the state
        let measurement = if rand::random::<f64>() < prob_0 { 0 } else { 1 };

        // Renormalize based on measurement
        let norm = if measurement == 0 {
            prob_0.sqrt()
        } else {
            (1.0 - prob_0).sqrt()
        };

        for state in 0..(1 << self.n_qubits) {
            if ((state >> qubit) & 1) != measurement as usize {
                // This state is inconsistent with measurement
                self.amplitudes[state] = Complex64::new(0.0, 0.0);
            } else {
                // Renormalize
                self.amplitudes[state] /= norm;
            }
        }

        measurement
    }

    /// Check if the state is normalized
    pub fn is_normalized(&self) -> bool {
        let norm: f64 = self.amplitudes.iter().map(|a| a.norm_sqr()).sum();
        (norm - 1.0).abs() < 1e-10
    }

    /// Get the probability of measuring a specific outcome
    pub fn probability(&self, outcome: usize) -> f64 {
        if outcome >= self.amplitudes.len() {
            0.0
        } else {
            self.amplitudes[outcome].norm_sqr()
        }
    }

    /// Create a Bell state between two qubits in an n-qubit system
    pub fn create_bell_pair(&mut self, qubit1: usize, qubit2: usize) {
        // Apply H to first qubit
        self.apply_single_qubit_gate(qubit1, Gate1Q::H);
        // Apply CNOT
        self.apply_two_qubit_gate(qubit1, qubit2, Gate2Q::CNOT);
    }
}

/// Single-qubit gate types
#[derive(Debug, Clone, Copy)]
pub enum Gate1Q {
    X,
    Y,
    Z,
    H,
    S,
    T,
    Rx(f64),
    Ry(f64),
    Rz(f64),
}

/// Two-qubit gate types
#[derive(Debug, Clone, Copy)]
pub enum Gate2Q {
    CNOT,
    CZ,
    SWAP,
    CY,
}
