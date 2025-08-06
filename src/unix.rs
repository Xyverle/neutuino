use std::ffi::{c_int, c_uint, c_ulong, c_ushort};
use std::io;
use std::sync::LazyLock;

use crate::input::{Event, Key, KeyMods, KeyType, press_key};
use std::ffi::{c_short, c_void};
use std::time::Duration;

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

const ERROR_MAGIC: i32 = 68905;

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
        Err(e) => Err(e.raw_os_error().unwrap_or(ERROR_MAGIC)),
    }
});

/// Enables raw mode, which disables line buffering, input echoing, and output canonicalization
///
/// # Errors
///
/// If there is no stdin,
/// stdin is not a tty,
/// or it fails to change terminal settings
pub fn enable_raw_mode() -> io::Result<()> {
    let mut termios = (*TERMIOS).map_err(|e| {
        if e == ERROR_MAGIC {
            return io::Error::other("Failed to get terminal properties");
        }
        io::Error::from_raw_os_error(e)
    })?;
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
    let mut termios = (*TERMIOS).map_err(|e| {
        if e == ERROR_MAGIC {
            return io::Error::other("Failed to get terminal properties");
        }
        io::Error::from_raw_os_error(e)
    })?;
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
    let mut fds = [PollFD {
        fd: STDIN_FILENO,
        events: POLLIN,
        revents: 0,
    }];
    let result = unsafe {
        #[allow(clippy::cast_possible_truncation)]
        poll(
            fds.as_mut_ptr(),
            fds.len() as c_ulong,
            timeout.as_millis() as c_int,
        )
    };
    let mut read_iter = ReadIterator::new(STDIN_FILENO);

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
    fd: c_int,
    buf: u8,
}

impl ReadIterator {
    fn new(fd: c_int) -> Self {
        Self { fd, buf: 0 }
    }
}

impl Iterator for ReadIterator {
    type Item = io::Result<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        let bytes_read = unsafe { read(self.fd, (&raw mut self.buf).cast::<c_void>(), 1) };

        match bytes_read {
            1.. => Some(Ok(self.buf)),
            0 => None,
            _ => Some(Err(io::Error::last_os_error())),
        }
    }
}

fn try_parse_event<I>(item: u8, iter: &mut I) -> io::Result<Event>
where
    I: Iterator<Item = io::Result<u8>>,
{
    match item {
        b'\x1b' => try_parse_ansi_sequence(iter),
        b'\r' => Ok(press_key(Key::Char('\r'), KeyMods::NONE)),
        b'\n' => Ok(press_key(Key::Char('j'), KeyMods::CTRL)),
        b'\t' => Ok(press_key(Key::Char('\t'), KeyMods::NONE)),
        b'\x7f' => Ok(press_key(Key::Backspace, KeyMods::NONE)),
        b'\0' => Ok(press_key(Key::Char(' '), KeyMods::CTRL)),
        c @ b'\x01'..=b'\x1a' => Ok(press_key(Key::Char((c + 96) as char), KeyMods::CTRL)),
        c @ b'\x1c'..=b'\x1f' => Ok(press_key(Key::Char((c + 24) as char), KeyMods::CTRL)),
        c => {
            let character = parse_utf8_char(c, iter)?;
            Ok(Event::Key(
                Key::Char(character),
                KeyType::Press,
                KeyMods::NONE.shift(character.is_uppercase()),
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

fn try_parse_ansi_sequence<I>(iter: &mut I) -> io::Result<Event>
where
    I: Iterator<Item = io::Result<u8>>,
{
    let error = io::Error::other("Could not parse event");
    match iter.next() {
        Some(Ok(b'O')) => match iter.next() {
            Some(Ok(val @ b'P'..=b's')) => Ok(press_key(Key::F(1 + val - b'P'), KeyMods::NONE)),
            _ => Err(error),
        },
        Some(Ok(b'[')) => try_parse_csi_sequence(iter).ok_or(error),
        Some(Ok(c)) => match c {
            b'\r' => Ok(press_key(Key::Char('\r'), KeyMods::ALT)),
            b'\n' => Ok(press_key(Key::Char('j'), KeyMods::CTRL.alt(true))),
            b'\t' => Ok(press_key(Key::Char('\t'), KeyMods::ALT)),
            b'\x7f' => Ok(press_key(Key::Backspace, KeyMods::ALT)),
            b'\0' => Ok(press_key(Key::Char(' '), KeyMods::CTRL.alt(true))),
            c @ b'\x01'..=b'\x1a' => Ok(press_key(
                Key::Char((c + 96) as char),
                KeyMods::CTRL.alt(true),
            )),
            c @ b'\x1c'..=b'\x1f' => Ok(press_key(
                Key::Char((c + 24) as char),
                KeyMods::CTRL.alt(true),
            )),
            c => {
                let character = parse_utf8_char(c, iter)?;
                Ok(Event::Key(
                    Key::Char(character),
                    KeyType::Press,
                    KeyMods::NONE.shift(character.is_uppercase()).alt(true),
                ))
            }
        },
        _ => Err(error),
    }
}

fn try_parse_csi_sequence<I>(iter: &mut I) -> Option<Event>
where
    I: Iterator<Item = io::Result<u8>>,
{
    match iter.next() {
        Some(Ok(b'[')) => match iter.next() {
            Some(Ok(val @ b'A'..=b'E')) => Some(press_key(Key::F(1 + val - b'A'), KeyMods::NONE)),
            _ => None,
        },
        Some(Ok(b'D')) => Some(press_key(Key::Left, KeyMods::NONE)),
        Some(Ok(b'C')) => Some(press_key(Key::Right, KeyMods::NONE)),
        Some(Ok(b'A')) => Some(press_key(Key::Up, KeyMods::NONE)),
        Some(Ok(b'B')) => Some(press_key(Key::Down, KeyMods::NONE)),
        Some(Ok(b'H')) => Some(press_key(Key::Home, KeyMods::NONE)),
        Some(Ok(b'F')) => Some(press_key(Key::End, KeyMods::NONE)),
        Some(Ok(b'Z')) => Some(press_key(Key::Tab, KeyMods::SHIFT)),
        _ => None,
    }
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
