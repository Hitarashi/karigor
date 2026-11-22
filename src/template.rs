//! MiniJinja-backed template rendering engine for code scaffolding and generators.

use std::{fs, path::Path};

use borno::StrExt;
use minijinja::{
    Environment,
    value::{Serde, Value as JinjaValue},
};
use serde_json::Value;

use crate::error::Error;

/// Creates a pre-configured MiniJinja environment with string inflection and casing filters.
#[must_use]
pub fn create_environment() -> Environment<'static> {
    let mut env = Environment::new();
    env.add_filter("snake_case", |val: &str| val.to_snake_case().into_owned());
    env.add_filter("pascal_case", |val: &str| val.to_pascal_case().into_owned());
    env.add_filter("kebab_case", |val: &str| val.to_kebab_case().into_owned());
    env.add_filter("plural", |val: &str| val.to_plural());
    env.add_filter("singular", |val: &str| val.to_singular());
    env
}

/// Renders a raw template string using the provided JSON context data.
pub fn render(template_str: &str, context: &Value) -> Result<String, Error> {
    let mut env = create_environment();
    env.add_template("template", template_str)?;
    let template = env.get_template("template")?;
    let ctx = JinjaValue::from(Serde(context));
    let rendered = template.render(ctx)?;
    Ok(rendered)
}

/// Publishes a collection of embedded templates to a target directory on disk.
pub fn publish_files(
    target_dir: &Path,
    files: &[(&str, &str)],
    force: bool,
) -> Result<Vec<(String, bool)>, std::io::Error> {
    fs::create_dir_all(target_dir)?;

    let mut outcomes = Vec::new();
    for &(filename, content) in files {
        let dest = target_dir.join(filename);
        if dest.exists() && !force {
            outcomes.push((filename.to_string(), false));
        } else {
            fs::write(&dest, content)?;
            outcomes.push((filename.to_string(), true));
        }
    }

    Ok(outcomes)
}
