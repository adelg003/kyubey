use crate::core::{SearchSystems, search_systems_read};
use askama::Template;
use poem::{
    Route,
    error::InternalServerError,
    get, handler,
    web::{Data, Html, Query},
};
use serde::Deserialize;
use sqlx::{PgPool, Postgres, Transaction};

/// Router for all UI Components
pub fn route() -> Route {
    Route::new().at("/search_systems", get(search_systems_get))
}

/// Paramiters to search by
#[derive(Deserialize)]
struct SearchParams {
    search_by: String,
    page: u32,
}

/// Web Component to search for your system
#[handler]
async fn search_systems_get(
    Data(pool): Data<&PgPool>,
    Query(params): Query<SearchParams>,
) -> Result<Html<String>, poem::Error> {
    // Start Transaction
    let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(InternalServerError)?;

    // Search for anything that meets our criteria
    let systems: SearchSystems =
        search_systems_read(&mut tx, &params.search_by, &params.page).await?;

    // Render the component
    let component: String = systems.render().map_err(InternalServerError)?;

    Ok(Html(component))
}
