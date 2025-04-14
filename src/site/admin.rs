use axum::{routing::get, Router};

pub fn router() -> Router {
    Router::new()
        .route("/", get(async || "TODO! Admin view"))
}
