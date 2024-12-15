// Core authentication logic (JWT handling, middleware)

use axum::{
    async_trait,
    body::Body,
    extract::FromRequestParts,
    extract::State,
    http::request::Parts,
    middleware::Next,
    response::Response, // Use axum::response::Response instead of http::Response
};
use axum_extra::headers::{authorization::Bearer, Authorization};
use axum_extra::TypedHeader;

use crate::error::ApiError;
use crate::models::User;
use http::Request;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

pub const JWT_SECRET: &[u8] = b"your-secret-key"; // In production, use env variable

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64, // user id
    pub exp: i64, // expiration timestamp
    pub iat: i64, // issued at timestamp
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub user: User,
}

pub fn create_token(user: &User) -> Result<String, ApiError> {
    let now = OffsetDateTime::now_utc();
    let expiry = now + Duration::hours(24);

    let claims = Claims {
        sub: user.id.unwrap(),
        exp: expiry.unix_timestamp(),
        iat: now.unix_timestamp(),
        role: user.role.to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
    .map_err(|_| ApiError::ValidationError("Token creation failed".to_string()))
}

#[derive(Debug)]
pub struct AuthUser {
    pub user_id: i64,
    pub role: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token using axum's typed headers
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, _state)
                .await
                .map_err(|_| {
                    ApiError::AuthenticationError("Invalid authorization header".to_string())
                })?;

        // Decode and validate the token
        let token_data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(JWT_SECRET),
            &Validation::default(),
        )
        .map_err(|_| ApiError::AuthenticationError("Invalid token".to_string()))?;

        Ok(AuthUser {
            user_id: token_data.claims.sub,
            role: token_data.claims.role,
        })
    }
}

// Add to existing auth.rs
#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    Admin,
    Agent,
    User,
}

impl Role {
    pub fn from_str(role: &str) -> Option<Self> {
        match role.to_lowercase().as_str() {
            "admin" => Some(Role::Admin),
            "agent" => Some(Role::Agent),
            "user" => Some(Role::User),
            _ => None,
        }
    }
}

// Create a middleware state struct
#[derive(Clone)]
pub struct RequireRole(pub Role);

pub async fn require_role(
    auth_user: AuthUser,
    State(role): State<RequireRole>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let user_role = Role::from_str(&auth_user.role)
        .ok_or_else(|| ApiError::AuthorizationError("Invalid role".to_string()))?;

    match (user_role, &role.0) {
        (Role::Admin, _) => Ok(next.run(request).await),
        (Role::Agent, Role::User) | (Role::Agent, Role::Agent) => Ok(next.run(request).await),
        (Role::User, Role::User) => Ok(next.run(request).await),
        _ => Err(ApiError::AuthorizationError(
            "Insufficient permissions".to_string(),
        )),
    }
    // Response type in Axum has a default body type, so we don't need to specify it
}

// Add refresh token functionality
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshToken {
    pub token: String,
}

pub fn create_refresh_token(user: &User) -> Result<String, ApiError> {
    let now = OffsetDateTime::now_utc();
    let expiry = now + Duration::days(30); // Refresh tokens last longer

    let claims = Claims {
        sub: user.id.unwrap(),
        exp: expiry.unix_timestamp(),
        iat: now.unix_timestamp(),
        role: user.role.to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
    .map_err(|_| ApiError::ValidationError("Refresh token creation failed".to_string()))
}
