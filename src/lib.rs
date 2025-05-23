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
//! - [ ] Events (Focus reporting, Bracketed-paste) (Unix)
//! - [ ] Events (Focus reporting, Bracketed-paste) (Windows)
//! - [ ] Mouse input (Unix)
//! - [ ] Mouse input (Windows)
//! - [ ] Feature completeness / API cleanup

#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

pub mod ansi;
pub mod input;
pub mod os;

pub mod prelude {
    //! Covenience re-export of common members
    pub use crate::ansi::*;
    pub use crate::input::*;
    pub use crate::os::*;
}
