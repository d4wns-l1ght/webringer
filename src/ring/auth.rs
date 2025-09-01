//! This module handles the implementation of authorization for the webring

use argon2::{
	Argon2,
	password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum_login::{AuthUser, AuthnBackend, UserId};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use std::fmt::Debug;
use tracing::{debug, error, info};

use super::{RingError, RingState};

pub(super) async fn hash_password(password_plaintext: String) -> Result<String, RingError> {
	match tokio::task::spawn_blocking(move || {
		let argon2 = Argon2::default();
		let salt = SaltString::generate(&mut OsRng);
		let password_hash = argon2
			.hash_password(password_plaintext.as_bytes(), &salt)
			.unwrap();
		password_hash.to_string()
	})
	.await
	{
		Ok(password) => {
			debug!("Successfully hashed password");
			Ok(password)
		}
		Err(e) => {
			error!("Error when joining task");
			Err(RingError::TaskJoin(e))
		}
	}
}

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct Admin {
	id: i64,
	pub username: String,
	pub email: String,
	password_phc: String,
}

// Manually impl so that password hash isn't shown
impl Debug for Admin {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Admin")
			.field("id", &self.id)
			.field("username", &self.username)
			.field("email", &"redacted")
			.field("password", &"redacted")
			.finish()
	}
}

impl AuthUser for Admin {
	type Id = i64;

	fn id(&self) -> Self::Id {
		self.id
	}

	fn session_auth_hash(&self) -> &[u8] {
		self.password_phc.as_bytes()
	}
}

#[derive(Clone, Deserialize)]
pub struct Credentials {
	pub username: String,
	pub password: String,
	pub next: Option<String>,
}

impl Debug for Credentials {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Credentials")
			.field("username", &self.username)
			.field("password", &"redacted")
			.field("next url", &self.next)
			.finish()
	}
}

impl AuthnBackend for RingState {
	type User = Admin;
	type Credentials = Credentials;
	type Error = RingError;

	async fn authenticate(
		&self,
		creds: Self::Credentials,
	) -> Result<Option<Self::User>, Self::Error> {
		let admin = match sqlx::query_as::<_, Admin>("SELECT * FROM admins WHERE username = ?")
			.bind(&creds.username)
			.fetch_optional(&self.database)
			.await?
		{
			Some(admin) => admin,
			None => {
				debug!("Couldn't find an admin with username {}", creds.username);
				return Ok(None);
			}
		};

		// Verifying the password is blocking and potentially slow, so we'll do so via `spawn_blocking`.
		tokio::task::spawn_blocking(move || {
			let password_hash = match PasswordHash::new(&admin.password_phc) {
				Ok(parsed_hash) => parsed_hash,
				Err(e) => {
					error!("Error parsing stored password hash for {:?}: {}", admin, e);
					return Err(RingError::PasswordVerification(e));
				}
			};

			match Argon2::default().verify_password(creds.password.as_bytes(), &password_hash) {
				Ok(()) => {
					info!("Verified admin {:?}", admin);
					Ok(Some(admin))
				}
				Err(argon2::password_hash::Error::Password) => {
					info!("Invalid login to admin {:?}", admin);
					Ok(None)
				}
				Err(e) => {
					info!("Problem when verfifying password: {}", e);
					Err(RingError::PasswordVerification(e))
				}
			}
		})
		.await?
	}

	async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
		let admin = sqlx::query_as("SELECT * FROM admins WHERE id = ?")
			.bind(user_id)
			.fetch_optional(&self.database)
			.await?;
		Ok(admin)
	}
}

impl RingState {
	pub async fn change_password(
		&mut self,
		logged_in_user: &Admin,
		current_password_plaintext: String,
		new_password_plaintext: String,
	) -> Result<(), RingError> {
		let admin = match self
			.authenticate(Credentials {
				username: logged_in_user.username.clone(),
				password: current_password_plaintext,
				next: None,
			})
			.await?
		{
			Some(admin) => admin,
			None => {
				return Err(RingError::UnauthorisedAdmin);
			}
		};
		let new_password_hashed = hash_password(new_password_plaintext).await?;
		if let Err(e) = sqlx::query!(
			"UPDATE admins SET password_phc = ? WHERE id = ?",
			new_password_hashed,
			admin.id
		)
		.execute(&self.database)
		.await
		{
			error!("Error when trying to update admin password: {e}");
			return Err(RingError::UnrecoverableDatabaseError(e));
		};
		Ok(())
	}
}

pub type AuthSession = axum_login::AuthSession<RingState>;
