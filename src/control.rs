//! Utilities that help control the terminal
//!
//! These are built to work on Windows, Linux, and MacOS

use std::io;

#[cfg(unix)]
pub use crate::unix::{
    disable_ansi, disable_mouse_input, disable_raw_mode, enable_mouse_input, enable_raw_mode,
    get_terminal_size,
};

#[cfg(windows)]
pub use crate::windows::{
    disable_ansi, disable_mouse_input, disable_raw_mode, enable_mouse_input, enable_raw_mode,
    get_terminal_size,
};

const ENABLE_KITTY_KEYBOARD: &str = "\x1b[>31u";
const DISABLE_KITTY_KEYBOARD: &str = "\x1b[<31u";

/// Enable kitty comprehensive keyboard handling protocol
pub fn enable_kitty_keyboard() {
    print!("{ENABLE_KITTY_KEYBOARD}");
}

/// Disable kitty comprehensive keyboard handling protocol
pub fn disable_kitty_keyboard() {
    print!("{DISABLE_KITTY_KEYBOARD}");
}

use crate::prelude::{ALT_SCREEN_ENTER, ALT_SCREEN_EXIT, enable_ansi};

pub fn tui_init() -> io::Result<()> {
    enable_ansi()?;
    enable_raw_mode()?;
    enable_mouse_input()?;
    print!("{ALT_SCREEN_ENTER}");
    enable_kitty_keyboard();
    Ok(())
}

pub fn tui_deinit() -> io::Result<()> {
    disable_kitty_keyboard();
    print!("{ALT_SCREEN_EXIT}");
    disable_mouse_input()?;
    disable_raw_mode()?;
    disable_ansi()?;
    Ok(())
}

pub fn cli_init() -> io::Result<()> {
    enable_ansi()?;
    enable_raw_mode()?;
    enable_mouse_input()?;
    enable_kitty_keyboard();
    Ok(())
}

pub fn cli_deinit() -> io::Result<()> {
    disable_ansi()?;
    disable_raw_mode()?;
    disable_mouse_input()?;
    disable_kitty_keyboard();
    Ok(())
}
