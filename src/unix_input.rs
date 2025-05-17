use crate::input::{Event, KeyEvent, MouseButton, MouseEvent};
use crate::os::{POLLIN, STDIN_FILENO};
use std::ffi::{c_int, c_short, c_ulong, c_void};
use std::io;
use std::time::Duration;

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

fn try_parse_event<I>(item: u8, iter: &mut I) -> io::Result<Event>
where
    I: Iterator<Item = io::Result<u8>>,
{
    match item {
        b'\x1b' => try_parse_ansi_sequence(iter),
        b'\n' | b'\r' => Ok(Event::Key(KeyEvent::Char('\n'))),
        b'\t' => Ok(Event::Key(KeyEvent::Tab)),
        b'\x7f' => Ok(Event::Key(KeyEvent::Backspace)),
        b'\0' => Ok(Event::Key(KeyEvent::Null)),
        c @ b'\x01'..=b'\x1a' => Ok(Event::Key(KeyEvent::Ctrl((c + 96) as char))),
        c @ b'\x1c'..=b'\x1f' => Ok(Event::Key(KeyEvent::Ctrl((c + 24) as char))),
        c => Ok(Event::Key(KeyEvent::Char(parse_utf8_char(c, iter)?))),
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
    let error = io::Error::new(io::ErrorKind::Other, "Could not parse event");
    match iter.next() {
        Some(Ok(b'O')) => match iter.next() {
            Some(Ok(val @ b'P'..=b's')) => Ok(Event::Key(KeyEvent::F(1 + val - b'P'))),
            _ => Err(error),
        },
        Some(Ok(b'[')) => try_parse_csi_sequence(iter).ok_or(error),
        _ => Err(error),
    }
}

fn try_parse_csi_sequence<I>(iter: &mut I) -> Option<Event>
where
    I: Iterator<Item = io::Result<u8>>,
{
    match iter.next() {
        Some(Ok(b'[')) => match iter.next() {
            Some(Ok(val @ b'A'..=b'E')) => Some(Event::Key(KeyEvent::F(1 + val - b'A'))),
            _ => None,
        },
        Some(Ok(b'D')) => Some(Event::Key(KeyEvent::Left)),
        Some(Ok(b'C')) => Some(Event::Key(KeyEvent::Right)),
        Some(Ok(b'A')) => Some(Event::Key(KeyEvent::Up)),
        Some(Ok(b'B')) => Some(Event::Key(KeyEvent::Down)),
        Some(Ok(b'H')) => Some(Event::Key(KeyEvent::Home)),
        Some(Ok(b'F')) => Some(Event::Key(KeyEvent::End)),
        Some(Ok(b'Z')) => Some(Event::Key(KeyEvent::BackTab)),
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
