//! ANSI color codes, text styling, escape-sequence stripping, and visible width calculation.

use std::{
    fmt,
    io::IsTerminal,
    ops::Deref,
    sync::atomic::{AtomicI8, Ordering},
};

static COLOR_OVERRIDE: AtomicI8 = AtomicI8::new(-1);

/// Controls whether ANSI styling is emitted globally.
pub fn set_color_enabled(enabled: bool) {
    COLOR_OVERRIDE.store(if enabled { 1 } else { 0 }, Ordering::SeqCst);
}

/// Resets the global color override to automatic detection.
pub fn reset_color_override() {
    COLOR_OVERRIDE.store(-1, Ordering::SeqCst);
}

/// Checks whether ANSI coloring is currently active.
#[must_use]
pub fn is_color_enabled() -> bool {
    let current_override = COLOR_OVERRIDE.load(Ordering::SeqCst);
    if current_override == 1 {
        return true;
    }
    if current_override == 0 {
        return false;
    }

    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }

    if std::env::var("TERM").is_ok_and(|term| term == "dumb") {
        return false;
    }

    std::io::stdout().is_terminal()
}

/// Strips all ANSI escape sequences from a string.
#[must_use]
pub fn strip_ansi(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut in_escape = false;

    let chars = input.chars();
    for ch in chars {
        if in_escape {
            if ch.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else if ch == '\x1b' {
            in_escape = true;
        } else {
            result.push(ch);
        }
    }

    result
}

/// Measures the visible character width of a string in terminal cells, ignoring ANSI escape sequences.
#[must_use]
pub fn visible_width(input: &str) -> usize {
    let stripped = strip_ansi(input);
    let mut width = 0;
    for ch in stripped.chars() {
        if ch.is_control() {
            continue;
        }
        // Basic fullwidth / emoji detection range without heavy unicode tables
        let code = ch as u32;
        if (0x1100..=0x115F).contains(&code)
            || (0x2E80..=0xA4CF).contains(&code)
            || (0xAC00..=0xD7A3).contains(&code)
            || (0xF900..=0xFAFF).contains(&code)
            || (0xFE10..=0xFE19).contains(&code)
            || (0xFE30..=0xFE6F).contains(&code)
            || (0xFF00..=0xFF60).contains(&code)
            || (0xFFE0..=0xFFE6).contains(&code)
            || (0x1F300..=0x1F64F).contains(&code)
            || (0x1F900..=0x1F9FF).contains(&code)
        {
            width += 2;
        } else {
            width += 1;
        }
    }
    width
}

/// Terminal colors supported by the ANSI styler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// Standard Black (ANSI 30 / 40).
    Black,
    /// Standard Red (ANSI 31 / 41).
    Red,
    /// Standard Green (ANSI 32 / 42).
    Green,
    /// Standard Yellow (ANSI 33 / 43).
    Yellow,
    /// Standard Blue (ANSI 34 / 44).
    Blue,
    /// Standard Magenta (ANSI 35 / 45).
    Magenta,
    /// Standard Cyan (ANSI 36 / 46).
    Cyan,
    /// Standard White (ANSI 37 / 47).
    White,
    /// High-intensity Dark Grey / Bright Black (ANSI 90 / 100).
    BrightBlack,
    /// High-intensity Bright Red (ANSI 91 / 101).
    BrightRed,
    /// High-intensity Bright Green (ANSI 92 / 102).
    BrightGreen,
    /// High-intensity Bright Yellow (ANSI 93 / 103).
    BrightYellow,
    /// High-intensity Bright Blue (ANSI 94 / 104).
    BrightBlue,
    /// High-intensity Bright Magenta (ANSI 95 / 105).
    BrightMagenta,
    /// High-intensity Bright Cyan (ANSI 96 / 106).
    BrightCyan,
    /// High-intensity Bright White (ANSI 97 / 107).
    BrightWhite,
    /// Extended 256-color palette index.
    Ansi256(u8),
    /// Truecolor 24-bit RGB values.
    Rgb(u8, u8, u8),
    /// Terminal default color.
    Default,
}

impl Color {
    /// Alias for high-intensity dark grey.
    pub const DARK_GREY: Self = Self::BrightBlack;

    #[must_use]
    fn foreground_code(&self) -> String {
        match self {
            Self::Black => "30".to_string(),
            Self::Red => "31".to_string(),
            Self::Green => "32".to_string(),
            Self::Yellow => "33".to_string(),
            Self::Blue => "34".to_string(),
            Self::Magenta => "35".to_string(),
            Self::Cyan => "36".to_string(),
            Self::White => "37".to_string(),
            Self::BrightBlack => "90".to_string(),
            Self::BrightRed => "91".to_string(),
            Self::BrightGreen => "92".to_string(),
            Self::BrightYellow => "93".to_string(),
            Self::BrightBlue => "94".to_string(),
            Self::BrightMagenta => "95".to_string(),
            Self::BrightCyan => "96".to_string(),
            Self::BrightWhite => "97".to_string(),
            Self::Ansi256(code) => format!("38;5;{code}"),
            Self::Rgb(red, green, blue) => format!("38;2;{red};{green};{blue}"),
            Self::Default => "39".to_string(),
        }
    }

    #[must_use]
    fn background_code(&self) -> String {
        match self {
            Self::Black => "40".to_string(),
            Self::Red => "41".to_string(),
            Self::Green => "42".to_string(),
            Self::Yellow => "43".to_string(),
            Self::Blue => "44".to_string(),
            Self::Magenta => "45".to_string(),
            Self::Cyan => "46".to_string(),
            Self::White => "47".to_string(),
            Self::BrightBlack => "100".to_string(),
            Self::BrightRed => "101".to_string(),
            Self::BrightGreen => "102".to_string(),
            Self::BrightYellow => "103".to_string(),
            Self::BrightBlue => "104".to_string(),
            Self::BrightMagenta => "105".to_string(),
            Self::BrightCyan => "106".to_string(),
            Self::BrightWhite => "107".to_string(),
            Self::Ansi256(code) => format!("48;5;{code}"),
            Self::Rgb(red, green, blue) => format!("48;2;{red};{green};{blue}"),
            Self::Default => "49".to_string(),
        }
    }
}

/// Text decoration style flags.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StyleFlags {
    /// Bold or increased intensity.
    pub bold: bool,
    /// Dim or decreased intensity.
    pub dim: bool,
    /// Italic text.
    pub italic: bool,
    /// Single underline.
    pub underline: bool,
    /// Invert foreground and background.
    pub reverse: bool,
    /// Concealed or hidden text.
    pub hidden: bool,
    /// Strikethrough text.
    pub strikethrough: bool,
}

/// An ANSI-decorated text wrapper that prints styling when colors are active.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyledText {
    content: String,
    foreground: Option<Color>,
    background: Option<Color>,
    styles: StyleFlags,
}

impl StyledText {
    /// Wraps a raw string with default styling.
    #[must_use]
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            foreground: None,
            background: None,
            styles: StyleFlags::default(),
        }
    }

    /// Sets the foreground color.
    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.foreground = Some(color);
        self
    }

    /// Sets the background color.
    #[must_use]
    pub fn on_color(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Applies bold weight.
    #[must_use]
    pub fn bold(mut self) -> Self {
        self.styles.bold = true;
        self
    }

    /// Applies dim weight.
    #[must_use]
    pub fn dimmed(mut self) -> Self {
        self.styles.dim = true;
        self
    }

    /// Applies italic slant.
    #[must_use]
    pub fn italic(mut self) -> Self {
        self.styles.italic = true;
        self
    }

    /// Applies underline decoration.
    #[must_use]
    pub fn underline(mut self) -> Self {
        self.styles.underline = true;
        self
    }

    /// Reverses foreground and background colors.
    #[must_use]
    pub fn reverse(mut self) -> Self {
        self.styles.reverse = true;
        self
    }

    /// Conceals text content.
    #[must_use]
    pub fn hidden(mut self) -> Self {
        self.styles.hidden = true;
        self
    }

    /// Applies strikethrough decoration.
    #[must_use]
    pub fn strikethrough(mut self) -> Self {
        self.styles.strikethrough = true;
        self
    }

    /// Convenience methods for foreground colors.
    #[must_use]
    pub fn black(self) -> Self {
        self.color(Color::Black)
    }

    #[must_use]
    pub fn red(self) -> Self {
        self.color(Color::Red)
    }

    #[must_use]
    pub fn green(self) -> Self {
        self.color(Color::Green)
    }

    #[must_use]
    pub fn yellow(self) -> Self {
        self.color(Color::Yellow)
    }

    #[must_use]
    pub fn blue(self) -> Self {
        self.color(Color::Blue)
    }

    #[must_use]
    pub fn magenta(self) -> Self {
        self.color(Color::Magenta)
    }

    #[must_use]
    pub fn cyan(self) -> Self {
        self.color(Color::Cyan)
    }

    #[must_use]
    pub fn white(self) -> Self {
        self.color(Color::White)
    }

    #[must_use]
    pub fn bright_black(self) -> Self {
        self.color(Color::BrightBlack)
    }

    #[must_use]
    pub fn bright_red(self) -> Self {
        self.color(Color::BrightRed)
    }

    #[must_use]
    pub fn bright_green(self) -> Self {
        self.color(Color::BrightGreen)
    }

    #[must_use]
    pub fn bright_yellow(self) -> Self {
        self.color(Color::BrightYellow)
    }

    #[must_use]
    pub fn bright_blue(self) -> Self {
        self.color(Color::BrightBlue)
    }

    #[must_use]
    pub fn bright_magenta(self) -> Self {
        self.color(Color::BrightMagenta)
    }

    #[must_use]
    pub fn bright_cyan(self) -> Self {
        self.color(Color::BrightCyan)
    }

    #[must_use]
    pub fn bright_white(self) -> Self {
        self.color(Color::BrightWhite)
    }

    /// Convenience methods for background colors.
    #[must_use]
    pub fn on_black(self) -> Self {
        self.on_color(Color::Black)
    }

    #[must_use]
    pub fn on_red(self) -> Self {
        self.on_color(Color::Red)
    }

    #[must_use]
    pub fn on_green(self) -> Self {
        self.on_color(Color::Green)
    }

    #[must_use]
    pub fn on_yellow(self) -> Self {
        self.on_color(Color::Yellow)
    }

    #[must_use]
    pub fn on_blue(self) -> Self {
        self.on_color(Color::Blue)
    }

    #[must_use]
    pub fn on_magenta(self) -> Self {
        self.on_color(Color::Magenta)
    }

    #[must_use]
    pub fn on_cyan(self) -> Self {
        self.on_color(Color::Cyan)
    }

    #[must_use]
    pub fn on_white(self) -> Self {
        self.on_color(Color::White)
    }

    #[must_use]
    pub fn on_bright_black(self) -> Self {
        self.on_color(Color::BrightBlack)
    }

    #[must_use]
    pub fn on_bright_red(self) -> Self {
        self.on_color(Color::BrightRed)
    }

    #[must_use]
    pub fn on_bright_green(self) -> Self {
        self.on_color(Color::BrightGreen)
    }

    #[must_use]
    pub fn on_bright_yellow(self) -> Self {
        self.on_color(Color::BrightYellow)
    }

    #[must_use]
    pub fn on_bright_blue(self) -> Self {
        self.on_color(Color::BrightBlue)
    }

    #[must_use]
    pub fn on_bright_magenta(self) -> Self {
        self.on_color(Color::BrightMagenta)
    }

    #[must_use]
    pub fn on_bright_cyan(self) -> Self {
        self.on_color(Color::BrightCyan)
    }

    #[must_use]
    pub fn on_bright_white(self) -> Self {
        self.on_color(Color::BrightWhite)
    }

    /// Accesses the underlying unformatted string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.content
    }
}

impl Deref for StyledText {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl fmt::Display for StyledText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !is_color_enabled() {
            return write!(formatter, "{}", self.content);
        }

        let mut codes = Vec::new();

        if self.styles.bold {
            codes.push("1".to_string());
        }
        if self.styles.dim {
            codes.push("2".to_string());
        }
        if self.styles.italic {
            codes.push("3".to_string());
        }
        if self.styles.underline {
            codes.push("4".to_string());
        }
        if self.styles.reverse {
            codes.push("7".to_string());
        }
        if self.styles.hidden {
            codes.push("8".to_string());
        }
        if self.styles.strikethrough {
            codes.push("9".to_string());
        }

        if let Some(fg) = &self.foreground {
            codes.push(fg.foreground_code());
        }
        if let Some(bg) = &self.background {
            codes.push(bg.background_code());
        }

        if codes.is_empty() {
            write!(formatter, "{}", self.content)
        } else {
            write!(
                formatter,
                "\x1b[{}m{}\x1b[0m",
                codes.join(";"),
                self.content
            )
        }
    }
}

/// Extension trait providing chained ANSI coloring methods for string types.
pub trait Colorize {
    /// Starts styling this string value.
    #[must_use]
    fn styled(&self) -> StyledText;

    /// Colors text black.
    #[must_use]
    fn black(&self) -> StyledText {
        self.styled().black()
    }

    /// Colors text red.
    #[must_use]
    fn red(&self) -> StyledText {
        self.styled().red()
    }

    /// Colors text green.
    #[must_use]
    fn green(&self) -> StyledText {
        self.styled().green()
    }

    /// Colors text yellow.
    #[must_use]
    fn yellow(&self) -> StyledText {
        self.styled().yellow()
    }

    /// Colors text blue.
    #[must_use]
    fn blue(&self) -> StyledText {
        self.styled().blue()
    }

    /// Colors text magenta.
    #[must_use]
    fn magenta(&self) -> StyledText {
        self.styled().magenta()
    }

    /// Colors text cyan.
    #[must_use]
    fn cyan(&self) -> StyledText {
        self.styled().cyan()
    }

    /// Colors text white.
    #[must_use]
    fn white(&self) -> StyledText {
        self.styled().white()
    }

    /// Colors text bright black / dark grey.
    #[must_use]
    fn bright_black(&self) -> StyledText {
        self.styled().bright_black()
    }

    /// Colors text bright red.
    #[must_use]
    fn bright_red(&self) -> StyledText {
        self.styled().bright_red()
    }

    /// Colors text bright green.
    #[must_use]
    fn bright_green(&self) -> StyledText {
        self.styled().bright_green()
    }

    /// Colors text bright yellow.
    #[must_use]
    fn bright_yellow(&self) -> StyledText {
        self.styled().bright_yellow()
    }

    /// Colors text bright blue.
    #[must_use]
    fn bright_blue(&self) -> StyledText {
        self.styled().bright_blue()
    }

    /// Colors text bright magenta.
    #[must_use]
    fn bright_magenta(&self) -> StyledText {
        self.styled().bright_magenta()
    }

    /// Colors text bright cyan.
    #[must_use]
    fn bright_cyan(&self) -> StyledText {
        self.styled().bright_cyan()
    }

    /// Colors text bright white.
    #[must_use]
    fn bright_white(&self) -> StyledText {
        self.styled().bright_white()
    }

    /// Colors background black.
    #[must_use]
    fn on_black(&self) -> StyledText {
        self.styled().on_black()
    }

    /// Colors background red.
    #[must_use]
    fn on_red(&self) -> StyledText {
        self.styled().on_red()
    }

    /// Colors background green.
    #[must_use]
    fn on_green(&self) -> StyledText {
        self.styled().on_green()
    }

    /// Colors background yellow.
    #[must_use]
    fn on_yellow(&self) -> StyledText {
        self.styled().on_yellow()
    }

    /// Colors background blue.
    #[must_use]
    fn on_blue(&self) -> StyledText {
        self.styled().on_blue()
    }

    /// Colors background magenta.
    #[must_use]
    fn on_magenta(&self) -> StyledText {
        self.styled().on_magenta()
    }

    /// Colors background cyan.
    #[must_use]
    fn on_cyan(&self) -> StyledText {
        self.styled().on_cyan()
    }

    /// Colors background white.
    #[must_use]
    fn on_white(&self) -> StyledText {
        self.styled().on_white()
    }

    /// Colors background bright black.
    #[must_use]
    fn on_bright_black(&self) -> StyledText {
        self.styled().on_bright_black()
    }

    /// Colors background bright red.
    #[must_use]
    fn on_bright_red(&self) -> StyledText {
        self.styled().on_bright_red()
    }

    /// Colors background bright green.
    #[must_use]
    fn on_bright_green(&self) -> StyledText {
        self.styled().on_bright_green()
    }

    /// Colors background bright yellow.
    #[must_use]
    fn on_bright_yellow(&self) -> StyledText {
        self.styled().on_bright_yellow()
    }

    /// Colors background bright blue.
    #[must_use]
    fn on_bright_blue(&self) -> StyledText {
        self.styled().on_bright_blue()
    }

    /// Colors background bright magenta.
    #[must_use]
    fn on_bright_magenta(&self) -> StyledText {
        self.styled().on_bright_magenta()
    }

    /// Colors background bright cyan.
    #[must_use]
    fn on_bright_cyan(&self) -> StyledText {
        self.styled().on_bright_cyan()
    }

    /// Colors background bright white.
    #[must_use]
    fn on_bright_white(&self) -> StyledText {
        self.styled().on_bright_white()
    }

    /// Formats text with bold weight.
    #[must_use]
    fn bold(&self) -> StyledText {
        self.styled().bold()
    }

    /// Formats text with dim weight.
    #[must_use]
    fn dimmed(&self) -> StyledText {
        self.styled().dimmed()
    }

    /// Formats text with italic slant.
    #[must_use]
    fn italic(&self) -> StyledText {
        self.styled().italic()
    }

    /// Formats text with underline decoration.
    #[must_use]
    fn underline(&self) -> StyledText {
        self.styled().underline()
    }

    /// Applies custom foreground color.
    #[must_use]
    fn color(&self, color: Color) -> StyledText {
        self.styled().color(color)
    }

    /// Applies custom background color.
    #[must_use]
    fn on_color(&self, color: Color) -> StyledText {
        self.styled().on_color(color)
    }
}

impl Colorize for str {
    fn styled(&self) -> StyledText {
        StyledText::new(self)
    }
}

impl Colorize for String {
    fn styled(&self) -> StyledText {
        StyledText::new(self.as_str())
    }
}

/// Backward compatibility alias for the colorize extension trait.
pub use Colorize as ColorizeExt;
