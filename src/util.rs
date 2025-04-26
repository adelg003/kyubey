use axum::http::StatusCode;
use tracing::error;

/// Map SQLx errors to Axum Status Codes
pub fn sqlx_to_axum_error(err: sqlx::Error) -> StatusCode {
    match err {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => {
            // Log any error that leads to a 500 response. That is an issue we need to fix on out
            // side.
            error!("{}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
