use crate::entanglement::EntanglementTracker;
use crate::error::QnectError;
use crate::network::nonlocal::{BellId, GhzId, NonlocalStore, PauliBits, SharedBell, SharedGhz};
use crate::state::{Gate1Q, Gate2Q, QuantumState};
use crate::system::QuantumSystem;
use crate::{
    backend::backend::QuantumBackend,
    builder::{BackendType, QuantumSystemBuilder},
    error::Result as QnectResult,
};
use std::collections::{HashMap, HashSet, VecDeque};
use std::f64::consts::PI;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug, Clone)]
pub struct PauliFrame {
    pub x_correction: bool,
    pub z_correction: bool,
}

/// Represents a quantum network node
pub struct QuantumNode {
    /// Node identifier
    pub id: String,
    /// Qubits owned by this node (for legacy mode)
    pub qubits: Vec<usize>,
    /// Local quantum memory/registers (for legacy mode)
    pub local_state: Option<QuantumState>,
    /// Classical communication channels
    pub classical_inbox: Option<Receiver<(String, Vec<u8>)>>,
    pub classical_outbox: HashMap<String, Sender<(String, Vec<u8>)>>,

    // === NEW FIELDS FOR SCALABILITY ===
    /// Own quantum system (for distributed mode)
    pub local_system: Option<QuantumSystem<Box<dyn QuantumBackend>>>,
    /// Tracks which qubits are allocated vs free
    pub qubit_allocator: QubitAllocator,
    /// Registry of entanglements with remote nodes
    pub entanglement_registry: HashMap<usize, RemoteEntanglement>,
    /// Node capabilities
    pub capabilities: NodeCapabilities,
}

// Manual Debug implementation since QuantumBackend isn't Debug
impl std::fmt::Debug for QuantumNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuantumNode")
            .field("id", &self.id)
            .field("qubits", &self.qubits)
            .field("local_state", &self.local_state)
            .field("qubit_allocator", &self.qubit_allocator)
            .field("entanglement_registry", &self.entanglement_registry)
            .field("capabilities", &self.capabilities)
            .field("has_local_system", &self.local_system.is_some())
            .finish()
    }
}

/// Tracks qubit allocation within a node
#[derive(Debug, Clone)]
pub struct QubitAllocator {
    pub total_qubits: usize,
    pub free_qubits: VecDeque<usize>,
    pub allocated_qubits: HashSet<usize>,
}

impl QubitAllocator {
    pub fn new(n_qubits: usize) -> Self {
        let mut free_qubits = VecDeque::new();
        for i in 0..n_qubits {
            free_qubits.push_back(i);
        }

        QubitAllocator {
            total_qubits: n_qubits,
            free_qubits,
            allocated_qubits: HashSet::new(),
        }
    }

    pub fn allocate(&mut self) -> Option<usize> {
        if let Some(qubit) = self.free_qubits.pop_front() {
            self.allocated_qubits.insert(qubit);
            Some(qubit)
        } else {
            None
        }
    }

    pub fn deallocate(&mut self, qubit: usize) -> QnectResult<()> {
        if !self.allocated_qubits.remove(&qubit) {
            return Err(crate::error::QnectError::invalid_operation(
                "deallocate".to_string(),
                "Qubit not allocated".to_string(),
            ));
        }
        self.free_qubits.push_back(qubit);
        Ok(())
    }

    pub fn get_free_count(&self) -> usize {
        self.free_qubits.len()
    }
}

#[derive(Debug, Clone)]
pub struct BlindComputationPattern {
    pub computation_graph: Vec<(usize, usize)>, // Edges in cluster state
    pub measurement_angles: Vec<f64>,           // Encrypted by client
    pub flow: Vec<usize>,                       // Measurement order
}

/// Represents entanglement with a remote qubit
#[derive(Debug, Clone)]
pub struct RemoteEntanglement {
    pub remote_node: String,
    pub remote_qubit: usize,
    pub link_id: String,
    pub fidelity: f64,
    pub created_at_ms: u64,
}

/// Node capabilities for heterogeneous networks
#[derive(Debug, Clone)]
pub struct NodeCapabilities {
    pub max_qubits: usize,
    pub can_create_epr: bool,
    pub supported_gates: HashSet<String>,
    pub backend_type: BackendType,
    pub memory_coherence_time_ms: u64,
}

impl Default for NodeCapabilities {
    fn default() -> Self {
        let mut gates = HashSet::new();
        gates.insert("H".to_string());
        gates.insert("X".to_string());
        gates.insert("Y".to_string());
        gates.insert("Z".to_string());
        gates.insert("CNOT".to_string());
        gates.insert("CZ".to_string());

        NodeCapabilities {
            max_qubits: 32,
            can_create_epr: true,
            supported_gates: gates,
            backend_type: BackendType::StateVector,
            memory_coherence_time_ms: 1000,
        }
    }
}

impl QuantumNode {
    pub fn new(id: impl Into<String>) -> Self {
        QuantumNode {
            id: id.into(),
            qubits: Vec::new(),
            local_state: None,
            classical_inbox: None,
            classical_outbox: HashMap::new(),
            local_system: None,
            qubit_allocator: QubitAllocator::new(0),
            entanglement_registry: HashMap::new(),
            capabilities: NodeCapabilities::default(),
        }
    }

    /// Create node with specific backend type
    pub fn with_backend(mut self, n_qubits: usize, backend_type: BackendType) -> QnectResult<Self> {
        // Create local quantum system
        let system = QuantumSystemBuilder::new()
            .with_backend(backend_type.clone())
            .with_qubits(n_qubits)
            .build()?;

        // Store the actual system
        self.local_system = Some(system);
        self.qubit_allocator = QubitAllocator::new(n_qubits);
        self.capabilities.backend_type = backend_type;

        // For compatibility with existing code
        self.qubits = (0..n_qubits).collect();

        Ok(self)
    }

    /// Allocate qubits to this node (compatibility method)
    pub fn allocate_qubits(&mut self, start_idx: usize, count: usize) {
        self.qubits.extend(start_idx..start_idx + count);
    }

    /// Get a free qubit from this node
    pub fn allocate_qubit(&mut self) -> Option<usize> {
        self.qubit_allocator.allocate()
    }

    /// Free a qubit
    pub fn deallocate_qubit(&mut self, qubit: usize) -> QnectResult<()> {
        // Clear any entanglements
        self.entanglement_registry.remove(&qubit);
        self.qubit_allocator.deallocate(qubit)
    }

    /// Send classical data to another node
    pub async fn send_classical(&self, to: &str, data: Vec<u8>) -> Result<(), NetworkError> {
        let sender = self
            .classical_outbox
            .get(to)
            .ok_or(NetworkError::NoConnection)?;

        sender
            .send((self.id.clone(), data))
            .await
            .map_err(|_| NetworkError::SendFailed)?;

        Ok(())
    }

    /// Receive classical data from a specific node
    pub async fn recv_classical(&mut self, from: &str) -> Result<Vec<u8>, NetworkError> {
        let inbox = self
            .classical_inbox
            .as_mut()
            .ok_or(NetworkError::NoConnection)?;

        while let Some((sender, data)) = inbox.recv().await {
            if sender == from {
                return Ok(data);
            }
            // Could buffer other messages here if needed
        }
        Err(NetworkError::ConnectionClosed)
    }

    /// Send a single classical bit (common in quantum protocols)
    pub async fn send_bit(&self, to: &str, bit: u8) -> Result<(), NetworkError> {
        self.send_classical(to, vec![bit]).await
    }

    /// Receive a single classical bit
    pub async fn recv_bit(&mut self, from: &str) -> Result<u8, NetworkError> {
        let data = self.recv_classical(from).await?;
        data.first().copied().ok_or(NetworkError::EmptyMessage)
    }

    /// Check if node supports a gate
    pub fn supports_gate(&self, gate: &str) -> bool {
        self.capabilities.supported_gates.contains(gate)
    }
}

/// Quantum link properties
#[derive(Debug, Clone)]
pub struct QuantumLink {
    pub id: String,
    pub node1: String,
    pub node2: String,
    pub link_type: LinkType,
    pub fidelity: f64,
    pub generation_rate_hz: f64,
    pub latency_us: u64,
}

#[derive(Debug, Clone)]
pub enum LinkType {
    Fiber { length_km: f64, loss_db_per_km: f64 },
    FreeSpace { distance_km: f64 },
    Satellite { orbital_height_km: f64 },
}

/// Operation recording for NetQASM generation
#[derive(Debug, Clone)]
pub enum NetworkOperation {
    CreateEPR {
        node1: String,
        node2: String,
        q1: usize,
        q2: usize,
    },
    AllocateLocal {
        node: String,
        qubit: usize,
    },
    FreeLocal {
        node: String,
        qubit: usize,
    },
    LocalGate {
        node: String,
        qubit: usize,
        gate: String,
    },
    Measure {
        node: String,
        qubit: usize,
        result: u8,
    },
    SendClassical {
        from: String,
        to: String,
        data: Vec<u8>,
    },
    RecvClassical {
        node: String,
        from: String,
    },
}

/// A quantum network with multiple nodes - Enhanced for true scalability
pub struct QuantumNetwork {
    /// Global quantum state (for backward compatibility - optional)
    pub state: QuantumState,
    /// Network nodes
    pub nodes: HashMap<String, QuantumNode>,
    /// Tracks entanglement (for backward compatibility)
    pub entanglement: EntanglementTracker,
    /// Total qubits in the network
    pub total_qubits: usize,
    /// Maps qubit index to node ID (for backward compatibility)
    pub qubit_ownership: HashMap<usize, String>,
    /// Channel senders for establishing connections
    channel_senders: HashMap<String, Sender<(String, Vec<u8>)>>,

    // === NEW FIELDS FOR SCALABILITY ===
    /// Network topology - quantum links
    pub links: HashMap<String, QuantumLink>,
    /// Operation history for NetQASM generation
    pub protocol_history: Vec<NetworkOperation>,
    /// Network mode: Legacy (global state) or Distributed (local states)
    pub mode: NetworkMode,

    pub nonlocal: NonlocalStore,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NetworkMode {
    /// Legacy mode - single global quantum state (limited to ~30 qubits)
    Legacy,
    /// Distributed mode - each node has its own quantum system (scales to 1000s of nodes)
    Distributed,
}

impl QuantumNetwork {
    /// Create a new quantum network
    pub fn new() -> Self {
        QuantumNetwork {
            state: QuantumState::zeros(0),
            nodes: HashMap::new(),
            entanglement: EntanglementTracker::new(0),
            total_qubits: 0,
            qubit_ownership: HashMap::new(),
            channel_senders: HashMap::new(),
            links: HashMap::new(),
            protocol_history: Vec::new(),
            mode: NetworkMode::Legacy,
            nonlocal: NonlocalStore::default(),
        }
    }

    /// Create a distributed quantum network (scalable)
    pub fn new_distributed() -> Self {
        let mut network = Self::new();
        network.mode = NetworkMode::Distributed;
        network
    }

    // Helper methods for nonlocal resources
    fn register_bell(&mut self, a: (String, usize), b: (String, usize)) -> BellId {
        let id = {
            self.nonlocal.next_bell_id += 1;
            self.nonlocal.next_bell_id
        };
        let bell = SharedBell {
            id,
            left: a.clone(),
            right: b.clone(),
            left_pf: PauliBits::default(),
            right_pf: PauliBits::default(),
            alive: true,
        };
        let arc = Arc::new(Mutex::new(bell));
        self.nonlocal.bells.insert(id, arc);
        self.nonlocal.bell_by_qubit.insert(a, id);
        self.nonlocal.bell_by_qubit.insert(b, id);
        id
    }

    fn register_ghz(&mut self, parties: Vec<(String, usize)>) -> GhzId {
        let id = {
            self.nonlocal.next_ghz_id += 1;
            self.nonlocal.next_ghz_id
        };
        let ghz = SharedGhz {
            id,
            parties: parties.clone(),
            phase_bit: 0,
            measured: HashMap::new(),
            pending_x_basis: HashSet::new(),
        };
        let arc = Arc::new(Mutex::new(ghz));
        self.nonlocal.ghzs.insert(id, arc);
        for p in parties {
            self.nonlocal.ghz_by_qubit.insert(p, id);
        }
        id
    }

    pub fn is_bell_qubit(&self, node: &str, q: usize) -> Option<BellId> {
        self.nonlocal
            .bell_by_qubit
            .get(&(node.to_string(), q))
            .copied()
    }

    fn is_ghz_qubit(&self, node: &str, q: usize) -> Option<GhzId> {
        self.nonlocal
            .ghz_by_qubit
            .get(&(node.to_string(), q))
            .copied()
    }

    pub fn allocate_local_qubit(&mut self, node_id: &str) -> Result<usize, NetworkError> {
        let node = self
            .nodes
            .get_mut(node_id)
            .ok_or(NetworkError::NodeNotFound)?;
        let qubit = node.allocate_qubit().ok_or(NetworkError::NoFreeQubits)?;

        // Record the allocation
        self.protocol_history.push(NetworkOperation::AllocateLocal {
            node: node_id.to_string(),
            qubit,
        });

        Ok(qubit)
    }

    /// Get link between two nodes
    pub fn get_link(&self, node1: &str, node2: &str) -> Option<&QuantumLink> {
        self.links
            .get(&format!("{}-{}", node1, node2))
            .or_else(|| self.links.get(&format!("{}-{}", node2, node1)))
    }

    /// Calculate total fidelity along a path
    pub fn calculate_path_fidelity(&self, path: &[String]) -> Result<f64, NetworkError> {
        if path.len() < 2 {
            return Ok(1.0);
        }

        let total_fidelity = path
            .windows(2)
            .map(|nodes| {
                self.get_link(&nodes[0], &nodes[1])
                    .map(|link| link.fidelity)
                    .ok_or(NetworkError::NoConnection)
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .product::<f64>();

        Ok(total_fidelity)
    }

    pub fn add_multiparty_link(
        &mut self,
        nodes: Vec<&str>,
        link_type: LinkType,
        fidelity: f64,
        generation_rate_hz: f64,
    ) -> Result<(), NetworkError> {
        // Verify all nodes exist
        for node in &nodes {
            if !self.nodes.contains_key(*node) {
                return Err(NetworkError::NodeNotFound);
            }
        }

        // Create pairwise links for full connectivity
        for i in 0..nodes.len() {
            for j in i + 1..nodes.len() {
                self.add_quantum_link(
                    nodes[i],
                    nodes[j],
                    link_type.clone(),
                    fidelity,
                    generation_rate_hz,
                )?;
            }
        }

        Ok(())
    }

    /// Find shortest path between nodes using breadth-first search
    pub fn find_shortest_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        if from == to {
            return Some(vec![from.to_string()]);
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent = HashMap::new();

        queue.push_back(from.to_string());
        visited.insert(from.to_string());

        while let Some(current) = queue.pop_front() {
            // Find all neighbors
            for link in self.links.values() {
                let neighbor = if link.node1 == current {
                    &link.node2
                } else if link.node2 == current {
                    &link.node1
                } else {
                    continue;
                };

                if !visited.contains(neighbor) {
                    visited.insert(neighbor.clone());
                    parent.insert(neighbor.clone(), current.clone());
                    queue.push_back(neighbor.clone());

                    if neighbor == to {
                        // Found the target, reconstruct path
                        let mut path = vec![to.to_string()];
                        let mut curr = to.to_string();

                        while let Some(p) = parent.get(&curr) {
                            path.push(p.clone());
                            curr = p.clone();
                        }

                        path.reverse();
                        return Some(path);
                    }
                }
            }
        }

        None // No path found
    }

    /// Add a node with a specified number of qubits (backward compatible)
    pub fn add_node(&mut self, id: impl Into<String>, n_qubits: usize) -> &mut QuantumNode {
        let id = id.into();

        match self.mode {
            NetworkMode::Legacy => {
                // Original implementation for backward compatibility
                let start_idx = self.total_qubits;

                // Expand global state
                let new_total = self.total_qubits + n_qubits;
                self.state = QuantumState::zeros(new_total);
                self.entanglement = EntanglementTracker::new(new_total);

                // Create node with classical channels
                let (tx, rx) = mpsc::channel(100);
                let mut node = QuantumNode::new(id.clone());
                node.allocate_qubits(start_idx, n_qubits);
                node.classical_inbox = Some(rx);

                // Connect to all existing nodes (full mesh)
                for (existing_id, existing_sender) in &self.channel_senders {
                    node.classical_outbox
                        .insert(existing_id.clone(), existing_sender.clone());
                }

                // Add reverse connections
                for existing_node in self.nodes.values_mut() {
                    existing_node
                        .classical_outbox
                        .insert(id.clone(), tx.clone());
                }

                // Track ownership
                for i in start_idx..start_idx + n_qubits {
                    self.qubit_ownership.insert(i, id.clone());
                }

                self.channel_senders.insert(id.clone(), tx);
                self.total_qubits = new_total;
                self.nodes.insert(id.clone(), node);
            }

            NetworkMode::Distributed => {
                // Scalable implementation - each node has its own quantum system
                self.add_distributed_node(&id, n_qubits, BackendType::StateVector)
                    .expect("Failed to add distributed node");
            }
        }

        self.nodes.get_mut(&id).unwrap()
    }

    /// Add a distributed node with its own quantum backend
    pub fn add_distributed_node(
        &mut self,
        id: &str,
        n_qubits: usize,
        backend_type: BackendType,
    ) -> QnectResult<&mut QuantumNode> {
        if self.nodes.contains_key(id) {
            return Err(crate::error::QnectError::invalid_operation(
                "add_distributed_node".to_string(),
                format!("Node {} already exists", id),
            ));
        }

        // Create node with its own quantum system
        let (tx, rx) = mpsc::channel(100);
        let mut node = QuantumNode::new(id).with_backend(n_qubits, backend_type)?;

        node.classical_inbox = Some(rx);

        // Connect to existing nodes - IMPORTANT: establish bidirectional connections
        for (existing_id, existing_sender) in &self.channel_senders {
            node.classical_outbox
                .insert(existing_id.clone(), existing_sender.clone());
        }

        // Store the sender for this node
        self.channel_senders.insert(id.to_string(), tx.clone());

        // Add reverse connections to ALL existing nodes
        for existing_node in self.nodes.values_mut() {
            existing_node
                .classical_outbox
                .insert(id.to_string(), tx.clone());
        }

        self.nodes.insert(id.to_string(), node);

        Ok(self.nodes.get_mut(id).unwrap())
    }

    /// Get a mutable reference to a node
    pub fn node_mut(&mut self, id: &str) -> Result<&mut QuantumNode, NetworkError> {
        self.nodes.get_mut(id).ok_or(NetworkError::NodeNotFound)
    }

    /// Add quantum link between nodes
    pub fn add_quantum_link(
        &mut self,
        node1: &str,
        node2: &str,
        link_type: LinkType,
        fidelity: f64,
        generation_rate_hz: f64,
    ) -> Result<(), NetworkError> {
        if !(0.0..=1.0).contains(&fidelity) {
            return Err(NetworkError::InvalidFidelity);
        }
        if generation_rate_hz < 0.0 {
            return Err(NetworkError::InvalidGenerationRate);
        }
        if !self.nodes.contains_key(node1) || !self.nodes.contains_key(node2) {
            return Err(NetworkError::NodeNotFound);
        }

        let latency_us = match &link_type {
            LinkType::Fiber { length_km, .. } => (length_km * 5.0) as u64, // ~5 μs/km
            LinkType::FreeSpace { distance_km } => (distance_km * 3.3) as u64, // ~3.3 μs/km
            LinkType::Satellite { orbital_height_km } => (orbital_height_km * 6.7) as u64, // ~6.7 μs/km
        };

        let link_id = format!("{}-{}", node1, node2);
        let link = QuantumLink {
            id: link_id.clone(),
            node1: node1.to_string(),
            node2: node2.to_string(),
            link_type,
            fidelity,
            generation_rate_hz,
            latency_us,
        };

        self.links.insert(link_id, link);
        Ok(())
    }

    /// Create an EPR pair between two nodes
    pub fn create_epr_pair(
        &mut self,
        node1: &str,
        node2: &str,
    ) -> Result<(usize, usize), NetworkError> {
        match self.mode {
            NetworkMode::Legacy => {
                // Original implementation
                let q1 = self.get_free_qubit(node1)?;
                let q2 = self.get_free_qubit(node2)?;

                // Create Bell pair
                self.state.apply_single_qubit_gate(q1, Gate1Q::H);
                self.state.apply_two_qubit_gate(q1, q2, Gate2Q::CNOT);

                // Track entanglement
                self.entanglement.entangle(q1, q2);

                // Record operation
                self.protocol_history.push(NetworkOperation::CreateEPR {
                    node1: node1.to_string(),
                    node2: node2.to_string(),
                    q1,
                    q2,
                });

                Ok((q1, q2))
            }

            NetworkMode::Distributed => {
                // Distributed implementation
                self.create_distributed_epr_pair(node1, node2)
            }
        }
    }

    /// Create EPR pair in distributed mode
    fn create_distributed_epr_pair(
        &mut self,
        node1: &str,
        node2: &str,
    ) -> Result<(usize, usize), NetworkError> {
        // Get link info first to ensure nodes are connected
        let link_id = format!("{}-{}", node1, node2);
        let _ = self
            .links
            .get(&link_id)
            .or_else(|| self.links.get(&format!("{}-{}", node2, node1)))
            .ok_or(NetworkError::NoConnection)?;

        // Allocate qubits on each node
        let q1 = {
            let node1_mut = self
                .nodes
                .get_mut(node1)
                .ok_or(NetworkError::NodeNotFound)?;
            node1_mut
                .allocate_qubit()
                .ok_or(NetworkError::NoFreeQubits)?
        };

        let q2 = {
            let node2_mut = self
                .nodes
                .get_mut(node2)
                .ok_or(NetworkError::NodeNotFound)?;
            node2_mut
                .allocate_qubit()
                .ok_or(NetworkError::NoFreeQubits)?
        };

        // NEW: Register a real nonlocal Bell resource
        self.register_bell((node1.to_string(), q1), (node2.to_string(), q2));

        // Record operation
        self.protocol_history.push(NetworkOperation::CreateEPR {
            node1: node1.to_string(),
            node2: node2.to_string(),
            q1,
            q2,
        });

        Ok((q1, q2))
    }

    // /// Create EPR pair in distributed mode
    // fn create_distributed_epr_pair(
    //     &mut self,
    //     node1: &str,
    //     node2: &str,
    // ) -> Result<(usize, usize), NetworkError> {
    //     // Get link info first to ensure nodes are connected
    //     let link_id = format!("{}-{}", node1, node2);
    //     let link = self
    //         .links
    //         .get(&link_id)
    //         .or_else(|| self.links.get(&format!("{}-{}", node2, node1)))
    //         .ok_or(NetworkError::NoConnection)?
    //         .clone(); // Clone to avoid borrow issues

    //     // Allocate qubits on each node
    //     let q1 = {
    //         let node1_mut = self
    //             .nodes
    //             .get_mut(node1)
    //             .ok_or(NetworkError::NodeNotFound)?;
    //         node1_mut
    //             .allocate_qubit()
    //             .ok_or(NetworkError::NoFreeQubits)?
    //     };

    //     let q2 = {
    //         let node2_mut = self
    //             .nodes
    //             .get_mut(node2)
    //             .ok_or(NetworkError::NodeNotFound)?;
    //         node2_mut
    //             .allocate_qubit()
    //             .ok_or(NetworkError::NoFreeQubits)?
    //     };

    //     // In distributed mode, EPR creation happens at the physical layer
    //     // For simulation, we apply operations on the first node that has both qubits temporarily
    //     // Apply H on first qubit
    //     {
    //         let node1_mut = self
    //             .nodes
    //             .get_mut(node1)
    //             .ok_or(NetworkError::NodeNotFound)?;
    //         if let Some(system) = &mut node1_mut.local_system {
    //             tokio::task::block_in_place(|| {
    //                 tokio::runtime::Handle::current().block_on(async { system.h(q1).await })
    //             })
    //             .map_err(|_| NetworkError::GateApplicationFailed)?;
    //         }
    //     }

    //     // Register entanglement on both nodes
    //     {
    //         let node1_mut = self
    //             .nodes
    //             .get_mut(node1)
    //             .ok_or(NetworkError::NodeNotFound)?;
    //         node1_mut
    //             .entanglement_registry
    //             .insert(q1, RemoteEntanglement {
    //                 remote_node: node2.to_string(),
    //                 remote_qubit: q2,
    //                 link_id: link.id.clone(),
    //                 fidelity: link.fidelity,
    //                 created_at_ms: 0, // Would use real time
    //             });
    //     }

    //     {
    //         let node2_mut = self
    //             .nodes
    //             .get_mut(node2)
    //             .ok_or(NetworkError::NodeNotFound)?;
    //         node2_mut
    //             .entanglement_registry
    //             .insert(q2, RemoteEntanglement {
    //                 remote_node: node1.to_string(),
    //                 remote_qubit: q1,
    //                 link_id: link.id,
    //                 fidelity: link.fidelity,
    //                 created_at_ms: 0,
    //             });
    //     }

    //     // Record operation
    //     self.protocol_history.push(NetworkOperation::CreateEPR {
    //         node1: node1.to_string(),
    //         node2: node2.to_string(),
    //         q1,
    //         q2,
    //     });

    //     Ok((q1, q2))
    // }

    // pub async fn entanglement_swapping(
    //     &mut self,
    //     repeater_node: &str,
    //     left_node: &str,
    //     left_epr: usize,
    //     right_node: &str,
    //     right_epr: usize,
    // ) -> Result<(), NetworkError> {
    //     // Bell measurement at repeater
    //     let repeater = self
    //         .nodes
    //         .get_mut(repeater_node)
    //         .ok_or(NetworkError::NodeNotFound)?;
    //     if let Some(system) = &mut repeater.local_system {
    //         tokio::task::block_in_place(|| {
    //             tokio::runtime::Handle::current().block_on(async {
    //                 system.cnot(left_epr, right_epr).await?;
    //                 system.h(left_epr).await?;
    //                 Ok::<(), QnectError>(())
    //             })
    //         })
    //         .map_err(|_| NetworkError::GateApplicationFailed)?;
    //     }

    //     // Measure both qubits
    //     let m1 = self.measure(repeater_node, left_epr)?;
    //     let m2 = self.measure(repeater_node, right_epr)?;

    //     // Send measurement results to end nodes
    //     {
    //         let repeater = self
    //             .nodes
    //             .get(repeater_node)
    //             .ok_or(NetworkError::NodeNotFound)?;
    //         repeater.send_bit(left_node, m1).await?;
    //         repeater.send_bit(left_node, m2).await?;
    //         repeater.send_bit(right_node, m1).await?;
    //         repeater.send_bit(right_node, m2).await?;
    //     }

    //     // End nodes apply corrections based on measurement results
    //     // This completes the entanglement swapping

    //     Ok(())
    // }

    pub async fn establish_end_to_end_entanglement(
        &mut self,
        source: &str,
        target: &str,
    ) -> Result<(usize, usize), NetworkError> {
        let path = self
            .find_shortest_path(source, target)
            .ok_or(NetworkError::NoConnection)?;

        // Check fidelity before proceeding
        let total_fidelity = self.calculate_path_fidelity(&path)?;
        if total_fidelity < 0.5 {
            return Err(NetworkError::FidelityTooLow);
        }

        if path.len() == 2 {
            // Direct connection
            return self.create_epr_pair(source, target);
        }

        // Multi-hop: use entanglement swapping
        let mut epr_pairs = Vec::new();

        // Create EPR pairs along the path
        for i in 0..path.len() - 1 {
            let (q1, q2) = self.create_epr_pair(&path[i], &path[i + 1])?;
            epr_pairs.push((path[i].clone(), q1, path[i + 1].clone(), q2));
        }

        // Track Pauli corrections needed at endpoints
        let mut pauli_corrections: HashMap<(String, usize), PauliFrame> = HashMap::new();

        // Perform swapping at intermediate nodes
        for i in 1..path.len() - 1 {
            let repeater = &path[i];
            let left_epr = epr_pairs[i - 1].3; // Right qubit of previous pair
            let right_epr = epr_pairs[i].1; // Left qubit of current pair

            // Perform Bell measurement at repeater
            {
                let repeater_node = self
                    .nodes
                    .get_mut(repeater)
                    .ok_or(NetworkError::NodeNotFound)?;
                if let Some(system) = &mut repeater_node.local_system {
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            system.cnot(left_epr, right_epr).await?;
                            system.h(left_epr).await?;
                            Ok::<(), QnectError>(())
                        })
                    })
                    .map_err(|_| NetworkError::GateApplicationFailed)?;
                }
            }

            // Measure at repeater
            let m1 = self.measure(repeater, left_epr)?;
            let m2 = self.measure(repeater, right_epr)?;

            // Send corrections to endpoints and log the communication
            {
                let repeater_node = self.nodes.get(repeater).ok_or(NetworkError::NodeNotFound)?;

                // Send measurement results to source
                repeater_node.send_bit(source, m1).await?;
                repeater_node.send_bit(source, m2).await?;

                // Log the sends to source
                self.protocol_history.push(NetworkOperation::SendClassical {
                    from: repeater.to_string(),
                    to: source.to_string(),
                    data: vec![m1],
                });
                self.protocol_history.push(NetworkOperation::SendClassical {
                    from: repeater.to_string(),
                    to: source.to_string(),
                    data: vec![m2],
                });

                // Send measurement results to target
                repeater_node.send_bit(target, m1).await?;
                repeater_node.send_bit(target, m2).await?;

                // Log the sends to target
                self.protocol_history.push(NetworkOperation::SendClassical {
                    from: repeater.to_string(),
                    to: target.to_string(),
                    data: vec![m1],
                });
                self.protocol_history.push(NetworkOperation::SendClassical {
                    from: repeater.to_string(),
                    to: target.to_string(),
                    data: vec![m2],
                });
            }

            // Track corrections needed at endpoints
            let source_qubit = epr_pairs[0].1;
            let target_qubit = epr_pairs.last().unwrap().3;

            // Update Pauli corrections for source
            pauli_corrections
                .entry((source.to_string(), source_qubit))
                .and_modify(|p| {
                    p.x_correction ^= m2 == 1;
                    p.z_correction ^= m1 == 1;
                })
                .or_insert(PauliFrame {
                    x_correction: m2 == 1,
                    z_correction: m1 == 1,
                });

            // Update Pauli corrections for target
            pauli_corrections
                .entry((target.to_string(), target_qubit))
                .and_modify(|p| {
                    p.x_correction ^= m2 == 1;
                    p.z_correction ^= m1 == 1;
                })
                .or_insert(PauliFrame {
                    x_correction: m2 == 1,
                    z_correction: m1 == 1,
                });

            // Deallocate the measured qubits at repeater
            {
                let repeater_node = self
                    .nodes
                    .get_mut(repeater)
                    .ok_or(NetworkError::NodeNotFound)?;
                repeater_node
                    .deallocate_qubit(left_epr)
                    .map_err(|_| NetworkError::QubitNotAllocated)?;
                repeater_node
                    .deallocate_qubit(right_epr)
                    .map_err(|_| NetworkError::QubitNotAllocated)?;
            }
        }

        // Source receives measurement results and logs
        {
            let source_node = self
                .nodes
                .get_mut(source)
                .ok_or(NetworkError::NodeNotFound)?;

            // Receive all measurement results from repeaters
            for i in 1..path.len() - 1 {
                let _m1 = source_node.recv_bit(&path[i]).await?;
                let _m2 = source_node.recv_bit(&path[i]).await?;

                // Log the receives
                self.protocol_history.push(NetworkOperation::RecvClassical {
                    node: source.to_string(),
                    from: path[i].clone(),
                });
                self.protocol_history.push(NetworkOperation::RecvClassical {
                    node: source.to_string(),
                    from: path[i].clone(),
                });
            }
        }

        // Target receives measurement results and logs
        {
            let target_node = self
                .nodes
                .get_mut(target)
                .ok_or(NetworkError::NodeNotFound)?;

            // Receive all measurement results from repeaters
            for i in 1..path.len() - 1 {
                let _m1 = target_node.recv_bit(&path[i]).await?;
                let _m2 = target_node.recv_bit(&path[i]).await?;

                // Log the receives
                self.protocol_history.push(NetworkOperation::RecvClassical {
                    node: target.to_string(),
                    from: path[i].clone(),
                });
                self.protocol_history.push(NetworkOperation::RecvClassical {
                    node: target.to_string(),
                    from: path[i].clone(),
                });
            }
        }

        // Apply accumulated Pauli corrections at endpoints
        for ((node, qubit), frame) in pauli_corrections {
            if frame.x_correction {
                self.apply_local_gate(&node, qubit, Gate1Q::X)?;
            }
            if frame.z_correction {
                self.apply_local_gate(&node, qubit, Gate1Q::Z)?;
            }
        }

        // Return the end-to-end entangled qubits
        Ok((epr_pairs[0].1, epr_pairs.last().unwrap().3))
    }

    /// Route classical messages through the network
    pub async fn send_classical_routed(
        &mut self,
        from: &str,
        to: &str,
        data: Vec<u8>,
    ) -> Result<(), NetworkError> {
        // In real quantum networks, classical communication is "free" via internet
        // All nodes have direct classical connections to all other nodes

        // Record for protocol history
        self.protocol_history.push(NetworkOperation::SendClassical {
            from: from.to_string(),
            to: to.to_string(),
            data: data.clone(),
        });

        // Just use the direct classical connection
        let from_node = self.nodes.get(from).ok_or(NetworkError::NodeNotFound)?;
        from_node.send_classical(to, data).await
    }

    pub async fn blind_computation_demo(
        &mut self,
        client: &str,
        server: &str,
    ) -> Result<u8, NetworkError> {
        // Client prepares random qubits
        let client_qubit = {
            let node = self
                .nodes
                .get_mut(client)
                .ok_or(NetworkError::NodeNotFound)?;
            let q = node.allocate_qubit().ok_or(NetworkError::NoFreeQubits)?;
            q
        };

        // Apply random rotations to blind the state
        let rx_angle = rand::random::<f64>() * 2.0 * PI;
        let rz_angle = rand::random::<f64>() * 2.0 * PI;
        self.apply_local_gate(client, client_qubit, Gate1Q::Rx(rx_angle))?;
        self.apply_local_gate(client, client_qubit, Gate1Q::Rz(rz_angle))?;

        // Establish end-to-end entanglement for teleportation
        let (client_epr, server_epr) = self
            .establish_end_to_end_entanglement(client, server)
            .await?;

        // Teleport using the established entanglement
        {
            let client_node = self
                .nodes
                .get_mut(client)
                .ok_or(NetworkError::NodeNotFound)?;
            if let Some(system) = &mut client_node.local_system {
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        system.cnot(client_qubit, client_epr).await?;
                        system.h(client_qubit).await?;
                        Ok::<(), QnectError>(())
                    })
                })
                .map_err(|_| NetworkError::GateApplicationFailed)?;
            }
        }

        // Measure and send results
        let m0 = self.measure(client, client_qubit)?;
        let m1 = self.measure(client, client_epr)?;

        {
            let client_node = self.nodes.get(client).ok_or(NetworkError::NodeNotFound)?;
            // In multi-hop, we need to route through the network
            // For now, assume classical communication can route
            client_node.send_bit(server, m0).await?;
            client_node.send_bit(server, m1).await?;
        }

        // Server receives and applies corrections
        {
            let server_node = self
                .nodes
                .get_mut(server)
                .ok_or(NetworkError::NodeNotFound)?;
            let c0 = server_node.recv_bit(client).await?;
            let c1 = server_node.recv_bit(client).await?;

            if c1 == 1 {
                self.apply_local_gate(server, server_epr, Gate1Q::X)?;
            }
            if c0 == 1 {
                self.apply_local_gate(server, server_epr, Gate1Q::Z)?;
            }
        }

        // Server performs blind computation
        self.apply_local_gate(server, server_epr, Gate1Q::H)?;

        // Measure and return result
        let result = self.measure(server, server_epr)?;

        // Clean up
        {
            let client_node = self
                .nodes
                .get_mut(client)
                .ok_or(NetworkError::NodeNotFound)?;
            client_node
                .deallocate_qubit(client_qubit)
                .map_err(|_| NetworkError::QubitNotAllocated)?;
            client_node
                .deallocate_qubit(client_epr)
                .map_err(|_| NetworkError::QubitNotAllocated)?;
        }
        {
            let server_node = self
                .nodes
                .get_mut(server)
                .ok_or(NetworkError::NodeNotFound)?;
            server_node
                .deallocate_qubit(server_epr)
                .map_err(|_| NetworkError::QubitNotAllocated)?;
        }

        Ok(result)
    }

    /// Enhanced blind computation with UBQC patterns
    pub async fn blind_computation_ubqc(
        &mut self,
        client: &str,
        server: &str,
        pattern: BlindComputationPattern,
    ) -> Result<Vec<u8>, NetworkError> {
        // Verify fidelity for computation
        let path = self
            .find_shortest_path(client, server)
            .ok_or(NetworkError::NoConnection)?;
        let fidelity = self.calculate_path_fidelity(&path)?;
        if fidelity < 0.7 {
            // Higher threshold for computation
            return Err(NetworkError::FidelityTooLow);
        }

        // Calculate number of qubits from the graph
        let mut max_vertex = 0;
        for (i, j) in &pattern.computation_graph {
            max_vertex = max_vertex.max(*i).max(*j);
        }
        let num_qubits = max_vertex + 1; // 0-indexed vertices

        // Prepare cluster state qubits
        let mut cluster_qubits = Vec::new();

        for _ in 0..num_qubits {
            // Use allocate_local_qubit to track the allocation
            let q = self.allocate_local_qubit(client)?;

            // Apply random rotation for blindness
            let theta = rand::random::<f64>() * 2.0 * PI;
            self.apply_local_gate(client, q, Gate1Q::Rx(theta))?;

            cluster_qubits.push((q, theta));
        }

        // Teleport all qubits to server
        let mut server_qubits = Vec::new();
        for (client_q, _) in &cluster_qubits {
            let server_q = self
                .quantum_teleportation(client, server, *client_q)
                .await?;
            server_qubits.push(server_q);
        }

        // Server creates cluster state
        for (i, j) in &pattern.computation_graph {
            let server_node = self
                .nodes
                .get_mut(server)
                .ok_or(NetworkError::NodeNotFound)?;
            if let Some(system) = &mut server_node.local_system {
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current()
                        .block_on(async { system.cz(server_qubits[*i], server_qubits[*j]).await })
                })
                .map_err(|_| NetworkError::GateApplicationFailed)?;
            }
        }

        // Perform measurements in order
        let mut results = Vec::new();
        for (idx, &qubit_idx) in pattern.flow.iter().enumerate() {
            // Client sends encrypted angle
            let angle = pattern.measurement_angles[idx] + cluster_qubits[qubit_idx].1;
            let angle_bytes = angle.to_le_bytes().to_vec();

            // Use routed classical communication
            self.send_classical_routed(client, server, angle_bytes)
                .await?;

            // Server receives angle and measures
            let server_node = self
                .nodes
                .get_mut(server)
                .ok_or(NetworkError::NodeNotFound)?;
            let received_angle_bytes = server_node.recv_classical(client).await?;
            self.protocol_history.push(NetworkOperation::RecvClassical {
                node: server.to_string(),
                from: client.to_string(),
            });
            let received_angle = f64::from_le_bytes(
                received_angle_bytes
                    .try_into()
                    .map_err(|_| NetworkError::EmptyMessage)?,
            );

            // Apply rotation and measure
            self.apply_local_gate(
                server,
                server_qubits[qubit_idx],
                Gate1Q::Rz(-received_angle),
            )?;
            self.apply_local_gate(server, server_qubits[qubit_idx], Gate1Q::H)?;

            let result = self.measure(server, server_qubits[qubit_idx])?;
            results.push(result);

            // Send result back to client
            self.send_classical_routed(server, client, vec![result])
                .await?;

            // Client receives the result
            {
                let client_node = self
                    .nodes
                    .get_mut(client)
                    .ok_or(NetworkError::NodeNotFound)?;
                let _ = client_node.recv_bit(server).await?;

                // Log the receive
                self.protocol_history.push(NetworkOperation::RecvClassical {
                    node: client.to_string(),
                    from: server.to_string(),
                });
            }
        }

        // Clean up - move server_qubits to avoid borrow issues
        let qubits_to_clean = server_qubits;
        for q in qubits_to_clean {
            if let Some(node) = self.nodes.get_mut(server) {
                let _ = node.deallocate_qubit(q);
                self.protocol_history.push(NetworkOperation::FreeLocal {
                    node: server.to_string(),
                    qubit: q,
                });
            }
        }

        Ok(results)
    }

    /// Find path with best fidelity (not just shortest)
    pub fn find_best_fidelity_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        // Use Dijkstra's algorithm with fidelity as weight
        use std::cmp::Ordering;
        use std::collections::BinaryHeap;

        #[derive(Clone)]
        struct State {
            node: String,
            fidelity: f64,
            path: Vec<String>,
        }

        impl PartialEq for State {
            fn eq(&self, other: &Self) -> bool {
                self.fidelity.to_bits() == other.fidelity.to_bits()
            }
        }

        impl Eq for State {}

        impl Ord for State {
            fn cmp(&self, other: &Self) -> Ordering {
                self.fidelity
                    .partial_cmp(&other.fidelity)
                    .unwrap_or(Ordering::Equal)
            }
        }

        impl PartialOrd for State {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut heap = BinaryHeap::new();
        let mut best_fidelity = HashMap::new();

        heap.push(State {
            node: from.to_string(),
            fidelity: 1.0,
            path: vec![from.to_string()],
        });
        best_fidelity.insert(from.to_string(), 1.0);

        while let Some(state) = heap.pop() {
            if state.node == to {
                return Some(state.path);
            }

            if state.fidelity < best_fidelity.get(&state.node).copied().unwrap_or(0.0) {
                continue;
            }

            // Check all neighbors
            for link in self.links.values() {
                let neighbor = if link.node1 == state.node {
                    &link.node2
                } else if link.node2 == state.node {
                    &link.node1
                } else {
                    continue;
                };

                let new_fidelity = state.fidelity * link.fidelity;

                if new_fidelity > best_fidelity.get(neighbor).copied().unwrap_or(0.0) {
                    best_fidelity.insert(neighbor.clone(), new_fidelity);
                    let mut new_path = state.path.clone();
                    new_path.push(neighbor.clone());

                    heap.push(State {
                        node: neighbor.clone(),
                        fidelity: new_fidelity,
                        path: new_path,
                    });
                }
            }
        }

        None
    }

    /// Visualize the network topology in ASCII
    pub fn visualize_network(&self) -> String {
        let mut output = String::new();

        output.push_str("=== Quantum Network Topology ===\n\n");

        // List nodes with their properties
        output.push_str("Nodes:\n");
        for (id, node) in &self.nodes {
            let qubit_count = match self.mode {
                NetworkMode::Legacy => node.qubits.len(),
                NetworkMode::Distributed => node.qubit_allocator.total_qubits,
            };

            let backend = match &node.capabilities.backend_type {
                BackendType::StateVector => "StateVector",
                BackendType::Stabilizer => "Stabilizer",
                _ => "Unknown",
            };

            output.push_str(&format!(
                "  [{}] - {} qubits ({})\n",
                id, qubit_count, backend
            ));

            // Show entanglements
            if !node.entanglement_registry.is_empty() {
                output.push_str("    Entangled with:\n");
                for (local_q, remote) in &node.entanglement_registry {
                    output.push_str(&format!(
                        "      q{} ←→ {}:q{} (fidelity: {:.3})\n",
                        local_q, remote.remote_node, remote.remote_qubit, remote.fidelity
                    ));
                }
            }
        }

        output.push_str("\nLinks:\n");

        // Show unique links (avoid duplicates)
        let mut drawn_links = HashSet::new();

        for link in self.links.values() {
            let key = if link.node1 < link.node2 {
                format!("{}-{}", link.node1, link.node2)
            } else {
                format!("{}-{}", link.node2, link.node1)
            };

            if drawn_links.contains(&key) {
                continue;
            }
            drawn_links.insert(key);

            let link_str = match &link.link_type {
                LinkType::Fiber { length_km, .. } => format!("══{}km══", length_km),
                LinkType::FreeSpace { distance_km } => format!("╌{}km╌", distance_km),
                LinkType::Satellite { orbital_height_km } => format!("∿{}km∿", orbital_height_km),
            };

            output.push_str(&format!(
                "  {} {} {} (F:{:.2}, {}Hz, {}μs)\n",
                link.node1,
                link_str,
                link.node2,
                link.fidelity,
                link.generation_rate_hz,
                link.latency_us
            ));
        }

        // Network statistics
        output.push_str(&format!("\nStatistics:\n"));
        output.push_str(&format!("  Total nodes: {}\n", self.nodes.len()));
        output.push_str(&format!("  Total links: {}\n", self.links.len()));
        output.push_str(&format!("  Network mode: {:?}\n", self.mode));

        output
    }

    /// Get a free qubit from a node
    pub fn get_free_qubit(&mut self, node_id: &str) -> Result<usize, NetworkError> {
        match self.mode {
            NetworkMode::Legacy => {
                let node = self.nodes.get(node_id).ok_or(NetworkError::NodeNotFound)?;

                // In legacy mode, find a qubit that's not currently entangled
                for &qubit in &node.qubits {
                    // Check if this qubit is free (not part of an entangled pair)
                    let mut is_free = true;
                    for other_qubit in 0..self.total_qubits {
                        if qubit != other_qubit
                            && self.entanglement.are_entangled(qubit, other_qubit)
                        {
                            is_free = false;
                            break;
                        }
                    }
                    if is_free {
                        return Ok(qubit);
                    }
                }
                Err(NetworkError::NoFreeQubits)
            }
            NetworkMode::Distributed => {
                // Must allocate, not just peek!
                let node = self
                    .nodes
                    .get_mut(node_id)
                    .ok_or(NetworkError::NodeNotFound)?;
                node.allocate_qubit().ok_or(NetworkError::NoFreeQubits)
            }
        }
    }

    /// Apply a local gate on a node's qubit
    pub fn apply_local_gate(
        &mut self,
        node_id: &str,
        qubit_idx: usize,
        gate: Gate1Q,
    ) -> Result<(), NetworkError> {
        if let Some(ghz_id) = self.is_ghz_qubit(node_id, qubit_idx) {
            let ghz_arc = self.nonlocal.ghzs.get(&ghz_id).unwrap().clone();
            let mut ghz = ghz_arc.lock().unwrap();

            match gate {
                Gate1Q::Z => {
                    // Flip global phase: Z on any party toggles GHZ phase bit
                    ghz.phase_bit ^= 1;
                    self.protocol_history.push(NetworkOperation::LocalGate {
                        node: node_id.to_string(),
                        qubit: qubit_idx,
                        gate: "Z".into(),
                    });
                    return Ok(());
                }
                Gate1Q::H => {
                    // Mark that this party will be measured in X-basis
                    ghz.pending_x_basis.insert((node_id.to_string(), qubit_idx));
                    self.protocol_history.push(NetworkOperation::LocalGate {
                        node: node_id.to_string(),
                        qubit: qubit_idx,
                        gate: "H".into(),
                    });
                    return Ok(());
                }
                _ => {
                    return Err(NetworkError::InvalidOperation);
                }
            }
        }

        // --- NEW: Nonlocal Bell pair handling ---
        if let Some(bell_id) = self.is_bell_qubit(node_id, qubit_idx) {
            let bell_arc = self.nonlocal.bells.get(&bell_id).unwrap().clone();
            let mut bell = bell_arc.lock().unwrap();
            if !bell.alive {
                return Err(NetworkError::InvalidOperation);
            }

            // Pick side
            let is_left = bell.left == (node_id.to_string(), qubit_idx);
            let pf = if is_left {
                &mut bell.left_pf
            } else {
                &mut bell.right_pf
            };

            match gate {
                Gate1Q::X => pf.ax ^= 1,
                Gate1Q::Z => pf.az ^= 1,
                Gate1Q::H => std::mem::swap(&mut pf.ax, &mut pf.az),
                Gate1Q::S => pf.az ^= pf.ax, // X->XZ, Z->Z
                Gate1Q::Y | Gate1Q::T | Gate1Q::Rx(_) | Gate1Q::Ry(_) | Gate1Q::Rz(_) => {
                    return Err(NetworkError::InvalidOperation);
                }
            }
            self.protocol_history.push(NetworkOperation::LocalGate {
                node: node_id.to_string(),
                qubit: qubit_idx,
                gate: format!("{:?}", gate),
            });
            return Ok(());
        }

        match self.mode {
            NetworkMode::Legacy => {
                // Original implementation
                if self.qubit_ownership.get(&qubit_idx) != Some(&node_id.to_string()) {
                    return Err(NetworkError::QubitNotOwned);
                }

                self.state.apply_single_qubit_gate(qubit_idx, gate);

                // Record operation
                self.protocol_history.push(NetworkOperation::LocalGate {
                    node: node_id.to_string(),
                    qubit: qubit_idx,
                    gate: format!("{:?}", gate),
                });

                Ok(())
            }
            NetworkMode::Distributed => {
                let node = self
                    .nodes
                    .get_mut(node_id)
                    .ok_or(NetworkError::NodeNotFound)?;

                if !node.qubit_allocator.allocated_qubits.contains(&qubit_idx) {
                    return Err(NetworkError::QubitNotOwned);
                }

                // Apply on node's local system
                if let Some(system) = &mut node.local_system {
                    // Use the QuantumSystem's methods directly
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            match gate {
                                Gate1Q::H => system.h(qubit_idx).await,
                                Gate1Q::X => system.x(qubit_idx).await,
                                Gate1Q::Y => system.y(qubit_idx).await,
                                Gate1Q::Z => system.z(qubit_idx).await,
                                Gate1Q::S => system.s(qubit_idx).await,
                                Gate1Q::T => system.t(qubit_idx).await,
                                Gate1Q::Rx(angle) => system.rx(qubit_idx, angle).await,
                                Gate1Q::Ry(angle) => system.ry(qubit_idx, angle).await,
                                Gate1Q::Rz(angle) => system.rz(qubit_idx, angle).await,
                            }
                        })
                    })
                    .map_err(|_| NetworkError::GateApplicationFailed)?;
                } else {
                    return Err(NetworkError::NoLocalSystem);
                }

                // Record operation
                self.protocol_history.push(NetworkOperation::LocalGate {
                    node: node_id.to_string(),
                    qubit: qubit_idx,
                    gate: format!("{:?}", gate),
                });

                Ok(())
            }
        }
    }

    /// Measure a qubit at a node
    pub fn measure(&mut self, node_id: &str, qubit_idx: usize) -> Result<u8, NetworkError> {
        if let Some(ghz_id) = self.is_ghz_qubit(node_id, qubit_idx) {
            use rand::Rng;
            let ghz_arc = self.nonlocal.ghzs.get(&ghz_id).unwrap().clone();
            let mut ghz = ghz_arc.lock().unwrap();

            let key = (node_id.to_string(), qubit_idx);

            // Check if this is X-basis measurement
            let x_basis = ghz.pending_x_basis.contains(&key);
            if !x_basis {
                return Err(NetworkError::InvalidOperation);
            }

            // Calculate measurement result
            let total = ghz.parties.len();
            let already = ghz.measured.len();

            let m = if already + 1 < total {
                // Not the last measurement - random
                rand::rng().random::<u8>() & 1
            } else {
                // Last measurement - enforce parity constraint
                let sum: u8 = ghz.measured.values().copied().sum::<u8>() % 2;
                ghz.phase_bit ^ sum
            };

            ghz.measured.insert(key.clone(), m);

            // Clean up if fully measured
            if ghz.measured.len() == total {
                for p in &ghz.parties {
                    self.nonlocal.ghz_by_qubit.remove(p);
                }
                drop(ghz);
                self.nonlocal.ghzs.remove(&ghz_id);
            }

            self.protocol_history.push(NetworkOperation::Measure {
                node: node_id.to_string(),
                qubit: qubit_idx,
                result: m,
            });
            return Ok(m);
        }

        // --- NEW: Nonlocal Bell pair measurement ---
        if let Some(bell_id) = self.is_bell_qubit(node_id, qubit_idx) {
            use rand::Rng;
            let bell_arc = self.nonlocal.bells.get(&bell_id).unwrap().clone();
            let mut bell = bell_arc.lock().unwrap();

            if !bell.alive {
                return Err(NetworkError::InvalidOperation);
            }

            // Z-basis measurement (random outcome)
            let m = rand::rng().random::<u8>() & 1;

            // Collapse the pair
            bell.alive = false;
            let (a, b) = (bell.left.clone(), bell.right.clone());
            drop(bell);

            self.nonlocal.bell_by_qubit.remove(&a);
            self.nonlocal.bell_by_qubit.remove(&b);
            self.nonlocal.bells.remove(&bell_id);

            self.protocol_history.push(NetworkOperation::Measure {
                node: node_id.to_string(),
                qubit: qubit_idx,
                result: m,
            });
            return Ok(m);
        }

        let result = match self.mode {
            NetworkMode::Legacy => {
                // Original implementation
                if self.qubit_ownership.get(&qubit_idx) != Some(&node_id.to_string()) {
                    return Err(NetworkError::QubitNotOwned);
                }

                let result = self.state.measure(qubit_idx);
                self.entanglement.measure(qubit_idx);
                result
            }
            NetworkMode::Distributed => {
                let node = self
                    .nodes
                    .get_mut(node_id)
                    .ok_or(NetworkError::NodeNotFound)?;

                if !node.qubit_allocator.allocated_qubits.contains(&qubit_idx) {
                    return Err(NetworkError::QubitNotOwned);
                }

                let result = if let Some(system) = &mut node.local_system {
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current()
                            .block_on(async { system.measure(qubit_idx).await })
                    })
                    .map_err(|_| NetworkError::MeasurementFailed)?
                } else {
                    return Err(NetworkError::NoLocalSystem);
                };

                // Clear entanglement registry for this qubit
                node.entanglement_registry.remove(&qubit_idx);

                result
            }
        };

        // Record operation
        self.protocol_history.push(NetworkOperation::Measure {
            node: node_id.to_string(),
            qubit: qubit_idx,
            result,
        });

        Ok(result)
    }

    /// Check if two qubits are entangled
    pub fn are_entangled(&self, q1: usize, q2: usize) -> bool {
        match self.mode {
            NetworkMode::Legacy => self.entanglement.are_entangled(q1, q2),

            NetworkMode::Distributed => {
                // Check entanglement registries
                for node in self.nodes.values() {
                    if let Some(entanglement) = node.entanglement_registry.get(&q1) {
                        if entanglement.remote_qubit == q2 {
                            return true;
                        }
                    }
                }
                false
            }
        }
    }

    /// Execute quantum teleportation protocol
    pub async fn quantum_teleportation(
        &mut self,
        alice: &str,
        bob: &str,
        alice_qubit: usize,
    ) -> Result<usize, NetworkError> {
        match self.mode {
            NetworkMode::Legacy => {
                // Original implementation for legacy mode
                if self.qubit_ownership.get(&alice_qubit) != Some(&alice.to_string()) {
                    return Err(NetworkError::QubitNotOwned);
                }

                // Find a free qubit for Alice's EPR half
                let alice_node = self.nodes.get(alice).ok_or(NetworkError::NodeNotFound)?;
                let alice_epr = alice_node
                    .qubits
                    .iter()
                    .find(|&&q| q != alice_qubit)
                    .copied()
                    .ok_or(NetworkError::NoFreeQubits)?;

                // Get Bob's EPR half
                let bob_epr = self.get_free_qubit(bob)?;

                // Create EPR pair
                self.state.apply_single_qubit_gate(alice_epr, Gate1Q::H);
                self.state
                    .apply_two_qubit_gate(alice_epr, bob_epr, Gate2Q::CNOT);
                self.entanglement.entangle(alice_epr, bob_epr);

                // Alice's Bell measurement
                self.state
                    .apply_two_qubit_gate(alice_qubit, alice_epr, Gate2Q::CNOT);
                self.state.apply_single_qubit_gate(alice_qubit, Gate1Q::H);

                let m0 = self.state.measure(alice_qubit);
                let m1 = self.state.measure(alice_epr);

                // Send classical bits
                let alice_node = self.nodes.get(alice).ok_or(NetworkError::NodeNotFound)?;
                alice_node.send_bit(bob, m0).await?;
                alice_node.send_bit(bob, m1).await?;

                // Bob receives and applies corrections
                let bob_node = self.nodes.get_mut(bob).ok_or(NetworkError::NodeNotFound)?;
                let c0 = bob_node.recv_bit(alice).await?;
                let c1 = bob_node.recv_bit(alice).await?;

                if c1 == 1 {
                    self.state.apply_single_qubit_gate(bob_epr, Gate1Q::X);
                }
                if c0 == 1 {
                    self.state.apply_single_qubit_gate(bob_epr, Gate1Q::Z);
                }

                Ok(bob_epr)
            }
            NetworkMode::Distributed => {
                // Verify Alice owns the qubit
                let alice_node = self.nodes.get(alice).ok_or(NetworkError::NodeNotFound)?;
                if !alice_node
                    .qubit_allocator
                    .allocated_qubits
                    .contains(&alice_qubit)
                {
                    return Err(NetworkError::QubitNotOwned);
                }

                // Check if nodes are directly connected
                let direct_link = self.get_link(alice, bob).is_some();

                let (alice_epr, bob_epr) = if direct_link {
                    // Direct connection - create EPR pair
                    self.create_epr_pair(alice, bob)?
                } else {
                    // Multi-hop - establish end-to-end entanglement
                    self.establish_end_to_end_entanglement(alice, bob).await?
                };

                // Alice performs Bell measurement
                {
                    let alice_node = self
                        .nodes
                        .get_mut(alice)
                        .ok_or(NetworkError::NodeNotFound)?;
                    if let Some(system) = &mut alice_node.local_system {
                        tokio::task::block_in_place(|| {
                            tokio::runtime::Handle::current().block_on(async {
                                system.cnot(alice_qubit, alice_epr).await?;
                                system.h(alice_qubit).await?;
                                Ok::<(), QnectError>(())
                            })
                        })
                        .map_err(|_| NetworkError::GateApplicationFailed)?;
                    }
                }

                // Measure Alice's qubits
                let m0 = self.measure(alice, alice_qubit)?;
                let m1 = self.measure(alice, alice_epr)?;

                // Send classical bits to Bob (classical is always connected)
                let alice_node = self.nodes.get(alice).ok_or(NetworkError::NodeNotFound)?;
                alice_node.send_bit(bob, m0).await?;
                alice_node.send_bit(bob, m1).await?;

                // Bob receives and applies corrections
                let bob_node = self.nodes.get_mut(bob).ok_or(NetworkError::NodeNotFound)?;
                let c0 = bob_node.recv_bit(alice).await?;
                let c1 = bob_node.recv_bit(alice).await?;

                // Apply corrections
                if c1 == 1 {
                    self.apply_local_gate(bob, bob_epr, Gate1Q::X)?;
                }
                if c0 == 1 {
                    self.apply_local_gate(bob, bob_epr, Gate1Q::Z)?;
                }

                // Deallocate Alice's measured qubits
                {
                    let alice_node = self
                        .nodes
                        .get_mut(alice)
                        .ok_or(NetworkError::NodeNotFound)?;
                    alice_node
                        .deallocate_qubit(alice_qubit)
                        .map_err(|_| NetworkError::QubitNotAllocated)?;
                    alice_node
                        .deallocate_qubit(alice_epr)
                        .map_err(|_| NetworkError::QubitNotAllocated)?;
                }

                Ok(bob_epr)
            }
        }
    }

    pub async fn create_distributed_ghz(
        &mut self,
        nodes: Vec<&str>,
    ) -> Result<Vec<usize>, NetworkError> {
        if nodes.len() < 2 {
            return Err(NetworkError::InsufficientNodes);
        }

        match self.mode {
            NetworkMode::Legacy => {
                // Original implementation
                let mut ghz_qubits = Vec::new();

                for node in &nodes {
                    let qubit = self.get_free_qubit(node)?;
                    ghz_qubits.push(qubit);
                }

                let first_node = nodes[0];
                self.apply_local_gate(first_node, ghz_qubits[0], Gate1Q::H)?;

                for i in 1..ghz_qubits.len() {
                    self.state
                        .apply_two_qubit_gate(ghz_qubits[0], ghz_qubits[i], Gate2Q::CNOT);
                    self.entanglement.entangle(ghz_qubits[0], ghz_qubits[i]);
                }

                Ok(ghz_qubits)
            }
            NetworkMode::Distributed => {
                // NEW: Simple, correct GHZ creation
                let mut qubits = Vec::new();
                for n in &nodes {
                    let q = self
                        .nodes
                        .get_mut(*n)
                        .ok_or(NetworkError::NodeNotFound)?
                        .allocate_qubit()
                        .ok_or(NetworkError::NoFreeQubits)?;
                    qubits.push(((*n).to_string(), q));
                }

                // Register a shared GHZ resource across these qubits
                self.register_ghz(qubits.clone());

                Ok(qubits.into_iter().map(|(_, q)| q).collect())
            }
        }
    }

    // /// Create distributed GHZ state
    // pub async fn create_distributed_ghz(
    //     &mut self,
    //     nodes: Vec<&str>,
    // ) -> Result<Vec<usize>, NetworkError> {
    //     if nodes.len() < 2 {
    //         return Err(NetworkError::InsufficientNodes);
    //     }

    //     match self.mode {
    //         NetworkMode::Legacy => {
    //             // Original implementation
    //             let mut ghz_qubits = Vec::new();

    //             for node in &nodes {
    //                 let qubit = self.get_free_qubit(node)?;
    //                 ghz_qubits.push(qubit);
    //             }

    //             let first_node = nodes[0];
    //             self.apply_local_gate(first_node, ghz_qubits[0], Gate1Q::H)?;

    //             for i in 1..ghz_qubits.len() {
    //                 self.state
    //                     .apply_two_qubit_gate(ghz_qubits[0], ghz_qubits[i], Gate2Q::CNOT);
    //                 self.entanglement.entangle(ghz_qubits[0], ghz_qubits[i]);
    //             }

    //             Ok(ghz_qubits)
    //         }
    //         NetworkMode::Distributed => {
    //             // Distributed GHZ creation for arbitrary network topology
    //             let mut ghz_qubits = Vec::new();

    //             // Allocate one qubit per node
    //             for node in &nodes {
    //                 let node_mut = self
    //                     .nodes
    //                     .get_mut(*node)
    //                     .ok_or(NetworkError::NodeNotFound)?;
    //                 let qubit = node_mut
    //                     .allocate_qubit()
    //                     .ok_or(NetworkError::NoFreeQubits)?;
    //                 ghz_qubits.push((node.to_string(), qubit));
    //             }

    //             // Apply H to first node's qubit
    //             {
    //                 let (first_node, first_qubit) = &ghz_qubits[0];
    //                 self.apply_local_gate(first_node, *first_qubit, Gate1Q::H)?;
    //             }

    //             // Strategy: Use a star topology from first node
    //             // Create EPR pairs and use teleportation-based CNOT
    //             let (center_node, center_qubit) = &ghz_qubits[0];

    //             for i in 1..nodes.len() {
    //                 let (target_node, target_qubit) = &ghz_qubits[i];

    //                 // Check if nodes are directly connected
    //                 let direct_link = self
    //                     .links
    //                     .contains_key(&format!("{}-{}", center_node, target_node))
    //                     || self
    //                         .links
    //                         .contains_key(&format!("{}-{}", target_node, center_node));

    //                 if direct_link {
    //                     // Use remote CNOT for directly connected nodes
    //                     self.remote_cnot(center_node, *center_qubit, target_node, *target_qubit)
    //                         .await?;
    //                 } else {
    //                     // Find a path and use multi-hop operations
    //                     let path = self
    //                         .find_shortest_path(center_node, target_node)
    //                         .ok_or(NetworkError::NoConnection)?;

    //                     if path.len() == 2 {
    //                         // Direct connection (should have been caught above)
    //                         self.remote_cnot(
    //                             center_node,
    //                             *center_qubit,
    //                             target_node,
    //                             *target_qubit,
    //                         )
    //                         .await?;
    //                     } else {
    //                         // Multi-hop: establish end-to-end entanglement first
    //                         let (epr1, epr2) = self
    //                             .establish_end_to_end_entanglement(center_node, target_node)
    //                             .await?;

    //                         // Use EPR pair to perform remote CNOT
    //                         // Apply local operations at center node
    //                         {
    //                             let center_mut = self
    //                                 .nodes
    //                                 .get_mut(center_node)
    //                                 .ok_or(NetworkError::NodeNotFound)?;
    //                             if let Some(system) = &mut center_mut.local_system {
    //                                 tokio::task::block_in_place(|| {
    //                                     tokio::runtime::Handle::current().block_on(async {
    //                                         system.cnot(*center_qubit, epr1).await
    //                                     })
    //                                 })
    //                                 .map_err(|_| NetworkError::GateApplicationFailed)?;
    //                             }
    //                         }

    //                         // Measure EPR qubit at center
    //                         let m = self.measure(center_node, epr1)?;

    //                         // Send result to target
    //                         {
    //                             let center_ref = self
    //                                 .nodes
    //                                 .get(center_node)
    //                                 .ok_or(NetworkError::NodeNotFound)?;
    //                             center_ref.send_bit(target_node, m).await?;
    //                         }

    //                         // Target applies corrections and completes CNOT
    //                         {
    //                             let target_mut = self
    //                                 .nodes
    //                                 .get_mut(target_node)
    //                                 .ok_or(NetworkError::NodeNotFound)?;
    //                             let correction = target_mut.recv_bit(center_node).await?;

    //                             if let Some(system) = &mut target_mut.local_system {
    //                                 tokio::task::block_in_place(|| {
    //                                     tokio::runtime::Handle::current().block_on(async {
    //                                         if correction == 1 {
    //                                             system.x(epr2).await?;
    //                                         }
    //                                         system.cnot(epr2, *target_qubit).await?;
    //                                         Ok::<(), QnectError>(())
    //                                     })
    //                                 })
    //                                 .map_err(|_| NetworkError::GateApplicationFailed)?;
    //                             }

    //                             // Clean up EPR qubit
    //                             target_mut
    //                                 .deallocate_qubit(epr2)
    //                                 .map_err(|_| NetworkError::QubitNotAllocated)?;
    //                         }

    //                         // Clean up center's EPR qubit
    //                         {
    //                             let center_mut = self
    //                                 .nodes
    //                                 .get_mut(center_node)
    //                                 .ok_or(NetworkError::NodeNotFound)?;
    //                             center_mut
    //                                 .deallocate_qubit(epr1)
    //                                 .map_err(|_| NetworkError::QubitNotAllocated)?;
    //                         }
    //                     }
    //                 }
    //             }

    //             // Extract just the qubit indices for return
    //             Ok(ghz_qubits.into_iter().map(|(_, q)| q).collect())
    //         }
    //     }
    // }

    /// Apply CNOT between qubits on different nodes using teleportation
    pub async fn remote_cnot(
        &mut self,
        control_node: &str,
        control_qubit: usize,
        target_node: &str,
        target_qubit: usize,
    ) -> Result<(), NetworkError> {
        // Verify ownership
        let control_node_ref = self
            .nodes
            .get(control_node)
            .ok_or(NetworkError::NodeNotFound)?;
        if !control_node_ref
            .qubit_allocator
            .allocated_qubits
            .contains(&control_qubit)
        {
            return Err(NetworkError::QubitNotOwned);
        }

        let target_node_ref = self
            .nodes
            .get(target_node)
            .ok_or(NetworkError::NodeNotFound)?;
        if !target_node_ref
            .qubit_allocator
            .allocated_qubits
            .contains(&target_qubit)
        {
            return Err(NetworkError::QubitNotOwned);
        }

        // Gate teleportation protocol for CNOT
        // 1. Create two EPR pairs
        let (ctrl_epr1, tgt_epr1) = self.create_epr_pair(control_node, target_node)?;
        let (ctrl_epr2, tgt_epr2) = self.create_epr_pair(control_node, target_node)?;

        // 2. Control node operations
        {
            let control_node_mut = self
                .nodes
                .get_mut(control_node)
                .ok_or(NetworkError::NodeNotFound)?;
            if let Some(system) = &mut control_node_mut.local_system {
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        // Apply local CNOTs
                        system.cnot(control_qubit, ctrl_epr1).await?;
                        system.cnot(ctrl_epr2, control_qubit).await?;
                        system.cnot(ctrl_epr2, ctrl_epr1).await?;

                        // Measure EPR qubits
                        Ok::<(), QnectError>(())
                    })
                })
                .map_err(|_| NetworkError::GateApplicationFailed)?;
            }
        }

        // Measure and send results
        let m1 = self.measure(control_node, ctrl_epr1)?;
        let m2 = self.measure(control_node, ctrl_epr2)?;

        {
            let control_node_ref = self
                .nodes
                .get(control_node)
                .ok_or(NetworkError::NodeNotFound)?;
            control_node_ref.send_bit(target_node, m1).await?;
            control_node_ref.send_bit(target_node, m2).await?;
        }

        // 3. Target node receives and applies corrections
        {
            let target_node_mut = self
                .nodes
                .get_mut(target_node)
                .ok_or(NetworkError::NodeNotFound)?;
            let c1 = target_node_mut.recv_bit(control_node).await?;
            let c2 = target_node_mut.recv_bit(control_node).await?;

            if let Some(system) = &mut target_node_mut.local_system {
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        // Apply corrections
                        if c2 == 1 {
                            system.x(target_qubit).await?;
                        }
                        if c1 == 1 {
                            system.z(tgt_epr1).await?;
                        }

                        // Complete the remote CNOT
                        system.cnot(tgt_epr1, target_qubit).await?;
                        system.cnot(tgt_epr2, tgt_epr1).await?;

                        Ok::<(), QnectError>(())
                    })
                })
                .map_err(|_| NetworkError::GateApplicationFailed)?;
            }

            // Clean up EPR qubits
            target_node_mut
                .deallocate_qubit(tgt_epr1)
                .map_err(|_| NetworkError::QubitNotAllocated)?;
            target_node_mut
                .deallocate_qubit(tgt_epr2)
                .map_err(|_| NetworkError::QubitNotAllocated)?;
        }

        // Clean up control node EPR qubits
        {
            let control_node_mut = self
                .nodes
                .get_mut(control_node)
                .ok_or(NetworkError::NodeNotFound)?;
            control_node_mut
                .deallocate_qubit(ctrl_epr1)
                .map_err(|_| NetworkError::QubitNotAllocated)?;
            control_node_mut
                .deallocate_qubit(ctrl_epr2)
                .map_err(|_| NetworkError::QubitNotAllocated)?;
        }

        Ok(())
    }

    /// Generate NetQASM code from protocol history
    pub fn generate_netqasm(&self) -> HashMap<String, String> {
        let mut programs: HashMap<String, String> = HashMap::new();
        let mut declared_qubits: HashMap<String, HashSet<usize>> = HashMap::new();
        let mut epr_sockets: HashMap<String, HashSet<String>> = HashMap::new();

        // Initialize programs for each node
        for node_id in self.nodes.keys() {
            let mut program = String::new();
            program.push_str(&format!("# NetQASM program for node {}\n", node_id));
            program.push_str("# Generated by Qnect Quantum Network Simulator\n\n");
            program.push_str("from netqasm.sdk import *\n\n");
            program.push_str(&format!(
                "def protocol_{node_id}(app):\n",
                node_id = node_id
            ));
            programs.insert(node_id.clone(), program);
            declared_qubits.insert(node_id.clone(), HashSet::new());
            epr_sockets.insert(node_id.clone(), HashSet::new());
        }

        // First pass: identify all EPR connections
        let mut epr_peers: HashMap<String, HashSet<String>> = HashMap::new();
        for op in &self.protocol_history {
            if let NetworkOperation::CreateEPR { node1, node2, .. } = op {
                epr_peers
                    .entry(node1.clone())
                    .or_default()
                    .insert(node2.clone());
                epr_peers
                    .entry(node2.clone())
                    .or_default()
                    .insert(node1.clone());
            }
        }

        // Add all EPR socket declarations at the beginning
        for (node_id, peers) in &epr_peers {
            if let Some(prog) = programs.get_mut(node_id) {
                if !peers.is_empty() {
                    prog.push_str("    # Setup EPR sockets\n");
                    for peer in peers {
                        prog.push_str(&format!(
                            "    epr_socket_{} = app.setup_epr_socket('{}')\n",
                            peer.replace("-", "_"),
                            peer
                        ));
                        epr_sockets.get_mut(node_id).unwrap().insert(peer.clone());
                    }
                    prog.push_str("\n");
                }
            }
        }

        // Track measurements for each node (for return values)
        let mut node_measurements: HashMap<String, Vec<usize>> = HashMap::new();

        // Convert operations to NetQASM
        for op in &self.protocol_history {
            match op {
                NetworkOperation::CreateEPR {
                    node1,
                    node2,
                    q1,
                    q2,
                } => {
                    if let Some(prog) = programs.get_mut(node1) {
                        prog.push_str(&format!("    # Create EPR pair with {}\n", node2));
                        prog.push_str(&format!(
                            "    q{} = epr_socket_{}.create_keep()[0]\n",
                            q1,
                            node2.replace("-", "_")
                        ));
                        declared_qubits.get_mut(node1).unwrap().insert(*q1);
                    }
                    if let Some(prog) = programs.get_mut(node2) {
                        prog.push_str(&format!("    # Receive EPR pair from {}\n", node1));
                        prog.push_str(&format!(
                            "    q{} = epr_socket_{}.recv_keep()[0]\n",
                            q2,
                            node1.replace("-", "_")
                        ));
                        declared_qubits.get_mut(node2).unwrap().insert(*q2);
                    }
                }

                NetworkOperation::LocalGate { node, qubit, gate } => {
                    if let Some(prog) = programs.get_mut(node) {
                        // Check if qubit needs declaration
                        if !declared_qubits.get(node).unwrap().contains(qubit) {
                            prog.push_str(&format!("    # Allocate local qubit\n"));
                            prog.push_str(&format!("    q{} = app.allocate_qubit()\n", qubit));
                            declared_qubits.get_mut(node).unwrap().insert(*qubit);
                        }

                        // Apply the gate with proper mapping
                        match gate.as_str() {
                            g if g.starts_with("Rx(") => {
                                let angle = g.trim_start_matches("Rx(").trim_end_matches(")");
                                prog.push_str(&format!("    q{}.rx({})\n", qubit, angle));
                            }
                            g if g.starts_with("Ry(") => {
                                let angle = g.trim_start_matches("Ry(").trim_end_matches(")");
                                prog.push_str(&format!("    q{}.ry({})\n", qubit, angle));
                            }
                            g if g.starts_with("Rz(") => {
                                let angle = g.trim_start_matches("Rz(").trim_end_matches(")");
                                prog.push_str(&format!("    q{}.rz({})\n", qubit, angle));
                            }
                            other => {
                                let netqasm_gate = gate_to_netqasm(other);
                                if netqasm_gate.is_empty() {
                                    eprintln!(
                                        "Error: Cannot generate NetQASM for gate '{}'",
                                        other
                                    );
                                } else {
                                    prog.push_str(&format!("    q{}.{}()\n", qubit, netqasm_gate));
                                }
                            }
                        }
                    }
                }

                NetworkOperation::Measure {
                    node,
                    qubit,
                    result,
                } => {
                    if let Some(prog) = programs.get_mut(node) {
                        // Track measurements for return values
                        node_measurements
                            .entry(node.clone())
                            .or_default()
                            .push(*qubit);

                        // Only allocate if not already declared
                        if !declared_qubits.get(node).unwrap().contains(qubit) {
                            prog.push_str(&format!("    # Allocate qubit for measurement\n"));
                            prog.push_str(&format!("    q{} = app.allocate_qubit()\n", qubit));
                            declared_qubits.get_mut(node).unwrap().insert(*qubit);
                        }
                        prog.push_str(&format!("    # Measure qubit q{}\n", qubit));
                        prog.push_str(&format!("    m{} = q{}.measure()\n", qubit, qubit));
                        prog.push_str(&format!(
                            "    yield from app.flush()  # Result was: {}\n",
                            result
                        ));
                    }
                }

                NetworkOperation::AllocateLocal { node, qubit } => {
                    if let Some(prog) = programs.get_mut(node) {
                        if !declared_qubits.get(node).unwrap().contains(qubit) {
                            prog.push_str(&format!("    q{} = app.allocate_qubit()\n", qubit));
                            declared_qubits.get_mut(node).unwrap().insert(*qubit);
                        }
                    }
                }

                NetworkOperation::FreeLocal { node, qubit } => {
                    if let Some(prog) = programs.get_mut(node) {
                        if declared_qubits.get(node).unwrap().contains(qubit) {
                            prog.push_str(&format!("    q{}.free()\n", qubit));
                            declared_qubits.get_mut(node).unwrap().remove(qubit);
                        }
                    }
                }

                NetworkOperation::SendClassical { from, to, data } => {
                    if let Some(prog) = programs.get_mut(from) {
                        prog.push_str(&format!("    app.send_classical('{}', {:?})\n", to, data));
                        prog.push_str("    yield from app.flush()\n");
                    }
                }

                NetworkOperation::RecvClassical { node, from } => {
                    if let Some(prog) = programs.get_mut(node) {
                        prog.push_str(&format!("    data = app.receive_classical('{}')\n", from));
                    }
                }
            }
        }

        // Add return statements with measurement results
        for (node_id, prog) in programs.iter_mut() {
            if let Some(measurements) = node_measurements.get(node_id) {
                if !measurements.is_empty() {
                    prog.push_str("\n    # Return measurement results\n");
                    prog.push_str(&format!(
                        "    return {{{}}}\n",
                        measurements
                            .iter()
                            .map(|q| format!("'m{}': m{}", q, q))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                } else {
                    prog.push_str("\n    return {}\n");
                }
            } else {
                prog.push_str("\n    return {}\n");
            }
        }

        programs
    }

    /// Get network statistics
    pub fn get_stats(&self) -> NetworkStats {
        let total_allocated = if self.mode == NetworkMode::Distributed {
            self.nodes
                .values()
                .map(|n| n.qubit_allocator.allocated_qubits.len())
                .sum()
        } else {
            self.total_qubits
        };

        NetworkStats {
            mode: self.mode.clone(),
            total_nodes: self.nodes.len(),
            total_qubits: total_allocated,
            total_links: self.links.len(),
            operations_recorded: self.protocol_history.len(),
        }
    }
    /// Quantum Anonymous Transmission Protocol
    /// Allows a sender to anonymously transmit classical bits using shared GHZ states
    pub async fn anonymous_transmission(
        &mut self,
        sender: &str,
        participants: Vec<&str>,
        bit: u8,
    ) -> Result<u8, NetworkError> {
        if !participants.contains(&sender) {
            return Err(NetworkError::NodeNotFound);
        }

        // Step 1: Create shared GHZ state
        let ghz_qubits = self.create_distributed_ghz(participants.clone()).await?;

        // Step 2: Sender applies phase flip if bit = 1
        if let Some(sender_idx) = participants.iter().position(|&p| p == sender) {
            if bit == 1 {
                self.apply_local_gate(sender, ghz_qubits[sender_idx], Gate1Q::Z)?;
            }
        }

        // Step 3: All participants apply Hadamard and measure
        let mut local_measurements = HashMap::new();

        for (i, participant) in participants.iter().enumerate() {
            self.apply_local_gate(participant, ghz_qubits[i], Gate1Q::H)?;
            let m = self.measure(participant, ghz_qubits[i])?;
            local_measurements.insert(participant.to_string(), m);
        }

        // Step 4: Broadcast all measurements (classical broadcast)
        // Each participant sends their measurement to everyone else
        for sender_p in &participants {
            let m = local_measurements[*sender_p];
            for receiver_p in &participants {
                if sender_p != receiver_p {
                    self.send_classical_routed(sender_p, receiver_p, vec![m])
                        .await?;
                }
            }
        }

        // Step 5: Each participant collects all measurements
        let mut all_measurements = Vec::new();

        // Add local measurement first (for any participant)
        let example_participant = participants[0];
        all_measurements.push(local_measurements[example_participant]);

        // Receive from all others
        for sender_p in &participants {
            if sender_p != &example_participant {
                let node = self
                    .nodes
                    .get_mut(example_participant)
                    .ok_or(NetworkError::NodeNotFound)?;
                let m = node.recv_bit(sender_p).await?;
                all_measurements.push(m);
            }
        }

        // Step 6: Compute parity of ALL measurements
        let parity = all_measurements.iter().sum::<u8>() % 2;

        Ok(parity)
    }

    /// Anonymous Entanglement (AE) - Two anonymous parties share EPR pair
    pub async fn anonymous_entanglement(
        &mut self,
        sender: &str,
        receiver: &str,
        participants: Vec<&str>,
    ) -> Result<(usize, usize), NetworkError> {
        // Step 1: Create GHZ state
        let ghz_qubits = self.create_distributed_ghz(participants.clone()).await?;

        // Step 2: Everyone except sender and receiver applies H and measures
        let mut measurements = Vec::new();
        for (i, participant) in participants.iter().enumerate() {
            if *participant != sender && *participant != receiver {
                self.apply_local_gate(participant, ghz_qubits[i], Gate1Q::H)?;
                let m = self.measure(participant, ghz_qubits[i])?;
                measurements.push(m);

                // Broadcast to sender and receiver
                self.send_classical_routed(participant, sender, vec![m])
                    .await?;
                self.send_classical_routed(participant, receiver, vec![m])
                    .await?;
            }
        }

        // Step 3: Sender picks random bit and broadcasts
        let b = rand::random::<u8>() % 2;
        for participant in &participants {
            if participant != &sender {
                self.send_classical_routed(sender, participant, vec![b])
                    .await?;
            }
        }

        // Step 4: Sender applies phase flip if b = 1
        let sender_idx = participants.iter().position(|&p| p == sender).unwrap();
        if b == 1 {
            self.apply_local_gate(sender, ghz_qubits[sender_idx], Gate1Q::Z)?;
        }

        // Step 5: Receiver corrects based on parity
        let receiver_idx = participants.iter().position(|&p| p == receiver).unwrap();
        let parity = measurements.iter().sum::<u8>() % 2;

        if b ^ parity == 1 {
            self.apply_local_gate(receiver, ghz_qubits[receiver_idx], Gate1Q::Z)?;
        }

        if self.mode == NetworkMode::Distributed {
            // Remove from GHZ registry (already partially collapsed)
            self.nonlocal
                .ghz_by_qubit
                .remove(&(sender.to_string(), ghz_qubits[sender_idx]));
            self.nonlocal
                .ghz_by_qubit
                .remove(&(receiver.to_string(), ghz_qubits[receiver_idx]));

            // Register as a Bell pair
            self.register_bell(
                (sender.to_string(), ghz_qubits[sender_idx]),
                (receiver.to_string(), ghz_qubits[receiver_idx]),
            );

            // Apply Z correction if needed
            if b ^ parity == 1 {
                // This now updates the Bell's Pauli frame!
                self.apply_local_gate(receiver, ghz_qubits[receiver_idx], Gate1Q::Z)?;
            }
        }

        Ok((ghz_qubits[sender_idx], ghz_qubits[receiver_idx]))
    }

    /// Send qubits anonymously using anonymous entanglement + teleportation
    pub async fn anonymous_quantum_transmission(
        &mut self,
        sender: &str,
        receiver: &str,
        participants: Vec<&str>,
        sender_qubit: usize,
    ) -> Result<usize, NetworkError> {
        // Step 1: Establish anonymous entanglement
        let (sender_epr, receiver_epr) = self
            .anonymous_entanglement(sender, receiver, participants.clone())
            .await?;

        // Step 2: Teleport using the anonymous EPR pair
        // (Standard teleportation, but over anonymous channel)
        {
            let sender_node = self
                .nodes
                .get_mut(sender)
                .ok_or(NetworkError::NodeNotFound)?;
            if let Some(system) = &mut sender_node.local_system {
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        system.cnot(sender_qubit, sender_epr).await?;
                        system.h(sender_qubit).await?;
                        Ok::<(), QnectError>(())
                    })
                })
                .map_err(|_| NetworkError::GateApplicationFailed)?;
            }
        }

        // Measure at sender
        let m0 = self.measure(sender, sender_qubit)?;
        let m1 = self.measure(sender, sender_epr)?;

        // Send corrections anonymously (using anonymous transmission)
        self.anonymous_transmission(sender, participants.clone(), m0)
            .await?;
        self.anonymous_transmission(sender, participants.clone(), m1)
            .await?;

        // Receiver gets the corrections and applies them
        // (In real protocol, receiver would compute from broadcast)
        if m1 == 1 {
            self.apply_local_gate(receiver, receiver_epr, Gate1Q::X)?;
        }
        if m0 == 1 {
            self.apply_local_gate(receiver, receiver_epr, Gate1Q::Z)?;
        }

        Ok(receiver_epr)
    }
}

/// Network statistics
#[derive(Debug)]
pub struct NetworkStats {
    pub mode: NetworkMode,
    pub total_nodes: usize,
    pub total_qubits: usize,
    pub total_links: usize,
    pub operations_recorded: usize,
}

/// Network errors - Extended for new functionality
#[derive(Debug)]
pub enum NetworkError {
    NodeNotFound,
    NoFreeQubits,
    QubitNotOwned,
    InvalidOperation,
    NoConnection,
    FidelityTooLow,
    SendFailed,
    ConnectionClosed,
    EmptyMessage,
    InsufficientNodes,
    InvalidFidelity,
    InvalidGenerationRate,
    QubitNotAllocated,
    GateApplicationFailed,
    MeasurementFailed,
    NoLocalSystem,
    DistributedGHZNotImplemented,
}

impl std::fmt::Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkError::NodeNotFound => write!(f, "Node not found in network"),
            NetworkError::NoFreeQubits => write!(f, "No free qubits available"),
            NetworkError::InvalidOperation => write!(f, "Invalid operation"),
            NetworkError::FidelityTooLow => write!(f, "Fidelity too low"),
            NetworkError::QubitNotOwned => write!(f, "Qubit not owned by this node"),
            NetworkError::NoConnection => write!(f, "No connection between nodes"),
            NetworkError::SendFailed => write!(f, "Failed to send message"),
            NetworkError::ConnectionClosed => write!(f, "Connection closed"),
            NetworkError::EmptyMessage => write!(f, "Empty message received"),
            NetworkError::InvalidFidelity => write!(f, "Fidelity must be between 0 and 1"),
            NetworkError::InvalidGenerationRate => {
                write!(f, "Generation rate must be non-negative")
            }
            NetworkError::InsufficientNodes => write!(f, "Insufficient nodes for operation"),
            NetworkError::QubitNotAllocated => write!(f, "Qubit not allocated"),
            NetworkError::GateApplicationFailed => write!(f, "Failed to apply gate"),
            NetworkError::MeasurementFailed => write!(f, "Failed to measure qubit"),
            NetworkError::NoLocalSystem => write!(f, "Node has no local quantum system"),
            NetworkError::DistributedGHZNotImplemented => {
                write!(f, "Distributed GHZ not yet implemented")
            }
        }
    }
}

fn gate_to_netqasm(gate: &str) -> String {
    match gate {
        "H" => "h".to_string(),
        "X" => "x".to_string(),
        "Y" => "y".to_string(),
        "Z" => "z".to_string(),
        "S" => "s".to_string(),
        "T" => "t".to_string(),
        "CNOT" => "cnot".to_string(),
        "CZ" => "cz".to_string(),
        // Parameterized gates are handled separately
        g if g.starts_with("Rx(") => g.to_string(),
        g if g.starts_with("Ry(") => g.to_string(),
        g if g.starts_with("Rz(") => g.to_string(),
        // For unknown gates, log a warning or error
        other => {
            eprintln!("Warning: Unknown gate '{}' in NetQASM generation", other);
            other.to_lowercase() // Fallback
        }
    }
}

impl std::error::Error for NetworkError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_classical_communication() {
        let mut network = QuantumNetwork::new();
        network.add_node("Alice", 2);
        network.add_node("Bob", 2);

        // Test sending classical data
        let alice = network.nodes.get("Alice").unwrap();
        alice.send_classical("Bob", vec![1, 2, 3]).await.unwrap();

        let bob = network.nodes.get_mut("Bob").unwrap();
        let data = bob.recv_classical("Alice").await.unwrap();

        assert_eq!(data, vec![1, 2, 3]);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_epr_creation() {
        let mut network = QuantumNetwork::new_distributed();
        network
            .add_distributed_node("Alice", 2, BackendType::StateVector)
            .unwrap();
        network
            .add_distributed_node("Bob", 2, BackendType::StateVector)
            .unwrap();

        network
            .add_quantum_link(
                "Alice",
                "Bob",
                LinkType::Fiber {
                    length_km: 10.0,
                    loss_db_per_km: 0.2,
                },
                0.98,
                1000.0,
            )
            .unwrap();

        let (q1, q2) = network.create_epr_pair("Alice", "Bob").unwrap();

        // Check that a Bell pair was registered in the nonlocal store
        assert!(network.is_bell_qubit("Alice", q1).is_some());
        assert!(network.is_bell_qubit("Bob", q2).is_some());

        // Verify they share the same Bell ID
        let bell_id_1 = network.is_bell_qubit("Alice", q1).unwrap();
        let bell_id_2 = network.is_bell_qubit("Bob", q2).unwrap();
        assert_eq!(bell_id_1, bell_id_2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_netqasm_generation() {
        let mut network = QuantumNetwork::new_distributed();
        network
            .add_distributed_node("Alice", 2, BackendType::StateVector)
            .unwrap();
        network
            .add_distributed_node("Bob", 2, BackendType::StateVector)
            .unwrap();

        network
            .add_quantum_link(
                "Alice",
                "Bob",
                LinkType::Fiber {
                    length_km: 10.0,
                    loss_db_per_km: 0.2,
                },
                0.98,
                1000.0,
            )
            .unwrap();

        network.create_epr_pair("Alice", "Bob").unwrap();
        network.apply_local_gate("Alice", 0, Gate1Q::H).unwrap();

        let programs = network.generate_netqasm();

        // Check for the actual NetQASM commands used
        assert!(programs["Alice"].contains("create_keep"));
        assert!(programs["Bob"].contains("recv_keep"));
        assert!(programs["Alice"].contains("q0.h()"));
    }

    #[test]
    fn test_distributed_mode() {
        let mut network = QuantumNetwork::new_distributed();

        // Add nodes with different backends
        network
            .add_distributed_node("Alice", 10, BackendType::StateVector)
            .unwrap();
        network
            .add_distributed_node("Bob", 1000, BackendType::Stabilizer)
            .unwrap();

        // Bob can have 1000 qubits with stabilizer backend!
        assert_eq!(network.nodes["Bob"].qubit_allocator.total_qubits, 1000);
    }

    #[test]
    fn test_qubit_allocation() {
        let mut node = QuantumNode::new("test");
        node.qubit_allocator = QubitAllocator::new(5);

        // Allocate some qubits
        let q1 = node.allocate_qubit().unwrap();
        let _q2 = node.allocate_qubit().unwrap();

        assert_eq!(node.qubit_allocator.get_free_count(), 3);

        // Deallocate
        node.deallocate_qubit(q1).unwrap();
        assert_eq!(node.qubit_allocator.get_free_count(), 4);
    }
}
