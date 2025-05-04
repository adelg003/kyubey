use crate::core::{SearchSystems, search_systems_read};
use askama::Template;
use poem::{
    Route,
    error::InternalServerError,
    get, handler,
    web::{Data, Html},
};
use sqlx::{PgPool, Postgres, Transaction};

/// Router for all UI Pages
pub fn route() -> Route {
    Route::new().at("/", get(index))
}

/// Search for a system
#[derive(Template)]
#[template(path = "page/index.html")]
struct Index {
    systems: SearchSystems,
}

/// Webpage to search for your system
#[handler]
async fn index(Data(pool): Data<&PgPool>) -> Result<Html<String>, poem::Error> {
    // Start Transaction
    let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(InternalServerError)?;

    // Pull the top of the list to pre-render the page.
    let systems: SearchSystems = search_systems_read(&mut tx, "", &0).await?;

    // Render the Index Page
    let index: String = Index { systems }.render().map_err(InternalServerError)?;

    Ok(Html(index))
}
