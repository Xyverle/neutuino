#![warn(clippy::all, clippy::pedantic)]
// this lint has way too many false positives
#![allow(clippy::doc_markdown)]
//! This crate is a simple and minimal TUI library that supports the following OSes:
//! - Windows 10+
//! - MacOS (currently untested)
//! - Linux
//!
//! ## Roadmap
//! - [x] Output (Unix)
//! - [x] Output (Windows)
//! - [x] Input (Unix) (Appears to work, more testing needed)
//! - [ ] Input (Windows) (WIP)
//! - [ ] Advanced Input (Kitty-like)
//! - [ ] Advanced Input (Windows)
//! - [ ] Events (Focus reporting, Bracketed-paste) (Unix)
//! - [ ] Events (Focus reporting, Bracketed-paste) (Windows)
//! - [ ] Mouse input (Unix)
//! - [ ] Mouse input (Windows)
//! - [ ] Feature completeness / API cleanup
//!
//! ## Support
//! This library generally attempts to have as much functionality as it can but sadly many terminal
//! emulators are heavily limited, there are a few protocols I have decided not to support but for
//! the most part this holds true
//!
//! ### Protocol Support
//! This is a list of terminal protocols and whether they will be supported, they still might not
//! work as this library is work in progress but eventually will be
//!
//! Just because a protocol is listed as not planned doesn't mean it definetly won't be added but
//! it is most likely not without good reason
//! - Standard Windows terminals (Full support planned)\*
//! - WinPTY (Windows psuedo-terminals) (Full support planned)
//! - Standard \*nix terminals (Full support planned)\*
//! - OSC 52 system clipboard (Full support planned)
//! - Kitty comprehensive keyboard handling (Full support planned)
//! - Kitty colored and styled underlines (Full support planned)
//! - Other Kitty protocols (there are a lot of them) (Not planned)
//!
//! \* Do not have full support for advanced input

use std::io;

#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

pub mod ansi;
pub mod control;
pub mod input;

pub mod prelude {
    //! Covenience re-export of common members
    pub use crate::ansi::*;
    pub use crate::control::*;
    pub use crate::input::*;
}

/// Struct that calls `func(true)` on construction
/// and `func(false)` on destruction
pub struct Handler {
    enabled: bool,
    func: &'static dyn Fn(bool) -> io::Result<()>,
}

impl Handler {
    /// Creates a new instance and turns it on
    ///
    /// # Errors
    ///
    /// If the function errors
    pub fn new(func: &'static dyn Fn(bool) -> io::Result<()>) -> io::Result<Self> {
        let mut handler = Self {
            enabled: true,
            func,
        };
        handler.set(true)?;
        return Ok(handler);
    }
    /// Calls `func(true)`
    ///
    /// # Errors
    ///
    /// If the function errors
    pub fn enable(&mut self) -> io::Result<()> {
        self.set(true)
    }
    /// Calls `func(false)`
    ///
    /// # Errors
    ///
    /// If the function errors
    pub fn disable(&mut self) -> io::Result<()> {
        self.set(false)
    }
    /// Calls `func(set)`
    ///
    /// # Errors
    ///
    /// If the function errors
    pub fn set(&mut self, set: bool) -> io::Result<()> {
        if self.enabled != set {
            (self.func)(set)?;
        }
        Ok(())
    }
    /// Gets if it is enabled
    #[must_use]
    pub fn get(&self) -> bool {
        self.enabled
    }
}

impl Drop for Handler {
    fn drop(&mut self) {
        self.disable().expect("Failed to disable terminal raw mode");
    }
}
