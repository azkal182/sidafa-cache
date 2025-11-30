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
use crate::wordpress::WordPressService;
use crate::youtube::YouTubeService;

#[derive(Clone)]
pub struct AppServices {
    pub youtube: YouTubeService,
    pub wordpress: WordPressService,
}

pub type AppState = Arc<AppServices>;

pub fn create_router(youtube_service: YouTubeService, wordpress_service: WordPressService) -> Router {
    let state: AppState = Arc::new(AppServices {
        youtube: youtube_service,
        wordpress: wordpress_service,
    });

    Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .nest("/api", api_routes())
        .with_state(state)
}

fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/youtube/:resource", get(get_youtube_data))
        .route("/youtube", get(list_youtube_resources))
        .route("/wp/*path", get(get_wordpress_data))
        .route("/wp", get(get_wordpress_root))
}

async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "message": "Sidafa Cache Server is running"
    }))
}

async fn list_youtube_resources() -> impl IntoResponse {
    Json(json!({
        "message": "YouTube API Cache Service",
        "available_resources": YouTubeService::allowed_resources(),
        "usage": "/api/youtube/:resource?param1=value1&param2=value2"
    }))
}

async fn get_youtube_data(
    State(services): State<AppState>,
    Path(resource): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    info!(
        resource = %resource,
        params = ?params,
        "YouTube API request received"
    );

    let data = services.youtube.get_cached_data(&resource, params).await?;

    Ok((StatusCode::OK, Json(data)))
}

async fn get_wordpress_root(
    State(services): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    info!(params = ?params, "WordPress API root request");

    let data = services.wordpress.get_cached_data("", params).await?;

    Ok((StatusCode::OK, Json(data)))
}

async fn get_wordpress_data(
    State(services): State<AppState>,
    Path(path): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    info!(
        path = %path,
        params = ?params,
        "WordPress API request received"
    );

    let data = services.wordpress.get_cached_data(&path, params).await?;

    Ok((StatusCode::OK, Json(data)))
}
