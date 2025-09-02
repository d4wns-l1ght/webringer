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
pub struct DenySiteForm {
	url: String,
	reason: String,
}

pub(super) async fn post(
	messages: Messages,
	auth_session: AuthSession,
	State(state): State<RingState>,
	Form(form): Form<DenySiteForm>,
) -> impl IntoResponse {
	let Some(admin) = auth_session.user else {
		return StatusCode::UNAUTHORIZED.into_response();
	};
	if let Err(e) = state.deny_site(&form.url, &form.reason, admin.id()).await {
		error!("Error when trying to deny site: {e}");
		return StatusCode::INTERNAL_SERVER_ERROR.into_response();
	}
	messages.info(format!(
		"Site {} denied with reason {}",
		form.url, form.reason
	));
	Redirect::to("/admin/view").into_response()
}
