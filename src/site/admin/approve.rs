use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_login::AuthUser;
use axum_messages::Messages;
use serde::Deserialize;
use tracing::error;

use crate::ring::{RingState, auth::AuthSession};

#[derive(Debug, Deserialize)]
pub struct ApproveSiteQuery {
    url: String,
}

pub(super) async fn post(
    messages: Messages,
    auth_session: AuthSession,
    State(state): State<RingState>,
    Query(query): Query<ApproveSiteQuery>,
) -> impl IntoResponse {
    let admin = match auth_session.user {
        Some(admin) => admin,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    if let Err(e) = state.approve_site(&query.url, admin.id()).await {
        error!("Error when trying to approve site: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    messages.info(format!("Site {} approved", query.url));
    Redirect::to("/admin/view").into_response()
}
