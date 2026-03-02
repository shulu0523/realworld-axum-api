use axum::{
    routing::{get, post, delete},
    Router,
};
use std::env;
use tracing_subscriber::EnvFilter;

// Import from the library crate
use realworld_axum_api::{
    handlers::{register, login, health_check, current_user, verify_email, delete_user, reset_password, forgot_password, refresh_token, logout, get_sessions, terminate_session, export_user_data, delete_account},
    state::AppState,
    middleware::rate_limit::rate_limit_middleware,
    auth::role::RequireAdmin,
};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    
    dotenvy::dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let app_state = AppState::new(&database_url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("Connected to database successfully!");

    // Admin-only route
    async fn admin_only(RequireAdmin(_): RequireAdmin) -> axum::Json<serde_json::Value> {
        axum::Json(serde_json::json!({
            "message": "Welcome, admin!"
        }))
    }

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/users", post(register))
        .route("/api/users/login", post(login).layer(axum::middleware::from_fn(rate_limit_middleware)))
        .route("/api/user", get(current_user))
        .route("/api/user", delete(delete_user))
        .route("/api/auth/verify-email", get(verify_email))
        .route("/api/auth/forgot-password", post(forgot_password))
        .route("/api/auth/reset-password", post(reset_password))
        .route("/api/auth/refresh-token", post(refresh_token))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/sessions", get(get_sessions))
        .route("/api/auth/sessions/terminate", post(terminate_session))
        .route("/api/auth/export-data", get(export_user_data))
        .route("/api/auth/delete-account", post(delete_account))
        .route("/api/admin", get(admin_only))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::info!("Server running on http://localhost:3000");
    tracing::info!("Available endpoints:");
    tracing::info!("  POST /api/users         - Register new user");
    tracing::info!("  POST /api/users/login   - Login existing user");
    tracing::info!("  GET  /api/user          - Get current user (requires auth)");
    tracing::info!("  DELETE /api/user       - Delete current user (requires auth)");
    tracing::info!("  GET  /api/auth/verify-email - Verify user email (requires auth)");
    tracing::info!("  POST /api/auth/forgot-password - Request password reset");
    tracing::info!("  POST /api/auth/reset-password - Reset user password");
    tracing::info!("  POST /api/auth/refresh-token - Refresh access token");
    tracing::info!("  POST /api/auth/logout   - Logout user");
    tracing::info!("  GET  /api/auth/sessions - Get user sessions (requires auth)");
    tracing::info!("  POST /api/auth/sessions/terminate - Terminate session (requires auth)");
    tracing::info!("  GET  /api/auth/export-data - Export user data (requires auth)");
    tracing::info!("  POST /api/auth/delete-account - Delete user account (requires auth)");
    tracing::info!("  GET  /api/admin         - Admin-only route (requires admin role)");
    tracing::info!("  GET  /health            - Health check");

    axum::serve(listener, app).await.unwrap();
}