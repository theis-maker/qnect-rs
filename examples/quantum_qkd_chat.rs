use std::sync::Arc;
use std::time::Duration;

use qnect::network::chat_protocol::NodeType;
use qnect::network::quantum_chat_node::QuantumChatNode;
use qnect::network::quantum_service::QuantumService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("service") => {
            // One line to start the EXACT service!
            let service = Arc::new(QuantumService::new().await?);
            service.run(6666).await?;
        }

        Some("alice") => {
            let mut alice = QuantumChatNode::new(
                "Alice",
                NodeType::Endpoint,
                (51.5074, -0.1278), // London
            )
            .await?;

            alice.listen(7000).await?;
            tokio::time::sleep(Duration::from_secs(1)).await;
            alice.connect_to("Repeater", "127.0.0.1:7001").await?;
            tokio::time::sleep(Duration::from_secs(2)).await;

            println!("⏳ Waiting for Bob to join network...");
            tokio::time::sleep(Duration::from_secs(3)).await;

            match alice.run_bb84_alice("Bob", 128).await {
                Ok(key) => {
                    println!("\n🔐 Quantum key established using REAL quantum states!");
                    alice.run_chat("Bob", key).await?;
                }
                Err(e) => eprintln!("❌ QKD failed: {}", e),
            }
        }

        Some("repeater") => {
            let repeater = QuantumChatNode::new(
                "Repeater",
                NodeType::Repeater,
                (48.8566, 2.3522), // Paris
            )
            .await?;

            repeater.listen(7001).await?;

            println!("\n🔄 Quantum Repeater Node Active");
            println!("📍 Location: Paris (48.8°N, 2.3°E)");
            println!("💾 Memory slots: 64 qubits");
            println!("🌉 Bridging Alice (London) <-> Bob (Berlin)");
            println!("⏳ Waiting for connections...\n");

            tokio::signal::ctrl_c().await?;
        }

        Some("bob") => {
            let mut bob = QuantumChatNode::new(
                "Bob",
                NodeType::Endpoint,
                (52.5200, 13.4050), // Berlin
            )
            .await?;

            bob.listen(7002).await?;
            tokio::time::sleep(Duration::from_secs(1)).await;
            bob.connect_to("Repeater", "127.0.0.1:7001").await?;
            bob.connect_to("Alice", "127.0.0.1:7000").await?;

            println!("⏳ Waiting for QKD from Alice...");

            match bob.run_bb84_bob("Alice").await {
                Ok(key) => {
                    println!("\n🔐 Quantum key received using REAL quantum measurements!");
                    bob.run_chat("Alice", key).await?;
                }
                Err(e) => eprintln!("❌ QKD failed: {}", e),
            }
        }

        _ => {
            println!("\n📋 Usage:");
            println!("   Terminal 1: cargo run --example quantum_chat_clean service");
            println!("   Terminal 2: cargo run --example quantum_chat_clean repeater");
            println!("   Terminal 3: cargo run --example quantum_chat_clean alice");
            println!("   Terminal 4: cargo run --example quantum_chat_clean bob");
        }
    }

    Ok(())
}
