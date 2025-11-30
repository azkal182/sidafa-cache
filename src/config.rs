use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub host: String,
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_password: Option<String>,
    pub youtube_api_key: String,
    pub channel_id: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("PORT must be a number"),
            host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            redis_host: env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            redis_port: env::var("REDIS_PORT")
                .unwrap_or_else(|_| "6379".to_string())
                .parse()
                .expect("REDIS_PORT must be a number"),
            redis_password: env::var("REDIS_PASSWORD").ok().filter(|s| !s.is_empty()),
            youtube_api_key: env::var("YOUTUBE_API_KEY").unwrap_or_else(|_| "key".to_string()),
            channel_id: env::var("CHANNEL_ID").unwrap_or_else(|_| "channelId".to_string()),
        }
    }

    pub fn redis_url(&self) -> String {
        match &self.redis_password {
            Some(password) => format!("redis://:{}@{}:{}", password, self.redis_host, self.redis_port),
            None => format!("redis://{}:{}", self.redis_host, self.redis_port),
        }
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
