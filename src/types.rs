use anyhow::Result;
use axum::{http::StatusCode, response::IntoResponse};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tera::Tera;
use tokio::sync::RwLock;
use tokio_postgres::{Client, Row};
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
    pub client: RwLock<Client>,
}

#[derive(Debug, Serialize)]
pub struct Page {
    pub id: Option<i32>,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub description: String,
    pub markdown: String,
    pub preview: String,
    pub published_at: Option<DateTime<Utc>>,
    pub revised_at: Option<DateTime<Utc>>,
    pub slug: String,
    pub title: String,
    pub updated_at: DateTime<Utc>,
}

impl Page {
    pub async fn published(client: &Client) -> Result<Vec<Self>> {
        let rows = client
            .query(
                "SELECT * FROM pages WHERE
                published_at IS NOT NULL ORDER
            BY GREATEST(revised_at , published_at) DESC",
                &[],
            )
            .await?;

        rows.into_iter().map(Self::try_from).collect()
    }

    pub async fn all(client: &Client) -> Result<Vec<Self>> {
        let rows = client.query("SELECT * FROM pages ORDER BY id", &[]).await?;

        rows.into_iter().map(Self::try_from).collect()
    }
}

impl TryFrom<Row> for Page {
    type Error = anyhow::Error;
    fn try_from(row: Row) -> Result<Self> {
        Ok(Page {
            content: row.try_get("content")?,
            created_at: row.try_get("created_at")?,
            description: row.try_get("description")?,
            id: row.try_get("id")?,
            markdown: row.try_get("markdown")?,
            preview: row.try_get("preview")?,
            published_at: row.try_get("published_at")?,
            revised_at: row.try_get("revised_at")?,
            slug: row.try_get("slug")?,
            title: row.try_get("title")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
