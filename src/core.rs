use crate::db::search_systems_select;
use askama::Template;
use chrono::NaiveDateTime;
use poem::error::InternalServerError;
use poem_openapi::Object;
use sqlx::{Postgres, Transaction};

/// A single system
#[derive(Object)]
pub struct System {
    pub client_name: String,
    pub client_id: String,
    pub system_name: String,
    pub system_id: String,
    pub team_name: String,
    pub team_id: String,
    pub latest_run: NaiveDateTime,
    pub number_of_dag_runs: u64,
}

/// Search for a system
#[derive(Object, Template)]
#[template(path = "component/search_systems.html")]
pub struct SearchSystems {
    systems: Vec<System>,
    search_by: String,
    page: u32,
    next_page: Option<u32>,
}

/// How much do we want to paginate by
const PAGE_SIZE: u32 = 50;

/// Search for a System
pub async fn search_systems_read(
    tx: &mut Transaction<'_, Postgres>,
    search_by: &str,
    page: &u32,
) -> Result<SearchSystems, poem::Error> {
    // Compute offset
    let offset: u32 = page * PAGE_SIZE;
    let next_offset: u32 = (page + 1) * PAGE_SIZE;

    // Pull the Systems
    let systems: Vec<System> = search_systems_select(tx, search_by, PAGE_SIZE, offset)
        .await
        .map_err(InternalServerError)?;

    // More Systems on next page?
    let more_systems: Vec<System> = search_systems_select(tx, search_by, PAGE_SIZE, next_offset)
        .await
        .map_err(InternalServerError)?;

    let next_page: Option<u32> = match more_systems.is_empty() {
        true => None,
        false => Some(page + 1),
    };

    Ok(SearchSystems {
        systems,
        search_by: search_by.to_string(),
        page: *page,
        next_page,
    })
}
