use crate::input::{Event, key_helper, Key};
use crate::windows::get_stdin_handle;

use std::io;
use std::mem;
use std::os::windows::raw::HANDLE;
use std::time::Duration;

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

    let result = unsafe { ReadConsoleInputW(handle, &raw mut record, 1, &raw mut read) };

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
        0x08 => key_helper("", Key::Backspace),
        0x09 => {
            if shift {
                key_helper("S", Key::Tab)
            } else {
                key_helper("", Key::Tab)
            }
        }
        0x0D => key_helper("", Key::Char('\n')),
        0x1B => key_helper("", Key::Escape),
        0x21 => key_helper("", Key::PageUp),
        0x22 => key_helper("", Key::PageDown),
        0x23 => key_helper("", Key::End),
        0x24 => key_helper("", Key::Home),
        0x25 => key_helper("", Key::Left),
        0x26 => key_helper("", Key::Up),
        0x27 => key_helper("", Key::Right),
        0x28 => key_helper("", Key::Down),
        0x2D => key_helper("", Key::Insert),
        0x2E => key_helper("", Key::Delete),
        // I don't think anybody is going to try to press F256 clippy
        #[allow(clippy::cast_possible_truncation)]
        0x70..=0x87 => key_helper("", Key::F((event.virtual_key_code - 0x6F) as u8)),
        _ => {
            let num = u32::from(unsafe { event.u_char.unicode_char });
            let c = char::from_u32(num).unwrap_or(' ');
            if ctrl && c.is_ascii_alphabetic() {
                key_helper("C", Key::Char(c))
            } else {
                key_helper("", Key::Char(c))
            }
        }
    }
}
