use axum::{extract::{FromRequestParts, FromRef}, http::{request::Parts, StatusCode}};
use crate::{auth::middleware::RequireAuth, models::user::UserRole, state::AppState};

// For routes that require admin role
pub struct RequireAdmin(pub RequireAuth);

impl<S> FromRequestParts<S> for RequireAdmin
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        // First extract the authenticated user
        let require_auth = RequireAuth::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        // Check if the user has admin role
        if require_auth.0.role != UserRole::Admin {
            return Err(StatusCode::FORBIDDEN);
        }

        Ok(RequireAdmin(require_auth))
    }
}
