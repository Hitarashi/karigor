//! Interactive and automated project scaffolding for Druto web applications.

use std::{fs, io::IsTerminal, path::Path, time::Duration};

use colored::Colorize;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};
use tracing::info;

use crate::{stubs, ui};

/// Executes project scaffolding.
pub async fn execute_new(initial_name: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    ui::print_banner(
        "Karigor Scaffolding Wizard",
        "Generate a new Druto web application",
    );

    let is_terminal = std::io::stdin().is_terminal();

    let (project_name, init_git) = if is_terminal && initial_name.is_none() {
        let theme = ColorfulTheme::default();

        let name: String = Input::with_theme(&theme)
            .with_prompt("Project name")
            .default("my-druto-app".to_string())
            .interact_text()?;

        let presets = [
            "REST API (Batteries-Included: Router, CORS, Limiter, Shield, Bodyparser)",
            "REST API (Minimal: Lightweight Router & JSON parser)",
            "Full-Stack Web (Router, Session, Redis, Shield)",
        ];

        let preset_index = Select::with_theme(&theme)
            .with_prompt("Choose architecture preset")
            .items(presets)
            .default(0)
            .interact()?;

        let init_git = Confirm::with_theme(&theme)
            .with_prompt("Initialize a Git repository?")
            .default(true)
            .interact()?;

        info!(
            project = %name,
            preset = %presets[preset_index],
            git = init_git,
            "Configured scaffolding parameters"
        );

        (name, init_git)
    } else {
        let name = initial_name.unwrap_or_else(|| "my-druto-app".to_string());
        (name, true)
    };

    ui::print_section(&format!(
        "Scaffolding Project: {}",
        project_name.bold().cyan()
    ));

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
            .template("{spinner:.cyan} {msg}")?,
    );

    let project_dir = Path::new(&project_name);
    if project_dir.exists() {
        return Err(format!("Directory '{project_name}' already exists!").into());
    }

    spinner.set_message("Creating directory structure...");
    fs::create_dir_all(project_dir.join("src").join("routes"))?;
    fs::create_dir_all(project_dir.join("src").join("controllers"))?;
    tokio::time::sleep(Duration::from_millis(150)).await;

    spinner.set_message("Writing Cargo.toml manifest with Druto framework...");
    let cargo_toml = format!(
        r#"[package]
name = "{project_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
druto = {{ git = "https://github.com/druto-rs/druto.git", branch = "main" }}
tokio = {{ version = "1.54.1", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter", "fmt"] }}
"#
    );
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;
    tokio::time::sleep(Duration::from_millis(100)).await;

    spinner.set_message("Generating main server entrypoint and route dispatcher...");
    let main_rs = r#"use druto::prelude::*;

mod controllers;
mod routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{host}:{port}");

    println!();
    println!("  ⚡ Druto server running at http://{addr}");
    println!("  ➜ Press Ctrl+C to terminate");
    println!();

    let router = routes::routes();
    Server::bind(addr).serve(router).await?;

    Ok(())
}
"#;
    fs::write(project_dir.join("src").join("main.rs"), main_rs)?;

    let routes_mod_rs = r#"use druto::prelude::*;

pub fn routes() -> Router {
    let mut router = Router::new();

    // Default welcome route
    router.get(
        "/",
        from_handler(|ctx| {
            Box::pin(async move {
                ctx.response.text("Welcome to Druto Framework! ⚡\n");
                Ok(())
            })
        }),
    );

    router
}
"#;
    fs::write(
        project_dir.join("src").join("routes").join("mod.rs"),
        routes_mod_rs,
    )?;

    let controllers_mod_rs = "// Controller modules registered by karigor make:controller\n";
    fs::write(
        project_dir.join("src").join("controllers").join("mod.rs"),
        controllers_mod_rs,
    )?;

    spinner.set_message("Ejecting customizable template stubs into ./stubs...");
    stubs::publish_stubs(&project_dir.join("stubs"), true)?;

    spinner.set_message("Generating environment files (.env, .env.example)...");
    let env_content = "HOST=127.0.0.1\nPORT=3000\nAPP_ENV=development\n";
    fs::write(project_dir.join(".env"), env_content)?;
    fs::write(project_dir.join(".env.example"), env_content)?;

    let gitignore_content = "/target\nCargo.lock\n.env\n*.local\n";
    fs::write(project_dir.join(".gitignore"), gitignore_content)?;

    let readme_content = format!(
        r#"# {project_name}

A modern web application built on the [Druto](https://github.com/druto-rs/druto) framework.

## Getting Started

```bash
# Start development server
cargo karigor dev

# Or directly with Cargo
cargo run
```

## Scaffolding Components

```bash
# Generate a new modular route with paired controller
cargo karigor make:route user --controller

# Generate custom onion middleware
cargo karigor make:middleware auth

# Inspect all registered routes
cargo karigor routes
```
"#
    );
    fs::write(project_dir.join("README.md"), readme_content)?;

    if init_git {
        spinner.set_message("Initializing git repository...");
        let git_init = std::process::Command::new("git")
            .arg("init")
            .arg(project_dir)
            .output();
        drop(git_init);
    }

    spinner.finish_and_clear();

    ui::print_success(&format!(
        "Created Druto application in {}",
        format!("./{project_name}").bold().cyan()
    ));
    ui::print_success("Wrote Cargo.toml with Druto framework dependencies");
    ui::print_success("Generated HTTP server entrypoint in src/main.rs");
    ui::print_success("Configured initial routing table in src/routes/mod.rs");
    ui::print_success("Published customizable stubs in stubs/");
    if init_git {
        ui::print_success("Initialized Git repository");
    }

    println!();
    println!(
        "{}",
        " ╭── Next Steps ─────────────────────────────────────────────╮".cyan()
    );
    println!(
        " {}                                                          {}",
        "│".cyan(),
        "│".cyan()
    );
    println!(
        " {}   cd {}                                            {}",
        "│".cyan(),
        project_name.bold().yellow(),
        "│".cyan()
    );
    println!(
        " {}   cargo karigor dev       {}                     {}",
        "│".cyan(),
        "# Start HTTP web server".dimmed(),
        "│".cyan()
    );
    println!(
        " {}   cargo karigor make:route user --controller             {}",
        "│".cyan(),
        "│".cyan()
    );
    println!(
        " {}                                                          {}",
        "│".cyan(),
        "│".cyan()
    );
    println!(
        "{}",
        " ╰───────────────────────────────────────────────────────────╯".cyan()
    );
    println!();

    Ok(())
}
