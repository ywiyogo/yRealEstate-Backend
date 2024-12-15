use crate::auth::{create_refresh_token, create_token, AuthResponse, AuthUser};
use crate::error::ApiError;
use crate::models::{LoginCredentials, NewUser, User, UserRole};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use sqlx::SqlitePool;
pub async fn create_user(
    State(pool): State<SqlitePool>,
    Json(new_user): Json<NewUser>,
) -> Result<Json<User>, StatusCode> {
    let password_hash = hash(new_user.password.as_bytes(), DEFAULT_COST)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (email, password_hash, full_name, phone, role, verified)
        VALUES (?, ?, ?, ?, ?, false)
        RETURNING id, email, password_hash, full_name, phone, role as "role: UserRole", 
                  verified, profile_image_url, created_at, updated_at,
            reset_token as "reset_token?",
            reset_token_expires as "reset_token_expires?: String"
        "#,
        new_user.email,
        password_hash,
        new_user.full_name,
        new_user.phone,
        new_user.role,
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(user))
}

pub async fn get_user(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<User>, StatusCode> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, email, password_hash, full_name, phone, 
               role as "role: UserRole", verified, profile_image_url, 
               created_at, updated_at,
               reset_token as "reset_token?",
               datetime(reset_token_expires) as "reset_token_expires?: String"
        FROM users
        WHERE id = ?
        "#,
        id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(user))
}

pub async fn get_users_by_role(
    State(pool): State<SqlitePool>,
    Path(role): Path<UserRole>,
) -> Result<Json<Vec<User>>, StatusCode> {
    let users = sqlx::query_as!(
        User,
        r#"
        SELECT id, email, password_hash, full_name, phone, 
               role as "role: UserRole", verified, profile_image_url, 
               created_at, updated_at,
               reset_token as "reset_token?",
               reset_token_expires as "reset_token_expires?: String"
        FROM users
        WHERE role = ?
        "#,
        role
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(users))
}

pub async fn list_users(State(pool): State<SqlitePool>) -> Result<Json<Vec<User>>, ApiError> {
    let users = sqlx::query_as!(
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
            datetime(created_at) as "created_at?: String",
            datetime(updated_at) as "updated_at?: String",
            reset_token as "reset_token?",
            datetime(reset_token_expires) as "reset_token_expires?: String"
        FROM users
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(ApiError::DatabaseError)?;

    Ok(Json(users))
}

pub async fn login(
    State(pool): State<SqlitePool>,
    Json(credentials): Json<LoginCredentials>,
) -> Result<Json<AuthResponse>, ApiError> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, email, password_hash, full_name, phone, 
               role as "role: UserRole", verified, profile_image_url, 
               created_at, updated_at, reset_token as "reset_token?",
            reset_token_expires as "reset_token_expires?: String"
        FROM users
        WHERE email = ?
        "#,
        credentials.email
    )
    .fetch_optional(&pool)
    .await
    .map_err(ApiError::DatabaseError)?
    .ok_or(ApiError::AuthenticationError(
        "Invalid credentials".to_string(),
    ))?;

    if !verify(credentials.password.as_bytes(), &user.password_hash)
        .map_err(|_| ApiError::AuthenticationError("Invalid credentials".to_string()))?
    {
        return Err(ApiError::AuthenticationError(
            "Invalid credentials".to_string(),
        ));
    }

    let token = create_token(&user)?;
    let refresh_token = create_refresh_token(&user)?;

    Ok(Json(AuthResponse {
        token,
        refresh_token,
        user,
    }))
}

// Example of a protected route
pub async fn get_profile(
    auth_user: AuthUser,
    State(pool): State<SqlitePool>,
) -> Result<Json<User>, ApiError> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, email, password_hash, full_name, phone, 
               role as "role: UserRole", verified, profile_image_url, 
               created_at, updated_at, reset_token as "reset_token?",
            reset_token_expires as "reset_token_expires?: String"
        FROM users
        WHERE id = ?
        "#,
        auth_user.user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(ApiError::DatabaseError)?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(user))
}
