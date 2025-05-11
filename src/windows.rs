use std::io::{self, Stdin, Stdout};
use std::os::windows::io::RawHandle;

unsafe extern "system" {
    fn GetStdHandle(std_handle: u32) -> usize;
    fn GetConsoleMode(console_handle: usize, mode: *mut u32) -> u32;
    fn SetConsoleMode(console_handle: usize, mode: *mut u32) -> u32;
    fn GetConsoleScreenBufferInfo(
        console_output: usize,
        console_screen_buffer_info: *mut ConsoleScreenBufferInfo,
    ) -> u32;
}

pub(crate) const STD_INPUT_HANDLE: u32 = 0xFFFF_FFF6;
pub(crate) const STD_OUTPUT_HANDLE: u32 = 0xFFFF_FFF5;
const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 4;
const ENABLE_ECHO_INPUT: u32 = 4;
const ENABLE_LINE_INPUT: u32 = 2;
const ENABLE_PROCESSED_INPUT: u32 = 1;
const INVALID_HANDLE_VALUE: usize = usize::MAX - 1;

#[repr(C)]
#[derive(Default)]
struct ConsoleScreenBufferInfo {
    x: u16,
    y: u16,
    _unused: [u16; 9],
}

/// This struct represents a raw terminal
///
/// This struct will automatically enable raw mode when it is created
/// and disable raw mode when it is destructed
///
/// This insures that you never exit with a terminal still in raw mode which is problematic for
/// users
pub struct RawTerminal;

impl RawTerminal {
    /// This constructs a terminal, automatically making it raw
    ///
    /// # Errors
    ///
    /// If there is no stdin,
    /// stdin is not a tty,
    /// if it fails to change terminal settings
    pub fn new() -> io::Result<Self> {
        let handle = get_std_handle(STD_INPUT_HANDLE)?;
        let mut mode = 0;
        get_console_mode(handle, &mut mode)?;
        mode &= !(ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT);
        set_console_mode(handle, &mut mode)?;
        Ok(Self)
    }
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        let handle = get_std_handle(STD_INPUT_HANDLE).expect("Failed to disable terminal raw mode");
        let mut mode = 0;
        get_console_mode(handle, &mut mode).expect("Failed to disable terminal raw mode");
        mode |= ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT;
        set_console_mode(handle, &mut mode).expect("Failed to disable terminal raw mode");
    }
}

/// Enables ANSI support on Windows terminals
///
/// ANSI is on by default on *nix machines but still exists on them for simpler usage
///
/// # Errors
///
/// Never on *nix
///
/// On Windows, if There is no stdout,
/// if stdout isn't a TTY, or
/// if it cannot change terminal properties
pub fn enable_ansi() -> io::Result<()> {
    let handle = get_std_handle(STD_OUTPUT_HANDLE)?;
    let mut mode = 0;
    get_console_mode(handle, &mut mode)?;
    mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
    set_console_mode(handle, &mut mode)?;
    Ok(())
}

/// Gets the size of the terminal
///
/// Returns in (width, height) format
///
/// # Errors
///
/// If there is no stdout,
/// if stdout isn't a TTY, or
/// if it fails to retrieve the terminal size
pub fn get_terminal_size() -> io::Result<(u16, u16)> {
    let handle = get_std_handle(STD_OUTPUT_HANDLE)?;
    let mut csbi = ConsoleScreenBufferInfo::default();
    if unsafe { GetConsoleScreenBufferInfo(handle, &mut csbi) != 0 } {
        let width = csbi.x;
        let height = csbi.y;
        return Ok((width, height));
    }
    Err(io::Error::last_os_error())
}

pub(crate) fn get_std_handle(handle: u32) -> io::Result<usize> {
    let handle = unsafe { GetStdHandle(handle) };
    if handle == INVALID_HANDLE_VALUE {
        Err(io::Error::last_os_error())
    } else {
        Ok(handle)
    }
}

pub(crate) fn set_console_mode(handle: usize, mode: &mut u32) -> io::Result<()> {
    if unsafe { SetConsoleMode(handle, mode) == 0 } {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub(crate) fn get_console_mode(handle: usize, mode: &mut u32) -> io::Result<()> {
    if unsafe { GetConsoleMode(handle, mode) == 0 } {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
