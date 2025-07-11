pub mod home;
pub mod sitemap;

use crate::types::{AppError, SharedState};
use anyhow::{Context, Result, anyhow};
use axum::{
    extract::{Path, State},
    response::{Html, Redirect},
};
use chrono::{DateTime, Utc};
use comrak::{Arena, Options, nodes::NodeValue, parse_document};
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};
use tokio::fs::{self, DirEntry, read_dir};
use tracing::instrument;

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct PublishedPage {
    pub published_at: DateTime<Utc>,
    pub path: String,
    pub frontmatter: Frontmatter,
}

pub type PublishedPages = Vec<PublishedPage>;

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct Frontmatter {
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
            Option::map(a.split_once(":"), |(key, value)| {
                (key.trim().to_string(), value.trim().to_string())
            })
        })
        .collect::<HashMap<String, String>>();

    Ok(frontmatter)
}

#[instrument]
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

#[instrument]
async fn without_frontmatter(content_file: &String) -> Result<String> {
    let contents = fs::read(content_file).await?;
    let contents = String::from_utf8(contents)?;

    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.front_matter_delimiter = Some(String::from("---"));
    let nodes = parse_document(&arena, &contents, &options);

    for node in nodes.descendants() {
        if let NodeValue::FrontMatter(ref mut a) = node.data.borrow_mut().value {
            *a = String::new();
        }
    }

    let mut html = Vec::new();
    comrak::format_html(nodes, &options, &mut html)?;
    Ok(String::from_utf8(html)?)
}

#[instrument]
pub async fn published_pages() -> Result<PublishedPages> {
    let mut content_files = read_dir("./content").await?;

    let mut published_pages: PublishedPages = Vec::new();

    while let Some(content_file) = content_files.next_entry().await? {
        let frontmatter = extract_frontmatter_from_content_file(&content_file).await?;

        match frontmatter.published_at {
            None => {}
            Some(published_at) => {
                published_pages.push(PublishedPage {
                    published_at,
                    path: content_file
                        .path()
                        .to_str()
                        .ok_or(anyhow!("could not extract path as string"))?
                        .to_string(),
                    frontmatter,
                });
            }
        }
    }

    published_pages.sort_by(|a, b| {
        b.published_at
            .timestamp_micros()
            .cmp(&a.published_at.timestamp_micros())
    });

    Ok(published_pages)
}

pub async fn build_response(
    Path(slug): Path<String>,
    State(shared_state): State<Arc<SharedState>>,
) -> Result<Html<String>, AppError> {
    let tera = &shared_state.tera;
    let mut context = tera::Context::new();

    let published_pages = published_pages().await?;
    let published_page = published_pages
        .iter()
        .find(|f| f.frontmatter.slug == slug)
        .ok_or(anyhow!("could not find page"))?;

    let content = without_frontmatter(&published_page.path).await?;
    let description = published_page.frontmatter.description.clone();
    let published_at = published_page.published_at;
    let revised_at = published_page.frontmatter.revised_at;
    let mut title = published_page.frontmatter.title.clone();
    title.push_str(" - Cory Buecker");

    context.insert("content", &content);
    context.insert("description", &description);
    context.insert("title", &title);
    context.insert("published_at", &published_at);
    context.insert("revised_at", &revised_at);

    let rendered = tera
        .render("pages/page.html", &context)
        .map_err(|e| anyhow!("could not render template: {e}"))?;

    Ok(Html(rendered))
}

pub async fn remove_slash(Path(path_slug): Path<String>) -> Redirect {
    let mut redirect = String::from("/post/");
    redirect.push_str(&path_slug);

    Redirect::permanent(&redirect)
}
