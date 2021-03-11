pub mod dns;
pub mod http;
pub mod injects;
pub mod tcp;
pub mod udp;

use crate::{
	config::Cfg,
	db::{
		mutation::{persist_downtime, persist_uptime},
		PgPool,
	},
};
use anyhow::{anyhow, Context as _, Result};
use async_trait::async_trait;
use futures::future;
use std::{fmt::Debug, sync::Arc, time::Duration};
use tokio::{
	sync::mpsc::{UnboundedReceiver, UnboundedSender},
	task, time,
};

#[derive(Debug, Clone)]
pub struct SvcMeta {
	pub team_id: String,
	pub vm_id: String,
	pub svc_id: String,
}

#[derive(Debug, Clone)]
pub enum ChanMsg {
	Error(Arc<SvcMeta>),
	Uptime(Arc<SvcMeta>),
}

#[async_trait]
pub trait Service: Send + Sync + Debug {
	async fn is_up(&self) -> Result<()>;
	async fn poll(
		&self,
		chan: UnboundedSender<ChanMsg>,
		timeout: Duration,
		cx: Arc<SvcMeta>,
	) -> Result<()> {
		let message = match time::timeout(timeout, self.is_up()).await {
			Ok(_) => ChanMsg::Uptime(cx.clone()),
			Err(_) => ChanMsg::Error(cx.clone()),
		};
		chan.send(message).with_context(|| {
			format!("Failed to send poll message to channel: {:?}", cx.clone())
		})
	}
}

pub async fn enter_event_loop(cfg: Arc<Cfg>, tx: UnboundedSender<ChanMsg>) {
	let jittered = cfg.checks.get_interval();
	let mut interval = time::interval(jittered);

	println!("Scoring with interval: {:?}", jittered);

	loop {
		task::spawn({
			let cfg = cfg.clone();
			let shared_tx = tx.clone();

			async move {
				let services = cfg._services.lock().await;

				future::join_all(services.iter().map(|svc| {
					svc.inner.poll(
						shared_tx.clone(),
						Duration::from_secs(cfg.checks.timeout as u64),
						svc.meta.clone(),
					)
				}))
				.await;
			}
		});

		interval.tick().await;
	}
}

pub async fn enter_recv_loop(
	mut rx: UnboundedReceiver<ChanMsg>,
	pool: PgPool,
) -> Result<()> {
	loop {
		let m = rx
			.recv()
			.await
			.ok_or(anyhow!("Failed to recieve message from channel!"))?;
		match m {
			ChanMsg::Uptime(meta) => persist_uptime(&meta, pool.clone()).await,
			ChanMsg::Error(meta) => persist_downtime(&meta, pool.clone()).await,
		}?;
	}
}
