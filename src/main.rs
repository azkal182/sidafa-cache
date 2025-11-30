mod cache;
mod config;
mod error;
mod routes;
mod youtube;

use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::http::Request;
use reqwest::Client;
use tower::ServiceBuilder;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::PeerIpKeyExtractor, GovernorLayer,
};
use tower_http::trace::TraceLayer;
use tracing::{info, info_span, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use cache::RedisCache;
use config::Config;
use routes::create_router;
use youtube::YouTubeService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file
    dotenvy::dotenv().ok();

    // Initialize logging
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new("sidafa_cache=debug,tower_http=debug,info")
        }))
        .with(fmt::layer().with_target(true).with_level(true))
        .init();

    info!("Starting Sidafa Cache Server...");

    // Load config
    let config = Config::from_env();
    info!(
        port = %config.port,
        host = %config.host,
        redis = %config.redis_url(),
        "Configuration loaded"
    );

    // Initialize Redis cache (TTL 1 hour = 3600 seconds)
    let cache = RedisCache::new(&config.redis_url(), 3600).await?;

    // Initialize HTTP client
    let http_client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    // Initialize YouTube service
    let youtube_service = YouTubeService::new(http_client, cache, config.clone());

    // Rate limiting configuration: 100 requests per minute per IP
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(60)
            .burst_size(100)
            .key_extractor(PeerIpKeyExtractor)
            .finish()
            .expect("Failed to build rate limiter config"),
    );

    let governor_limiter = governor_conf.limiter().clone();

    // Spawn background task to clean up rate limiter
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            governor_limiter.retain_recent();
        }
    });

    // Build router with middleware
    let app = create_router(youtube_service)
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(|request: &Request<_>| {
                            info_span!(
                                "http_request",
                                method = %request.method(),
                                uri = %request.uri(),
                            )
                        })
                        .on_response(
                            tower_http::trace::DefaultOnResponse::new().level(Level::INFO),
                        ),
                )
                .layer(GovernorLayer {
                    config: governor_conf,
                }),
        );

    // Parse server address
    let addr: SocketAddr = config.server_addr().parse()?;
    info!(address = %addr, "Server listening");

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    info!("========================================");
    info!("  Sidafa Cache Server is ready!");
    info!("  URL: http://{}", addr);
    info!("  Rate Limit: 100 requests/minute per IP");
    info!("========================================");

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;

    Ok(())
}
