use crate::core::System;
use chrono::NaiveDateTime;
use sqlx::{Postgres, Transaction, query_as};

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
        .filter_map(|row: SystemRow| {
            let client_name: String = match &row.client_name {
                Some(client_name) => client_name.to_string(),
                None => return None,
            };

            let client_id: String = match &row.client_id {
                Some(client_id) => client_id.to_string(),
                None => return None,
            };

            let system_name: String = match &row.system_name {
                Some(system_name) => system_name.to_string(),
                None => return None,
            };

            let system_id: String = match &row.system_id {
                Some(system_id) => system_id.to_string(),
                None => return None,
            };

            let team_name: String = match &row.team_name {
                Some(team_name) => team_name.to_string(),
                None => return None,
            };

            let team_id: String = match &row.team_id {
                Some(team_id) => team_id.to_string(),
                None => return None,
            };

            let latest_run: NaiveDateTime = match &row.latest_run {
                Some(latest_run) => *latest_run,
                None => return None,
            };

            let number_of_dag_runs: u64 = match &row.number_of_dag_runs {
                Some(number_of_dag_runs) => match u64::try_from(*number_of_dag_runs) {
                    Ok(number_of_dag_runs) => number_of_dag_runs,
                    Err(_) => return None,
                },
                None => return None,
            };

            Some(System {
                client_name,
                client_id,
                system_name,
                system_id,
                team_name,
                team_id,
                latest_run,
                number_of_dag_runs,
            })
        })
        .collect();

    Ok(systems)
}
