use crate::{
    Config,
    core::{System, Task, log_read, search_systems_read, task_read},
};
use maud::{Markup, html};
use poem::{
    error::InternalServerError,
    handler,
    http::StatusCode,
    web::{Data, Query},
};
use serde::Deserialize;
use sqlx::{PgPool, Postgres, Transaction};

/// Data for System Search Web Component
pub async fn search_systems_component(
    tx: &mut Transaction<'_, Postgres>,
    search_by: &str,
    page: &u32,
) -> Result<Markup, poem::Error> {
    // Search for anything that meets our criteria
    let systems: Vec<System> = search_systems_read(tx, search_by, page).await?;

    // More Systems on next page?
    let more_systems: Vec<System> = search_systems_read(tx, search_by, &(page + 1)).await?;

    let next_page: Option<u32> = match more_systems.is_empty() {
        true => None,
        false => Some(page + 1),
    };

    Ok(html! {
        // One Row per System retuened
        @for system in systems {
            tr
                id={ "row_" (system.system_id) }
                class="hover:bg-base-300 cursor-pointer animate-fade-up"
                href={ "/dag_runs/" (system.system_id) }
                onclick={ "window.location='/dag_runs/" (system.system_id) "';" } {
                td { (system.client_name) }
                td { (system.client_id) }
                td { (system.system_name) }
                td { (system.system_id) }
                td { (system.latest_run) }
                td class="text-center" { (system.number_of_dag_runs) }
            }
        }
        // Pagination Placeholder
        @if let Some(next_page) = next_page {
            tr
                id="search_pagination"
                hx-get={ "/component/search_systems?search_by=" (search_by) r"&page=" (next_page) }
                hx-trigger="revealed"
                hx-swap="outerHTML"
                hx-target="#search_pagination" {
            }
        }
    })
}

/// Paramiters to search by
#[derive(Deserialize)]
struct SearchParams {
    search_by: String,
    page: u32,
}

/// Web Component to search for your system
#[handler]
pub async fn search_systems_get(
    Data(pool): Data<&PgPool>,
    Query(params): Query<SearchParams>,
) -> Result<Markup, poem::Error> {
    // Start Transaction
    let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(InternalServerError)?;

    // Render component
    search_systems_component(&mut tx, &params.search_by, &params.page).await
}

/// Web Component for showing logs
pub async fn log_component(
    config: &Config,
    dag_id: &str,
    run_id: &str,
    task_id: &str,
    attempt: &u32,
    try_number: &u32,
) -> Result<Markup, poem::Error> {
    // Log for a task attempt
    let log: String = log_read(config, dag_id, run_id, task_id, attempt).await?;

    // Normalized all new lines to expected new lines
    let log = log.replace("\r\n", "\n").replace('\r', "\n");

    Ok(html! {
        div id="logs" class="pl-4 pr-4" {
            // Tabs
            div role="tablist" class="tabs tabs-box" {
                @for current_try in 1..=*try_number {
                    // Highlight the current attempt tab
                    @if current_try == *attempt {
                        a
                            role="tab"
                            class="tab tab-active" {
                            "Attempt: " (current_try)
                        }
                    // Have all other attemps link to their logs
                    } @else {
                        a
                            role="tab"
                            class="tab"
                            hx-get={ "/component/log?dag_id=" (dag_id) "&run_id=" (run_id) "&task_id=" (task_id) "&attempt=" (current_try)  }
                            hx-trigger="click"
                            hx-swap="outerHTML"
                            hx-target="#logs" {
                            "Attempt: " (current_try)
                        }
                    }
                }
            }
            // Show the logs
            div class="mockup-code w-full animate-fade" {
                // Need to keep pre and code in the same line to avoid adding empty lines in teh logs
                @for (index, line) in log.lines().enumerate() {
                    pre data-prefix=(index + 1) { code { (line) } }
                }
            }
        }
    })
}

/// Paramiters to Pull a log
#[derive(Deserialize)]
struct LogParams {
    dag_id: String,
    run_id: String,
    task_id: String,
    attempt: u32,
}

/// Web Component to search for your system
#[handler]
pub async fn log_get(
    Data(pool): Data<&PgPool>,
    Data(config): Data<&Config>,
    Query(params): Query<LogParams>,
) -> Result<Markup, poem::Error> {
    // Start Transaction
    let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(InternalServerError)?;

    // Pull task details so we know how mmany runs their should be.
    let task: Task = task_read(&mut tx, &params.run_id, &params.task_id).await?;

    // Make sure we should have logs from a run
    let try_number: u32 = match task.try_number {
        Some(try_number) if try_number > 0 => Ok(try_number),
        _ => Err(poem::Error::from_string(
            "No logs yet",
            StatusCode::NOT_FOUND,
        )),
    }?;

    // Render component
    log_component(
        config,
        &params.dag_id,
        &params.run_id,
        &params.task_id,
        &params.attempt,
        &try_number,
    )
    .await
}
