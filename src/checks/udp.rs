use super::Service;
use anyhow::Result;
use async_std::{
	future,
	net::{SocketAddrV4, UdpSocket},
};
use async_trait::async_trait;
use std::time::Duration;

#[derive(Debug)]
pub struct UdpCheck {
	pub sock: SocketAddrV4,
}

#[async_trait]
impl Service for UdpCheck {
	async fn is_up(&self, timeout: Duration) -> Result<bool> {
		Ok(future::timeout(timeout, UdpSocket::bind(&self.sock))
			.await?
			.is_ok())
	}
}
