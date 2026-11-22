//! Pre-styled terminal table formatting with rounded UTF-8 borders, padding, and cell alignments.

use std::fmt;

use crate::ui::ansi::{Color, ColorizeExt, visible_width};

/// Text alignment within a table cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Alignment {
    /// Align text to the left margin.
    #[default]
    Left,
    /// Center text within the column width.
    Center,
    /// Align text to the right margin.
    Right,
}

/// Text decoration attribute for table cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attribute {
    /// Bold text weight.
    Bold,
    /// Dim text weight.
    Dim,
    /// Italic text slant.
    Italic,
    /// Single underline.
    Underline,
}

/// Content layout arrangement modes for table cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContentArrangement {
    /// Content wrapping is disabled and preserves exact cell bounds.
    #[default]
    Disabled,
    /// Content automatically reflows to terminal dimensions.
    Dynamic,
}

/// Border styling presets for tabular rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TablePreset {
    /// Elegant rounded border presentation.
    #[default]
    Rounded,
    /// Plain borderless presentation.
    Plain,
    /// Compact single-line divider presentation.
    Compact,
}

/// Individual table cell with content, styling, and alignment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    content: String,
    foreground: Option<Color>,
    background: Option<Color>,
    alignment: Alignment,
    bold: bool,
    dim: bool,
}

impl Cell {
    /// Creates a new cell with the given textual content.
    #[must_use]
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            foreground: None,
            background: None,
            alignment: Alignment::Left,
            bold: false,
            dim: false,
        }
    }

    /// Sets the foreground color of the cell.
    #[must_use]
    pub fn fg(mut self, color: Color) -> Self {
        self.foreground = Some(color);
        self
    }

    /// Sets the background color of the cell.
    #[must_use]
    pub fn bg(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Sets the horizontal alignment for cell contents.
    #[must_use]
    pub fn align(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Adds a formatting attribute to the cell.
    #[must_use]
    pub fn add_attribute(mut self, attribute: Attribute) -> Self {
        match attribute {
            Attribute::Bold => self.bold = true,
            Attribute::Dim => self.dim = true,
            Attribute::Italic | Attribute::Underline => {}
        }
        self
    }

    /// Returns the raw text content of the cell.
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    fn render_line(&self, line: &str, width: usize) -> String {
        let text_width = visible_width(line);
        let padding = width.saturating_sub(text_width);

        let (left_pad, right_pad) = match self.alignment {
            Alignment::Left => (0, padding),
            Alignment::Right => (padding, 0),
            Alignment::Center => {
                let left = padding / 2;
                let right = padding - left;
                (left, right)
            }
        };

        let mut styled = line.styled();
        if let Some(fg) = self.foreground {
            styled = styled.color(fg);
        }
        if let Some(bg) = self.background {
            styled = styled.on_color(bg);
        }
        if self.bold {
            styled = styled.bold();
        }
        if self.dim {
            styled = styled.dimmed();
        }

        format!("{}{styled}{}", " ".repeat(left_pad), " ".repeat(right_pad))
    }
}

impl<T: Into<String>> From<T> for Cell {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

/// Formatted tabular view with borders, auto-column sizing, and cell alignments.
#[derive(Debug, Clone, Default)]
pub struct Table {
    header: Option<Vec<Cell>>,
    rows: Vec<Vec<Cell>>,
    preset: TablePreset,
    content_arrangement: ContentArrangement,
}

impl Table {
    /// Creates a new empty table.
    #[must_use]
    pub fn new() -> Self {
        Self {
            header: None,
            rows: Vec::new(),
            preset: TablePreset::Rounded,
            content_arrangement: ContentArrangement::Disabled,
        }
    }

    /// Configures the header row.
    pub fn set_header<I>(&mut self, cells: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Into<Cell>,
    {
        self.header = Some(cells.into_iter().map(Into::into).collect());
        self
    }

    /// Adds a data row to the table.
    pub fn add_row<I>(&mut self, row: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Into<Cell>,
    {
        self.rows.push(row.into_iter().map(Into::into).collect());
        self
    }

    /// Sets the layout preset style for borders.
    pub fn load_style(&mut self, preset: TablePreset) -> &mut Self {
        self.preset = preset;
        self
    }

    /// Sets the content arrangement mode.
    pub fn set_content_arrangement(&mut self, arrangement: ContentArrangement) -> &mut Self {
        self.content_arrangement = arrangement;
        self
    }

    fn calculate_column_widths(&self) -> Vec<usize> {
        let mut column_count = 0;
        if let Some(ref header) = self.header {
            column_count = column_count.max(header.len());
        }
        for row in &self.rows {
            column_count = column_count.max(row.len());
        }

        let mut widths = vec![0; column_count];

        if let Some(ref header) = self.header {
            for (index, cell) in header.iter().enumerate() {
                for line in cell.content().lines() {
                    widths[index] = widths[index].max(visible_width(line));
                }
            }
        }

        for row in &self.rows {
            for (index, cell) in row.iter().enumerate() {
                for line in cell.content().lines() {
                    widths[index] = widths[index].max(visible_width(line));
                }
            }
        }

        widths
    }
}

impl fmt::Display for Table {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let column_widths = self.calculate_column_widths();
        if column_widths.is_empty() {
            return Ok(());
        }

        let (top_left, top_mid, top_right) = match self.preset {
            TablePreset::Rounded => ("╭", "┬", "╮"),
            TablePreset::Plain => ("", "", ""),
            TablePreset::Compact => ("", "", ""),
        };

        let (mid_left, mid_mid, mid_right) = match self.preset {
            TablePreset::Rounded => ("├", "┼", "┤"),
            TablePreset::Plain => ("", "", ""),
            TablePreset::Compact => ("-", "+", "-"),
        };

        let (bot_left, bot_mid, bot_right) = match self.preset {
            TablePreset::Rounded => ("╰", "┴", "╯"),
            TablePreset::Plain => ("", "", ""),
            TablePreset::Compact => ("", "", ""),
        };

        let v_bar = match self.preset {
            TablePreset::Rounded => "│",
            TablePreset::Plain => " ",
            TablePreset::Compact => "|",
        };

        let h_bar = match self.preset {
            TablePreset::Rounded | TablePreset::Compact => "─",
            TablePreset::Plain => "",
        };

        // Render top border
        if self.preset == TablePreset::Rounded {
            let mut top_line = top_left.to_string();
            for (index, width) in column_widths.iter().enumerate() {
                top_line.push_str(&h_bar.repeat(width + 2));
                if index + 1 < column_widths.len() {
                    top_line.push_str(top_mid);
                }
            }
            top_line.push_str(top_right);
            writeln!(formatter, "{top_line}")?;
        }

        // Render header if present
        if let Some(ref header) = self.header {
            let mut max_lines = 1;
            let mut cell_lines: Vec<Vec<&str>> = Vec::new();
            for cell in header {
                let lines: Vec<&str> = cell.content().lines().collect();
                max_lines = max_lines.max(lines.len().max(1));
                cell_lines.push(lines);
            }

            for line_idx in 0..max_lines {
                let mut row_str = v_bar.to_string();
                for (col_idx, width) in column_widths.iter().enumerate() {
                    let cell = header.get(col_idx);
                    let line_content = cell_lines
                        .get(col_idx)
                        .and_then(|lines| lines.get(line_idx).copied())
                        .unwrap_or("");
                    let rendered = cell.map_or_else(
                        || " ".repeat(*width),
                        |c| c.render_line(line_content, *width),
                    );
                    row_str.push_str(&format!(" {rendered} {v_bar}"));
                }
                writeln!(formatter, "{row_str}")?;
            }

            // Divider after header
            if self.preset != TablePreset::Plain {
                let mut div_line = mid_left.to_string();
                for (index, width) in column_widths.iter().enumerate() {
                    div_line.push_str(&h_bar.repeat(width + 2));
                    if index + 1 < column_widths.len() {
                        div_line.push_str(mid_mid);
                    }
                }
                div_line.push_str(mid_right);
                writeln!(formatter, "{div_line}")?;
            }
        }

        // Render data rows
        for (row_idx, row) in self.rows.iter().enumerate() {
            let mut max_lines = 1;
            let mut cell_lines: Vec<Vec<&str>> = Vec::new();
            for cell in row {
                let lines: Vec<&str> = cell.content().lines().collect();
                max_lines = max_lines.max(lines.len().max(1));
                cell_lines.push(lines);
            }

            for line_idx in 0..max_lines {
                let mut row_str = v_bar.to_string();
                for (col_idx, width) in column_widths.iter().enumerate() {
                    let cell = row.get(col_idx);
                    let line_content = cell_lines
                        .get(col_idx)
                        .and_then(|lines| lines.get(line_idx).copied())
                        .unwrap_or("");
                    let rendered = cell.map_or_else(
                        || " ".repeat(*width),
                        |c| c.render_line(line_content, *width),
                    );
                    row_str.push_str(&format!(" {rendered} {v_bar}"));
                }
                writeln!(formatter, "{row_str}")?;
            }

            // Row dividers for compact if needed, or spacing
            if self.preset == TablePreset::Compact && row_idx + 1 < self.rows.len() {
                // Keep compact rows clean
            }
        }

        // Render bottom border
        if self.preset == TablePreset::Rounded {
            let mut bot_line = bot_left.to_string();
            for (index, width) in column_widths.iter().enumerate() {
                bot_line.push_str(&h_bar.repeat(width + 2));
                if index + 1 < column_widths.len() {
                    bot_line.push_str(bot_mid);
                }
            }
            bot_line.push_str(bot_right);
            write!(formatter, "{bot_line}")?;
        }

        Ok(())
    }
}

/// Helper module maintaining compatibility with the previous comfy-table preset paths.
pub mod presets {
    use super::TablePreset;

    /// Rounded UTF-8 full border preset.
    pub const UTF8_FULL: TablePreset = TablePreset::Rounded;

    impl TablePreset {
        /// Fluent builder returning rounded styling.
        #[must_use]
        pub const fn with_rounded_corners(self) -> Self {
            Self::Rounded
        }
    }
}
