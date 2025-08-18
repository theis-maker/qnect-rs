use crate::{
    backend::backend::{Gate1, Gate2, QuantumBackend},
    error::{QnectError, Result},
};
use async_trait::async_trait;
use num_complex::Complex64;
use std::f64::consts::PI;

/// Current state vector backend
pub struct StateVectorBackend {
    n_qubits: usize,
    amplitudes: Vec<Complex64>,
}

impl StateVectorBackend {
    pub fn new(n_qubits: usize) -> Self {
        let size = 1 << n_qubits;
        let mut amplitudes = vec![Complex64::new(0.0, 0.0); size];
        amplitudes[0] = Complex64::new(1.0, 0.0);

        StateVectorBackend {
            n_qubits,
            amplitudes,
        }
    }

    fn check_qubit(&self, qubit: usize) -> Result<()> {
        if qubit >= self.n_qubits {
            Err(QnectError::qubit_out_of_range(qubit, self.n_qubits))
        } else {
            Ok(())
        }
    }
}

#[async_trait]
impl QuantumBackend for StateVectorBackend {
    async fn apply_single_gate(&mut self, qubit: usize, gate: Gate1) -> Result<()> {
        self.check_qubit(qubit)?;

        // Apply gate to all relevant amplitude pairs
        for state in 0..(1 << self.n_qubits) {
            if state & (1 << qubit) == 0 {
                let state_1 = state | (1 << qubit);
                let a0 = self.amplitudes[state];
                let a1 = self.amplitudes[state_1];

                let (new_a0, new_a1) = match gate {
                    Gate1::X => (a1, a0),
                    Gate1::Y => (-Complex64::i() * a1, Complex64::i() * a0),
                    Gate1::Z => (a0, -a1),
                    Gate1::H => {
                        let sqrt2 = 2.0_f64.sqrt();
                        ((a0 + a1) / sqrt2, (a0 - a1) / sqrt2)
                    }
                    Gate1::S => (a0, Complex64::i() * a1),
                    Gate1::T => (a0, Complex64::from_polar(1.0, PI / 4.0) * a1),
                    Gate1::Rx(theta) => {
                        let cos = Complex64::new((theta / 2.0).cos(), 0.0);
                        let sin = Complex64::new(0.0, -(theta / 2.0).sin());
                        (cos * a0 + sin * a1, sin * a0 + cos * a1)
                    }
                    Gate1::Ry(theta) => {
                        let cos = (theta / 2.0).cos();
                        let sin = (theta / 2.0).sin();
                        (
                            Complex64::new(cos, 0.0) * a0 + Complex64::new(-sin, 0.0) * a1,
                            Complex64::new(sin, 0.0) * a0 + Complex64::new(cos, 0.0) * a1,
                        )
                    }
                    Gate1::Rz(theta) => (
                        a0 * Complex64::from_polar(1.0, -theta / 2.0),
                        a1 * Complex64::from_polar(1.0, theta / 2.0),
                    ),
                };

                self.amplitudes[state] = new_a0;
                self.amplitudes[state_1] = new_a1;
            }
        }
        Ok(())
    }

    async fn apply_two_gate(&mut self, q1: usize, q2: usize, gate: Gate2) -> Result<()> {
        self.check_qubit(q1)?;
        self.check_qubit(q2)?;

        if q1 == q2 {
            return Err(QnectError::invalid_gate(
                "Cannot apply two-qubit gate to same qubit",
            ));
        }

        for state in 0..(1 << self.n_qubits) {
            if (state & (1 << q1)) == 0 && (state & (1 << q2)) == 0 {
                let s00 = state;
                let s01 = state | (1 << q2);
                let s10 = state | (1 << q1);
                let s11 = state | (1 << q1) | (1 << q2);

                let a00 = self.amplitudes[s00];
                let a01 = self.amplitudes[s01];
                let a10 = self.amplitudes[s10];
                let a11 = self.amplitudes[s11];

                let (new_a00, new_a01, new_a10, new_a11) = match gate {
                    Gate2::CNOT => (a00, a01, a11, a10),
                    Gate2::CZ => (a00, a01, a10, -a11),
                    Gate2::SWAP => (a00, a10, a01, a11),
                    Gate2::CY => (a00, a01, -Complex64::i() * a11, Complex64::i() * a10),
                };

                self.amplitudes[s00] = new_a00;
                self.amplitudes[s01] = new_a01;
                self.amplitudes[s10] = new_a10;
                self.amplitudes[s11] = new_a11;
            }
        }
        Ok(())
    }

    async fn measure(&mut self, qubit: usize) -> Result<u8> {
        self.check_qubit(qubit)?;

        let mut prob_0 = 0.0;
        for state in 0..(1 << self.n_qubits) {
            if (state & (1 << qubit)) == 0 {
                prob_0 += self.amplitudes[state].norm_sqr();
            }
        }

        let measurement = if rand::random::<f64>() < prob_0 { 0 } else { 1 };
        let norm = if measurement == 0 {
            prob_0.sqrt()
        } else {
            (1.0 - prob_0).sqrt()
        };

        for state in 0..(1 << self.n_qubits) {
            if ((state >> qubit) & 1) != measurement as usize {
                self.amplitudes[state] = Complex64::new(0.0, 0.0);
            } else {
                self.amplitudes[state] /= norm;
            }
        }

        Ok(measurement)
    }

    async fn create_entanglement(&mut self, q1: usize, q2: usize) -> Result<()> {
        self.apply_single_gate(q1, Gate1::H).await?;
        self.apply_two_gate(q1, q2, Gate2::CNOT).await?;
        Ok(())
    }

    fn qubit_count(&self) -> usize {
        self.n_qubits
    }
}
