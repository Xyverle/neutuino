use crate::input::{Event, KeyEvent, MouseEvent, MouseButton};
use std::io;

pub fn try_parse_event<I>(item: u8, iter: &mut I) -> io::Result<Event>
where
    I: Iterator<Item = io::Result<u8>>,
{
    match item {
        b'\x1b' => {
            try_parse_ansi_sequence(iter)
        }
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
            return Ok(string.chars().next().unwrap())
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
        Some(Ok(b'O')) => {
            match iter.next() {
                Some(Ok(val @ b'P'..=b's')) => Ok(Event::Key(KeyEvent::F(1 + val - b'P'))),
                _ => Err(error)
            }
        }
        Some(Ok(b'[')) => {
            try_parse_csi_sequence(iter).ok_or(error)
        }
        _ => Err(error),
    }
}

fn try_parse_csi_sequence<I>(iter: &mut I) -> Option<Event>
where
    I: Iterator<Item = io::Result<u8>>,
{
    match iter.next() {
        Some(Ok(b'[')) => {
            match iter.next() {
                Some(Ok(val @ b'A'..=b'E')) => Some(Event::Key(KeyEvent::F(1 + val - b'A'))),
                _ => None

            }
        }
        Some(Ok(b'D')) => Some(Event::Key(KeyEvent::Left)),
        Some(Ok(b'C')) => Some(Event::Key(KeyEvent::Right)),
        Some(Ok(b'A')) => Some(Event::Key(KeyEvent::Up)),
        Some(Ok(b'B')) => Some(Event::Key(KeyEvent::Down)),
        Some(Ok(b'H')) => Some(Event::Key(KeyEvent::Home)),
        Some(Ok(b'F')) => Some(Event::Key(KeyEvent::End)),
        Some(Ok(b'Z')) => Some(Event::Key(KeyEvent::BackTab)),
        Some(Ok(b'M')) => {
            try_parse_x10_mouse(iter)
        }
        Some(Ok(b'<')) => {
            try_parse_xterm_mouse(iter)
        }
        Some(Ok(c @ b'0'..=b'9')) => {
            try_parse_rxvt_mouse(c, iter)
        }
        _ => None

    }
}

fn try_parse_x10_mouse<I>(iter: &mut I) -> Option<Event>
where
    I: Iterator<Item = io::Result<u8>>,
{
    let cb = iter.next()?.ok()? - 32;
    
    let cx = u16::from(iter.next()?.ok()?.saturating_sub(33));
    let cy = u16::from(iter.next()?.ok()?.saturating_sub(33));
    match cb & 0x11 {
        0 => {
            if cb & 0x40 != 0 {
                Some(Event::Mouse(MouseEvent::Press(MouseButton::WheelUp, cx, cy)))
            } else {
                Some(Event::Mouse(MouseEvent::Press(MouseButton::Left, cx, cy)))
            }
        }
        1 => {
            if cb & 0x40 != 0 {
                Some(Event::Mouse(MouseEvent::Press(MouseButton::WheelDown, cx, cy)))
            
            } else {
                Some(Event::Mouse(MouseEvent::Press(MouseButton::Middle, cx, cy)))
            }
        }
        2 => {
            if cb & 0x40 != 0 {
                Some(Event::Mouse(MouseEvent::Press(MouseButton::WheelLeft, cx, cy)))
            } else {
                Some(Event::Mouse(MouseEvent::Press(MouseButton::Right, cx, cy)))
            }
        }
        3 => {
            if cb & 0x40 != 0 {
                Some(Event::Mouse(MouseEvent::Press(MouseButton::WheelRight, cx, cy)))
            } else {
                Some(Event::Mouse(MouseEvent::Release(cx, cy)))
            }
        }
        _ => None,
    }
}

fn try_parse_xterm_mouse<I>(iter: &mut I) -> Option<Event>
where
    I: Iterator<Item = io::Result<u8>>,
{
    let mut buf = Vec::new();
    let mut character = iter.next()?.ok()?;
    while !matches!(character, b'm' | b'M' ) {
        buf.push(character);
        character = iter.next()?.ok()?;
    }
    let str_buf = String::from_utf8(buf).ok()?;
    let nums = &mut str_buf.split(';');

    let cb = nums.next()?.parse::<u16>().ok()?;

    let cx = nums.next()?.parse::<u16>().ok()?;
    let cy = nums.next()?.parse::<u16>().ok()?;

    let event = match cb {
        0..=2 | 64..=67 => {
            let button = match cb {
                0 => MouseButton::Left,
                1 => MouseButton::Middle,
                2 => MouseButton::Right,
                64 => MouseButton::WheelUp,
                65 => MouseButton::WheelDown,
                66 => MouseButton::WheelLeft,
                67 => MouseButton::WheelRight,
                _ => unreachable!(),
            };
            match character {
                b'M' => MouseEvent::Press(button, cx, cy),
                b'm' => MouseEvent::Release(cx, cy),
                _ => return None,
            }
        },
        32 | 3 => MouseEvent::Hold(cx, cy),
        _ => return None,
    };
    Some(Event::Mouse(event))
}

fn try_parse_rxvt_mouse<I>(c: u8, iter: &mut I) -> Option<Event>
where
    I: Iterator<Item = io::Result<u8>>,
{
    let mut buf = vec![c];
    let mut c = iter.next()?.ok()?;
    while !(64..=126).contains(&c) {
        buf.push(c);
        c = iter.next()?.ok()?;
    }
    if c == b'M' {
        let str_buf = String::from_utf8(buf).ok()?;

        let nums: Vec<u16> = str_buf.split(';').map(|n| n.parse().unwrap()).collect();

        let cb = nums[0];
        let cx = nums[1];
        let cy = nums[2];

        let event = match cb {
            32 => MouseEvent::Press(MouseButton::Left, cx, cy),
            33 => MouseEvent::Press(MouseButton::Middle, cx, cy),
            34 => MouseEvent::Press(MouseButton::Right, cx, cy),
            35 => MouseEvent::Release(cx, cy),
            64 => MouseEvent::Hold(cx, cy),
            96 | 97 => MouseEvent::Press(MouseButton::WheelUp, cx, cy),
            _ => return None,
        };

        return Some(Event::Mouse(event));
    }
    None
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
