use qnect::builder::BackendType;
use qnect::create;
use qnect::error::QnectError;

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
    // Test that we can create with different backends
    let mut sv = create()
        .with_backend(BackendType::StateVector)
        .with_qubits(5)
        .build()
        .unwrap();

    sv.h(0).await.unwrap();
    assert_eq!(sv.qubit_count(), 5);

    // Future: test other backends when implemented
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
