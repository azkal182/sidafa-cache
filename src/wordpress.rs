use std::collections::HashMap;

use reqwest::Client;
use serde_json::Value;
use tracing::{debug, info, warn};

use crate::cache::RedisCache;
use crate::error::AppError;

const WORDPRESS_API_BASE: &str = "https://amtsilatipusat.net/wp-json/wp/v2";

#[derive(Clone)]
pub struct WordPressService {
    client: Client,
    cache: RedisCache,
}

impl WordPressService {
    pub fn new(client: Client, cache: RedisCache) -> Self {
        Self { client, cache }
    }

    pub async fn get_cached_data(
        &self,
        path: &str,
        params: HashMap<String, String>,
    ) -> Result<Value, AppError> {
        let cache_key = format!(
            "wp:{}:{}",
            path,
            serde_json::to_string(&params).unwrap_or_default()
        );

        // Try to fetch from cache
        if let Some(cached_data) = self.cache.get::<Value>(&cache_key).await? {
            info!(path = %path, "Returning cached WordPress data");
            return Ok(cached_data);
        }

        // Fetch from WordPress API
        info!(path = %path, "Fetching data from WordPress API");
        let data = self.fetch_from_api(path, &params).await?;

        // Cache the data
        self.cache.set(&cache_key, &data).await?;

        Ok(data)
    }

    async fn fetch_from_api(
        &self,
        path: &str,
        params: &HashMap<String, String>,
    ) -> Result<Value, AppError> {
        let url = if path.is_empty() {
            WORDPRESS_API_BASE.to_string()
        } else {
            format!("{}/{}", WORDPRESS_API_BASE, path.trim_start_matches('/'))
        };

        debug!(url = %url, params = ?params, "Making WordPress API request");

        let response = self
            .client
            .get(&url)
            .query(params)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            warn!(status = %status, body = %error_body, "WordPress API error");
            return Err(AppError::WordPressApi(format!(
                "WordPress API returned status {}: {}",
                status, error_body
            )));
        }

        let data: Value = response.json().await?;
        debug!("WordPress API response received successfully");
        Ok(data)
    }
}
