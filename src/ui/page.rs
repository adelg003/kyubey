use crate::{
    Config,
    core::{
        DagRun, DagRunTasks, System, SystemDagRuns, Task, dag_run_read, dag_runs_for_system_read,
        system_for_dag_run_read, system_read, task_read, tasks_for_dag_run_read,
    },
    ui::{
        component::{log_component, search_systems_component},
        layout::base_layout,
        snippet::{dag_run_stats, system_stats, task_stats},
        util::{dag_state_badge_type, task_state_badge_type},
    },
};
use maud::{Markup, html};
use poem::{
    error::InternalServerError,
    handler,
    http::StatusCode,
    web::{Data, Path},
};
use sqlx::{PgPool, Postgres, Transaction};

/// Index Page
#[handler]
pub async fn index(Data(pool): Data<&PgPool>) -> Result<Markup, poem::Error> {
    // Start Transaction
    let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(InternalServerError)?;

    // Pull the top of the list to pre-render the page.
    let search: Markup = search_systems_component(&mut tx, "", &0).await?;

    Ok(base_layout(
        "Search",
        &None,
        &None,
        &None,
        html! {
            // Search for a System
            fieldset class="fieldset m-8 animate-fade" {
                legend class="fieldset-legend" { "Search by:" }
                input
                    id="search_by_input"
                    name="search_by"
                    class="input"
                    type="search"
                    placeholder="Begin typing to search Systems..."
                    hx-get="/component/search_systems?page=0"
                    hx-trigger="input changed delay:500ms, keyup[key=='Enter']"
                    hx-target="#search_results"
                    hx-swap="innerHTML";

            }
            // Search Results
            table class="table table-zebra table-sm animate-fade" {
                thead {
                    tr {
                        th { "Client Name" }
                        th { "Client ID" }
                        th { "System Name" }
                        th { "System ID" }
                        th { "Lastest Dag Run" }
                        th { "Dag Runs" }
                    }
                }
                tbody id="search_results" {
                    (search)
                }
            }
        },
    ))
}

/// Webpage to view dag runs for a system
#[handler]
pub async fn dag_runs(
    Data(pool): Data<&PgPool>,
    Path(system_id): Path<String>,
) -> Result<Markup, poem::Error> {
    // Start Transaction
    let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(InternalServerError)?;

    // Search for anything that meets our criteria
    let dag_runs: SystemDagRuns = dag_runs_for_system_read(&mut tx, &system_id).await?;

    Ok(base_layout(
        "Dag Runs",
        &Some(system_id),
        &None,
        &None,
        html! {
            div class="animate-fade" { (system_stats(&dag_runs.system)) }
            // Dag Run Table
            table class="table table-zebra table-sm animate-fade" {
                thead {
                    tr {
                        th { "Dag ID" }
                        th { "Execution Date" }
                        th { "State" }
                        th { "Run ID" }
                        th { "Start Date" }
                        th { "End Date" }
                    }
                }
                tbody class="animate-fade-up" {
                    @for dag_run in dag_runs.dag_runs {
                        tr
                            id={ "row_" (dag_run.run_id) }
                            class="hover:bg-base-300 cursor-pointer"
                            onclick={ "window.location='/tasks/" (dag_run.run_id) "';"} {
                            // Dag ID and Execution Date
                            td { (dag_run.dag_id) }
                            td { (dag_run.execution_date) }
                            // Dag State Badge
                            @if let Some(state) = dag_run.state {
                                td class={ "badge " (dag_state_badge_type(&state)) } { (state) }
                            } @else {
                                td {}
                            }
                            td { (dag_run.run_id) }
                            // Start and End Dates
                            td { @if let Some(start_date) = dag_run.start_date { (start_date) } }
                            td { @if let Some(end_date) = dag_run.end_date { (end_date) } }
                        }
                    }
                }
            }
        },
    ))
}

// Webpage to view Tasks for a Dag Run
#[handler]
pub async fn tasks(
    Data(pool): Data<&PgPool>,
    Path(run_id): Path<String>,
) -> Result<Markup, poem::Error> {
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

    Ok(base_layout(
        "Tasks",
        &Some(system_id),
        &Some(run_id),
        &None,
        html! {
            (system_stats(&system))
            div class="animate-fade" { (dag_run_stats(&tasks.dag_run)) }
            // Task Table
            table class="table table-zebra table-sm animate-fade" {
                thead {
                    tr {
                        th { "Task ID" }
                        th { "State" }
                        th { "Start Date" }
                        th { "End Date" }
                        th { "Attempts" }
                    }
                }
                tbody class="animate-fade-up" {
                    @for task in tasks.tasks {
                        // Pre-compute / formate some values
                        @let try_number: u32 = task.try_number.unwrap_or(0);
                        @let state_cell: Markup = match task.state {
                            Some(state) => html! {
                                td class={ "badge " (task_state_badge_type(&state)) } { (state) }
                            },
                            None => html! { td {} },
                        };
                        @let start_date: String = match task.start_date {
                            Some(start_date) => start_date.to_string(),
                            None => "".to_string()
                        };
                        @let end_date: String = match task.end_date {
                            Some(end_date) => end_date.to_string(),
                            None => "".to_string()
                        };
                        // Should the row link to the logs?
                        @if try_number > 0 {
                            tr
                                id={ "row_" (task.task_id) }
                                class="hover:bg-base-300 cursor-pointer"
                                onclick={ "window.location='/logs/" (task.run_id) "/" (task.task_id) "';" } {
                                td { (task.task_id) }
                                // Task State Badge
                                (state_cell)
                                // Start and End Date
                                td { (start_date) }
                                td { (end_date) }
                                // Attepts
                                td class="text-center" { (try_number) }
                            }
                        } @else {
                            tr
                                id={ "row_" (task.task_id) }
                                class="hover:bg-base-300" {
                                td { (task.task_id) }
                                // Task State Badge
                                (state_cell)
                                // Start and End Date
                                td { (start_date) }
                                td { (end_date) }
                                // Attepts
                                td  {}
                            }
                        }
                    }
                }
            }
        },
    ))
}

/// Webpage to view logs for a task run
#[handler]
pub async fn logs(
    Data(config): Data<&Config>,
    Data(pool): Data<&PgPool>,
    Path((run_id, task_id)): Path<(String, String)>,
) -> Result<Markup, poem::Error> {
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
    let log: Markup = log_component(
        config,
        &dag_run.dag_id,
        &dag_run.run_id,
        &task.task_id,
        &try_number, // Use the latest run as the attempt on page load
        &try_number,
    )
    .await?;

    Ok(base_layout(
        "Tasks",
        &Some(system.system_id.clone()),
        &Some(run_id),
        &Some(task_id),
        html! {
            (system_stats(&system))
            (dag_run_stats(&dag_run))
            div class="animate-fade" { (task_stats(&task)) }
            // Logs Text Box
            (log)
        },
    ))
}
