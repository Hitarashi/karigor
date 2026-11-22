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
#[cfg(feature = "async")]
pub use command::CommandFuture;
pub use command::{Argument, Command, CommandBuilder, Context, Flag, FlagType, FlagValue};
pub use error::{Error, KarigorError};
pub use kernel::Kernel;
pub use template::{publish_files, render as render_template};
pub use ui::{
    Action, Alert, AlertKind, Alignment, Attribute, Cell, Color, Colorize, ColorizeExt,
    ContentArrangement, DefinitionList, Instruction, ProgressBar, ProgressStyle, Steps, Sticker,
    Table, TablePreset, Ui,
};

/// Essential types and traits for building command-line applications.
pub mod prelude {
    pub use borno::StrExt;

    #[cfg(feature = "async")]
    pub use crate::CommandFuture;
    pub use crate::{
        Action, Alert, AlertKind, Alignment, Argument, Attribute, Cell, Color, Colorize,
        ColorizeExt, Command, CommandBuilder, ContentArrangement, Context, DefinitionList, Error,
        Flag, FlagType, FlagValue, Instruction, KarigorError, Kernel, ProgressBar, ProgressStyle,
        Steps, Sticker, Table, TablePreset, Ui,
    };
}
