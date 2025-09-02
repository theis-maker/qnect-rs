use crate::network::network::QuantumNetwork;
use crate::physics::NoiseModel;
use crate::physics::QuantumChannel;

/// Quantum repeater node with entanglement swapping capability
#[derive(Debug)]
pub struct QuantumRepeaterNode {
    pub node_id: String,
    pub location: (f64, f64), // Geographic coordinates
    pub memory_slots: Vec<MemorySlot>,
    pub swap_success_rate: f64,
    pub noise_model: NoiseModel,
}

#[derive(Debug)]
pub struct MemorySlot {
    pub slot_id: usize,
    pub qubit: Option<usize>,
    pub entangled_with: Option<(String, usize)>, // (node_id, qubit_id)
    pub fidelity: f64,
    pub created_at: Instant,
}

impl QuantumRepeaterNode {
    pub fn new(node_id: String, location: (f64, f64), memory_slots: usize) -> Self {
        let slots = (0..memory_slots)
            .map(|i| MemorySlot {
                slot_id: i,
                qubit: None,
                entangled_with: None,
                fidelity: 0.0,
                created_at: Instant::now(),
            })
            .collect();

        QuantumRepeaterNode {
            node_id,
            location,
            memory_slots: slots,
            swap_success_rate: 0.85, // 85% swapping success
            noise_model: NoiseModel::realistic_nv_center(),
        }
    }

    /// Store entangled pair in memory
    pub fn store_entanglement(
        &mut self,
        qubit: usize,
        partner_node: String,
        partner_qubit: usize,
        fidelity: f64,
    ) -> Result<usize, Box<dyn Error>> {
        // Find free memory slot
        let slot = self
            .memory_slots
            .iter_mut()
            .find(|s| s.qubit.is_none())
            .ok_or("No free memory slots")?;

        slot.qubit = Some(qubit);
        slot.entangled_with = Some((partner_node, partner_qubit));
        slot.fidelity = fidelity;
        slot.created_at = Instant::now();

        Ok(slot.slot_id)
    }

    /// Check if we can perform swapping
    pub fn can_swap(&self) -> bool {
        let occupied_slots = self
            .memory_slots
            .iter()
            .filter(|s| s.qubit.is_some())
            .count();
        occupied_slots >= 2
    }

    /// Get current memory utilization
    pub fn memory_utilization(&self) -> f64 {
        let occupied = self
            .memory_slots
            .iter()
            .filter(|s| s.qubit.is_some())
            .count();
        occupied as f64 / self.memory_slots.len() as f64
    }
}

/// Multi-hop quantum network path
#[derive(Debug, Clone)]
pub struct QuantumPath {
    pub nodes: Vec<String>,
    pub total_distance: f64,
    pub expected_fidelity: f64,
    pub expected_generation_time: Duration,
}

/// Entanglement swapping manager
pub struct EntanglementSwapper {
    network: Arc<Mutex<QuantumNetwork>>,
    pub repeaters: HashMap<String, QuantumRepeaterNode>,
}

impl EntanglementSwapper {
    pub fn new(network: Arc<Mutex<QuantumNetwork>>) -> Self {
        EntanglementSwapper {
            network,
            repeaters: HashMap::new(),
        }
    }

    /// Add repeater to network
    pub fn add_repeater(&mut self, repeater: QuantumRepeaterNode) {
        self.repeaters.insert(repeater.node_id.clone(), repeater);
    }

    /// Perform entanglement swapping at a repeater
    /// Perform entanglement swapping at a repeater
    pub async fn perform_swap(
        &mut self,
        repeater_id: &str,
        left_slot: usize,
        right_slot: usize,
    ) -> Result<SwapResult, Box<dyn Error>> {
        let repeater = self
            .repeaters
            .get_mut(repeater_id)
            .ok_or("Repeater not found")?;

        // Extract needed information before borrowing
        let (left_qubit, left_partner, left_fidelity, left_created) = {
            let left = &repeater.memory_slots[left_slot];
            let qubit = left.qubit.ok_or("Left slot empty")?;
            let partner = left
                .entangled_with
                .as_ref()
                .ok_or("Left qubit not entangled")?
                .clone();
            (qubit, partner, left.fidelity, left.created_at)
        };

        let (right_qubit, right_partner, right_fidelity, right_created) = {
            let right = &repeater.memory_slots[right_slot];
            let qubit = right.qubit.ok_or("Right slot empty")?;
            let partner = right
                .entangled_with
                .as_ref()
                .ok_or("Right qubit not entangled")?
                .clone();
            (qubit, partner, right.fidelity, right.created_at)
        };

        // Check fidelity before swapping
        let combined_fidelity = left_fidelity * right_fidelity * repeater.swap_success_rate;

        // Apply decoherence based on storage time
        let left_decoherence = calculate_decoherence(&repeater.noise_model, left_created.elapsed());
        let right_decoherence =
            calculate_decoherence(&repeater.noise_model, right_created.elapsed());

        let final_fidelity = combined_fidelity * left_decoherence * right_decoherence;

        if final_fidelity < 0.5 {
            return Err("Fidelity too low for swapping".into());
        }

        // Perform Bell measurement on repeater qubits
        let mut network = self.network.lock().await;

        // Apply CNOT
        if let Some(node) = network.nodes.get_mut(repeater_id) {
            if let Some(system) = &mut node.local_system {
                system.cnot(left_qubit, right_qubit).await?;
                system.h(left_qubit).await?;
            }
        }

        // Measure both qubits
        let m1 = network.measure(repeater_id, left_qubit)?;
        let m2 = network.measure(repeater_id, right_qubit)?;

        // Drop the network lock before accessing repeater again
        drop(network);

        // Now we can safely clear memory slots
        let repeater = self.repeaters.get_mut(repeater_id).unwrap();
        repeater.memory_slots[left_slot].qubit = None;
        repeater.memory_slots[left_slot].entangled_with = None;
        repeater.memory_slots[right_slot].qubit = None;
        repeater.memory_slots[right_slot].entangled_with = None;

        Ok(SwapResult {
            left_partner,
            right_partner,
            measurement: (m1, m2),
            final_fidelity,
        })
    }

    /// Establish end-to-end entanglement through multiple hops
    pub async fn establish_multihop_entanglement(
        &mut self,
        path: &QuantumPath,
    ) -> Result<MultihopResult, Box<dyn Error>> {
        log::debug!(
            "\n🔗 Establishing {}-hop entanglement",
            path.nodes.len() - 1
        );
        log::debug!("   Path: {}", path.nodes.join(" → "));

        let mut segment_results = Vec::new();
        let start_time = Instant::now();

        // Step 1: Generate entanglement for each segment
        for i in 0..path.nodes.len() - 1 {
            let node1 = &path.nodes[i];
            let node2 = &path.nodes[i + 1];

            log::debug!(" Segment {}: {} ↔ {}", i + 1, node1, node2);

            // Generate EPR pair
            let mut network = self.network.lock().await;
            let (q1, q2) = network.create_epr_pair(node1, node2)?;

            // Calculate segment fidelity based on distance
            let distance = self.calculate_distance(node1, node2);
            let channel = QuantumChannel::realistic_fiber(distance);
            let segment_fidelity = channel.output_fidelity(0.98);

            log::debug!(
                "      Distance: {:.1} km, Fidelity: {:.3}",
                distance,
                segment_fidelity
            );

            // Store in repeater memory if not endpoint
            if i > 0 && node1 != &path.nodes[0] {
                if let Some(repeater) = self.repeaters.get_mut(node1) {
                    repeater.store_entanglement(q1, node2.clone(), q2, segment_fidelity)?;
                }
            }

            if i < path.nodes.len() - 2 && node2 != path.nodes.last().unwrap() {
                if let Some(repeater) = self.repeaters.get_mut(node2) {
                    repeater.store_entanglement(q2, node1.clone(), q1, segment_fidelity)?;
                }
            }

            segment_results.push(SegmentResult {
                node1: node1.clone(),
                node2: node2.clone(),
                qubit1: q1,
                qubit2: q2,
                fidelity: segment_fidelity,
            });

            // Simulate generation time
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        // Step 2: Perform swapping at intermediate nodes
        log::debug!(" Performing entanglement swapping...");

        let mut final_fidelity = 1.0;
        let mut corrections = Vec::new();

        for i in 1..path.nodes.len() - 1 {
            let repeater_id = &path.nodes[i];
            log::debug!("      Swapping at {}", repeater_id);

            // Find the slots to swap
            let repeater = self
                .repeaters
                .get(repeater_id)
                .ok_or("Repeater not found")?;

            let slots: Vec<_> = repeater
                .memory_slots
                .iter()
                .enumerate()
                .filter(|(_, s)| s.qubit.is_some())
                .map(|(idx, _)| idx)
                .collect();

            if slots.len() >= 2 {
                let swap_result = self.perform_swap(repeater_id, slots[0], slots[1]).await?;
                final_fidelity *= swap_result.final_fidelity;
                corrections.push(swap_result.measurement);

                log::debug!(
                    "        ✓ Swap successful, F={:.3}",
                    swap_result.final_fidelity
                );
            }
        }

        let total_time = start_time.elapsed();

        Ok(MultihopResult {
            source: path.nodes[0].clone(),
            target: path.nodes.last().unwrap().clone(),
            hops: path.nodes.len() - 1,
            final_fidelity,
            generation_time: total_time,
            corrections,
            memory_usage: self.get_memory_stats(),
        })
    }

    fn calculate_distance(&self, node1: &str, node2: &str) -> f64 {
        // Simple distance calculation based on coordinates
        // In real implementation, would use actual geographic data
        match (node1, node2) {
            ("Singapore", "Mumbai") => 3900.0,
            ("Mumbai", "Amsterdam") => 7000.0,
            ("Amsterdam", "NewYork") => 5900.0,
            _ => 1000.0, // Default
        }
    }

    fn get_memory_stats(&self) -> HashMap<String, f64> {
        self.repeaters
            .iter()
            .map(|(id, r)| (id.clone(), r.memory_utilization()))
            .collect()
    }
}

#[derive(Debug)]
pub struct SwapResult {
    pub left_partner: (String, usize),
    pub right_partner: (String, usize),
    pub measurement: (u8, u8),
    pub final_fidelity: f64,
}

#[derive(Debug)]
pub struct SegmentResult {
    pub node1: String,
    pub node2: String,
    pub qubit1: usize,
    pub qubit2: usize,
    pub fidelity: f64,
}

#[derive(Debug)]
pub struct MultihopResult {
    pub source: String,
    pub target: String,
    pub hops: usize,
    pub final_fidelity: f64,
    pub generation_time: Duration,
    pub corrections: Vec<(u8, u8)>,
    pub memory_usage: HashMap<String, f64>,
}

fn calculate_decoherence(noise: &NoiseModel, elapsed: Duration) -> f64 {
    let t = elapsed.as_secs_f64();
    (-t * noise.dephasing_rate).exp()
}

use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
