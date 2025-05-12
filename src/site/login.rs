use axum::{
    Form,
    extract::Query,
    response::{IntoResponse, Redirect},
};
use serde::Deserialize;
use tracing::error;

use crate::ring::auth::{AuthSession, Credentials};

#[derive(Debug, Deserialize)]
pub struct NextUrl {
    next: Option<String>,
}

pub async fn post(
    mut auth_session: AuthSession,
    Form(creds): Form<Credentials>,
) -> impl IntoResponse {
    let admin = match auth_session.authenticate(creds.clone()).await {
        Ok(Some(admin)) => admin,
        Ok(None) => {
            // TODO: User response that they didn't log in
            let mut login_url = "/login".to_owned();
            if let Some(next) = creds.next {
                login_url = format!("{}?next={}", login_url, next)
            };

            return Redirect::to(&login_url);
        }
        Err(e) => {
            // TODO: User response to tell about error
            error!("Error when authenticating admin: {}", e);
            return Redirect::to(".");
        }
    };

    if let Err(e) = auth_session.login(&admin).await {
        error!("Error when logging in admin: {}", e);
        return Redirect::to(".");
    }

    if let Some(ref next) = creds.next {
        Redirect::to(next)
    } else {
        Redirect::to("/")
    }
}

pub async fn get(Query(NextUrl { next: _next }): Query<NextUrl>) -> &'static str {
    // TODO: Add templates
    "This is where you log in"
}
