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
    warn!("leave::get called");
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
) -> Html<String> {
    debug!("Write locking state");
    let mut state = state.write().await;
    match state.remove_site(&data.url).await {
        Ok(query_outcome) => {
            if query_outcome.rows_affected() == 0 {
                info!("Someone tried to remove {} but it didn't exist", data.url);
                Html("There has been an error: that site does not exist".to_owned())
            } else {
                info!("Site {} removed from webring", data.url);
                Html("Your site has been removed from the webring!".to_owned())
            }
        }
        Err(e) => {
            warn!("Error removing site: {}", e);
            Html(format!("There was an error: {}", e))
        }
    }
}
