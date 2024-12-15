use axum::{
    routing::{get, post},
    Router};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

mod handlers;
mod models;
mod error;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:db/realestate.db".to_string());
    
    let pool = SqlitePool::connect(&database_url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let app = Router::new()
        // User routes
        .route("/api/users", post(handlers::create_user))
        .route("/api/users/:id", get(handlers::get_user))
        .route("/api/users/by-role/:role", get(handlers::get_users_by_role))
        // Property routes
        .route("/api/properties", get(handlers::list_properties))
        .route("/api/properties", post(handlers::create_property))
        .route("/api/properties/:id", get(handlers::get_property))
        // Message routes
        .route("/api/conversations", post(handlers::create_conversation))
        .route("/api/conversations/:id/messages", post(handlers::send_message))
        .route("/api/conversations/:id/messages", get(handlers::get_messages))
        .route("/api/users/:id/conversations", get(handlers::get_user_conversations))
        .with_state(pool)
        .layer(CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}