use async_trait::async_trait;
use axum_login::{AuthUser, AuthnBackend, UserId};
use password_auth::verify_password;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use std::fmt::Debug;
use tokio::task;

use super::{RingError, RingState};

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
            .field("email", &self.email)
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

#[async_trait]
impl AuthnBackend for RingState {
    type User = Admin;
    type Credentials = Credentials;
    type Error = RingError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let admin: Option<Self::User> = sqlx::query_as("SELECT * FROM admins WHERE username = ?")
            .bind(creds.username)
            .fetch_optional(&self.database)
            .await?;

        task::spawn_blocking(|| {
            Ok(admin.filter(|a| verify_password(creds.password, &a.password_phc).is_ok()))
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

pub type AuthSession = axum_login::AuthSession<RingState>;
