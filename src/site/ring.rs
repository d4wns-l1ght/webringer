use std::sync::Arc;

use axum::extract::State;
use axum::response::Redirect;
use tokio::sync::RwLock;

use crate::ring::RingState;

pub async fn next(state: State<Arc<RwLock<RingState>>>) -> &'static str {
    "You are attempting to go to the next site"
}

pub async fn prev(state: State<Arc<RwLock<RingState>>>) -> &'static str {
    "You'd like to go to the previous site"
}

pub async fn random(state: State<Arc<RwLock<RingState>>>) -> Redirect {
    let state = state.read().await;
    let site_url = match sqlx::query!("SELECT * FROM verified_sites ORDER BY random() LIMIT 1").fetch_one(&state.database).await {
        Ok(record) => record.root_url,
        Err(_e) => "Webring url".to_owned(),
    };
    Redirect::to(&site_url)
}
