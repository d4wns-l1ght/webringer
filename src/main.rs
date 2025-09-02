#![warn(clippy::pedantic, clippy::all, clippy::cargo)]
#![allow(clippy::module_name_repetitions, clippy::multiple_crate_versions)]
use std::fmt::Debug;
use std::str::FromStr;
use std::time::Duration;

use axum::Router;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum_login::AuthManagerLayerBuilder;
use axum_login::tower_sessions::{MemoryStore, SessionManagerLayer};
use axum_messages::MessagesManagerLayer;
use clap::{Parser, arg};
use sqlx::SqlitePool;
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqlitePoolOptions;
use tokio::signal;
use tracing::{Instrument, error, info, info_span, instrument, trace, warn};

use webringer::ring;
use webringer::site;

/// A server for hosting a webring!
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
	/// Address to bind to
	#[arg(short, long, value_name = "ADDR", default_value = "0.0.0.0")]
	address: String,

	/// Port to listen on
	#[arg(short, long, value_name = "PORT", default_value_t = 10983)]
	port: u16,
}

fn read_env_var<T>(key: &str, default: T) -> T
where
	T: FromStr + Debug,
{
	dotenvy::var(key)
		.ok()
		.and_then(|s| s.parse().ok())
		.unwrap_or_else(|| {
			info!("Using default {:?} value of {:?}", key, default);
			default
		})
}

#[derive(rust_embed::Embed)]
#[folder = "static/"]
struct Static;

struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
	T: Into<String>,
{
	fn into_response(self) -> axum::response::Response {
		let path: String = self.0.into();

		match Static::get(path.as_str()) {
			Some(content) => {
				let mime = mime_guess::from_path(path).first_or_octet_stream();
				(
					[(axum::http::header::CONTENT_TYPE, mime.as_ref())],
					content.data,
				)
					.into_response()
			}
			None => (axum::http::StatusCode::NOT_FOUND, "404 Not Found").into_response(),
		}
	}
}

async fn static_handler(uri: axum::http::Uri) -> impl IntoResponse {
	let mut path = uri.path().trim_start_matches('/').to_string();

	if path.starts_with("static/") {
		path = path.replace("static/", "");
	}

	StaticFile(path)
}

async fn not_found() -> impl IntoResponse {
	static_handler(axum::http::Uri::from_static("/static/404.html")).await
}

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

async fn get_db_pool() -> SqlitePool {
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

#[tokio::main(flavor = "current_thread")]
#[instrument]
async fn main() {
	tracing_subscriber::fmt::init();
	let args = Args::parse();

	if let Err(e) = dotenvy::dotenv() {
		warn!("No .env file found: {}", e);
	}

	let address = format!("{}:{}", args.address, args.port);

	let db_pool = get_db_pool();

	let session_store = MemoryStore::default();
	let session_layer = SessionManagerLayer::new(session_store);
	let backend = ring::RingState::new(db_pool.await);
	let auth_layer = AuthManagerLayerBuilder::new(backend.clone(), session_layer).build();

	let router = Router::new()
		.route("/", get(site::index))
		.route("/join", get(site::join::get))
		.route("/join", post(site::join::post))
		.route("/leave", get(site::leave::get))
		.route("/leave", post(site::leave::post))
		.route("/next", get(site::ring::next))
		.route("/prev", get(site::ring::prev))
		.route("/random", get(site::ring::random))
		.route("/list", get(site::ring::list))
		.route("/login", get(site::login::get))
		.route("/login", post(site::login::post))
		.with_state(backend.clone())
		.nest("/admin", site::admin::router(backend))
		.layer(MessagesManagerLayer)
		.layer(auth_layer)
		.route("/static/{*file}", get(static_handler))
		.fallback_service(get(not_found));

	info!("Binding to {}", address);
	let listener = match tokio::net::TcpListener::bind(address)
		.instrument(info_span!("TcpListener bind"))
		.await
	{
		Ok(listener) => listener,
		Err(e) => {
			error!("Tcp binding error: {}", e);
			panic!()
		}
	};
	if let Err(e) = axum::serve(listener, router)
		.with_graceful_shutdown(async {
			if let Err(e) = signal::ctrl_c().await {
				error!("Failed to listen for ctrl_c signal: {}", e);
				panic!()
			}
			info!("Gracefully shutting down from SIGINT");
		})
		.await
	{
		error!("Axum serving error: {}", e);
	}
}
