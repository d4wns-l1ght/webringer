use axum::{
    Router,
    routing::{get, post},
};
use axum_login::login_required;

use crate::ring::RingState;

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
        .route_layer(login_required!(RingState, login_url = "/login"))
}

async fn landing_page() -> &'static str {
    "TODO! Admin view"
}

async fn view() -> &'static str {
    "TODO! Admin view"
}
