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
use tracing::{debug, error, info, warn};

use crate::ring::{
	ApprovedSite, DeniedSite, RingError, RingState, UnapprovedSite, auth::AuthSession,
};

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
pub struct AdminLandingPageTemplate {
	messages: Vec<Message>,
}

async fn landing_page(messages: Messages) -> impl IntoResponse {
	match {
		AdminLandingPageTemplate {
			messages: messages.into_iter().collect(),
		}
	}
	.render()
	{
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
	messages: Vec<Message>,
	unapproved_sites: Vec<UnapprovedSite>,
	approved_sites: Vec<ApprovedSite>,
	denied_sites: Vec<DeniedSite>,
}

async fn view(messages: Messages, State(state): State<RingState>) -> impl IntoResponse {
	match (AdminViewSitesTemplate {
		messages: messages.into_iter().collect(),
		unapproved_sites: {
			match state.get_list_unapproved().await {
				Ok(sites) => sites,
				Err(RingError::RowNotFound(_query)) => vec![],
				Err(e) => {
					error!("Error when getting the admin view of sites: {e}");
					return StatusCode::INTERNAL_SERVER_ERROR.into_response();
				}
			}
		},
		approved_sites: {
			match state.get_list_approved().await {
				Ok(sites) => sites,
				Err(RingError::RowNotFound(_query)) => vec![],
				Err(e) => {
					error!("Error when getting the admin view of sites: {e}");
					return StatusCode::INTERNAL_SERVER_ERROR.into_response();
				}
			}
		},
		denied_sites: {
			match state.get_list_denied().await {
				Ok(sites) => sites,
				Err(RingError::RowNotFound(_query)) => vec![],
				Err(e) => {
					error!("Error when getting the admin view of sites: {e}");
					return StatusCode::INTERNAL_SERVER_ERROR.into_response();
				}
			}
		},
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
		Ok(Some(admin)) => info!("Admin {:?} logged out", admin),
		Ok(None) => warn!("Tried to logout but there was no active user"),
		Err(e) => error!("Error when logging out admin: {}", e),
	};
	([("content-length", "0")], Redirect::to("/"))
}
