use serde::{Deserialize, Serialize};
use validator::Validate;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
pub struct RegisterUserRequest {
    pub user: RegisterUserData,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterUserData {
    #[validate(length(min = 3, max = 50, message = "Username must be between 3 and 50 characters"))]
    pub username: String,

    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUserRequest {
    pub user: LoginUserData,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user: UserData,
    pub access_token: String,   // New: separate access token
    pub refresh_token: String,  // New: refresh token
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginUserData {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub user: UserData,
}

#[derive(Debug, Serialize)]
pub struct UserData {
    pub email: String,
    pub token: String,
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub email_verified: bool,  // NEW FIELD
    pub role: crate::models::user::UserRole,  // NEW FIELD
}

impl UserData {
    pub fn from_user_with_token(user: crate::models::User, token: String) -> Self {
        Self {
            email: user.email,
            token,
            username: user.username,
            bio: user.bio,
            image: user.image,
            email_verified: user.email_verified,  // NEW
            role: user.role,  // NEW
        }
    }
}

// Refresh token request
#[derive(Debug, Deserialize, Validate)]
pub struct RefreshTokenRequest {
    #[validate(length(min = 1, message = "Refresh token is required"))]
    pub refresh_token: String,
}

// Refresh token response
#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

// Logout request
#[derive(Debug, Deserialize, Validate)]
pub struct LogoutRequest {
    #[validate(length(min = 1, message = "Refresh token is required"))]
    pub refresh_token: String,
}

// Logout response
#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub message: String,
}

// Session response
#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_used: bool,
}

// Sessions list response
#[derive(Debug, Serialize)]
pub struct SessionsListResponse {
    pub sessions: Vec<SessionResponse>,
}

// Terminate session request
#[derive(Debug, Deserialize, Validate)]
pub struct TerminateSessionRequest {
    #[validate(length(min = 1, message = "Session ID is required"))]
    pub session_id: String,
}

// Terminate session response
#[derive(Debug, Serialize)]
pub struct TerminateSessionResponse {
    pub message: String,
}

// Delete account request
#[derive(Debug, Deserialize, Validate)]
pub struct DeleteAccountRequest {
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

// Delete account response
#[derive(Debug, Serialize)]
pub struct DeleteAccountResponse {
    pub message: String,
}

// User data export response
#[derive(Debug, Serialize)]
pub struct UserDataExportResponse {
    pub user: serde_json::Value,
    pub export_date: DateTime<Utc>,
}
