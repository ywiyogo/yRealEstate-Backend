use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
};
use sqlx::SqlitePool;
use bcrypt::{hash, DEFAULT_COST};
use crate::models::{User, UserRole, NewUser};

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
                  verified, profile_image_url, created_at, updated_at
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
               created_at, updated_at
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
               created_at, updated_at
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