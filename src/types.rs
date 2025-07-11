use axum::{http::StatusCode, response::IntoResponse};
use tera::Tera;
use tracing::error;

#[derive(Debug)]
pub struct AppError(anyhow::Error);

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        error!("{}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Something has gone wrong.",
        )
            .into_response()
    }
}

pub struct SharedState {
    pub tera: Tera,
}
