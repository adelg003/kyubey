use crate::{search::db::search_systems_select, util::sqlx_to_axum_error};
use axum::http::StatusCode;
use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::{Postgres, Transaction};

/// Results for a single system
#[derive(Serialize)]
pub struct System {
    pub client_name: Option<String>,
    pub client_id: Option<String>,
    pub system_name: Option<String>,
    pub system_id: Option<String>,
    pub team_name: Option<String>,
    pub team_id: Option<String>,
    pub latest_run: Option<NaiveDateTime>,
    pub number_of_dag_runs: Option<i64>,
}

/// Search for a system
#[derive(Serialize)]
pub struct SearchSystems {
    systems: Vec<System>,
    page: u32,
    more: bool,
}

/// How much do we want to paginate by
const PAGE_SIZE: u32 = 50;

/// Search for a System
pub async fn search_systems_read(
    tx: &mut Transaction<'_, Postgres>,
    search_by: &Option<String>,
    page: u32,
) -> Result<SearchSystems, StatusCode> {
    // Compute offset
    let offset: i64 = (page * PAGE_SIZE).into();
    let next_offset: i64 = ((page + 1) * PAGE_SIZE).into();

    // Pull the Systems
    let systems: Vec<System> = search_systems_select(tx, search_by, PAGE_SIZE.into(), offset)
        .await
        .map_err(sqlx_to_axum_error)?;

    // More Systems on next page?
    let more_systems: Vec<System> =
        search_systems_select(tx, search_by, PAGE_SIZE.into(), next_offset)
            .await
            .map_err(sqlx_to_axum_error)?;

    let more: bool = !more_systems.is_empty();

    Ok(SearchSystems {
        systems,
        page,
        more,
    })
}
