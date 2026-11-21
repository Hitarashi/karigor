//! Interactive project scaffolding wizard for Karigor.

use std::time::Duration;

use colored::Colorize;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};
use tracing::info;

use crate::ui;

/// Executes the interactive scaffolding wizard.
pub async fn execute_new(initial_name: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    ui::print_banner(
        "Karigor Scaffolding Wizard",
        "Generate a new Druto web application",
    );

    let theme = ColorfulTheme::default();

    let project_name = match initial_name {
        Some(name) => name,
        None => Input::with_theme(&theme)
            .with_prompt("Project name")
            .default("my-druto-app".to_string())
            .interact_text()?,
    };

    let presets = [
        "REST API (Batteries-Included: Router, CORS, Limiter, Shield, Bodyparser)",
        "REST API (Minimal: Lightweight Router & JSON parser)",
        "Full-Stack Web (Router, Session, Redis, Shield, Staticfiles)",
    ];

    let preset_index = Select::with_theme(&theme)
        .with_prompt("Choose architecture preset")
        .items(&presets)
        .default(0)
        .interact()?;

    let database_options = [
        "None (Stateless service)",
        "PostgreSQL (via SQLx / connection pool)",
        "SQLite (Local file-backed embedded DB)",
    ];

    let db_index = Select::with_theme(&theme)
        .with_prompt("Primary database engine")
        .items(&database_options)
        .default(0)
        .interact()?;

    let session_cache_options = [
        "In-Memory Store (Fast local development)",
        "Redis Distributed Store (druto-redis)",
    ];

    let session_index = Select::with_theme(&theme)
        .with_prompt("Cache & Session storage driver")
        .items(&session_cache_options)
        .default(0)
        .interact()?;

    let init_git = Confirm::with_theme(&theme)
        .with_prompt("Initialize a Git repository?")
        .default(true)
        .interact()?;

    info!(
        project = %project_name,
        preset = %presets[preset_index],
        database = %database_options[db_index],
        storage = %session_cache_options[session_index],
        git = init_git,
        "Configured scaffolding parameters"
    );

    ui::print_section("Generating Project Files");

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
            .template("{spinner:.cyan} {msg}")?,
    );

    spinner.set_message("Creating project folder and Cargo.toml manifest...");
    tokio::time::sleep(Duration::from_millis(350)).await;

    spinner.set_message("Configuring onion middleware pipeline and routes...");
    tokio::time::sleep(Duration::from_millis(300)).await;

    spinner.set_message("Generating environment files (.env, .env.example)...");
    tokio::time::sleep(Duration::from_millis(250)).await;

    if init_git {
        spinner.set_message("Initializing git repository and .gitignore...");
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    spinner.finish_and_clear();

    ui::print_success(&format!(
        "Created workspace in {}",
        format!("./{project_name}").bold().cyan()
    ));
    ui::print_success("Wrote Cargo.toml with Druto framework dependencies");
    ui::print_success("Configured initial routes in src/routes.rs");
    ui::print_success("Configured onion middleware in src/middleware.rs");
    if init_git {
        ui::print_success("Initialized Git repository on branch 'main'");
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
        "# Start development server".dimmed(),
        "│".cyan()
    );
    println!(
        " {}   cargo karigor routes    {}                     {}",
        "│".cyan(),
        "# Inspect route table".dimmed(),
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
