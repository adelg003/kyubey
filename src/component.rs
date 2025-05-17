use crate::{
    Config,
    core::{System, Task, log_read, search_systems_read, task_read},
};
use askama::Template;
use poem::{
    Route,
    error::InternalServerError,
    get, handler,
    http::StatusCode,
    web::{Data, Html, Query},
};
use serde::Deserialize;
use sqlx::{PgPool, Postgres, Transaction};

/// Router for all reactive UI Components
pub fn route() -> Route {
    Route::new()
        .at("/search_systems", get(search_systems_get))
        .at("/log", get(log_get))
}

/// Search for a system
#[derive(Template)]
#[template(path = "component/search_rows.html")]
pub struct SearchComponent {
    systems: Vec<System>,
    search_by: String,
    next_page: Option<u32>,
}

/// Data for System Search Web Component
pub async fn search_systems_component(
    tx: &mut Transaction<'_, Postgres>,
    search_by: &str,
    page: &u32,
) -> Result<SearchComponent, poem::Error> {
    // Search for anything that meets our criteria
    let systems: Vec<System> = search_systems_read(tx, search_by, page).await?;

    // More Systems on next page?
    let more_systems: Vec<System> = search_systems_read(tx, search_by, &(page + 1)).await?;

    let next_page: Option<u32> = match more_systems.is_empty() {
        true => None,
        false => Some(page + 1),
    };

    // Render the component
    Ok(SearchComponent {
        systems,
        search_by: search_by.to_string(),
        next_page,
    })
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

    // Render component
    let component: String = search_systems_component(&mut tx, &params.search_by, &params.page)
        .await?
        .render()
        .map_err(InternalServerError)?;

    Ok(Html(component))
}

/// Data for Log Web Component
#[derive(Template)]
#[template(path = "component/log.html")]
pub struct LogComponent {
    dag_id: String,
    run_id: String,
    task_id: String,
    attempt: u32,
    try_number: u32,
    log: String,
}

/// Web Component for showing logs
pub async fn log_component(
    config: &Config,
    dag_id: &str,
    run_id: &str,
    task_id: &str,
    attempt: &u32,
    try_number: &u32,
) -> Result<LogComponent, poem::Error> {
    // Log for a task attempt
    let log: String = log_read(config, dag_id, run_id, task_id, attempt).await?;

    Ok(LogComponent {
        dag_id: dag_id.to_string(),
        run_id: run_id.to_string(),
        task_id: task_id.to_string(),
        attempt: *attempt,
        try_number: *try_number,
        log,
    })
}

/// Paramiters to Pull a log
#[derive(Deserialize)]
struct LogParams {
    dag_id: String,
    run_id: String,
    task_id: String,
    attempt: u32,
}

/// Web Component to search for your system
#[handler]
async fn log_get(
    Data(pool): Data<&PgPool>,
    Data(config): Data<&Config>,
    Query(params): Query<LogParams>,
) -> Result<Html<String>, poem::Error> {
    // Start Transaction
    let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(InternalServerError)?;

    // Pull task details so we know how mmany runs their should be.
    let task: Task = task_read(&mut tx, &params.run_id, &params.task_id).await?;

    // Make sure we should have logs from a run
    let try_number: u32 = match task.try_number {
        Some(try_number) if try_number > 0 => Ok(try_number),
        _ => Err(poem::Error::from_string(
            "No logs yet",
            StatusCode::NOT_FOUND,
        )),
    }?;

    // Render component
    let component: String = log_component(
        config,
        &params.dag_id,
        &params.run_id,
        &params.task_id,
        &params.attempt,
        &try_number,
    )
    .await?
    .render()
    .map_err(InternalServerError)?;

    Ok(Html(component))
}
