use crate::{
	config::Cfg,
	db::{
		mutation::{persist_downtime, persist_uptime},
		PgPool,
	},
};
use anyhow::Result;
use async_std::{
	channel::{Receiver, Sender},
	stream, task,
};
use async_trait::async_trait;
use futures::{future, StreamExt};
use std::{fmt::Debug, sync::Arc, time::Duration};

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
	async fn is_up(&self, timeout: Duration) -> Result<bool>;
	async fn poll(
		&self,
		chan: Sender<ChanMsg>,
		timeout: Duration,
		cx: Arc<SvcMeta>,
	) {
		let message = match self.is_up(timeout).await {
			Ok(true) => ChanMsg::Uptime(cx),
			_ => ChanMsg::Error(cx),
		};
		chan.send(message).await.unwrap();
	}
}

pub async fn enter_event_loop(cfg: Arc<Cfg>, tx: Sender<ChanMsg>) {
	let jittered = cfg.checks.get_interval();
	let mut interval = stream::interval(jittered);

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
						Duration::from_secs(5),
						svc.meta.clone(),
					)
				}))
				.await;
			}
		});

		interval.next().await;
	}
}

pub async fn enter_recv_loop(rx: Receiver<ChanMsg>, pool: PgPool) {
	loop {
		let m = rx.recv().await.unwrap();
		match m {
			ChanMsg::Uptime(meta) => persist_uptime(&meta, pool.clone()).await,
			ChanMsg::Error(meta) => persist_downtime(&meta, pool.clone()).await,
		};
	}
}

pub mod injects;
pub mod tcp;
pub mod udp;
