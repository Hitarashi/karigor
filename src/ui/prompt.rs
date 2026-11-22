//! Interactive console prompts with testable backends and non-interactive fallbacks.

use std::{
    io::{self, BufRead, IsTerminal, Write},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::ui::ansi::ColorizeExt;

static GLOBAL_NON_INTERACTIVE: AtomicBool = AtomicBool::new(false);

/// Configures whether terminal prompts globally operate in non-interactive mode.
pub fn set_non_interactive(non_interactive: bool) {
    GLOBAL_NON_INTERACTIVE.store(non_interactive, Ordering::SeqCst);
}

/// Returns whether terminal prompts are globally forced to operate non-interactively.
#[must_use]
pub fn is_non_interactive() -> bool {
    GLOBAL_NON_INTERACTIVE.load(Ordering::SeqCst)
        || std::env::var_os("KARIGOR_NON_INTERACTIVE").is_some()
        || std::env::var_os("CI").is_some()
        || std::env::var_os("GITHUB_ACTIONS").is_some()
}

/// Interface for interactive and automated prompt handlers.
pub trait PromptBackend: Send + Sync {
    /// Prompts the user for a text string.
    fn prompt_input(&self, prompt: &str, default: Option<&str>) -> io::Result<String>;
    /// Prompts the user for a boolean confirmation.
    fn prompt_confirm(&self, prompt: &str, default: bool) -> io::Result<bool>;
    /// Prompts the user to select one option from a list.
    fn prompt_select(&self, prompt: &str, items: &[String], default: usize) -> io::Result<usize>;
    /// Prompts the user for a masked or silent password.
    fn prompt_password(&self, prompt: &str) -> io::Result<String>;
    /// Prompts the user to select multiple options from a list.
    fn prompt_multiselect(
        &self,
        prompt: &str,
        items: &[String],
        defaults: &[usize],
    ) -> io::Result<Vec<usize>>;
}

#[cfg(unix)]
fn read_password_raw() -> io::Result<String> {
    use std::os::fd::AsRawFd;
    let stdin = io::stdin();
    let fd = stdin.as_raw_fd();
    if !stdin.is_terminal() {
        let mut line = String::new();
        stdin.lock().read_line(&mut line)?;
        return Ok(line.trim_end_matches(['\r', '\n']).to_string());
    }

    let mut termios = std::mem::MaybeUninit::uninit();
    // SAFETY: tcgetattr is called with a valid standard input descriptor.
    if unsafe { libc::tcgetattr(fd, termios.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }

    // SAFETY: termios is initialized by successful tcgetattr call.
    let orig = unsafe { termios.assume_init() };
    let mut modified = orig;
    modified.c_lflag &= !(libc::ECHO | libc::ECHONL);

    // SAFETY: tcsetattr is called with valid descriptor and modified termios flags.
    if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &modified) } != 0 {
        return Err(io::Error::last_os_error());
    }

    let mut line = String::new();
    let read_result = stdin.lock().read_line(&mut line);

    // SAFETY: Restoring original terminal attributes unconditionally.
    unsafe { libc::tcsetattr(fd, libc::TCSANOW, &orig) };
    read_result?;
    Ok(line.trim_end_matches(['\r', '\n']).to_string())
}

#[cfg(not(unix))]
fn read_password_raw() -> io::Result<String> {
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line.trim_end_matches(['\r', '\n']).to_string())
}

/// Standard interactive terminal prompt backend reading from stdin.
#[derive(Debug, Clone, Copy, Default)]
pub struct TerminalPromptBackend {
    force_non_interactive: bool,
}

impl TerminalPromptBackend {
    /// Creates a standard terminal prompt backend.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            force_non_interactive: false,
        }
    }

    /// Creates a terminal prompt backend explicitly configured for non-interactive execution.
    #[must_use]
    pub const fn non_interactive() -> Self {
        Self {
            force_non_interactive: true,
        }
    }

    fn is_interactive(&self) -> bool {
        if self.force_non_interactive || is_non_interactive() {
            return false;
        }
        io::stdin().is_terminal() && io::stdout().is_terminal()
    }
}

impl PromptBackend for TerminalPromptBackend {
    fn prompt_input(&self, prompt: &str, default: Option<&str>) -> io::Result<String> {
        if !self.is_interactive() {
            if let Some(fallback) = default {
                return Ok(fallback.to_string());
            }
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("Non-interactive environment: prompt `{prompt}` requires terminal input"),
            ));
        }

        let mut out = io::stdout().lock();
        let mut input = io::stdin().lock();
        prompt_input_stream(&mut input, &mut out, prompt, default)
    }

    fn prompt_confirm(&self, prompt: &str, default: bool) -> io::Result<bool> {
        if !self.is_interactive() {
            return Ok(default);
        }

        let mut out = io::stdout().lock();
        let mut input = io::stdin().lock();
        prompt_confirm_stream(&mut input, &mut out, prompt, default)
    }

    fn prompt_select(&self, prompt: &str, items: &[String], default: usize) -> io::Result<usize> {
        if items.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot select from an empty items list",
            ));
        }

        if !self.is_interactive() {
            if default < items.len() {
                return Ok(default);
            }
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!(
                    "Non-interactive environment: selection `{prompt}` requires terminal input"
                ),
            ));
        }

        let mut out = io::stdout().lock();
        let mut input = io::stdin().lock();
        prompt_select_stream(&mut input, &mut out, prompt, items, default)
    }

    fn prompt_password(&self, prompt: &str) -> io::Result<String> {
        if !self.is_interactive() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("Non-interactive environment: password `{prompt}` requires terminal input"),
            ));
        }

        let mut out = io::stdout().lock();
        write!(out, "{}: ", prompt.bold().white())?;
        out.flush()?;
        let secret = read_password_raw()?;
        writeln!(out)?;
        Ok(secret)
    }

    fn prompt_multiselect(
        &self,
        prompt: &str,
        items: &[String],
        defaults: &[usize],
    ) -> io::Result<Vec<usize>> {
        if items.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot select from an empty items list",
            ));
        }

        if !self.is_interactive() {
            return Ok(defaults.to_vec());
        }

        let mut out = io::stdout().lock();
        let mut input = io::stdin().lock();
        prompt_multiselect_stream(&mut input, &mut out, prompt, items, defaults)
    }
}

/// Prompts for text input reading from an arbitrary stream.
pub fn prompt_input_stream<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    prompt: &str,
    default: Option<&str>,
) -> io::Result<String> {
    if let Some(fallback) = default {
        write!(writer, "{} [{fallback}]: ", prompt.bold().white())?;
    } else {
        write!(writer, "{}: ", prompt.bold().white())?;
    }
    writer.flush()?;

    let mut line = String::new();
    reader.read_line(&mut line)?;
    let trimmed = line.trim();

    if trimmed.is_empty() {
        if let Some(fallback) = default {
            return Ok(fallback.to_string());
        }
        return Ok(String::new());
    }

    Ok(trimmed.to_string())
}

/// Prompts for boolean confirmation reading from an arbitrary stream.
pub fn prompt_confirm_stream<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    prompt: &str,
    default: bool,
) -> io::Result<bool> {
    let hint = if default { "Y/n" } else { "y/N" };
    write!(writer, "{} [{hint}]: ", prompt.bold().white())?;
    writer.flush()?;

    let mut line = String::new();
    reader.read_line(&mut line)?;
    let trimmed = line.trim().to_lowercase();

    if trimmed.is_empty() {
        return Ok(default);
    }

    match trimmed.as_str() {
        "y" | "yes" | "true" | "1" => Ok(true),
        "n" | "no" | "false" | "0" => Ok(false),
        _ => Ok(default),
    }
}

/// Prompts for item selection reading from an arbitrary stream.
pub fn prompt_select_stream<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    prompt: &str,
    items: &[String],
    default: usize,
) -> io::Result<usize> {
    if items.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Cannot select from an empty items list",
        ));
    }

    writeln!(writer, "{}:", prompt.bold().white())?;
    for (index, item) in items.iter().enumerate() {
        let marker = if index == default { ">" } else { " " };
        writeln!(writer, "  {marker} {}. {item}", index + 1)?;
    }
    write!(writer, "Select [1-{}]: ", items.len())?;
    writer.flush()?;

    let mut line = String::new();
    reader.read_line(&mut line)?;
    let trimmed = line.trim();

    if trimmed.is_empty() {
        return Ok(default);
    }

    if let Ok(choice) = trimmed.parse::<usize>()
        && (1..=items.len()).contains(&choice)
    {
        return Ok(choice - 1);
    }

    Ok(default)
}

/// Prompts for password input reading from an arbitrary stream.
pub fn prompt_password_stream<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    prompt: &str,
) -> io::Result<String> {
    write!(writer, "{}: ", prompt.bold().white())?;
    writer.flush()?;

    let mut line = String::new();
    reader.read_line(&mut line)?;
    Ok(line.trim_end_matches(['\r', '\n']).to_string())
}

/// Prompts for multiple item selection reading from an arbitrary stream.
pub fn prompt_multiselect_stream<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    prompt: &str,
    items: &[String],
    defaults: &[usize],
) -> io::Result<Vec<usize>> {
    if items.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Cannot select from an empty items list",
        ));
    }

    writeln!(writer, "{}:", prompt.bold().white())?;
    for (index, item) in items.iter().enumerate() {
        let is_default = defaults.contains(&index);
        let marker = if is_default { "[x]" } else { "[ ]" };
        writeln!(writer, "  {marker} {}. {item}", index + 1)?;
    }

    let default_hint: Vec<String> = defaults.iter().map(|i| (i + 1).to_string()).collect();
    if default_hint.is_empty() {
        write!(writer, "Select comma-separated [1-{}]: ", items.len())?;
    } else {
        write!(
            writer,
            "Select comma-separated [1-{}] (default: {}): ",
            items.len(),
            default_hint.join(", ")
        )?;
    }
    writer.flush()?;

    let mut line = String::new();
    reader.read_line(&mut line)?;
    let trimmed = line.trim();

    if trimmed.is_empty() {
        return Ok(defaults.to_vec());
    }

    let mut selected = Vec::new();
    for token in trimmed.split([',', ' ']) {
        let clean = token.trim();
        if clean.is_empty() {
            continue;
        }
        if let Ok(choice) = clean.parse::<usize>()
            && (1..=items.len()).contains(&choice)
        {
            let zero_index = choice - 1;
            if !selected.contains(&zero_index) {
                selected.push(zero_index);
            }
        }
    }

    selected.sort_unstable();
    Ok(selected)
}

/// In-memory mock prompt backend for deterministic automated test execution.
#[derive(Debug, Default)]
pub struct MockPromptBackend {
    inputs: Mutex<Vec<String>>,
    confirms: Mutex<Vec<bool>>,
    selects: Mutex<Vec<usize>>,
    passwords: Mutex<Vec<String>>,
    multiselects: Mutex<Vec<Vec<usize>>>,
}

impl MockPromptBackend {
    /// Creates a new empty mock prompt backend.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enqueues a pre-configured text input response.
    pub fn push_input(&self, value: impl Into<String>) {
        if let Ok(mut lock) = self.inputs.lock() {
            lock.push(value.into());
        }
    }

    /// Enqueues a pre-configured confirmation boolean response.
    pub fn push_confirm(&self, value: bool) {
        if let Ok(mut lock) = self.confirms.lock() {
            lock.push(value);
        }
    }

    /// Enqueues a pre-configured selection index response.
    pub fn push_select(&self, value: usize) {
        if let Ok(mut lock) = self.selects.lock() {
            lock.push(value);
        }
    }

    /// Enqueues a pre-configured password or secret response.
    pub fn push_password(&self, value: impl Into<String>) {
        if let Ok(mut lock) = self.passwords.lock() {
            lock.push(value.into());
        }
    }

    /// Enqueues a pre-configured multi-selection response.
    pub fn push_multiselect(&self, value: Vec<usize>) {
        if let Ok(mut lock) = self.multiselects.lock() {
            lock.push(value);
        }
    }
}

impl PromptBackend for MockPromptBackend {
    fn prompt_input(&self, prompt: &str, default: Option<&str>) -> io::Result<String> {
        let queued = self
            .inputs
            .lock()
            .ok()
            .filter(|lock| !lock.is_empty())
            .map(|mut lock| lock.remove(0));
        if let Some(val) = queued {
            return Ok(val);
        }
        default.map(ToString::to_string).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!("No mock input for `{prompt}`"),
            )
        })
    }

    fn prompt_confirm(&self, prompt: &str, default: bool) -> io::Result<bool> {
        let _ = prompt;
        let queued = self
            .confirms
            .lock()
            .ok()
            .filter(|lock| !lock.is_empty())
            .map(|mut lock| lock.remove(0));
        if let Some(val) = queued {
            return Ok(val);
        }
        Ok(default)
    }

    fn prompt_select(&self, prompt: &str, items: &[String], default: usize) -> io::Result<usize> {
        let _ = (prompt, items);
        let queued = self
            .selects
            .lock()
            .ok()
            .filter(|lock| !lock.is_empty())
            .map(|mut lock| lock.remove(0));
        if let Some(val) = queued {
            return Ok(val);
        }
        Ok(default)
    }

    fn prompt_password(&self, prompt: &str) -> io::Result<String> {
        let queued = self
            .passwords
            .lock()
            .ok()
            .filter(|lock| !lock.is_empty())
            .map(|mut lock| lock.remove(0));
        if let Some(val) = queued {
            return Ok(val);
        }
        Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("No mock password for `{prompt}`"),
        ))
    }

    fn prompt_multiselect(
        &self,
        prompt: &str,
        items: &[String],
        defaults: &[usize],
    ) -> io::Result<Vec<usize>> {
        let _ = (prompt, items);
        let queued = self
            .multiselects
            .lock()
            .ok()
            .filter(|lock| !lock.is_empty())
            .map(|mut lock| lock.remove(0));
        if let Some(val) = queued {
            return Ok(val);
        }
        Ok(defaults.to_vec())
    }
}

/// Global prompt handler wrapper.
#[derive(Clone)]
pub struct PromptHandler {
    backend: Arc<dyn PromptBackend>,
}

impl PromptHandler {
    /// Creates a new prompt handler backed by the given backend.
    #[must_use]
    pub fn new(backend: Arc<dyn PromptBackend>) -> Self {
        Self { backend }
    }

    /// Creates a standard terminal-backed handler.
    #[must_use]
    pub fn terminal() -> Self {
        Self::new(Arc::new(TerminalPromptBackend::new()))
    }

    /// Creates a mock-backed handler.
    #[must_use]
    pub fn mock(mock: Arc<MockPromptBackend>) -> Self {
        Self::new(mock)
    }

    /// Prompts for text input.
    pub fn input(&self, prompt: &str, default: Option<&str>) -> io::Result<String> {
        self.backend.prompt_input(prompt, default)
    }

    /// Prompts for boolean confirmation.
    pub fn confirm(&self, prompt: &str, default: bool) -> io::Result<bool> {
        self.backend.prompt_confirm(prompt, default)
    }

    /// Prompts for item selection.
    pub fn select<T: ToString>(
        &self,
        prompt: &str,
        items: &[T],
        default: usize,
    ) -> io::Result<usize> {
        let string_items: Vec<String> = items.iter().map(ToString::to_string).collect();
        self.backend.prompt_select(prompt, &string_items, default)
    }

    /// Prompts for a masked or silent password.
    pub fn password(&self, prompt: &str) -> io::Result<String> {
        self.backend.prompt_password(prompt)
    }

    /// Prompts for multiple item selection from a list.
    pub fn multiselect<T: ToString>(
        &self,
        prompt: &str,
        items: &[T],
        defaults: &[usize],
    ) -> io::Result<Vec<usize>> {
        let string_items: Vec<String> = items.iter().map(ToString::to_string).collect();
        self.backend
            .prompt_multiselect(prompt, &string_items, defaults)
    }
}

impl Default for PromptHandler {
    fn default() -> Self {
        Self::terminal()
    }
}
