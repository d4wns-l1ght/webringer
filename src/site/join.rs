use std::sync::Arc;

use axum::{
    extract::{Form, State},
    response::Html,
};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::ring::RingState;

pub async fn get() -> &'static str {
    "You want to join the webring, and you didn't click the form."
}

#[derive(Debug, Deserialize)]
pub struct JoinForm {
    url: String,
    email: String,
}

pub async fn post(
    State(state): State<Arc<RwLock<RingState>>>,
    Form(data): Form<JoinForm>,
) -> Html<String> {
    let state = state.write().await;
    match sqlx::query!(
        "INSERT INTO sites (root_url, email) values (?, ?)",
        data.url,
        data.email
    )
    .execute(&state.database)
    .await
    {
        Ok(_query_outcome) => {
            Html("You want to join and you clicked the form! An admin will be in contact with you soon to verify your site.".to_owned())
        }
        Err(e) => Html(format!("There was an error: {}", e)),
    }
}
