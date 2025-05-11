use axum::{
    extract::{Form, State},
    response::{Html, IntoResponse},
};
use serde::Deserialize;
use tracing::{info, instrument, warn};

use crate::ring::{RingError, RingState};

#[instrument]
pub async fn get() -> &'static str {
    info!("join::get called");
    "You want to join the webring, and you didn't click the form."
}

#[derive(Debug, Deserialize)]
pub struct JoinForm {
    url: String,
    email: String,
}

#[instrument]
pub async fn post(State(state): State<RingState>, Form(data): Form<JoinForm>) -> impl IntoResponse {
    match state.add_site(&data.url, &data.email).await {
        Ok(()) => {
            Html("You want to join and you clicked the form! An admin will be in contact with you soon to verify your site.".to_owned())
        }
        Err(RingError::SiteAlreadyPresent(site)) => {
            Html(format!("The site {site} has already been registered"))
        }
        Err(RingError::UnrecoverableDatabaseError(_e)) => {
            Html("We are having some backend problems currently, please try again later".to_string())
        }
        Err(e) => {
            warn!("The add_site function is returning an error we're not designed to handle: {}",e);
            Html("We are having some backend problems currently, please try again later".to_string())
        }
    }
}
