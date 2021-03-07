use crate::config::{service_from_config, Cfg, SharedService};
use async_std::{sync::Mutex, task};
use chrono::{Duration, Utc};
use futures::future;
use std::sync::Arc;

pub async fn enter_waiter(
	cfg: Arc<Cfg>,
	services: Arc<Mutex<Vec<SharedService>>>,
) {
	future::join_all(cfg.injects.iter().map(|inject| {
		let cfg = cfg.clone();
		let services = services.clone();

		async move {
			let until = (cfg.start
				+ Duration::seconds(
					((inject.offset + inject.duration) * 60) as i64,
				)) - Utc::now();

			println!(
				"Preparing to add inject \"{}\" in {} minutes!",
				inject.meta.title,
				until.num_minutes()
			);

			task::sleep(until.to_std().unwrap()).await;
			let mut services = services.lock().await;

			for (bx_name, bx_svcs) in inject.new_services.iter() {
				for svc in bx_svcs {
					for team in cfg.teams.iter() {
						services.push(service_from_config(
							svc,
							team,
							(&bx_name, &cfg.boxes[bx_name]),
						));

						println!(
							"Added inject {} on box {}!",
							inject.meta.title, bx_name
						);
					}
				}
			}
		}
	}))
	.await;
}
