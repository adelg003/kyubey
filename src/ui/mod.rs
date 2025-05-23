mod component;
mod page;
mod snippet;

use component::{log_get, search_systems_get};
use page::{dag_runs, index, logs, tasks};
use poem::{Route, get};

/// Router for UI
pub fn route() -> Route {
    Route::new()
        .at("/", get(index))
        .at("/component/log", get(log_get))
        .at("/component/search_systems", get(search_systems_get))
        .at("/dag_runs/:sysetem_id", get(dag_runs))
        .at("/logs/:run_id/:task_id", get(logs))
        .at("/tasks/:run_id", get(tasks))
}
