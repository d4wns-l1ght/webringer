//! This module handles the actual webring capabilities

use argon2::password_hash;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use thiserror::Error;
use tokio::task;
use tracing::{debug, error, info, instrument};

pub mod auth;

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct ApprovedSite {
	pub site_id: i64,
	pub root_url: String,
	pub site_email: String,
	pub date_added: String,
	pub admin_id: i64,
	pub admin_username: String,
	pub admin_email: String,
}

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct UnapprovedSite {
	pub id: i64,
	pub root_url: String,
	pub email: String,
}

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct DeniedSite {
	pub site_id: i64,
	pub root_url: String,
	pub site_email: String,
	pub date_added: String,
	pub reason: String,
	pub admin_id: i64,
	pub admin_username: String,
	pub admin_email: String,
}

#[derive(Debug, Clone)]
pub struct RingState {
	database: SqlitePool,
}

#[derive(Debug, Error)]
pub enum RingError {
	#[error("The query {0} did not return any rows")]
	RowNotFound(String),
	#[error("The row {0} is already present in the database")]
	UniqueRowAlreadyPresent(String),
	#[error("The site {0} is not approved")]
	SiteNotApproved(String),
	#[error(transparent)]
	UnrecoverableDatabaseError(#[from] sqlx::Error),
	#[error(transparent)]
	TaskJoin(#[from] task::JoinError),
	#[error("Password verification error: {0}")]
	PasswordVerification(password_hash::Error),
	#[error("An admin method was called outside of an authorised session")]
	UnauthorisedAdmin,
}

impl RingState {
	pub fn new(database: SqlitePool) -> Self {
		RingState { database }
	}

	/// Add a site to the webring
	/// Returns [RingError::SiteAlreadyPresent] if the site has already been registered
	/// Otherwise, [RingError::UnrecoverableDatabaseError]
	#[instrument]
	pub async fn add_site(&self, root_url: &str, email: &str) -> Result<(), RingError> {
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
				info!("Unapproved site {} added to database", root_url);
				Ok(())
			}
			Err(sqlx::Error::Database(ref e)) if e.code().as_deref() == Some("2067") => {
				info!(
					"Someone tried to register their site {} but it was already registered",
					root_url
				);
				Err(RingError::UniqueRowAlreadyPresent(root_url.to_owned()))
			}
			Err(e) => {
				error!(
					"There was an unrecoverable database error in add_site: {}",
					e
				);
				Err(RingError::UnrecoverableDatabaseError(e))
			}
		}
	}

	/// Removes a site from the webring
	/// Returns [RingError::SiteNotPresent] if the site is not present
	/// Otherwise, [RingError::UnrecoverableDatabaseError]
	#[instrument]
	pub async fn remove_site(&self, root_url: &str) -> Result<(), RingError> {
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
					Err(RingError::RowNotFound(root_url.to_owned()))
				} else {
					info!("Site {} removed from webring", root_url);
					Ok(())
				}
			}
			Err(e) => {
				error!(
					"There was an unrecoverable database error in remove_site: {}",
					e
				);
				Err(RingError::UnrecoverableDatabaseError(e))
			}
		}
	}

	#[instrument]
	pub async fn approve_site(&self, root_url: &str, admin_id: i64) -> Result<(), RingError> {
		let mut tx = match self.database.begin().await {
			Ok(tx) => tx,
			Err(e) => return Err(RingError::UnrecoverableDatabaseError(e)),
		};

		let approval_id = match sqlx::query!(
			"INSERT INTO approval_records (date_added, admin_id) VALUES (date('now'), ?)",
			admin_id
		)
		.execute(&mut *tx)
		.await
		{
			Ok(query_outcome) => query_outcome.last_insert_rowid(),
			Err(e) => {
				error!("There was an error when adding an approval record");
				return Err(RingError::UnrecoverableDatabaseError(e));
			}
		};

		if let Err(e) = sqlx::query!(
			"UPDATE sites SET approval_id = ? WHERE root_url = ?",
			approval_id,
			root_url
		)
		.execute(&mut *tx)
		.await
		{
			// TODO: Distinguish for the type of error you get when there is a constraint error
			// (e.g. there is already a denial_id set or vice versa)
			return Err(RingError::UnrecoverableDatabaseError(e));
		};

		if let Err(e) = tx.commit().await {
			return Err(RingError::UnrecoverableDatabaseError(e));
		}

		Ok(())
	}

	#[instrument]
	pub async fn deny_site(
		&self,
		root_url: &str,
		reason: &str,
		admin_id: i64,
	) -> Result<(), RingError> {
		let mut tx = match self.database.begin().await {
			Ok(tx) => tx,
			Err(e) => return Err(RingError::UnrecoverableDatabaseError(e)),
		};

		let denial_id = match sqlx::query!(
			"INSERT INTO denial_records (date_added, admin_id, reason) VALUES (date('now'), ?, ?)",
			admin_id,
			reason
		)
		.execute(&mut *tx)
		.await
		{
			Ok(query_outcome) => query_outcome.last_insert_rowid(),
			Err(e) => {
				error!("There was an error when adding a denial record");
				return Err(RingError::UnrecoverableDatabaseError(e));
			}
		};

		if let Err(e) = sqlx::query!(
			"UPDATE sites SET denial_id = ? WHERE root_url = ?",
			denial_id,
			root_url
		)
		.execute(&mut *tx)
		.await
		{
			return Err(RingError::UnrecoverableDatabaseError(e));
		};

		if let Err(e) = tx.commit().await {
			return Err(RingError::UnrecoverableDatabaseError(e));
		}

		Ok(())
	}

	/// Gets the webring site after the current one
	/// Returns [RingError::SiteNotApproved] if the current site is not part of the webring
	/// Returns [RingError::RowNotFound] if the current site is last in the webring
	/// Otherwise, [RingError::UnrecoverableDatabaseError]
	#[instrument]
	pub async fn get_next(&self, current_url: &str) -> Result<String, RingError> {
		let id = self.get_approved_site_id(current_url).await?;
		match sqlx::query!(
            "SELECT root_url FROM approved_sites WHERE site_id > ? ORDER BY site_id ASC LIMIT 1",
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
	/// Returns [RingError::SiteNotApproved] if the current site is not part of the webring
	/// Returns [RingError::RowNotFound] if the current site is last in the webring
	/// Otherwise, [RingError::UnrecoverableDatabaseError]
	#[instrument]
	pub async fn get_prev(&self, current_url: &str) -> Result<String, RingError> {
		let id = self.get_approved_site_id(current_url).await?;
		match sqlx::query!(
            "SELECT root_url FROM approved_sites WHERE site_id > ? ORDER BY site_id ASC LIMIT 1",
            id
        )
        .fetch_one(&self.database)
        .await
        {
            Ok(record) => Ok(record.root_url),
            Err(sqlx::Error::RowNotFound) => Err(RingError::RowNotFound("SELECT root_url FROM verified_sites WHERE site_id < ? ORDER BY site_id ASC LIMIT 1".to_owned())),
            Err(e) => {
                error!("There was an unrecoverable database error in get_prev: {}", e);
                Err(RingError::UnrecoverableDatabaseError(e))
            }
        }
	}

	#[instrument]
	async fn get_approved_site_id(&self, root_url: &str) -> Result<i64, RingError> {
		match sqlx::query!(
			"SELECT site_id FROM approved_sites WHERE root_url=?",
			root_url
		)
		.fetch_one(&self.database)
		.await
		{
			Ok(record) => Ok(record
				.site_id
				.ok_or(RingError::SiteNotApproved(root_url.to_owned()))?),
			Err(sqlx::Error::RowNotFound) => {
				info!("The unapproved site {root_url} tried to be a part of the webring");
				return Err(RingError::SiteNotApproved(root_url.to_owned()));
			}
			Err(e) => {
				error!(
					"There was an unrecoverable database error in get_verified_id: {}",
					e
				);
				Err(RingError::UnrecoverableDatabaseError(e))
			}
		}
	}

	/// Gets a random site from the webring
	/// Returns [RingError::RowNotFound] if there are no approved sites
	/// Otherwise, [RingError::UnrecoverableDatabaseError]
	#[instrument]
	pub async fn get_random_site(&self) -> Result<String, RingError> {
		match sqlx::query!("SELECT root_url FROM approved_sites ORDER BY random() LIMIT 1")
			.fetch_one(&self.database)
			.await
		{
			Ok(record) => Ok(record.root_url),
			Err(sqlx::Error::RowNotFound) => Err(RingError::RowNotFound(
				"SELECT root_url FROM verified_sites ORDER BY random() LIMIT 1".to_owned(),
			)),
			Err(e) => {
				error!(
					"There was an unrecoverable database error in get_random_site: {}",
					e
				);
				Err(RingError::UnrecoverableDatabaseError(e))
			}
		}
	}

	/// Gets a list of all approved webring sites
	/// Returns [RingError::RowNotFound] if there are no verified sites
	/// Otherwise, [RingError::UnrecoverableDatabaseError]
	#[instrument]
	pub async fn get_list_approved(&self) -> Result<Vec<ApprovedSite>, RingError> {
		match sqlx::query_as("SELECT * FROM approved_sites ORDER BY random()")
			.fetch_all(&self.database)
			.await
		{
			Ok(sites) => Ok(sites),
			Err(sqlx::Error::RowNotFound) => Err(RingError::RowNotFound(
				"SELECT root_url FROM verified_sites ORDER BY random()".to_owned(),
			)),
			Err(e) => {
				error!(
					"There was an unrecoverable database error in get_list_approved: {}",
					e
				);
				Err(RingError::UnrecoverableDatabaseError(e))
			}
		}
	}

	#[instrument]
	pub async fn get_list_denied(&self) -> Result<Vec<DeniedSite>, RingError> {
		match sqlx::query_as("SELECT * FROM denied_sites")
			.fetch_all(&self.database)
			.await
		{
			Ok(sites) => Ok(sites),
			Err(sqlx::Error::RowNotFound) => Err(RingError::RowNotFound(
				"SELECT root_url FROM denied_sites".to_owned(),
			)),
			Err(e) => {
				error!(
					"There was an unrecoverable database error in get_list_denied: {}",
					e
				);
				Err(RingError::UnrecoverableDatabaseError(e))
			}
		}
	}

	/// Gets a list of all unapproved webring sites
	#[instrument]
	pub async fn get_list_unapproved(&self) -> Result<Vec<UnapprovedSite>, RingError> {
		match sqlx::query_as("SELECT * FROM unapproved_sites ORDER BY id")
			.fetch_all(&self.database)
			.await
		{
			Ok(sites) => Ok(sites),
			Err(sqlx::Error::RowNotFound) => Err(RingError::RowNotFound(
				"SELECT * FROM unverified_sites ORDER BY id".to_owned(),
			)),
			Err(e) => {
				error!(
					"There was an unrecoverable database error in get_list_unapproved: {}",
					e
				);
				Err(RingError::UnrecoverableDatabaseError(e))
			}
		}
	}

	#[instrument]
	pub async fn add_admin(
		&self,
		username: String,
		email: String,
		password_plaintext: String,
	) -> Result<(), RingError> {
		debug!("Add admin function running");
		let password_hashed = auth::hash_password(password_plaintext).await?;
		match sqlx::query!(
			"INSERT INTO admins (username, email, password_phc) values (?, ?, ?)",
			username,
			email,
			password_hashed
		)
		.execute(&self.database)
		.await
		{
			Ok(_query_result) => {
				info!("Added admin to database: {} {}", username, email);
				Ok(())
			}
			Err(sqlx::Error::Database(ref e)) if e.code().as_deref() == Some("2067") => {
				info!(
					"Admin username {} or email {} already taken",
					username, email
				);
				Err(RingError::UniqueRowAlreadyPresent(format!(
					"{username} {email}"
				)))
			}
			Err(e) => {
				error!(
					"There was an unrecoverable database error in add_admin: {}",
					e
				);
				Err(RingError::UnrecoverableDatabaseError(e))
			}
		}
	}
	#[instrument]
	pub async fn delete_admin(&self, admin_id: i64) -> Result<(), RingError> {
		match sqlx::query("DELETE FROM admins WHERE id = ?")
			.bind(admin_id)
			.execute(&self.database)
			.await
		{
			Ok(query) if query.rows_affected() == 0 => {
				error!("No admin found to delete. {:?}", query);
				Err(RingError::RowNotFound(format!(
					"Admin with admin id {admin_id:?}"
				)))
			}
			Ok(query) => {
				info!(
					"Successfully deleted admin account with id {:?}: {:?}",
					admin_id, query
				);
				Ok(())
			}
			Err(e) => {
				error!(
					"There was a database error when trying to delete admin with id {}: {}",
					admin_id, e
				);
				Err(RingError::UnrecoverableDatabaseError(e))
			}
		}
	}
}
