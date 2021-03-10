use anyhow::Result;
use clap::Clap;
use libscylla::{
	checks::{enter_event_loop, enter_recv_loop, injects, ChanMsg},
	cli::{Opts, SubCommand},
	config::Cfg,
	db::{establish_pg_conn, mutation, PgPool},
	web,
};
use std::{fs, sync::Arc};
use tokio::{sync::mpsc, task};

#[tokio::main]
async fn main() -> Result<()> {
	let opts = Opts::parse();

	let content = fs::read_to_string("./scylla.hocon")?;
	let cfg = Arc::new(hocon::de::from_str::<Cfg>(&content)?.set_services()?);
	let pool = establish_pg_conn(&cfg.database).await?;

	match opts.subcmd {
		SubCommand::Prepare => mutation::setup(cfg, pool).await,
		SubCommand::Start => run(cfg, pool).await,
	}
}

async fn run(cfg: Arc<Cfg>, pool: PgPool) -> Result<()> {
	let (tx, rx) = mpsc::unbounded_channel::<ChanMsg>();

	// periodically run checks
	task::spawn(enter_event_loop(cfg.clone(), tx));

	// start web server
	task::spawn(web::start(pool.clone(), cfg.clone()));

	// begin inject waiter
	task::spawn(injects::wait(cfg.clone()));

	// recieve messages from channel on main task
	enter_recv_loop(rx, pool.clone()).await
}
