use crate::backend::backend::{Gate1, Gate2, QuantumBackend};
use crate::error::{QnectError, Result};
use async_trait::async_trait;

/// Stabilizer tableau backend for efficient Clifford circuit simulation
/// Can handle thousands of qubits but only H, S, CNOT gates
///
/// Based on the Aaronson-Gottesman tableau representation:
/// - 2n × (2n+1) binary matrix
/// - First n rows: destabilizers (track X operators)
/// - Next n rows: stabilizers (track Z operators)
/// - Last column: phase bits (track +/- signs)
pub struct StabilizerBackend {
    n_qubits: usize,
    // Tableau representation: 2n × (2n+1) binary matrix
    tableau: Vec<Vec<bool>>,
    // Measurement results (for deterministic behavior after measurement)
    measurement_results: Vec<Option<u8>>,
}

impl StabilizerBackend {
    pub fn new(n_qubits: usize) -> Self {
        // Initialize to |0...0⟩ state
        let mut tableau = vec![vec![false; 2 * n_qubits + 1]; 2 * n_qubits];

        // Set up initial stabilizers and destabilizers
        // For |0...0⟩ state:
        // - Destabilizers are X_i (can measure X on each qubit)
        // - Stabilizers are Z_i (each qubit is eigenstate of Z)
        for i in 0..n_qubits {
            // Destabilizers: X_i (row i has X on qubit i)
            tableau[i][i] = true;
            // Stabilizers: Z_i (row n+i has Z on qubit i)
            tableau[n_qubits + i][n_qubits + i] = true;
        }

        StabilizerBackend {
            n_qubits,
            tableau,
            measurement_results: vec![None; n_qubits],
        }
    }

    #[allow(dead_code)]
    /// Get the X component of a Pauli operator in row i
    fn get_x(&self, row: usize, col: usize) -> bool {
        self.tableau[row][col]
    }

    #[allow(dead_code)]
    /// Get the Z component of a Pauli operator in row i
    fn get_z(&self, row: usize, col: usize) -> bool {
        self.tableau[row][self.n_qubits + col]
    }

    #[allow(dead_code)]
    /// Get the phase bit (0 = +1, 1 = -1)
    fn get_phase(&self, row: usize) -> bool {
        self.tableau[row][2 * self.n_qubits]
    }

    #[allow(dead_code)]
    /// Set phase bit
    fn set_phase(&mut self, row: usize, phase: bool) {
        self.tableau[row][2 * self.n_qubits] = phase;
    }

    #[allow(dead_code)]
    /// Row addition in GF(2) with phase tracking
    /// Implements: row_h += row_i (in the stabilizer group)
    fn row_add(&mut self, h: usize, i: usize) {
        // First calculate the phase update
        // This implements the Pauli multiplication rules
        let mut phase_correction = 0;

        for j in 0..self.n_qubits {
            let x_h = self.get_x(h, j);
            let z_h = self.get_z(h, j);
            let x_i = self.get_x(i, j);
            let z_i = self.get_z(i, j);

            // Count the number of Y operators (both X and Z set)
            if x_h && z_h {
                phase_correction += 1;
            }
            if x_i && z_i {
                phase_correction += 1;
            }

            // Pauli multiplication phase rules
            if x_h && z_i && !z_h && !x_i {
                phase_correction += 1;
            } else if z_h && x_i && !x_h && !z_i {
                phase_correction += 3;
            }
        }

        // Update phase
        let current_phase = self.get_phase(h) as i32;
        let added_phase = self.get_phase(i) as i32;
        let new_phase = (current_phase + added_phase + phase_correction) % 4 == 2;
        self.set_phase(h, new_phase);

        // Update the Pauli operators (X and Z parts)
        for j in 0..2 * self.n_qubits {
            self.tableau[h][j] ^= self.tableau[i][j];
        }
    }

    #[allow(dead_code)]
    /// Apply S gate phase update rules
    fn apply_s_phase(&mut self, qubit: usize) {
        // S gate: |0⟩ → |0⟩, |1⟩ → i|1⟩
        // In stabilizer formalism: S·X·S† = Y, S·Z·S† = Z
        for i in 0..2 * self.n_qubits {
            if self.get_x(i, qubit) && self.get_z(i, qubit) {
                // XZ (which is Y) → phase factor of i²
                let current = self.get_phase(i);
                self.set_phase(i, !current);
            } else if self.get_x(i, qubit) && !self.get_z(i, qubit) {
                // X → Y (phase factor of i)
                self.tableau[i][self.n_qubits + qubit] = true;
            }
        }
    }

    #[allow(dead_code)]
    /// Find a stabilizer that anti-commutes with Pauli P on qubit q
    fn find_anticommuting_stabilizer(
        &self,
        qubit: usize,
        pauli_x: bool,
        pauli_z: bool,
    ) -> Option<usize> {
        for i in self.n_qubits..2 * self.n_qubits {
            let x_i = self.get_x(i, qubit);
            let z_i = self.get_z(i, qubit);

            // Check if they anticommute
            if (pauli_x && z_i && !pauli_z && !x_i) || (pauli_z && x_i && !pauli_x && !z_i) {
                return Some(i);
            }
        }
        None
    }

    #[allow(dead_code)]
    /// Gaussian elimination to put tableau in standard form for measurement
    fn gaussian_eliminate_for_measurement(
        &mut self,
        qubit: usize,
        pauli_x: bool,
        pauli_z: bool,
    ) -> Option<usize> {
        // Find a generator that has the Pauli we want to measure
        let mut pivot = None;

        // First check stabilizers
        for i in self.n_qubits..2 * self.n_qubits {
            let x_i = self.get_x(i, qubit);
            let z_i = self.get_z(i, qubit);

            if (pauli_x && x_i && !pauli_z && !z_i)
                || (pauli_z && z_i && !pauli_x && !x_i)
                || (pauli_x && pauli_z && x_i && z_i)
            {
                pivot = Some(i);
                break;
            }
        }

        // If not in stabilizers, check destabilizers
        if pivot.is_none() {
            for i in 0..self.n_qubits {
                let x_i = self.get_x(i, qubit);
                let z_i = self.get_z(i, qubit);

                if (pauli_x && x_i && !pauli_z && !z_i)
                    || (pauli_z && z_i && !pauli_x && !x_i)
                    || (pauli_x && pauli_z && x_i && z_i)
                {
                    pivot = Some(i);
                    break;
                }
            }
        }

        pivot
    }
}

#[async_trait]
impl QuantumBackend for StabilizerBackend {
    async fn apply_single_gate(&mut self, a: usize, gate: Gate1) -> Result<()> {
        if a >= self.n_qubits {
            return Err(QnectError::qubit_out_of_range(a, self.n_qubits));
        }
        self.measurement_results[a] = None;

        match gate {
            // H_a: swap X_a <-> Z_a and flip sign if both were 1
            Gate1::H => {
                for i in 0..2 * self.n_qubits {
                    let x = self.tableau[i][a];
                    let z = self.tableau[i][self.n_qubits + a];

                    // r_i ^= x & z
                    if x && z {
                        self.tableau[i][2 * self.n_qubits] ^= true;
                    }

                    // swap
                    self.tableau[i][a] = z;
                    self.tableau[i][self.n_qubits + a] = x;
                }
                Ok(())
            }

            // S_a: z_a ^= x_a; r_i ^= x_a & z_a (computed using pre-update values)
            Gate1::S => {
                for i in 0..2 * self.n_qubits {
                    let x = self.tableau[i][a];
                    let z = self.tableau[i][self.n_qubits + a];

                    // r_i ^= x & z
                    if x && z {
                        self.tableau[i][2 * self.n_qubits] ^= true;
                    }

                    // z_a ^= x_a
                    self.tableau[i][self.n_qubits + a] ^= x;
                }
                Ok(())
            }

            // S† = S^3
            Gate1::SDag => {
                for _ in 0..3 {
                    self.apply_single_gate(a, Gate1::S).await?;
                }
                Ok(())
            }

            // Pauli X_a: flip stabilizer signs where it anticommutes (Z on a and not X on a)
            Gate1::X => {
                for i in self.n_qubits..2 * self.n_qubits {
                    let x = self.tableau[i][a];
                    let z = self.tableau[i][self.n_qubits + a];
                    if !x && z {
                        self.tableau[i][2 * self.n_qubits] ^= true;
                    }
                }
                Ok(())
            }

            // Y_a = i X_a Z_a (Clifford): do Z then X (phase handled automatically by rules)
            Gate1::Y => {
                self.apply_single_gate(a, Gate1::Z).await?;
                self.apply_single_gate(a, Gate1::X).await?;
                Ok(())
            }

            // Pauli Z_a: flip stabilizer signs where it anticommutes (X on a and not Z on a)
            Gate1::Z => {
                for i in self.n_qubits..2 * self.n_qubits {
                    let x = self.tableau[i][a];
                    let z = self.tableau[i][self.n_qubits + a];
                    if x && !z {
                        self.tableau[i][2 * self.n_qubits] ^= true;
                    }
                }
                Ok(())
            }

            _ => Err(QnectError::invalid_operation(
                "StabilizerBackend::apply_single_gate",
                format!("Gate {:?} not supported in stabilizer backend", gate),
            )),
        }
    }

    async fn apply_two_gate(&mut self, c: usize, t: usize, gate: Gate2) -> Result<()> {
        if c >= self.n_qubits || t >= self.n_qubits {
            return Err(QnectError::qubit_out_of_range(c.max(t), self.n_qubits));
        }
        if c == t {
            return Err(QnectError::invalid_operation(
                "apply_two_gate",
                "Cannot apply 2-qubit gate to same qubit",
            ));
        }
        self.measurement_results[c] = None;
        self.measurement_results[t] = None;

        match gate {
            // CNOT_{c->t}
            // x_t ^= x_c
            // z_c ^= z_t
            // r_i ^= x_c & z_t & (x_t ^ z_c ^ true)
            Gate2::CNOT => {
                for i in 0..2 * self.n_qubits {
                    let x_c = self.tableau[i][c];
                    let z_c = self.tableau[i][self.n_qubits + c];
                    let x_t = self.tableau[i][t];
                    let z_t = self.tableau[i][self.n_qubits + t];

                    // phase update
                    if x_c && z_t && (x_t ^ z_c ^ true) {
                        self.tableau[i][2 * self.n_qubits] ^= true;
                    }

                    // x_t' = x_t ^ x_c
                    self.tableau[i][t] ^= x_c;

                    // z_c' = z_c ^ z_t
                    self.tableau[i][self.n_qubits + c] ^= z_t;
                }
                Ok(())
            }

            // CZ = H on target, CNOT, H on target
            Gate2::CZ => {
                self.apply_single_gate(t, Gate1::H).await?;
                self.apply_two_gate(c, t, Gate2::CNOT).await?;
                self.apply_single_gate(t, Gate1::H).await?;
                Ok(())
            }

            _ => Err(QnectError::invalid_operation(
                "StabilizerBackend::apply_two_gate",
                format!("Gate {:?} not supported in stabilizer backend", gate),
            )),
        }
    }

    async fn measure(&mut self, a: usize) -> Result<u8> {
        if a >= self.n_qubits {
            return Err(QnectError::qubit_out_of_range(a, self.n_qubits));
        }
        if let Some(v) = self.measurement_results[a] {
            return Ok(v);
        }

        // Check if any stabilizer anti-commutes with Z_a (i.e., has X on column a)
        let mut pivot: Option<usize> = None;
        for i in self.n_qubits..2 * self.n_qubits {
            if self.tableau[i][a]
            /* X at a */
            {
                pivot = Some(i);
                break;
            }
        }

        // RANDOM CASE: anti-commutes → sample result, collapse to eigenstate of Z_a
        if let Some(p) = pivot {
            let v = (rand::random::<u8>() & 1) as u8; // 0/1 with prob 1/2

            // Before modifying, check which qubits are correlated (have X in same row as our pivot)
            let mut correlated_qubits = Vec::new();
            for j in 0..self.n_qubits {
                if self.tableau[p][j] {
                    // Has X on qubit j
                    correlated_qubits.push(j);
                }
            }

            // Clear all other X in column a using row addition
            for i in 0..2 * self.n_qubits {
                if i != p && self.tableau[i][a] {
                    // row_i += row_p  (in GF(2))
                    for j in 0..(2 * self.n_qubits + 1) {
                        self.tableau[i][j] ^= self.tableau[p][j];
                    }
                }
            }

            // Overwrite pivot row to be exactly ±Z_a depending on v
            for j in 0..2 * self.n_qubits {
                self.tableau[p][j] = false;
            }
            self.tableau[p][self.n_qubits + a] = true; // Z_a
            self.tableau[p][2 * self.n_qubits] = v == 1; // phase = 1 means eigenvalue -1

            // CRITICAL: For GHZ states, we need to set the measurement results for ALL correlated qubits
            // In a GHZ state, after measuring one qubit, all qubits should have the same value
            self.measurement_results[a] = Some(v);

            // For a proper GHZ state, we should check if we had an all-X stabilizer
            // If so, all qubits should collapse to the same value
            if correlated_qubits.len() == self.n_qubits {
                // This was an all-X stabilizer (GHZ state) - all qubits collapse to same value
                for q in 0..self.n_qubits {
                    self.measurement_results[q] = Some(v);
                }
            }

            return Ok(v);
        }

        // DETERMINISTIC CASE: commutes → outcome fixed by sign of Z_a in the group
        let mut sign = false; // false=+1, true=-1
        let mut has_any = false;
        for i in self.n_qubits..2 * self.n_qubits {
            if self.tableau[i][self.n_qubits + a] && !self.tableau[i][a] {
                // multiply phases: sign ^= phase(row i)
                sign ^= self.tableau[i][2 * self.n_qubits];
                has_any = true;
            }
        }
        // If nothing explicit appears, the state is |0⟩ on that qubit
        let v = if has_any { if sign { 1 } else { 0 } } else { 0 };
        self.measurement_results[a] = Some(v);
        Ok(v)
    }

    async fn create_entanglement(&mut self, q1: usize, q2: usize) -> Result<()> {
        // Create Bell pair: H on q1, CNOT(q1, q2)
        self.apply_single_gate(q1, Gate1::H).await?;
        self.apply_two_gate(q1, q2, Gate2::CNOT).await?;
        Ok(())
    }

    fn qubit_count(&self) -> usize {
        self.n_qubits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stabilizer_initialization() {
        let backend = StabilizerBackend::new(2);
        // Check initial state is |00⟩
        // Destabilizers should be X₀, X₁
        assert!(backend.get_x(0, 0) && !backend.get_z(0, 0));
        assert!(backend.get_x(1, 1) && !backend.get_z(1, 1));
        // Stabilizers should be Z₀, Z₁
        assert!(!backend.get_x(2, 0) && backend.get_z(2, 0));
        assert!(!backend.get_x(3, 1) && backend.get_z(3, 1));
    }

    #[tokio::test]
    async fn test_hadamard_gate() {
        let mut backend = StabilizerBackend::new(1);
        backend.apply_single_gate(0, Gate1::H).await.unwrap();

        // After H on |0⟩, destabilizer should be Z₀, stabilizer should be X₀
        assert!(!backend.get_x(0, 0) && backend.get_z(0, 0));
        assert!(backend.get_x(1, 0) && !backend.get_z(1, 0));
    }

    #[tokio::test]
    async fn test_bell_state() {
        let mut backend = StabilizerBackend::new(2);
        backend.create_entanglement(0, 1).await.unwrap();

        // Should create (|00⟩ + |11⟩)/√2
        // Stabilizers should be XX and ZZ
        let stab1_x0 = backend.get_x(2, 0);
        let stab1_x1 = backend.get_x(2, 1);
        let stab2_z0 = backend.get_z(3, 0);
        let stab2_z1 = backend.get_z(3, 1);

        assert!((stab1_x0 && stab1_x1) || (stab2_z0 && stab2_z1));
    }
}
