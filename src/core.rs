use crate::db::{dag_runs_by_system_select, search_systems_select, system_select};
use askama::Template;
use chrono::{DateTime, NaiveDateTime, Utc};
use poem::error::{InternalServerError, NotFound};
use poem_openapi::{Enum, Object};
use sqlx::{Postgres, Transaction};
use std::{fmt, str::FromStr};

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
#[template(path = "component/search_rows.html")]
pub struct SearchSystems {
    systems: Vec<System>,
    search_by: String,
    page: u32,
    next_page: Option<u32>,
}

/// All States a DAG can be in
#[derive(Enum)]
#[oai(rename_all = "lowercase")]
pub enum DagState {
    Failed,
    Queued,
    Running,
    Success,
}

impl FromStr for DagState {
    type Err = ();

    /// Map a string to a DAG Run Status
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "failed" => Ok(Self::Failed),
            "queued" => Ok(Self::Queued),
            "runnning" => Ok(Self::Running),
            "success" => Ok(Self::Success),
            _ => Err(()),
        }
    }
}

impl fmt::Display for DagState {
    /// How to formate the DagState for HTML rendering
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text: &str = match self {
            DagState::Failed => "failed",
            DagState::Queued => "queued",
            DagState::Running => "runnning",
            DagState::Success => "success",
        };
        write!(formatter, "{}", text)
    }
}

/// A single dag run
#[derive(Object)]
pub struct DagRun {
    pub dag_id: String,
    pub execution_date: NaiveDateTime,
    pub run_id: String,
    pub state: DagState,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

/// All Dag Runs for a System
#[derive(Object)]
pub struct SystemDagRuns {
    pub system: System,
    pub dag_runs: Vec<DagRun>,
}

/// How much do we want to paginate by
const PAGE_SIZE: u32 = 50;

/// Pull details for a single system
async fn system_read(
    tx: &mut Transaction<'_, Postgres>,
    system_id: &str,
) -> Result<System, poem::Error> {
    // Pull details Systems
    let system: System = match system_select(tx, system_id).await {
        Ok(system) => Ok(system),
        Err(sqlx::Error::RowNotFound) => Err(NotFound(sqlx::Error::RowNotFound)),
        Err(err) => Err(InternalServerError(err)),
    }?;

    Ok(system)
}

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

/// Dag Runs by System
pub async fn dag_runs_by_system_read(
    tx: &mut Transaction<'_, Postgres>,
    system_id: &str,
) -> Result<SystemDagRuns, poem::Error> {
    // Pull the Systems
    let system: System = system_read(tx, system_id).await?;

    // Pull dag runs for that system
    let dag_runs: Vec<DagRun> = dag_runs_by_system_select(tx, system_id)
        .await
        .map_err(InternalServerError)?;

    Ok(SystemDagRuns { system, dag_runs })
}
