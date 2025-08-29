use askama::Template;
use axum::{
	extract::{Form, Query, State},
	http::StatusCode,
	response::{Html, IntoResponse, Redirect},
};
use axum_messages::{Message, Messages};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tracing::{debug, error, instrument};

use crate::ring::{RingError, RingState};

#[derive(Template)]
#[template(path = "join.html")]
pub struct JoinTemplate {
	messages: Vec<Message>,
	url: String,
	email: String,
	url_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct JoinParams {
	url: String,
	email: String,
}

#[instrument]
pub async fn get(messages: Messages, Query(params): Query<JoinParams>) -> impl IntoResponse {
	let hasher = Sha256::new();
	Sha256::new().update(&params.url);
	let hashed_url = hasher.finalize();
	let hashed_url_hex = hex::encode(hashed_url);

	match {
		JoinTemplate {
			messages: messages.into_iter().collect(),
			url: params.url,
			email: params.email,
			url_hash: hashed_url_hex,
		}
	}
	.render()
	{
		Ok(s) => {
			debug!("Successfully rendered join html");
			Html(s).into_response()
		}
		Err(e) => {
			error!("Error when rendering join html: {}", e);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	}
}

#[derive(Debug, Deserialize)]
pub struct JoinForm {
	url: String,
	email: String,
	url_hash: String,
}

#[instrument]
pub async fn post(
	messages: Messages,
	State(state): State<RingState>,
	Form(data): Form<JoinForm>,
) -> impl IntoResponse {
	let redirect_here =
		Redirect::to(&format!("/join?url={}&email={}", data.url, data.email)).into_response();
	let response = match ureq::get(format!("{}/webringer/auth", data.url)).call() {
		Ok(response) if response.status() == StatusCode::NOT_FOUND => {
			messages.error(format!(
				"Got a 404 error when trying to get {}/webringer/auth",
				data.url
			));
			return redirect_here;
		}
		Ok(response) => match response.into_body().read_to_string() {
			Ok(text) => text.trim().to_owned(),
			Err(e) => {
				error!("Error when converting join verify body to text: {e}");
				return StatusCode::INTERNAL_SERVER_ERROR.into_response();
			}
		},
		Err(e) => {
			messages.error(format!(
				"There was an error when getting the verification string from your site: {e}"
			));
			return redirect_here;
		}
	};

	if response != data.url_hash {
		error!("Response: {} Url hash: {}", response, data.url_hash);
		messages.error(format!(
			"Url hash found but did not match:\n{}\n{}",
			response, data.url_hash
		));
		return redirect_here;
	}
	match state.add_site(&data.url, &data.email).await {
		Ok(()) => {
			Html("Your site has been registered, please wait for admin to approve it".to_owned())
				.into_response()
		}
		Err(RingError::UniqueRowAlreadyPresent(site)) => {
			messages.error(format!("The site {site} has already been registered"));
			redirect_here
		}
		Err(RingError::UnrecoverableDatabaseError(e)) => {
			error!("There was a database error when adding a site: {}", e);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
		Err(e) => {
			error!(
				"The add_site function is returning an error we're not designed to handle: {}",
				e
			);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	}
}
