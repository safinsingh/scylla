use super::Service;
use anyhow::Result;
use async_trait::async_trait;
use std::net::SocketAddrV4;
use tokio::net::UdpSocket;

#[derive(Debug)]
pub struct UdpCheck {
	pub remote: SocketAddrV4,
	pub socket_addr: SocketAddrV4,
}

#[async_trait]
impl Service for UdpCheck {
	async fn is_up(&self) -> Result<()> {
		let sock = UdpSocket::bind(&self.socket_addr).await?;
		Ok(sock.connect(&self.remote).await?)
	}
}
