use std::sync::Arc;

use axum::extract::State;
use tokio::sync::RwLock;

use crate::ring::RingState;

pub async fn get() -> &'static str {
    "You want to join the webring, and you didn't click the form."
}
pub async fn post(state: State<Arc<RwLock<RingState>>>) -> &'static str {
    "You want to join and you clicked the form!"
}
