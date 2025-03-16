//! Collection of ANSI escape code consts/functions
//!
//! These should work on *most* terminals (i.e. Xterm compatible terminals)
//!
//! For these to work on Windows you need to run the `enable_ansi` function in the os module

/// Sets the terminal to an arbitrary 12-bit/truecolor color
#[must_use]
pub fn rgb_color_code(red: u8, green: u8, blue: u8) -> String {
    format!("\x1b[38;2;{red};{green};{blue}m")
}

/// Sets the title of the window
///
/// The title must be only in ASCII characters or **weird** things will happen
///
/// # Panics
///
/// When the title is more than 255 characters
#[must_use]
pub fn set_window_title(title: &str) -> String {
    assert!(
        title.len() <= 255,
        "Title length longer than maximum of 255"
    );
    format!("\x1b]0;{title}\x1b\x5c")
}

/// Moves the cursor up {num} characters
#[must_use]
pub fn move_cursor_up(num: u16) -> String {
    format!("\x1b[{num}A")
}

/// Moves the cursor down {num} characters
#[must_use]
pub fn move_cursor_down(num: u16) -> String {
    format!("\x1b[{num}B")
}

/// Moves the cursor right {num} characters
#[must_use]
pub fn move_cursor_right(num: u16) -> String {
    format!("\x1b[{num}C")
}

/// Moves the cursor left {num} characters
#[must_use]
pub fn move_cursor_left(num: u16) -> String {
    format!("\x1b[{num}A")
}

/// Moves the cursor to {row}
///
/// Origin is 1, 1
#[must_use]
pub fn move_cursor_to_row(line: u16) -> String {
    debug_assert!(
        line != 0,
        "Tried to go to line 0, when position is (1, 1)-based"
    );
    format!("\x1b[{line}d")
}

/// Moves the cursor to {column}
///
/// Origin is 1, 1
#[must_use]
pub fn move_cursor_to_column(column: u16) -> String {
    debug_assert!(
        column != 0,
        "Tried to go to column 0, when position is (1, 1)-based"
    );
    format!("\x1b[{column}G")
}

/// Moves the cursor to Position {x}, {y}
///
/// Origin is 1, 1
#[must_use]
pub fn move_cursor_to_position(column: u16, line: u16) -> String {
    debug_assert!(
        line != 0,
        "Tried to go to line 0, when position is (1, 1)-based"
    );
    debug_assert!(
        column != 0,
        "Tried to go to column 0, when position is (1, 1)-based"
    );
    format!("\x1b[{line};{column}H")
}

/// Saves the current cursor position
pub const CURSOR_POSITION_SAVE: &str = "\x1b7";
/// Restores the saved cursor position
pub const CURSOR_POSITION_RESTORE: &str = "\x1b8";

/// Enters the alternate screen
///
/// The alternate screen is a blank screen that won't interrupt the main screen (e.g. vi)
pub const ALT_SCREEN_ENTER: &str = "\x1b[?1049h";
/// Exits the alternate screen
///
/// The alternate screen is a blank screen that won't interrupt the main screen (e.g. vi)
pub const ALT_SCREEN_EXIT: &str = "\x1b[?1049l";

/// Sets the cursor shape to the user-specified default
pub const SHAPE_RESET: &str = "\x1b[0q";
/// Sets the cursor shape to a blinking block
pub const SHAPE_BLOCK_BLINKING: &str = "\x1b[1q";
/// Sets the cursor shape to a steady block
pub const SHAPE_BLOCK_STEADY: &str = "\x1b[2q";
/// Sets the cursor shape to a blinking underline
pub const SHAPE_UNDERLINE_BLINKING: &str = "\x1b[3q";
/// Sets the cursor shape to a steady underline
pub const SHAPE_UNDERLINE_STEADY: &str = "\x1b[4q";
/// Sets the cursor shape to a blinking bar
pub const SHAPE_BAR_BLINKING: &str = "\x1b[5q";
/// Sets the cursor shape to a steady bar
pub const SHAPE_BAR_STEADY: &str = "\x1b[6q";

/// Erases the entire screen while leaving cursor in place
pub const ERASE_SCREEN: &str = "\x1b[2J";
/// Erases the line the cursor is on while leaving cursor in place
pub const ERASE_LINE: &str = "\x1b[2K";
/// Erases from the screen start to the cursor while leaving cursor in place
pub const ERASE_CURSOR_TO_SCREEN_START: &str = "\x1b[1J";
/// Erases from the cursor to the screen end while leaving cursor in place
pub const ERASE_CURSOR_TO_SCREEN_END: &str = "\x1b[0J";
/// Erases from the line start to the cursor while leaving cursor in place
pub const ERASE_CURSOR_TO_LINE_START: &str = "\x1b[1K";
/// Erases from the cursor to the line end while leaving cursor in place
pub const ERASE_CURSOR_TO_LINE_END: &str = "\x1b[0K";

/// Makes characters sent to the screen bold
pub const STYLE_BOLD: &str = "\x1b[1m";
/// Makes characters sent to the screen dim
pub const STYLE_DIM: &str = "\x1b[2m";
/// Makes characters sent to the screen italic
pub const STYLE_ITALIC: &str = "\x1b[3m";
/// Makes characters sent to the screen underlined
///
/// This is less commonly supported than other styles
pub const STYLE_UNDERLINE: &str = "\x1b[4m";
/// Makes characters sent to the screen blinking
///
/// This is less commonly supported than other styles
pub const STYLE_BLINKING: &str = "\x1b[5m";
/// Makes characters sent to the screen reversed
///
/// This is less commonly supported than other styles
pub const STYLE_REVERSE: &str = "\x1b[7m";
/// Makes characters sent to the screen hidden
///
/// This is less commonly supported than other styles
pub const STYLE_HIDDEN: &str = "\x1b[8m";
/// Makes characters sent to the screen struckthrough
///
/// This is less commonly supported than other styles
pub const STYLE_STRIKETHROUGH: &str = "\x1b[9m";

/// Resets all styles and colors
pub const STYLE_RESET: &str = "\x1b[0m";
/// Resets bold
///
/// Often bold & dim's implementations are overlapping and will likely unset both
pub const STYLE_RESET_BOLD: &str = "\x1b[21m";
/// Resets dim
///
/// Often bold & dim's implementations are overlapping and will likely unset both
pub const STYLE_RESET_DIM: &str = "\x1b[22m";
/// Reset italic
pub const STYLE_RESET_ITALIC: &str = "\x1b[23m";
/// Reset underline
pub const STYLE_RESET_UNDERLINE: &str = "\x1b[24m";
/// Reset blinking
pub const STYLE_RESET_BLINKING: &str = "\x1b[25m";
/// Reset reverse
pub const STYLE_RESET_REVERSE: &str = "\x1b[27m";
/// Reset hidden
pub const STYLE_RESET_HIDDEN: &str = "\x1b[28m";
/// Reset strikethrough
pub const STYLE_RESET_STRIKETHROUGH: &str = "\x1b[29m";

/// Makes characters sent to the screen have a black foreground
pub const COLOR_BLACK_FG: &str = "\x1b[30m";
/// Makes characters sent to the screen have a black background
pub const COLOR_BLACK_BG: &str = "\x1b[40m";
/// Makes characters sent to the screen have a red foreground
pub const COLOR_RED_FG: &str = "\x1b[31m";
/// Makes characters sent to the screen have a red background
pub const COLOR_RED_BG: &str = "\x1b[41m";
/// Makes characters sent to the screen have a green foreground
pub const COLOR_GREEN_FG: &str = "\x1b[32m";
/// Makes characters sent to the screen have a green background
pub const COLOR_GREEN_BG: &str = "\x1b[42m";
/// Makes characters sent to the screen have a yellow foreground
pub const COLOR_YELLOW_FG: &str = "\x1b[33m";
/// Makes characters sent to the screen have a yellow background
pub const COLOR_YELLOW_BG: &str = "\x1b[43m";
/// Makes characters sent to the screen have a blue foreground
pub const COLOR_BLUE_FG: &str = "\x1b[34m";
/// Makes characters sent to the screen have a blue background
pub const COLOR_BLUE_BG: &str = "\x1b[44m";
/// Makes characters sent to the screen have a magenta foreground
pub const COLOR_MAGENTA_FG: &str = "\x1b[35m";
/// Makes characters sent to the screen have a magenta background
pub const COLOR_MAGENTA_BG: &str = "\x1b[45m";
/// Makes characters sent to the screen have a cyan foreground
pub const COLOR_CYAN_FG: &str = "\x1b[36m";
/// Makes characters sent to the screen have a cyan background
pub const COLOR_CYAN_BG: &str = "\x1b[46m";
/// Makes characters sent to the screen have a white foreground
pub const COLOR_WHITE_FG: &str = "\x1b[37m";
/// Makes characters sent to the screen have a white background
pub const COLOR_WHITE_BG: &str = "\x1b[47m";
/// Makes characters sent to the screen have a default foreground
pub const COLOR_DEFAULT_FG: &str = "\x1b[39m";
/// Makes characters sent to the screen have a default background
pub const COLOR_DEFAULT_BG: &str = "\x1b[49m";
