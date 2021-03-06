use super::PgPool;
use crate::{checks::SvcMeta, config::Cfg};
use anyhow::{Context, Result};
use std::sync::Arc;

pub async fn setup(cfg: Arc<Cfg>, pool: PgPool) -> Result<()> {
	for (team_id, team_meta) in cfg.teams.iter() {
		sqlx::query!(
			"INSERT INTO teams(team_id, pass) VALUES($1, $2);",
			team_id,
			team_meta.password
		)
		.execute(&pool)
		.await?;

		for (vm_id, vm_meta) in cfg.boxes.iter() {
			sqlx::query!(
				"INSERT INTO vms(vm_id, team_id) VALUES($1, $2);",
				vm_id,
				team_id
			)
			.execute(&pool)
			.await?;

			for svc in vm_meta.services.iter() {
				sqlx::query!(
					r#"
					INSERT INTO services(svc_id, vm_id, team_id)
						VALUES($1, $2, $3);
					"#,
					svc.id,
					vm_id,
					team_id
				)
				.execute(&pool)
				.await?;
			}
		}
	}

	println!("Preparation complete!");
	Ok(())
}

pub async fn persist_uptime(meta: &SvcMeta, pool: PgPool) -> Result<()> {
	sqlx::query!(
		r#"
		UPDATE	services
			SET	check_count = check_count + 1,
					uptime_score = uptime_score + 1,
					recurring_down = 0,
					latest_uptime_status = TRUE
		 WHERE	svc_id = $1 AND
					vm_id = $2 AND
					team_id = $3;
		"#,
		&*meta.svc_id,
		&*meta.vm_id,
		&*meta.team_id
	)
	.execute(&pool)
	.await
	.with_context(|| {
		format!(
			"Failed to persist uptime for box: {} - {} - {}!",
			&*meta.svc_id, &*meta.vm_id, &*meta.team_id,
		)
	})
	.map(|_| ())
}

pub async fn persist_downtime(meta: &SvcMeta, pool: PgPool) -> Result<()> {
	#[derive(sqlx::FromRow, Debug)]
	struct DowntimeReturn {
		recurring_down: i32,
	}

	let service = sqlx::query_as!(
		DowntimeReturn,
		r#"
		UPDATE	services
			SET	check_count = check_count + 1,
					recurring_down = recurring_down + 1,
					latest_uptime_status = FALSE
		 WHERE	svc_id = $1 AND
					vm_id = $2 AND
					team_id = $3
	RETURNING	services.recurring_down;
		"#,
		&*meta.svc_id,
		&*meta.vm_id,
		&*meta.team_id
	)
	.fetch_one(&pool)
	.await
	.with_context(|| {
		format!(
			"Failed to persist downtime for box: {} - {} - {}!",
			&*meta.svc_id, &*meta.vm_id, &*meta.team_id,
		)
	})?;

	if service.recurring_down >= 5 {
		persist_sla(meta, pool).await?;
	}

	Ok(())
}

pub async fn persist_sla(meta: &SvcMeta, pool: PgPool) -> Result<()> {
	sqlx::query!(
		r#"
		UPDATE	services
			SET	sla_count = sla_count + 1
		 WHERE	svc_id = $1 AND
					vm_id = $2 AND
					team_id = $3;
		"#,
		&*meta.svc_id,
		&*meta.vm_id,
		&*meta.team_id
	)
	.execute(&pool)
	.await
	.with_context(|| {
		format!(
			"Failed to persist SLA for box: {} - {} - {}!",
			&*meta.svc_id, &*meta.vm_id, &*meta.team_id,
		)
	})
	.map(|_| ())
}
