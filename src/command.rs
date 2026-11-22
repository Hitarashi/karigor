//! Command trait definitions, typed arguments, flags, and execution context.

use std::collections::HashMap;
#[cfg(feature = "async")]
use std::{future::Future, pin::Pin};

use crate::{error::Error, ui::Ui};

#[cfg(feature = "async")]
/// Type definition for the future returned by a command's asynchronous execution lifecycle.
pub type CommandFuture<'a> = Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>;

/// Handler function for synchronous command execution closures.
pub type CommandHandler = Box<dyn Fn(&mut Context) -> Result<(), Error> + Send + Sync>;

#[cfg(feature = "async")]
/// Handler function for asynchronous command execution closures.
pub type AsyncCommandHandler =
    Box<dyn for<'a> Fn(&'a mut Context) -> CommandFuture<'a> + Send + Sync>;

/// Value representations for command-line flags.
#[derive(Debug, Clone, PartialEq)]
pub enum FlagValue {
    /// Boolean toggle flag (e.g. `--force`, `--dry-run`).
    Bool(bool),
    /// Parameterized string flag (e.g. `--port 5555`, `--host 127.0.0.1`).
    Str(String),
    /// Repeatable multi-value flag (e.g. `--include dir1 --include dir2`).
    Array(Vec<String>),
}

/// Supported data types for command-line options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlagType {
    /// Flag acts as a boolean switch.
    Boolean,
    /// Flag accepts an associated value.
    String,
    /// Flag accepts multiple repeatable values.
    Array,
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
    /// Whether the argument captures multiple trailing positional tokens.
    pub variadic: bool,
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
            variadic: false,
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
            variadic: false,
        }
    }

    /// Creates a variadic positional argument capturing all trailing positional parameters.
    #[must_use]
    pub fn variadic(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            required: false,
            default: None,
            variadic: true,
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
    /// Allowed value choices if constrained.
    pub choices: Vec<String>,
    /// Whether this flag must be explicitly supplied.
    pub required: bool,
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
            choices: Vec::new(),
            required: false,
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
            choices: Vec::new(),
            required: false,
        }
    }

    /// Creates a repeatable array option accepting multiple values.
    #[must_use]
    pub fn array(
        name: impl Into<String>,
        short: Option<char>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            short,
            description: description.into(),
            flag_type: FlagType::Array,
            default: None,
            choices: Vec::new(),
            required: false,
        }
    }

    /// Restricts allowed values to an explicit list of choices.
    #[must_use]
    pub fn choices<I, S>(mut self, choices: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.choices = choices.into_iter().map(Into::into).collect();
        self
    }

    /// Marks this flag as mandatory.
    #[must_use]
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

/// Execution context provided to commands during runtime execution.
#[derive(Debug, Clone)]
pub struct Context {
    /// Parsed positional arguments indexed by name.
    pub args: HashMap<String, String>,
    /// Parsed variadic arguments indexed by name.
    pub arg_lists: HashMap<String, Vec<String>>,
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

    /// Retrieves a variadic argument token list by name.
    #[must_use]
    pub fn arg_list(&self, name: &str) -> Vec<&str> {
        self.arg_lists
            .get(name)
            .map_or_else(Vec::new, |items| items.iter().map(String::as_str).collect())
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

    /// Retrieves a repeatable array flag's values by name.
    #[must_use]
    pub fn flag_array(&self, name: &str) -> Vec<&str> {
        match self.flags.get(name) {
            Some(FlagValue::Array(items)) => items.iter().map(String::as_str).collect(),
            Some(FlagValue::Str(item)) => vec![item.as_str()],
            _ => Vec::new(),
        }
    }

    /// Prompts the user for a text string.
    pub fn input(&self, prompt: &str, default: Option<&str>) -> std::io::Result<String> {
        self.ui.input(prompt, default)
    }

    /// Prompts the user for boolean confirmation.
    pub fn confirm(&self, prompt: &str, default: bool) -> std::io::Result<bool> {
        self.ui.confirm(prompt, default)
    }

    /// Prompts the user to select an item from a list.
    pub fn select<T: ToString + std::fmt::Display>(
        &self,
        prompt: &str,
        items: &[T],
        default: usize,
    ) -> std::io::Result<usize> {
        self.ui.select(prompt, items, default)
    }

    /// Prompts the user for a masked or silent password.
    pub fn password(&self, prompt: &str) -> std::io::Result<String> {
        self.ui.password(prompt)
    }

    /// Prompts the user to select multiple items from a list.
    pub fn multiselect<T: ToString + std::fmt::Display>(
        &self,
        prompt: &str,
        items: &[T],
        defaults: &[usize],
    ) -> std::io::Result<Vec<usize>> {
        self.ui.multiselect(prompt, items, defaults)
    }
}

/// Trait implemented by CLI commands to define metadata, arguments, flags, and execution logic.
pub trait Command: Send + Sync {
    /// Primary invocation identifier for the command (e.g. `"build"`, `"make:route"`).
    fn name(&self) -> &str;

    /// Short human-readable summary of what the command performs.
    fn description(&self) -> &str;

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

    /// Executes the command synchronously.
    fn run(&self, ctx: &mut Context) -> Result<(), Error> {
        let _ = ctx;
        Err(Error::Execution(
            format!(
                "Command `{}` does not provide a synchronous execution handler",
                self.name()
            )
            .into(),
        ))
    }

    #[cfg(feature = "async")]
    /// Executes the command asynchronously.
    fn run_async<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
        let result = self.run(ctx);
        Box::pin(std::future::ready(result))
    }
}

/// Fluent and closure-based builder for defining commands without boilerplate structs.
pub struct CommandBuilder {
    name: &'static str,
    description: &'static str,
    aliases: &'static [&'static str],
    arguments: Vec<Argument>,
    flags: Vec<Flag>,
    handler: Option<CommandHandler>,
    #[cfg(feature = "async")]
    async_handler: Option<AsyncCommandHandler>,
}

impl CommandBuilder {
    /// Creates a new command builder with the given name and description.
    #[must_use]
    pub fn new(name: &'static str, description: &'static str) -> Self {
        Self {
            name,
            description,
            aliases: &[],
            arguments: Vec::new(),
            flags: Vec::new(),
            handler: None,
            #[cfg(feature = "async")]
            async_handler: None,
        }
    }

    /// Sets alternative aliases for this command.
    #[must_use]
    pub fn aliases(mut self, aliases: &'static [&'static str]) -> Self {
        self.aliases = aliases;
        self
    }

    /// Appends a positional argument specification.
    #[must_use]
    pub fn argument(mut self, argument: Argument) -> Self {
        self.arguments.push(argument);
        self
    }

    /// Appends a flag or option specification.
    #[must_use]
    pub fn flag(mut self, flag: Flag) -> Self {
        self.flags.push(flag);
        self
    }

    /// Configures the synchronous execution closure for this command.
    #[must_use]
    pub fn handle<F>(mut self, handler: F) -> Self
    where
        F: Fn(&mut Context) -> Result<(), Error> + Send + Sync + 'static,
    {
        self.handler = Some(Box::new(handler));
        self
    }

    #[cfg(feature = "async")]
    /// Configures the asynchronous execution closure for this command.
    #[must_use]
    pub fn handle_async<F>(mut self, handler: F) -> Self
    where
        F: for<'a> Fn(&'a mut Context) -> CommandFuture<'a> + Send + Sync + 'static,
    {
        self.async_handler = Some(Box::new(handler));
        self
    }
}

impl Command for CommandBuilder {
    fn name(&self) -> &str {
        self.name
    }

    fn description(&self) -> &str {
        self.description
    }

    fn aliases(&self) -> &'static [&'static str] {
        self.aliases
    }

    fn arguments(&self) -> Vec<Argument> {
        self.arguments.clone()
    }

    fn flags(&self) -> Vec<Flag> {
        self.flags.clone()
    }

    fn run(&self, ctx: &mut Context) -> Result<(), Error> {
        if let Some(ref handler) = self.handler {
            handler(ctx)
        } else {
            let _ = ctx;
            Err(Error::Execution(
                format!(
                    "Command `{}` does not provide a synchronous execution handler",
                    self.name
                )
                .into(),
            ))
        }
    }

    #[cfg(feature = "async")]
    fn run_async<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
        if let Some(ref async_handler) = self.async_handler {
            async_handler(ctx)
        } else if let Some(ref sync_handler) = self.handler {
            let result = sync_handler(ctx);
            Box::pin(std::future::ready(result))
        } else {
            let _ = ctx;
            Box::pin(std::future::ready(Err(Error::Execution(
                format!(
                    "Command `{}` does not provide an execution handler",
                    self.name
                )
                .into(),
            ))))
        }
    }
}
