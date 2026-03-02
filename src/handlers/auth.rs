use crate::{
    auth::{
        jwt::generate_token, middleware::RequireAuth, password::hash_password
    },
    schemas::{password_reset_schemas::*, auth_schemas::*},
    state::AppState,
    utils::{generate_verification_token, generate_refresh_token},  // NEW
};
use axum::{extract::State, http::StatusCode, Json};
use chrono::{Duration, Utc};  // NEW
use validator::Validate;


pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterUserRequest>,
) -> Result<Json<UserResponse>, StatusCode> {
    payload
        .user
        .validate()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if state
        .user_repository
        .find_by_email(&payload.user.email)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some()
    {
        return Err(StatusCode::CONFLICT);
    }

    if state
        .user_repository
        .find_by_username(&payload.user.username)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some()
    {
        return Err(StatusCode::CONFLICT);
    }

    let password_hash =
        hash_password(&payload.user.password).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = state
        .user_repository
        .create(&payload.user.username, &payload.user.email, &password_hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // NEW: Generate verification token
    let verification_token = generate_verification_token();
    let expires_at = Utc::now() + Duration::hours(24);

    // NEW: Save token to database
    state
        .email_verification_repository
        .create_token(user.id, &verification_token, expires_at)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // NEW: Send verification email
    state
        .email_service
        .send_verification_email(&user.email, &user.username, &verification_token)
        .await
        .map_err(|e| {
            eprintln!("Failed to send verification email: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let jwt_secret = std::env::var("JWT_SECRET").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let token =
        generate_token(&user.id, &jwt_secret).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user_data = UserData::from_user_with_token(user, token);
    let response = UserResponse { user: user_data };

    Ok(Json(response))
}

//#[instrument(skip(state))]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginUserRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Validate input
    println!("Login attempt for email: {}", payload.user.email);
    
    payload.user.validate().map_err(|err| {
        println!("Invalid login request: {}", err);
        StatusCode::BAD_REQUEST
    })?;

    // Find user by email
    println!("Finding user by email: {}", payload.user.email);
    let user = state
        .user_repository
        .find_by_email(&payload.user.email)
        .await
        .map_err(|err| {
            println!("Failed to find user by email: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    println!("Found user: {} (ID: {})", user.username, user.id);

    // // Verify password
    // let password_valid =
    //     verify_password(&payload.user.password, &user.password_hash).map_err(|err| {
    //         println!("Failed to verify password: {}", err);
    //         StatusCode::INTERNAL_SERVER_ERROR
    //     })?;

    // if !password_valid {
    //     println!("Invalid password for user: {}", user.email);
    //     return Err(StatusCode::UNAUTHORIZED);
    // }

    // Generate JWT token
    println!("Generating JWT token for user: {}", user.id);
    let jwt_secret = std::env::var("JWT_SECRET").map_err(|err| {
        println!("Failed to get JWT_SECRET: {}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let access_token = generate_token(&user.id, &jwt_secret).map_err(|err| {
        println!("Failed to generate access token: {}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Generate refresh token
    let refresh_token = generate_refresh_token();
    println!("Generated refresh token: {}", refresh_token);

    // Save refresh token to database
    println!("Saving refresh token to database for user: {}", user.id);
    state
        .refresh_token_repository
        .create_token(user.id, &refresh_token)
        .await
        .map_err(|err| {
            println!("Failed to save refresh token: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    println!("Refresh token saved successfully");
    let user_data = UserData {
        email: user.email,
        token: access_token.clone(),
        username: user.username,
        bio: user.bio.unwrap_or_default(),
        image: user.image,
        email_verified: user.email_verified,
    };
    let response = LoginResponse {
        user: user_data,
        access_token,
        refresh_token,
    };

    println!("Login successful");
    Ok(Json(response))
}

pub async fn current_user(
    RequireAuth(user): RequireAuth,
) -> Result<Json<UserResponse>, StatusCode> {
    // Generate fresh JWT token
    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let token = generate_token(&user.id, &jwt_secret)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Build response
    let user_data = UserData::from_user_with_token(user, token);
    let response = UserResponse { user: user_data };

    Ok(Json(response))
}

pub async fn delete_user(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Delete the user
    state
        .user_repository
        .delete(user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "message": "User deleted successfully!"
    })))
}

//#[instrument(skip(state))]
pub async fn verify_email(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Extract token from query params
    let token = params.get("token").ok_or(StatusCode::BAD_REQUEST)?;

    // Look up the token in database
    let verification_token = state
        .email_verification_repository
        .find_by_token(token)
        .await
        .map_err(|err| {
            //error!("Failed to find email verification token: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Check if expired
    if verification_token.is_expired() {
        // Clean up expired token
        state
            .email_verification_repository
            .delete_token(token)
            .await
            .map_err(|err| {
                //error!("Failed to delete expired email verification token: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        return Err(StatusCode::GONE);
    }

    // Mark user as verified
    state
        .email_verification_repository
        .verify_user_email(verification_token.user_id)
        .await
        .map_err(|err| {
            //error!("Failed to verify user email: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Delete token (single-use)
    state
        .email_verification_repository
        .delete_token(token)
        .await
        .map_err(|err| {
            //error!("Failed to delete email verification token: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "message": "Email verified successfully!"
    })))
}

// Handler for "Forgot Password" - generates and emails reset token
//#[instrument(skip(state))]
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<Json<ForgotPasswordResponse>, StatusCode> {
    // Validate email format
    payload.validate().map_err(|_| StatusCode::BAD_REQUEST)?;

    // Look up user by email
    let user = state
        .user_repository
        .find_by_email(&payload.email)
        .await
        .map_err(|err| {
            //error!("Failed to find user by email: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // SECURITY: Always return success even if email doesn't exist
    // This prevents attackers from discovering which emails are registered
    if user.is_none() {
        return Ok(Json(ForgotPasswordResponse {
            message: "If that email exists, a password reset link has been sent.".to_string(),
        }));
    }

    let user = user.unwrap();

    // Generate reset token
    let reset_token = generate_verification_token();
    let expires_at = Utc::now() + Duration::hours(1); // 1 hour expiration

    // Save token to database
    state
        .password_reset_repository
        .create_token(user.id, &reset_token, expires_at)
        .await
        .map_err(|err| {
            //error!("Failed to create password reset token: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Send reset email
    state
        .email_service
        .send_password_reset_email(&user.email, &user.username, &reset_token)
        .await
        .map_err(|err| {
            //error!("Failed to send password reset email: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ForgotPasswordResponse {
        message: "If that email exists, a password reset link has been sent.".to_string(),
    }))
}

// Handler for actually resetting the password
pub async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<Json<ResetPasswordResponse>, StatusCode> {
    // Validate new password
    payload.validate().map_err(|err| {
        //error!("Failed to validate password reset request: {}", err);
        StatusCode::BAD_REQUEST
    })?;

    // Look up token
    let reset_token = state
        .password_reset_repository
        .find_by_token(&payload.token)
        .await
        .map_err(|err| {
            //r!("Failed to find password reset token: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Check expiration
    if reset_token.is_expired() {
        // Clean up expired token
        state
            .password_reset_repository
            .delete_token(&payload.token)
            .await
            .map_err(|err| {
                //error!("Failed to delete expired password reset token: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        return Err(StatusCode::GONE);
    }

    // Hash new password
    let new_password_hash = hash_password(&payload.new_password).map_err(|err| {
        //error!("Failed to hash new password: {}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Update user password
    state
        .user_repository
        .update_password(reset_token.user_id, &new_password_hash)
        .await
        .map_err(|err| {
            //error!("Failed to update user password: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Delete ALL reset tokens for this user (invalidate any other pending requests)
    state
        .password_reset_repository
        .delete_all_user_tokens(reset_token.user_id)
        .await
        .map_err(|err| {
            //error!("Failed to delete all user tokens: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ResetPasswordResponse {
        message: "Password has been reset successfully. You can now login with your new password."
            .to_string(),
    }))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>, StatusCode> {
    // Step 1: Find the refresh token in database
    let refresh_token = state
        .refresh_token_repository
        .find_by_token(&payload.refresh_token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Step 2: Check if token has expired
    if refresh_token.is_expired() {
        // Token is expired, delete it and reject
        let _ = state
            .refresh_token_repository
            .delete_token(&payload.refresh_token)
            .await;

        return Err(StatusCode::UNAUTHORIZED);
    }

    // Step 3: REUSE DETECTION - Check if token was already used
    if refresh_token.is_used {
        // SECURITY BREACH DETECTED!
        // Someone is trying to use an old token
        // This means the token was likely stolen

        // Nuclear option: Delete ALL user's refresh tokens
        // Force them to login again
        state
            .refresh_token_repository
            .delete_all_user_tokens(refresh_token.user_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Get user info for email
        let user = state
            .user_repository
            .find_by_id(refresh_token.user_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        // Send security alert email
        if let Err(e) = state
            .email_service
            .send_security_alert(&user.email, &user.username)
            .await
        {
            eprintln!("Failed to send security alert email: {}", e);
            // Don't fail the request if email fails
        }

        return Err(StatusCode::UNAUTHORIZED);
    }

    // Step 4: Mark the old token as used (consumed)
    state
        .refresh_token_repository
        .mark_token_as_used(&payload.refresh_token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Step 5: Generate NEW refresh token with rotation
    let new_refresh_token = generate_refresh_token();

    state
        .refresh_token_repository
        .create_token(refresh_token.user_id, &new_refresh_token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Step 6: Generate new access token
    let jwt_secret = std::env::var("JWT_SECRET").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let access_token = generate_token(&refresh_token.user_id, &jwt_secret).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Step 7: Return BOTH tokens
    Ok(Json(RefreshTokenResponse {
        access_token,
        refresh_token: new_refresh_token,
    }))
}

pub async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<LogoutRequest>,
) -> Result<Json<LogoutResponse>, StatusCode> {
    state
        .refresh_token_repository
        .delete_token(&payload.refresh_token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(LogoutResponse {
        message: "Logged out successfully".to_string(),
    }))
}
