use crate::{
    core::{DagRun, System, Task},
    ui::util::{dag_state_badge_type, task_state_badge_type},
};
use maud::{Markup, html};

/// HTML Page Head
pub fn head() -> Markup {
    html! {
        head {
            title {"／人◕ ‿‿ ◕人＼"}
            meta name="description" content="Inspect Airflow Systems and Dag Runs";
            meta name="keywords" content="Airflow, Dag Runs, Kyubey, Logs, Search, Tasks";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            link rel="icon" type="image/x-icon" href="/assets/images/favicon.ico";
            link rel="stylesheet" type="text/css" href="/assets/css/main.css";
            script defer src="/assets/scripts/htmx.js" {}
        }
    }
}

/// NavBar
fn navbar(system_id: &Option<String>, run_id: &Option<String>, task_id: &Option<String>) -> Markup {
    html! {
        nav class="breadcrumbs ml-8" {
            ul {
              li { a href="/" { "Search" } }
                @if let Some(system_id) = system_id {
                    li { a href={ "/dag_runs/" (system_id) } { "DagRuns" } }
                    @if let Some(run_id) = run_id {
                        li { a href={ "/tasks/" (run_id) } { "Tasks" } }
                        @if let Some(task_id) = task_id {
                            li { a href={ "/logs/" (run_id) "/" (task_id) } { "Logs" } }
                        }
                    }
                }
            }
        }
    }
}

// Page Title
fn page_title(title: &str) -> Markup {
    html! {
        h1 class="text-6xl ml-8 pb-2 bg-gradient-to-r from-orange-700 to-amber-200 bg-clip-text text-transparent" {
            "Kyubey: "
            span class="animate-fade bg-gradient-to-r from-amber-600 to-amber-400 bg-clip-text text-transparent" {
                (title)
            }
        }
    }
}

/// Header at the top of every page
pub fn header(
    title: &str,
    system_id: &Option<String>,
    run_id: &Option<String>,
    task_id: &Option<String>,
) -> Markup {
    html! {
        header {
            (navbar(system_id, run_id, task_id))
            (page_title(title))
        }
    }
}

/// List out Sysetem Details
pub fn system_stats(system: &System) -> Markup {
    html! {
        div class="stats shadow" {
            // Client
            div class="stat" {
                div class="stat-title" { "Client Name" }
                div class="stat-value" { (system.client_name) }
                div class="stat-desc" { "ID: " (system.client_id) }
            }
            // System
            div class="stat" {
                div class="stat-title" { "System Name" }
                div class="stat-value" { (system.system_name) }
                div class="stat-desc" { "ID: " (system.system_id) }
            }
            // Dag Runs
            div class="stat" {
                div class="stat-title" { "Dag Runs" }
                div class="stat-value text-left" { (system.number_of_dag_runs) }
                div class="stat-desc" { "Latest: " (system.latest_run) }
            }
        }
    }
}

/// List out Dag Run Details
pub fn dag_run_stats(dag_run: &DagRun) -> Markup {
    html! {
        div class="stats shadow" {
            // Dag In and Run ID
            div class="stat" {
                div class="stat-title" { "DAG ID" }
                div class="stat-value" { (dag_run.dag_id) }
                div class="stat-desc" { "Run ID: " (dag_run.run_id) }
            }
            // Execution Date
            div class="stat" {
                div class="stat-title" { "Execution Data" }
                div class="stat-value" { (dag_run.execution_date) }
                div class="stat-desc" {
                    @if let Some(start_date) = dag_run.start_date { "Start Date: " (start_date) }
                    @if let Some(end_date) = dag_run.end_date { "End Date: " (end_date) }
                }
            }
            // Dag State
            div class="stat" {
                div class="stat-title" { "State" }
                @if let Some(state) = &dag_run.state {
                    div class={ "stat-value badge badge-xl " (dag_state_badge_type(state)) } { (state) }
                } @else {
                    div class="stat-value" {}
                }
                div class="stat-desc" {}
            }
        }
    }
}

/// List out Task Details
pub fn task_stats(task: &Task) -> Markup {
    html! {
        div class="stats shadow" {
            // Task ID and Runtime
            div class="stat" {
                div class="stat-title" { "Task ID" }
                div class="stat-value" { (task.task_id) }
                div class="stat-desc" {
                    @if let Some(start_date) = task.start_date { "Start Date: " (start_date) }
                    @if let Some(end_date) = task.end_date { "End Date: " (end_date) }
                }
            }
            // Task State
            div class="stat" {
                div class="stat-title" { "State" }
                @if let Some(state) = &task.state {
                    div class={ "stat-value badge badge-xl " (task_state_badge_type(state)) } { (state) }
                } @else {
                    div class="stat-value" {}
                }
                div class="stat-desc" {}
            }
            // Attempts
            div class="stat" {
                div class="stat-title" { "Attempts" }
                @ if let Some(try_number) = task.try_number {
                    div class="stat-value" { (try_number) }
                } @else {
                    div class="stat-value" {}
                }
                div class="stat-desc" {}
            }
        }
    }
}
