#![warn(clippy::all, clippy::pedantic)]
// this lint has way too many false positives
#![allow(clippy::doc_markdown)]
#![doc = include_str!("../README.md")]
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
