#![warn(clippy::all, clippy::pedantic)]

use neutuino::prelude::*;
use std::{
    io::{self, IsTerminal, Write},
    thread, time,
};

fn main() -> io::Result<()> {
    assert!(io::stdout().is_terminal(), "Not running in a terminal");

    enable_ansi()?;

    // makes the terminal raw until this value is dropped
    enable_raw_mode()?;
    println!("{ALT_SCREEN_ENTER}");

    // gets the size of the terminal
    let terminal_size = get_terminal_size()?;
    let middle = (terminal_size.0 / 2, terminal_size.1 / 2);

    let string = "Hello, World!";

    let adjusted_middle = (middle.0 - ((string.len() / 2) as u16), middle.1);

    print!(
        "{COLOR_RED_BG}{}{string}",
        move_cursor_to_position(adjusted_middle.0, adjusted_middle.1)
    );
    io::stdout().flush()?; // VERY IMPORTANT!

    thread::sleep(time::Duration::new(3, 0));

    disable_raw_mode()?;
    println!("{ALT_SCREEN_EXIT}");

    // no flush needed here as the program is about to end and it will be auto flushed
    Ok(())
}
