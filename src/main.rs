use auth::{require_role, RequireRole, Role};
use axum::{
    middleware::{self, from_fn_with_state},
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

mod auth;
mod error;
mod handlers;
mod models;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:db/realestate.db".to_string());

    let pool = SqlitePool::connect(&database_url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let app = Router::new()
        // Public routes
        .route("/api/login", post(handlers::login))
        .route("/api/refresh", post(handlers::refresh_token))
        .route(
            "/api/password-reset",
            post(handlers::request_password_reset),
        )
        .route(
            "/api/password-reset/confirm",
            post(handlers::reset_password),
        )
        // Protected routes with role-based auth
        .route(
            "/api/admin/users",
            get(handlers::list_users).route_layer(middleware::from_fn_with_state(
                RequireRole(Role::Admin),
                require_role,
            )),
        )
        .route(
            "/api/properties/create",
            post(handlers::create_property).route_layer(middleware::from_fn_with_state(
                RequireRole(Role::Agent),
                require_role,
            )),
        )
        // User routes
        .route("/api/users", post(handlers::create_user))
        .route("/api/users/:id", get(handlers::get_user))
        .route("/api/users/by-role/:role", get(handlers::get_users_by_role))
        // Property routes
        .route("/api/properties", get(handlers::list_properties))
        .route("/api/properties/:id", get(handlers::get_property))
        // Message routes
        .route("/api/conversations", post(handlers::create_conversation))
        .route(
            "/api/conversations/:id/messages",
            post(handlers::send_message),
        )
        .route(
            "/api/conversations/:id/messages",
            get(handlers::get_messages),
        )
        .route(
            "/api/users/:id/conversations",
            get(handlers::get_user_conversations),
        )
        .with_state(pool)
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        );

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
