pub mod templates;

use self::templates::{info_to_scores, Leaderboard, PatchServer, TplMode};
use crate::{
	config::Cfg,
	db::{
		query::{get_all_services, get_leaderboard, get_team_info},
		PgPool,
	},
};
use sqlx::Postgres;
use std::{fs, sync::Arc};
use tide::Request;
use tide_sqlx::{SQLxMiddleware, SQLxRequestExt};

pub async fn endpoint(req: Request<Arc<Cfg>>, mode: TplMode) -> tide::Result {
	let state = req.state();
	let mut conn = req.sqlx_conn::<Postgres>().await;
	let teams = get_team_info(&mut *conn).await;
	let services = get_all_services(&mut *conn).await;

	Ok(info_to_scores(&*state, teams, services, mode).into())
}

async fn scores(req: Request<Arc<Cfg>>) -> tide::Result {
	endpoint(req, TplMode::Scores).await
}

async fn uptime(req: Request<Arc<Cfg>>) -> tide::Result {
	endpoint(req, TplMode::Uptime).await
}

async fn slas(req: Request<Arc<Cfg>>) -> tide::Result {
	endpoint(req, TplMode::SLAs).await
}

async fn patch_server(req: Request<Arc<Cfg>>) -> tide::Result {
	let state = req.state();
	let files = fs::read_dir(&*state.patch_server).unwrap();
	Ok(PatchServer {
		round: &*state.round,
		files: files
			.into_iter()
			.map(|f| f.unwrap().file_name().into_string().unwrap())
			.collect::<Vec<_>>(),
	}
	.into())
}

async fn leaderboard(req: Request<Arc<Cfg>>) -> tide::Result {
	let state = req.state();
	let mut conn = req.sqlx_conn::<Postgres>().await;
	let teams = get_leaderboard(&mut *conn).await;

	Ok(Leaderboard {
		round: &*state.round,
		teams,
	}
	.into())
}

pub async fn start(pool: PgPool, cfg: Arc<Cfg>) {
	let mut app = tide::with_state(cfg.clone());

	app.with(SQLxMiddleware::from(pool));
	app.at("/").get(scores);
	app.at("/scores").get(scores);
	app.at("/uptime").get(uptime);
	app.at("/slas").get(slas);
	app.at("/leaderboard").get(leaderboard);
	app.at("/patch-server").get(patch_server);
	app.at("/patch-files")
		.serve_dir(&*cfg.patch_server)
		.unwrap();

	app.listen("0.0.0.0:5112").await.unwrap();
}
