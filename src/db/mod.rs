pub mod mutation;
pub mod query;

use anyhow::{Context as _, Result};
use sqlx::{pool::PoolOptions, Pool, Postgres};

pub type PgPool = Pool<Postgres>;
pub async fn establish_pg_conn(database: &str) -> Result<PgPool> {
	PoolOptions::default()
		.connect(database)
		.await
		.context("Failed to establish connection to PostgreSQL database")
}
