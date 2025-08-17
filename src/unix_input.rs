use std::ffi::{c_int, c_short, c_ulong, c_void};
use std::io;
use std::time::Duration;

use crate::input::{ButtonType, Event, Key, Modifiers, MouseButton, key_helper, simple_key};
use crate::unix::{POLLIN, STDIN_FILENO};
// Some of this input code has been modified from [termion](https://github.com/redox-os/termion)

/// Attempts to fetch input from stdin
///
/// # Errors
/// If the timeout has expired or
/// there was an error getting the data
pub fn poll_input(timeout: Duration) -> io::Result<Event> {
    let result = poll_timeout(timeout);
    let mut read_iter = ReadIterator::new();

    let timed_out: io::Error = io::ErrorKind::TimedOut.into();

    match result {
        1.. => {
            let item = read_iter.next().ok_or(timed_out)??;
            parse_event(item, &mut read_iter)
        }
        0 => Err(timed_out),
        _ => Err(io::Error::last_os_error()),
    }
}

fn poll_timeout(timeout: Duration) -> i32 {
    let mut fds = [PollFD {
        fd: STDIN_FILENO,
        events: POLLIN,
        revents: 0,
    }];
    unsafe {
        #[allow(clippy::cast_possible_truncation)]
        poll(
            fds.as_mut_ptr(),
            fds.len() as c_ulong,
            timeout.as_millis() as c_int,
        )
    }
}

unsafe extern "C" {
    fn poll(fds: *mut PollFD, nfds: c_ulong, timeout: c_int) -> c_int;
    fn read(fd: c_int, buf: *mut c_void, count: c_ulong) -> c_short;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct PollFD {
    fd: c_int,
    events: c_short,
    revents: c_short,
}

struct ReadIterator {
    buf: u8,
}

impl ReadIterator {
    fn new() -> Self {
        Self { buf: 0 }
    }
}

impl Iterator for ReadIterator {
    type Item = io::Result<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        let bytes_poll = poll_timeout(Duration::ZERO);
        let bytes_read = match bytes_poll {
            1.. => Some(Ok(unsafe {
                read(STDIN_FILENO, (&raw mut self.buf).cast::<c_void>(), 1)
            })),
            0 => None,
            _ => Some(Err(io::Error::last_os_error())),
        };
        match bytes_read? {
            Ok(1..) => Some(Ok(self.buf)),
            Ok(0) => None,
            _ => Some(Err(io::Error::last_os_error())),
        }
    }
}

pub(crate) fn parse_event<I>(item: u8, iter: &mut I) -> io::Result<Event>
where
    I: Iterator<Item = io::Result<u8>>,
{
    match item {
        b'\x1b' => parse_ansi_sequence(iter),
        b'\r' => Ok(key_helper("", Key::Char('\r'))),
        b'\n' => Ok(key_helper("C", Key::Char('j'))),
        b'\t' => Ok(key_helper("", Key::Char('\t'))),
        b'\x7f' => Ok(key_helper("", Key::Backspace)),
        b'\0' => Ok(key_helper("C", Key::Char(' '))),
        c @ b'\x01'..=b'\x1a' => Ok(key_helper("C", Key::Char((c + 96) as char))),
        c @ b'\x1c'..=b'\x1f' => Ok(key_helper("C", Key::Char((c + 24) as char))),
        c => {
            let character = parse_utf8_char(c, iter)?;
            Ok(Event::Key(
                Key::Char(character),
                ButtonType::Press,
                Modifiers::NONE.shift(character.is_uppercase()),
            ))
        }
    }
}

fn parse_utf8_char<I>(c: u8, iter: &mut I) -> io::Result<char>
where
    I: Iterator<Item = io::Result<u8>>,
{
    let error = || io::Error::new(io::ErrorKind::InvalidData, "Input char is not valid UTF-8");
    let mut bytes = vec![c];

    for _ in 1..=4 {
        if let Ok(string) = std::str::from_utf8(&bytes) {
            return Ok(string.chars().next().unwrap());
        }
        bytes.push(iter.next().ok_or_else(error)??);
    }
    Err(error())
}

fn parse_ansi_sequence<I>(iter: &mut I) -> io::Result<Event>
where
    I: Iterator<Item = io::Result<u8>>,
{
    let error = io::Error::other("Could not parse event");
    match iter.next() {
        None => Ok(key_helper("", Key::Escape)),
        Some(Ok(b'O')) => match iter.next() {
            Some(Ok(val @ b'P'..=b's')) => Ok(key_helper("", Key::F(1 + val - b'P'))),
            _ => Err(error),
        },
        Some(Ok(b'[')) => parse_csi_sequence(iter).ok_or(error),
        Some(Ok(c)) => match c {
            b'\r' => Ok(key_helper("A", Key::Char('\r'))),
            b'\n' => Ok(key_helper("CA", Key::Char('j'))),
            b'\t' => Ok(key_helper("A", Key::Char('\t'))),
            b'\x7f' => Ok(key_helper("A", Key::Backspace)),
            b'\0' => Ok(key_helper("CA", Key::Char(' '))),
            c @ b'\x01'..=b'\x1a' => Ok(key_helper("CA", Key::Char((c + 96) as char))),
            c @ b'\x1c'..=b'\x1f' => Ok(key_helper("CA", Key::Char((c + 24) as char))),
            c => {
                let character = parse_utf8_char(c, iter)?;
                Ok(Event::Key(
                    Key::Char(character),
                    ButtonType::Press,
                    Modifiers::NONE.shift(character.is_uppercase()).alt(true),
                ))
            }
        },
        _ => Err(error),
    }
}

fn parse_csi_sequence<I>(iter: &mut I) -> Option<Event>
where
    I: Iterator<Item = io::Result<u8>>,
{
    match iter.next() {
        Some(Ok(b'[')) => match iter.next() {
            Some(Ok(val @ b'A'..=b'E')) => Some(key_helper("", Key::F(1 + val - b'A'))),
            _ => None,
        },
        Some(Ok(b'D')) => Some(key_helper("", Key::Left)),
        Some(Ok(b'C')) => Some(key_helper("", Key::Right)),
        Some(Ok(b'A')) => Some(key_helper("", Key::Up)),
        Some(Ok(b'B')) => Some(key_helper("", Key::Down)),
        Some(Ok(b'H')) => Some(key_helper("", Key::Home)),
        Some(Ok(b'F')) => Some(key_helper("", Key::End)),
        Some(Ok(b'Z')) => Some(key_helper("", Key::Tab)),
        Some(Ok(b'<')) => parse_xterm_mouse(iter),
        Some(Ok(b'M')) => Some(parse_x10_mouse(iter)),
        Some(Ok(c @ b'0'..=b'9')) => parse_numbered_escape(iter, c),
        None => Some(key_helper("A", Key::Char('['))),
        _ => None,
    }
}

fn parse_numbered_escape<I>(iter: &mut I, c: u8) -> Option<Event>
where
    I: Iterator<Item = io::Result<u8>>,
{
    let mut buf = Vec::new();
    buf.push(c);
    let mut c = iter.next().unwrap().unwrap();
    // The final byte of a CSI sequence can be in the range 64-126, so let's keep reading
    // anything else.
    while !(64..=126).contains(&c) {
        buf.push(c);
        c = iter.next().unwrap().unwrap();
    }
    match c {
        // rxvt mouse encoding:
        // ESC [ Cb ; Cx ; Cy ; M
        b'M' => {
            let str_buf = String::from_utf8(buf).unwrap();

            let nums: Vec<u16> = str_buf.split(';').map(|n| n.parse().unwrap()).collect();

            let cb = nums[0];
            let cx = nums[1];
            let cy = nums[2];

            let mods = Modifiers::NONE;

            let event = match cb {
                32 => Event::Mouse(mods, MouseButton::Left, ButtonType::Press, cx, cy),
                33 => Event::Mouse(mods, MouseButton::Middle, ButtonType::Press, cx, cy),
                34 => Event::Mouse(mods, MouseButton::Right, ButtonType::Press, cx, cy),
                35 => Event::Mouse(mods, MouseButton::Unknown, ButtonType::Release, cx, cy),
                64 => Event::Mouse(mods, MouseButton::Unknown, ButtonType::Held, cx, cy),
                96 | 97 => Event::Mouse(mods, MouseButton::WheelUp, ButtonType::Press, cx, cy),
                _ => return None,
            };

            Some(event)
        }
        // Special key code.
        b'~' => {
            let str_buf = String::from_utf8(buf).unwrap();

            // This CSI sequence can be a list of semicolon-separated
            // numbers.
            let nums: Vec<u8> = str_buf.split(';').map(|n| n.parse().unwrap()).collect();

            if nums.is_empty() {
                return None;
            }

            // TODO: handle multiple values for key modififiers (ex: values
            // [3, 2] means Shift+Delete)
            if nums.len() > 1 {
                return None;
            }

            match nums[0] {
                1 | 7 => Some(key_helper("", Key::Home)),
                2 => Some(key_helper("", Key::Insert)),
                3 => Some(key_helper("", Key::Delete)),
                4 | 8 => Some(key_helper("", Key::End)),
                5 => Some(key_helper("", Key::PageUp)),
                6 => Some(key_helper("", Key::PageDown)),
                v @ 11..=15 => Some(key_helper("", Key::F(v - 10))),
                v @ 17..=21 => Some(key_helper("", Key::F(v - 11))),
                v @ 23..=24 => Some(key_helper("", Key::F(v - 12))),
                _ => None,
            }
        }
        b'u' => {
            let str_buf = String::from_utf8(buf).unwrap();
            let mut iter = str_buf.split(';');
            let key_code: u32 = iter.next()?.parse().ok()?;
            let mut iter = iter.next().unwrap_or("0:1").split(':');
            let modifier: u32 = iter.next()?.parse().ok()?;
            let key_type: u32 = iter.next().unwrap_or("1").parse().ok()?;
            println!("{str_buf}\r");
            println!("{modifier}\r");

            let char = char::from_u32(key_code);
            // let shift = modifier & 1 == 1;
            // let alt = modifier & 2 == 2;
            // let ctrl = modifier & 4 == 4;
            let button_type = match key_type {
                1 => ButtonType::Press,
                2 => ButtonType::Held,
                3 => ButtonType::Release,
                _ => return None,
            };

            Some(Event::Key(Key::Char(char?), button_type, Modifiers::NONE))
        }
        b'A' | b'B' | b'C' | b'D' | b'F' | b'H' => {
            let str_buf = String::from_utf8(buf).unwrap();

            // This CSI sequence can be a list of semicolon-separated
            // numbers.
            let nums: Vec<u8> = str_buf.split(';').map(|n| n.parse().unwrap()).collect();

            if !(nums.len() == 2 && nums[0] == 1) {
                return None;
            }
            let mods = nums[1] - 1;
            let shift = mods & 1 == 1;
            let alt = mods & 2 == 2;
            let ctrl = mods & 4 == 4;
            match c {
                b'D' => Some(simple_key(Key::Left, shift, alt, ctrl)),
                b'C' => Some(simple_key(Key::Right, shift, alt, ctrl)),
                b'A' => Some(simple_key(Key::Up, shift, alt, ctrl)),
                b'B' => Some(simple_key(Key::Down, shift, alt, ctrl)),
                b'H' => Some(simple_key(Key::Home, shift, alt, ctrl)),
                b'F' => Some(simple_key(Key::End, shift, alt, ctrl)),
                _ => None,
            }
        }

        _ => None,
    }
}

fn parse_x10_mouse<I>(iter: &mut I) -> Event
where
    I: Iterator<Item = io::Result<u8>>,
{
    // X10 emulation mouse encoding: ESC [ CB Cx Cy (6 characters only).
    let mut next = || iter.next().unwrap().unwrap();

    let cb = next().wrapping_sub(32);
    // (0, 0) are the coords for upper left.
    let cx = u16::from(next().saturating_sub(33));
    let cy = u16::from(next().saturating_sub(33));

    let mods = Modifiers::NONE;
    match cb & 0b11 {
        0 => {
            if cb & 0x40 != 0 {
                Event::Mouse(mods, MouseButton::WheelUp, ButtonType::Press, cx, cy)
            } else {
                Event::Mouse(mods, MouseButton::Left, ButtonType::Press, cx, cy)
            }
        }
        1 => {
            if cb & 0x40 != 0 {
                Event::Mouse(mods, MouseButton::WheelDown, ButtonType::Press, cx, cy)
            } else {
                Event::Mouse(mods, MouseButton::Middle, ButtonType::Press, cx, cy)
            }
        }
        2 => {
            if cb & 0x40 != 0 {
                Event::Mouse(mods, MouseButton::WheelLeft, ButtonType::Press, cx, cy)
            } else {
                Event::Mouse(mods, MouseButton::Right, ButtonType::Press, cx, cy)
            }
        }
        3 => {
            if cb & 0x40 != 0 {
                Event::Mouse(mods, MouseButton::WheelRight, ButtonType::Press, cx, cy)
            } else {
                Event::Mouse(mods, MouseButton::Unknown, ButtonType::Release, cx, cy)
            }
        }
        _ => unreachable!(),
    }
}

fn parse_xterm_mouse<I>(iter: &mut I) -> Option<Event>
where
    I: Iterator<Item = io::Result<u8>>,
{
    // xterm/SGR mouse encoding:
    let mut buf = Vec::new();
    let mut c = iter.next().unwrap().unwrap();
    while !matches!(c, b'm' | b'M') {
        buf.push(c);
        c = iter.next().unwrap().unwrap();
    }
    let str_buf = String::from_utf8(buf).unwrap();
    let nums = &mut str_buf.split(';');

    let cb = nums.next()?.parse::<u16>().unwrap();
    let cx = nums.next()?.parse::<u16>().unwrap().saturating_sub(1);
    let cy = nums.next()?.parse::<u16>().unwrap().saturating_sub(1);

    let shift = cb & 4 == 4;
    let alt = cb & 8 == 8;
    let ctrl = cb & 16 == 16;
    let mods = Modifiers::new(shift, alt, ctrl);
    let trimmed_cb = cb ^ (cb & 0b0001_1100);

    let event = match trimmed_cb {
        0..=2 | 64..=67 => {
            let button = match trimmed_cb {
                0 => MouseButton::Left,
                1 => MouseButton::Middle,
                2 => MouseButton::Right,
                64 => MouseButton::WheelUp,
                65 => MouseButton::WheelDown,
                66 => MouseButton::WheelLeft,
                67 => MouseButton::WheelRight,
                _ => unreachable!(),
            };
            match c {
                b'M' => Event::Mouse(mods, button, ButtonType::Press, cx, cy),
                b'm' => Event::Mouse(mods, button, ButtonType::Release, cx, cy),
                _ => return None,
            }
        }
        32 => Event::Mouse(mods, MouseButton::Left, ButtonType::Held, cx, cy),
        33 => Event::Mouse(mods, MouseButton::Middle, ButtonType::Held, cx, cy),
        34 => Event::Mouse(mods, MouseButton::Right, ButtonType::Held, cx, cy),
        35 => Event::Mouse(mods, MouseButton::None, ButtonType::Held, cx, cy),
        3 => Event::Mouse(mods, MouseButton::Unknown, ButtonType::Release, cx, cy),
        _ => return None,
    };

    Some(event)
}

#[test]
fn test_parse_utf8() {
    let string = "abcéŷ¤£€ù%323";
    let ref mut bytes = string.bytes().map(|x| Ok(x));
    let chars = string.chars();
    for c in chars {
        let b = bytes.next().unwrap().unwrap();
        let character = parse_utf8_char(b, bytes).unwrap();
        assert!(c == character);
    }
}
