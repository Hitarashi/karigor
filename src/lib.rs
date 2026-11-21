//! Craftsman CLI, interactive scaffolding, dev server, and code generators for Druto.

use clap::{Parser, Subcommand};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, fmt};

pub mod commands;
pub mod injector;
pub mod stubs;
pub mod ui;

/// Command-line argument specifications for Karigor.
#[derive(Parser, Debug)]
#[command(
    name = "karigor",
    version,
    about = "Craftsman CLI, interactive scaffolding, dev server, and code generators for Druto",
    long_about = None
)]
pub struct Cli {
    /// Enable verbose debug tracing logs
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress non-essential output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

/// Primary Karigor subcommands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new Druto project with interactive scaffolding
    New {
        /// Project name or destination directory
        name: Option<String>,
    },
    /// Start the development server with live reload
    #[command(alias = "serve")]
    Dev {
        /// Host address to bind
        #[arg(short = 'H', long)]
        host: Option<String>,
        /// Port to listen on
        #[arg(short, long)]
        port: Option<u16>,
        /// Preview configuration without spawning child process
        #[arg(long)]
        dry_run: bool,
    },
    /// Display all compiled application routes in a formatted table
    Routes {
        /// Filter routes by path, method, or handler name
        #[arg(short, long)]
        filter: Option<String>,
        /// Output route table in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Display the active onion middleware pipeline
    Middleware,
    /// Eject default template stubs into ./stubs for customization
    #[command(name = "stub:publish", alias = "stubs:publish")]
    StubPublish {
        /// Overwrite existing stubs in ./stubs
        #[arg(short, long)]
        force: bool,
    },
    /// Generate application components (controller, middleware, command)
    #[command(subcommand)]
    Make(MakeCommands),
    /// Run system, environment, port, and connection diagnostics
    Doctor,
}

/// Component code generators.
#[derive(Subcommand, Debug)]
pub enum MakeCommands {
    /// Scaffold a modular route file and mount it
    Route {
        /// Name of the route module
        name: String,
        /// Generate a paired controller delegating handlers to it
        #[arg(short, long)]
        controller: bool,
        /// Preview generated code without writing to disk
        #[arg(long)]
        dry_run: bool,
    },
    /// Scaffold a new route controller
    Controller {
        /// Name of the controller
        name: String,
        /// Generate API controller without view templates
        #[arg(long)]
        api: bool,
        /// Preview generated code without writing to disk
        #[arg(long)]
        dry_run: bool,
    },
    /// Scaffold a custom onion middleware
    Middleware {
        /// Name of the middleware
        name: String,
        /// Preview generated code without writing to disk
        #[arg(long)]
        dry_run: bool,
    },
    /// Scaffold a custom console command or background task
    Command {
        /// Name of the command
        name: String,
        /// Preview generated code without writing to disk
        #[arg(long)]
        dry_run: bool,
    },
}

/// Normalizes command line arguments for both Cargo subcommand and direct invocation.
pub fn normalize_args<I, T>(raw_args: I) -> Vec<String>
where
    I: IntoIterator<Item = T>,
    T: Into<String>,
{
    let mut args: Vec<String> = raw_args.into_iter().map(Into::into).collect();

    // When invoked as `cargo karigor ...`, Cargo passes "karigor" as the second token (args[1])
    if args.len() > 1 && args[1] == "karigor" {
        args.remove(1);
    }

    // Support both `make:controller` and `make controller` syntax
    let mut normalized = Vec::new();
    for arg in args {
        if let Some(sub) = arg.strip_prefix("make:") {
            normalized.push("make".to_string());
            normalized.push(sub.to_string());
        } else {
            normalized.push(arg);
        }
    }

    normalized
}

/// Main entry point executing Karigor commands.
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = normalize_args(std::env::args());
    let cli = Cli::parse_from(args);

    let filter = if cli.verbose {
        LevelFilter::DEBUG
    } else {
        LevelFilter::WARN
    };

    fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(filter.into())
                .from_env_lossy(),
        )
        .with_target(false)
        .compact()
        .init();

    match cli.command {
        Commands::New { name } => commands::new::execute_new(name).await?,
        Commands::Dev {
            host,
            port,
            dry_run,
        } => commands::dev::execute_dev(host, port, dry_run).await?,
        Commands::Routes { filter, json } => commands::routes::execute_routes(filter, json)?,
        Commands::Middleware => commands::middleware::execute_middleware()?,
        Commands::StubPublish { force } => commands::make::execute_stub_publish(force)?,
        Commands::Make(MakeCommands::Route {
            name,
            controller,
            dry_run,
        }) => commands::make::execute_make_route(&name, controller, dry_run)?,
        Commands::Make(MakeCommands::Controller { name, api, dry_run }) => {
            commands::make::execute_make_controller(&name, api, dry_run)?
        }
        Commands::Make(MakeCommands::Middleware { name, dry_run }) => {
            commands::make::execute_make_middleware(&name, dry_run)?
        }
        Commands::Make(MakeCommands::Command { name, dry_run }) => {
            commands::make::execute_make_command(&name, dry_run)?
        }
        Commands::Doctor => commands::doctor::execute_doctor()?,
    }

    Ok(())
}
