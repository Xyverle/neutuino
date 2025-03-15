#[cfg(unix)]
#[path = "unix.rs"] mod unix;

#[cfg(unix)]
use unix as os;

#[cfg(windows)]
#[path = "windows.rs"] mod windows;

#[cfg(windows)]
use windows as os;

/// Checks if stdout is a terminal
pub use os::is_terminal;

/// Enables ANSI support on Windows terminals
///
/// ANSI is on by default on *nix machines but still exists for ease of use
pub use os::enable_ansi;

/// Gets the size of the terminal
pub use os::get_terminal_size;

/// Enables raw mode
///
/// Disables input echoing, line feeding, etc.
pub use os::enable_raw_mode;

/// Disables raw mode
///
/// Enables input echoing, line feeding, etc.
pub use os::disable_raw_mode;
