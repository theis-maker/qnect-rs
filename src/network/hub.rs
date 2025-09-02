//! Quantum hub implementation for routing and resource distribution

use crate::error::{QnectError, Result};
use crate::network::network::LinkType;
use crate::network::node_types::{NodeType, RoutingStrategy};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Information about a connection to the hub
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub link_type: LinkType,
    pub established: Instant,
    pub reserved_qubits: usize,
    pub total_routed: usize,
    pub avg_fidelity: f64,
}

/// Metrics for a connection
#[derive(Debug, Clone, Default)]
pub struct ConnectionMetrics {
    pub latency_ms: f64,
    pub throughput: f64,
    pub error_rate: f64,
    pub uptime: f64,
}

/// EPR pair pool for efficient distribution
pub struct EPRPool {
    pairs: Vec<(usize, usize)>,
    capacity: usize,
}

impl EPRPool {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            pairs: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn add_pair(&mut self, q1: usize, q2: usize) {
        if self.pairs.len() < self.capacity {
            self.pairs.push((q1, q2));
        }
    }

    pub fn get_pair(&mut self) -> Option<(usize, usize)> {
        self.pairs.pop()
    }

    pub fn available(&self) -> usize {
        self.pairs.len()
    }
}

/// A quantum hub node for routing and distribution
pub struct QuantumHub {
    pub name: String,
    pub location: (f64, f64),
    pub node_type: NodeType,

    // Core hub functionality
    connected_nodes: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
    routing_table: Arc<RwLock<RoutingTable>>,
    epr_pool: Arc<RwLock<EPRPool>>,

    // Resource management
    available_qubits: Arc<RwLock<usize>>,
    active_connections: Arc<RwLock<HashMap<(String, String), ConnectionState>>>,

    // Metrics
    total_routed: Arc<RwLock<usize>>,
    avg_fidelity: Arc<RwLock<f64>>,
}

/// State of an active connection through the hub
#[derive(Debug, Clone)]
pub struct ConnectionState {
    pub established: Instant,
    pub qubits_used: usize,
    pub can_reroute: bool,
}

/// Routing table for the hub
pub struct RoutingTable {
    strategy: RoutingStrategy,
    routes: HashMap<(String, String), Route>,
    node_metrics: HashMap<String, NodeMetrics>,
}

/// A route between two nodes
#[derive(Debug, Clone)]
pub struct Route {
    pub path: Vec<String>,
    pub total_distance: f64,
    pub expected_fidelity: f64,
    pub hops: usize,
}

/// Metrics for a node
#[derive(Debug, Clone, Default)]
pub struct NodeMetrics {
    pub load: f64,
    pub avg_latency: f64,
    pub success_rate: f64,
}

impl RoutingTable {
    pub fn new(strategy: RoutingStrategy) -> Self {
        Self {
            strategy,
            routes: HashMap::new(),
            node_metrics: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: &str, metrics: NodeMetrics) {
        self.node_metrics.insert(node.to_string(), metrics);
    }

    pub fn update_route(&mut self, from: &str, to: &str, route: Route) {
        self.routes
            .insert((from.to_string(), to.to_string()), route);
    }

    pub fn get_route(&self, from: &str, to: &str) -> Option<&Route> {
        self.routes.get(&(from.to_string(), to.to_string()))
    }
}

impl QuantumHub {
    /// Create a new quantum hub
    pub async fn new(
        name: &str,
        location: (f64, f64),
        capacity: usize,
        strategy: RoutingStrategy,
    ) -> Result<Self> {
        Ok(Self {
            name: name.to_string(),
            location,
            node_type: NodeType::Hub {
                capacity,
                routing_strategy: strategy.clone(),
            },
            connected_nodes: Arc::new(RwLock::new(HashMap::new())),
            routing_table: Arc::new(RwLock::new(RoutingTable::new(strategy))),
            epr_pool: Arc::new(RwLock::new(EPRPool::with_capacity(capacity * 10))),
            available_qubits: Arc::new(RwLock::new(capacity * 4)),
            active_connections: Arc::new(RwLock::new(HashMap::new())),
            total_routed: Arc::new(RwLock::new(0)),
            avg_fidelity: Arc::new(RwLock::new(1.0)),
        })
    }

    /// Get hub capacity
    pub fn get_capacity(&self) -> usize {
        match &self.node_type {
            NodeType::Hub { capacity, .. } => *capacity,
            _ => 0,
        }
    }

    /// Accept a connection from a node
    pub async fn accept_connection(&self, node: &str, link: LinkType) -> Result<()> {
        let mut nodes = self.connected_nodes.write().await;

        // Check capacity
        if nodes.len() >= self.get_capacity() {
            return Err(QnectError::NetworkError {
                message: format!("Hub {} is at full capacity", self.name),
            });
        }

        // Add to routing table
        let mut routing = self.routing_table.write().await;
        routing.add_node(node, NodeMetrics::default());

        // Reserve qubits
        let reserved = 4;
        let mut available = self.available_qubits.write().await;
        if *available < reserved {
            return Err(QnectError::InsufficientQubits);
        }
        *available -= reserved;

        // Add connection
        nodes.insert(node.to_string(), ConnectionInfo {
            link_type: link,
            established: Instant::now(),
            reserved_qubits: reserved,
            total_routed: 0,
            avg_fidelity: 1.0,
        });

        log::info!(" Hub {} accepted connection from {}", self.name, node);
        Ok(())
    }

    /// Route a quantum state between two nodes
    pub async fn route_quantum_state(&self, from: &str, to: &str, qubit: usize) -> Result<usize> {
        // Check if both nodes are connected
        let nodes = self.connected_nodes.read().await;
        if !nodes.contains_key(from) || !nodes.contains_key(to) {
            return Err(QnectError::NetworkError {
                message: format!("Cannot route: {} or {} not connected to hub", from, to),
            });
        }
        drop(nodes);

        // Update metrics
        let mut total = self.total_routed.write().await;
        *total += 1;

        // Log routing
        log::debug!(
            "📡 Hub {} routing: {} → {} (qubit {})",
            self.name,
            from,
            to,
            qubit
        );

        // In real implementation, would perform actual quantum operations
        // For now, return a new qubit ID representing the routed state
        Ok(qubit + 1000) // Placeholder for routed qubit
    }

    /// Get current load (0.0 to 1.0)
    pub async fn get_load(&self) -> f64 {
        let nodes = self.connected_nodes.read().await;
        let connections = self.active_connections.read().await;

        let node_load = nodes.len() as f64 / self.get_capacity() as f64;
        let conn_load = connections.len() as f64 / (self.get_capacity() * 2) as f64;

        (node_load + conn_load) / 2.0
    }

    /// Get hub statistics
    pub async fn get_stats(&self) -> HubStats {
        HubStats {
            name: self.name.clone(),
            connected_nodes: self.connected_nodes.read().await.len(),
            total_routed: *self.total_routed.read().await,
            avg_fidelity: *self.avg_fidelity.read().await,
            current_load: self.get_load().await,
            available_qubits: *self.available_qubits.read().await,
        }
    }
}

/// Statistics for a hub
#[derive(Debug, Clone)]
pub struct HubStats {
    pub name: String,
    pub connected_nodes: usize,
    pub total_routed: usize,
    pub avg_fidelity: f64,
    pub current_load: f64,
    pub available_qubits: usize,
}
