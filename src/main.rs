#![warn(clippy::pedantic, clippy::all, clippy::cargo)]
#![allow(clippy::module_name_repetitions, clippy::multiple_crate_versions)]
use axum::{
	Router,
	routing::{get, post},
};
use axum_login::AuthManagerLayerBuilder;
use axum_login::tower_sessions::{MemoryStore, SessionManagerLayer};
use axum_messages::MessagesManagerLayer;
use clap::Parser;
use tokio::signal;
use tracing::{Instrument, error, info, info_span, instrument, warn};

use webringer::ring;
use webringer::site;

mod args;
mod database;
mod embed;

#[tokio::main(flavor = "current_thread")]
#[instrument]
async fn main() {
	tracing_subscriber::fmt::init();
	let args = args::Args::parse();

	if let Err(e) = dotenvy::dotenv() {
		warn!("No .env file found: {}", e);
	}

	let address = format!("{}:{}", args.address, args.port);

	let db_pool = database::get_db_pool();

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
		.route("/static/{*file}", get(embed::static_handler))
		.fallback_service(get(embed::not_found));

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
