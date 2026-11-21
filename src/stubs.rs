//! Embedded template stubs, customization ejection, and Jinja rendering for Karigor.

use std::{borrow::Cow, fs, path::Path};

use minijinja::{
    Environment,
    value::{Serde, Value as JinjaValue},
};
use serde_json::Value;

/// Raw embedded route stub compiled into the Karigor binary.
pub const ROUTE_STUB: &str = r#"use druto::prelude::*;
{% if has_controller %}
use crate::controllers::{{ name_snake }};

pub fn routes() -> Router {
    let mut router = Router::new();
    router.get("/", {{ name_snake }}::index);
    router.post("/", {{ name_snake }}::store);
    router.get("/:id", {{ name_snake }}::show);
    router.put("/:id", {{ name_snake }}::update);
    router.delete("/:id", {{ name_snake }}::destroy);
    router
}
{% else %}
pub fn routes() -> Router {
    let mut router = Router::new();
    router.get(
        "/",
        from_handler(|ctx| {
            Box::pin(async move {
                ctx.response.text("Route: {{ name_snake }}::index\n");
                Ok(())
            })
        }),
    );
    router.get(
        "/:id",
        from_handler(|ctx| {
            Box::pin(async move {
                let id = ctx.request.param("id").unwrap_or_default();
                ctx.response.text(format!("Route: {{ name_snake }}::show ({id})\n"));
                Ok(())
            })
        }),
    );
    router
}
{% endif %}
"#;

/// Raw embedded controller stub compiled into the Karigor binary.
pub const CONTROLLER_STUB: &str = r#"use druto::prelude::*;
use druto::router::BoxFuture;

/// Renders the {{ name_plural }} overview.
pub fn index<'a>(ctx: &'a mut HttpContext) -> BoxFuture<'a, Result<(), RouterError>> {
    Box::pin(async move {
        ctx.response.html("<h1>{{ name_pascal }} Controller</h1>\n");
        Ok(())
    })
}

/// Creates a new {{ name_singular }}.
pub fn store<'a>(ctx: &'a mut HttpContext) -> BoxFuture<'a, Result<(), RouterError>> {
    Box::pin(async move {
        ctx.response.set_status(StatusCode::CREATED);
        Ok(())
    })
}

/// Renders the {{ name_singular }} detail view.
pub fn show<'a>(ctx: &'a mut HttpContext) -> BoxFuture<'a, Result<(), RouterError>> {
    Box::pin(async move {
        let id = ctx.request.param("id").unwrap_or_default();
        ctx.response.html(format!("<h1>{{ name_pascal }} Detail: {id}</h1>\n"));
        Ok(())
    })
}

/// Updates an existing {{ name_singular }}.
pub fn update<'a>(ctx: &'a mut HttpContext) -> BoxFuture<'a, Result<(), RouterError>> {
    Box::pin(async move {
        ctx.response.set_status(StatusCode::NO_CONTENT);
        Ok(())
    })
}

/// Deletes an existing {{ name_singular }}.
pub fn destroy<'a>(ctx: &'a mut HttpContext) -> BoxFuture<'a, Result<(), RouterError>> {
    Box::pin(async move {
        ctx.response.set_status(StatusCode::NO_CONTENT);
        Ok(())
    })
}
"#;

/// Raw embedded middleware stub compiled into the Karigor binary.
pub const MIDDLEWARE_STUB: &str = r#"use druto::prelude::*;

/// Custom onion middleware: {{ name_pascal }}.
pub struct {{ name_pascal }};

impl Middleware for {{ name_pascal }} {
    async fn handle(&self, ctx: &mut HttpContext, next: Next<'_>) -> Result<Response, Error> {
        let start = std::time::Instant::now();

        let response = next.run(ctx).await?;

        let duration = start.elapsed();
        tracing::debug!(
            middleware = "{{ name_pascal }}",
            elapsed_micros = duration.as_micros(),
            "Middleware execution completed"
        );

        Ok(response)
    }
}
"#;

/// Raw embedded command stub compiled into the Karigor binary.
pub const COMMAND_STUB: &str = r#"use tracing::info;

/// Custom console command or background task: {{ name_pascal }}.
pub async fn execute() -> Result<(), Box<dyn std::error::Error>> {
    info!(command = "{{ name_pascal }}", "Executing custom command");
    println!("Executing {{ name_pascal }} command...");
    Ok(())
}
"#;

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
