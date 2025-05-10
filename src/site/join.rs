use std::sync::Arc;

use axum::{
    extract::{Form, State},
    response::Html,
};
use serde::Deserialize;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

use crate::ring::RingState;

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
pub async fn post(
    State(state): State<Arc<RwLock<RingState>>>,
    Form(data): Form<JoinForm>,
) -> Html<String> {
    debug!("Write locking state");
    let mut state = state.write().await;
    match state.add_site(&data.url, &data.email).await {
        Ok(_query_outcome) => {
            info!("Unverified site added to database");
            Html("You want to join and you clicked the form! An admin will be in contact with you soon to verify your site.".to_owned())
        }
        Err(e) => {
            warn!("Error adding site: {}", e);
            Html("There was an error when registering your site - are you sure you haven't registered it before?".to_string())
        }
    }
}
