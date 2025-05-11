//! This module handles the actual webring capabilities

use sqlx::SqlitePool;
use thiserror::Error;
use tracing::{debug, error, info, instrument};

#[derive(Debug)]
pub struct RingState {
    database: SqlitePool,
}

#[derive(Debug, Error)]
pub enum RingError {
    #[error("The query {0} did not return any rows")]
    RowNotFound(String),
    #[error("The site {0} is not verified")]
    SiteNotVerified(String),
    #[error("The site {0} is already present in the database")]
    SiteAlreadyPresent(String),
    #[error("The site {0} is not present in the database")]
    SiteNotPresent(String),
    #[error("The database had some kind of issue we couldn't recover from: {0}")]
    UnrecoverableDatabaseError(sqlx::Error),
}

impl RingState {
    pub fn new(database: SqlitePool) -> Self {
        RingState {
            database,
        }
    }

    /// Add a site to the webring
    /// Returns [RingError::SiteAlreadyPresent] if the site has already been registered
    /// Otherwise, [RingError::UnrecoverableDatabaseError]
    #[instrument]
    pub async fn add_site(&self, root_url: &str, email: &str) -> Result<(), RingError> {
        debug!(
            "Running query 'INSERT INTO sites (root_url, email) values ({}, {})'",
            root_url, email
        );
        match sqlx::query!(
            "INSERT INTO sites (root_url, email) values (?, ?)",
            root_url,
            email
        )
        .bind(root_url)
        .bind(email)
        .execute(&self.database)
        .await
        {
            Ok(_query_outcome) => {
                info!("Unverified site {} added to database", root_url);
                Ok(())
            }
            Err(sqlx::Error::Database(e)) => {
                if e.code().as_deref() == Some("2067") {
                    info!(
                        "Someone tried to register their site {} but it was already registered",
                        root_url
                    );
                    Err(RingError::SiteAlreadyPresent(root_url.to_owned()))
                } else {
                    error!("There was an unrecoverable database error: {}", e);
                    Err(RingError::UnrecoverableDatabaseError(
                        sqlx::Error::Database(e),
                    ))
                }
            }
            Err(e) => {
                error!("There was an unrecoverable database error: {}", e);
                Err(RingError::UnrecoverableDatabaseError(e))
            }
        }
    }

    /// Removes a site from the webring
    /// Returns [RingError::SiteNotPresent] if the site is not present
    /// Otherwise, [RingError::UnrecoverableDatabaseError]
    #[instrument]
    pub async fn remove_site(&self, root_url: &str) -> Result<(), RingError> {
        debug!(
            "Running query 'DELETE FROM sites WHERE root_url = {}'",
            root_url
        );
        match sqlx::query!("DELETE FROM sites WHERE root_url = ?", root_url)
            .bind(root_url)
            .execute(&self.database)
            .await
        {
            Ok(query_outcome) => {
                if query_outcome.rows_affected() == 0 {
                    info!(
                        "Someone tried to remove their site {} but it was already not there",
                        root_url
                    );
                    Err(RingError::SiteNotPresent(root_url.to_owned()))
                } else {
                    info!("Site {} removed from webring", root_url);
                    Ok(())
                }
            }
            Err(e) => {
                error!("There was an unrecoverable database error: {}", e);
                Err(RingError::UnrecoverableDatabaseError(e))
            }
        }
    }

    /// Gets the webring site after the current one
    /// Returns [RingError::SiteNotVerified] if the current site is not part of the webring
    /// Returns [RingError::RowNotFound] if the current site is last in the webring
    /// Otherwise, [RingError::UnrecoverableDatabaseError]
    #[instrument]
    pub async fn get_next(&self, current_url: &str) -> Result<String, RingError> {
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
            Err(sqlx::Error::RowNotFound) => Err(RingError::RowNotFound("SELECT root_url FROM verified_sites WHERE site_id > ? ORDER BY site_id ASC LIMIT 1".to_owned())),
            Err(e) => Err(RingError::UnrecoverableDatabaseError(e)),
        }
    }

    /// Gets the webring site before the current one
    /// Returns [RingError::SiteNotVerified] if the current site is not part of the webring
    /// Returns [RingError::RowNotFound] if the current site is last in the webring
    /// Otherwise, [RingError::UnrecoverableDatabaseError]
    #[instrument]
    pub async fn get_prev(&self, current_url: &str) -> Result<String, RingError> {
        let id = self.get_verified_id(current_url).await?;
        debug!(
            "Running query SELECT root_url FROM verified_sites WHERE site_id < {id} ORDER BY site_id ASC LIMIT 1"
        );
        match sqlx::query!(
            "SELECT root_url FROM verified_sites WHERE site_id > ? ORDER BY site_id ASC LIMIT 1",
            id
        )
        .fetch_one(&self.database)
        .await
        {
            Ok(record) => Ok(record.root_url),
            Err(sqlx::Error::RowNotFound) => Err(RingError::RowNotFound("SELECT root_url FROM verified_sites WHERE site_id < ? ORDER BY site_id ASC LIMIT 1".to_owned())),
            Err(e) => {
                error!("There was an unrecoverable database error: {}", e);
                Err(RingError::UnrecoverableDatabaseError(e))
            }
        }
    }

    #[instrument]
    async fn get_verified_id(&self, root_url: &str) -> Result<i64, RingError> {
        debug!("Running query SELECT site_id FROM verified_sites WHERE root_url={root_url}");
        match sqlx::query!(
            "SELECT site_id FROM verified_sites WHERE root_url=?",
            root_url
        )
        .fetch_one(&self.database)
        .await
        {
            Ok(record) => Ok(record
                .site_id
                .ok_or(RingError::SiteNotVerified(root_url.to_owned()))?),
            Err(sqlx::Error::RowNotFound) => {
                info!("The unverified site {root_url} tried to be a part of the webring");
                return Err(RingError::SiteNotVerified(root_url.to_owned()));
            }
            Err(e) => {
                error!("There was an unrecoverable database error: {}", e);
                Err(RingError::UnrecoverableDatabaseError(e))
            }
        }
    }

    /// Gets a random site from the webring
    /// Returns [RingError::RowNotFound] if there are no verified sites
    /// Otherwise, [RingError::UnrecoverableDatabaseError]
    #[instrument]
    pub async fn get_random_site(&self) -> Result<String, RingError> {
        debug!("Running query 'SELECT root_url FROM verified_sites ORDER BY random() LIMIT 1");
        match sqlx::query!("SELECT root_url FROM verified_sites ORDER BY random() LIMIT 1")
            .fetch_one(&self.database)
            .await
        {
            Ok(record) => Ok(record.root_url),
            Err(sqlx::Error::RowNotFound) => Err(RingError::RowNotFound(
                "SELECT root_url FROM verified_sites ORDER BY random() LIMIT 1".to_owned(),
            )),
            Err(e) => {
                error!("There was an unrecoverable database error: {}", e);
                Err(RingError::UnrecoverableDatabaseError(e))
            }
        }
    }

    /// Gets a list of all webring sites
    /// Returns [RingError::RowNotFound] if there are no verified sites
    /// Otherwise, [RingError::UnrecoverableDatabaseError]
    #[instrument]
    pub async fn get_list(&self) -> Result<Vec<String>, RingError> {
        debug!("Running query SELECT root_url FROM verified_sites ORDER BY random()");
        match sqlx::query!("SELECT root_url FROM verified_sites ORDER BY random()")
            .fetch_all(&self.database)
            .await
        {
            Ok(urls) => Ok(urls.into_iter().map(|row| row.root_url).collect()),
            Err(sqlx::Error::RowNotFound) => Err(RingError::RowNotFound(
                "SELECT root_url FROM verified_sites ORDER BY random()".to_owned(),
            )),
            Err(e) => {
                error!("There was an unrecoverable database error: {}", e);
                Err(RingError::UnrecoverableDatabaseError(e))
            }
        }
    }
}
