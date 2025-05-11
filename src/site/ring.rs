use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http,
    response::{Html, IntoResponse, Redirect},
};
use serde::Deserialize;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

use crate::ring::RingState;

#[derive(Debug, Deserialize)]
pub struct MoveParams {
    current: String,
}

#[instrument]
pub async fn next(
    Query(params): Query<MoveParams>,
    State(state): State<Arc<RwLock<RingState>>>,
) -> impl IntoResponse {
    debug!("Locking state for read");
    let state = state.read().await;
    match state.get_next(&params.current).await {
        Ok(url) => Redirect::to(&url),
        Err(e) => match e.downcast_ref::<sqlx::Error>() {
            Some(sqlx::Error::RowNotFound) => Redirect::to("."),
            _ => {
                error!("Error getting next site: {e}");
                info!("Redirecting user to {0}", params.current);
                Redirect::to(&params.current)
            }
        },
    }
}

#[instrument]
pub async fn prev(
    Query(params): Query<MoveParams>,
    State(state): State<Arc<RwLock<RingState>>>,
) -> impl IntoResponse {
    debug!("Locking state for read");
    let state = state.read().await;
    match state.get_prev(&params.current).await {
        Ok(url) => Redirect::to(&url),
        Err(e) => match e.downcast_ref::<sqlx::Error>() {
            Some(sqlx::Error::RowNotFound) => Redirect::to("."),
            _ => {
                error!("Error getting next site: {e}");
                info!("Redirecting user to {0}", params.current);
                Redirect::to(&params.current)
            }
        },
    }
}

#[instrument]
pub async fn random(State(state): State<Arc<RwLock<RingState>>>) -> impl IntoResponse {
    debug!("Locking state for read");
    let state = state.read().await;
    match state.get_random_site().await {
        Ok(url) => {
            info!("Redirecting user to {}", &url);
            Redirect::to(&url).into_response()
        }
        Err(e) => match e.downcast_ref::<sqlx::Error>() {
            Some(sqlx::Error::RowNotFound) => {
                let default_url = "Webring url".to_owned();
                warn!(
                    "No webring sites available. Defaulting to home url {}",
                    &default_url
                );
                Redirect::to(&default_url).into_response()
            }
            _ => {
                error!("Error when getting random site: {e}");
                http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
    }
}

#[instrument]
pub async fn list(State(state): State<Arc<RwLock<RingState>>>) -> impl IntoResponse {
    let mut output: String = "<p>Webring sites:</p><ul>".to_owned();
    for url in {
        debug!("Locking state for read");
        let state = state.read().await;
        match state.get_list().await {
            Ok(urls) => urls,
            Err(e) => match e.downcast_ref::<sqlx::Error>() {
                Some(sqlx::Error::RowNotFound) => {
                    info!("No rows found for list");
                    vec![]
                }
                _ => {
                    error!("Error when getting the list of sites: {}", e);
                    return http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            },
        }
    } {
        output = output + &format!("<li><a href=\"{url}\">{url}</a></li>");
    }
    output += "</ul>";
    debug!("Sending html: {output}");
    Html(output).into_response()
}
