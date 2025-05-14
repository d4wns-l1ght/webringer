use axum::{
    Router,
    routing::{get, post},
};
use axum_login::login_required;

use crate::ring::RingState;

mod add;
mod deny;
mod verify;

pub fn router(state: RingState) -> Router {
    Router::new()
        .route("/", get(landing_page))
        .route("/view", get(view))
        .route("/deny", post(deny::post))
        .route("/approve", post(verify::post))
        .route("/add", get(add::get))
        .route("/add", post(add::post))
        .route("/logout", post(logout))
        .with_state(state)
        .route_layer(login_required!(RingState, login_url = "/login"))
}

async fn landing_page() -> &'static str {
    "TODO! Admin view"
}

async fn view() -> &'static str {
    "TODO! Admin view"
}

async fn logout() -> &'static str {
    "TODO! Admin logout"
}
