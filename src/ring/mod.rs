//! This module handles the actual webring capabilities

use sqlx::{SqlitePool, sqlite::SqliteQueryResult};
use tracing::{debug, instrument};

#[derive(Debug)]
pub struct RingState {
    ring_data: WebRing,
    database: SqlitePool,
}

impl RingState {
    pub fn new(database: SqlitePool) -> Self {
        RingState {
            ring_data: WebRing {},
            database,
        }
    }

    #[instrument]
    pub async fn add_site(
        &mut self,
        root_url: &str,
        email: &str,
    ) -> Result<SqliteQueryResult, sqlx::Error> {
        debug!(
            "Running query 'INSERT INTO sites (root_url, email) values ({}, {})'",
            root_url, email
        );
        sqlx::query!(
            "INSERT INTO sites (root_url, email) values (?, ?)",
            root_url,
            email
        )
        .bind(root_url)
        .bind(email)
        .execute(&self.database)
        .await
    }

    #[instrument]
    pub async fn remove_site(&mut self, root_url: &str) -> Result<SqliteQueryResult, sqlx::Error> {
        debug!(
            "Running query 'DELETE FROM sites WHERE root_url = {}'",
            root_url
        );
        sqlx::query!("DELETE FROM sites WHERE root_url = ?", root_url)
            .bind(root_url)
            .execute(&self.database)
            .await
    }

    #[instrument]
    pub async fn get_random_site(&self) -> Result<String, sqlx::Error> {
        debug!("Running query 'SELECT root_url FROM verified_sites ORDER BY random() LIMIT 1");
        match sqlx::query!("SELECT root_url FROM verified_sites ORDER BY random() LIMIT 1")
            .fetch_one(&self.database)
            .await
        {
            Ok(record) => Ok(record.root_url),
            Err(e) => Err(e),
        }
    }

    #[instrument]
    pub async fn get_list(&self) -> Result<Vec<String>, sqlx::Error> {
        debug!("Running query SELECT root_url FROM verified_sites ORDER BY random()");
        Ok(
            sqlx::query!("SELECT root_url FROM verified_sites ORDER BY random()")
                .fetch_all(&self.database)
                .await?
                .into_iter()
                .map(|row| row.root_url)
                .collect(),
        )
    }
}

#[derive(Debug)]
struct WebRing {}
