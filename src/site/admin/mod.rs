use axum::{
    Router,
    routing::{get, post},
};

mod deny;
mod login;
mod verify;

pub fn router() -> Router {
    Router::new()
        .route("/", get(async || "TODO! Admin view"))
        .route("/deny", get(deny::get))
        .route("/deny", post(deny::post))
        .route("/approve", get(verify::get))
        .route("/approve", post(verify::post))
        .route("/login", get(login::get))
        .route("/login", post(login::post))
}
