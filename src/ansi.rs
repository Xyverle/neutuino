use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CursorMovement {
    Move(u16, u16),
    Up(u16),
    Down(u16),
    Right(u16),
    Left(u16),
    Save,
    Restore,
}

impl fmt::Display for CursorMovement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match *self {
            Self::Move(x, y) => &format!("[{x};{y}H"),
            Self::Up(n) => &format!("[{n}A"),
            Self::Down(n) => &format!("[{n}B"),
            Self::Left(n) => &format!("[{n}D"),
            Self::Right(n) => &format!("[{n}C"),
            Self::Save => "7",
            Self::Restore => "8",
        };
        write!(f, "\x1b{str}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CursorShape {
    Reset,
    BlockBlinking,
    BlockSteady,
    UnderlineBlinking,
    UnderlineSteady,
    BarBlinking,
    BarSteady,
}

impl fmt::Display for CursorShape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match *self {
            Self::Reset => "0",
            Self::BlockBlinking => "1",
            Self::BlockSteady => "2",
            Self::UnderlineBlinking => "3",
            Self::UnderlineSteady => "4",
            Self::BarBlinking => "5",
            Self::BarSteady => "6",
        };
        write!(f, "\x1b[{str} q")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Erase {
    Screen,
    Line,
    CursorToScreenStart,
    CursorToScreenEnd,
    CursorToLineStart,
    CursorToLineEnd,
}

impl fmt::Display for Erase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match *self {
            Self::Screen => "2J",
            Self::Line => "2K",
            Self::CursorToScreenStart => "1J",
            Self::CursorToScreenEnd => "0J",
            Self::CursorToLineStart => "1K",
            Self::CursorToLineEnd => "0K",
        };
        write!(f, "\x1b[{str}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Style {
    ResetAll,
    Bold,
    Dim,
    Italic,
    Underline,
    Blinking,
    Reverse,
    Hidden,
    Strikethrough,
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match *self {
            Self::ResetAll => "0",
            Self::Bold => "1",
            Self::Dim => "2",
            Self::Italic => "3",
            Self::Underline => "4",
            Self::Blinking => "5",
            Self::Reverse => "7",
            Self::Hidden => "8",
            Self::Strikethrough => "9",
        };
        write!(f, "\x1b[{str}m")
    }
}

impl fmt::LowerHex for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match *self {
            Self::ResetAll => "0",
            Self::Bold | Self::Dim => "22",
            Self::Italic => "23",
            Self::Underline => "24",
            Self::Blinking => "25",
            Self::Reverse => "27",
            Self::Hidden => "28",
            Self::Strikethrough => "29",
        };
        write!(f, "\x1b[{str}m")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Rgb(u8, u8, u8),
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Default,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match *self {
            Self::Rgb(r, g, b) => &format!("38;2;{r};{g};{b}"),
            Self::Black => "30",
            Self::Red => "31",
            Self::Green => "32",
            Self::Yellow => "33",
            Self::Blue => "34",
            Self::Magenta => "35",
            Self::Cyan => "36",
            Self::White => "37",
            Self::Default => "39",
        };
        write!(f, "\x1b[{str}m")
    }
}

impl fmt::Binary for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match *self {
            Self::Rgb(r, g, b) => &format!("48;2;{r};{g};{b}"),
            Self::Black => "40",
            Self::Red => "41",
            Self::Green => "42",
            Self::Yellow => "43",
            Self::Blue => "44",
            Self::Magenta => "45",
            Self::Cyan => "46",
            Self::White => "47",
            Self::Default => "49",
        };
        write!(f, "\x1b[{str}m")
    }
}

pub fn enter_alternate_screen() {
    println!("\x1b[?1049h");
}

pub fn exit_alternate_screen() {
    println!("\x1b[?1049l");
}

pub fn set_window_title(title: &str) {
    assert!(
        title.len() <= 255,
        "Title length longer than maximum of 255"
    );
    println!("\x1b]0;{title}\x1b\x5c");
}
