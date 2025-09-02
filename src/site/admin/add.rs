use askama::Template;
use axum::{
	Form,
	extract::State,
	http::StatusCode,
	response::{Html, IntoResponse, Redirect},
};
use axum_messages::{Message, Messages};
use serde::Deserialize;
use std::fmt::Debug;
use tracing::{debug, error};

use crate::ring::{RingError, RingState};

#[derive(Template)]
#[template(path = "admin/add.html")]
pub struct AdminAddTemplate {
	messages: Vec<Message>,
}

#[derive(Deserialize)]
pub struct AddAdminForm {
	username: String,
	email: String,
	password: String,
	confirm_password: String,
}

impl Debug for AddAdminForm {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Admin")
			.field("username", &self.username)
			.field("email", &self.email)
			.field("password", &"redacted")
			.field("confirm_password", &"redacted")
			.finish()
	}
}

pub(super) async fn post(
	State(state): State<RingState>,
	messages: Messages,
	Form(data): Form<AddAdminForm>,
) -> impl IntoResponse {
	static PATH: &str = "/admin/add";
	if data.password != data.confirm_password {
		debug!("Passwords don't match");
		messages.error("Passwords do not match");
		return Redirect::to(PATH).into_response();
	}
	if let Err(e) = state
		.add_admin(data.username.clone(), data.email, data.password)
		.await
	{
		match e {
			RingError::UniqueRowAlreadyPresent(values) => {
				messages.error(format!("Email or username is already taken: {values}"));
				Redirect::to(PATH).into_response()
			}
			RingError::TaskJoin(e) => {
				error!("Task join error when adding admin: {}", e);
				StatusCode::INTERNAL_SERVER_ERROR.into_response()
			}
			RingError::UnrecoverableDatabaseError(e) => {
				error!("Unrecoverable database error when adding admin: {}", e);
				StatusCode::INTERNAL_SERVER_ERROR.into_response()
			}
			e => {
				error!(
					"Recieved an error site::admin::add::post is not equipped to handle: {}",
					e
				);
				StatusCode::INTERNAL_SERVER_ERROR.into_response()
			}
		}
	} else {
		messages.info(format!("Added new admin {}", data.username));
		Redirect::to(PATH).into_response()
	}
}

pub(super) async fn get(messages: Messages) -> impl IntoResponse {
	match {
		AdminAddTemplate {
			messages: messages.into_iter().collect(),
		}
	}
	.render()
	{
		Ok(s) => {
			debug!("Successfully rendered admin add html");
			Html(s).into_response()
		}
		Err(e) => {
			error!("Error when rendering admin add html: {}", e);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	}
}
