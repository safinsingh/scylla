pub mod mutation;
pub mod query;

use dotenv::dotenv;
use sqlx::{pool::PoolOptions, Pool, Postgres};
use std::env;

pub type PgPool = Pool<Postgres>;
pub async fn establish_pg_conn() -> PgPool {
	dotenv().ok();

	let database_url =
		env::var("DATABASE_URL").expect("DATABASE_URL must be set");

	PoolOptions::default().connect(&database_url).await.unwrap()
}
