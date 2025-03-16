use std::io;

unsafe extern "system" {
    fn GetStdHandle(std_handle: u32) -> usize;
    fn GetConsoleMode(console_handle: usize, mode: *mut u32) -> u32;
    fn SetConsoleMode(console_handle: usize, mode: *mut u32) -> u32;
    fn GetConsoleScreenBufferInfo(
        console_output: usize,
        console_screen_buffer_info: *mut ConsoleScreenBufferInfo,
    ) -> u32;
}

const STD_INPUT_HANDLE: u32 = 0xFFFF_FFF6;
const STD_OUTPUT_HANDLE: u32 = 0xFFFF_FFF5;
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
    get_console_mode(handle, &raw mut mode)?;
    mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
    set_console_mode(handle, &raw mut mode)?;
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

/// Enables raw mode
///
/// Disables input echoing, line feeding, etc.
///
/// # Errors
///
/// If there is no stdout,
/// if stdout isn't a TTY, or
/// if it fails to get or set terminal settings
pub fn enable_raw_mode() -> io::Result<()> {
    let handle = get_std_handle(STD_INPUT_HANDLE)?;
    let mut mode = 0;
    get_console_mode(handle, &raw mut mode)?;
    mode &= !(ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT);
    set_console_mode(handle, &raw mut mode)?;
    Ok(())
}

/// Disables raw mode
///
/// Enables input echoing, line feeding, etc.
///
/// # Errors
///
/// If there is no stdout,
/// if stdout isn't a TTY, or
/// if it fails to get or set terminal settings
pub fn disable_raw_mode() -> io::Result<()> {
    let handle = get_std_handle(STD_INPUT_HANDLE)?;
    let mut mode = 0;
    get_console_mode(handle, &raw mut mode)?;
    mode |= ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT;
    set_console_mode(handle, &raw mut mode)?;
    Ok(())
}

fn get_std_handle(handle: u32) -> io::Result<usize> {
    let handle = unsafe { GetStdHandle(handle) };
    if handle == INVALID_HANDLE_VALUE {
        Err(io::Error::last_os_error())
    } else {
        Ok(handle)
    }
}

fn set_console_mode(handle: usize, mode: *mut u32) -> io::Result<()> {
    if unsafe { SetConsoleMode(handle, mode) == 0 } {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn get_console_mode(handle: usize, mode: *mut u32) -> io::Result<()> {
    if unsafe { GetConsoleMode(handle, mode) == 0 } {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
