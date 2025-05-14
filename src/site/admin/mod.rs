use askama::Template;
use axum::{
    Router,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use axum_login::login_required;
use tracing::{debug, error};

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

#[derive(Template)]
#[template(path = "admin/landing_page.html")]
pub struct AdminLandingPageTemplate {}

async fn landing_page() -> impl IntoResponse {
    match { AdminLandingPageTemplate {} }.render() {
        Ok(s) => {
            debug!("Successfully rendered admin landing page html");
            Html(s).into_response()
        }
        Err(e) => {
            error!("Error when rendering login html: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn view() -> &'static str {
    "TODO! Admin view"
}

async fn logout() -> &'static str {
    "TODO! Admin logout"
}
