use async_std::{
	channel, fs,
	sync::{Arc, Mutex},
	task,
};
use clap::Clap;
use libscylla::{
	checks::{enter_event_loop, enter_recv_loop, injects, ChanMsg},
	cli::{Opts, SubCommand},
	config::Cfg,
	db::{establish_pg_conn, mutation, PgPool},
	web,
};

#[async_std::main]
async fn main() -> anyhow::Result<()> {
	let content = fs::read_to_string("./scylla.hocon").await?;
	let cfg = Arc::new(hocon::de::from_str::<Cfg>(&content)?);
	let pool = establish_pg_conn().await;

	let opts = Opts::parse();
	match opts.subcmd {
		SubCommand::Prepare => mutation::setup(&*cfg, pool).await,
		SubCommand::Start => run(cfg, pool).await,
	}

	Ok(())
}

async fn run(cfg: Arc<Cfg>, pool: PgPool) {
	let services = Arc::new(Mutex::new(cfg.get_services()));
	let (tx, rx) = channel::unbounded::<ChanMsg>();

	// periodically run checks
	task::spawn(enter_event_loop(cfg.clone(), services.clone(), tx));

	// start web server
	task::spawn(web::start(pool.clone(), cfg.clone())).await;

	// // begin inject waiter
	task::spawn(injects::enter_waiter(cfg.clone(), services.clone()));

	// // recieve messages from channel on main task
	enter_recv_loop(rx, pool.clone()).await;
}
