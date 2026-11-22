//! Bordered instruction callout box for developer guidance and next-step actions.

use crate::ui::{
    Ui,
    ansi::{ColorizeExt, visible_width},
};

/// Bordered callout box displaying structured guidance and recommendations.
#[derive(Debug, Clone)]
pub struct Instruction {
    heading: String,
    items: Vec<String>,
}

impl Instruction {
    /// Creates a new instruction box with the given heading title.
    #[must_use]
    pub fn new(heading: impl Into<String>) -> Self {
        Self {
            heading: heading.into(),
            items: Vec::new(),
        }
    }

    /// Adds a bulleted item instruction.
    pub fn add(&mut self, item: impl Into<String>) -> &mut Self {
        self.items.push(item.into());
        self
    }

    /// Formats the instruction box into a rounded UTF-8 bordered block.
    #[must_use]
    pub fn prepare(&self) -> String {
        let mut widest = visible_width(&self.heading);
        for item in &self.items {
            widest = widest.max(visible_width(item) + 2);
        }

        let padding = 4;
        let box_width = widest + padding;

        let top_border = format!("╭{}╮", "─".repeat(box_width + 2));
        let mid_border = format!("├{}┤", "─".repeat(box_width + 2));
        let bot_border = format!("╰{}╯", "─".repeat(box_width + 2));

        let mut lines = Vec::new();
        lines.push(top_border.cyan().to_string());

        let heading_pad = box_width.saturating_sub(visible_width(&self.heading));
        lines.push(format!(
            "{}  {}{}  {}",
            "│".cyan(),
            self.heading.bold().white(),
            " ".repeat(heading_pad),
            "│".cyan()
        ));

        if !self.items.is_empty() {
            lines.push(mid_border.cyan().to_string());
            for item in &self.items {
                let item_text = format!("➜ {item}");
                let item_pad = box_width.saturating_sub(visible_width(&item_text));
                lines.push(format!(
                    "{}  {}{}  {}",
                    "│".cyan(),
                    item_text.cyan(),
                    " ".repeat(item_pad),
                    "│".cyan()
                ));
            }
        }

        lines.push(bot_border.cyan().to_string());
        lines.join("\n")
    }

    /// Renders the instruction box to the given UI instance.
    pub fn render(&self, ui: &Ui) {
        ui.writer().print(&self.prepare());
    }
}
