#![allow(unused_imports)]

#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

#[cfg(unix)]
use unix as os;

#[cfg(windows)]
use windows as os;

/// Checks if stdout is a terminal
pub use os::istty;

/// Enables ANSI support on Windows terminals
///
/// ANSI is on by default on *nix machines but still exists for ease of use
pub use os::enable_ansi;

/// Gets the size of the terminal
pub use os::get_terminal_size;


/// Struct representing a raw terminal
///
/// This was done due to weirdness in the termios API (you have to store the original state of the
/// terminal to restore it)
pub use os::RawTerminal;

/// Enables raw mode
///
/// Disables input echoing, line feeding, etc.
pub use os::enable_raw_mode;

/// Disables raw mode
///
/// Enables input echoing, line feeding, etc.
pub use os::disable_raw_mode;
