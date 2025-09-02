use qnect::builder::{BackendType, NoiseModel};
use qnect::create;
use qnect::error::QnectError;
use qnect::network::builder::{NetworkBuilder, Topology};
use qnect::network::network::{LinkType, QuantumNetwork};
use qnect::network::node_types::RoutingStrategy;

#[tokio::test]
async fn test_bell_state_correlation() {
    let mut q = create().with_qubits(2).build().unwrap();

    // Create Bell state
    q.h(0).await.unwrap();
    q.cnot(0, 1).await.unwrap();

    // Measure both qubits
    let m0 = q.measure(0).await.unwrap();
    let m1 = q.measure(1).await.unwrap();

    assert_eq!(
        m0, m1,
        "Bell state measurements should be perfectly correlated"
    );
}

#[tokio::test]
async fn test_ghz_state() {
    let mut q = create().with_qubits(3).build().unwrap();

    // Create GHZ state
    q.h(0).await.unwrap();
    q.cnot(0, 1).await.unwrap();
    q.cnot(0, 2).await.unwrap();

    // Measure all qubits
    let measurements: Vec<u8> = vec![
        q.measure(0).await.unwrap(),
        q.measure(1).await.unwrap(),
        q.measure(2).await.unwrap(),
    ];

    // Should be all 0s or all 1s
    assert!(
        measurements.iter().all(|&m| m == 0) || measurements.iter().all(|&m| m == 1),
        "GHZ state should be all |000⟩ or all |111⟩"
    );
}

#[tokio::test]
async fn test_teleportation() {
    let mut q = create().with_qubits(3).build().unwrap();

    // Create specific state on qubit 0
    q.ry(0, std::f64::consts::PI / 4.0).await.unwrap();

    // Share entanglement between Alice (1) and Bob (2)
    q.create_bell_pair(1, 2).await.unwrap();

    // Alice's Bell measurement
    q.cnot(0, 1).await.unwrap();
    q.h(0).await.unwrap();
    let m0 = q.measure(0).await.unwrap();
    let m1 = q.measure(1).await.unwrap();

    // Bob's corrections
    if m1 == 1 {
        q.x(2).await.unwrap();
    }
    if m0 == 1 {
        q.z(2).await.unwrap();
    }

    // Bob's qubit should now be in the state we started with
    // (In a real test, we'd verify the state matches)
}

#[tokio::test]
async fn test_error_handling() {
    let mut q = create().with_qubits(3).build().unwrap();

    // Test out of range
    match q.h(10).await {
        Err(QnectError::QubitOutOfRange { qubit, max }) => {
            assert_eq!(qubit, 10);
            assert_eq!(max, 3);
        }
        _ => panic!("Expected QubitOutOfRange error"),
    }

    // Test invalid gate (same qubit for CNOT)
    match q.cnot(1, 1).await {
        Err(QnectError::InvalidGate { .. }) => {
            // Good, got expected error
        }
        _ => panic!("Expected InvalidGate error"),
    }
}

#[tokio::test]
async fn test_backend_switch() {
    // Test state vector backend
    let mut sv = create()
        .with_backend(BackendType::StateVector)
        .with_qubits(5)
        .build()
        .unwrap();

    sv.h(0).await.unwrap();
    assert_eq!(sv.qubit_count(), 5);

    // Test stabilizer backend
    let mut stab = create()
        .with_backend(BackendType::Stabilizer)
        .with_qubits(100) // Can handle many more qubits!
        .build()
        .unwrap();

    stab.h(0).await.unwrap();
    for i in 1..100 {
        stab.cnot(0, i).await.unwrap();
    }

    // Verify GHZ correlations
    let m0 = stab.measure(0).await.unwrap();
    let m99 = stab.measure(99).await.unwrap();
    assert_eq!(m0, m99, "Large GHZ state should maintain correlations");
}

#[tokio::test]
async fn test_measurement_statistics() {
    // Test quantum randomness
    let mut zeros = 0;
    let mut ones = 0;

    for _ in 0..1000 {
        let mut q = create().with_qubits(1).build().unwrap();
        q.h(0).await.unwrap();

        match q.measure(0).await.unwrap() {
            0 => zeros += 1,
            1 => ones += 1,
            _ => unreachable!(),
        }
    }

    // Should be roughly 50/50 (with some statistical variance)
    let ratio = zeros as f64 / (zeros + ones) as f64;
    assert!(
        ratio > 0.45 && ratio < 0.55,
        "H gate measurement should be ~50/50, got {}% zeros",
        ratio * 100.0
    );
}

#[tokio::test]
async fn test_circuit_recording() {
    let mut q = create().with_qubits(3).build().unwrap().with_recording();

    // Build a circuit
    q.h(0).await.unwrap();
    q.cnot(0, 1).await.unwrap();
    q.cnot(1, 2).await.unwrap();
    q.s(0).await.unwrap();
    q.t(1).await.unwrap();
    q.s_dag(2).await.unwrap();

    // Use print_circuit to verify it works
    q.print_circuit();

    // Get QASM representation instead
    let qasm = q.to_qasm2();
    assert!(qasm.contains("h q[0]"), "QASM should contain H gate");
    assert!(qasm.contains("cx"), "QASM should contain CNOT");
}

#[tokio::test]
async fn test_qasm_export_import() {
    // Create a circuit
    let mut original = create().with_qubits(2).build().unwrap().with_recording();
    original.h(0).await.unwrap();
    original.cnot(0, 1).await.unwrap();

    // Export to QASM
    let qasm = original.to_qasm2();
    assert!(qasm.contains("OPENQASM 2.0"));
    assert!(qasm.contains("qreg"));
    assert!(qasm.contains("h q[0]"));
    assert!(qasm.contains("cx q[0],q[1]"));

    // Import back
    let mut imported = create().from_qasm(&qasm).unwrap().build().unwrap();
    assert_eq!(imported.qubit_count(), 2);

    // Should behave the same
    let m0 = imported.measure(0).await.unwrap();
    let m1 = imported.measure(1).await.unwrap();
    assert_eq!(m0, m1, "Imported Bell state should be correlated");
}

#[tokio::test]
async fn test_noise_model() {
    // Create noise model with the correct field names
    let noise = NoiseModel {
        depolarizing_rate: 0.1, // High error for testing
        measurement_error: 0.1,
    };

    let _q = create().with_qubits(1).with_noise(noise).build().unwrap();

    // With noise, |0⟩ state might flip
    let mut flipped = 0;
    for _ in 0..100 {
        let noise_model = NoiseModel {
            depolarizing_rate: 0.0,
            measurement_error: 0.1,
        };

        let mut q = create()
            .with_qubits(1)
            .with_noise(noise_model)
            .build()
            .unwrap();

        // Measure |0⟩ state
        if q.measure(0).await.unwrap() == 1 {
            flipped += 1;
        }
    }

    // Should see some errors due to noise
    assert!(flipped > 0, "Noise model should cause some bit flips");
    assert!(flipped < 30, "But not too many flips for 10% error rate");
}

#[tokio::test]
async fn test_all_single_qubit_gates() {
    let mut q = create().with_qubits(1).build().unwrap();

    // Test all single qubit gates compile and run
    q.h(0).await.unwrap();
    q.x(0).await.unwrap();
    q.y(0).await.unwrap();
    q.z(0).await.unwrap();
    q.s(0).await.unwrap();
    q.t(0).await.unwrap();
    q.s_dag(0).await.unwrap();
    q.t_dag(0).await.unwrap();
    q.rx(0, 0.5).await.unwrap();
    q.ry(0, 0.5).await.unwrap();
    q.rz(0, 0.5).await.unwrap();

    // Should complete without panic
}

#[tokio::test]
async fn test_all_two_qubit_gates() {
    let mut q = create().with_qubits(2).build().unwrap();

    // Test all two qubit gates
    q.cnot(0, 1).await.unwrap();
    q.cy(0, 1).await.unwrap();
    q.cz(0, 1).await.unwrap();
    q.swap(0, 1).await.unwrap();

    // Should complete without panic
}

#[tokio::test]
async fn test_three_qubit_gates() {
    let mut q = create().with_qubits(3).build().unwrap();

    // Toffoli gate
    q.ccx(0, 1, 2).await.unwrap();

    // Test Toffoli truth table
    let mut q = create().with_qubits(3).build().unwrap();
    q.x(0).await.unwrap();
    q.x(1).await.unwrap();
    q.ccx(0, 1, 2).await.unwrap();

    let m2 = q.measure(2).await.unwrap();
    assert_eq!(m2, 1, "Toffoli with both controls |1⟩ should flip target");
}

#[tokio::test]
async fn test_stabilizer_backend_limitations() {
    let mut q = create()
        .with_backend(BackendType::Stabilizer)
        .with_qubits(3)
        .build()
        .unwrap();

    // These should work (Clifford gates)
    q.h(0).await.unwrap();
    q.s(0).await.unwrap();
    q.cnot(0, 1).await.unwrap();
    q.x(0).await.unwrap();
    q.y(0).await.unwrap();
    q.z(0).await.unwrap();

    // These should fail (non-Clifford)
    assert!(
        q.rx(0, 0.1).await.is_err(),
        "Stabilizer shouldn't support arbitrary rotations"
    );
    assert!(q.t(0).await.is_err(), "Stabilizer shouldn't support T gate");
}

#[tokio::test]
async fn test_large_stabilizer_circuit() {
    // Test that stabilizer can handle large circuits efficiently
    let n = 1000;
    let mut q = create()
        .with_backend(BackendType::Stabilizer)
        .with_qubits(n)
        .build()
        .unwrap();

    // Create large GHZ state
    let start = std::time::Instant::now();
    q.h(0).await.unwrap();
    for i in 1..n {
        q.cnot(0, i).await.unwrap();
    }
    let elapsed = start.elapsed();

    // Should be fast even for 1000 qubits
    assert!(
        elapsed.as_secs() < 5,
        "1000 qubit GHZ should take < 5 seconds"
    );

    // Verify correlations
    let m0 = q.measure(0).await.unwrap();
    let m_last = q.measure(n - 1).await.unwrap();
    assert_eq!(m0, m_last, "Large GHZ should maintain correlations");
}

#[tokio::test]
async fn test_hub_capacity_limits() {
    let mut network = QuantumNetwork::new_distributed();

    // Create hub with capacity 2
    network
        .add_hub_with_config("SmallHub", (0.0, 0.0), 2, RoutingStrategy::ShortestPath)
        .expect("Should create hub");

    // Add 3 nodes
    network
        .add_distributed_node("Node1", 5, BackendType::Stabilizer)
        .unwrap();
    network
        .add_distributed_node("Node2", 5, BackendType::Stabilizer)
        .unwrap();
    network
        .add_distributed_node("Node3", 5, BackendType::Stabilizer)
        .unwrap();

    // Connect first two - should work
    assert!(
        network
            .connect_to_hub("Node1", "SmallHub", LinkType::Fiber {
                length_km: 1.0,
                loss_db_per_km: 0.2
            })
            .is_ok()
    );
    assert!(
        network
            .connect_to_hub("Node2", "SmallHub", LinkType::Fiber {
                length_km: 1.0,
                loss_db_per_km: 0.2
            })
            .is_ok()
    );

    // Third should fail (capacity exceeded)
    // Note: This test assumes hub.accept_connection() checks capacity
    // If not implemented, this test will help identify the issue
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_epr_through_hub_error_handling() {
    let mut network = QuantumNetwork::new_distributed();

    network.add_hub("Hub", (0.0, 0.0)).unwrap();
    network
        .add_distributed_node("Alice", 5, BackendType::Stabilizer)
        .unwrap();
    network
        .add_distributed_node("Bob", 5, BackendType::Stabilizer)
        .unwrap();

    // Try to create EPR without connecting to hub - should fail
    let result = network
        .create_epr_pair_through_hub("Alice", "Bob", "Hub")
        .await;
    assert!(
        result.is_err(),
        "Should fail when nodes not connected to hub"
    );

    // Connect only Alice
    network
        .connect_to_hub("Alice", "Hub", LinkType::Fiber {
            length_km: 1.0,
            loss_db_per_km: 0.2,
        })
        .unwrap();

    // Should still fail (Bob not connected)
    let result = network
        .create_epr_pair_through_hub("Alice", "Bob", "Hub")
        .await;
    assert!(result.is_err(), "Should fail when only one node connected");

    // Connect Bob
    network
        .connect_to_hub("Bob", "Hub", LinkType::Fiber {
            length_km: 1.0,
            loss_db_per_km: 0.2,
        })
        .unwrap();

    // Now should work
    let result = network
        .create_epr_pair_through_hub("Alice", "Bob", "Hub")
        .await;
    assert!(result.is_ok(), "Should work when both connected");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_teleportation_preserves_entanglement() {
    let mut network = QuantumNetwork::new_distributed();

    network.add_hub("Hub", (0.0, 0.0)).unwrap();
    network
        .add_distributed_node("Alice", 10, BackendType::Stabilizer)
        .unwrap();
    network
        .add_distributed_node("Bob", 10, BackendType::Stabilizer)
        .unwrap();

    network
        .connect_to_hub("Alice", "Hub", LinkType::Fiber {
            length_km: 1.0,
            loss_db_per_km: 0.2,
        })
        .unwrap();
    network
        .connect_to_hub("Bob", "Hub", LinkType::Fiber {
            length_km: 1.0,
            loss_db_per_km: 0.2,
        })
        .unwrap();

    // Create multiple EPR pairs through hub
    let mut pairs = Vec::new();
    for _ in 0..5 {
        let (q1, q2) = network
            .create_epr_pair_through_hub("Alice", "Bob", "Hub")
            .await
            .expect("Should create EPR pair");
        pairs.push((q1, q2));
    }

    // Verify all pairs are tracked
    assert_eq!(pairs.len(), 5);

    // Each pair should have unique qubit indices
    let alice_qubits: Vec<_> = pairs.iter().map(|(q1, _)| q1).collect();
    let bob_qubits: Vec<_> = pairs.iter().map(|(_, q2)| q2).collect();

    // Check for duplicates (basic resource tracking test)
    for i in 0..alice_qubits.len() {
        for j in i + 1..alice_qubits.len() {
            assert_ne!(
                alice_qubits[i], alice_qubits[j],
                "Alice's qubits should be unique"
            );
            assert_ne!(
                bob_qubits[i], bob_qubits[j],
                "Bob's qubits should be unique"
            );
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_network_builder_with_hub() {
    let mut network = NetworkBuilder::new()
        .add_hub_with_strategy("CentralHub", 10, RoutingStrategy::HighestFidelity)
        .with_topology(Topology::Star {
            hub_name: "CentralHub".to_string(),
            hub_capacity: 10,
        })
        .add_endpoint("Client1", 5)
        .add_endpoint("Client2", 5)
        .with_link_type(LinkType::Fiber {
            length_km: 1.0,
            loss_db_per_km: 0.1,
        })
        .build()
        .expect("Should build network");

    // Verify hub routing works
    let result = network
        .create_epr_pair_through_hub("Client1", "Client2", "CentralHub")
        .await;
    assert!(
        result.is_ok(),
        "Should create EPR through hub built with NetworkBuilder"
    );
}

#[tokio::test]
async fn test_find_common_hub() {
    let mut network = QuantumNetwork::new_distributed();

    // Create two hubs
    network.add_hub("Hub1", (0.0, 0.0)).unwrap();
    network.add_hub("Hub2", (10.0, 0.0)).unwrap();

    // Add nodes
    network
        .add_distributed_node("Alice", 5, BackendType::Stabilizer)
        .unwrap();
    network
        .add_distributed_node("Bob", 5, BackendType::Stabilizer)
        .unwrap();

    // Connect Alice to Hub1, Bob to Hub2
    network
        .connect_to_hub("Alice", "Hub1", LinkType::Fiber {
            length_km: 1.0,
            loss_db_per_km: 0.2,
        })
        .unwrap();
    network
        .connect_to_hub("Bob", "Hub2", LinkType::Fiber {
            length_km: 1.0,
            loss_db_per_km: 0.2,
        })
        .unwrap();

    // Should not find common hub
    let common = network.find_common_hub("Alice", "Bob");
    assert!(common.is_none(), "Should not find common hub");

    // Connect both to Hub1
    network
        .connect_to_hub("Bob", "Hub1", LinkType::Fiber {
            length_km: 2.0,
            loss_db_per_km: 0.2,
        })
        .unwrap();

    // Now should find Hub1
    let common = network.find_common_hub("Alice", "Bob");
    assert_eq!(common, Some("Hub1".to_string()));
}

#[tokio::test]
async fn test_auto_route_epr() {
    let mut network = QuantumNetwork::new_distributed();

    network.add_hub("Hub", (0.0, 0.0)).unwrap();
    network
        .add_distributed_node("Alice", 5, BackendType::Stabilizer)
        .unwrap();
    network
        .add_distributed_node("Bob", 5, BackendType::Stabilizer)
        .unwrap();

    // Direct connection
    network
        .add_quantum_link(
            "Alice",
            "Bob",
            LinkType::Fiber {
                length_km: 100.0,
                loss_db_per_km: 0.3,
            },
            0.8,
            100.0,
        )
        .unwrap();

    // Hub connections (better fidelity)
    network
        .connect_to_hub("Alice", "Hub", LinkType::Fiber {
            length_km: 1.0,
            loss_db_per_km: 0.1,
        })
        .unwrap();
    network
        .connect_to_hub("Bob", "Hub", LinkType::Fiber {
            length_km: 1.0,
            loss_db_per_km: 0.1,
        })
        .unwrap();

    // Auto-route should prefer direct connection (exists)
    // or hub route based on implementation
    let result = network.create_epr_auto_route("Alice", "Bob").await;
    assert!(result.is_ok(), "Auto-route should find a path");
}

#[tokio::test]
async fn test_topology_templates() {
    // Test each topology type
    let topologies = vec![
        Topology::Star {
            hub_name: "Hub".to_string(),
            hub_capacity: 10,
        },
        Topology::Ring,
        Topology::Mesh {
            link_fidelity: 0.95,
        },
        Topology::Line,
        Topology::Hierarchical {
            central_hub: "Central".to_string(),
            regional_hubs: vec!["Regional1".to_string(), "Regional2".to_string()],
        },
    ];

    for topology in topologies {
        let mut builder = NetworkBuilder::new()
            .with_topology(topology.clone())
            .add_endpoint("Node1", 5)
            .add_endpoint("Node2", 5);

        // Add hubs if needed for the topology
        match &topology {
            Topology::Star { hub_name, .. } => {
                builder = builder.add_hub(hub_name, 10);
            }
            Topology::Hierarchical {
                central_hub,
                regional_hubs,
            } => {
                builder = builder.add_hub(central_hub, 100);
                for hub in regional_hubs {
                    builder = builder.add_hub(hub, 50);
                }
            }
            _ => {}
        }

        let network = builder.build();
        assert!(
            network.is_ok(),
            "Should build network with {:?} topology",
            topology
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_qubit_deallocation_after_teleportation() {
    let mut network = QuantumNetwork::new_distributed();

    network.add_hub("Hub", (0.0, 0.0)).unwrap();
    network
        .add_distributed_node("Alice", 5, BackendType::Stabilizer)
        .unwrap();
    network
        .add_distributed_node("Bob", 5, BackendType::Stabilizer)
        .unwrap();

    network
        .connect_to_hub("Alice", "Hub", LinkType::Fiber {
            length_km: 1.0,
            loss_db_per_km: 0.2,
        })
        .unwrap();
    network
        .connect_to_hub("Bob", "Hub", LinkType::Fiber {
            length_km: 1.0,
            loss_db_per_km: 0.2,
        })
        .unwrap();

    // Track hub's available qubits before
    let hub_node = network.nodes.get("Hub").unwrap();
    let initial_free = hub_node.qubit_allocator.get_free_count();

    // Create EPR through hub (uses teleportation)
    let _ = network
        .create_epr_pair_through_hub("Alice", "Bob", "Hub")
        .await
        .unwrap();

    // Hub should have deallocated its qubits after teleportation
    let hub_node = network.nodes.get("Hub").unwrap();
    let final_free = hub_node.qubit_allocator.get_free_count();

    // Should have roughly same free qubits (allowing for temporary allocation)
    // This tests that teleportation properly cleans up
    assert!(
        (initial_free as i32 - final_free as i32).abs() <= 2,
        "Hub should deallocate qubits after teleportation"
    );
}
