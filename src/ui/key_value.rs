//! Aligned key-value definition list formatter for diagnostics and status reporting.

use crate::ui::{
    Ui,
    ansi::{ColorizeExt, visible_width},
};

/// Key-value property pair list formatted with leader alignment.
#[derive(Debug, Clone, Default)]
pub struct DefinitionList {
    entries: Vec<(String, String)>,
}

impl DefinitionList {
    /// Creates a new empty definition list.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a key-value pair to the list.
    pub fn add(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.entries.push((key.into(), value.into()));
        self
    }

    /// Formats the definition list into aligned string lines with dot leaders.
    #[must_use]
    pub fn prepare(&self) -> String {
        let max_key_width = self
            .entries
            .iter()
            .map(|(key, _)| visible_width(key))
            .max()
            .unwrap_or(0);

        let target_width = max_key_width + 4;
        let mut lines = Vec::new();

        for (key, value) in &self.entries {
            let key_width = visible_width(key);
            let leader_len = target_width.saturating_sub(key_width);
            let leader = ".".repeat(leader_len).dimmed();

            lines.push(format!(
                "  {} {leader} {}",
                key.bold().white(),
                value.cyan()
            ));
        }

        lines.join("\n")
    }

    /// Renders the definition list to the given UI instance.
    pub fn render(&self, ui: &Ui) {
        ui.writer().print(&self.prepare());
    }
}
