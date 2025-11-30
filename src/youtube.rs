use std::collections::HashMap;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, info, warn};

use crate::cache::RedisCache;
use crate::config::Config;
use crate::error::AppError;

const ALLOWED_RESOURCES: &[&str] = &["search", "videos", "channels", "playlists", "playlistItems"];
const YOUTUBE_API_BASE: &str = "https://www.googleapis.com/youtube/v3";

#[derive(Clone)]
pub struct YouTubeService {
    client: Client,
    cache: RedisCache,
    config: Config,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheKey {
    endpoint: String,
    params: HashMap<String, String>,
}

impl YouTubeService {
    pub fn new(client: Client, cache: RedisCache, config: Config) -> Self {
        Self {
            client,
            cache,
            config,
        }
    }

    pub async fn get_cached_data(
        &self,
        resource: &str,
        params: HashMap<String, String>,
    ) -> Result<Value, AppError> {
        // Validate resource
        if !ALLOWED_RESOURCES.contains(&resource) {
            let allowed = ALLOWED_RESOURCES.join(", ");
            warn!(resource = %resource, "Invalid resource requested");
            return Err(AppError::Validation(format!(
                "Invalid resource: \"{}\". Allowed: {}",
                resource, allowed
            )));
        }

        let endpoint = format!("/{}", resource);
        let cache_key = CacheKey {
            endpoint: endpoint.clone(),
            params: params.clone(),
        };
        let cache_key_str = serde_json::to_string(&cache_key)
            .map_err(|e| AppError::Internal(e.into()))?;

        // Try to fetch from cache
        if let Some(cached_data) = self.cache.get::<Value>(&cache_key_str).await? {
            info!(resource = %resource, "Returning cached data from Redis");
            return Ok(cached_data);
        }

        // Fetch from YouTube API
        info!(resource = %resource, "Fetching data from YouTube API");
        let data = self.fetch_from_api(&endpoint, &params).await?;

        // Cache the data (TTL 1 hour = 3600 seconds)
        self.cache.set(&cache_key_str, &data).await?;

        Ok(data)
    }

    async fn fetch_from_api(
        &self,
        endpoint: &str,
        params: &HashMap<String, String>,
    ) -> Result<Value, AppError> {
        let url = format!("{}{}", YOUTUBE_API_BASE, endpoint);

        let mut query_params = params.clone();
        query_params.insert("key".to_string(), self.config.youtube_api_key.clone());
        query_params.insert("channelId".to_string(), self.config.channel_id.clone());

        debug!(url = %url, params = ?query_params, "Making YouTube API request");

        let response = self
            .client
            .get(&url)
            .query(&query_params)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            warn!(status = %status, body = %error_body, "YouTube API error");
            return Err(AppError::YouTubeApi(format!(
                "YouTube API returned status {}: {}",
                status, error_body
            )));
        }

        let data: Value = response.json().await?;
        debug!("YouTube API response received successfully");
        Ok(data)
    }

    pub fn allowed_resources() -> &'static [&'static str] {
        ALLOWED_RESOURCES
    }
}
