use crate::types::{AppError, Page, SharedState};
use anyhow::{Context, anyhow};
use axum::{
    extract::{Path, State},
    response::{Html, Redirect},
};
use std::sync::Arc;

pub mod home;
pub mod sitemap;

pub async fn build_response(
    Path(slug): Path<String>,
    State(shared_state): State<Arc<SharedState>>,
) -> Result<Html<String>, AppError> {
    let tera = &shared_state.tera;
    let mut context = tera::Context::new();
    let client = shared_state.client.read().await;

    let page: Page = client
        .query_one(
            "SELECT * FROM pages WHERE published_at IS NOT NULL AND slug = $1",
            &[&slug],
        )
        .await
        .context("could not find page")?
        .try_into()?;

    context.insert("page", &page);
    context.insert("description", &page.description);
    let mut title = page.title.to_owned();
    title.push_str(" - Cory Buecker");
    context.insert("title", &title);

    let rendered = tera
        .render("pages/page.html", &context)
        .map_err(|e| anyhow!("could not render template: {}", e))?;

    Ok(Html(rendered))
}

pub async fn remove_slash(Path(path_slug): Path<String>) -> Redirect {
    let mut redirect = String::from("/post/");
    redirect.push_str(&path_slug);

    Redirect::permanent(&redirect)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utilities::tests::helpers::{
        cleanup_test_data, create_test_shared_state, random_slug, setup_test_data,
    };
    use chrono::Utc;

    #[tokio::test]
    async fn test_build_response_happy_path() -> anyhow::Result<()> {
        // Create a test shared state
        let shared_state = create_test_shared_state().await?;
        let client = shared_state.client.read().await;

        // Set up test data
        setup_test_data(&client).await?;

        // Insert a test page
        let now = Utc::now();
        let test_slug = random_slug("homepage");
        shared_state.client.read().await.execute(
            "INSERT INTO pages (title, slug, content, markdown, description, preview, created_at, updated_at, published_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            &[
                &"Test Page",
                &test_slug,
                &"<p>Test content</p>",
                &"Test content", 
                &"Test description",
                &"Test preview",
                &now,
                &now,
                &now
            ]
        ).await?;

        // Call the function under test
        let result = build_response(Path(test_slug.to_string()), State(shared_state.clone()))
            .await
            .unwrap();

        // Verify the result
        let html = result.0;
        assert!(html.contains("Test Page"));
        assert!(html.contains("Test content"));
        assert!(html.contains("Test Page - Cory Buecker")); // Check title format

        // Clean up test data
        cleanup_test_data(&client).await?;

        Ok(())
    }
}
