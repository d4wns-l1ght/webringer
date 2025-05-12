use askama::Template;
use axum::{
    Form,
    extract::Query,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
};
use serde::Deserialize;
use tracing::{debug, error};

use crate::ring::auth::{AuthSession, Credentials};

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    next: Option<String>,
}

// This allows us to extract the "next" field from the query string. We use this
// to redirect after log in.
#[derive(Debug, Deserialize)]
pub struct NextUrl {
    next: Option<String>,
}

pub async fn post(
    mut auth_session: AuthSession,
    Form(creds): Form<Credentials>,
) -> impl IntoResponse {
    let admin = match auth_session.authenticate(creds.clone()).await {
        Ok(Some(admin)) => {
            debug!("Authenticated admin {:?}", &admin);
            admin
        }
        Ok(None) => {
            // TODO: User response that they didn't log in
            debug!("Authentication failed with credentials {:?}", &creds);
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
        debug!("Login successful, redirecting to {}", &next);
        Redirect::to(next)
    } else {
        debug!("Login successful, redirecting to /");
        Redirect::to("/admin")
    }
}

pub async fn get(Query(NextUrl { next }): Query<NextUrl>) -> impl IntoResponse {
    let t = LoginTemplate { next };
    match t.render() {
        Ok(s) => {
            debug!("Successfully rendered login html");
            Html(s).into_response()
        }
        Err(e) => {
            error!("Error when rendering login html: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
