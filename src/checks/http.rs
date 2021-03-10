use super::Service;
use anyhow::{bail, Result};
use async_trait::async_trait;
use reqwest::{Client, Method, Request, Url};
use std::{net::SocketAddrV4, str::FromStr};

#[derive(Debug)]
pub struct HttpCheck {
	pub remote: SocketAddrV4,
	pub method: Method,
	pub ssl: bool,
	pub content_hash: Option<String>,
}

#[async_trait]
impl Service for HttpCheck {
	async fn is_up(&self) -> Result<()> {
		let url = Url::from_str(&format!(
			"http{ssl}://{}",
			self.remote.to_string(),
			ssl = if self.ssl { "s" } else { "" }
		))?;

		let req = Request::new(self.method.to_owned(), url);
		let res = Client::new().execute(req).await?;

		if let Some(ref hash) = self.content_hash {
			let digest = md5::compute(res.text().await?.as_bytes());
			if !(&format!("{:x}", digest) == hash) {
				bail!("Hash comparison failed")
			}
		}

		Ok(())
	}
}
