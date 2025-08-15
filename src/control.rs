//! Utilities that help control the terminal
//!
//! These are built to work on Windows, Linux, and MacOS

#[cfg(unix)]
pub use crate::unix::{
    disable_kitty_keyboard, disable_mouse_input, disable_raw_mode, enable_kitty_keyboard,
    enable_mouse_input, enable_raw_mode, get_terminal_size,
};

#[cfg(windows)]
pub use crate::windows::{
    disable_kitty_keyboard, disable_mouse_input, disable_raw_mode, enable_kitty_keyboard,
    enable_mouse_input, enable_raw_mode, get_terminal_size,
};
