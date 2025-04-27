use crate::types::{AppError, Page, SharedState};
use anyhow::Context;
use axum::{extract::State, response::Html};
use serde::Serialize;
use std::{collections::VecDeque, sync::Arc};

#[derive(Serialize, Debug)]
struct Link {
    title: String,
    slug: String,
}

pub async fn build_response(
    State(shared_state): State<Arc<SharedState>>,
) -> Result<Html<String>, AppError> {
    let tera = &shared_state.tera;
    let client = shared_state.client.read().await;
    let pages = client
        .query(
            "SELECT title, slug FROM pages WHERE published_at IS NOT NULL ORDER BY published_at DESC",
            &[],
        )
        .await
        .context("could not fetch pages")?;
    let homepage: Page = client
        .query_one(
            "SELECT * FROM pages WHERE published_at IS NOT NULL ORDER BY published_at DESC LIMIT 1",
            &[],
        )
        .await
        .context("could not fetch homepage")?
        .try_into()?;

    let mut context = tera::Context::new();
    let mut pages: VecDeque<Link> = pages
        .into_iter()
        .map(|row| Link {
            title: row.get("title"),
            slug: row.get("slug"),
        })
        .collect();

    pages.pop_front();

    context.insert("pages", &pages);
    context.insert("homepage", &homepage);

    context.insert("description", &homepage.description);
    let mut title = homepage.title.to_owned();
    title.push_str(" - Cory Buecker");
    context.insert("title", &title);

    let rendered = tera
        .render("pages/home.html", &context)
        .context("could not render template")?;

    Ok(Html(rendered))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utilities::tests::helpers::{
        cleanup_test_data, create_test_shared_state, random_slug, setup_test_data,
    };
    use chrono::{Days, Utc};

    #[tokio::test]
    async fn test_build_response_happy_path() -> anyhow::Result<()> {
        // Create a test shared state
        let shared_state = create_test_shared_state().await?;
        let client = shared_state.client.read().await;
        // Set up test data
        setup_test_data(&client).await?;

        // Insert a test page that will be used as the homepage
        let now = Utc::now();
        let day_ago = Utc::now().checked_sub_days(Days::new(1)).unwrap();

        shared_state.client.read().await.execute(
            "INSERT INTO pages (title, slug, content, markdown, description, preview, created_at, updated_at, published_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            &[
                &"Test Homepage",
                &random_slug("homepage"),
                &"<p>Test content</p>",
                &"Test content",
                &"Test description",
                &"Test preview",
                &now,
                &now,
                &now
            ]
        ).await?;

        // Insert another test page that will be in the pages list
        shared_state.client.read().await.execute(
            "INSERT INTO pages (title, slug, content, markdown, description, preview, created_at, updated_at, published_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            &[
                &"Test Page",
                &random_slug("nonhomepage"),
                &"<p>Another test content</p>",
                &"Another test content",
                &"Another test description",
                &"Another test preview",
                &day_ago,
                &day_ago,
                &day_ago
            ]
        ).await?;

        // Call the function under test
        let result = build_response(State(shared_state.clone())).await.unwrap();

        // Verify the result
        let html = result.0;
        assert!(html.contains("Test Homepage"));
        assert!(html.contains("Test Page"));

        // Clean up test data
        cleanup_test_data(&client).await?;

        Ok(())
    }
}
