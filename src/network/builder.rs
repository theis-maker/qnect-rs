//! Fluent API for building quantum networks

use log::info;

use crate::builder::BackendType;
use crate::error::QnectError;
use crate::network::{
    network::{LinkType, QuantumNetwork},
    node_types::RoutingStrategy,
};

/// Topology templates for common network architectures
#[derive(Debug, Clone)]
pub enum Topology {
    /// Star topology with central hub
    Star {
        hub_name: String,
        hub_capacity: usize,
    },
    /// Ring topology where each node connects to two neighbors
    Ring,
    /// Full mesh where everyone connects to everyone
    Mesh { link_fidelity: f64 },
    /// Linear chain of nodes
    Line,
    /// Hierarchical with multiple hub levels
    Hierarchical {
        central_hub: String,
        regional_hubs: Vec<String>,
    },
}

/// Node specification for the builder
#[derive(Debug, Clone)]
struct NodeSpec {
    name: String,
    qubits: usize,
    backend: BackendType,
    location: Option<(f64, f64)>,
}

/// Hub specification for the builder
#[derive(Debug, Clone)]
struct HubSpec {
    name: String,
    capacity: usize,
    strategy: RoutingStrategy,
    location: (f64, f64),
}

/// Builder for quantum networks with fluent API
pub struct NetworkBuilder {
    nodes: Vec<NodeSpec>,
    hubs: Vec<HubSpec>,
    links: Vec<(String, String, LinkType, f64)>, // (from, to, type, fidelity)
    topology: Option<Topology>,
    default_link_type: LinkType,
    default_fidelity: f64,
}

impl NetworkBuilder {
    /// Create a new network builder
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            hubs: Vec::new(),
            links: Vec::new(),
            topology: None,
            default_link_type: LinkType::Fiber {
                length_km: 1.0,
                loss_db_per_km: 0.2,
            },
            default_fidelity: 0.95,
        }
    }

    /// Set the network topology
    pub fn with_topology(mut self, topology: Topology) -> Self {
        self.topology = Some(topology);
        self
    }

    /// Add an endpoint node
    pub fn add_endpoint(mut self, name: impl Into<String>, qubits: usize) -> Self {
        self.nodes.push(NodeSpec {
            name: name.into(),
            qubits,
            backend: BackendType::Stabilizer,
            location: None,
        });
        self
    }

    /// Add an endpoint with specific backend
    pub fn add_endpoint_with_backend(
        mut self,
        name: impl Into<String>,
        qubits: usize,
        backend: BackendType,
    ) -> Self {
        self.nodes.push(NodeSpec {
            name: name.into(),
            qubits,
            backend,
            location: None,
        });
        self
    }

    /// Add a hub to the network
    pub fn add_hub(mut self, name: impl Into<String>, capacity: usize) -> Self {
        self.hubs.push(HubSpec {
            name: name.into(),
            capacity,
            strategy: RoutingStrategy::ShortestPath,
            location: (0.0, 0.0),
        });
        self
    }

    /// Add a hub with custom strategy
    pub fn add_hub_with_strategy(
        mut self,
        name: impl Into<String>,
        capacity: usize,
        strategy: RoutingStrategy,
    ) -> Self {
        self.hubs.push(HubSpec {
            name: name.into(),
            capacity,
            strategy,
            location: (0.0, 0.0),
        });
        self
    }

    /// Set default link type for all connections
    pub fn with_link_type(mut self, link_type: LinkType) -> Self {
        self.default_link_type = link_type;
        self
    }

    /// Set default fidelity for all connections
    pub fn with_fidelity(mut self, fidelity: f64) -> Self {
        self.default_fidelity = fidelity;
        self
    }

    /// Add a custom link between specific nodes
    pub fn add_link(
        mut self,
        from: impl Into<String>,
        to: impl Into<String>,
        link_type: LinkType,
        fidelity: f64,
    ) -> Self {
        self.links
            .push((from.into(), to.into(), link_type, fidelity));
        self
    }

    /// Build the quantum network
    pub fn build(self) -> Result<QuantumNetwork, QnectError> {
        let mut network = QuantumNetwork::new_distributed();

        // Add all hubs first
        for hub in &self.hubs {
            network
                .add_hub_with_config(&hub.name, hub.location, hub.capacity, hub.strategy.clone())
                .map_err(|_| {
                    QnectError::invalid_operation(
                        "network_build".to_owned(),
                        "Failed to add hub to network".to_string(),
                    )
                })?;
            info!("Added hub: {} (capacity: {})", hub.name, hub.capacity);
        }

        // Add all nodes
        for node in &self.nodes {
            network.add_distributed_node(&node.name, node.qubits, node.backend.clone())?;

            info!("Added node: {} ({} qubits)", node.name, node.qubits);
        }

        // Apply topology if specified
        if let Some(topology) = &self.topology {
            self.apply_topology(&mut network, topology)?;
        }

        // Add any custom links
        for (from, to, link_type, fidelity) in &self.links {
            network
                .add_quantum_link(
                    from,
                    to,
                    link_type.clone(),
                    *fidelity,
                    1000.0, // Default generation rate
                )
                .map_err(|_| {
                    QnectError::invalid_operation(
                        "network_build".to_owned(),
                        "Failed to add hub to network".to_string(),
                    )
                })?;
            info!("Connected {} ← → {}", from, to);
        }

        Ok(network)
    }

    /// Apply topology pattern to the network
    fn apply_topology(
        &self,
        network: &mut QuantumNetwork,
        topology: &Topology,
    ) -> Result<(), QnectError> {
        match topology {
            Topology::Star { hub_name, .. } => {
                // Connect all nodes to the central hub
                for node in &self.nodes {
                    network
                        .connect_to_hub(&node.name, hub_name, self.default_link_type.clone())
                        .map_err(|_| {
                            QnectError::invalid_operation(
                                "network_build".to_owned(),
                                "Failed to add hub to network".to_string(),
                            )
                        })?;
                    info!("Connected {} to hub {}", node.name, hub_name);
                }
            }

            Topology::Ring => {
                // Connect each node to its neighbors in a ring
                let all_names: Vec<String> = self.nodes.iter().map(|n| n.name.clone()).collect();

                for i in 0..all_names.len() {
                    let next = (i + 1) % all_names.len();
                    network
                        .add_quantum_link(
                            &all_names[i],
                            &all_names[next],
                            self.default_link_type.clone(),
                            self.default_fidelity,
                            1000.0,
                        )
                        .map_err(|_| {
                            QnectError::invalid_operation(
                                "network_build".to_owned(),
                                "Failed to add hub to network".to_string(),
                            )
                        })?;
                    info!("Ring: {} → {}", all_names[i], all_names[next]);
                }
            }

            Topology::Mesh { link_fidelity } => {
                // Connect everyone to everyone
                let all_names: Vec<String> = self.nodes.iter().map(|n| n.name.clone()).collect();

                for i in 0..all_names.len() {
                    for j in i + 1..all_names.len() {
                        network
                            .add_quantum_link(
                                &all_names[i],
                                &all_names[j],
                                self.default_link_type.clone(),
                                *link_fidelity,
                                1000.0,
                            )
                            .map_err(|_| {
                                QnectError::invalid_operation(
                                    "network_build".to_owned(),
                                    "Failed to add hub to network".to_string(),
                                )
                            })?;
                        info!("Mesh: {} ← → {}", all_names[i], all_names[j]);
                    }
                }
            }

            Topology::Line => {
                // Linear chain
                let all_names: Vec<String> = self.nodes.iter().map(|n| n.name.clone()).collect();

                for i in 0..all_names.len() - 1 {
                    network
                        .add_quantum_link(
                            &all_names[i],
                            &all_names[i + 1],
                            self.default_link_type.clone(),
                            self.default_fidelity,
                            1000.0,
                        )
                        .map_err(|_| {
                            QnectError::invalid_operation(
                                "network_build".to_owned(),
                                "Failed to add hub to network".to_string(),
                            )
                        })?;
                    info!("Line: {} - {}", all_names[i], all_names[i + 1]);
                }
            }

            Topology::Hierarchical {
                central_hub,
                regional_hubs,
            } => {
                // Connect regional hubs to central hub
                for regional in regional_hubs {
                    network
                        .add_quantum_link(
                            regional,
                            central_hub,
                            self.default_link_type.clone(),
                            0.98, // High fidelity for backbone
                            10000.0,
                        )
                        .map_err(|_| {
                            QnectError::invalid_operation(
                                "network_build".to_owned(),
                                "Failed to add hub to network".to_string(),
                            )
                        })?;
                    info!("Connected regional {} to central {}", regional, central_hub);
                }

                // Connect nodes to nearest regional hub (simplified)
                // In real implementation, would use location data
                for (i, node) in self.nodes.iter().enumerate() {
                    let hub_idx = i % regional_hubs.len();
                    network
                        .connect_to_hub(
                            &node.name,
                            &regional_hubs[hub_idx],
                            self.default_link_type.clone(),
                        )
                        .map_err(|_| {
                            QnectError::invalid_operation(
                                "network_build".to_owned(),
                                "Failed to add hub to network".to_string(),
                            )
                        })?;
                    info!(
                        "Connected {} to regional hub {}",
                        node.name, regional_hubs[hub_idx]
                    );
                }
            }
        }

        Ok(())
    }
}

impl Default for NetworkBuilder {
    fn default() -> Self {
        Self::new()
    }
}
