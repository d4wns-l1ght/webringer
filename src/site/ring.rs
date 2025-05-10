use std::sync::Arc;

use axum::extract::State;
use axum::response::Redirect;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

use crate::ring::RingState;

#[instrument]
pub async fn next(state: State<Arc<RwLock<RingState>>>) -> &'static str {
    "You are attempting to go to the next site"
}

#[instrument]
pub async fn prev(state: State<Arc<RwLock<RingState>>>) -> &'static str {
    "You'd like to go to the previous site"
}

#[instrument]
pub async fn random(state: State<Arc<RwLock<RingState>>>) -> Redirect {
    debug!("Locking state for read");
    let state = state.read().await;
    let site_url = match state.get_random_site().await
    {
        Ok(url) => url,
        Err(e) => {
            let default_url = "Webring url".to_owned();
            warn!(
                "Random site error: {} Defaulting to home url {}",
                e, &default_url
            );
            default_url
        }
    };
    info!("Redirecting user to {}", &site_url);
    Redirect::to(&site_url)
}

#[instrument]
pub async fn list(state: State<Arc<RwLock<RingState>>>) -> &'static str {
    "You want to see a list of the webring sites? sorgy.."
}
