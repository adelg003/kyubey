mod search;
mod util;

use color_eyre::eyre;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    // Lets get pretty error reports
    color_eyre::install()?;

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Connect to PostgreSQL
    let pool = PgPool::connect(&dotenvy::var("DATABASE_URL")?).await?;
    let state = AppState { pool };

    //Build top level Router
    let (router, swagger) = OpenApiRouter::new()
        // Add System endpoints
        .merge(search::router())
        
        .with_state(state)
        .split_for_parts();

    // Add Swagger page
    let router = router
        .merge(SwaggerUi::new("/swagger").url("/openapi.json", swagger))
        // Add tracing to service
        .layer(TraceLayer::new_for_http());

    // Where should the webserver listen
    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    // Host the rounts in out webserver
    axum::serve(listener, router).await?;

    Ok(())
}
