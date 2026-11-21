//! Error types for CLI command execution, argument parsing, and terminal interaction.

use std::{fmt, io};

/// Errors encountered during command execution or kernel dispatch.
#[derive(Debug)]
pub enum KarigorError {
    /// Command not found in the registry.
    CommandNotFound {
        /// The name of the unrecognized command.
        name: String,
        /// Optional closest matching command name.
        suggestion: Option<String>,
    },
    /// Invalid argument or missing required argument.
    InvalidArgument(String),
    /// Invalid flag or missing flag value.
    InvalidFlag(String),
    /// Command execution failure.
    Execution(Box<dyn std::error::Error + Send + Sync>),
    /// Template rendering failure.
    Template(minijinja::Error),
    /// Underlying I/O error.
    Io(io::Error),
}

impl fmt::Display for KarigorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CommandNotFound { name, suggestion } => {
                if let Some(candidate) = suggestion {
                    write!(
                        formatter,
                        "Unknown command `{name}`. Did you mean `{candidate}`?"
                    )
                } else {
                    write!(formatter, "Unknown command `{name}`")
                }
            }
            Self::InvalidArgument(msg) => write!(formatter, "Invalid argument: {msg}"),
            Self::InvalidFlag(msg) => write!(formatter, "Invalid flag: {msg}"),
            Self::Execution(err) => write!(formatter, "Command execution error: {err}"),
            Self::Template(err) => write!(formatter, "Template error: {err}"),
            Self::Io(err) => write!(formatter, "I/O error: {err}"),
        }
    }
}

impl std::error::Error for KarigorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Execution(err) => err.source(),
            Self::Template(err) => Some(err),
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for KarigorError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<minijinja::Error> for KarigorError {
    fn from(err: minijinja::Error) -> Self {
        Self::Template(err)
    }
}
