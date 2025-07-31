use askama::Template;
use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
};
use axum_login::AuthUser;
use serde::Deserialize;
use tracing::{debug, error, info, warn};

use crate::ring::{
    RingError, RingState,
    auth::{Admin, AuthSession},
};

mod change_password;

pub(super) fn router(state: RingState) -> Router {
    Router::new()
        .route("/", get(view))
        .route("/delete", post(delete_account))
        .route("/change-password", get(change_password::get))
        .route("/change-password", post(change_password::post))
        .with_state(state)
}

#[derive(Template)]
#[template(path = "admin/account.html")]
pub struct AdminAccountViewTemplate {
    admin: Admin,
    delete_button_pressed: bool,
}

async fn view(auth_session: AuthSession, Query(params): Query<DeleteParams>) -> impl IntoResponse {
    match (AdminAccountViewTemplate {
        admin: match auth_session.user {
            Some(admin) => admin,
            None => {
                error!("Admin method called without logged in admin");
                return StatusCode::UNAUTHORIZED.into_response();
            }
        },
        delete_button_pressed: {
            if params.delete_pressed == Some("true".to_owned()) {
                debug!("Account delete button pressed, asking for confirmation");
                true
            } else {
                false
            }
        },
    })
    .render()
    {
        Ok(s) => {
            debug!("Successfully rendered admin view HTML");
            Html(s).into_response()
        }
        Err(e) => {
            error!("Error when attempting to render admin view HTML: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DeleteParams {
    delete_pressed: Option<String>,
    delete_confirmed: Option<String>,
}

async fn delete_account(
    mut auth_session: AuthSession,
    State(state): State<RingState>,
    Query(params): Query<DeleteParams>,
) -> impl IntoResponse {
    if params.delete_confirmed != Some("true".to_owned()) {
        return (
            [("content-length", "0")],
            Redirect::to("/admin/account?delete_pressed=true"),
        )
            .into_response();
    }
    let admin = match auth_session.logout().await {
        Ok(Some(admin)) => {
            info!(
                "Successfully logged out admin as part of account deletion {:?}",
                admin
            );
            admin
        }
        Ok(None) => {
            warn!("Tried to logout for account deletion but there was no active user");
            return StatusCode::UNAUTHORIZED.into_response();
        }
        Err(e) => {
            error!("Error when logging out admin for account deletion: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    match state.delete_admin(admin.id()).await {
        Ok(()) => {
            info!("Deleted admin {:?}", admin);
            ([("content-type", "0")], Redirect::to("/")).into_response()
        }
        Err(RingError::RowNotFound(_message)) => {
            error!(
                "Tried to delete an admin that was not present in the database: {:?}",
                admin
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
        Err(e) => {
            error!("There was an error when trying to delete an admin: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
