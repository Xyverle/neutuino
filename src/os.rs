//! Collection of functions that help control the terminal
//!
//! These are built to work at least on these platforms:
//! Windows, Linux, and Mac, but are likely to work on more

#[cfg(unix)]
#[path = "unix.rs"]
mod unix;

#[cfg(unix)]
pub use unix::*;

#[cfg(windows)]
#[path = "windows.rs"]
mod windows;

#[cfg(windows)]
pub use windows::*;
