//! Utilities that help control the terminal
//!
//! These are built to work on Windows, Linux, and MacOS

use std::io;

#[cfg(unix)]
pub use crate::unix::{disable_raw_mode, enable_raw_mode, get_terminal_size};

#[cfg(windows)]
pub use crate::windows::{disable_raw_mode, enable_raw_mode, get_terminal_size};

fn raw_mode(bool: bool) -> std::io::Result<()> {
    if bool {
        enable_raw_mode()?;
    } else {
        disable_raw_mode()?;
    }
    Ok(())
}

/// Creates a handler for raw mode
pub fn raw_mode_handler() -> io::Result<crate::Handler> {
    crate::Handler::new(&raw_mode)
}
