use crate::search::core::System;
use sqlx::{Postgres, Transaction, query_as};

pub async fn search_systems_select(
    tx: &mut Transaction<'_, Postgres>,
    search_by: &Option<String>,
    limit: i64,
    offset: i64,
) -> Result<Vec<System>, sqlx::Error> {
    // Format seach_by so it supports SQL wildcars while allowing for save SQL preperation.
    //let search_by: String = search_by.unwrap_or("".to_string());
    let search_by: String = format!("%{}%", search_by.clone().unwrap_or("".to_string()));

    // Pull all systems that meet our query
    query_as!(
        System,
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
        limit,
        offset,
    )
    .fetch_all(&mut **tx)
    .await
}
