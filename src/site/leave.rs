use std::sync::Arc;

use axum::{
    extract::{Form, State},
    response::Html,
};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::ring::RingState;

pub async fn get() -> &'static str {
    "You want to leave, but the form means nothing to you...."
}

#[derive(Debug, Deserialize)]
pub struct LeaveForm {
    url: String,
}

pub async fn post(
    State(state): State<Arc<RwLock<RingState>>>,
    Form(data): Form<LeaveForm>,
) -> Html<String> {
    let state = state.write().await;
    match sqlx::query!("DELETE FROM sites WHERE root_url = ?", data.url,)
        .execute(&state.database)
        .await
    {
        Ok(_query_outcome) => Html("Your site has been removed from the webring!".to_owned()),
        Err(e) => Html(format!("There was an error: {}", e)),
    }
}
