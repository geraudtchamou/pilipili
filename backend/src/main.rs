use actix_web::{middleware, web, App, HttpServer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod handlers;
mod middleware as app_middleware;
mod models;
mod services;
mod utils;

use crate::config::Config;
use crate::db::establish_connection_pool;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "fintech_backend=info,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");

    // Establish database connection pool
    let db_pool = establish_connection_pool(&config.database_url)
        .await
        .expect("Failed to create database pool");

    // Establish Redis connection
    let redis_client = redis::Client::open(config.redis_url.clone())
        .expect("Failed to create Redis client");

    tracing::info!("Starting server on {}:{}", config.host, config.port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(redis_client.clone()))
            .app_data(web::Data::new(config.clone()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(handlers::configure_routes)
    })
    .bind((config.host.as_str(), config.port))?
    .run()
    .await
}
