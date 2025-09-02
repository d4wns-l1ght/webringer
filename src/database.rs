use std::time::Duration;

use crate::args::read_env_var;
use sqlx::SqlitePool;
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqlitePoolOptions;
use tracing::{Instrument, error, info_span, trace};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub(super) async fn get_db_pool() -> SqlitePool {
	let db_url: String = read_env_var("DATABASE_URL", "sqlite://data.db".to_string());

	if db_url.starts_with("sqlite://") {
		let path = db_url.trim_start_matches("sqlite://");
		if !std::path::Path::new(path).exists() {
			std::fs::File::create(path).expect("Failed to create database file");
		}
	}

	let min_connections = read_env_var("MIN_CONNECTIONS", 5);
	let max_connections = read_env_var("MAX_CONNECTIONS", 5);
	let acquire_timeout = Duration::from_secs(read_env_var("ACQUIRE_TIMEOUT_SECS", 10u64));
	let idle_timeout = Some(Duration::from_secs(read_env_var(
		"IDLE_TIMEOUT_SECS",
		300u64,
	)));

	let pool = match SqlitePoolOptions::new()
		.min_connections(min_connections)
		.max_connections(max_connections)
		.acquire_timeout(acquire_timeout) // 10 seconds
		.idle_timeout(idle_timeout) // 5 minutes
		.max_lifetime(None)
		.connect(&db_url)
		.instrument(info_span!("Create Database"))
		.await
	{
		Ok(db_pool) => db_pool,
		Err(e) => {
			error!("Could not connect to database: {}", e);
			panic!();
		}
	};

	trace!("Running migrations");
	MIGRATOR.run(&pool).await.expect("Could not run migrations");

	pool
}
