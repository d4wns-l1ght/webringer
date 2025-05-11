//! This module handles the actual webring capabilities

use anyhow::{Result, anyhow};
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
    ) -> Result<SqliteQueryResult> {
        debug!(
            "Running query 'INSERT INTO sites (root_url, email) values ({}, {})'",
            root_url, email
        );
        Ok(sqlx::query!(
            "INSERT INTO sites (root_url, email) values (?, ?)",
            root_url,
            email
        )
        .bind(root_url)
        .bind(email)
        .execute(&self.database)
        .await?)
    }

    #[instrument]
    pub async fn remove_site(
        &mut self,
        root_url: &str,
    ) -> Result<SqliteQueryResult> {
        debug!(
            "Running query 'DELETE FROM sites WHERE root_url = {}'",
            root_url
        );
        Ok(
            sqlx::query!("DELETE FROM sites WHERE root_url = ?", root_url)
                .bind(root_url)
                .execute(&self.database)
                .await?,
        )
    }

    #[instrument]
    pub async fn get_random_site(&self) -> Result<String> {
        debug!("Running query 'SELECT root_url FROM verified_sites ORDER BY random() LIMIT 1");
        match sqlx::query!("SELECT root_url FROM verified_sites ORDER BY random() LIMIT 1")
            .fetch_one(&self.database)
            .await
        {
            Ok(record) => Ok(record.root_url),
            Err(e) => Err(anyhow!(e)),
        }
    }

    #[instrument]
    pub async fn get_list(&self) -> Result<Vec<String>> {
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

    #[instrument]
    pub async fn get_next(&self, current_url: &str) -> Result<String> {
        let id = self.get_verified_id(current_url).await?;
        debug!(
            "Running query SELECT root_url FROM verified_sites WHERE site_id > {id} ORDER BY site_id ASC LIMIT 1"
        );
        match sqlx::query!(
            "SELECT root_url FROM verified_sites WHERE site_id > ? ORDER BY site_id ASC LIMIT 1",
            id
        )
        .fetch_one(&self.database)
        .await
        {
            Ok(record) => Ok(record.root_url),
            Err(sqlx::Error::RowNotFound) => Ok("Webring Url".to_string()),
            Err(e) => Err(anyhow!(e)),
        }
    }

    #[instrument]
    pub async fn get_prev(&self, current_url: &str) -> Result<String> {
        let id = self.get_verified_id(current_url).await?;
        debug!(
            "Running query SELECT root_url FROM verified_sites WHERE site_id < {id} ORDER BY site_id ASC LIMIT 1"
        );
        match sqlx::query!(
            "SELECT root_url FROM verified_sites WHERE site_id < ? ORDER BY site_id ASC LIMIT 1",
            id
        )
        .fetch_one(&self.database)
        .await
        {
            Ok(record) => Ok(record.root_url),
            Err(sqlx::Error::RowNotFound) => Ok("Webring Url".to_string()),
            Err(e) => Err(anyhow!(e)),
        }
    }

    #[instrument]
    async fn get_verified_id(&self, root_url: &str) -> Result<i64> {
        debug!("Running query SELECT site_id FROM verified_sites WHERE root_url={root_url}");
        sqlx::query!(
            "SELECT site_id FROM verified_sites WHERE root_url=?",
            root_url
        )
        .fetch_one(&self.database)
        .await?
        .site_id
        .ok_or(anyhow!("Site url {root_url} is not a verified site"))
    }
}

#[derive(Debug)]
struct WebRing {}
