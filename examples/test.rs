#![warn(clippy::all, clippy::pedantic)]

use neutuino::ansi::{
    COLORS_BG, COLORS_FG, STYLE_BOLD, STYLE_ITALIC, STYLE_RESET, STYLE_UNDERLINE,
    move_cursor_to_column, set_window_title,
};
use neutuino::input::{Event, KeyEvent, poll_input};
use neutuino::os::{disable_raw_mode, enable_ansi, enable_raw_mode, get_terminal_size};
use std::{io, time::Duration};

fn print_line_style_reset(string: &str) {
    println!("{}{}{}", string, STYLE_RESET, move_cursor_to_column(0));
}

fn main() -> io::Result<()> {
    let all_styles = format!("{STYLE_BOLD}{STYLE_ITALIC}{STYLE_UNDERLINE}");

    enable_ansi()?;
    enable_raw_mode()?;

    println!("q to quit{}", move_cursor_to_column(0));
    let next = |x: usize| (x + 1) % COLORS_FG.len();

    let terminal_size = get_terminal_size()?;
    let terminal_size_str = format!("{terminal_size:?}");
    print!("{}", set_window_title(terminal_size_str).unwrap());

    let mut counter = 0;

    loop {
        let mut input = Err(io::ErrorKind::Other.into());
        while input.is_err() {
            input = poll_input(Duration::new(1, 0));
        }
        let input = input.unwrap();
        let string = format!("{input:?}");
        print_line_style_reset(&format!(
            "{all_styles}{}{}{string}",
            COLORS_FG[counter],
            COLORS_BG[next(counter)]
        ));
        // q to quit
        if input == Event::Key(KeyEvent::Char('q')) {
            break;
        }
        counter = next(counter);
    }
    disable_raw_mode()?;
    Ok(())
}
