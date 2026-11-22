//! Bordered card and sticker widget for service announcements, status, and diagnostic summaries.

use crate::ui::{
    Color, Ui,
    ansi::{ColorizeExt, visible_width},
};

/// Bordered rectangular card for status overviews, server endpoints, and diagnostic telemetry.
#[derive(Debug, Clone)]
pub struct Sticker {
    title: Option<String>,
    border_color: Color,
    min_width: Option<usize>,
    lines: Vec<String>,
}

impl Default for Sticker {
    fn default() -> Self {
        Self::new()
    }
}

impl Sticker {
    /// Creates a new empty card widget with default styling.
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: None,
            border_color: Color::Cyan,
            min_width: None,
            lines: Vec::new(),
        }
    }

    /// Sets the heading title displayed at the top of the card.
    pub fn title(&mut self, title: impl Into<String>) -> &mut Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the ANSI border color for the card's rounded frame.
    pub fn border_color(&mut self, color: Color) -> &mut Self {
        self.border_color = color;
        self
    }

    /// Sets an explicit minimum inner width in characters.
    pub fn min_width(&mut self, width: usize) -> &mut Self {
        self.min_width = Some(width);
        self
    }

    /// Appends a formatted line of text to the card content.
    pub fn line(&mut self, text: impl Into<String>) -> &mut Self {
        self.lines.push(text.into());
        self
    }

    /// Appends an empty spacer line inside the card frame.
    pub fn blank(&mut self) -> &mut Self {
        self.lines.push(String::new());
        self
    }

    /// Appends a muted advisory or action note.
    pub fn note(&mut self, note: impl Into<String>) -> &mut Self {
        self.lines.push(note.into().dimmed().to_string());
        self
    }

    /// Formats the card into a styled, bordered string.
    #[must_use]
    pub fn prepare(&self) -> String {
        let mut widest = self
            .title
            .as_ref()
            .map_or(0, |heading| visible_width(heading) + 4);

        for line in &self.lines {
            widest = widest.max(visible_width(line));
        }

        let inner_width = self.min_width.unwrap_or(0).max(widest + 4).max(36);

        let top_border = format!("╭{}╮", "─".repeat(inner_width))
            .color(self.border_color)
            .to_string();
        let mid_divider = format!("├{}┤", "─".repeat(inner_width))
            .color(self.border_color)
            .to_string();
        let bot_border = format!("╰{}╯", "─".repeat(inner_width))
            .color(self.border_color)
            .to_string();
        let vertical_bar = "│".color(self.border_color).to_string();

        let mut output_lines = Vec::new();
        output_lines.push(top_border);

        if let Some(ref heading) = self.title {
            let heading_text = heading.bold().white().to_string();
            let pad = inner_width.saturating_sub(2 + visible_width(heading));
            output_lines.push(format!(
                "{vertical_bar}  {heading_text}{}{vertical_bar}",
                " ".repeat(pad)
            ));
            output_lines.push(mid_divider);
        }

        for line in &self.lines {
            let width = visible_width(line);
            let pad = inner_width.saturating_sub(2 + width);
            output_lines.push(format!(
                "{vertical_bar}  {line}{}{vertical_bar}",
                " ".repeat(pad)
            ));
        }

        output_lines.push(bot_border);
        output_lines.join("\n")
    }

    /// Renders the card directly through the provided UI manager.
    pub fn render(&self, ui: &Ui) {
        ui.writer().print(&self.prepare());
    }
}
