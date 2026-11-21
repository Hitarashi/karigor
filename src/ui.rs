//! Terminal UI design system and presentation helpers for Karigor.

use colored::Colorize;
use comfy_table::{
    Attribute, Cell, Color, ContentArrangement, Table, modifiers::UTF8_ROUND_CORNERS,
    presets::UTF8_FULL,
};

/// Formats a clean header banner with rounded corners and subtitle.
pub fn print_banner(title: &str, subtitle: &str) {
    println!();
    println!(
        "{}",
        " ╭──────────────────────────────────────────────────────────╮".cyan()
    );
    println!(
        " {}  {} {}",
        "│".cyan(),
        format!("⚡ {title}").bold().white(),
        " ".repeat(53_usize.saturating_sub(title.len() + 3))
    );
    if !subtitle.is_empty() {
        println!(
            " {}  {} {}",
            "│".cyan(),
            subtitle.dimmed(),
            " ".repeat(55_usize.saturating_sub(subtitle.len()))
        );
    }
    println!(
        "{}",
        " ╰──────────────────────────────────────────────────────────╯".cyan()
    );
    println!();
}

/// Prints a section header.
pub fn print_section(title: &str) {
    println!();
    println!("{} {}", "◆".cyan(), title.bold().white());
}

/// Prints a success bullet with message.
pub fn print_success(message: &str) {
    println!("  {} {}", "✔".green().bold(), message);
}

/// Prints an informational bullet.
pub fn print_info(message: &str) {
    println!("  {} {}", "➜".blue().bold(), message);
}

/// Prints a warning bullet.
pub fn print_warn(message: &str) {
    println!("  {} {}", "⚠".yellow().bold(), message);
}

/// Prints an error bullet.
pub fn print_error(message: &str) {
    eprintln!("  {} {}", "✖".red().bold(), message);
}

/// Formats an HTTP method with standardized badge coloring as a styled table cell.
pub fn cell_http_method(method: &str) -> Cell {
    let cell = Cell::new(format!(" {method} ")).add_attribute(Attribute::Bold);
    match method.to_uppercase().as_str() {
        "GET" => cell.fg(Color::Black).bg(Color::Green),
        "POST" => cell.fg(Color::Black).bg(Color::Blue),
        "PUT" => cell.fg(Color::Black).bg(Color::Yellow),
        "PATCH" => cell.fg(Color::Black).bg(Color::Magenta),
        "DELETE" => cell.fg(Color::White).bg(Color::Red),
        "HEAD" => cell.fg(Color::Black).bg(Color::Cyan),
        "OPTIONS" => cell.fg(Color::White).bg(Color::DarkGrey),
        _ => cell.fg(Color::Black).bg(Color::White),
    }
}

/// Formats an HTTP method with standardized badge coloring for standalone strings.
pub fn format_http_method(method: &str) -> String {
    match method.to_uppercase().as_str() {
        "GET" => " GET ".black().on_green().bold().to_string(),
        "POST" => " POST ".black().on_blue().bold().to_string(),
        "PUT" => " PUT ".black().on_yellow().bold().to_string(),
        "PATCH" => " PATCH ".black().on_magenta().bold().to_string(),
        "DELETE" => " DELETE ".white().on_red().bold().to_string(),
        "HEAD" => " HEAD ".black().on_cyan().bold().to_string(),
        "OPTIONS" => " OPTIONS ".white().on_bright_black().bold().to_string(),
        other => format!(" {other} ").black().on_white().bold().to_string(),
    }
}

/// Creates a pre-configured table with rounded borders and responsive formatting.
pub fn create_table(headers: &[&str]) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Disabled);

    let header_cells = headers.iter().map(|header| {
        Cell::new(header)
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan)
    });

    table.set_header(header_cells);
    table
}
