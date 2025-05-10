use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::routing::{get, post};
use clap::{Parser, arg};
use sqlx::sqlite::SqlitePoolOptions;
use tokio::sync::RwLock;
use tower_http::services::{ServeDir, ServeFile};

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

// TODO: return an error or something so we don't have to keep using unwrap
#[tokio::main(flavor = "current_thread")]
async fn main() {
    // The .env file is required so it should panic here
    dotenvy::dotenv().unwrap();

    let args = Args::parse();
    let address = format!("{}:{}", args.address, args.port);

    let static_files = ServeDir::new("static").not_found_service(ServeFile::new("static/404.html"));

    let db_pool = SqlitePoolOptions::new()
        .min_connections(5)
        .max_connections(20)
        .acquire_timeout(Duration::new(10, 0)) // 10 seconds
        .idle_timeout(Some(Duration::new(300, 0))) // 5 minutes
        .max_lifetime(None)
        // I think it's okay for this to panic as this env var really needs to be set
        .connect(&dotenvy::var("DATABASE_URL").unwrap())
        // Same here maybe? Although if the program can't connect to the DB maybe it could create
        // one but idk how that would work
        .await
        .unwrap();

    let router = Router::new()
        .route("/join", get(join::get))
        .route("/join", post(join::post))
        .route("/leave", get(leave::get))
        .route("/leave", post(leave::post))
        .route("/next", get(ring::next))
        .route("/prev", get(ring::prev))
        .route("/random", get(ring::random))
        .route("/list", get(ring::list))
        .with_state(Arc::new(RwLock::new(RingState {
            ring_data: WebRing {},
            database: db_pool,
        })))
        .nest("/admin", admin::router())
        .fallback_service(static_files);

    // TODO: Switch to tracing subscriber or whatever logging library is popular rn
    println!("Binding to {}", address);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
