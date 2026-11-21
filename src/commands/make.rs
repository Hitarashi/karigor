//! Code generator commands for Karigor with MiniJinja template rendering and smart module injection.

use std::{fs, path::Path};

use borno::StrExt;
use colored::Colorize;
use serde_json::json;
use tracing::info;

use crate::{injector, stubs, ui};

/// Executes modular route generator.
pub fn execute_make_route(
    name: &str,
    controller: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let name_snake = name.to_snake_case().into_owned();
    let name_pascal = name.to_pascal_case().into_owned();
    let plural_str = name.to_plural();
    let name_plural = plural_str.to_snake_case().into_owned();
    let singular_str = name.to_singular();
    let name_singular = singular_str.to_snake_case().into_owned();

    info!(
        name = name,
        controller = controller,
        dry_run = dry_run,
        "Generating modular route component"
    );

    ui::print_section(&format!("Scaffolding Route Module: {}", name.bold().cyan()));

    let context = json!({
        "name": name,
        "name_snake": name_snake,
        "name_pascal": name_pascal,
        "name_singular": name_singular,
        "name_plural": name_plural,
        "has_controller": controller,
    });

    let route_template = stubs::get_stub("route.stub");
    let rendered_route = stubs::render_stub(&route_template, &context)?;

    let route_path = format!("src/routes/{name_snake}.rs");
    let routes_mod_path = "src/routes/mod.rs";

    if !dry_run {
        let route_file = Path::new(&route_path);
        if let Some(parent) = route_file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(route_file, &rendered_route)?;
        format_file(route_file);

        let outcome = injector::inject_route_module(Path::new(routes_mod_path), &name_snake)?;

        ui::print_success(&format!("Generated {}", route_path.bold().white()));
        if outcome.module_injected {
            ui::print_info(&format!(
                "Injected {} into {}",
                format!("pub mod {name_snake};").yellow(),
                routes_mod_path.cyan()
            ));
        } else {
            ui::print_info(&format!(
                "Module {} already present in {}",
                format!("pub mod {name_snake};").yellow(),
                routes_mod_path.cyan()
            ));
        }

        if outcome.route_mounted {
            ui::print_info(&format!(
                "Mounted {} in {}",
                format!("router.nest(\"/{name_snake}\", {name_snake}::routes());").yellow(),
                routes_mod_path.cyan()
            ));
        } else {
            ui::print_info(&format!(
                "Route {} already mounted in {}",
                format!("\"/{name_snake}\"").yellow(),
                routes_mod_path.cyan()
            ));
        }
        format_file(Path::new(routes_mod_path));
    } else {
        ui::print_info(&format!(
            "[dry-run] Would generate {}",
            route_path.bold().white()
        ));
        ui::print_info(&format!(
            "[dry-run] Would inject into {}",
            routes_mod_path.cyan()
        ));
    }

    if controller {
        let controller_context = json!({
            "name": name,
            "name_snake": name_snake,
            "name_pascal": name_pascal,
            "name_singular": name_singular,
            "name_plural": name_plural,
            "api": false,
        });

        let controller_template = stubs::get_stub("controller.stub");
        let rendered_controller = stubs::render_stub(&controller_template, &controller_context)?;

        let controller_path = format!("src/controllers/{name_snake}.rs");
        let controllers_mod_path = "src/controllers/mod.rs";

        if !dry_run {
            let controller_file = Path::new(&controller_path);
            if let Some(parent) = controller_file.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(controller_file, &rendered_controller)?;
            format_file(controller_file);

            let injected =
                injector::inject_controller_module(Path::new(controllers_mod_path), &name_snake)?;

            ui::print_success(&format!(
                "Generated paired controller {}",
                controller_path.bold().white()
            ));
            if injected {
                ui::print_info(&format!(
                    "Injected {} into {}",
                    format!("pub mod {name_snake};").yellow(),
                    controllers_mod_path.cyan()
                ));
            } else {
                ui::print_info(&format!(
                    "Controller module {} already present in {}",
                    format!("pub mod {name_snake};").yellow(),
                    controllers_mod_path.cyan()
                ));
            }
            format_file(Path::new(controllers_mod_path));
        } else {
            ui::print_info(&format!(
                "[dry-run] Would generate controller {}",
                controller_path.bold().white()
            ));
        }
    }

    println!();
    println!("  {}", "Route Definition Preview:".bold().dimmed());
    println!(
        "  {}",
        "───────────────────────────────────────────────────────".dimmed()
    );
    for line in rendered_route.lines() {
        println!("  {line}");
    }
    println!(
        "  {}",
        "───────────────────────────────────────────────────────".dimmed()
    );
    println!();

    Ok(())
}

/// Executes controller generator.
pub fn execute_make_controller(
    name: &str,
    api: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let name_snake = name.to_snake_case().into_owned();
    let name_pascal = name.to_pascal_case().into_owned();
    let plural_str = name.to_plural();
    let name_plural = plural_str.to_snake_case().into_owned();
    let singular_str = name.to_singular();
    let name_singular = singular_str.to_snake_case().into_owned();

    info!(
        name = name,
        api = api,
        dry_run = dry_run,
        "Generating controller component"
    );

    let context = json!({
        "name": name,
        "name_snake": name_snake,
        "name_pascal": name_pascal,
        "name_singular": name_singular,
        "name_plural": name_plural,
        "api": api,
    });

    let template = stubs::get_stub("controller.stub");
    let rendered = stubs::render_stub(&template, &context)?;

    let target_path = format!("src/controllers/{name_snake}.rs");
    let mod_path = "src/controllers/mod.rs";

    ui::print_section(&format!("Scaffolding Controller: {}", name.bold().cyan()));

    if !dry_run {
        let file = Path::new(&target_path);
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file, &rendered)?;
        format_file(file);

        let injected = injector::inject_controller_module(Path::new(mod_path), &name_snake)?;

        ui::print_success(&format!("Generated {}", target_path.bold().white()));
        if injected {
            ui::print_info(&format!(
                "Injected {} into {}",
                format!("pub mod {name_snake};").yellow(),
                mod_path.cyan()
            ));
        } else {
            ui::print_info(&format!(
                "Controller module {} already present in {}",
                format!("pub mod {name_snake};").yellow(),
                mod_path.cyan()
            ));
        }
        format_file(Path::new(mod_path));
    } else {
        ui::print_info(&format!(
            "[dry-run] Would generate {}",
            target_path.bold().white()
        ));
        ui::print_info(&format!("[dry-run] Would inject into {}", mod_path.cyan()));
    }

    println!();
    println!("  {}", "Component Preview:".bold().dimmed());
    println!(
        "  {}",
        "───────────────────────────────────────────────────────".dimmed()
    );
    for line in rendered.lines() {
        println!("  {line}");
    }
    println!(
        "  {}",
        "───────────────────────────────────────────────────────".dimmed()
    );
    println!();

    Ok(())
}

/// Executes middleware generator.
pub fn execute_make_middleware(
    name: &str,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let name_snake = name.to_snake_case();
    let name_pascal = name.to_pascal_case();

    info!(
        name = name,
        dry_run = dry_run,
        "Generating middleware component"
    );

    let context = json!({
        "name": name,
        "name_snake": name_snake,
        "name_pascal": name_pascal,
    });

    let template = stubs::get_stub("middleware.stub");
    let rendered = stubs::render_stub(&template, &context)?;

    let target_path = format!("src/middleware/{name_snake}.rs");
    let mod_path = "src/middleware/mod.rs";

    ui::print_section(&format!("Scaffolding Middleware: {}", name.bold().cyan()));

    if !dry_run {
        let file = Path::new(&target_path);
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file, &rendered)?;
        format_file(file);

        let injected = injector::inject_controller_module(Path::new(mod_path), &name_snake)?;

        ui::print_success(&format!("Generated {}", target_path.bold().white()));
        if injected {
            ui::print_info(&format!(
                "Injected {} into {}",
                format!("pub mod {name_snake};").yellow(),
                mod_path.cyan()
            ));
        }
        format_file(Path::new(mod_path));
    } else {
        ui::print_info(&format!(
            "[dry-run] Would generate {}",
            target_path.bold().white()
        ));
    }

    println!();
    println!("  {}", "Component Preview:".bold().dimmed());
    println!(
        "  {}",
        "───────────────────────────────────────────────────────".dimmed()
    );
    for line in rendered.lines() {
        println!("  {line}");
    }
    println!(
        "  {}",
        "───────────────────────────────────────────────────────".dimmed()
    );
    println!();

    Ok(())
}

/// Executes command / task generator.
pub fn execute_make_command(name: &str, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let name_snake = name.to_snake_case();
    let name_pascal = name.to_pascal_case();

    info!(
        name = name,
        dry_run = dry_run,
        "Generating command component"
    );

    let context = json!({
        "name": name,
        "name_snake": name_snake,
        "name_pascal": name_pascal,
    });

    let template = stubs::get_stub("command.stub");
    let rendered = stubs::render_stub(&template, &context)?;

    let target_path = format!("src/commands/{name_snake}.rs");

    ui::print_section(&format!("Scaffolding Task Command: {}", name.bold().cyan()));

    if !dry_run {
        let file = Path::new(&target_path);
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file, &rendered)?;
        format_file(file);

        ui::print_success(&format!("Generated {}", target_path.bold().white()));
    } else {
        ui::print_info(&format!(
            "[dry-run] Would generate {}",
            target_path.bold().white()
        ));
    }

    println!();
    println!("  {}", "Component Preview:".bold().dimmed());
    println!(
        "  {}",
        "───────────────────────────────────────────────────────".dimmed()
    );
    for line in rendered.lines() {
        println!("  {line}");
    }
    println!(
        "  {}",
        "───────────────────────────────────────────────────────".dimmed()
    );
    println!();

    Ok(())
}

/// Publishes default stubs into `./stubs/` directory.
pub fn execute_stub_publish(force: bool) -> Result<(), Box<dyn std::error::Error>> {
    ui::print_section("Publishing Default Template Stubs");

    let target_dir = Path::new("stubs");
    let results = stubs::publish_stubs(target_dir, force)?;

    for (filename, written) in results {
        if written {
            ui::print_success(&format!("Published stubs/{}", filename.bold().white()));
        } else {
            ui::print_warn(&format!(
                "Skipped stubs/{filename} (already exists, use --force to overwrite)"
            ));
        }
    }

    println!();
    ui::print_info("You can now customize these stubs to match your project conventions.");
    println!();

    Ok(())
}

/// Formats a generated Rust file with rustfmt if cargo is available.
fn format_file(path: &Path) {
    if let Some(path_str) = path.to_str() {
        let result = std::process::Command::new("cargo")
            .args(["fmt", "--", path_str])
            .output();
        drop(result);
    }
}
