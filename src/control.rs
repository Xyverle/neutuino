//! Utilities that help control the terminal
//!
//! These are built to work on Windows, Linux, and MacOS

#[cfg(unix)]
pub use crate::unix::{disable_raw_mode, enable_raw_mode, get_terminal_size};

#[cfg(windows)]
pub use crate::windows::{disable_raw_mode, enable_raw_mode, get_terminal_size};
