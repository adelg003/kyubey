use crate::core::{DagState, TaskState};

/// Translate a DagState to a Badge Type
pub fn dag_state_badge_type(state: &DagState) -> &'static str {
    match state {
        DagState::Failed => "badge-error",
        DagState::Queued => "badge-neutral",
        DagState::Running => "badge-primary",
        DagState::Success => "badge-success",
    }
}

/// Translate a DagState to a Badge Type
pub fn task_state_badge_type(state: &TaskState) -> &'static str {
    match state {
        TaskState::Deferred => "badge-info",
        TaskState::Failed => "badge-error",
        TaskState::Queued => "badge-neutral",
        TaskState::Removed => "badge-neutral",
        TaskState::Restarting => "badge-secondary",
        TaskState::Running => "badge-primary",
        TaskState::Scheduled => "badge-neutral",
        TaskState::Skipped => "badge-neutral",
        TaskState::Success => "badge-success",
        TaskState::UpForReschedule => "badge-warning",
        TaskState::UpForRetry => "badge-warning",
        TaskState::UpstreamFailed => "badge-warning",
    }
}
