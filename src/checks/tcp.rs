use super::Service;
use anyhow::Result;
use async_std::{
	future,
	net::{SocketAddrV4, TcpStream},
};
use async_trait::async_trait;
use std::time::Duration;

#[derive(Debug)]
pub struct TcpCheck {
	pub sock: SocketAddrV4,
}

#[async_trait]
impl Service for TcpCheck {
	async fn is_up(&self, timeout: Duration) -> Result<bool> {
		Ok(future::timeout(timeout, TcpStream::connect(&self.sock))
			.await?
			.is_ok())
	}
}
