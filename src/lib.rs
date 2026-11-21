//! Framework-agnostic, developer-friendly CLI application builder and terminal UI toolkit for Rust.
//!
//! # Architecture
//! - **Kernel**: Central application orchestrator managing command discovery, argument parsing,
//!   colon-delimited subcommands (`make:route` ↔ `make route`), and execution lifecycle.
//! - **Command Trait**: Declarative interface for defining commands, arguments, and flags.
//! - **Terminal UI**: Polished ANSI presentation primitives, styled tables, interactive prompts, and spinners.
//! - **Template Engine**: Embedded code generation templates powered by MiniJinja.

pub mod command;
pub mod error;
pub mod kernel;
pub mod template;
pub mod ui;

pub use borno::StrExt;
pub use command::{Argument, Command, CommandFuture, Context, Flag, FlagType, FlagValue};
pub use error::KarigorError;
pub use kernel::Kernel;
pub use template::{publish_files, render as render_template};
pub use ui::{Cell, Table, Ui};

/// Essential types and traits for building command-line applications.
pub mod prelude {
    pub use borno::StrExt;

    pub use crate::{
        Argument, Cell, Command, CommandFuture, Context, Flag, FlagType, FlagValue, KarigorError,
        Kernel, Table, Ui,
    };
}
