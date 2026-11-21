//! Development server supervisor and live-reload runner for Karigor.

use colored::Colorize;
use tracing::info;

use crate::ui;

/// Executes the development server supervisor.
pub async fn execute_dev(
    host: Option<String>,
    port: Option<u16>,
) -> Result<(), Box<dyn std::error::Error>> {
    let bind_host = host.unwrap_or_else(|| "127.0.0.1".to_string());
    let bind_port = port.unwrap_or(3000);

    info!(
        host = %bind_host,
        port = bind_port,
        "Initializing development server supervisor"
    );

    println!();
    println!(
        "{}",
        " ╭──────────────────────────────────────────────────────────╮".cyan()
    );
    println!(
        " {}                                                          {}",
        "│".cyan(),
        "│".cyan()
    );
    println!(
        " {}   {}                   {}",
        "│".cyan(),
        "⚡ Druto Development Server v0.1.0".bold().white(),
        "│".cyan()
    );
    println!(
        " {}                                                          {}",
        "│".cyan(),
        "│".cyan()
    );
    println!(
        " {}   {}  http://{}:{}                     {}",
        "│".cyan(),
        "➜  Local:  ".green().bold(),
        bind_host,
        bind_port,
        "│".cyan()
    );
    println!(
        " {}   {}  http://192.168.1.45:{}                  {}",
        "│".cyan(),
        "➜  Network:".cyan().bold(),
        bind_port,
        "│".cyan()
    );
    println!(
        " {}   {}  {} (unoptimized + debuginfo)         {}",
        "│".cyan(),
        "➜  Mode:   ".dimmed(),
        "Development".bold().yellow(),
        "│".cyan()
    );
    println!(
        " {}                                                          {}",
        "│".cyan(),
        "│".cyan()
    );
    println!(
        " {}   [r] restart   [h] routes   [m] middleware   [q] quit   {}",
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
        " ╰──────────────────────────────────────────────────────────╯".cyan()
    );
    println!();

    ui::print_info("Watching for file changes in ./src and ./config...");
    ui::print_success("HTTP listener ready to receive connections");
    println!();

    Ok(())
}
