//! Various input functions, structs, etc.
//!
//! Very incomplete currently
//!
//! # Support
//! This library attempts to support as much on all platforms but many platforms are very weird
//!
//! In general the best support will be on Kitty-like linux terminals and Windows, due to historical
//! reasons input on normal *nix terminals are limited

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
pub struct KeyEvent(pub Key, pub KeyType, pub KeyModifiers);

/// An event that happens upon a key being pressed
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Key {
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
    /// The Escape key
    Escape,
    /// A null byte sent to the terminal
    ///
    /// Can mean several different things
    Null,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(clippy::struct_excessive_bools)]
pub struct KeyModifiers {
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
    pub meta: bool,
}

impl KeyModifiers {
    #[must_use]
    pub const fn none() -> Self {
        Self {
            shift: false,
            alt: false,
            ctrl: false,
            meta: false,
        }
    }
    #[must_use]
    pub const fn shift(self) -> Self {
        let mut value = self;
        value.shift = true;
        value
    }
    #[must_use]
    pub const fn alt(self) -> Self {
        let mut value = self;
        value.alt = true;
        value
    }
    #[must_use]
    pub const fn ctrl(self) -> Self {
        let mut value = self;
        value.ctrl = true;
        value
    }
    #[must_use]
    pub const fn meta(self) -> Self {
        let mut value = self;
        value.meta = true;
        value
    }
}

/// This is the type of the key that is sent to the terminal
///
/// This is implemented in Windows, and Kitty-like Terminals
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum KeyType {
    Press,
    Repeat,
    Release,
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
