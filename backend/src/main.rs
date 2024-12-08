mod db;
mod errors;
mod models;
mod rest_handlers;
mod static_handlers;
mod ws_handlers;

use crate::ws_handlers::Connections;
use axum::{
    routing::{get, post},
    Router,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let pool = db::init_db().await?;
    let connections: Connections = Arc::new(RwLock::new(HashMap::new()));

    // Regular API routes with just pool
    let api_routes = Router::new()
        .route("/api/create-group", post(rest_handlers::create_group))
        .route("/api/group", get(rest_handlers::get_group))
        .route("/api/user-groups", get(rest_handlers::get_user_groups))
        .with_state(pool.clone());

    // Routes needing websocket connections
    let ws_routes = Router::new()
        .route("/ws", get(ws_handlers::message_handler))
        .route("/api/generate-pairs", post(ws_handlers::generate_pairs))
        .route("/api/join-group", post(ws_handlers::join_group))
        .with_state((pool, connections));

    let app = api_routes
        .merge(ws_routes)
        .fallback(static_handlers::handler)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Server running on port 8080");

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
