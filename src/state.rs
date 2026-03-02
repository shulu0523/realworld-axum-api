use std::sync::Arc;
use crate::repositories::{
    UserRepository,
    UserRepositoryTrait,
    EmailVerificationRepository,
    EmailVerificationRepositoryTrait,
    PasswordResetRepository,
    PasswordResetRepositoryTrait,
    RefreshTokenRepository,
    RefreshTokenRepositoryTrait,
};
use axum::extract::FromRef;
use sqlx::PgPool;
use crate::services::EmailService;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub db: PgPool,
    pub user_repository: Arc<dyn UserRepositoryTrait>,
    pub email_verification_repository: Arc<dyn EmailVerificationRepositoryTrait>,  // NEW
    pub email_service: Arc<EmailService>,  // NEW
    pub password_reset_repository: Arc<dyn PasswordResetRepositoryTrait>,  // NEW
    pub refresh_token_repository: Arc<dyn RefreshTokenRepositoryTrait>,  // NEW
}

impl AppState {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let db = PgPool::connect(database_url).await?;
        
        sqlx::migrate!("./migrations").run(&db).await?;

        let user_repository: Arc<dyn UserRepositoryTrait> =
            Arc::new(UserRepository::new(db.clone()));

        let email_verification_repository: Arc<dyn EmailVerificationRepositoryTrait> =
            Arc::new(EmailVerificationRepository::new(db.clone()));

        let email_service = Arc::new(
            EmailService::new().expect("Failed to initialize email service")
        );

        let password_reset_repository: Arc<dyn PasswordResetRepositoryTrait> =
            Arc::new(PasswordResetRepository::new(db.clone()));

        let refresh_token_repository: Arc<dyn RefreshTokenRepositoryTrait> =
            Arc::new(RefreshTokenRepository::new(db.clone()));

        Ok(Self {
            db,
            user_repository,
            email_verification_repository,
            password_reset_repository,
            email_service,
            refresh_token_repository,
        })
    }
}