use axum::{
    extract::{Form, State},
    response::{Html, IntoResponse},
};
use serde::Deserialize;
use tracing::{info, instrument, warn};

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
    State(state): State<RingState>,
    Form(data): Form<LeaveForm>,
) -> impl IntoResponse {
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
