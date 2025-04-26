use anyhow::Result;
use include_dir::{Dir, include_dir};
use std::{collections::HashMap, time::SystemTime};
use tera::{Function, Tera, Value};

static TEMPLATES: Dir = include_dir!("templates");

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
    let templates = TEMPLATES
        .find("**/*.html")
        .unwrap()
        .map(|d| {
            (
                d.path().to_str().unwrap(),
                TEMPLATES
                    .get_file(d.path())
                    .unwrap()
                    .contents_utf8()
                    .unwrap(),
            )
        })
        .collect::<Vec<(&str, &str)>>();

    tera.add_raw_templates(templates).unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tera::{Context, Tera};

    #[test]
    fn test_digest_asset() {
        // Create a new Tera instance
        let mut tera = Tera::default();

        // Register the digest_asset function
        tera.register_function("digest_asset", digest_asset());

        // Add a simple template that uses the function
        tera.add_raw_template("test.html", "{{ digest_asset(file='js/app.js') | safe }}")
            .unwrap();

        // Render the template
        let result = tera.render("test.html", &Context::new()).unwrap();

        // Assert the result starts with the expected prefix
        assert!(result.starts_with("/assets/js/app.js?v="));

        // Assert the result contains a version parameter
        let parts: Vec<&str> = result.split("?v=").collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "/assets/js/app.js");

        // The version should be a numeric timestamp
        let version = parts[1];
        assert!(version.parse::<u64>().is_ok());
    }

    #[test]
    fn test_digest_asset_missing_file() {
        // Create a new Tera instance
        let mut tera = Tera::default();

        // Register the digest_asset function
        tera.register_function("digest_asset", digest_asset());

        // Add a template that calls the function without a file parameter
        tera.add_raw_template("test.html", "{{ digest_asset() }}")
            .unwrap();

        // Render the template, which should fail
        let result = tera.render("test.html", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_digest_asset_non_string_file() {
        // Create a new Tera instance
        let mut tera = Tera::default();

        // Register the digest_asset function
        tera.register_function("digest_asset", digest_asset());

        // Add a template that calls the function with a non-string file parameter
        tera.add_raw_template("test.html", "{{ digest_asset(file=42) }}")
            .unwrap();

        // Render the template, which should fail
        let result = tera.render("test.html", &Context::new());
        assert!(result.is_err());
    }
}
