//! Node types for quantum networks: Endpoints, Repeaters, and Hubs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    /// Endpoint nodes generate or consume quantum states
    Endpoint { role: EndpointRole },

    /// Repeater nodes extend quantum communication range
    Repeater {
        max_distance: f64,
        swap_efficiency: f64,
    },

    /// Hub nodes route and distribute quantum resources
    Hub {
        capacity: usize,
        routing_strategy: RoutingStrategy,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EndpointRole {
    /// Only initiates quantum connections
    Client,

    /// Only receives quantum connections
    Server,

    /// Can both initiate and receive
    Peer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoutingStrategy {
    /// Choose path with minimum distance
    ShortestPath,

    /// Choose path with least traffic
    LeastCongested,

    /// Choose path with highest fidelity
    HighestFidelity,

    /// Distribute load evenly
    RoundRobin,
}

impl Default for NodeType {
    fn default() -> Self {
        NodeType::Endpoint {
            role: EndpointRole::Peer,
        }
    }
}

impl NodeType {
    /// Create a client endpoint
    pub fn client() -> Self {
        NodeType::Endpoint {
            role: EndpointRole::Client,
        }
    }

    /// Create a server endpoint
    pub fn server() -> Self {
        NodeType::Endpoint {
            role: EndpointRole::Server,
        }
    }

    /// Create a standard repeater
    pub fn repeater() -> Self {
        NodeType::Repeater {
            max_distance: 1000.0, // km
            swap_efficiency: 0.85,
        }
    }

    /// Create a standard hub
    pub fn hub() -> Self {
        NodeType::Hub {
            capacity: 100,
            routing_strategy: RoutingStrategy::ShortestPath,
        }
    }

    /// Check if node can initiate connections
    pub fn can_initiate(&self) -> bool {
        match self {
            NodeType::Endpoint { role } => match role {
                EndpointRole::Client | EndpointRole::Peer => true,
                EndpointRole::Server => false,
            },
            NodeType::Hub { .. } => true,
            NodeType::Repeater { .. } => false,
        }
    }

    /// Check if node can receive connections
    pub fn can_receive(&self) -> bool {
        match self {
            NodeType::Endpoint { role } => match role {
                EndpointRole::Server | EndpointRole::Peer => true,
                EndpointRole::Client => false,
            },
            NodeType::Hub { .. } | NodeType::Repeater { .. } => true,
        }
    }
}
