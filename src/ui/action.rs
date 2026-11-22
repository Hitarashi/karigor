//! Action lifecycle logging with status badges, elapsed execution duration, and failure reporting.

use std::time::Instant;

use crate::ui::{Ui, ansi::ColorizeExt};

/// Represents an active or completed command execution subtask.
pub struct Action<'a> {
    title: String,
    start_time: Instant,
    ui: &'a Ui,
}

impl<'a> Action<'a> {
    /// Initiates a new tracked action.
    #[must_use]
    pub fn new(title: impl Into<String>, ui: &'a Ui) -> Self {
        Self {
            title: title.into(),
            start_time: Instant::now(),
            ui,
        }
    }

    /// Marks the action as completed successfully.
    pub fn succeeded(self) {
        let elapsed = self.start_time.elapsed();
        let duration_ms = elapsed.as_millis();
        let timing = format!("({duration_ms}ms)").dimmed();
        let badge = "  DONE   ".green().bold();
        self.ui
            .writer()
            .print(&format!("{badge} {} {timing}", self.title.white()));
    }

    /// Marks the action as skipped with an optional reason.
    pub fn skipped(self, reason: &str) {
        let badge = " SKIPPED ".cyan().bold();
        let reason_str = format!("({reason})").dimmed();
        self.ui
            .writer()
            .print(&format!("{badge} {} {reason_str}", self.title.white()));
    }

    /// Marks the action as failed with an explanatory error message.
    pub fn failed(self, error: &str) {
        let badge = " FAILED  ".red().bold();
        let error_str = format!("({error})").red();
        self.ui
            .writer()
            .print_err(&format!("{badge} {} {error_str}", self.title.white()));
    }
}
