//! Embedded template stubs, customization ejection, and Jinja rendering for Karigor.

use std::{borrow::Cow, fs, path::Path};

use minijinja::{
    Environment,
    value::{Serde, Value as JinjaValue},
};
use serde_json::Value;

/// Raw embedded stubs shipped directly with Karigor binary.
pub const ROUTE_STUB: &str = include_str!("../stubs/route.stub");
pub const CONTROLLER_STUB: &str = include_str!("../stubs/controller.stub");
pub const MIDDLEWARE_STUB: &str = include_str!("../stubs/middleware.stub");
pub const COMMAND_STUB: &str = include_str!("../stubs/command.stub");

/// List of all standard stubs.
pub const ALL_STUBS: &[(&str, &str)] = &[
    ("route.stub", ROUTE_STUB),
    ("controller.stub", CONTROLLER_STUB),
    ("middleware.stub", MIDDLEWARE_STUB),
    ("command.stub", COMMAND_STUB),
];

/// Retrieves a template stub, prioritizing local `./stubs/<name>` over embedded defaults.
pub fn get_stub(name: &str) -> Cow<'static, str> {
    let local_path = Path::new("stubs").join(name);
    if let Ok(content) = fs::read_to_string(&local_path) {
        return Cow::Owned(content);
    }

    match name {
        "route.stub" => Cow::Borrowed(ROUTE_STUB),
        "controller.stub" => Cow::Borrowed(CONTROLLER_STUB),
        "middleware.stub" => Cow::Borrowed(MIDDLEWARE_STUB),
        "command.stub" => Cow::Borrowed(COMMAND_STUB),
        _ => Cow::Borrowed(""),
    }
}

/// Renders a template stub string with the provided JSON context using MiniJinja.
pub fn render_stub(template_str: &str, context: &Value) -> Result<String, minijinja::Error> {
    let mut env = Environment::new();
    env.add_template("stub", template_str)?;
    let template = env.get_template("stub")?;
    let ctx = JinjaValue::from(Serde(context));
    template.render(ctx)
}

/// Publishes all default stubs to the local `./stubs/` directory.
pub fn publish_stubs(
    target_dir: &Path,
    force: bool,
) -> Result<Vec<(String, bool)>, std::io::Error> {
    fs::create_dir_all(target_dir)?;

    let mut outcomes = Vec::new();
    for &(filename, content) in ALL_STUBS {
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
