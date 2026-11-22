//! High-visibility alert banners for status notices, operational guidelines, and destructive actions.

use crate::ui::{
    Color, Ui,
    ansi::{ColorizeExt, visible_width},
};

/// Severity classification for alert presentations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertKind {
    /// Informational operational update.
    Info,
    /// Successful outcome or verification status.
    Success,
    /// Non-fatal warning or schema drift advisory.
    Warning,
    /// Critical error, data hazard, or destructive operation warning.
    Danger,
}

impl AlertKind {
    /// Returns the badge text and ANSI background/foreground styling.
    #[must_use]
    pub fn badge(self) -> String {
        match self {
            Self::Info => " INFO ".black().on_blue().bold().to_string(),
            Self::Success => " SUCCESS ".black().on_green().bold().to_string(),
            Self::Warning => " WARNING ".black().on_yellow().bold().to_string(),
            Self::Danger => " DANGER ".white().on_red().bold().to_string(),
        }
    }

    /// Returns the primary accent border color associated with this alert kind.
    #[must_use]
    pub const fn border_color(self) -> Color {
        match self {
            Self::Info => Color::Cyan,
            Self::Success => Color::Green,
            Self::Warning => Color::Yellow,
            Self::Danger => Color::Red,
        }
    }
}

/// Bordered callout container communicating critical status, consequences, and recovery suggestions.
#[derive(Debug, Clone)]
pub struct Alert {
    kind: AlertKind,
    title: String,
    message: Option<String>,
    suggestion: Option<String>,
    consequence: Option<String>,
    bullets: Vec<String>,
}

impl Alert {
    /// Creates a new alert with the given kind and headline title.
    #[must_use]
    pub fn new(kind: AlertKind, title: impl Into<String>) -> Self {
        Self {
            kind,
            title: title.into(),
            message: None,
            suggestion: None,
            consequence: None,
            bullets: Vec::new(),
        }
    }

    /// Sets the explanatory message body.
    pub fn message(&mut self, text: impl Into<String>) -> &mut Self {
        self.message = Some(text.into());
        self
    }

    /// Sets the remediation or next-step suggestion.
    pub fn suggestion(&mut self, text: impl Into<String>) -> &mut Self {
        self.suggestion = Some(text.into());
        self
    }

    /// Sets the consequence statement explaining the risk of proceeding.
    pub fn consequence(&mut self, text: impl Into<String>) -> &mut Self {
        self.consequence = Some(text.into());
        self
    }

    /// Appends a bulleted detail item.
    pub fn bullet(&mut self, text: impl Into<String>) -> &mut Self {
        self.bullets.push(text.into());
        self
    }

    /// Prepares the alert container as a formatted ANSI string.
    #[must_use]
    pub fn prepare(&self) -> String {
        let badge_width = match self.kind {
            AlertKind::Info => 6,
            AlertKind::Success => 9,
            AlertKind::Warning => 9,
            AlertKind::Danger => 8,
        };

        let mut widest = badge_width + 3 + visible_width(&self.title);

        if let Some(ref msg) = self.message {
            for line in msg.lines() {
                widest = widest.max(visible_width(line));
            }
        }

        for bullet in &self.bullets {
            widest = widest.max(visible_width(bullet) + 4);
        }

        if let Some(ref sug) = self.suggestion {
            widest = widest.max(visible_width(sug) + 14);
        }

        if let Some(ref cons) = self.consequence {
            widest = widest.max(visible_width(cons) + 15);
        }

        let inner_width = widest.max(54) + 4;
        let border_color = self.kind.border_color();

        let top_border = format!("╭{}╮", "─".repeat(inner_width))
            .color(border_color)
            .to_string();
        let mid_divider = format!("├{}┤", "─".repeat(inner_width))
            .color(border_color)
            .to_string();
        let bot_border = format!("╰{}╯", "─".repeat(inner_width))
            .color(border_color)
            .to_string();
        let vertical_bar = "│".color(border_color).to_string();

        let mut lines = Vec::new();
        lines.push(top_border);

        // Header line: badge + title
        let header_content = format!("{} {}", self.kind.badge(), self.title.bold().white());
        let header_visible_len = badge_width + 1 + visible_width(&self.title);
        let header_pad = inner_width.saturating_sub(2 + header_visible_len);
        lines.push(format!(
            "{vertical_bar}  {header_content}{}{vertical_bar}",
            " ".repeat(header_pad)
        ));

        lines.push(mid_divider);

        if let Some(ref msg) = self.message {
            for line in msg.lines() {
                let pad = inner_width.saturating_sub(2 + visible_width(line));
                lines.push(format!(
                    "{vertical_bar}  {line}{}{vertical_bar}",
                    " ".repeat(pad)
                ));
            }
        }

        for item in &self.bullets {
            let bullet_line = format!("• {item}");
            let pad = inner_width.saturating_sub(2 + visible_width(&bullet_line));
            lines.push(format!(
                "{vertical_bar}  {bullet_line}{}{vertical_bar}",
                " ".repeat(pad)
            ));
        }

        if let Some(ref cons) = self.consequence {
            let cons_line = format!("{}: {cons}", "Consequence".bold().red());
            let cons_visible_len = 13 + visible_width(cons);
            let pad = inner_width.saturating_sub(2 + cons_visible_len);
            lines.push(format!(
                "{vertical_bar}  {cons_line}{}{vertical_bar}",
                " ".repeat(pad)
            ));
        }

        if let Some(ref sug) = self.suggestion {
            let sug_line = format!("{}: {sug}", "Suggestion".bold().cyan());
            let sug_visible_len = 12 + visible_width(sug);
            let pad = inner_width.saturating_sub(2 + sug_visible_len);
            lines.push(format!(
                "{vertical_bar}  {sug_line}{}{vertical_bar}",
                " ".repeat(pad)
            ));
        }

        lines.push(bot_border);
        lines.join("\n")
    }

    /// Renders the alert block directly to the provided UI manager.
    pub fn render(&self, ui: &Ui) {
        ui.writer().print(&self.prepare());
    }
}
