use super::{CheckMeta, Service};
use anyhow::Result;
use async_std::{future, net::UdpSocket};
use async_trait::async_trait;
use std::{sync::Arc, time::Duration};

#[derive(Debug)]
pub struct UdpCheck {
	pub host: Arc<String>,
	pub meta: Arc<CheckMeta>,
}

impl UdpCheck {
	pub fn new(host: Arc<String>, meta: CheckMeta) -> UdpCheck {
		Self {
			host,
			meta: Arc::new(meta),
		}
	}
}

#[async_trait]
impl Service for UdpCheck {
	fn get_meta(&self) -> Arc<CheckMeta> { self.meta.clone() }
	async fn is_up(&self, timeout: Duration) -> Result<bool> {
		Ok(future::timeout(timeout, UdpSocket::bind(&*self.host))
			.await?
			.is_ok())
	}
}
