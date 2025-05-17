mod api;
mod component;
mod core;
mod db;
mod page;

use api::Api;
use color_eyre::eyre;
use poem::{
    EndpointExt, Route, Server, endpoint::EmbeddedFilesEndpoint, listener::TcpListener,
    middleware::Tracing,
};
use poem_openapi::OpenApiService;
use rust_embed::Embed;
use sqlx::PgPool;

/// Struct to put our Configs into
#[derive(Clone)]
struct Config {
    database_url: String,
    log_path: String,
}

/// Static files hosted via webserver
#[derive(Embed)]
#[folder = "assets"]
struct Assets;

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    // Lets get pretty error reports
    color_eyre::install()?;

    // Enable Poem's logging
    tracing_subscriber::fmt::init();

    // Generate our configs to share between threads
    let config = Config {
        database_url: dotenvy::var("DATABASE_URL")?,
        log_path: dotenvy::var("LOG_PATH")?,
    };

    // Setup our OpenAPI Service
    let api_service = OpenApiService::new(Api, "Kyubey", "0.1.0").server("http://0.0.0.0:3000/api");
    let spec = api_service.spec_endpoint();
    let swagger = api_service.swagger_ui();

    // Connect to PostgreSQL
    let pool = PgPool::connect(&config.database_url).await?;

    // Route inbound traffic
    let app = Route::new()
        // Developer friendly locations
        .nest("/api", api_service)
        .nest("/assets", EmbeddedFilesEndpoint::<Assets>::new())
        .at("/spec", spec)
        .nest("/swagger", swagger)
        // User UI
        .nest("/", page::route())
        .nest("/component", component::route())
        // Global context to be shared
        .data(config)
        .data(pool)
        // Utilites being added to our services
        .with(Tracing);

    // Lets run our service
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await?;

    Ok(())
}
