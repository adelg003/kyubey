use crate::core::{
    DagRun, DagRunTasks, SearchSystems, System, SystemDagRuns, Task, dag_runs_for_system_read,
    search_systems_read, tasks_for_dag_run_read,
};
use askama::Template;
use poem::{
    Route,
    error::InternalServerError,
    get, handler,
    web::{Data, Html, Path},
};
use sqlx::{PgPool, Postgres, Transaction};

/// Router for all UI Pages
pub fn route() -> Route {
    Route::new()
        .at("/", get(index))
        .at("/dag_runs/:sysetem_id", get(dag_runs))
        .at("/tasks/:run_id", get(tasks))
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
    systems: SearchSystems,
    navbar: NavBar,
}

/// Webpage to search for your system
#[handler]
async fn index(Data(pool): Data<&PgPool>) -> Result<Html<String>, poem::Error> {
    // Start Transaction
    let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(InternalServerError)?;

    // Pull the top of the list to pre-render the page.
    let systems: SearchSystems = search_systems_read(&mut tx, "", &0).await?;

    // Render the Index Page
    let index: String = Index {
        systems,
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

    // Render Tasks page
    let task_page: String = DagRunTaskPage {
        dag_run: tasks.dag_run,
        tasks: tasks.tasks,
        navbar: NavBar {
            system_id: Some(tasks.system.system_id),
            run_id: Some(run_id),
            task_id: None,
        },
    }
    .render()
    .map_err(InternalServerError)?;

    Ok(Html(task_page))
}
