use std::ffi::{c_int, c_uint, c_ulong, c_ushort};
use std::io;
use std::sync::LazyLock;

use crate::input::{ButtonType, Event, Key, Modifiers, MouseButton, key_helper, simple_key};
use std::ffi::{c_short, c_void};
use std::time::Duration;

const ENABLE_MOUSE: &str = "\x1b[?1000h\x1b[?1002h\x1b[?1015h\x1b[?1006h\x1b[?1003h";
const DISABLE_MOUSE: &str = "\x1b[?1006l\x1b[?1015l\x1b[?1002l\x1b[?1000l\x1b[?1003l";

const ENABLE_KITTY_KEYBOARD: &str = "\x1b[>31u";
const DISABLE_KITTY_KEYBOARD: &str = "\x1b[<31u";

unsafe extern "C" {
    fn ioctl(fd: c_int, request: c_ulong, argp: *mut u8) -> c_int;
    fn cfmakeraw(termios: *mut Termios);
    fn tcgetattr(fd: c_int, termios: *mut Termios) -> c_int;
    fn tcsetattr(fd: c_int, optional_actions: c_int, termios: *const Termios) -> c_int;
}

const STDIN_FILENO: c_int = 0;
const STDOUT_FILENO: c_int = 1;
const POLLIN: c_short = 1;

#[cfg(not(target_os = "macos"))]
const TIOCGWINSZ: c_ulong = 0x5413;
#[cfg(not(target_os = "macos"))]
const NCCS: usize = 0x20;

#[cfg(target_os = "macos")]
const TIOCGWINSZ: c_ulong = 0x4008_7468;
#[cfg(target_os = "macos")]
const NCCS: usize = 0x14;

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
struct Winsize {
    row: c_ushort,
    col: c_ushort,
    xpixel: c_ushort,
    ypixel: c_ushort,
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
struct Termios {
    iflag: c_uint,
    oflag: c_uint,
    cflag: c_uint,
    lflag: c_uint,
    cc: [u8; NCCS],
    #[cfg(target_os = "linux")]
    ispeed: c_ulong,
    #[cfg(target_os = "linux")]
    ospeed: c_ulong,
}

fn get_attributes(fd: c_int, termios: &mut Termios) -> io::Result<()> {
    if unsafe { tcgetattr(fd, &raw mut *termios) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn set_attributes(fd: c_int, termios: &mut Termios) -> io::Result<()> {
    if unsafe { tcsetattr(fd, 0, std::ptr::from_mut(termios)) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

static TERMIOS: LazyLock<Result<Termios, i32>> = LazyLock::new(|| {
    let mut orig_termios = unsafe { std::mem::zeroed() };
    let attributes = get_attributes(STDIN_FILENO, &mut orig_termios);
    match attributes {
        Ok(()) => Ok(orig_termios),
        Err(e) => Err(e.raw_os_error().unwrap()),
    }
});

/// CSI>31u
/// Enable kitty comprehensive keyboard handling protocol
pub fn enable_kitty_keyboard() {
    print!("{ENABLE_KITTY_KEYBOARD}");
}

/// Disable kitty comprehensive keyboard handling protocol
pub fn disable_kitty_keyboard() {
    print!("{DISABLE_KITTY_KEYBOARD}");
}

/// Enable mouse input, if available
///
/// # Errors
///
/// Never currently
pub fn enable_mouse_input() -> io::Result<()> {
    print!("{ENABLE_MOUSE}");
    Ok(())
}

/// Disable mouse input, if available
///
/// # Errors
///
/// Never currently
pub fn disable_mouse_input() -> io::Result<()> {
    print!("{DISABLE_MOUSE}");
    Ok(())
}

/// Enables raw mode, which disables line buffering, input echoing, and output canonicalization
///
/// # Errors
///
/// If there is no stdin,
/// stdin is not a tty,
/// or it fails to change terminal settings
pub fn enable_raw_mode() -> io::Result<()> {
    let mut termios = (*TERMIOS).map_err(|e| io::Error::from_raw_os_error(e))?;
    unsafe {
        cfmakeraw(&mut termios);
    }
    set_attributes(STDIN_FILENO, &mut termios)?;
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
    let mut termios = (*TERMIOS).map_err(|e| io::Error::from_raw_os_error(e))?;
    set_attributes(STDIN_FILENO, &mut termios)?;
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
/// If There is no stdout,
/// if stdout isn't a TTY, or
/// if it cannot change terminal properties on Windows
#[cfg(unix)]
pub fn enable_ansi() -> io::Result<()> {
    // ANSI is on by default on unix platforms
    // This is here for compatibility with the windows version of this API
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
    let mut winsize = Winsize::default();
    let ioctl_result = unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, (&raw mut winsize).cast::<u8>()) };

    if ioctl_result == 0 {
        Ok((winsize.col, winsize.row))
    } else {
        Err(io::Error::last_os_error())
    }
}

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
            try_parse_event(item, &mut read_iter)
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

fn try_parse_event<I>(item: u8, iter: &mut I) -> io::Result<Event>
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
        Some(Ok(b'M')) => parse_x10_mouse(iter),
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
    while c < 64 || c > 126 {
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
                _ => return None,
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
                _ => return None,
            }
        }

        _ => None,
    }
}

fn parse_x10_mouse<I>(iter: &mut I) -> Option<Event>
where
    I: Iterator<Item = io::Result<u8>>,
{
    // X10 emulation mouse encoding: ESC [ CB Cx Cy (6 characters only).
    let mut next = || iter.next().unwrap().unwrap();

    let cb = next() as i8 - 32;
    // (0, 0) are the coords for upper left.
    let cx = next().saturating_sub(33) as u16;
    let cy = next().saturating_sub(33) as u16;

    let mods = Modifiers::NONE;
    Some(match cb & 0b11 {
        0 => {
            if cb & 0x40 != 0 {
                Event::Mouse(mods, MouseButton::WheelUp, ButtonType::Press, cx, cy)
            } else {
                Event::Mouse(mods, MouseButton::WheelUp, ButtonType::Press, cx, cy)
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
    })
}

fn parse_xterm_mouse<I>(iter: &mut I) -> Option<Event>
where
    I: Iterator<Item = io::Result<u8>>,
{
    // xterm mouse encoding:
    // ESC [ < Cb ; Cx ; Cy (;) (M or m)
    let mut buf = Vec::new();
    let mut c = iter.next().unwrap().unwrap();
    while match c {
        b'm' | b'M' => false,
        _ => true,
    } {
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
    let trimmed_cb = cb ^ (cb & 0b00011100);

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
