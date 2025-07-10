use super::types::PostForm;
use crate::types::{AppError, Page, SharedState};
use anyhow::{Context, anyhow};
use axum::{
    Form,
    extract::{Path, State},
    response::{Html, Redirect},
};
use chrono::{NaiveDate, Utc};
use comrak::markdown_to_html;
use comrak::{Options, Plugins};
use comrak::{adapters::SyntaxHighlighterAdapter, markdown_to_html_with_plugins};
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::Arc;

pub async fn index(State(state): State<Arc<SharedState>>) -> Result<Html<String>, AppError> {
    let client = state.client.read().await;
    let pages = Page::all(&client).await?;

    let mut context = tera::Context::new();
    context.insert("title", "Admin");
    context.insert("pages", &pages);

    let rendered = state
        .tera
        .render("admin/index.html", &context)
        .map_err(|e| anyhow!("{}", e))?;

    Ok(Html(rendered))
}

pub async fn new(State(state): State<Arc<SharedState>>) -> Result<Html<String>, AppError> {
    let mut context = tera::Context::new();
    context.insert("title", "Admin");
    let rendered = state
        .tera
        .render("admin/new.html", &context)
        .context("could not render template")?;

    Ok(Html(rendered))
}

pub async fn edit(
    State(state): State<Arc<SharedState>>,
    Path(id): Path<i32>,
) -> Result<Html<String>, AppError> {
    let tera = &state.tera;
    let client = state.client.read().await;
    let page: Page = client
        .query_one("SELECT * FROM pages WHERE id = $1 LIMIT 1", &[&id])
        .await
        .context("could not find page")?
        .try_into()?;

    let mut context = tera::Context::new();
    context.insert("title", "Admin");
    context.insert("page", &page);
    let rendered = tera
        .render("admin/edit.html", &context)
        .context("error rendering template")?;

    Ok(Html(rendered))
}

pub async fn create(
    State(state): State<Arc<SharedState>>,
    Form(form): Form<PostForm>,
) -> Result<Redirect, AppError> {
    let new_page = Page {
        markdown: markdown_to_html(&form.content, &Options::default()),
        content: form.content,
        created_at: Utc::now(),
        description: form.description,
        id: None,
        preview: form.preview,
        published_at: None,
        revised_at: None,
        slug: form.slug,
        title: form.title,
        updated_at: Utc::now(),
    };

    let client = state.client.read().await;

    client.execute(
        "INSERT INTO pages (content, created_at, description, markdown, preview, slug, title, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        &[
            &new_page.content,
            &new_page.created_at,
            &new_page.description,
            &new_page.markdown,
            &new_page.preview,
            &new_page.slug,
            &new_page.title,
            &new_page.updated_at
        ]
    ).await.map_err(|e| anyhow!("could not create page: {}", e))?;

    Ok(Redirect::to("/admin/pages"))
}

#[derive(Debug, Copy, Clone)]
pub struct SyntaxAdapter {}

impl SyntaxAdapter {
    pub fn new() -> Self {
        SyntaxAdapter {}
    }
}

impl SyntaxHighlighterAdapter for SyntaxAdapter {
    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        _lang: Option<&str>,
        code: &str,
    ) -> io::Result<()> {
        write!(output, "{code}")
    }

    fn write_pre_tag(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<String, String>,
    ) -> io::Result<()> {
        if attributes.contains_key("lang") {
            write!(
                output,
                "<pre class=\"not-prose\" lang=\"{}\">",
                attributes["lang"]
            )
        } else {
            output.write_all(b"<pre class=\"not-prose\">")
        }
    }

    fn write_code_tag(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<String, String>,
    ) -> io::Result<()> {
        if attributes.contains_key("class") {
            write!(output, "<code class=\"not-prose {}\">", attributes["class"])
        } else {
            output.write_all(b"<code>")
        }
    }
}

pub async fn update(
    State(state): State<Arc<SharedState>>,
    Path(id): Path<i32>,
    Form(form): Form<PostForm>,
) -> Result<Redirect, AppError> {
    let client = state.client.read().await;

    let mut page: Page = client
        .query_one("SELECT * FROM pages WHERE id = $1 LIMIT 1", &[&id])
        .await
        .context("could not find page")?
        .try_into()?;

    if let Some(date) = form.published_at {
        if let Ok(date) = NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
            page.published_at = Some(date.and_hms_opt(0, 0, 0).unwrap().and_utc())
        }
    }

    if let Some(date) = form.revised_at {
        if let Ok(date) = NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
            page.revised_at = Some(date.and_hms_opt(0, 0, 0).unwrap().and_utc())
        }
    }

    let adapter = SyntaxAdapter::new();
    let options = Options::default();
    let mut plugins = Plugins::default();

    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    page.markdown = markdown_to_html_with_plugins(&form.content, &options, &plugins);
    page.content = form.content;
    page.description = form.description;
    page.title = form.title;
    page.preview = form.preview;
    page.updated_at = Utc::now();

    client
        .execute(
            "UPDATE pages SET content = $1, description = $2, markdown = $3, preview = $4,
        slug = $5, title = $6, updated_at = $7, published_at = $8, revised_at = $9 WHERE id = $10",
            &[
                &page.content,
                &page.description,
                &page.markdown,
                &page.preview,
                &page.slug,
                &page.title,
                &page.updated_at,
                &page.published_at,
                &page.revised_at,
                &id,
            ],
        )
        .await
        .map_err(|e| anyhow!("could not create page: {}", e))?;

    Ok(Redirect::to("/admin/pages"))
}
