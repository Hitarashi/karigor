//! Route inspection and endpoint visualization for Karigor.

use colored::Colorize;
use comfy_table::Cell;
use serde::Serialize;
use tracing::info;

use crate::ui;

#[derive(Serialize)]
struct RouteInfo {
    method: &'static str,
    path: &'static str,
    handler: &'static str,
    middlewares: &'static [&'static str],
}

const SAMPLE_ROUTES: &[RouteInfo] = &[
    RouteInfo {
        method: "GET",
        path: "/",
        handler: "app::handlers::home",
        middlewares: &["cors"],
    },
    RouteInfo {
        method: "GET",
        path: "/api/health",
        handler: "app::handlers::health_check",
        middlewares: &["cors"],
    },
    RouteInfo {
        method: "POST",
        path: "/api/v1/auth/login",
        handler: "app::handlers::auth::login",
        middlewares: &["cors", "limiter", "bodyparser"],
    },
    RouteInfo {
        method: "POST",
        path: "/api/v1/auth/logout",
        handler: "app::handlers::auth::logout",
        middlewares: &["cors", "session", "shield::csrf"],
    },
    RouteInfo {
        method: "GET",
        path: "/api/v1/users",
        handler: "app::handlers::users::index",
        middlewares: &["cors", "session", "limiter"],
    },
    RouteInfo {
        method: "POST",
        path: "/api/v1/users",
        handler: "app::handlers::users::create",
        middlewares: &["cors", "session", "shield::csrf", "bodyparser"],
    },
    RouteInfo {
        method: "GET",
        path: "/api/v1/users/:id",
        handler: "app::handlers::users::show",
        middlewares: &["cors", "session"],
    },
    RouteInfo {
        method: "PUT",
        path: "/api/v1/users/:id",
        handler: "app::handlers::users::update",
        middlewares: &["cors", "session", "shield::csrf", "bodyparser"],
    },
    RouteInfo {
        method: "DELETE",
        path: "/api/v1/users/:id",
        handler: "app::handlers::users::destroy",
        middlewares: &["cors", "session", "shield::csrf"],
    },
];

/// Formats and displays the compiled application routes.
pub fn execute_routes(
    filter: Option<String>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(filter = ?filter, json = json, "Inspecting compiled application routes");

    let filtered: Vec<&RouteInfo> = match &filter {
        Some(query) => {
            let lower = query.to_lowercase();
            SAMPLE_ROUTES
                .iter()
                .filter(|route| {
                    route.path.to_lowercase().contains(&lower)
                        || route.method.to_lowercase().contains(&lower)
                        || route.handler.to_lowercase().contains(&lower)
                })
                .collect()
        }
        None => SAMPLE_ROUTES.iter().collect(),
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&filtered)?);
        return Ok(());
    }

    ui::print_banner(
        "Application Routes",
        "Compiled route dispatcher and middleware registry",
    );

    let mut table = ui::create_table(&["Method", "URI Path", "Handler Action", "Middlewares"]);

    for route in &filtered {
        let middleware_display = if route.middlewares.is_empty() {
            "none".to_string()
        } else {
            route.middlewares.join(", ")
        };

        table.add_row(vec![
            ui::cell_http_method(route.method),
            Cell::new(route.path)
                .fg(comfy_table::Color::Cyan)
                .add_attribute(comfy_table::Attribute::Bold),
            Cell::new(route.handler).fg(comfy_table::Color::White),
            Cell::new(middleware_display).fg(comfy_table::Color::DarkGrey),
        ]);
    }

    println!("{table}");
    println!(
        "\n  {} Showing {} route endpoints.\n",
        "◆".cyan(),
        filtered.len().to_string().bold().green()
    );

    Ok(())
}
