use axum::{
    routing::{get, post, delete},
    Router,
};
use std::env;

// Import from the library crate
use realworld_axum_api::{
    handlers::{register, login, health_check, current_user, verify_email, delete_user, reset_password, forgot_password, refresh_token, logout},
    state::AppState,
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let app_state = AppState::new(&database_url)
        .await
        .expect("Failed to connect to database");

    println!("Connected to database successfully!");

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/users", post(register))
        .route("/api/users/login", post(login))
        .route("/api/user", get(current_user))
        .route("/api/user", delete(delete_user))
        .route("/api/auth/verify-email", get(verify_email))
        .route("/api/auth/forgot-password", post(forgot_password))
        .route("/api/auth/reset-password", post(reset_password))
        .route("/api/auth/refresh-token", post(refresh_token))
        .route("/api/auth/logout", post(logout))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Server running on http://localhost:3000");
    println!("Available endpoints:");
    println!("  POST /api/users         - Register new user");
    println!("  POST /api/users/login   - Login existing user");
    println!("  GET  /api/user          - Get current user (requires auth)");
    println!("  DELETE /api/user       - Delete current user (requires auth)");
    println!("  GET  /api/auth/verify-email - Verify user email (requires auth)");
    println!("  POST /api/auth/forgot-password - Request password reset");
    println!("  POST /api/auth/reset-password - Reset user password");
    println!("  POST /api/auth/refresh-token - Refresh access token");
    println!("  POST /api/auth/logout   - Logout user");
    println!("  GET  /health            - Health check");

    axum::serve(listener, app).await.unwrap();
}