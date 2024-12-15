use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub enum ApiError {
    DatabaseError(sqlx::Error),
    NotFound,
    ValidationError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::DatabaseError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
            }
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Not found").into_response(),
            ApiError::ValidationError(msg) => {
                (StatusCode::BAD_REQUEST, msg).into_response()
            }
        }
    }
}

// Helper method to convert SQLx errors into our ApiError
impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ApiError::NotFound,
            _ => ApiError::DatabaseError(err)
        }
    }
}
