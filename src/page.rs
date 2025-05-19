use crate::{
    Config,
    component::{LogComponent, SearchComponent, log_component, search_systems_component},
    core::{
        DagRun, DagRunTasks, System, SystemDagRuns, Task, dag_run_read, dag_runs_for_system_read,
        system_for_dag_run_read, system_read, task_read, tasks_for_dag_run_read,
    },
};
use askama::Template;
use poem::{
    Route,
    error::InternalServerError,
    get, handler,
    http::StatusCode,
    web::{Data, Html, Path},
};
use sqlx::{PgPool, Postgres, Transaction};

/// Router for all UI Pages
pub fn route() -> Route {
    Route::new()
        .at("/", get(index))
        .at("/dag_runs/:sysetem_id", get(dag_runs))
        .at("/tasks/:run_id", get(tasks))
        .at("/logs/:run_id/:task_id", get(logs))
}

/// Navbar for all pages
struct NavBar {
    system_id: Option<String>,
    run_id: Option<String>,
    task_id: Option<String>,
}

/// Search for a system
#[derive(Template)]
#[template(path = "page/index.html")]
struct Index {
    search: SearchComponent,
    navbar: NavBar,
}

/// Webpage to search for your system
#[handler]
async fn index(Data(pool): Data<&PgPool>) -> Result<Html<String>, poem::Error> {
    // Start Transaction
    let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(InternalServerError)?;

    // Pull the top of the list to pre-render the page.
    let search: SearchComponent = search_systems_component(&mut tx, "", &0).await?;

    // Render the Index Page
    let index: String = Index {
        search,
        navbar: NavBar {
            system_id: None,
            run_id: None,
            task_id: None,
        },
    }
    .render()
    .map_err(InternalServerError)?;

    Ok(Html(index))
}

/// All Dag Runs for a System
#[derive(Template)]
#[template(path = "page/dag_runs.html")]
struct SystemDagRunPage {
    system: System,
    dag_runs: Vec<DagRun>,
    navbar: NavBar,
}

/// Webpage to view dag runs for a system
#[handler]
async fn dag_runs(
    Data(pool): Data<&PgPool>,
    Path(system_id): Path<String>,
) -> Result<Html<String>, poem::Error> {
    // Start Transaction
    let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(InternalServerError)?;

    // Search for anything that meets our criteria
    let dag_runs: SystemDagRuns = dag_runs_for_system_read(&mut tx, &system_id).await?;

    // Render Dag Run page
    let dag_run_page: String = SystemDagRunPage {
        system: dag_runs.system,
        dag_runs: dag_runs.dag_runs,
        navbar: NavBar {
            system_id: Some(system_id),
            run_id: None,
            task_id: None,
        },
    }
    .render()
    .map_err(InternalServerError)?;

    Ok(Html(dag_run_page))
}

/// All Task for a Dag Run
#[derive(Template)]
#[template(path = "page/tasks.html")]
struct DagRunTaskPage {
    system: System,
    dag_run: DagRun,
    tasks: Vec<Task>,
    navbar: NavBar,
}

/// Webpage to view Tasks for a Dag Run
#[handler]
async fn tasks(
    Data(pool): Data<&PgPool>,
    Path(run_id): Path<String>,
) -> Result<Html<String>, poem::Error> {
    // Start Transaction
    let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(InternalServerError)?;

    // Search for anything that meets our criteria
    let tasks: DagRunTasks = tasks_for_dag_run_read(&mut tx, &run_id).await?;

    // Make sure our dag run has a parent system
    let system_id: String = match &tasks.dag_run.system_id {
        Some(system_id) => Ok(system_id.to_string()),
        None => Err(poem::Error::from_string(
            "No Parent System Found",
            StatusCode::NOT_FOUND,
        )),
    }?;

    // Pull System details
    let system: System = system_read(&mut tx, &system_id).await?;

    // Render Tasks page
    let task_page: String = DagRunTaskPage {
        navbar: NavBar {
            system_id: Some(system.system_id.clone()),
            run_id: Some(run_id),
            task_id: None,
        },
        system,
        dag_run: tasks.dag_run,
        tasks: tasks.tasks,
    }
    .render()
    .map_err(InternalServerError)?;

    Ok(Html(task_page))
}

/// Page to view logs
#[derive(Template)]
#[template(path = "page/logs.html")]
struct LogPage {
    system: System,
    dag_run: DagRun,
    task: Task,
    log: LogComponent,
    navbar: NavBar,
}

/// Webpage to view logs for a task run
#[handler]
async fn logs(
    Data(config): Data<&Config>,
    Data(pool): Data<&PgPool>,
    Path((run_id, task_id)): Path<(String, String)>,
) -> Result<Html<String>, poem::Error> {
    // Start Transaction
    let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(InternalServerError)?;

    // Pull system, dag run and task details
    let system: System = system_for_dag_run_read(&mut tx, &run_id).await?;
    let dag_run: DagRun = dag_run_read(&mut tx, &run_id).await?;
    let task: Task = task_read(&mut tx, &run_id, &task_id).await?;

    // Make sure we should have logs from a run
    let try_number: u32 = match task.try_number {
        Some(try_number) if try_number > 0 => Ok(try_number),
        _ => Err(poem::Error::from_string(
            "No logs yet",
            StatusCode::NOT_FOUND,
        )),
    }?;

    // Pull the log component
    let log: LogComponent = log_component(
        config,
        &dag_run.dag_id,
        &dag_run.run_id,
        &task.task_id,
        &try_number, // Use the latest run as the attempt on page load
        &try_number,
    )
    .await?;

    // Render Log page
    let task_page: String = LogPage {
        navbar: NavBar {
            system_id: Some(system.system_id.clone()),
            run_id: Some(run_id),
            task_id: Some(task_id),
        },
        system,
        dag_run,
        task,
        log,
    }
    .render()
    .map_err(InternalServerError)?;

    Ok(Html(task_page))
}
