use axum::{
    extract::{Query, State},
    http,
    response::{Html, IntoResponse, Redirect},
};
use serde::Deserialize;
use tracing::{debug, info, instrument, warn};

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
        Err(RingError::SiteNotVerified(_url)) => http::StatusCode::UNAUTHORIZED.into_response(),
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
            // TODO: Indicate this to the user somehow
            warn!("There are currently no verified sites in the webring");
            Redirect::to(".").into_response()
        }
        Err(e) => {
            warn!("{e}");
            Redirect::to(".").into_response()
        }
    }
}

#[instrument]
pub async fn list(State(state): State<RingState>) -> impl IntoResponse {
    let mut output: String = "<p>Webring sites:</p><ul>".to_owned();
    for url in {
        match state.get_list().await {
            Ok(urls) => urls,
            Err(RingError::RowNotFound(_query)) => {
                warn!("There are currently no verified sites in the webring");
                vec![]
            }
            Err(e) => {
                warn!("{e}");
                vec![]
            }
        }
    } {
        output = output + &format!("<li><a href=\"{url}\">{url}</a></li>");
    }
    output += "</ul>";
    debug!("Sending html: {output}");
    Html(output).into_response()
}
