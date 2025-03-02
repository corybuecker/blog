use super::types::PostForm;
use crate::types::{AppError, Page, SharedState};
use anyhow::Context;
use axum::extract::Path;
use axum::{
    Form,
    extract::State,
    response::{Html, Redirect},
};
use chrono::NaiveDate;
use comrak::Options;
use comrak::Plugins;
use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::markdown_to_html;
use comrak::markdown_to_html_with_plugins;
use mongodb::Collection;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use std::collections::HashMap;
use std::io::{self, Write};
use std::str::FromStr;
use tracing::debug;

pub async fn index(State(state): State<SharedState>) -> Result<Html<String>, AppError> {
    let collection: Collection<Page> = state.mongo.database("blog").collection("pages");

    let mut cursor = collection.find(doc! {}).await.context("database error")?;

    let mut pages: Vec<Page> = Vec::new();

    while cursor.advance().await.context("database error")? {
        let mut page = cursor.deserialize_current().context("database error")?;
        page.id = Some(page._id.to_hex());
        pages.push(page)
    }

    let mut context = tera::Context::new();
    context.insert("pages", &pages);

    let rendered = state
        .tera
        .render("admin/index.html", &context)
        .context("could not render template")?;

    Ok(Html(rendered))
}

pub async fn new(State(state): State<SharedState>) -> Result<Html<String>, AppError> {
    let rendered = state
        .tera
        .render("admin/new.html", &tera::Context::new())
        .context("could not render template")?;

    Ok(Html(rendered))
}

pub async fn edit(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Html<String>, AppError> {
    let tera = &state.tera;
    let database: &mongodb::Database = &state.mongo.database("blog");
    let mut context = tera::Context::new();
    let oid = ObjectId::from_str(&id).context("invalid ID")?;

    let collection: Collection<Page> = database.collection("pages");
    let mut page = collection
        .find_one(doc! {"_id": oid})
        .await
        .context("database error")?
        .context("could not find page")?;

    page.id = Some(page._id.to_string());

    context.insert("page", &page);
    let rendered = tera
        .render("admin/edit.html", &context)
        .context("error rendering template")?;

    Ok(Html(rendered))
}

pub async fn create(
    State(shared_state): State<SharedState>,
    Form(form): Form<PostForm>,
) -> Result<Redirect, AppError> {
    let collection = shared_state.mongo.database("blog").collection("pages");

    let new_page = Page {
        _id: mongodb::bson::oid::ObjectId::new(),
        markdown: markdown_to_html(&form.content, &Options::default()),
        content: form.content,
        created_at: mongodb::bson::DateTime::now(),
        description: form.description,
        id: None,
        preview: form.preview,
        published_at: None,
        revised_at: None,
        slug: form.slug,
        title: form.title,
        updated_at: mongodb::bson::DateTime::now(),
    };

    let _result = collection
        .insert_one(new_page)
        .await
        .context("could not create page")?;

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
        write!(output, "{}", code)
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
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Form(form): Form<PostForm>,
) -> Result<Redirect, AppError> {
    let database: &mongodb::Database = &state.mongo.database("blog");
    let collection: Collection<Page> = database.collection("pages");
    let oid = ObjectId::from_str(&id).context("invalid ID")?;

    let mut page = collection
        .find_one(doc! {"_id": oid})
        .await
        .context("database error")?
        .context("could not find page")?;

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

    let _result = collection.replace_one(doc! {"_id": oid}, page).await;

    Ok(Redirect::to("/admin/pages"))
}
