use axum::{Router, routing::get};
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
    // build our application with a single route
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", args.address, args.port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
