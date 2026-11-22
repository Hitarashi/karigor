//! Terminal progress spinner and status activity indicator.

use std::{
    io::{IsTerminal, Write},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use crate::ui::ansi::ColorizeExt;

/// Visual formatting style for the progress spinner.
#[derive(Debug, Clone)]
pub struct ProgressStyle {
    template: String,
    tick_chars: Vec<char>,
}

impl ProgressStyle {
    /// Returns default spinner formatting style.
    #[must_use]
    pub fn default_spinner() -> Self {
        Self {
            template: "{spinner} {msg}".to_string(),
            tick_chars: "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏".chars().collect(),
        }
    }

    /// Sets the characters used for the ticking animation.
    #[must_use]
    pub fn tick_chars(mut self, chars: &str) -> Self {
        self.tick_chars = chars.chars().collect();
        self
    }

    /// Configures the string template.
    pub fn template(mut self, template: &str) -> Result<Self, &'static str> {
        self.template = template.to_string();
        Ok(self)
    }
}

/// Progress spinner for displaying active command execution progress.
#[derive(Debug, Clone)]
pub struct ProgressBar {
    message: Arc<Mutex<String>>,
    active: Arc<AtomicBool>,
    hidden: bool,
    start_time: Instant,
    tick_index: Arc<AtomicUsize>,
    tick_chars: Vec<char>,
}

impl ProgressBar {
    /// Creates a new progress spinner initialized with default styling.
    #[must_use]
    pub fn new_spinner() -> Self {
        let is_term = std::io::stdout().is_terminal() && std::io::stderr().is_terminal();
        let is_dumb = std::env::var("TERM").is_ok_and(|term| term == "dumb");
        let hidden = !is_term || is_dumb;

        Self {
            message: Arc::new(Mutex::new(String::new())),
            active: Arc::new(AtomicBool::new(true)),
            hidden,
            start_time: Instant::now(),
            tick_index: Arc::new(AtomicUsize::new(0)),
            tick_chars: "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏".chars().collect(),
        }
    }

    /// Creates a hidden progress indicator that produces no terminal output.
    #[must_use]
    pub fn hidden() -> Self {
        let mut spinner = Self::new_spinner();
        spinner.hidden = true;
        spinner
    }

    /// Configures whether the spinner is hidden or suppressed.
    #[must_use]
    pub const fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    /// Checks whether the spinner is currently suppressed or hidden.
    #[must_use]
    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    /// Configures the visual style of the spinner.
    pub fn set_style(&mut self, style: ProgressStyle) {
        self.tick_chars = style.tick_chars;
    }

    /// Updates the status message displayed alongside the spinner.
    pub fn set_message(&self, message: impl Into<String>) {
        if let Ok(mut lock) = self.message.lock() {
            *lock = message.into();
        }
    }

    /// Enables periodic automatic ticking in a background thread.
    pub fn enable_steady_tick(&self, interval: Duration) {
        if self.hidden {
            return;
        }

        let active = Arc::clone(&self.active);
        let message = Arc::clone(&self.message);
        let tick_index = Arc::clone(&self.tick_index);
        let tick_chars = self.tick_chars.clone();

        thread::spawn(move || {
            while active.load(Ordering::SeqCst) {
                let current_index = tick_index.fetch_add(1, Ordering::SeqCst);
                let char_idx = current_index % tick_chars.len();
                let spinner_char = tick_chars[char_idx];
                let msg = message
                    .lock()
                    .map_or_else(|_| String::new(), |guard| guard.clone());

                let formatted = format!("\r\x1b[2K{} {}", spinner_char.to_string().cyan(), msg);
                let mut out = std::io::stderr().lock();
                let _ = out.write_all(formatted.as_bytes());
                let _ = out.flush();

                thread::sleep(interval);
            }
        });
    }

    /// Advances the spinner animation by a single tick.
    pub fn tick(&self) {
        if self.hidden {
            return;
        }
        let current_index = self.tick_index.fetch_add(1, Ordering::SeqCst);
        let char_idx = current_index % self.tick_chars.len();
        let spinner_char = self.tick_chars[char_idx];
        let msg = self
            .message
            .lock()
            .map_or_else(|_| String::new(), |guard| guard.clone());

        let formatted = format!("\r\x1b[2K{} {}", spinner_char.to_string().cyan(), msg);
        let mut out = std::io::stderr().lock();
        let _ = out.write_all(formatted.as_bytes());
        let _ = out.flush();
    }

    /// Cleans up and clears the spinner line from the terminal.
    pub fn finish_and_clear(&self) {
        self.active.store(false, Ordering::SeqCst);
        if !self.hidden {
            let mut out = std::io::stderr().lock();
            let _ = out.write_all(b"\r\x1b[2K\r");
            let _ = out.flush();
        }
    }

    /// Concludes the spinner and prints a final message line.
    pub fn finish_with_message(&self, message: &str) {
        self.active.store(false, Ordering::SeqCst);
        if !self.hidden {
            let mut out = std::io::stderr().lock();
            let formatted = format!("\r\x1b[2K{message}\n");
            let _ = out.write_all(formatted.as_bytes());
            let _ = out.flush();
        }
    }

    /// Returns the elapsed execution duration since the spinner started.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}
