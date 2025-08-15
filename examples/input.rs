#![warn(clippy::all, clippy::pedantic)]

use neutuino::prelude::*;
use std::io::IsTerminal;
use std::{io, time::Duration};

fn print_line_style_reset(string: &str) {
    println!("{}{}{}", string, STYLE_RESET, move_cursor_to_column(0));
}

fn main() -> io::Result<()> {
    assert!(io::stdout().is_terminal(), "Not running in a terminal");

    enable_ansi()?;
    enable_raw_mode()?;
    enable_mouse_input()?;
    // enable_kitty_keyboard();

    println!("q to quit{}", move_cursor_to_column(0));
    let next = |x: usize| (x + 1) % COLORS_FG.len();

    let terminal_size = get_terminal_size()?;
    let terminal_size_str = format!("{:?}", terminal_size);
    print!("{}", set_window_title(terminal_size_str).unwrap());

    let mut counter = 0;

    loop {
        let input = poll_input(Duration::new(1, 0));
        let string = format!("{input:?}");
        match &input {
            Err(e) => match e.kind() {
                io::ErrorKind::TimedOut => {}
                _ => {
                    print_line_style_reset(&string);
                }
            },
            Ok(_) => {
                print_line_style_reset(&string);
            }
        }
        // q to quit
        if input.is_ok()
            && input.unwrap() == Event::Key(Key::Char('q'), ButtonType::Press, Modifiers::NONE)
        {
            break;
        }
        counter = next(counter);
    }

    // disable_kitty_keyboard();
    disable_raw_mode()?;
    disable_mouse_input()?;
    Ok(())
}
