use axum::{
    Router,
    routing::{get, post},
};

mod deny;
mod verify;

pub fn router() -> Router {
    Router::new()
        .route("/", get(landing_page))
        .route("/view", get(view))
        .route("/deny", get(deny::get))
        .route("/deny", post(deny::post))
        .route("/approve", get(verify::get))
        .route("/approve", post(verify::post))
}

async fn landing_page() -> &'static str {
    "TODO! Admin view"
}

async fn view() -> &'static str {
    "TODO! Admin view"
}
