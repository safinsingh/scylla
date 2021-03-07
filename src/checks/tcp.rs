use super::{CheckMeta, Service};
use anyhow::Result;
use async_std::{future, net::TcpStream};
use async_trait::async_trait;
use std::{sync::Arc, time::Duration};

#[derive(Debug)]
pub struct TcpCheck {
	host: Arc<String>,
	pub meta: Arc<CheckMeta>,
}

impl TcpCheck {
	pub fn new(host: Arc<String>, meta: CheckMeta) -> TcpCheck {
		Self {
			host,
			meta: Arc::new(meta),
		}
	}
}

#[async_trait]
impl Service for TcpCheck {
	fn get_meta(&self) -> Arc<CheckMeta> { self.meta.clone() }
	async fn is_up(&self, timeout: Duration) -> Result<bool> {
		Ok(future::timeout(timeout, TcpStream::connect(&*self.host))
			.await?
			.is_ok())
	}
}
