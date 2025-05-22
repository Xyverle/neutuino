#![warn(clippy::all, clippy::pedantic)]

pub mod ansi;

#[cfg(unix)]
mod unix;

#[cfg(unix)]
pub use crate::unix::*;

#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use crate::windows::*;

pub mod input {
    //! Various input functions, structs, etc.
    //!
    //! Very incomplete currently

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Event {
        Key(KeyEvent),
        FocusGained,
        FocusLost,
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum KeyEvent {
        Backspace,
        Up,
        Down,
        Left,
        Right,
        Home,
        End,
        PageUp,
        PageDown,
        Tab,
        BackTab,
        Delete,
        Insert,
        F(u8),
        Char(char),
        Ctrl(char),
        Escape,
        Null,
    }

    impl From<KeyEvent> for Event {
        fn from(value: KeyEvent) -> Self {
            Self::Key(value)
        }
    }

    #[cfg(unix)]
    pub use crate::unix::input::*;

    #[cfg(windows)]
    pub use crate::windows::input::*;
}
