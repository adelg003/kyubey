use crate::{
    Config,
    core::{
        DagRun, DagRunTasks, System, SystemDagRuns, Task, dag_run_read, dag_runs_for_system_read,
        log_read, search_systems_read, system_read, task_read, tasks_for_dag_run_read,
    },
};
use poem::{error::InternalServerError, web::Data};
use poem_openapi::{
    OpenApi, Tags,
    param::{Path, Query},
    payload::{Json, PlainText},
};
use sqlx::{PgPool, Postgres, Transaction};

#[derive(Tags)]
enum Tag {
    #[oai(rename = "Dag Run")]
    DagRun,
    Log,
    System,
    Task,
}

/// Struct we will use to build our REST API
pub struct Api;

#[OpenApi]
impl Api {
    /// System details
    #[oai(path = "/system", method = "get", tag = Tag::System)]
    async fn systems_get(
        &self,
        Data(pool): Data<&PgPool>,
        Query(system_id): Query<String>,
    ) -> Result<Json<System>, poem::Error> {
        // Start Transaction
        let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(InternalServerError)?;

        // Search for anything that meets our criteria
        let system: System = system_read(&mut tx, &system_id).await?;

        Ok(Json(system))
    }

    /// Search for your system
    #[oai(path = "/search_systems", method = "get", tag = Tag::System)]
    async fn search_systems_get(
        &self,
        Data(pool): Data<&PgPool>,
        Query(search_by): Query<String>,
        Query(page): Query<u32>,
    ) -> Result<Json<Vec<System>>, poem::Error> {
        // Start Transaction
        let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(InternalServerError)?;

        // Search for anything that meets our criteria
        let systems: Vec<System> = search_systems_read(&mut tx, &search_by, &page).await?;

        Ok(Json(systems))
    }

    /// Dag Run Details
    #[oai(path = "/dag_run/:run_id", method = "get", tag = Tag::DagRun)]
    async fn dag_run_get(
        &self,
        Data(pool): Data<&PgPool>,
        Path(run_id): Path<String>,
    ) -> Result<Json<DagRun>, poem::Error> {
        // Start Transaction
        let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(InternalServerError)?;

        // Dag Runs for a System
        let dag_run: DagRun = dag_run_read(&mut tx, &run_id).await?;

        Ok(Json(dag_run))
    }

    /// Provide dag run and system details
    #[oai(path = "/dag_runs/:system_id", method = "get", tag = Tag::DagRun)]
    async fn dag_runs_for_system_get(
        &self,
        Data(pool): Data<&PgPool>,
        Path(system_id): Path<String>,
    ) -> Result<Json<SystemDagRuns>, poem::Error> {
        // Start Transaction
        let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(InternalServerError)?;

        // Dag Runs for a System
        let dag_runs: SystemDagRuns = dag_runs_for_system_read(&mut tx, &system_id).await?;

        Ok(Json(dag_runs))
    }

    /// Task Details
    #[oai(path = "/task/:run_id/:task_id", method = "get", tag = Tag::Task)]
    async fn task_get(
        &self,
        Data(pool): Data<&PgPool>,
        Path(run_id): Path<String>,
        Path(task_id): Path<String>,
    ) -> Result<Json<Task>, poem::Error> {
        // Start Transaction
        let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(InternalServerError)?;

        // Tasks for a Dag Runs
        let task: Task = task_read(&mut tx, &run_id, &task_id).await?;

        Ok(Json(task))
    }

    /// Provide Tasks that make up a Dag Run
    #[oai(path = "/tasks/:run_id", method = "get", tag = Tag::Task)]
    async fn tasks_for_dag_run_get(
        &self,
        Data(pool): Data<&PgPool>,
        Path(run_id): Path<String>,
    ) -> Result<Json<DagRunTasks>, poem::Error> {
        // Start Transaction
        let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(InternalServerError)?;

        // Tasks for a Dag Runs
        let tasks: DagRunTasks = tasks_for_dag_run_read(&mut tx, &run_id).await?;

        Ok(Json(tasks))
    }

    /// Provide the Log for a task
    #[oai(path = "/log", method = "get", tag = Tag::Log)]
    async fn log_get(
        &self,
        Data(config): Data<&Config>,
        Query(dag_id): Query<String>,
        Query(run_id): Query<String>,
        Query(task_id): Query<String>,
        Query(attempt): Query<u32>,
    ) -> Result<PlainText<String>, poem::Error> {
        // Log for a task attempt
        let log: String = log_read(config, &dag_id, &run_id, &task_id, &attempt).await?;

        Ok(PlainText(log))
    }
}
