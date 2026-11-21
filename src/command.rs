//! Command trait definitions, typed arguments, flags, and execution context.

use std::{collections::HashMap, future::Future, pin::Pin};

use crate::{error::KarigorError, ui::Ui};

/// Type definition for the future returned by a command's asynchronous execution lifecycle.
pub type CommandFuture<'a> = Pin<Box<dyn Future<Output = Result<(), KarigorError>> + Send + 'a>>;

/// Value representations for command-line flags.
#[derive(Debug, Clone, PartialEq)]
pub enum FlagValue {
    /// Boolean toggle flag (e.g. `--force`, `--dry-run`).
    Bool(bool),
    /// Parameterized string flag (e.g. `--port 5555`, `--host 127.0.0.1`).
    Str(String),
}

/// Supported data types for command-line options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlagType {
    /// Flag acts as a boolean switch.
    Boolean,
    /// Flag accepts an associated value.
    String,
}

/// Specification for a positional command argument.
#[derive(Debug, Clone)]
pub struct Argument {
    /// Identifier name of the positional argument.
    pub name: String,
    /// Human-readable description displayed in help listings.
    pub description: String,
    /// Whether the argument must be supplied by the caller.
    pub required: bool,
    /// Optional default fallback value if omitted.
    pub default: Option<String>,
}

impl Argument {
    /// Creates a new required positional argument.
    #[must_use]
    pub fn required(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            required: true,
            default: None,
        }
    }

    /// Creates a new optional positional argument with an optional default.
    #[must_use]
    pub fn optional(
        name: impl Into<String>,
        description: impl Into<String>,
        default: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            required: false,
            default,
        }
    }
}

/// Specification for a named command flag/option.
#[derive(Debug, Clone)]
pub struct Flag {
    /// Primary long name of the flag (e.g. `"force"`).
    pub name: String,
    /// Optional single-character short identifier (e.g. `'f'`).
    pub short: Option<char>,
    /// Human-readable description displayed in help listings.
    pub description: String,
    /// Value type expected by the flag.
    pub flag_type: FlagType,
    /// Optional default fallback value for string options.
    pub default: Option<String>,
}

impl Flag {
    /// Creates a boolean toggle flag.
    #[must_use]
    pub fn boolean(
        name: impl Into<String>,
        short: Option<char>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            short,
            description: description.into(),
            flag_type: FlagType::Boolean,
            default: None,
        }
    }

    /// Creates a string option accepting an associated value.
    #[must_use]
    pub fn string(
        name: impl Into<String>,
        short: Option<char>,
        description: impl Into<String>,
        default: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            short,
            description: description.into(),
            flag_type: FlagType::String,
            default,
        }
    }
}

/// Execution context provided to commands during runtime execution.
#[derive(Debug, Clone)]
pub struct Context {
    /// Parsed positional arguments indexed by name.
    pub args: HashMap<String, String>,
    /// Parsed flags and options indexed by name.
    pub flags: HashMap<String, FlagValue>,
    /// Raw unparsed positional arguments.
    pub raw_args: Vec<String>,
    /// Interactive terminal UI instance.
    pub ui: Ui,
}

impl Context {
    /// Retrieves a positional argument value by name.
    #[must_use]
    pub fn arg(&self, name: &str) -> Option<&str> {
        self.args.get(name).map(String::as_str)
    }

    /// Retrieves a boolean flag value, defaulting to `false` if not set.
    #[must_use]
    pub fn flag_bool(&self, name: &str) -> bool {
        match self.flags.get(name) {
            Some(FlagValue::Bool(val)) => *val,
            _ => false,
        }
    }

    /// Retrieves a string flag value by name if set.
    #[must_use]
    pub fn flag_str(&self, name: &str) -> Option<&str> {
        match self.flags.get(name) {
            Some(FlagValue::Str(val)) => Some(val.as_str()),
            _ => None,
        }
    }
}

/// Trait implemented by CLI commands to define metadata, arguments, flags, and execution logic.
pub trait Command: Send + Sync {
    /// Primary invocation identifier for the command (e.g. `"build"`, `"make:route"`).
    fn name(&self) -> &'static str;

    /// Short human-readable summary of what the command performs.
    fn description(&self) -> &'static str;

    /// Alternative command aliases that resolve to this command.
    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }

    /// Positional arguments accepted by the command.
    fn arguments(&self) -> Vec<Argument> {
        Vec::new()
    }

    /// Flags and options accepted by the command.
    fn flags(&self) -> Vec<Flag> {
        Vec::new()
    }

    /// Executes the command lifecycle asynchronously.
    fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a>;
}
