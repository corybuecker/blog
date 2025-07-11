use anyhow::Result;
use std::{collections::HashMap, time::SystemTime};
use tera::{Function, Tera, Value};

static LAYOUT_TEMPLATE: &str = include_str!("../../templates/layout.html");
static PAGES_HOME_TEMPLATE: &str = include_str!("../../templates/pages/home.html");
static PAGES_PAGE_TEMPLATE: &str = include_str!("../../templates/pages/page.html");

pub fn digest_asset() -> impl Function {
    let key = SystemTime::now();
    let key = key
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("could not generate asset timestamp");
    let key = key.as_secs().to_string();

    move |args: &HashMap<String, Value>| -> tera::Result<Value> {
        match args.get("file") {
            Some(file) => {
                let mut path = "/assets/".to_string();

                let Some(file) = file.as_str() else {
                    return Err("".to_string().into());
                };

                path.push_str(file);
                path.push_str("?v=");
                path.push_str(&key);

                Ok(path.into())
            }
            None => Err("".to_string().into()),
        }
    }
}

pub fn embed_templates(tera: &mut Tera) -> Result<()> {
    let templates = vec![
        ("layout.html", LAYOUT_TEMPLATE),
        ("pages/home.html", PAGES_HOME_TEMPLATE),
        ("pages/page.html", PAGES_PAGE_TEMPLATE),
    ];
    tera.add_raw_templates(templates).unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tera::{Context, Tera};

    #[test]
    fn test_digest_asset() {
        let mut tera = Tera::default();
        tera.register_function("digest_asset", digest_asset());
        tera.add_raw_template("test.html", "{{ digest_asset(file='js/app.js') | safe }}")
            .unwrap();
        let result = tera.render("test.html", &Context::new()).unwrap();
        assert!(result.starts_with("/assets/js/app.js?v="));
        let parts: Vec<&str> = result.split("?v=").collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "/assets/js/app.js");
        let version = parts[1];
        assert!(version.parse::<u64>().is_ok());
    }

    #[test]
    fn test_digest_asset_missing_file() {
        let mut tera = Tera::default();
        tera.register_function("digest_asset", digest_asset());
        tera.add_raw_template("test.html", "{{ digest_asset() }}")
            .unwrap();
        let result = tera.render("test.html", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_digest_asset_non_string_file() {
        let mut tera = Tera::default();
        tera.register_function("digest_asset", digest_asset());
        tera.add_raw_template("test.html", "{{ digest_asset(file=42) }}")
            .unwrap();
        let result = tera.render("test.html", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_embed_templates_success() {
        let mut tera = Tera::default();
        let result = embed_templates(&mut tera);
        assert!(result.is_ok());
    }

    #[test]
    fn test_embed_templates_loads_templates() {
        let mut tera = Tera::default();
        embed_templates(&mut tera).unwrap();

        // Check that templates are accessible
        let template_names = tera.get_template_names().collect::<Vec<_>>();
        assert!(!template_names.is_empty());

        // Check for specific templates that should exist
        assert!(template_names.contains(&"layout.html"));
        assert!(template_names.contains(&"pages/home.html"));
        assert!(template_names.contains(&"pages/page.html"));
    }

    #[test]
    fn test_embed_templates_renders_correctly() {
        let mut tera = Tera::default();
        tera.register_function("digest_asset", digest_asset());

        embed_templates(&mut tera).unwrap();

        // Test that we can render a template (this assumes layout.html exists and is valid)
        let mut context = Context::new();
        context.insert("content", "content");
        context.insert("title", "title");
        let result = tera.render("layout.html", &context).unwrap();
        assert!(result.contains("<!DOCTYPE html>"));
    }
}
