use include_dir::{include_dir, Dir};
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

pub fn embed_templates(tera: &mut Tera) -> Result<(), Box<dyn std::error::Error>> {
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
