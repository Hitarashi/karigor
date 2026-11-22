//! Terminal UI design system, presentation primitives, and interactive prompts for Karigor.

pub mod action;
pub mod alert;
pub mod ansi;
pub mod instruction;
pub mod key_value;
pub mod prompt;
pub mod spinner;
pub mod steps;
pub mod sticker;
pub mod table;
pub mod writer;

use std::sync::{Arc, Mutex};

pub use action::Action;
pub use alert::{Alert, AlertKind};
pub use ansi::{
    Color, Colorize, ColorizeExt, StyledText, is_color_enabled, reset_color_override,
    set_color_enabled, strip_ansi, visible_width,
};
pub use instruction::Instruction;
pub use key_value::DefinitionList;
pub use prompt::{
    MockPromptBackend, PromptBackend, PromptHandler, TerminalPromptBackend, is_non_interactive,
    prompt_confirm_stream, prompt_input_stream, prompt_multiselect_stream, prompt_password_stream,
    prompt_select_stream, set_non_interactive,
};
pub use spinner::{ProgressBar, ProgressStyle};
pub use steps::Steps;
pub use sticker::Sticker;
pub use table::{Alignment, Attribute, Cell, ContentArrangement, Table, TablePreset, presets};
pub use writer::UiWriter;

/// Formats a clean header banner with rounded corners and subtitle.
pub fn print_banner(title: &str, subtitle: &str) {
    let ui = Ui::new();
    ui.banner(title, subtitle);
}

/// Prints a section header with an indicator icon.
pub fn print_section(title: &str) {
    let ui = Ui::new();
    ui.section(title);
}

/// Prints a success status message.
pub fn print_success(message: &str) {
    let ui = Ui::new();
    ui.success(message);
}

/// Prints an informational note.
pub fn print_info(message: &str) {
    let ui = Ui::new();
    ui.info(message);
}

/// Prints a warning notification.
pub fn print_warn(message: &str) {
    let ui = Ui::new();
    ui.warn(message);
}

/// Prints an error notification.
pub fn print_error(message: &str) {
    let ui = Ui::new();
    ui.error(message);
}

/// Formats an HTTP method with standardized badge coloring as a styled table cell.
#[must_use]
pub fn cell_http_method(method: &str) -> Cell {
    let cell = Cell::new(format!(" {method} ")).add_attribute(Attribute::Bold);
    match method.to_uppercase().as_str() {
        "GET" => cell.fg(Color::Black).bg(Color::Green),
        "POST" => cell.fg(Color::Black).bg(Color::Blue),
        "PUT" => cell.fg(Color::Black).bg(Color::Yellow),
        "PATCH" => cell.fg(Color::Black).bg(Color::Magenta),
        "DELETE" => cell.fg(Color::White).bg(Color::Red),
        "HEAD" => cell.fg(Color::Black).bg(Color::Cyan),
        "OPTIONS" => cell.fg(Color::White).bg(Color::BrightBlack),
        _ => cell.fg(Color::Black).bg(Color::White),
    }
}

/// Formats an HTTP method with standardized badge coloring for standalone strings.
#[must_use]
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
    table.load_style(TablePreset::Rounded);

    let header_cells = headers.iter().map(|header| {
        Cell::new(*header)
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
    spinner.set_message(message);
    spinner
}

/// Prompts the user for a text value with an optional default.
pub fn prompt_input(prompt: &str, default: Option<&str>) -> Result<String, std::io::Error> {
    PromptHandler::terminal().input(prompt, default)
}

/// Prompts the user with a yes/no confirmation prompt.
pub fn prompt_confirm(prompt: &str, default: bool) -> Result<bool, std::io::Error> {
    PromptHandler::terminal().confirm(prompt, default)
}

/// Prompts the user to select one item from a list.
pub fn prompt_select<T: ToString + std::fmt::Display>(
    prompt: &str,
    items: &[T],
    default: usize,
) -> Result<usize, std::io::Error> {
    PromptHandler::terminal().select(prompt, items, default)
}

/// Prompts the user for a masked or silent password.
pub fn prompt_password(prompt: &str) -> Result<String, std::io::Error> {
    PromptHandler::terminal().password(prompt)
}

/// Prompts the user to select multiple items from a list.
pub fn prompt_multiselect<T: ToString + std::fmt::Display>(
    prompt: &str,
    items: &[T],
    defaults: &[usize],
) -> Result<Vec<usize>, std::io::Error> {
    PromptHandler::terminal().multiselect(prompt, items, defaults)
}

/// Formats and prints a rectangular status card.
pub fn print_sticker(title: &str, lines: &[&str]) {
    let ui = Ui::new();
    ui.render_sticker(title, |sticker| {
        for &line in lines {
            sticker.line(line);
        }
    });
}

/// Formats and prints an alert notification.
pub fn print_alert(kind: AlertKind, title: &str, message: &str) {
    let ui = Ui::new();
    ui.render_alert(kind, title, |alert| {
        alert.message(message);
    });
}

/// Interactive terminal UI manager.
#[derive(Clone, Default)]
pub struct Ui {
    quiet: bool,
    writer: UiWriter,
    prompts: PromptHandler,
    captured_buffer: Option<Arc<Mutex<Vec<u8>>>>,
}

impl std::fmt::Debug for Ui {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Ui")
            .field("quiet", &self.quiet)
            .finish()
    }
}

impl Ui {
    /// Creates a new `Ui` instance with standard verbosity and console output.
    #[must_use]
    pub fn new() -> Self {
        Self {
            quiet: false,
            writer: UiWriter::Console,
            prompts: PromptHandler::terminal(),
            captured_buffer: None,
        }
    }

    /// Creates an in-memory capturing `Ui` instance for automated tests.
    #[must_use]
    pub fn buffered() -> Self {
        let (writer, buffer) = UiWriter::buffer();
        Self {
            quiet: false,
            writer,
            prompts: PromptHandler::new(Arc::new(TerminalPromptBackend::non_interactive())),
            captured_buffer: Some(buffer),
        }
    }

    /// Reconfigures the UI with a custom prompt handler backend.
    #[must_use]
    pub fn with_prompts(mut self, handler: PromptHandler) -> Self {
        self.prompts = handler;
        self
    }

    /// Sets whether non-critical output should be silenced.
    #[must_use]
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    /// Returns a reference to the active writer.
    #[must_use]
    pub fn writer(&self) -> &UiWriter {
        &self.writer
    }

    /// Retrieves captured output string if in-memory buffer mode is active.
    #[must_use]
    pub fn output_string(&self) -> Option<String> {
        self.captured_buffer.as_ref().and_then(|buf| {
            buf.lock()
                .ok()
                .and_then(|bytes| String::from_utf8(bytes.clone()).ok())
        })
    }

    /// Prints a header banner unless quiet mode is active.
    pub fn banner(&self, title: &str, subtitle: &str) {
        if self.quiet {
            return;
        }

        let content_width = visible_width(title).max(visible_width(subtitle));
        let inner_width: usize = 58.max(content_width + 4);

        self.writer.print("");
        self.writer.print(&format!(
            " {}",
            format!("╭{}╮", "─".repeat(inner_width)).cyan()
        ));

        let title_pad = inner_width.saturating_sub(2 + visible_width(title));
        self.writer.print(&format!(
            " {}  {}{}{}",
            "│".cyan(),
            title.bold().white(),
            " ".repeat(title_pad),
            "│".cyan()
        ));

        if !subtitle.is_empty() {
            let sub_pad = inner_width.saturating_sub(2 + visible_width(subtitle));
            self.writer.print(&format!(
                " {}  {}{}{}",
                "│".cyan(),
                subtitle.dimmed(),
                " ".repeat(sub_pad),
                "│".cyan()
            ));
        }

        self.writer.print(&format!(
            " {}",
            format!("╰{}╯", "─".repeat(inner_width)).cyan()
        ));
        self.writer.print("");
    }

    /// Prints a section header unless quiet mode is active.
    pub fn section(&self, title: &str) {
        if !self.quiet {
            self.writer.print("");
            self.writer
                .print(&format!("{} {}", "◆".cyan(), title.bold().white()));
        }
    }

    /// Prints a success message unless quiet mode is active.
    pub fn success(&self, message: &str) {
        if !self.quiet {
            self.writer
                .print(&format!("  {} {}", "✔".green().bold(), message));
        }
    }

    /// Prints an informational message unless quiet mode is active.
    pub fn info(&self, message: &str) {
        if !self.quiet {
            self.writer
                .print(&format!("  {} {}", "➜".blue().bold(), message));
        }
    }

    /// Prints a warning message unless quiet mode is active.
    pub fn warn(&self, message: &str) {
        if !self.quiet {
            self.writer
                .print(&format!("  {} {}", "⚠".yellow().bold(), message));
        }
    }

    /// Prints an error message.
    pub fn error(&self, message: &str) {
        self.writer
            .print_err(&format!("  {} {}", "✖".red().bold(), message));
    }

    /// Prints a standard line of text unless quiet mode is active.
    pub fn print(&self, message: impl AsRef<str>) {
        if !self.quiet {
            self.writer.print(message.as_ref());
        }
    }

    /// Initiates a tracked execution action with status indicators and timing.
    #[must_use]
    pub fn action(&self, title: impl Into<String>) -> Action<'_> {
        Action::new(title, self)
    }

    /// Creates a multi-step workflow guide builder.
    #[must_use]
    pub fn steps(&self) -> Steps {
        Steps::new()
    }

    /// Renders sequential workflow steps unless quiet mode is active.
    pub fn render_steps(&self, steps: &Steps) {
        if !self.quiet {
            steps.render(self);
        }
    }

    /// Creates an instruction callout box builder.
    #[must_use]
    pub fn instruction(&self, heading: impl Into<String>) -> Instruction {
        Instruction::new(heading)
    }

    /// Creates a key-value definition list builder.
    #[must_use]
    pub fn definition_list(&self) -> DefinitionList {
        DefinitionList::new()
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
        self.prompts.input(prompt, default)
    }

    /// Prompts the user with a confirmation prompt.
    pub fn confirm(&self, prompt: &str, default: bool) -> Result<bool, std::io::Error> {
        self.prompts.confirm(prompt, default)
    }

    /// Prompts the user to select from a list.
    pub fn select<T: ToString + std::fmt::Display>(
        &self,
        prompt: &str,
        items: &[T],
        default: usize,
    ) -> Result<usize, std::io::Error> {
        self.prompts.select(prompt, items, default)
    }

    /// Prompts the user for a masked or silent password.
    pub fn password(&self, prompt: &str) -> Result<String, std::io::Error> {
        self.prompts.password(prompt)
    }

    /// Prompts the user to select multiple items from a list.
    pub fn multiselect<T: ToString + std::fmt::Display>(
        &self,
        prompt: &str,
        items: &[T],
        defaults: &[usize],
    ) -> Result<Vec<usize>, std::io::Error> {
        self.prompts.multiselect(prompt, items, defaults)
    }

    /// Creates a bordered sticker card builder.
    #[must_use]
    pub fn sticker(&self) -> Sticker {
        Sticker::new()
    }

    /// Configures and renders a sticker card via a scoped closure.
    pub fn render_sticker<F>(&self, title: impl Into<String>, configure: F)
    where
        F: FnOnce(&mut Sticker),
    {
        if !self.quiet {
            let mut sticker = Sticker::new();
            sticker.title(title);
            configure(&mut sticker);
            sticker.render(self);
        }
    }

    /// Creates an alert container builder.
    #[must_use]
    pub fn alert(&self, kind: AlertKind, title: impl Into<String>) -> Alert {
        Alert::new(kind, title)
    }

    /// Configures and renders an alert container via a scoped closure.
    pub fn render_alert<F>(&self, kind: AlertKind, title: impl Into<String>, configure: F)
    where
        F: FnOnce(&mut Alert),
    {
        if !self.quiet {
            let mut alert = Alert::new(kind, title);
            configure(&mut alert);
            alert.render(self);
        }
    }

    /// Configures and renders an informational alert via a scoped closure.
    pub fn alert_info<F>(&self, title: impl Into<String>, configure: F)
    where
        F: FnOnce(&mut Alert),
    {
        self.render_alert(AlertKind::Info, title, configure);
    }

    /// Configures and renders a success alert via a scoped closure.
    pub fn alert_success<F>(&self, title: impl Into<String>, configure: F)
    where
        F: FnOnce(&mut Alert),
    {
        self.render_alert(AlertKind::Success, title, configure);
    }

    /// Configures and renders a warning alert via a scoped closure.
    pub fn alert_warning<F>(&self, title: impl Into<String>, configure: F)
    where
        F: FnOnce(&mut Alert),
    {
        self.render_alert(AlertKind::Warning, title, configure);
    }

    /// Configures and renders a danger alert via a scoped closure.
    pub fn alert_danger<F>(&self, title: impl Into<String>, configure: F)
    where
        F: FnOnce(&mut Alert),
    {
        self.render_alert(AlertKind::Danger, title, configure);
    }

    /// Formats an HTTP method cell badge for display in tables.
    #[must_use]
    pub fn cell_http_method(&self, method: &str) -> Cell {
        cell_http_method(method)
    }

    /// Formats an HTTP method with standardized ANSI background badges.
    #[must_use]
    pub fn format_http_method(&self, method: &str) -> String {
        format_http_method(method)
    }
}
