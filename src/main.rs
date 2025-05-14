use std::time::Duration;

use axum::Router;
use axum::routing::{get, post};
use axum_login::AuthManagerLayerBuilder;
use axum_login::tower_sessions::{MemoryStore, SessionManagerLayer};
use axum_messages::MessagesManagerLayer;
use clap::{Parser, arg};
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use tokio::signal;
use tower_http::services::{ServeDir, ServeFile};
use tracing::{Instrument, error, info, info_span, instrument, warn};

use webringer::ring::*;
use webringer::site::*;

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

async fn get_db_pool() -> SqlitePool {
    match SqlitePoolOptions::new()
        .min_connections(5)
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(10)) // 10 seconds
        .idle_timeout(Some(Duration::from_secs(300))) // 5 minutes
        .max_lifetime(None)
        .connect(&dotenvy::var("DATABASE_URL").unwrap_or_else(|e| {
            let default_url = "sqlite://data.db".to_owned();
            warn!("Could not find DATABASE_URL environment variable. Using default url {}. Error message: {}", &default_url, e);
            default_url
        }))
        .instrument(info_span!("Create Database"))
        .await
    {
        Ok(db_pool) => db_pool,
        Err(e) => {
            error!("Could not connect to database: {}", e);
            panic!();
        }
    }
}

#[tokio::main(flavor = "current_thread")]
#[instrument]
async fn main() {
    tracing_subscriber::fmt::init();
    if let Err(e) = dotenvy::dotenv() {
        warn!("No .env file found: {}", e);
    }

    let args = Args::parse();
    let address = format!("{}:{}", args.address, args.port);

    let static_files = ServeDir::new("static").not_found_service(ServeFile::new("static/404.html"));

    let db_pool = get_db_pool();

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store);
    let backend = RingState::new(db_pool.await);
    let auth_layer = AuthManagerLayerBuilder::new(backend.clone(), session_layer).build();

    let router = Router::new()
        .route("/join", get(join::get))
        .route("/join", post(join::post))
        .route("/leave", get(leave::get))
        .route("/leave", post(leave::post))
        .route("/next", get(ring::next))
        .route("/prev", get(ring::prev))
        .route("/random", get(ring::random))
        .route("/list", get(ring::list))
        .route("/login", get(login::get))
        .route("/login", post(login::post))
        .with_state(backend.clone())
        .nest("/admin", admin::router(backend))
        .layer(MessagesManagerLayer)
        .layer(auth_layer)
        .nest_service("/static", static_files.clone())
        .fallback_service(static_files);

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
        error!("Axum serving error: {}", e)
    }
}
