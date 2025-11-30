use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::json;
use tracing::info;

use crate::error::AppError;
use crate::youtube::YouTubeService;

pub type AppState = Arc<YouTubeService>;

pub fn create_router(youtube_service: YouTubeService) -> Router {
    let state: AppState = Arc::new(youtube_service);

    Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .nest("/api", api_routes())
        .with_state(state)
}

fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/youtube/:resource", get(get_youtube_data))
        .route("/youtube", get(list_resources))
}

async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "message": "Sidafa Cache Server is running"
    }))
}

async fn list_resources() -> impl IntoResponse {
    Json(json!({
        "message": "YouTube API Cache Service",
        "available_resources": YouTubeService::allowed_resources(),
        "usage": "/api/youtube/:resource?param1=value1&param2=value2"
    }))
}

async fn get_youtube_data(
    State(service): State<AppState>,
    Path(resource): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    info!(
        resource = %resource,
        params = ?params,
        "YouTube API request received"
    );

    let data = service.get_cached_data(&resource, params).await?;

    Ok((StatusCode::OK, Json(data)))
}
