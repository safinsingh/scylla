use crate::config::DnsRecord;

use super::Service;
use anyhow::{bail, Result};
use async_trait::async_trait;
use std::net::Ipv4Addr;
use trust_dns_proto::{
	rr::{RData, RecordType},
	xfer::DnsRequestOptions,
};
use trust_dns_resolver::AsyncResolver;

#[derive(Debug)]
pub struct DnsCheck {
	name: Ipv4Addr,
	record: DnsRecord,
}

#[async_trait]
impl Service for DnsCheck {
	async fn is_up(&self) -> Result<()> {
		let resolver = AsyncResolver::tokio_from_system_conf()?;
		let lookup = resolver
			.lookup(
				self.name,
				match self.record {
					DnsRecord::A { .. } => RecordType::A,
					DnsRecord::AAAA { .. } => RecordType::AAAA,
				},
				DnsRequestOptions::default(),
			)
			.await?;

		if lookup
			.record_iter()
			.find(|r| match r.into_data() {
				RData::A(inner) => {
					if let DnsRecord::A { addr } = self.record {
						addr == inner
					} else {
						false
					}
				}
				RData::AAAA(inner) => {
					if let DnsRecord::AAAA { addr } = self.record {
						addr == inner
					} else {
						false
					}
				}
				_ => false,
			})
			.is_some()
		{
			Ok(())
		} else {
			bail!("Failed to verify that record exists")
		}
	}
}
