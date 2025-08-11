use askama::Template;
use axum::{
	Form,
	http::StatusCode,
	response::{Html, IntoResponse, Redirect},
};
use axum_messages::{Message, Messages};
use serde::Deserialize;
use tracing::{debug, error, info};

use crate::ring::{RingError, auth::AuthSession};

#[derive(Template)]
#[template(path = "admin/account/change-password.html")]
pub struct ChangePasswordTemplate {
	messages: Vec<Message>,
}

pub(super) async fn get(messages: Messages) -> impl IntoResponse {
	match {
		ChangePasswordTemplate {
			messages: messages.into_iter().collect(),
		}
	}
	.render()
	{
		Ok(s) => {
			debug!("Successfully rendered change password html");
			Html(s).into_response()
		}
		Err(e) => {
			error!("Error when rendering change password html: {e}");
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	}
}

#[derive(Clone, Deserialize)]
pub struct ChangePasswordForm {
	pub current_password: String,
	pub new_password: String,
	pub confirm_new_password: String,
}

pub(super) async fn post(
	mut auth_session: AuthSession,
	messages: Messages,
	Form(input): Form<ChangePasswordForm>,
) -> impl IntoResponse {
	let admin = match auth_session.user {
		Some(ref a) => a,
		None => {
			error!("Tried to alter an admin account when not logged in");
			return StatusCode::UNAUTHORIZED.into_response();
		}
	};
	if input.new_password != input.confirm_new_password {
		messages.error("New password and Confirmed new password do not match");
		return Redirect::to("./change-password").into_response();
	}
	match auth_session
		.backend
		.change_password(admin, input.current_password, input.new_password)
		.await
	{
		Ok(()) => {
			info!("Successfully changed admin password");
			// TODO: Make this an actual page
			Html(
				"Your password has successfully been changed. Please <a href=\"/login\">login</a> again.",
			)
			.into_response()
		}
		Err(RingError::UnauthorisedAdmin) => {
			messages.error("Current password not valid.");
			Redirect::to("./change-password").into_response()
		}
		Err(e) => {
			error!("Error when trying to change the users password: {e}");
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	}
}
