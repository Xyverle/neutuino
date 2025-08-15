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
    Key(Key, ButtonType, Modifiers),
    /// An event that happens upon a mouse action
    ///
    /// The last two are the x and y position of the event, 0-based
    Mouse(Modifiers, MouseButton, ButtonType, u16, u16),
    /// An event that happens upon focus to the terminal window being gained
    FocusGained,
    /// An event that happens upon focus to the terminal window being lost
    FocusLost,
}

/// The key on the mouse that was pressed
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MouseButton {
    /// The left mouse button
    Left,
    /// The right mouse button
    Right,
    /// The middle mouse button
    Middle,
    /// The mouse wheel going up
    WheelUp,
    /// The mouse wheel going down
    WheelDown,
    /// The mouse wheel going left
    WheelLeft,
    /// The mouse wheel going right
    WheelRight,
    /// The protocol does not specify, typically only on release/held buttons
    Unknown,
    /// No mouse button was pressed
    None,
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
pub struct Modifiers {
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
}

impl Modifiers {
    pub const fn new(shift: bool, alt: bool, ctrl: bool) -> Self {
        Self { shift, alt, ctrl }
    }

    pub const NONE: Self = Self {
        shift: false,
        alt: false,
        ctrl: false,
    };
    pub const SHIFT: Self = Self::NONE.shift(true);
    pub const ALT: Self = Self::NONE.alt(true);
    pub const CTRL: Self = Self::NONE.ctrl(true);
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
}

/// This is the type of the key that is sent to the terminal
///
/// This is implemented on keys on Windows and Kitty-like Terminals
///
/// This is implemented on most mouse implementations
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ButtonType {
    Press,
    Held,
    Release,
}

#[inline(always)]
pub(crate) const fn key_helper(mods: &str, key: Key) -> Event {
    let mut key_mods = Modifiers::NONE;
    let mut key_type = ButtonType::Press;

    let string = mods.as_bytes();
    let mut i = 0;
    while i < string.len() {
        key_mods.alt = key_mods.alt | (string[i] == b'A');
        key_mods.ctrl = key_mods.ctrl | (string[i] == b'C');
        key_mods.shift = key_mods.shift | (string[i] == b'S');
        if string[i] == b'-' {
            key_type = ButtonType::Release;
        }
        if string[i] == b'*' {
            key_type = ButtonType::Held;
        }
        i += 1;
    }
    Event::Key(key, key_type, key_mods)
}

#[cfg(unix)]
pub(crate) const fn simple_key(key: Key, shift: bool, alt: bool, ctrl: bool) -> Event {
    Event::Key(key, ButtonType::Press, Modifiers::new(shift, alt, ctrl))
}

#[cfg(unix)]
pub use crate::unix::poll_input;

#[cfg(windows)]
pub use crate::windows::poll_input;

#[test]
fn test_key_helper() {
    let event = key_helper("ACS*", Key::Char('c'));
    assert_eq!(
        event,
        Event::Key(
            Key::Char('c'),
            ButtonType::Held,
            Modifiers::SHIFT.ctrl(true).alt(true)
        )
    )
}
