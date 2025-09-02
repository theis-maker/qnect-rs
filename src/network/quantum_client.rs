use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader},
    net::TcpStream,
};

use crate::network::quantum_chat_node::QUANTUM_SERVICE;

// The REAL quantum client from examples 28/29
pub struct QuantumClient {
    pub reader: AsyncBufReader<tokio::net::tcp::OwnedReadHalf>,
    pub writer: tokio::net::tcp::OwnedWriteHalf,
}

impl QuantumClient {
    pub async fn connect() -> Result<Self, Box<dyn std::error::Error>> {
        let stream = TcpStream::connect(QUANTUM_SERVICE).await?;
        let (reader, writer) = stream.into_split();
        Ok(Self {
            reader: AsyncBufReader::new(reader),
            writer,
        })
    }

    pub async fn allocate_qubit(
        &mut self,
        node: &str,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        self.writer
            .write_all(format!("ALLOCATE:{}\n", node).as_bytes())
            .await?;
        self.writer.flush().await?;

        let mut response = String::new();
        self.reader.read_line(&mut response).await?;

        if response.starts_with("OK:") {
            Ok(response.trim().strip_prefix("OK:").unwrap().parse()?)
        } else {
            Err(format!("Allocate failed: {}", response).into())
        }
    }

    pub async fn apply_gate(
        &mut self,
        node: &str,
        qubit: usize,
        gate: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.writer
            .write_all(format!("GATE:{}:{}:{}\n", node, qubit, gate).as_bytes())
            .await?;
        self.writer.flush().await?;

        let mut response = String::new();
        self.reader.read_line(&mut response).await?;

        if response.starts_with("OK") {
            Ok(())
        } else {
            Err(format!("Gate failed: {}", response).into())
        }
    }

    pub async fn measure(
        &mut self,
        node: &str,
        qubit: usize,
    ) -> Result<u8, Box<dyn std::error::Error>> {
        self.writer
            .write_all(format!("MEASURE:{}:{}\n", node, qubit).as_bytes())
            .await?;
        self.writer.flush().await?;

        let mut response = String::new();
        self.reader.read_line(&mut response).await?;

        if response.starts_with("OK:") {
            Ok(response.trim().strip_prefix("OK:").unwrap().parse()?)
        } else {
            Err(format!("Measure failed: {}", response).into())
        }
    }

    pub async fn teleport(
        &mut self,
        from: &str,
        to: &str,
        qubit: usize,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        self.writer
            .write_all(format!("TELEPORT:{}:{}:{}\n", from, to, qubit).as_bytes())
            .await?;
        self.writer.flush().await?;

        let mut response = String::new();
        self.reader.read_line(&mut response).await?;

        if response.starts_with("OK:") {
            Ok(response.trim().strip_prefix("OK:").unwrap().parse()?)
        } else {
            Err(format!("Teleport failed: {}", response).into())
        }
    }
}
