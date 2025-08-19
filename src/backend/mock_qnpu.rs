use crate::backend::backend::{Gate1, Gate2, QuantumBackend};
use crate::error::{QnectError, Result};
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Mock QNPU Backend that simulates hardware API calls
/// Demonstrates what a real hardware backend would look like
pub struct MockQnpuBackend {
    /// QNPU endpoint URL
    _endpoint: String,
    /// Node identifier
    node_id: String,
    /// Number of qubits
    n_qubits: usize,
    /// Simulated EPR success rate
    link_success_rate: f64,
    /// Pending EPR pairs awaiting heralding
    pending_eprs: Arc<Mutex<HashMap<u64, (usize, usize)>>>,
    /// Internal state (still simulated, but via "API calls")
    internal_state: Arc<Mutex<Vec<u8>>>, // Mock measurement results
    /// API call counter for debugging
    api_calls: Arc<Mutex<u64>>,
}

impl MockQnpuBackend {
    pub fn new(endpoint: String, node_id: String, n_qubits: usize) -> Self {
        MockQnpuBackend {
            _endpoint: endpoint,
            node_id,
            n_qubits,
            link_success_rate: 0.85, // 85% EPR success rate
            pending_eprs: Arc::new(Mutex::new(HashMap::new())),
            internal_state: Arc::new(Mutex::new(vec![0; n_qubits])),
            api_calls: Arc::new(Mutex::new(0)),
        }
    }

    /// Simulate an API call to the QNPU
    async fn api_call(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        // Increment call counter
        {
            let mut calls = self.api_calls.lock().await;
            *calls += 1;
        }

        // Simulate network latency (would be real network call)
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;

        // Log the API call (in real implementation, this would be HTTP/gRPC)
        println!(
            "[QNPU API #{:04}] {} -> {}: {}",
            self.api_calls.lock().await,
            self.node_id,
            method,
            params
        );

        // Simulate responses based on method
        match method {
            "gate/apply" => Ok(json!({ "success": true, "gate_id": rand::random::<u64>() })),
            "measure" => {
                let qubit = params["qubit"].as_u64().unwrap() as usize;
                let result = if rand::random::<f64>() < 0.5 { 0 } else { 1 };
                let mut state = self.internal_state.lock().await;
                state[qubit] = result;
                Ok(json!({ "success": true, "result": result }))
            }
            "epr/create" => {
                let success = rand::random::<f64>() < self.link_success_rate;
                if success {
                    let epr_id = rand::random::<u64>();
                    Ok(json!({
                        "success": true,
                        "epr_id": epr_id,
                        "fidelity": 0.92 + rand::random::<f64>() * 0.06 // 0.92-0.98
                    }))
                } else {
                    Ok(json!({ "success": false, "reason": "EPR generation failed" }))
                }
            }
            _ => Ok(json!({ "success": true })),
        }
    }
}

#[async_trait]
impl QuantumBackend for MockQnpuBackend {
    async fn apply_single_gate(&mut self, qubit: usize, gate: Gate1) -> Result<()> {
        if qubit >= self.n_qubits {
            return Err(QnectError::qubit_out_of_range(qubit, self.n_qubits));
        }

        // Simulate API call to hardware
        let gate_name = match gate {
            Gate1::H => "H",
            Gate1::X => "X",
            Gate1::Y => "Y",
            Gate1::Z => "Z",
            Gate1::S => "S",
            Gate1::SDag => "Sdag",
            Gate1::T => "T",
            Gate1::TDag => "Tdag",
            Gate1::Rx(_) => "Rx",
            Gate1::Ry(_) => "Ry",
            Gate1::Rz(_) => "Rz",
        };

        let params = match gate {
            Gate1::Rx(theta) | Gate1::Ry(theta) | Gate1::Rz(theta) => {
                json!({
                    "gate": gate_name,
                    "qubit": qubit,
                    "params": { "theta": theta }
                })
            }
            _ => {
                json!({
                    "gate": gate_name,
                    "qubit": qubit
                })
            }
        };

        self.api_call("gate/apply", params).await?;
        Ok(())
    }

    async fn apply_two_gate(&mut self, q1: usize, q2: usize, gate: Gate2) -> Result<()> {
        if q1 >= self.n_qubits || q2 >= self.n_qubits {
            return Err(QnectError::qubit_out_of_range(q1.max(q2), self.n_qubits));
        }
        if q1 == q2 {
            return Err(QnectError::invalid_gate(
                "Cannot apply two-qubit gate to same qubit",
            ));
        }

        let gate_name = match gate {
            Gate2::CNOT => "CNOT",
            Gate2::CZ => "CZ",
            Gate2::SWAP => "SWAP",
            Gate2::CY => "CY",
        };

        let params = json!({
            "gate": gate_name,
            "control": q1,
            "target": q2
        });

        self.api_call("gate/apply", params).await?;
        Ok(())
    }

    async fn measure(&mut self, qubit: usize) -> Result<u8> {
        if qubit >= self.n_qubits {
            return Err(QnectError::qubit_out_of_range(qubit, self.n_qubits));
        }

        let params = json!({ "qubit": qubit });
        let response = self.api_call("measure", params).await?;

        let result = response["result"].as_u64().unwrap() as u8;
        Ok(result)
    }

    async fn create_entanglement(&mut self, q1: usize, q2: usize) -> Result<()> {
        if q1 >= self.n_qubits || q2 >= self.n_qubits {
            return Err(QnectError::qubit_out_of_range(q1.max(q2), self.n_qubits));
        }

        // In a real QNPU, this would trigger hardware EPR generation
        let params = json!({
            "method": "heralded_epr",
            "qubits": [q1, q2],
            "target_fidelity": 0.95
        });

        let response = self.api_call("epr/create", params).await?;

        if !response["success"].as_bool().unwrap() {
            return Err(QnectError::invalid_operation(
                "create_entanglement".to_string(),
                "EPR generation failed".to_string(),
            ));
        }

        // Store EPR ID for heralding
        let epr_id = response["epr_id"].as_u64().unwrap();
        let mut eprs = self.pending_eprs.lock().await;
        eprs.insert(epr_id, (q1, q2));

        Ok(())
    }

    fn qubit_count(&self) -> usize {
        self.n_qubits
    }
}

// Extension methods for hardware-specific features
impl MockQnpuBackend {
    /// Request EPR pair with heralding (hardware-specific)
    pub async fn request_epr_with_heralding(
        &mut self,
        peer: &str,
        local_qubit: usize,
        remote_qubit: usize,
    ) -> Result<(u64, f64)> {
        let params = json!({
            "peer": peer,
            "local_qubit": local_qubit,
            "remote_qubit": remote_qubit,
            "heralded": true
        });

        let response = self.api_call("epr/create", params).await?;

        if response["success"].as_bool().unwrap() {
            let epr_id = response["epr_id"].as_u64().unwrap();
            let fidelity = response["fidelity"].as_f64().unwrap();
            Ok((epr_id, fidelity))
        } else {
            Err(QnectError::invalid_operation(
                "request_epr_with_heralding".to_string(),
                "EPR generation failed".to_string(),
            ))
        }
    }

    /// Check if EPR generation succeeded (heralding)
    pub async fn check_epr_heralding(&self, epr_id: u64) -> Result<bool> {
        // Simulate heralding delay
        tokio::time::sleep(tokio::time::Duration::from_micros(50)).await;

        let eprs = self.pending_eprs.lock().await;
        Ok(eprs.contains_key(&epr_id))
    }

    /// Get current hardware time in nanoseconds
    pub async fn get_hardware_time_ns(&self) -> Result<u64> {
        let response = self.api_call("system/time", json!({})).await?;
        Ok(response["time_ns"].as_u64().unwrap_or(0))
    }
}
