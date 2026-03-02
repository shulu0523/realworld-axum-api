use async_trait::async_trait;
use chrono::{Utc, Duration};
use sqlx::PgPool;
use uuid::Uuid;

use super::traits::RefreshTokenRepositoryTrait;
use crate::models::RefreshToken;

#[derive(Clone)]
pub struct RefreshTokenRepository {
    db: PgPool,
}

impl RefreshTokenRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl RefreshTokenRepositoryTrait for RefreshTokenRepository {
    async fn create_token(
        &self,
        user_id: Uuid,
        token: &str,
    ) -> Result<RefreshToken, sqlx::Error> {
        // Refresh token expires in 7 days
        let expires_at = Utc::now() + Duration::days(7);
        
        let refresh_token = sqlx::query_as::<_, RefreshToken>(
            r#"
            INSERT INTO refresh_tokens (user_id, token, expires_at, is_used)
            VALUES ($1, $2, $3, false)
            RETURNING id, user_id, token, expires_at, created_at, used_at, is_used
            "#,
        )
        .bind(user_id)
        .bind(token)
        .bind(expires_at)
        .fetch_one(&self.db)
        .await?;

        Ok(refresh_token)
    }

    async fn find_by_token(&self, token: &str) -> Result<Option<RefreshToken>, sqlx::Error> {
        let refresh_token = sqlx::query_as::<_, RefreshToken>(
            r#"
            SELECT id, user_id, token, expires_at, created_at, used_at, is_used
            FROM refresh_tokens
            WHERE token = $1
            "#,
        )
        .bind(token)
        .fetch_optional(&self.db)
        .await?;

        Ok(refresh_token)
    }

    async fn delete_token(&self, token: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM refresh_tokens
            WHERE token = $1
            "#,
        )
        .bind(token)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn delete_all_user_tokens(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM refresh_tokens
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn mark_token_as_used(&self, token: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET is_used = true, used_at = $2
            WHERE token = $1
            "#,
        )
        .bind(token)
        .bind(Utc::now())
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<RefreshToken>, sqlx::Error> {
        let refresh_token = sqlx::query_as::<_, RefreshToken>(
            r#"
            SELECT id, user_id, token, expires_at, created_at, used_at, is_used
            FROM refresh_tokens
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?;

        Ok(refresh_token)
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<RefreshToken>, sqlx::Error> {
        let refresh_tokens = sqlx::query_as::<_, RefreshToken>(
            r#"
            SELECT id, user_id, token, expires_at, created_at, used_at, is_used
            FROM refresh_tokens
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        Ok(refresh_tokens)
    }
}
