use askama::Template;
use axum::{
    http::StatusCode, response::{Html, IntoResponse, Redirect}, routing::{get, post}, Router
};
use axum_login::login_required;
use tracing::{debug, error, warn};

use crate::ring::{auth::AuthSession, RingState};

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

async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
    match auth_session.logout().await {
        Ok(Some(admin)) => debug!("Successfully logged out admin {:?}", admin),
        Ok(None) => warn!("Tried to logout but there was no active user"),
        Err(e) => error!("Error when logging out admin: {}", e),
    };
    ([("content-length", "0")], Redirect::to("/"))
}
