use std::sync::Arc;

use axum::extract::State;
use tokio::sync::RwLock;

use crate::ring::RingState;

pub async fn next(state: State<Arc<RwLock<RingState>>>) -> &'static str {
    "You are attempting to go to the next site"
}

pub async fn prev(state: State<Arc<RwLock<RingState>>>) -> &'static str {
    "You'd like to go to the previous site"
}

pub async fn random(state: State<Arc<RwLock<RingState>>>) -> &'static str {
    "You want a random site."
}
