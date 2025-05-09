//! This module handles the actual webring capabilities

use sqlx::SqlitePool;

pub struct RingState {
    pub ring_data: WebRing,
    pub database: SqlitePool,
}

pub struct WebRing {}
