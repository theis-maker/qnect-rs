use crate::{
    backend::backend::{Gate1, Gate2, QuantumBackend},
    error::Result,
};
use async_trait::async_trait;
use rand::rngs::StdRng;
use rand::{Rng, RngCore, SeedableRng, rng};
use std::collections::HashMap;

/// Helper enum for single-qubit Pauli operators
#[derive(Copy, Clone, Debug)]
enum SinglePauli {
    I,
    X,
    Y,
    Z,
}

impl SinglePauli {
    /// Apply this Pauli operator to a qubit
    #[inline]
    async fn apply<B: QuantumBackend>(self, backend: &mut B, qubit: usize) -> Result<()> {
        match self {
            SinglePauli::I => Ok(()),
            SinglePauli::X => backend.apply_single_gate(qubit, Gate1::X).await,
            SinglePauli::Y => backend.apply_single_gate(qubit, Gate1::Y).await,
            SinglePauli::Z => backend.apply_single_gate(qubit, Gate1::Z).await,
        }
    }
}

/// Decode 2-bit value to Pauli operator (00=I, 01=X, 10=Y, 11=Z)
#[inline]
fn decode_two_bits(code: u8) -> SinglePauli {
    match code & 0b11 {
        0 => SinglePauli::I,
        1 => SinglePauli::X,
        2 => SinglePauli::Y,
        _ => SinglePauli::Z,
    }
}

/// Noisy quantum backend wrapper that adds realistic error models
///
/// This backend wraps any other quantum backend and adds:
/// - Depolarizing noise after gates (random Pauli errors)
/// - Measurement bit-flip errors
/// - Proper two-qubit depolarizing channels
///
/// The noise model is physically motivated:
/// - Single-qubit gates: Apply random X, Y, or Z with probability p
/// - Two-qubit gates: Apply random two-qubit Pauli with probability 2p
/// - Measurements: Flip the bit with probability m
///
/// Measurements are cached until a gate operates on that qubit,
/// matching the physical behavior of quantum systems.
pub struct NoisyBackend<B: QuantumBackend> {
    inner: B,
    depolarizing_rate: f64,
    measurement_error: f64,
    measured_bits: HashMap<usize, u8>,
    rng: StdRng,
}

impl<B: QuantumBackend> NoisyBackend<B> {
    /// Clamp a value to [0,1] range, handling NaN
    fn clamp01(x: f64) -> f64 {
        if x.is_finite() {
            x.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Create a new noisy backend with random seed (non-deterministic)
    pub fn new(inner: B, depolarizing_rate: f64, measurement_error: f64) -> Self {
        // Use thread_rng to get a random seed for non-deterministic behavior
        let mut thread_rng = rng();
        let seed = thread_rng.next_u64();
        Self::new_with_seed(inner, depolarizing_rate, measurement_error, seed)
    }

    /// Create a new noisy backend with specific seed for reproducibility
    pub fn new_with_seed(
        inner: B,
        depolarizing_rate: f64,
        measurement_error: f64,
        seed: u64,
    ) -> Self {
        Self {
            inner,
            depolarizing_rate: Self::clamp01(depolarizing_rate),
            measurement_error: Self::clamp01(measurement_error),
            measured_bits: HashMap::new(),
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Apply single-qubit depolarizing noise
    /// With probability p, apply a random Pauli from {X, Y, Z}
    #[inline]
    async fn apply_noise_1q(&mut self, qubit: usize) -> Result<()> {
        if self.rng.random::<f64>() < self.depolarizing_rate {
            // Choose X, Y, or Z uniformly
            match self.rng.random_range(0u8..3) {
                0 => self.inner.apply_single_gate(qubit, Gate1::X).await?,
                1 => self.inner.apply_single_gate(qubit, Gate1::Y).await?,
                _ => self.inner.apply_single_gate(qubit, Gate1::Z).await?,
            }
        }
        Ok(())
    }

    /// Apply two-qubit depolarizing noise
    ///
    /// This implements a proper two-qubit depolarizing channel:
    /// - With probability p2, apply a uniformly random non-identity two-qubit Pauli
    /// - There are 15 such Paulis: {I,X,Y,Z}⊗{I,X,Y,Z} \ {I⊗I}
    ///
    /// This models correlated errors that occur in real two-qubit gates
    #[inline]
    async fn apply_noise_2q(&mut self, q1: usize, q2: usize) -> Result<()> {
        // Two-qubit gates typically have ~2x the error rate
        let p2 = (2.0 * self.depolarizing_rate).clamp(0.0, 1.0);

        if self.rng.random::<f64>() < p2 {
            // Sample uniformly from codes 1..=15 (excluding 0 which is I⊗I)
            let code = self.rng.random_range(1u8..=15u8);

            // Decode: high 2 bits → first qubit, low 2 bits → second qubit
            let p_left = decode_two_bits(code >> 2);
            let p_right = decode_two_bits(code);

            // Apply P⊗Q as two single-qubit operations
            p_left.apply(&mut self.inner, q1).await?;
            p_right.apply(&mut self.inner, q2).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl<B: QuantumBackend> QuantumBackend for NoisyBackend<B> {
    async fn apply_single_gate(&mut self, qubit: usize, gate: Gate1) -> Result<()> {
        // Invalidate cached classical value if this qubit is touched
        self.measured_bits.remove(&qubit);

        // First apply the gate, then add noise
        self.inner.apply_single_gate(qubit, gate).await?;
        self.apply_noise_1q(qubit).await
    }

    async fn apply_two_gate(&mut self, q1: usize, q2: usize, gate: Gate2) -> Result<()> {
        // Invalidate caches for both qubits
        self.measured_bits.remove(&q1);
        self.measured_bits.remove(&q2);

        // First apply the gate, then add noise
        self.inner.apply_two_gate(q1, q2, gate).await?;
        self.apply_noise_2q(q1, q2).await
    }

    async fn measure(&mut self, qubit: usize) -> Result<u8> {
        // Check if this qubit was already measured (collapsed to classical)
        if let Some(&bit) = self.measured_bits.get(&qubit) {
            return Ok(bit);
        }

        // Perform the quantum measurement
        let raw_result = self.inner.measure(qubit).await?;

        // Apply measurement error (bit flip)
        let final_result = if self.rng.random::<f64>() < self.measurement_error {
            1 - raw_result
        } else {
            raw_result
        };

        // Cache the result - repeated measurements return the same value
        self.measured_bits.insert(qubit, final_result);
        Ok(final_result)
    }

    async fn create_entanglement(&mut self, q1: usize, q2: usize) -> Result<()> {
        // Invalidate caches for both qubits
        self.measured_bits.remove(&q1);
        self.measured_bits.remove(&q2);

        // Create entanglement, then add two-qubit noise
        self.inner.create_entanglement(q1, q2).await?;
        self.apply_noise_2q(q1, q2).await
    }

    fn qubit_count(&self) -> usize {
        self.inner.qubit_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::state_vector::StateVectorBackend;

    #[tokio::test]
    async fn test_measurement_caching() {
        // Ensure repeated measurements return the same value
        let inner = StateVectorBackend::new(1);
        let mut noisy = NoisyBackend::new_with_seed(inner, 0.0, 0.5, 42);

        let first = noisy.measure(0).await.unwrap();
        let second = noisy.measure(0).await.unwrap();
        assert_eq!(
            first, second,
            "Repeated measurements should return same value"
        );
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        // Ensure gates invalidate cached measurements
        let inner = StateVectorBackend::new(1);
        let mut noisy = NoisyBackend::new_with_seed(inner, 0.0, 0.0, 42);

        // Measure, apply gate, measure again
        let first = noisy.measure(0).await.unwrap();
        noisy.apply_single_gate(0, Gate1::X).await.unwrap();
        let second = noisy.measure(0).await.unwrap();

        // With X gate, the measurement should flip
        assert_ne!(first, second, "Gate should invalidate cached measurement");
    }

    #[tokio::test]
    async fn test_noise_rates_clamped() {
        // Test that invalid noise rates are handled gracefully
        let inner = StateVectorBackend::new(1);
        let noisy = NoisyBackend::new(inner, -0.5, 1.5);

        assert_eq!(noisy.depolarizing_rate, 0.0);
        assert_eq!(noisy.measurement_error, 1.0);
    }

    #[tokio::test]
    async fn test_deterministic_with_seed() {
        // Test measurement error is deterministic
        let inner1 = StateVectorBackend::new(1);
        let mut noisy1 = NoisyBackend::new_with_seed(inner1, 0.0, 1.0, 12345); // 100% measurement error

        let inner2 = StateVectorBackend::new(1);
        let mut noisy2 = NoisyBackend::new_with_seed(inner2, 0.0, 1.0, 12345); // Same seed

        // Both start in |0⟩, with 100% measurement error should flip to 1
        let m1 = noisy1.measure(0).await.unwrap();
        let m2 = noisy2.measure(0).await.unwrap();

        assert_eq!(
            m1, m2,
            "Same seed should give deterministic measurement errors"
        );
        assert_eq!(m1, 1, "100% measurement error should flip |0⟩ to 1");
    }
}
