use askama::Template;
use axum::{
	extract::{Query, State},
	http::{self, StatusCode},
	response::{Html, IntoResponse, Redirect},
};
use serde::Deserialize;
use tracing::{debug, error, info, instrument, warn};

use crate::ring::{RingError, RingState};

#[derive(Debug, Deserialize)]
pub struct MoveParams {
	current: String,
}

#[instrument]
pub async fn next(
	Query(params): Query<MoveParams>,
	State(state): State<RingState>,
) -> impl IntoResponse {
	next_prev_redirect(state.get_next(&params.current).await, params.current).await
}

#[instrument]
pub async fn prev(
	Query(params): Query<MoveParams>,
	State(state): State<RingState>,
) -> impl IntoResponse {
	next_prev_redirect(state.get_prev(&params.current).await, params.current).await
}

#[instrument]
async fn next_prev_redirect(
	url: Result<String, RingError>,
	original_url: String,
) -> impl IntoResponse {
	match url {
		Ok(url) => {
			debug!("Redirecting user to {url}");
			Redirect::to(&url).into_response()
		}
		Err(RingError::SiteNotApproved(url)) => {
			debug!("Site {} unauthorized", url);
			http::StatusCode::UNAUTHORIZED.into_response()
		}
		Err(RingError::RowNotFound(_query)) => {
			debug!("End of webring found, redirecting user to home");
			Redirect::to(".").into_response()
		}
		Err(RingError::UnrecoverableDatabaseError(_e)) => {
			Redirect::to(&original_url).into_response()
		}
		Err(e) => {
			warn!(
				"The next_prev_redirect function is returning an error we're not designed to handle: {}",
				e
			);
			Redirect::to(&original_url).into_response()
		}
	}
}

#[instrument]
pub async fn random(State(state): State<RingState>) -> impl IntoResponse {
	match state.get_random_site().await {
		Ok(url) => {
			info!("Redirecting user to {}", &url);
			Redirect::to(&url).into_response()
		}
		Err(RingError::RowNotFound(_query)) => {
			warn!("There are currently no approved sites in the webring");
			Html("<h1>Error</h1><br><p>There are currently no approved sites in the webring :( Maybe you should add yours!").into_response()
		}
		Err(e) => {
			warn!("{e}");
			Redirect::to(".").into_response()
		}
	}
}

#[derive(Template)]
#[template(path = "list.html")]
pub struct ListTemplate {
	sites: Vec<String>,
}

#[instrument]
pub async fn list(State(state): State<RingState>) -> impl IntoResponse {
	match match state.get_list_approved().await {
		Ok(sites) => ListTemplate {
			sites: sites.into_iter().map(|site| site.root_url).collect(),
		},
		Err(RingError::RowNotFound(_query)) => {
			warn!("There are currently no approved sites in the webring");
			ListTemplate { sites: vec![] }
		}
		Err(e) => {
			error!("Error when getting the list of approved sites: {e}");
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	}
	.render()
	{
		Ok(s) => {
			debug!("Successfully rendered list html");
			Html(s).into_response()
		}
		Err(e) => {
			error!("Error when rendering list html: {}", e);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	}
}
