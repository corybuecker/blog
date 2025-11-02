pub mod home;
pub mod page;
pub mod sitemap;

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use comrak::options::Plugins;
use comrak::{
    Arena, Options, adapters::SyntaxHighlighterAdapter, nodes::NodeValue, parse_document,
};
use serde::Serialize;
use std::borrow::Cow;
use std::fmt::{self, Write};
use std::{collections::HashMap, pin::Pin};
use tokio::fs::{self, read_dir};
use tracing::instrument;

#[derive(Debug, Serialize, Clone)]
pub struct PublishedPage {
    pub published_at: DateTime<Utc>,
    pub path: String,
    pub frontmatter: Frontmatter,
}

#[derive(Debug, Serialize, Clone)]
pub struct Frontmatter {
    pub description: String,
    pub preview: String,
    pub published_at: Option<DateTime<Utc>>,
    pub revised_at: Option<DateTime<Utc>>,
    pub slug: String,
    pub title: String,
}

#[derive(Default)]
pub struct PublishedPages {
    pages: Vec<PublishedPage>,
}

pub trait PublicationManager: Send + Sync {
    fn get_all(&self) -> Result<Vec<PublishedPage>>;

    fn publish<'f>(&'f mut self)
    -> Pin<Box<dyn Future<Output = Result<usize>> + Send + Sync + 'f>>;

    fn read<'f>(
        &'f self,
        path: &'f str,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + Sync + 'f>>;
}

impl PublicationManager for PublishedPages {
    fn get_all(&self) -> Result<Vec<PublishedPage>> {
        Ok(self.pages.clone())
    }

    fn publish<'f>(
        &'f mut self,
    ) -> Pin<Box<dyn Future<Output = Result<usize>> + Send + Sync + 'f>> {
        Box::pin(async {
            let published_pages = published_pages().await?;
            let length = published_pages.len();

            self.pages = published_pages;

            Ok(length)
        })
    }

    fn read<'f>(
        &'f self,
        path: &'f str,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + Sync + 'f>> {
        Box::pin(read_content_from_path(path))
    }
}

#[instrument]
async fn read_content_from_path(path: &str) -> Result<String> {
    let content = fs::read(path)
        .await
        .map_err(|e| anyhow!("could not read file: {}", e))?;

    String::from_utf8(content).map_err(|e| anyhow!("could not read file: {}", e))
}

impl Frontmatter {
    #[instrument]
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

#[instrument]
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
async fn extract_frontmatter_from_content_file(content: &str) -> Result<Frontmatter> {
    let mut frontmatter = String::new();
    let arena = Arena::new();

    let mut options = Options::default();
    options.extension.front_matter_delimiter = Some(String::from("---"));

    let nodes = parse_document(&arena, content, &options);

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
async fn without_frontmatter(content: &str) -> Result<String> {
    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.front_matter_delimiter = Some(String::from("---"));

    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&(SyntaxAdapter {}));

    let nodes = parse_document(&arena, content, &options);

    for node in nodes.descendants() {
        if let NodeValue::FrontMatter(ref mut a) = node.data.borrow_mut().value {
            *a = String::new();
        }
    }

    let mut html = String::new();
    comrak::format_html_with_plugins(nodes, &options, &mut html, &plugins)?;
    Ok(html)
}

#[instrument]
async fn published_pages() -> Result<Vec<PublishedPage>> {
    let mut content_files = read_dir("./content").await?;

    let mut published_pages: Vec<PublishedPage> = Vec::new();

    while let Some(content_file) = content_files.next_entry().await? {
        let content = fs::read(content_file.path()).await?;
        let content = String::from_utf8(content)?;
        let frontmatter = extract_frontmatter_from_content_file(&content).await?;

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

#[derive(Debug, Copy, Clone)]
pub struct SyntaxAdapter;

impl SyntaxHighlighterAdapter for SyntaxAdapter {
    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        _lang: Option<&str>,
        code: &str,
    ) -> fmt::Result {
        output.write_str(code)
    }

    fn write_pre_tag(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<&'static str, Cow<str>>,
    ) -> fmt::Result {
        if attributes.contains_key("lang") {
            write!(
                output,
                "<pre class=\"not-prose\" lang=\"{}\">",
                attributes["lang"]
            )
        } else {
            output.write_str("<pre class=\"not-prose\">")
        }
    }

    fn write_code_tag(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<&'static str, Cow<str>>,
    ) -> fmt::Result {
        if attributes.contains_key("class") {
            output.write_str(&format!("<code class=\"{}\">", attributes["class"]))
        } else {
            output.write_str("<code>")
        }
    }
}
