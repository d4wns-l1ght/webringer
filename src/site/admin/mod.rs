use askama::Template;
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
};
use axum_login::login_required;
use axum_messages::{Message, Messages};
use tracing::{debug, error, warn};

use crate::ring::{RingError, RingState, UnverifiedSite, auth::AuthSession};

mod account;
mod add;
mod approve;
mod deny;

pub fn router(state: RingState) -> Router {
    Router::new()
        .route("/", get(landing_page))
        .route("/view", get(view))
        .route("/deny", post(deny::post))
        .route("/approve", post(approve::post))
        .route("/add", get(add::get))
        .route("/add", post(add::post))
        .route("/logout", post(logout))
        .with_state(state.clone())
        .nest("/account", account::router(state))
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

#[derive(Template)]
#[template(path = "admin/sites_view.html")]
pub struct AdminViewSitesTemplate {
    unverified_sites: Vec<UnverifiedSite>,
    messages: Vec<Message>,
}

async fn view(messages: Messages, State(state): State<RingState>) -> impl IntoResponse {
    match (AdminViewSitesTemplate {
        unverified_sites: {
            match state.get_list_unverified().await {
                Ok(sites) => sites,
                Err(RingError::RowNotFound(_query)) => vec![],
                Err(e) => {
                    error!("Error when getting the admin view of sites: {e}");
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            }
        },
        messages: messages.into_iter().collect(),
    })
    .render()
    {
        Ok(s) => {
            debug!("Successfully rendered admin list view html");
            Html(s).into_response()
        }
        Err(e) => {
            error!("Error when rendering list html: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
    match auth_session.logout().await {
        Ok(Some(admin)) => debug!("Successfully logged out admin {:?}", admin),
        Ok(None) => warn!("Tried to logout but there was no active user"),
        Err(e) => error!("Error when logging out admin: {}", e),
    };
    ([("content-length", "0")], Redirect::to("/"))
}
