use crate::ui::snippet::{head, header};
use maud::{DOCTYPE, Markup, html};

/// Base Page Layout
pub fn base_layout(
    title: &str,
    system_id: &Option<String>,
    run_id: &Option<String>,
    task_id: &Option<String>,
    main: Markup,
) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en-US" {
            (head())
            body {
                (header(title, system_id, run_id, task_id))
                main {
                    (main)
                }
            }
        }
    }
}
