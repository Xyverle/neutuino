use neutuino::prelude::*;
use std::io::{self, IsTerminal, Read, Write};

fn main() -> io::Result<()> {
    assert!(io::stdout().is_terminal(), "Not running in a terminal");
    enable_ansi()?;
    enable_raw_mode()?;
    // enable_kitty_keyboard();
    io::stdout().flush()?;
    print!("\x1b[?1003h");

    let mut buf = [0; 1];
    let mut stdin = io::stdin();
    while buf != [b'q'] {
        _ = stdin.read(&mut buf);
        if buf[0] > 127 || !(buf[0] as char).is_control() {
            println!("{:x} \"{}\"\r", buf[0], buf[0] as char);
        } else {
            println!("{:x} ?\r", buf[0]);
        }
    }

    // disable_kitty_keyboard();
    disable_raw_mode()?;
    print!("\x1b[?1003l");
    Ok(())
}
