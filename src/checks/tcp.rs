use super::Service;
use anyhow::Result;
use async_trait::async_trait;
use std::net::SocketAddrV4;
use tokio::net::TcpStream;

#[derive(Debug)]
pub struct TcpCheck {
	pub remote: SocketAddrV4,
}

#[async_trait]
impl Service for TcpCheck {
	async fn is_up(&self) -> Result<()> {
		Ok(TcpStream::connect(&self.remote).await.map(|_| ())?)
	}
}
