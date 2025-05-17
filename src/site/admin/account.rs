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
use tracing::{debug, error, warn};

use crate::ring::{
    RingError, RingState,
    auth::{Admin, AuthSession},
};

pub(super) fn router(state: RingState) -> Router {
    Router::new()
        .route("/", get(view))
        .route("/delete", post(delete_account))
        .route("/change-password", get(change_password::get))
        .route("/change-password", post(change_password::post))
        .with_state(state)
}

mod change_password {
    use askama::Template;
    use axum::{
        Form,
        http::StatusCode,
        response::{Html, IntoResponse, Redirect},
    };
    use axum_messages::{Message, Messages};
    use serde::Deserialize;
    use tracing::{debug, error, info};

    use crate::ring::{RingError, auth::AuthSession};

    #[derive(Template)]
    #[template(path = "admin/account/change-password.html")]
    pub struct ChangePasswordTemplate {
        messages: Vec<Message>,
    }

    pub(super) async fn get(messages: Messages) -> impl IntoResponse {
        match {
            ChangePasswordTemplate {
                messages: messages.into_iter().collect(),
            }
        }
        .render()
        {
            Ok(s) => {
                debug!("Successfully rendered change password html");
                Html(s).into_response()
            }
            Err(e) => {
                error!("Error when rendering change password html: {e}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }

    #[derive(Clone, Deserialize)]
    pub struct ChangePasswordForm {
        pub current_password: String,
        pub new_password: String,
        pub confirm_new_password: String,
    }

    pub(super) async fn post(
        mut auth_session: AuthSession,
        messages: Messages,
        Form(input): Form<ChangePasswordForm>,
    ) -> impl IntoResponse {
        let admin = match auth_session.user {
            Some(ref a) => a,
            None => {
                error!("Tried to alter an admin account when not logged in");
                return StatusCode::UNAUTHORIZED.into_response();
            }
        };
        if input.new_password != input.confirm_new_password {
            messages.error("New password and Confirmed new password do not match");
            return Redirect::to("./change-password").into_response();
        }
        match auth_session
            .backend
            .change_password(admin, input.current_password, input.new_password)
            .await
        {
            Ok(()) => {
                info!("Successfully changed admin password");
                // TODO: Make this an actual page
                Html("Your password has successfully been changed. Please <a href=\"/login\">login</a> again.").into_response()
            }
            Err(RingError::UnauthorisedAdmin) => {
                messages.error("Current password not valid.");
                Redirect::to("./change-password").into_response()
            }
            Err(e) => {
                error!("Error when trying to change the users password: {e}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
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
            debug!(
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
        Ok(()) => ([("content-type", "0")], Redirect::to("/")).into_response(),
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
