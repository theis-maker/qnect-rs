use crate::builder::BackendType;
use crate::network::chat_protocol::{NetworkMessage, NodeType};
use crate::network::network::{LinkType, QuantumNetwork};
use crate::network::quantum_client::QuantumClient;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc};

pub const QUANTUM_SERVICE: &str = "127.0.0.1:6666";

pub struct QuantumChatNode {
    name: String,
    node_type: NodeType,
    location: (f64, f64),
    network: Arc<Mutex<QuantumNetwork>>,
    quantum_client: Arc<Mutex<QuantumClient>>,
    message_rx: mpsc::Receiver<(String, NetworkMessage)>,
    message_tx: mpsc::Sender<(String, NetworkMessage)>,
    peers: Arc<Mutex<HashMap<String, mpsc::Sender<NetworkMessage>>>>,
}

impl QuantumChatNode {
    pub async fn new(
        name: &str,
        node_type: NodeType,
        location: (f64, f64),
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut network = QuantumNetwork::new_distributed();
        network.add_distributed_node("Alice", 50, BackendType::Stabilizer)?;
        network.add_distributed_node("Repeater", 16, BackendType::Stabilizer)?;
        network.add_distributed_node("Bob", 50, BackendType::Stabilizer)?;

        network.add_quantum_link(
            "Alice",
            "Repeater",
            LinkType::Fiber {
                length_km: 500.0,
                loss_db_per_km: 0.2,
            },
            0.85,
            100.0,
        )?;

        network.add_quantum_link(
            "Repeater",
            "Bob",
            LinkType::Fiber {
                length_km: 500.0,
                loss_db_per_km: 0.2,
            },
            0.85,
            100.0,
        )?;

        log::debug!("[{}] Connecting to quantum service...", name);
        let quantum_client = QuantumClient::connect().await?;
        log::debug!("[{}] ✓ Connected to quantum backend!", name);

        let (tx, rx) = mpsc::channel(100);

        Ok(QuantumChatNode {
            name: name.to_string(),
            node_type,
            location,
            network: Arc::new(Mutex::new(network)),
            quantum_client: Arc::new(Mutex::new(quantum_client)),
            message_rx: rx,
            message_tx: tx,
            peers: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    // EXACT listen method
    pub async fn listen(&self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        log::debug!("[{}] Listening on port {}", self.name, port);

        let name = self.name.clone();
        let peers = self.peers.clone();
        let tx = self.message_tx.clone();

        tokio::spawn(async move {
            loop {
                if let Ok((stream, addr)) = listener.accept().await {
                    log::debug!("[{}] Connection from {}", name, addr);
                    let name_clone = name.clone();
                    let peers_clone = peers.clone();
                    let tx_clone = tx.clone();

                    tokio::spawn(async move {
                        handle_connection(stream, name_clone, peers_clone, tx_clone).await;
                    });
                }
            }
        });

        Ok(())
    }

    pub async fn connect_to(
        &self,
        peer_name: &str,
        addr: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let stream = TcpStream::connect(addr).await?;
        log::debug!("[{}] Connected to {} at {}", self.name, peer_name, addr);

        let (reader, mut writer) = stream.into_split();

        // Send our info
        let msg = NetworkMessage::NodeJoin {
            name: self.name.clone(),
            node_type: self.node_type.clone(),
            location: self.location,
        };
        let json = serde_json::to_string(&msg)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;

        // Create channel for this peer
        let (peer_tx, mut peer_rx) = mpsc::channel(100);
        {
            let mut peers = self.peers.lock().await;
            peers.insert(peer_name.to_string(), peer_tx);
        }

        // Spawn writer
        tokio::spawn(async move {
            while let Some(msg) = peer_rx.recv().await {
                let json = serde_json::to_string(&msg).unwrap();
                let _ = writer.write_all(format!("{}\n", json).as_bytes()).await;
            }
        });

        // Spawn reader
        let tx = self.message_tx.clone();
        let peer_name_clone = peer_name.to_string();
        tokio::spawn(async move {
            let reader = AsyncBufReader::new(reader);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if let Ok(msg) = serde_json::from_str::<NetworkMessage>(&line) {
                    let _ = tx.send((peer_name_clone.clone(), msg)).await;
                }
            }
        });

        Ok(())
    }

    pub async fn run_bb84_alice(
        &mut self,
        bob: &str,
        rounds: usize,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        log::debug!(
            "\n[{}] Starting BB84 with {} ({} rounds)",
            self.name,
            bob,
            rounds
        );

        // Verify path exists
        let network = self.network.lock().await;
        let path = network
            .find_shortest_path(&self.name, bob)
            .ok_or("No path to Bob")?;
        log::debug!("[{}] Path found: {:?}", self.name, path);
        drop(network);

        // Send start signal
        self.send_to(bob, NetworkMessage::BB84Start { rounds })
            .await?;

        let mut alice_bits = Vec::new();
        let mut alice_bases = Vec::new();
        let mut bob_results = Vec::new();
        let mut shared_key = Vec::new();

        log::debug!("[{}] Preparing and sending quantum states...", self.name);

        for round in 0..rounds {
            // Alice's REAL quantum state preparation
            let bit = rand::random::<bool>();
            let alice_basis = rand::random::<bool>();
            alice_bits.push(bit);
            alice_bases.push(alice_basis);

            // REAL qubit allocation
            let mut quantum = self.quantum_client.lock().await;
            let alice_q = quantum.allocate_qubit(&self.name).await?;

            // REAL gate application
            if bit {
                quantum.apply_gate(&self.name, alice_q, "X").await?;
            }
            if alice_basis {
                quantum.apply_gate(&self.name, alice_q, "H").await?;
            }

            // REAL quantum teleportation to Bob
            let bob_q = quantum.teleport(&self.name, bob, alice_q).await?;
            drop(quantum);

            // Tell Bob which qubit to measure
            self.send_to(bob, NetworkMessage::BB84Measure {
                round,
                qubit_id: bob_q,
            })
            .await?;

            // Wait for Bob's measurement
            match tokio::time::timeout(Duration::from_secs(5), self.message_rx.recv()).await {
                Ok(Some((
                    from,
                    NetworkMessage::BB84Basis {
                        round: r,
                        basis: bob_basis,
                        result,
                    },
                ))) if from == bob && r == round => {
                    bob_results.push((bob_basis, result));

                    if alice_basis == bob_basis {
                        shared_key.push(result);
                        print!("■");
                    } else {
                        print!("□");
                    }
                }
                _ => {
                    log::debug!("\n[{}] Timeout waiting for Bob's measurement", self.name);
                }
            }

            if (round + 1) % 32 == 0 {
                log::debug!(" {}/{}", shared_key.len(), round + 1);
            }
        }

        log::debug!("\n[{}] Sending key reconciliation...", self.name);
        self.send_to(bob, NetworkMessage::BB84KeyBits {
            count: shared_key.len(),
        })
        .await?;

        for round in 0..rounds {
            if round < alice_bases.len()
                && round < bob_results.len()
                && alice_bases[round] == bob_results[round].0
            {
                self.send_to(bob, NetworkMessage::BB84Use { round }).await?;
            }
        }
        self.send_to(bob, NetworkMessage::BB84EndKey).await?;

        log::debug!(
            "[{}] ✅ Key established: {} bits",
            self.name,
            shared_key.len()
        );

        Ok(shared_key)
    }

    async fn send_to(
        &self,
        peer: &str,
        msg: NetworkMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let peers = self.peers.lock().await;
        if let Some(tx) = peers.get(peer) {
            tx.send(msg).await?;
        }
        Ok(())
    }

    // BOB's BB84 - using REAL quantum measurements!
    pub async fn run_bb84_bob(
        &mut self,
        alice: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        log::debug!("\n[{}] Waiting for BB84 from {}", self.name, alice);

        // Wait for start
        loop {
            match self.message_rx.recv().await {
                Some((from, NetworkMessage::BB84Start { rounds })) if from == alice => {
                    log::debug!("[{}] BB84 started: {} rounds", self.name, rounds);
                    break;
                }
                _ => {}
            }
        }

        let mut all_measurements = Vec::new();

        log::debug!("[{}] Receiving and measuring quantum states...", self.name);

        // Phase 1: Measure all qubits (like example 29)
        loop {
            match tokio::time::timeout(Duration::from_secs(10), self.message_rx.recv()).await {
                Ok(Some((from, NetworkMessage::BB84Measure { round, qubit_id })))
                    if from == alice =>
                {
                    // Bob's REAL quantum measurement
                    let bob_basis = rand::random::<bool>();

                    let mut quantum = self.quantum_client.lock().await;

                    // Apply basis if H
                    if bob_basis {
                        quantum.apply_gate(&self.name, qubit_id, "H").await?;
                    }

                    // REAL measurement
                    let result = quantum.measure(&self.name, qubit_id).await?;
                    drop(quantum);

                    all_measurements.push(result);

                    // Send basis and result back to Alice
                    self.send_to(alice, NetworkMessage::BB84Basis {
                        round,
                        basis: bob_basis,
                        result,
                    })
                    .await?;
                }
                Ok(Some((from, NetworkMessage::BB84KeyBits { .. }))) if from == alice => {
                    // Key reconciliation phase started
                    self.message_tx
                        .send((from, NetworkMessage::BB84KeyBits { count: 0 }))
                        .await?;
                    break;
                }
                _ => {}
            }
        }

        // Phase 2: Alice tells us which bits to keep
        let mut shared_key = Vec::new();

        loop {
            match self.message_rx.recv().await {
                Some((from, NetworkMessage::BB84Use { round })) if from == alice => {
                    if round < all_measurements.len() {
                        shared_key.push(all_measurements[round]);
                    }
                }
                Some((from, NetworkMessage::BB84EndKey)) if from == alice => {
                    break;
                }
                _ => {}
            }
        }

        log::debug!(
            "[{}] ✅ Received {} key bits from Alice",
            self.name,
            shared_key.len()
        );

        Ok(shared_key)
    }

    // Chat function using the quantum-derived key
    pub async fn run_chat(
        &mut self,
        peer: &str,
        key: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("\n💬 Quantum-Secured Chat Active!");
        log::debug!("Type 'quit' to exit\n");

        let key_clone = key.clone();
        let peer_name = peer.to_string();
        let my_name = self.name.clone();

        // Spawn message receiver
        let (decrypt_tx, decrypt_rx) = mpsc::channel::<String>(10);
        // let msg_tx = self.message_tx.clone();

        tokio::spawn(async move {
            let mut msg_rx = decrypt_rx;
            while let Some(encrypted_hex) = msg_rx.recv().await {
                if let Ok(encrypted_bytes) = hex::decode(&encrypted_hex) {
                    let decrypted: Vec<u8> = encrypted_bytes
                        .iter()
                        .zip(key_clone.iter().cycle())
                        .map(|(e, k)| e ^ k)
                        .collect();

                    let msg = String::from_utf8_lossy(&decrypted);
                    log::debug!("\n{}: {}", peer_name, msg);
                    print!("{} > ", my_name);
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                }
            }
        });

        // Handle stdin
        let (input_tx, mut input_rx) = mpsc::channel::<String>(10);

        std::thread::spawn(move || {
            let stdin = std::io::stdin();
            let reader = BufReader::new(stdin);

            for line in reader.lines() {
                if let Ok(line) = line {
                    let _ = input_tx.blocking_send(line);
                }
            }
        });

        print!("{} > ", self.name);
        std::io::Write::flush(&mut std::io::stdout())?;

        loop {
            tokio::select! {
                // Handle incoming messages
                Some((from, msg)) = self.message_rx.recv() => {
                    if from == peer {
                        if let NetworkMessage::EncryptedMessage { data } = msg {
                            decrypt_tx.send(data).await?;
                        }
                    }
                }

                // Handle user input
                Some(line) = input_rx.recv() => {
                    let message = line.trim();

                    if message == "quit" {
                        break;
                    }

                    if !message.is_empty() {
                        // Encrypt with quantum key
                        let encrypted: Vec<u8> = message.bytes()
                            .zip(key.iter().cycle())
                            .map(|(m, k)| m ^ k)
                            .collect();

                        self.send_to(peer, NetworkMessage::EncryptedMessage {
                            data: hex::encode(&encrypted)
                        }).await?;
                    }

                    print!("{} > ", self.name);
                    std::io::Write::flush(&mut std::io::stdout())?;
                }
            }
        }

        Ok(())
    }
}

async fn handle_connection(
    stream: TcpStream,
    our_name: String,
    peers: Arc<Mutex<HashMap<String, mpsc::Sender<NetworkMessage>>>>,
    message_tx: mpsc::Sender<(String, NetworkMessage)>,
) {
    let (reader, mut writer) = stream.into_split();
    let mut lines = AsyncBufReader::new(reader).lines();

    // Read peer info
    if let Ok(Some(line)) = lines.next_line().await {
        if let Ok(NetworkMessage::NodeJoin { name, .. }) =
            serde_json::from_str::<NetworkMessage>(&line)
        {
            log::debug!("[{}] Peer identified as {}", our_name, name);

            // Create channel for this peer
            let (peer_tx, mut peer_rx) = mpsc::channel(100);
            {
                let mut p = peers.lock().await;
                p.insert(name.clone(), peer_tx);
            }

            // Spawn writer
            tokio::spawn(async move {
                while let Some(msg) = peer_rx.recv().await {
                    let json = serde_json::to_string(&msg).unwrap();
                    let _ = writer.write_all(format!("{}\n", json).as_bytes()).await;
                }
            });

            // Continue reading
            while let Ok(Some(line)) = lines.next_line().await {
                if let Ok(msg) = serde_json::from_str::<NetworkMessage>(&line) {
                    let _ = message_tx.send((name.clone(), msg)).await;
                }
            }
        }
    }
}
