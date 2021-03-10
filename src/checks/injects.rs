use crate::config::{Cfg, SharedService};
use anyhow::{Error, Result};
use async_std::task;
use chrono::{Duration, Utc};
use futures::future;
use std::sync::Arc;

pub async fn wait(cfg: Arc<Cfg>) -> Result<()> {
	future::try_join_all(cfg.injects.iter().map(|inject| {
		let new_cfg = cfg.clone();

		async move {
			let until = (new_cfg.start
				+ Duration::seconds(
					((inject.offset + inject.duration) * 60) as i64,
				)) - Utc::now();

			println!(
				"Preparing to add inject \"{}\" in {} minutes!",
				inject.meta.title,
				until.num_minutes()
			);

			task::sleep(until.to_std()?).await;
			let mut services = new_cfg._services.lock().await;

			for (bx_id, bx_svcs) in inject.new_services.iter() {
				for svc in bx_svcs {
					for team in new_cfg.teams.iter() {
						services.push(SharedService::from_config(
							svc,
							team,
							(&bx_id, &new_cfg.boxes[bx_id]),
						)?);

						println!(
							"Added inject {} on box {}!",
							inject.meta.title, bx_id
						);
					}
				}
			}

			Ok::<_, Error>(())
		}
	}))
	.await
	.map(|_| ())
}
