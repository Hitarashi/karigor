//! Terminal UI design system, presentation primitives, and interactive prompts for Karigor.

use std::time::Duration;

use colored::Colorize;
use comfy_table::presets::UTF8_FULL;
pub use comfy_table::{self, Attribute, Cell, Color, ContentArrangement, Table};
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};

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

/// Prints a section header with an indicator icon.
pub fn print_section(title: &str) {
    println!();
    println!("{} {}", "◆".cyan(), title.bold().white());
}

/// Prints a success status message.
pub fn print_success(message: &str) {
    println!("  {} {}", "✔".green().bold(), message);
}

/// Prints an informational note.
pub fn print_info(message: &str) {
    println!("  {} {}", "➜".blue().bold(), message);
}

/// Prints a warning notification.
pub fn print_warn(message: &str) {
    println!("  {} {}", "⚠".yellow().bold(), message);
}

/// Prints an error notification to standard error.
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
#[must_use]
pub fn create_table(headers: &[&str]) -> Table {
    let mut table = Table::new();
    table
        .load_style(UTF8_FULL.with_rounded_corners())
        .set_content_arrangement(ContentArrangement::Disabled);

    let header_cells = headers.iter().map(|header| {
        Cell::new(header)
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan)
    });

    table.set_header(header_cells);
    table
}

/// Creates a new progress spinner initialized with the given message.
#[must_use]
pub fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
            .template("{spinner:.cyan} {msg}")
            .expect("valid template"),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner
}

/// Prompts the user for a text value with an optional default.
pub fn prompt_input(prompt: &str, default: Option<&str>) -> Result<String, std::io::Error> {
    let theme = ColorfulTheme::default();
    let input = Input::with_theme(&theme);
    let mut builder = input.with_prompt(prompt);
    if let Some(def) = default {
        builder = builder.default(def.to_string());
    }
    builder
        .interact_text()
        .map_err(|err| std::io::Error::other(err.to_string()))
}

/// Prompts the user with a yes/no confirmation prompt.
pub fn prompt_confirm(prompt: &str, default: bool) -> Result<bool, std::io::Error> {
    let theme = ColorfulTheme::default();
    Confirm::with_theme(&theme)
        .with_prompt(prompt)
        .default(default)
        .interact()
        .map_err(|err| std::io::Error::other(err.to_string()))
}

/// Prompts the user to select one item from a list.
pub fn prompt_select<T: ToString + std::fmt::Display>(
    prompt: &str,
    items: &[T],
    default: usize,
) -> Result<usize, std::io::Error> {
    let theme = ColorfulTheme::default();
    Select::with_theme(&theme)
        .with_prompt(prompt)
        .items(items)
        .default(default)
        .interact()
        .map_err(|err| std::io::Error::other(err.to_string()))
}

/// Interactive terminal UI manager.
#[derive(Debug, Clone, Default)]
pub struct Ui {
    quiet: bool,
}

impl Ui {
    /// Creates a new `Ui` instance with standard verbosity.
    #[must_use]
    pub fn new() -> Self {
        Self { quiet: false }
    }

    /// Sets whether non-critical output should be silenced.
    #[must_use]
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    /// Prints a header banner unless quiet mode is active.
    pub fn banner(&self, title: &str, subtitle: &str) {
        if !self.quiet {
            print_banner(title, subtitle);
        }
    }

    /// Prints a section header unless quiet mode is active.
    pub fn section(&self, title: &str) {
        if !self.quiet {
            print_section(title);
        }
    }

    /// Prints a success message unless quiet mode is active.
    pub fn success(&self, message: &str) {
        if !self.quiet {
            print_success(message);
        }
    }

    /// Prints an informational message unless quiet mode is active.
    pub fn info(&self, message: &str) {
        if !self.quiet {
            print_info(message);
        }
    }

    /// Prints a warning message unless quiet mode is active.
    pub fn warn(&self, message: &str) {
        if !self.quiet {
            print_warn(message);
        }
    }

    /// Prints an error message.
    pub fn error(&self, message: &str) {
        print_error(message);
    }

    /// Creates a formatted table.
    #[must_use]
    pub fn table(&self, headers: &[&str]) -> Table {
        create_table(headers)
    }

    /// Starts a spinner unless quiet mode is active.
    #[must_use]
    pub fn spinner(&self, message: &str) -> ProgressBar {
        if self.quiet {
            ProgressBar::hidden()
        } else {
            create_spinner(message)
        }
    }

    /// Prompts the user for a text value.
    pub fn input(&self, prompt: &str, default: Option<&str>) -> Result<String, std::io::Error> {
        prompt_input(prompt, default)
    }

    /// Prompts the user with a confirmation prompt.
    pub fn confirm(&self, prompt: &str, default: bool) -> Result<bool, std::io::Error> {
        prompt_confirm(prompt, default)
    }

    /// Prompts the user to select from a list.
    pub fn select<T: ToString + std::fmt::Display>(
        &self,
        prompt: &str,
        items: &[T],
        default: usize,
    ) -> Result<usize, std::io::Error> {
        prompt_select(prompt, items, default)
    }
}
