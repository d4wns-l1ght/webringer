//! This module handles the actual webring capabilities

use sqlx::SqlitePool;

#[derive(Debug)]
pub struct RingState {
    pub ring_data: WebRing,
    pub database: SqlitePool,
}

#[derive(Debug)]
pub struct WebRing {}
