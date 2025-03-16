use std::ffi::{c_int, c_uint, c_ulong, c_ushort};
use std::io;

unsafe extern "C" {
    fn ioctl(fd: c_int, request: c_ulong, argp: *mut u8) -> c_int;
    fn tcgetattr(fd: c_int, termios_p: *mut Termios) -> c_int;
    fn tcsetattr(fd: c_int, optional_actions: c_int, termios: *mut Termios) -> c_int;
}

#[cfg(any(target_os = "linux", target_os = "redox"))]
const TIOCGWINSZ: c_ulong = 0x5413;

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
const TIOCGWINSZ: c_ulong = 0x40087468;

#[cfg(any(target_os = "linux", target_os = "redox"))]
const NCCS: usize = 32;

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
const NCCS: usize = 20;

const STDIN_FILENO: c_int = 0;
const STDOUT_FILENO: c_int = 1;

const ECHO: c_uint = 8;
const ICANON: c_uint = 2;
const ISIG: c_uint = 1;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Winsize {
    row: c_ushort,
    col: c_ushort,
    xpixel: c_ushort,
    ypixel: c_ushort,
}

#[repr(C)]
#[derive(Debug, Clone)]
struct Termios {
    iflag: c_uint,
    oflag: c_uint,
    cflag: c_uint,
    lflag: c_uint,
    cc: [u8; NCCS],
}

/// Enables ANSI support on Windows terminals
///
/// ANSI is on by default on *nix machines but still exists on them for simpler usage
///
/// # Errors
///
/// Never on *nix
///
/// If There is no stdout,
/// if stdout isn't a TTY, or
/// if it cannot change terminal properties on Windows
pub fn enable_ansi() -> io::Result<()> {
    // ANSI is on by default on unix platforms
    // This is here for compatibility with the windows version of this API
    Ok(())
}

/// Gets the size of the terminal
///
/// Returns in (width, height) format
///
/// # Errors
///
/// If there is no stdout,
/// if stdout isn't a TTY, or
/// if it fails to retrieve the terminal size
pub fn get_terminal_size() -> io::Result<(u16, u16)> {
    let mut winsize = unsafe { std::mem::zeroed::<Winsize>() };
    let ioctl_result = unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, (&raw mut winsize).cast::<u8>()) };

    if ioctl_result == 0 {
        Ok((winsize.col as u16, winsize.row as u16))
    } else {
        Err(io::Error::last_os_error())
    }
}

/// Enables raw mode
///
/// Disables input echoing, line feeding, etc.
///
/// # Errors
///
/// If there is no stdout,
/// if stdout isn't a TTY, or
/// if it fails to get or set terminal settings
pub fn enable_raw_mode() -> io::Result<()> {
    let mut termios = unsafe { std::mem::zeroed::<Termios>() };
    get_attributes(STDIN_FILENO, &mut termios)?;
    termios.lflag &= !(ECHO | ISIG | ICANON);
    set_attributes(STDIN_FILENO, &mut termios)?;
    Ok(())
}

/// Disables raw mode
///
/// Enables input echoing, line feeding, etc.
///
/// # Errors
///
/// If there is no stdout,
/// if stdout isn't a TTY, or
/// if it fails to get or set terminal settings
pub fn disable_raw_mode() -> io::Result<()> {
    let mut termios = unsafe { std::mem::zeroed::<Termios>() };
    get_attributes(STDIN_FILENO, &mut termios)?;
    termios.lflag |= ECHO | ISIG | ICANON;
    set_attributes(STDIN_FILENO, &mut termios)?;
    Ok(())
}

fn get_attributes(fd: c_int, termios: &mut Termios) -> io::Result<()> {
    if unsafe { tcgetattr(fd, &raw mut *termios) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn set_attributes(fd: c_int, termios: &mut Termios) -> io::Result<()> {
    if unsafe { tcsetattr(fd, 0, std::ptr::from_mut(termios)) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
