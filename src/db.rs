use crate::core::{DagRun, DagState, System};
use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::{Postgres, Transaction, query_as};
use std::str::FromStr;

/// Results for a single system
struct SystemRow {
    client_name: Option<String>,
    client_id: Option<String>,
    system_name: Option<String>,
    system_id: Option<String>,
    team_name: Option<String>,
    team_id: Option<String>,
    latest_run: Option<NaiveDateTime>,
    number_of_dag_runs: Option<i64>,
}

impl SystemRow {
    /// Convert a SystemRow into a System
    fn into_system(self) -> Option<System> {
        match self {
            SystemRow {
                client_name: Some(client_name),
                client_id: Some(client_id),
                system_name: Some(system_name),
                system_id: Some(system_id),
                team_name: Some(team_name),
                team_id: Some(team_id),
                latest_run: Some(latest_run),
                number_of_dag_runs: Some(number_of_dag_runs),
            } => Some(System {
                client_name,
                client_id,
                system_name,
                system_id,
                team_name,
                team_id,
                latest_run,
                number_of_dag_runs: u64::try_from(number_of_dag_runs).ok()?,
            }),
            _ => None,
        }
    }
}

/// A row of the Dag Run table
struct DagRunRow {
    dag_id: Option<String>,
    execution_date: Option<NaiveDateTime>,
    run_id: Option<String>,
    state: Option<String>,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
}

impl DagRunRow {
    /// Convert a DagRunRow to a DagRun
    fn into_dag_run(self) -> Option<DagRun> {
        match self {
            DagRunRow {
                dag_id: Some(dag_id),
                execution_date: Some(execution_date),
                run_id: Some(run_id),
                state: Some(state),
                start_date,
                end_date,
            } => Some(DagRun {
                dag_id,
                execution_date,
                run_id,
                state: DagState::from_str(&state).ok()?,
                start_date,
                end_date,
            }),
            _ => None,
        }
    }
}

/// Featch a single system
pub async fn system_select(
    tx: &mut Transaction<'_, Postgres>,
    system_id: &str,
) -> Result<System, sqlx::Error> {
    // Pull all systems that meet our query
    let row = query_as!(
        SystemRow,
        "SELECT
            details ->> 'client_name' AS client_name,
            details ->> 'client_id' AS client_id,
            details ->> 'system_name' AS system_name,
            details ->> 'system_id' AS system_id,
            details ->> 'team_name' AS team_name,
            details ->> 'team_id' AS team_id,
            MAX(execution_date) AS latest_run,
            COUNT(*) as number_of_dag_runs
        FROM
            api_trigger
        WHERE
            details ->> 'system_id' = $1
        GROUP BY
            details ->> 'client_name',
            details ->> 'client_id',
            details ->> 'system_name',
            details ->> 'system_id',
            details ->> 'team_name',
            details ->> 'team_id'
        HAVING
            details ->> 'client_name' IS NOT NULL
            AND details ->> 'client_id' IS NOT NULL
            AND details ->> 'system_name' IS NOT NULL
            AND details ->> 'system_id' IS NOT NULL
            AND details ->> 'team_name' IS NOT NULL
            AND details ->> 'team_id' IS NOT NULL
        ",
        system_id,
    )
    .fetch_one(&mut **tx)
    .await?;

    // Filter out partial system rows. Only full details allowed
    let system: System = match row.into_system() {
        Some(system) => Ok(system),
        None => Err(sqlx::Error::RowNotFound),
    }?;

    Ok(system)
}

/// Search our database for systems
pub async fn search_systems_select(
    tx: &mut Transaction<'_, Postgres>,
    search_by: &str,
    limit: u32,
    offset: u32,
) -> Result<Vec<System>, sqlx::Error> {
    // Format seach_by so it supports SQL wildcars while allowing for save SQL preperation.
    let search_by: String = format!("%{}%", search_by);

    // Pull all systems that meet our query
    let rows = query_as!(
        SystemRow,
        "SELECT
            details ->> 'client_name' AS client_name,
            details ->> 'client_id' AS client_id,
            details ->> 'system_name' AS system_name,
            details ->> 'system_id' AS system_id,
            details ->> 'team_name' AS team_name,
            details ->> 'team_id' AS team_id,
            MAX(execution_date) AS latest_run,
            COUNT(*) as number_of_dag_runs
        FROM
            api_trigger
        WHERE
            details ->> 'client_name' ILIKE $1
            OR details ->> 'client_id' ILIKE $1
            OR details ->> 'system_name' ILIKE $1
            OR details ->> 'system_id' ILIKE $1
            OR details ->> 'team_name' ILIKE $1
            OR details ->> 'team_id' ILIKE $1
        GROUP BY
            details ->> 'client_name',
            details ->> 'client_id',
            details ->> 'system_name',
            details ->> 'system_id',
            details ->> 'team_name',
            details ->> 'team_id'
        HAVING
            details ->> 'client_name' IS NOT NULL
            AND details ->> 'client_id' IS NOT NULL
            AND details ->> 'system_name' IS NOT NULL
            AND details ->> 'system_id' IS NOT NULL
            AND details ->> 'team_name' IS NOT NULL
            AND details ->> 'team_id' IS NOT NULL
        ORDER BY
            MAX(execution_date) DESC,
            details ->> 'system_id'
        LIMIT
            $2
        OFFSET
            $3",
        search_by,
        i64::from(limit),
        i64::from(offset),
    )
    .fetch_all(&mut **tx)
    .await?;

    // Filter out partial system rows. Only full details allowed
    let systems: Vec<System> = rows
        .into_iter()
        .filter_map(|row: SystemRow| row.into_system())
        .collect();

    Ok(systems)
}

/// Pull all DAG Runs for a System
pub async fn dag_runs_by_system_select(
    tx: &mut Transaction<'_, Postgres>,
    system_id: &str,
) -> Result<Vec<DagRun>, sqlx::Error> {
    //Pull all dag runs for a system
    let rows = query_as!(
        DagRunRow,
        "SELECT
            api_trigger.dag_id,
            api_trigger.execution_date,
            api_trigger.run_id,
            dag_run.state,
            dag_run.start_date,
            dag_run.end_date
        FROM
            api_trigger
        LEFT JOIN
            dag_run
        ON
            dag_run.run_id = api_trigger.run_id
        WHERE
            api_trigger.details ->> 'system_id' = $1
            AND api_trigger.details ->> 'client_name' IS NOT NULL
            AND api_trigger.details ->> 'client_id' IS NOT NULL
            AND api_trigger.details ->> 'system_name' IS NOT NULL
            AND api_trigger.details ->> 'team_name' IS NOT NULL
            AND api_trigger.details ->> 'team_id' IS NOT NULL
       ORDER BY
            api_trigger.execution_date,
            api_trigger.dag_id",
        system_id,
    )
    .fetch_all(&mut **tx)
    .await?;

    // Filter out partial dag run rows. Only full details allowed
    let dag_runs: Vec<DagRun> = rows
        .into_iter()
        .filter_map(|row: DagRunRow| row.into_dag_run())
        .collect();

    Ok(dag_runs)
}
