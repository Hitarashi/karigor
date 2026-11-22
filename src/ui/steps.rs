//! Sequential workflow guidance steps for onboarding and post-generation instructions.

use crate::ui::{Ui, ansi::ColorizeExt};

/// Formats a numbered list of progressive operational steps.
#[derive(Debug, Clone, Default)]
pub struct Steps {
    items: Vec<(String, Option<String>)>,
}

impl Steps {
    /// Creates a new empty steps collection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a step with a title and optional detail explanation.
    pub fn add(
        &mut self,
        title: impl Into<String>,
        detail: Option<impl Into<String>>,
    ) -> &mut Self {
        self.items.push((title.into(), detail.map(Into::into)));
        self
    }

    /// Adds a step with a title and detail explanation.
    pub fn step(&mut self, title: impl Into<String>, detail: impl Into<String>) -> &mut Self {
        self.add(title, Some(detail))
    }

    /// Prepares the formatted string representation of the steps sequence.
    #[must_use]
    pub fn prepare(&self) -> String {
        let mut lines = Vec::new();

        for (index, (title, detail)) in self.items.iter().enumerate() {
            let step_number = index + 1;
            let counter = format!("{step_number}.").cyan().bold();
            lines.push(format!("{counter} {}", title.bold().white()));

            if let Some(text) = detail {
                for line in text.lines() {
                    lines.push(format!("   {line}"));
                }
            }

            if index + 1 < self.items.len() {
                lines.push(String::new());
            }
        }

        lines.join("\n")
    }

    /// Renders the steps directly to the provided UI instance.
    pub fn render(&self, ui: &Ui) {
        let output = self.prepare();
        ui.writer().print(&output);
    }
}
