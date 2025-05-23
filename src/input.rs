//! Various input functions, structs, etc.
//!
//! Very incomplete currently

/// Different events that can happen through the terminal
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Event {
    /// An event that happens upon a key being pressed
    Key(KeyEvent),
    /// An event that happens upon focus to the terminal window being gained
    FocusGained,
    /// An event that happens upon focus to the terminal window being lost
    FocusLost,
}

/// An event that happens upon a key being pressed
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum KeyEvent {
    /// The Backspace key
    Backspace,
    /// The Up arrow key
    Up,
    /// The Down arrow key
    Down,
    /// The Left arrow key
    Left,
    /// The Right arrow key
    Right,
    /// The Home key
    Home,
    /// The End key
    End,
    /// The PageUp key
    PageUp,
    /// The PageDown key
    PageDown,
    /// The Tab key
    Tab,
    /// Shift + Tab key
    ShiftTab,
    /// The delete key
    Delete,
    /// The insert key
    Insert,
    /// The f1-f12 keys
    F(u8),
    /// Any character inputted by the keyboard
    Char(char),
    /// Ctrl + Char
    Ctrl(char),
    /// The Escape key
    Escape,
    /// A null byte sent to the terminal
    ///
    /// Can mean several different things
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
