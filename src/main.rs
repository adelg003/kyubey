//TODO mod systems;

use axum::{Router, routing::get};
use color_eyre::eyre;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    // Lets get pretty error reports
    color_eyre::install()?;

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    //TODO Build a placehodler Route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .layer(TraceLayer::new_for_http());

    // Where should the webserver listen
    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    // Host the rounts in out webserver
    axum::serve(listener, app).await?;

    Ok(())
}
