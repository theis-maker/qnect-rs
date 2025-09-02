use rand::Rng;
use std::time::{Duration, Instant};

/// Realistic quantum noise model
#[derive(Debug, Clone)]
pub struct NoiseModel {
    /// Single-qubit gate error rate
    pub gate_error_1q: f64,
    /// Two-qubit gate error rate
    pub gate_error_2q: f64,
    /// Measurement error rate
    pub measurement_error: f64,
    /// Dephasing rate (1/T2 in Hz)
    pub dephasing_rate: f64,
    /// Relaxation rate (1/T1 in Hz)
    pub relaxation_rate: f64,
    /// Dark count rate for detectors
    pub dark_count_rate: f64,
}

impl NoiseModel {
    pub fn realistic_nv_center() -> Self {
        NoiseModel {
            gate_error_1q: 0.001,    // 0.1% single-qubit gate error
            gate_error_2q: 0.01,     // 1% two-qubit gate error
            measurement_error: 0.03, // 3% measurement error
            dephasing_rate: 1000.0,  // T2 = 1ms
            relaxation_rate: 100.0,  // T1 = 10ms
            dark_count_rate: 1e-6,   // 1 Hz dark counts
        }
    }

    pub fn realistic_ion_trap() -> Self {
        NoiseModel {
            gate_error_1q: 0.0001,    // 0.01% - ions are good!
            gate_error_2q: 0.001,     // 0.1%
            measurement_error: 0.001, // 0.1%
            dephasing_rate: 10.0,     // T2 = 100ms
            relaxation_rate: 0.1,     // T1 = 10s
            dark_count_rate: 1e-8,    // Very low dark counts
        }
    }

    pub fn apply_gate_noise(&self, is_two_qubit: bool) -> bool {
        let error_rate = if is_two_qubit {
            self.gate_error_2q
        } else {
            self.gate_error_1q
        };
        rand::random::<f64>() < error_rate
    }

    pub fn apply_measurement_noise(&self, perfect_outcome: u8) -> u8 {
        if rand::random::<f64>() < self.measurement_error {
            1 - perfect_outcome // Flip the bit
        } else {
            perfect_outcome
        }
    }
}

/// Quantum memory with decoherence
#[derive(Debug, Clone)]
pub struct QubitMemory {
    pub qubit_id: usize,
    pub allocation_time: Instant,
    pub last_operation_time: Instant,
    pub coherence_time: Duration,
    pub initial_fidelity: f64,
    pub stored_state: Option<QuantumState>,
}

impl QubitMemory {
    pub fn new(qubit_id: usize, coherence_time: Duration) -> Self {
        let now = Instant::now();
        QubitMemory {
            qubit_id,
            allocation_time: now,
            last_operation_time: now,
            coherence_time,
            initial_fidelity: 1.0,
            stored_state: None,
        }
    }

    /// Calculate current fidelity based on elapsed time
    pub fn current_fidelity(&self) -> f64 {
        let elapsed = self.last_operation_time.elapsed();
        let decay_factor = (-elapsed.as_secs_f64() / self.coherence_time.as_secs_f64()).exp();
        self.initial_fidelity * decay_factor
    }

    /// Check if qubit has decohered beyond usability
    pub fn is_decohered(&self, threshold: f64) -> bool {
        self.current_fidelity() < threshold
    }
}

/// Realistic quantum channel model
#[derive(Debug, Clone)]
pub struct QuantumChannel {
    pub channel_type: ChannelType,
    pub length_km: f64,
    pub loss_coefficient: f64, // dB/km
    pub depolarization_rate: f64,
    pub coupling_efficiency: f64,
}

#[derive(Debug, Clone)]
pub enum ChannelType {
    Fiber,
    FreeSpace,
    Satellite,
}

impl QuantumChannel {
    pub fn realistic_fiber(length_km: f64) -> Self {
        QuantumChannel {
            channel_type: ChannelType::Fiber,
            length_km,
            loss_coefficient: 0.2, // 0.2 dB/km for telecom wavelength
            depolarization_rate: 0.001 * length_km, // 0.1% per km
            coupling_efficiency: 0.9, // 90% coupling
        }
    }

    pub fn realistic_satellite(distance_km: f64) -> Self {
        QuantumChannel {
            channel_type: ChannelType::Satellite,
            length_km: distance_km,
            loss_coefficient: 0.0,     // No absorption in vacuum
            depolarization_rate: 0.01, // From beam divergence
            coupling_efficiency: 0.3,  // 30% due to pointing, turbulence
        }
    }

    /// Calculate transmission probability
    pub fn transmission_probability(&self) -> f64 {
        let loss_db = self.loss_coefficient * self.length_km;
        let transmission = 10.0_f64.powf(-loss_db / 10.0);
        transmission * self.coupling_efficiency
    }

    /// Calculate fidelity after transmission
    pub fn output_fidelity(&self, input_fidelity: f64) -> f64 {
        // Fix: ensure fidelity stays between 0 and 1
        let transmission_prob = self.transmission_probability();
        let depolarization_factor = (1.0 - self.depolarization_rate).max(0.0);

        // Fidelity decreases with distance but stays positive
        (input_fidelity * depolarization_factor * transmission_prob).clamp(0.0, 1.0)
    }

    /// Simulate photon transmission with loss
    pub fn transmit_photon(&self) -> bool {
        rand::random::<f64>() < self.transmission_probability()
    }
}

/// EPR source with realistic parameters
#[derive(Debug, Clone)]
pub struct EPRSource {
    pub source_type: EPRSourceType,
    pub generation_rate: f64, // Hz
    pub raw_fidelity: f64,
    pub heralding_efficiency: f64,
}

#[derive(Debug, Clone)]
pub enum EPRSourceType {
    SPDC,           // Spontaneous Parametric Down-Conversion
    QuantumDot,     // Quantum dot single photon source
    AtomicEnsemble, // DLCZ protocol
    NVCenter,       // Diamond NV centers
}

impl EPRSource {
    pub fn realistic_spdc() -> Self {
        EPRSource {
            source_type: EPRSourceType::SPDC,
            generation_rate: 1e6, // 1 MHz raw rate
            raw_fidelity: 0.98,
            heralding_efficiency: 0.1, // 10% after filtering
        }
    }

    pub fn realistic_nv_center() -> Self {
        EPRSource {
            source_type: EPRSourceType::NVCenter,
            generation_rate: 100.0, // 100 Hz
            raw_fidelity: 0.95,
            heralding_efficiency: 0.01, // 1% success rate
        }
    }

    /// Time to generate an EPR pair
    pub fn generation_time(&self) -> Duration {
        let effective_rate = self.generation_rate * self.heralding_efficiency;
        Duration::from_secs_f64(1.0 / effective_rate)
    }
}

/// Quantum repeater with realistic operations
#[derive(Debug)]
pub struct QuantumRepeater {
    pub location: String,
    pub memory_qubits: usize,
    pub noise_model: NoiseModel,
    pub swap_success_rate: f64,
    pub purification_rounds: usize,
}

impl QuantumRepeater {
    pub fn realistic_nv_repeater(location: String) -> Self {
        QuantumRepeater {
            location,
            memory_qubits: 8, // 8 memory qubits
            noise_model: NoiseModel::realistic_nv_center(),
            swap_success_rate: 0.9, // 90% swap success
            purification_rounds: 2, // 2 rounds of purification
        }
    }

    /// Perform entanglement swapping with realistic success
    pub fn perform_swap(&self) -> (bool, f64) {
        let success = rand::random::<f64>() < self.swap_success_rate;
        let fidelity_loss = 0.02 * self.purification_rounds as f64;
        (success, 1.0 - fidelity_loss)
    }
}

/// Full quantum state including noise
#[derive(Debug, Clone)]
pub struct QuantumState {
    pub amplitude: Vec<Complex64>,
    pub fidelity: f64,
    pub purity: f64,
}

use num_complex::Complex64;

impl QuantumState {
    pub fn new_pure(n_qubits: usize) -> Self {
        let size = 1 << n_qubits;
        let mut amplitude = vec![Complex64::new(0.0, 0.0); size];
        amplitude[0] = Complex64::new(1.0, 0.0);

        QuantumState {
            amplitude,
            fidelity: 1.0,
            purity: 1.0,
        }
    }

    /// Apply decoherence based on elapsed time
    pub fn apply_decoherence(&mut self, noise: &NoiseModel, elapsed: Duration) {
        let t = elapsed.as_secs_f64();

        // T1 decay (relaxation)
        let t1_factor = (-t * noise.relaxation_rate).exp();

        // T2 decay (dephasing)
        let t2_factor = (-t * noise.dephasing_rate).exp();

        // Apply decoherence
        self.fidelity *= t1_factor.max(t2_factor);
        self.purity *= t2_factor;

        // Add random phase errors
        for amp in &mut self.amplitude {
            let phase_error = rand::random::<f64>() * 0.01 * t;
            *amp *= Complex64::from_polar(1.0, phase_error);
        }
    }
}

/// Detector with realistic parameters
#[derive(Debug, Clone)]
pub struct QuantumDetector {
    pub efficiency: f64,
    pub dark_count_rate: f64, // Hz
    pub timing_jitter: f64,   // seconds
    pub dead_time: Duration,
}

impl QuantumDetector {
    pub fn realistic_superconducting() -> Self {
        QuantumDetector {
            efficiency: 0.95,
            dark_count_rate: 10.0, // 10 Hz
            timing_jitter: 1e-9,   // 1 ns
            dead_time: Duration::from_nanos(50),
        }
    }

    pub fn detect_photon(&self, photon_present: bool) -> (bool, Instant) {
        let dark_count = rand::random::<f64>() < self.dark_count_rate * 1e-9;
        let detection = if photon_present {
            rand::random::<f64>() < self.efficiency
        } else {
            dark_count
        };

        let jitter = rand::rng().random_range(-self.timing_jitter..self.timing_jitter);
        let detection_time = Instant::now() + Duration::from_secs_f64(jitter);

        (detection, detection_time)
    }
}
