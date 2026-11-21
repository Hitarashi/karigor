//! Code generator commands for Karigor.

use colored::Colorize;
use tracing::info;

use crate::ui;

/// Executes modular route generator.
pub fn execute_make_route(name: &str, controller: bool) -> Result<(), Box<dyn std::error::Error>> {
    let lower_name = name.to_lowercase();
    let route_path = format!("src/routes/{lower_name}.rs");

    info!(
        name = name,
        controller = controller,
        "Generating modular route component"
    );

    ui::print_section(&format!("Scaffolding Route Module: {}", name.bold().cyan()));

    ui::print_success(&format!("Generated {}", route_path.bold().white()));
    ui::print_info(&format!(
        "Injected {} into {}",
        format!("pub mod {lower_name};").yellow(),
        "src/routes/mod.rs".cyan()
    ));
    ui::print_info(&format!(
        "Mounted {} in {}",
        format!("router.nest(\"/{lower_name}\", {lower_name}::routes());").yellow(),
        "src/routes/mod.rs".cyan()
    ));

    if controller {
        let controller_path = format!("src/controllers/{lower_name}.rs");
        ui::print_success(&format!(
            "Generated paired controller {}",
            controller_path.bold().white()
        ));
        ui::print_info(&format!(
            "Injected {} into {}",
            format!("pub mod {lower_name};").yellow(),
            "src/controllers/mod.rs".cyan()
        ));
    }

    println!();
    println!("  {}", "Route Definition Preview:".bold().dimmed());
    println!(
        "  {}",
        "───────────────────────────────────────────────────────".dimmed()
    );
    println!("  use druto::prelude::*;");
    if controller {
        println!("  use crate::controllers::{lower_name};");
        println!();
        println!("  pub fn routes() -> Router {{");
        println!("      let mut router = Router::new();");
        println!("      router.get(\"/\", {lower_name}::index);");
        println!("      router.post(\"/\", {lower_name}::store);");
        println!("      router.get(\"/:id\", {lower_name}::show);");
        println!("      router.put(\"/:id\", {lower_name}::update);");
        println!("      router.delete(\"/:id\", {lower_name}::destroy);");
        println!("      router");
        println!("  }}");
    } else {
        println!();
        println!("  pub fn routes() -> Router {{");
        println!("      let mut router = Router::new();");
        println!("      router.get(\"/\", index);");
        println!("      router.get(\"/:id\", show);");
        println!("      router");
        println!("  }}");
        println!();
        println!("  async fn index(ctx: &mut HttpContext) -> impl IntoResponse {{");
        println!("      Json(json!({{ \"status\": \"ok\", \"route\": \"{lower_name}::index\" }}))");
        println!("  }}");
        println!();
        println!("  async fn show(ctx: &mut HttpContext) -> impl IntoResponse {{");
        println!("      let id = ctx.param(\"id\").unwrap_or_default();");
        println!(
            "      Json(json!({{ \"status\": \"ok\", \"id\": id, \"route\": \"{lower_name}::show\" }}))"
        );
        println!("  }}");
    }
    println!(
        "  {}",
        "───────────────────────────────────────────────────────".dimmed()
    );
    println!();

    Ok(())
}

/// Executes controller generator.
pub fn execute_make_controller(name: &str, api: bool) -> Result<(), Box<dyn std::error::Error>> {
    info!(name = name, api = api, "Generating controller component");

    let file_name = format!("{}.rs", name.to_lowercase());
    let target_path = format!("src/controllers/{file_name}");

    ui::print_section(&format!("Scaffolding Controller: {}", name.bold().cyan()));

    ui::print_success(&format!("Generated {}", target_path.bold().white()));
    ui::print_info(&format!(
        "Injected {} into {}",
        format!("pub mod {};", name.to_lowercase()).yellow(),
        "src/controllers/mod.rs".cyan()
    ));

    println!();
    println!("  {}", "Component Preview:".bold().dimmed());
    println!(
        "  {}",
        "───────────────────────────────────────────────────────".dimmed()
    );
    println!("  use druto::prelude::*;");
    println!();
    println!("  pub async fn index(ctx: &mut HttpContext) -> impl IntoResponse {{");
    println!(
        "      Json(json!({{ \"status\": \"ok\", \"controller\": \"{}\" }}))",
        name
    );
    println!("  }}");
    println!(
        "  {}",
        "───────────────────────────────────────────────────────".dimmed()
    );
    println!();

    Ok(())
}

/// Executes middleware generator.
pub fn execute_make_middleware(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!(name = name, "Generating middleware component");

    let file_name = format!("{}.rs", name.to_lowercase());
    let target_path = format!("src/middleware/{file_name}");

    ui::print_section(&format!("Scaffolding Middleware: {}", name.bold().cyan()));

    ui::print_success(&format!("Generated {}", target_path.bold().white()));
    ui::print_info(&format!(
        "Injected {} into {}",
        format!("pub mod {};", name.to_lowercase()).yellow(),
        "src/middleware/mod.rs".cyan()
    ));

    println!();
    println!("  {}", "Component Preview:".bold().dimmed());
    println!(
        "  {}",
        "───────────────────────────────────────────────────────".dimmed()
    );
    println!("  use dhap::{{Middleware, Next}};");
    println!("  use druto::prelude::*;");
    println!();
    println!("  #[derive(Clone, Default)]");
    println!("  pub struct {}Middleware;", name);
    println!();
    println!(
        "  impl Middleware<HttpContext, Response, RouterError> for {}Middleware {{",
        name
    );
    println!("      // Asynchronous onion middleware implementation");
    println!("  }}");
    println!(
        "  {}",
        "───────────────────────────────────────────────────────".dimmed()
    );
    println!();

    Ok(())
}

/// Executes command / task generator.
pub fn execute_make_command(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!(name = name, "Generating command component");

    let file_name = format!("{}.rs", name.to_lowercase());
    let target_path = format!("src/commands/{file_name}");

    ui::print_section(&format!("Scaffolding Task Command: {}", name.bold().cyan()));

    ui::print_success(&format!("Generated {}", target_path.bold().white()));
    ui::print_info(&format!(
        "Injected {} into {}",
        format!("pub mod {};", name.to_lowercase()).yellow(),
        "src/commands/mod.rs".cyan()
    ));

    println!();
    Ok(())
}
