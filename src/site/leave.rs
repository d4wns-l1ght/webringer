use std::sync::Arc;

use axum::extract::State;
use tokio::sync::RwLock;

use crate::ring::RingState;

pub async fn get() -> &'static str {
    "You want to leave, but the form means nothing to you...."
}
pub async fn post(state: State<Arc<RwLock<RingState>>>) -> &'static str {
    "You are trying to leave through the form."
}
