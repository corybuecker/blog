pub mod home;
pub mod sitemap;

use crate::types::{AppError, Page, SharedState};
use anyhow::{Context, Result, anyhow};
use axum::{
    extract::{Path, State},
    response::{Html, Redirect},
};
use chrono::{DateTime, Utc};
use comrak::{Arena, Options, nodes::NodeValue, parse_document};
use std::{collections::HashMap, sync::Arc};
use tokio::fs::{self, DirEntry, read_dir};
use tracing::{debug, instrument};

#[derive(Debug)]
pub struct PublishedPage {
    pub published_at: DateTime<Utc>,
    pub path: String,
}

pub type PublishedPages = Vec<PublishedPage>;

#[derive(Debug)]
struct Frontmatter {
    pub description: String,
    pub preview: String,
    pub published_at: Option<DateTime<Utc>>,
    pub revised_at: Option<DateTime<Utc>>,
    pub slug: String,
    pub title: String,
}

impl Frontmatter {
    fn from_hashmap(map: HashMap<String, String>) -> Result<Self> {
        let description = map
            .get("description")
            .context("missing description")?
            .clone();
        let preview = map.get("preview").context("missing preview")?.clone();
        let slug = map.get("slug").context("missing slug")?.clone();
        let title = map.get("title").context("missing title")?.clone();

        let published_at = map
            .get("published_at")
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let revised_at = map
            .get("revised_at")
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        Ok(Frontmatter {
            description,
            preview,
            published_at,
            revised_at,
            slug,
            title,
        })
    }
}

fn frontmatter_to_hashmap(frontmatter_string: &str) -> Result<HashMap<String, String>> {
    let frontmatter = frontmatter_string.to_string();
    let frontmatter = frontmatter.replace("---\n", "").trim().to_string();
    let frontmatter = frontmatter
        .split("\n")
        .filter_map(|a| {
            if let Some((key, value)) = a.split_once(":") {
                Some((key.trim().to_string(), value.trim().to_string()))
            } else {
                None
            }
        })
        .collect::<HashMap<String, String>>();

    Ok(frontmatter)
}

async fn extract_frontmatter_from_content_file(content_file: &DirEntry) -> Result<Frontmatter> {
    let contents = fs::read(content_file.path()).await?;
    let contents = String::from_utf8(contents)?;

    let mut frontmatter = String::new();

    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.front_matter_delimiter = Some(String::from("---"));
    let nodes = parse_document(&arena, &contents, &options);

    for node in nodes.descendants() {
        if let NodeValue::FrontMatter(ref mut a) = node.data.borrow_mut().value {
            frontmatter = a.clone();
            *a = String::new();
        }
    }

    let frontmatter = frontmatter_to_hashmap(&frontmatter)?;
    Frontmatter::from_hashmap(frontmatter)
}

pub async fn published_pages() -> Result<PublishedPages> {
    let mut content_files = read_dir("./content").await?;

    let mut published_pages: Vec<(DateTime<Utc>, DirEntry)> = Vec::new();

    while let Some(content_file) = content_files.next_entry().await? {
        let frontmatter = extract_frontmatter_from_content_file(&content_file).await?;

        match frontmatter.published_at {
            None => {}
            Some(published_at) => {
                published_pages.push((published_at, content_file));
            }
        }
    }

    published_pages.sort_by(|a, b| b.0.timestamp_micros().cmp(&a.0.timestamp_micros()));

    let published_pages: PublishedPages = published_pages
        .iter()
        .filter_map(|(a, b)| match b.path().to_str() {
            Some(path) => Some(PublishedPage {
                published_at: a.clone(),
                path: path.to_string(),
            }),
            None => None,
        })
        .collect();

    Ok(published_pages)
}

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
