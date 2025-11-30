use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug, info};

use crate::error::AppError;

#[derive(Clone)]
pub struct RedisCache {
    conn: ConnectionManager,
    default_ttl: u64,
}

impl RedisCache {
    pub async fn new(redis_url: &str, default_ttl: u64) -> Result<Self, AppError> {
        info!(url = %redis_url, "Connecting to Redis...");
        let client = redis::Client::open(redis_url)?;
        let conn = ConnectionManager::new(client).await?;
        info!("Redis connection established");
        Ok(Self { conn, default_ttl })
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, AppError> {
        let mut conn = self.conn.clone();
        let cached: Option<String> = conn.get(key).await?;

        match cached {
            Some(data) => {
                debug!(key = %key, "Cache HIT");
                let value: T = serde_json::from_str(&data)
                    .map_err(|e| AppError::Internal(e.into()))?;
                Ok(Some(value))
            }
            None => {
                debug!(key = %key, "Cache MISS");
                Ok(None)
            }
        }
    }

    pub async fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<(), AppError> {
        let mut conn = self.conn.clone();
        let serialized = serde_json::to_string(value)
            .map_err(|e| AppError::Internal(e.into()))?;

        conn.set_ex::<_, _, ()>(key, serialized, self.default_ttl).await?;
        debug!(key = %key, ttl = %self.default_ttl, "Data cached");
        Ok(())
    }
}
