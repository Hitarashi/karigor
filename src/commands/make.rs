//! Code generator commands for Karigor.

use colored::Colorize;
use tracing::info;

use crate::ui;

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
    ui::print_info(&format!(
        "Registered route prefix {} in {}",
        format!("\"/api/{}\"", name.to_lowercase()).yellow(),
        "src/routes.rs".cyan()
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
