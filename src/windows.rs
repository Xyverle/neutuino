use crate::input::{Event, Key, KeyMods, KeyType, press_key};
use std::os::windows::raw::HANDLE;
use std::{io, mem, time::Duration};

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetStdHandle(std_handle: i32) -> HANDLE;
    fn GetConsoleMode(console_handle: HANDLE, mode: *mut u32) -> u32;
    fn SetConsoleMode(console_handle: HANDLE, mode: u32) -> u32;
    fn GetConsoleScreenBufferInfo(
        console_output: HANDLE,
        console_screen_buffer_info: *mut ConsoleScreenBufferInfo,
    ) -> u32;
}

const STD_INPUT_HANDLE: i32 = -10;
const STD_OUTPUT_HANDLE: i32 = -11;
const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 4;
const ENABLE_VIRTUAL_TERMINAL_INPUT: u32 = 0x200;
const ENABLE_ECHO_INPUT: u32 = 4;
const ENABLE_LINE_INPUT: u32 = 2;
const ENABLE_PROCESSED_INPUT: u32 = 1;
const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;

#[repr(C)]
#[derive(Default)]
struct ConsoleScreenBufferInfo {
    x: u16,
    y: u16,
    _unused: [u16; 9],
}

fn get_stdin_handle() -> io::Result<HANDLE> {
    let handle = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
    if handle == INVALID_HANDLE_VALUE {
        Err(io::Error::last_os_error())
    } else {
        Ok(handle)
    }
}

fn get_stdout_handle() -> io::Result<HANDLE> {
    let handle = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
    if handle == INVALID_HANDLE_VALUE {
        Err(io::Error::last_os_error())
    } else {
        Ok(handle)
    }
}

fn set_console_mode(handle: HANDLE, mode: u32) -> io::Result<()> {
    if unsafe { SetConsoleMode(handle, mode) == 0 } {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn get_console_mode(handle: HANDLE, mode: &mut u32) -> io::Result<()> {
    if unsafe { GetConsoleMode(handle, mode) == 0 } {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

/// Enables raw mode, which disables line buffering, input echoing, and output canonicalization
///
/// # Errors
///
/// If there is no stdin,
/// stdin is not a tty,
/// or it fails to change terminal settings
pub fn enable_raw_mode() -> io::Result<()> {
    let handle = get_stdin_handle()?;
    let mut mode = 0;
    get_console_mode(handle, &mut mode)?;
    mode &= !(ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT);
    set_console_mode(handle, mode)?;
    Ok(())
}

/// Disables raw mode, which enables line buffering, input echoing, and output canonicalization
///
/// # Errors
///
/// If there is no stdin,
/// stdin is not a tty,
/// or it fails to change terminal settings
pub fn disable_raw_mode() -> io::Result<()> {
    let handle = get_stdin_handle()?;
    let mut mode = 0;
    get_console_mode(handle, &mut mode)?;
    mode |= ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT;
    set_console_mode(handle, mode)?;
    Ok(())
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
    let handle = get_stdout_handle()?;
    let mut mode = 0;
    get_console_mode(handle, &mut mode)?;
    mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
    set_console_mode(handle, mode)?;

    let handle = get_stdin_handle()?;
    let mut mode = 0;
    get_console_mode(handle, &mut mode)?;
    mode &= !ENABLE_VIRTUAL_TERMINAL_INPUT;
    set_console_mode(handle, mode)?;
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
    let handle = get_stdout_handle()?;
    let mut csbi = ConsoleScreenBufferInfo::default();
    if unsafe { GetConsoleScreenBufferInfo(handle, &mut csbi) != 0 } {
        let width = csbi.x;
        let height = csbi.y;
        return Ok((width, height));
    }
    Err(io::Error::last_os_error())
}

#[repr(C)]
#[derive(Copy, Clone)]
struct InputRecord {
    event_type: u16,
    event: EventRecord,
}

#[repr(C)]
#[derive(Copy, Clone)]
union EventRecord {
    key: KeyEventRecord,
    focus: FocusEventRecord,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct KeyEventRecord {
    key_down: i32,
    repeat_count: u16,
    virtual_key_code: u16,
    virtual_scan_code: u16,
    u_char: CharUnion,
    control_key_state: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct FocusEventRecord {
    set_focus: i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
union CharUnion {
    unicode_char: u16,
    ascii_char: u8,
}

unsafe extern "system" {
    fn ReadConsoleInputW(
        console_input: HANDLE,
        buffer: *mut InputRecord,
        length: u32,
        number_of_events_read: *mut u32,
    ) -> i32;
    fn WaitForSingleObject(handle: HANDLE, wait_time_ms: u32) -> u32;
}

/// Attempts to fetch input from stdin
///
/// # Errors
/// If the timeout has expired or
/// there was an error getting the data
pub fn poll_input(timeout: Duration) -> io::Result<Event> {
    let handle = get_stdin_handle()?;
    let mut record: InputRecord = unsafe { mem::zeroed() };
    let mut read = 0;

    // shut up clippy no reasonable person would expect to be able to have a poll longer than a
    // month
    #[allow(clippy::cast_possible_truncation)]
    let wait_time_millis = timeout.as_millis() as u32;
    let result = unsafe { WaitForSingleObject(handle, wait_time_millis) };

    // The function timed out
    if result != 0 {
        return Err(io::ErrorKind::TimedOut.into());
    }

    let result = unsafe { ReadConsoleInputW(handle, &mut record, 1, &mut read) };

    if result == 0 {
        Err(io::Error::last_os_error())?;
    }
    match record.event_type {
        0x10 => {
            // Focus Event
            Err(io::ErrorKind::InvalidData.into())
        }
        0x1 => {
            // Key Event
            let key_event: KeyEventRecord = unsafe { record.event.key };
            if key_event.key_down == 0 {
                // return Ok(Event::Key(KeyEvent::Null));
                // I don't quite know why but this seems to happen a lot, until I investigate
                // more this will have to do
                return Err(io::ErrorKind::Other.into());
            }
            Ok(parse_key_event(&key_event))
        }
        _ => {
            //TODO Make this better
            Err(io::ErrorKind::InvalidData.into())
        }
    }
}

fn parse_key_event(event: &KeyEventRecord) -> Event {
    let ctrl = event.control_key_state & (0x0008 | 0x0004) != 0; // LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED
    let shift = event.control_key_state & 0x0010 != 0; // SHIFT_PRESSED

    match event.virtual_key_code {
        0x08 => press_key(Key::Backspace, KeyMods::NONE),
        0x09 => {
            if shift {
                press_key(Key::Tab, KeyMods::SHIFT)
            } else {
                press_key(Key::Tab, KeyMods::NONE)
            }
        }
        0x0D => press_key(Key::Char('\n'), KeyMods::NONE),
        0x1B => press_key(Key::Escape, KeyMods::NONE),
        0x21 => press_key(Key::PageUp, KeyMods::NONE),
        0x22 => press_key(Key::PageDown, KeyMods::NONE),
        0x23 => press_key(Key::End, KeyMods::NONE),
        0x24 => press_key(Key::Home, KeyMods::NONE),
        0x25 => press_key(Key::Left, KeyMods::NONE),
        0x26 => press_key(Key::Up, KeyMods::NONE),
        0x27 => press_key(Key::Right, KeyMods::NONE),
        0x28 => press_key(Key::Down, KeyMods::NONE),
        0x2D => press_key(Key::Insert, KeyMods::NONE),
        0x2E => press_key(Key::Delete, KeyMods::NONE),
        // I don't think anybody is going to try to press F256 clippy
        #[allow(clippy::cast_possible_truncation)]
        0x70..=0x87 => press_key(Key::F(event.virtual_key_code - 0x6F) as u8), KeyMods::NONE),
        _ => {
            let num = u32::from(unsafe { event.u_char.unicode_char });
            let c = char::from_u32(num).unwrap_or(' ');
            if ctrl && c.is_ascii_alphabetic() {
                press_key(Key::Char(c), KeyMods::CTRL)
            } else {
                press_key(Key::Char(c), KeyMods::NONE)
            }
        }
    }
}
