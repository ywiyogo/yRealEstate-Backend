// HTTP handlers for auth routes (login, password reset, etc.)

use crate::{
    auth::{self, AuthResponse, Claims},
    error::ApiError,
    models::{User, UserRole},
};
use axum::{extract::State, Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetConfirm {
    pub token: String,
    pub new_password: String,
}

pub async fn request_password_reset(
    State(pool): State<SqlitePool>,
    Json(request): Json<PasswordResetRequest>,
) -> Result<Json<()>, ApiError> {
    // Generate a reset token
    let reset_token = Uuid::new_v4().to_string();
    // Format datetime as string that SQLite can handle
    let expires_at = Utc::now()
        .checked_add_signed(chrono::Duration::hours(1))
        .unwrap()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    sqlx::query!(
        r#"
        UPDATE users 
        SET reset_token = ?, reset_token_expires = ?
        WHERE email = ?
        "#,
        reset_token,
        expires_at,
        request.email
    )
    .execute(&pool)
    .await
    .map_err(ApiError::DatabaseError)?;

    // In a real application, you would send this token via email
    // For now, we'll just return success
    Ok(Json(()))
}

pub async fn reset_password(
    State(pool): State<SqlitePool>,
    Json(reset): Json<PasswordResetConfirm>,
) -> Result<Json<()>, ApiError> {
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Find user with valid reset token
    let user = sqlx::query!(
        r#"
        SELECT id 
        FROM users 
        WHERE reset_token = ? AND reset_token_expires > ?
        "#,
        reset.token,
        now
    )
    .fetch_optional(&pool)
    .await
    .map_err(ApiError::DatabaseError)?
    .ok_or_else(|| ApiError::ValidationError("Invalid or expired reset token".to_string()))?;

    // Hash new password and update user
    let password_hash = hash(reset.new_password.as_bytes(), DEFAULT_COST)
        .map_err(|_| ApiError::ValidationError("Password hashing failed".to_string()))?;

    sqlx::query!(
        r#"
        UPDATE users 
        SET password_hash = ?, reset_token = NULL, reset_token_expires = NULL
        WHERE id = ?
        "#,
        password_hash,
        user.id
    )
    .execute(&pool)
    .await
    .map_err(ApiError::DatabaseError)?;

    Ok(Json(()))
}

#[derive(Debug, Deserialize)]
pub struct RefreshToken {
    pub token: String,
}

pub async fn refresh_token(
    State(pool): State<SqlitePool>,
    Json(refresh): Json<RefreshToken>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Validate refresh token
    let claims = decode::<Claims>(
        &refresh.token,
        &DecodingKey::from_secret(auth::JWT_SECRET),
        &Validation::default(),
    )
    .map_err(|_| ApiError::AuthenticationError("Invalid refresh token".to_string()))?
    .claims;

    // Get user
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT 
            id,
            email,
            password_hash,
            full_name,
            phone as "phone?",
            role as "role: UserRole",
            verified,
            profile_image_url as "profile_image_url?",
            created_at as "created_at?: String",
            updated_at as "updated_at?: String",
            reset_token as "reset_token?",
            reset_token_expires as "reset_token_expires?: String"
        FROM users 
        WHERE id = ?
        "#,
        claims.sub
    )
    .fetch_optional(&pool)
    .await
    .map_err(ApiError::DatabaseError)?
    .ok_or_else(|| ApiError::AuthenticationError("User not found".to_string()))?;

    // Create new tokens
    let access_token = auth::create_token(&user)?;
    let refresh_token = auth::create_refresh_token(&user)?;

    Ok(Json(AuthResponse {
        token: access_token,
        refresh_token,
        user,
    }))
}
