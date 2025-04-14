use axum::Router;
use axum::routing::{get, post};
use tower_http::services::{ServeDir, ServeFile};
use clap::{Parser, arg};

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

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    let static_files = ServeDir::new("static")
        .not_found_service(ServeFile::new("static/404.html"));
    let router = Router::new()
        // .route("/join", get(join::get))
        // .route("/join", post(join::post))
        // .route("/leave", get(leave::get))
        // .route("/leave", post(leave::post))
        // .route("/next", get(ring::next))
        // .route("/next", get(ring::prev))
        // .route("/random", get(ring::random))
        .fallback_service(static_files);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", args.address, args.port)).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
