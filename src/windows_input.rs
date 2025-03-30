use crate::input::{Event, KeyEvent};
use crate::os::{STD_INPUT_HANDLE, get_std_handle};

use std::{io, mem, time::Duration};

#[repr(C)]
#[derive(Copy, Clone)]
struct InputRecord {
    event_type: u16,
    event: EventRecord,
}

#[repr(C)]
#[derive(Copy, Clone)]
union EventRecord {
    key_event: KeyEventRecord,
    mouse_event: MouseEventRecord,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct KeyEventRecord {
    key_down: i32,
    virtual_key_code: u16,
    u_char: CharUnion,
    control_key_state: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
union CharUnion {
    unicode_char: u16,
    ascii_char: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct MouseEventRecord {
    mouse_position: Coord,
    button_state: u32,
    control_key_state: u32,
    event_flags: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Coord {
    x: i16,
    y: i16,
}

unsafe extern "system" {
    fn ReadConsoleInputW(
        h_console_input: usize,
        lp_buffer: *mut InputRecord,
        n_length: u32,
        lp_number_of_events_read: *mut u32,
    ) -> i32;
    fn WaitForMultipleObjects(
        n_count: u32,
        lp_handles: *mut usize,
        b_wait_all: i32,
        dw_wait_time: u32,
    ) -> u32;
}

pub fn poll_input(timeout: Duration) -> Option<io::Result<Event>> {
    let handle = get_std_handle(STD_INPUT_HANDLE).ok()?;
    let mut record: InputRecord = unsafe { mem::zeroed() };
    let mut read = 0;

    let wait_time_millis = timeout.as_millis() as u32;
    let mut handles = [handle];
    let result = unsafe { WaitForMultipleObjects(1, handles.as_mut_ptr(), 0, wait_time_millis) };

    if result != 0 {
        return None;
    }

    let result = unsafe { ReadConsoleInputW(handle, &mut record, 1, &mut read) };

    if result == 0 {
        return Some(Err(io::Error::last_os_error()));
    }
    if record.event_type == 1 {
        let key_event = unsafe { record.event.key_event };
        if key_event.key_down == 0 {
            return Some(Ok(Event::Key(KeyEvent::Null)));
        }
        return Some(Ok(Event::Key(parse_key_event(&key_event))));
    }
    None
}

fn parse_key_event(event: &KeyEventRecord) -> KeyEvent {
    let ctrl = event.control_key_state & (0x0008 | 0x0004) != 0; // LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED
    let shift = event.control_key_state & 0x0010 != 0; // SHIFT_PRESSED

    match event.virtual_key_code {
        0x08 => KeyEvent::Backspace, // VK_BACK
        0x09 => {
            if shift {
                KeyEvent::BackTab
            } else {
                KeyEvent::Tab
            }
        }
        0x0D => KeyEvent::Char('\n'),
        0x1B => KeyEvent::Escape,
        0x21 => KeyEvent::PageUp,
        0x22 => KeyEvent::PageDown,
        0x23 => KeyEvent::End,
        0x24 => KeyEvent::Home,
        0x25 => KeyEvent::Left,
        0x26 => KeyEvent::Up,
        0x27 => KeyEvent::Right,
        0x28 => KeyEvent::Down,
        0x2D => KeyEvent::Insert,
        0x2E => KeyEvent::Delete,
        0x70..=0x87 => KeyEvent::F((event.virtual_key_code - 0x6F) as u8), // F1-F24
        _ => {
            let c = unsafe { event.u_char.unicode_char } as u8 as char;
            if ctrl && c.is_ascii_alphabetic() {
                KeyEvent::Ctrl(c)
            } else {
                KeyEvent::Char(c)
            }
        }
    }
}
