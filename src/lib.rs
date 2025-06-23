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
//! it is most likely a no without good reason
//! - Standard Windows terminals (Full support planned)*
//! - WinPTY (Windows psuedo-terminals) (Full support planned)
//! - Standard *nix terminals (Full support planned)**
//! - OSC 52 system clipboard (Full support planned)
//! - Kitty comprehensive keyboard handling (Full support planned)
//! - Kitty colored and styled underlines (Full support planned)
//! - Other Kitty protocols (there are a lot of them) (Not planned)
//!
//! \* Standard Windows termian
//!
//! \** Standard *nix terminals do not have support for some advanced input

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
