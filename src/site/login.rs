use askama::Template;
use axum::{
    Form,
    extract::Query,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
};
use axum_messages::{Message, Messages};
use serde::Deserialize;
use tracing::{debug, error, info};

use crate::ring::auth::{AuthSession, Credentials};

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    messages: Vec<Message>,
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
    messages: Messages,
    Form(creds): Form<Credentials>,
) -> impl IntoResponse {
    let admin = match auth_session.authenticate(creds.clone()).await {
        Ok(Some(admin)) => {
            info!("Authenticated admin {:?}", &admin);
            admin
        }
        Ok(None) => {
            debug!("Authentication failed with credentials {:?}", &creds);
            messages.error("Invalid credentials");
            let mut login_url = "/login".to_owned();
            if let Some(next) = creds.next {
                login_url = format!("{login_url}?next={next}")
            };

            return Redirect::to(&login_url);
        }
        Err(e) => {
            // TODO: Maybe make this return a statuscode instead?
            messages.error(format!("Error when authenticating admin: {e}"));
            error!("Error when authenticating admin: {}", e);
            return Redirect::to(".");
        }
    };

    if let Err(e) = auth_session.login(&admin).await {
        error!("Error when logging in admin: {}", e);
        return Redirect::to(".");
    }

    // FIXME: This isn't working
    if let Some(ref next) = creds.next {
        debug!("Login successful, redirecting to {}", &next);
        Redirect::to(next)
    } else {
        debug!("Login successful, redirecting to /");
        Redirect::to("/")
    }
}

pub async fn get(messages: Messages, Query(NextUrl { next }): Query<NextUrl>) -> impl IntoResponse {
    match {
        LoginTemplate {
            messages: messages.into_iter().collect(),
            next,
        }
    }
    .render()
    {
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
