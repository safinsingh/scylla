pub mod templates;

use self::templates::{Leaderboard, PatchServer, Scores, TplMode};
use crate::{
	config::Cfg,
	db::{
		query::{get_all_services, get_leaderboard, get_team_info},
		PgPool,
	},
};
use anyhow::{Context, Result};
use askama::Template;
use rocket::{
	http::Status,
	response::{content::Html, status::Custom as RocketResult},
	Config, State,
};
use rocket_contrib::serve::StaticFiles;
use std::{
	fs,
	net::{IpAddr, Ipv4Addr},
	sync::Arc,
};

type TplResult = RocketResult<Html<String>>;
fn render_tpl<T: Template>(tpl: T) -> TplResult {
	match tpl.render() {
		Ok(tpl) => RocketResult(Status::Ok, Html(tpl)),
		Err(_) => RocketResult(
			Status::InternalServerError,
			Html("Error hidden for security purposes.".into()),
		),
	}
}

pub async fn endpoint<'r>(
	cfg: State<'r, Arc<Cfg>>,
	pool: State<'r, PgPool>,
	mode: TplMode,
) -> TplResult {
	let mut conn = pool.acquire().await.unwrap();
	let teams = get_team_info(&mut conn).await;
	let services = get_all_services(&mut conn).await;

	render_tpl(Scores::from_info(&*cfg, teams, services, mode))
}

#[get("/")]
async fn root<'r>(
	cfg: State<'r, Arc<Cfg>>,
	pool: State<'r, PgPool>,
) -> TplResult {
	endpoint(cfg, pool, TplMode::Scores).await
}

#[get("/scores")]
async fn scores<'r>(
	cfg: State<'r, Arc<Cfg>>,
	pool: State<'r, PgPool>,
) -> TplResult {
	endpoint(cfg, pool, TplMode::Scores).await
}

#[get("/uptime")]
async fn uptime<'r>(
	cfg: State<'r, Arc<Cfg>>,
	pool: State<'r, PgPool>,
) -> TplResult {
	endpoint(cfg, pool, TplMode::Scores).await
}

#[get("/slas")]
async fn slas<'r>(
	cfg: State<'r, Arc<Cfg>>,
	pool: State<'r, PgPool>,
) -> TplResult {
	endpoint(cfg, pool, TplMode::Scores).await
}

#[get("/patch-server")]
async fn patch_server<'r>(cfg: State<'r, Arc<Cfg>>) -> TplResult {
	let files = fs::read_dir(&*cfg.patch_server).unwrap();
	render_tpl(PatchServer {
		round: &*cfg.round,
		files: files
			.into_iter()
			.map(|f| f.unwrap().file_name().into_string().unwrap())
			.collect::<Vec<_>>(),
	})
}

#[get("/leaderboard")]
async fn leaderboard<'r>(
	cfg: State<'r, Arc<Cfg>>,
	pool: State<'r, PgPool>,
) -> TplResult {
	let mut conn = pool.acquire().await.unwrap();
	let teams = get_leaderboard(&mut conn).await;

	render_tpl(Leaderboard {
		round: &*cfg.round,
		teams,
	})
}

pub async fn start(pool: PgPool, cfg: Arc<Cfg>) -> Result<()> {
	let config = Config {
		port: cfg.web.port,
		address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
		..Config::default()
	};

	rocket::custom(config)
		.mount(
			"/",
			routes![root, scores, uptime, slas, patch_server, leaderboard],
		)
		.mount("/patch-files", StaticFiles::from(&*cfg.patch_server))
		.manage(cfg.clone())
		.manage(pool.clone())
		.launch()
		.await
		.context("Failed to launch web server!")
}
