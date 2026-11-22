//! Decoupled output writer interface supporting console streams and in-memory test capture buffers.

use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};

/// Target stream for terminal UI rendering.
#[derive(Debug, Clone, Default)]
pub enum UiWriter {
    /// Writes directly to process stdout and stderr.
    #[default]
    Console,
    /// Captures all written output into a thread-safe shared buffer.
    Buffer(Arc<Mutex<Vec<u8>>>),
}

impl UiWriter {
    /// Creates a new in-memory capturing writer.
    #[must_use]
    pub fn buffer() -> (Self, Arc<Mutex<Vec<u8>>>) {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        (Self::Buffer(Arc::clone(&buffer)), buffer)
    }

    /// Emits a normal output line.
    pub fn print(&self, message: &str) {
        match self {
            Self::Console => {
                let mut out = io::stdout().lock();
                let _ = writeln!(out, "{message}");
            }
            Self::Buffer(buffer) => {
                if let Ok(mut lock) = buffer.lock() {
                    let _ = writeln!(lock, "{message}");
                }
            }
        }
    }

    /// Emits an error output line.
    pub fn print_err(&self, message: &str) {
        match self {
            Self::Console => {
                let mut err = io::stderr().lock();
                let _ = writeln!(err, "{message}");
            }
            Self::Buffer(buffer) => {
                if let Ok(mut lock) = buffer.lock() {
                    let _ = writeln!(lock, "{message}");
                }
            }
        }
    }
}
