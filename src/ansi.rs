use std::fmt;

pub struct CursorHome {}

impl fmt::Display for CursorHome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}

pub enum Cursor {
    Move(u16, u16),
    Up(u16),
    Down(u16),
    Right(u16),
    Left(u16),
    Column(u16),
    Home,
    Save,
    Restore,
}
