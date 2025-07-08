//! Input Utilities
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
    Key(Key, KeyType, KeyMods),
    /// An event that happens upon focus to the terminal window being gained
    FocusGained,
    /// An event that happens upon focus to the terminal window being lost
    FocusLost,
}

/// The base key that was pressed
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
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct KeyMods {
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
    pub meta: bool,
}

impl KeyMods {
    pub const NONE: Self = Self {
        shift: false,
        alt: false,
        ctrl: false,
        meta: false,
    };
    pub const SHIFT: Self = Self::NONE.shift(true);
    pub const ALT: Self = Self::NONE.alt(true);
    pub const CTRL: Self = Self::NONE.ctrl(true);
    pub const META: Self = Self::NONE.meta(true);
    #[must_use]
    pub const fn shift(mut self, on: bool) -> Self {
        self.shift = on;
        self
    }
    #[must_use]
    pub const fn alt(mut self, on: bool) -> Self {
        self.alt = on;
        self
    }
    #[must_use]
    pub const fn ctrl(mut self, on: bool) -> Self {
        self.ctrl = on;
        self
    }
    #[must_use]
    pub const fn meta(mut self, on: bool) -> Self {
        self.meta = on;
        self
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

pub fn press_key(key: Key, key_mods: KeyMods) -> Event {
    Event::Key(key, KeyType::Press, key_mods)
}

#[cfg(unix)]
pub use crate::unix::poll_input;

#[cfg(windows)]
pub use crate::windows::poll_input;
