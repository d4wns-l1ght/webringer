use std::sync::Arc;

use axum::{
    extract::{Form, State},
    response::{Html, IntoResponse},
};
use serde::Deserialize;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

use crate::ring::{RingError, RingState};

#[instrument]
pub async fn get() -> &'static str {
    info!("leave::get called");
    "You want to leave, but the form means nothing to you...."
}

#[derive(Debug, Deserialize)]
pub struct LeaveForm {
    url: String,
}

#[instrument]
pub async fn post(
    State(state): State<Arc<RwLock<RingState>>>,
    Form(data): Form<LeaveForm>,
) -> impl IntoResponse {
    debug!("Write locking state");
    let mut state = state.write().await;
    match state.remove_site(&data.url).await {
        Ok(()) => Html("Your site has been removed from the webring!".to_owned()),
        Err(RingError::SiteNotPresent(site)) => {
            Html(format!("The site {site} isn't present in our systems"))
        }
        Err(RingError::UnrecoverableDatabaseError(_e)) => Html(
            "We are having some backend problems currently, please try again later".to_string(),
        ),
        Err(e) => {
            warn!(
                "The remove_site function is returning an error we're not designed to handle: {}",
                e
            );
            Html(
                "We are having some backend problems currently, please try again later".to_string(),
            )
        }
    }
}
