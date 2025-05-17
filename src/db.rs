use crate::core::{DagRun, DagState, System, Task, TaskState};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
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
                latest_run: Utc.from_utc_datetime(&latest_run),
                number_of_dag_runs: u64::try_from(number_of_dag_runs).ok()?,
            }),
            _ => None,
        }
    }
}

/// A row of the Dag Run table
struct DagRunRow {
    dag_id: String,
    execution_date: DateTime<Utc>,
    run_id: String,
    system_id: Option<String>,
    state: Option<String>,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
}

impl DagRunRow {
    /// Convert a DagRunRow to a DagRun
    fn into_dag_run(self) -> DagRun {
        let state: Option<DagState> = match self.state {
            Some(text) => DagState::from_str(&text).ok(),
            None => None,
        };

        DagRun {
            dag_id: self.dag_id,
            execution_date: self.execution_date,
            run_id: self.run_id,
            system_id: self.system_id,
            state,
            start_date: self.start_date,
            end_date: self.end_date,
        }
    }
}

/// A row of the Task table
struct TaskRow {
    run_id: String,
    task_id: String,
    state: Option<String>,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
    try_number: Option<i32>,
}

impl TaskRow {
    /// Convert a TaskRow to a Task
    fn into_task(self) -> Task {
        let state: Option<TaskState> = match self.state {
            Some(text) => TaskState::from_str(&text).ok(),
            None => None,
        };

        let try_number: Option<u32> = match self.try_number {
            Some(number) => u32::try_from(number).ok(),
            None => None,
        };

        Task {
            run_id: self.run_id,
            task_id: self.task_id,
            state,
            start_date: self.start_date,
            end_date: self.end_date,
            try_number,
        }
    }
}

/// Featch a single system
pub async fn system_select(
    tx: &mut Transaction<'_, Postgres>,
    system_id: &str,
) -> Result<System, sqlx::Error> {
    // Pull a system that meet our query
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

/// Featch the system for a Dag Run
pub async fn system_for_dag_run_select(
    tx: &mut Transaction<'_, Postgres>,
    run_id: &str,
) -> Result<System, sqlx::Error> {
    // Pull a system for a dag run
    let row = query_as!(
        SystemRow,
        "SELECT
            a.details ->> 'client_name' AS client_name,
            a.details ->> 'client_id' AS client_id,
            a.details ->> 'system_name' AS system_name,
            a.details ->> 'system_id' AS system_id,
            a.details ->> 'team_name' AS team_name,
            a.details ->> 'team_id' AS team_id,
            MAX(a.execution_date) AS latest_run,
            COUNT(a.*) as number_of_dag_runs
        FROM
            api_trigger a
        WHERE
            EXISTS(
                SELECT
                    *
                FROM
                    api_trigger b
                WHERE
                    b.details ->> 'system_id' = a.details ->> 'system_id'
                    AND b.run_id = $1
            )
        GROUP BY
            a.details ->> 'client_name',
            a.details ->> 'client_id',
            a.details ->> 'system_name',
            a.details ->> 'system_id',
            a.details ->> 'team_name',
            a.details ->> 'team_id'
        HAVING
            a.details ->> 'client_name' IS NOT NULL
            AND a.details ->> 'client_id' IS NOT NULL
            AND a.details ->> 'system_name' IS NOT NULL
            AND a.details ->> 'system_id' IS NOT NULL
            AND a.details ->> 'team_name' IS NOT NULL
            AND a.details ->> 'team_id' IS NOT NULL
        ",
        run_id,
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
            dag_run.dag_id,
            dag_run.execution_date,
            dag_run.run_id,
            api_trigger.details ->> 'system_id' AS system_id,
            dag_run.state,
            dag_run.start_date,
            dag_run.end_date
        FROM
            dag_run
        INNER JOIN
            api_trigger
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
            dag_run.execution_date,
            dag_run.dag_id",
        system_id,
    )
    .fetch_all(&mut **tx)
    .await?;

    // Filter out partial dag run rows. Only full details allowed
    let dag_runs: Vec<DagRun> = rows
        .into_iter()
        .map(|row: DagRunRow| row.into_dag_run())
        .collect();

    Ok(dag_runs)
}

/// Featch a single Dag Run
pub async fn dag_run_select(
    tx: &mut Transaction<'_, Postgres>,
    run_id: &str,
) -> Result<DagRun, sqlx::Error> {
    // Pull a Dag Run that meets our query
    let row = query_as!(
        DagRunRow,
        "SELECT
            dag_run.dag_id,
            dag_run.execution_date,
            dag_run.run_id,
            api_trigger.details ->> 'system_id' AS system_id,
            dag_run.state,
            dag_run.start_date,
            dag_run.end_date
        FROM
            dag_run
        LEFT JOIN
            api_trigger
        ON
            dag_run.run_id = api_trigger.run_id
        WHERE
            dag_run.run_id = $1
        ",
        run_id,
    )
    .fetch_one(&mut **tx)
    .await?;

    // Filter out partial system rows. Only full details allowed
    let dag_run: DagRun = row.into_dag_run();

    Ok(dag_run)
}

/// Pull all Tasks for a Run ID
pub async fn tasks_for_dag_run_select(
    tx: &mut Transaction<'_, Postgres>,
    run_id: &str,
) -> Result<Vec<Task>, sqlx::Error> {
    // Pull all tasks for a dag run
    let rows = query_as!(
        TaskRow,
        "SELECT
            run_id,
            task_id,
            state,
            start_date,
            end_date,
            try_number
        FROM
            task_instance
        WHERE
            run_id = $1
        ORDER BY
            dag_id,
            priority_weight,
            task_id",
        run_id,
    )
    .fetch_all(&mut **tx)
    .await?;

    // Filter out partial dag run rows. Only full details allowed
    let tasks: Vec<Task> = rows
        .into_iter()
        .map(|row: TaskRow| row.into_task())
        .collect();

    Ok(tasks)
}

/// Featch a single Task details
pub async fn task_select(
    tx: &mut Transaction<'_, Postgres>,
    run_id: &str,
    task_id: &str,
) -> Result<Task, sqlx::Error> {
    // Pull all tasks for a dag run
    let row = query_as!(
        TaskRow,
        "SELECT
            run_id,
            task_id,
            state,
            start_date,
            end_date,
            try_number
        FROM
            task_instance
        WHERE
            run_id = $1
            AND task_id = $2",
        run_id,
        task_id,
    )
    .fetch_one(&mut **tx)
    .await?;

    // Filter out partial system rows. Only full details allowed
    let task: Task = row.into_task();

    Ok(task)
}
