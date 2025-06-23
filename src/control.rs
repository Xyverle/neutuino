//! Collection of functions that help control the terminal
//!
//! These are built to work on Windows, Linux, and MacOS

use std::io;

#[cfg(unix)]
pub use crate::unix::{disable_raw_mode, enable_raw_mode, get_terminal_size};

#[cfg(windows)]
pub use crate::windows::{disable_raw_mode, enable_raw_mode, get_terminal_size};

/// Struct that calls `enable_raw_mode` on construction
/// and `disable_raw_mode` on destruction
///
/// Prefered over function as it calls `disable_raw_mode` on panic
pub struct RawModeHandler {
    enabled: bool,
}

impl RawModeHandler {
    /// Creates a new instance and sets the terminal to raw mode
    ///
    /// # Errors
    ///
    /// If there is no stdin,
    /// stdin is not a tty,
    /// or it fails to change terminal settings
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(Self { enabled: true })
    }
    /// Enables raw mode
    ///
    /// # Errors
    ///
    /// Never errors if raw mode is already enabled
    ///
    /// If there is no stdin,
    /// stdin is not a tty,
    /// or it fails to change terminal settings
    pub fn enable(&mut self) -> io::Result<()> {
        self.set(true)
    }
    /// Disables raw mode
    ///
    /// # Errors
    ///
    /// Never errors if raw mode is already disabled
    ///
    /// If there is no stdin,
    /// stdin is not a tty,
    /// or it fails to change terminal settings
    pub fn disable(&mut self) -> io::Result<()> {
        self.set(false)
    }
    /// Sets raw mode
    ///
    /// # Errors
    ///
    /// Never errors if raw mode is in the same state as the boolean
    ///
    /// If there is no stdin,
    /// stdin is not a tty,
    /// or it fails to change terminal settings
    pub fn set(&mut self, raw: bool) -> io::Result<()> {
        if self.enabled == raw {
            return Ok(());
        }
        if raw {
            enable_raw_mode()?;
        } else {
            disable_raw_mode()?;
        }
        self.enabled = raw;
        Ok(())
    }
    /// Gets if raw mode is enabled
    #[must_use]
    pub fn get(&self) -> bool {
        self.enabled
    }
}

impl Drop for RawModeHandler {
    fn drop(&mut self) {
        self.disable().expect("Failed to disable terminal raw mode");
    }
}
