use crate::{
	config::{Cfg, SharedService},
	db::{
		mutation::{persist_downtime, persist_uptime},
		PgPool,
	},
};
use anyhow::Result;
use async_std::{
	channel::{Receiver, Sender},
	stream,
	sync::Mutex,
	task,
};
use async_trait::async_trait;
use futures::{future, StreamExt};
use std::{fmt::Debug, sync::Arc, time::Duration};

#[derive(Debug)]
pub enum ChanMsg {
	Error(Arc<CheckMeta>),
	Uptime(Arc<CheckMeta>),
}

#[derive(Debug)]
pub struct CheckMeta {
	pub team_name: String,
	pub vm_name: String,
	pub svc_name: String,
}

#[async_trait]
pub trait Service: Send + Sync + Debug {
	fn get_meta(&self) -> Arc<CheckMeta>;
	async fn is_up(&self, timeout: Duration) -> Result<bool>;
	async fn poll(&self, chan: Sender<ChanMsg>, timeout: Duration) {
		let message = match self.is_up(timeout).await {
			Ok(true) => ChanMsg::Uptime(self.get_meta()),
			_ => ChanMsg::Error(self.get_meta()),
		};
		chan.send(message).await.unwrap();
	}
}

pub async fn enter_event_loop(
	cfg: Arc<Cfg>,
	services: Arc<Mutex<Vec<SharedService>>>,
	tx: Sender<ChanMsg>,
) {
	let jittered = cfg.checks.get_interval();
	let mut interval = stream::interval(jittered);

	println!("Scoring with interval: {:?}", jittered);

	loop {
		task::spawn({
			let services = services.clone();
			let shared_tx = tx.clone();

			async move {
				let services = services.lock().await;

				future::join_all(services.iter().map(|svc| {
					svc.poll(shared_tx.clone(), Duration::from_secs(5))
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
			ChanMsg::Uptime(meta) => persist_uptime(&*meta, pool.clone()).await,
			ChanMsg::Error(meta) => {
				persist_downtime(&*meta, pool.clone()).await
			}
		};
	}
}

pub mod injects;
pub mod tcp;
pub mod udp;
