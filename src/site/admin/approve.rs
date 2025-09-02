use axum::{
	Form,
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Redirect},
};
use axum_login::AuthUser;
use axum_messages::Messages;
use serde::Deserialize;
use tracing::error;

use crate::ring::{RingState, auth::AuthSession};

#[derive(Debug, Deserialize)]
pub struct ApproveSiteForm {
	url: String,
}

pub(super) async fn post(
	messages: Messages,
	auth_session: AuthSession,
	State(state): State<RingState>,
	Form(form): Form<ApproveSiteForm>,
) -> impl IntoResponse {
	let Some(admin) = auth_session.user else {
		return StatusCode::UNAUTHORIZED.into_response();
	};
	if let Err(e) = state.approve_site(&form.url, admin.id()).await {
		error!("Error when trying to approve site: {e}");
		return StatusCode::INTERNAL_SERVER_ERROR.into_response();
	}
	messages.info(format!("Site {} approved", form.url));
	Redirect::to("/admin/view").into_response()
}
