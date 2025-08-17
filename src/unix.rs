use std::ffi::{c_int, c_short, c_uint, c_ulong, c_ushort};
use std::io;
use std::sync::LazyLock;

const ENABLE_MOUSE: &str = "\x1b[?1000h\x1b[?1002h\x1b[?1015h\x1b[?1006h\x1b[?1003h";
const DISABLE_MOUSE: &str = "\x1b[?1006l\x1b[?1015l\x1b[?1002l\x1b[?1000l\x1b[?1003l";

unsafe extern "C" {
    fn ioctl(fd: c_int, request: c_ulong, argp: *mut u8) -> c_int;
    fn cfmakeraw(termios: *mut Termios);
    fn tcgetattr(fd: c_int, termios: *mut Termios) -> c_int;
    fn tcsetattr(fd: c_int, optional_actions: c_int, termios: *const Termios) -> c_int;
}

pub(crate) const STDIN_FILENO: c_int = 0;
pub(crate) const STDOUT_FILENO: c_int = 1;
pub(crate) const POLLIN: c_short = 1;

#[cfg(not(target_os = "macos"))]
const TIOCGWINSZ: c_ulong = 0x5413;
#[cfg(not(target_os = "macos"))]
const NCCS: usize = 0x20;

#[cfg(target_os = "macos")]
const TIOCGWINSZ: c_ulong = 0x4008_7468;
#[cfg(target_os = "macos")]
const NCCS: usize = 0x14;

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
struct Winsize {
    row: c_ushort,
    col: c_ushort,
    xpixel: c_ushort,
    ypixel: c_ushort,
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
struct Termios {
    iflag: c_uint,
    oflag: c_uint,
    cflag: c_uint,
    lflag: c_uint,
    cc: [u8; NCCS],
    #[cfg(target_os = "linux")]
    ispeed: c_ulong,
    #[cfg(target_os = "linux")]
    ospeed: c_ulong,
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

static TERMIOS: LazyLock<Result<Termios, i32>> = LazyLock::new(|| {
    let mut orig_termios = unsafe { std::mem::zeroed() };
    let attributes = get_attributes(STDIN_FILENO, &mut orig_termios);
    match attributes {
        Ok(()) => Ok(orig_termios),
        Err(e) => Err(e.raw_os_error().unwrap()),
    }
});

/// Enable mouse input, if available
///
/// # Errors
///
/// Never currently
pub fn enable_mouse_input() -> io::Result<()> {
    print!("{ENABLE_MOUSE}");
    Ok(())
}

/// Disable mouse input, if available
///
/// # Errors
///
/// Never currently
pub fn disable_mouse_input() -> io::Result<()> {
    print!("{DISABLE_MOUSE}");
    Ok(())
}

/// Enables raw mode, which disables line buffering, input echoing, and output canonicalization
///
/// # Errors
///
/// If there is no stdin,
/// stdin is not a tty,
/// or it fails to change terminal settings
pub fn enable_raw_mode() -> io::Result<()> {
    let mut termios = (*TERMIOS).map_err(io::Error::from_raw_os_error)?;
    unsafe {
        cfmakeraw(&raw mut termios);
    }
    set_attributes(STDIN_FILENO, &mut termios)?;
    Ok(())
}

/// Disables raw mode, which enables line buffering, input echoing, and output canonicalization
///
/// # Errors
///
/// If there is no stdin,
/// stdin is not a tty,
/// or it fails to change terminal settings
pub fn disable_raw_mode() -> io::Result<()> {
    let mut termios = (*TERMIOS).map_err(io::Error::from_raw_os_error)?;
    set_attributes(STDIN_FILENO, &mut termios)?;
    Ok(())
}

/// Enables ANSI support on Windows terminals
///
/// ANSI is always enabled on *nix machines but these function still exist for simpler usage
///
/// # Errors
///
/// Never on *nix
///
/// On Windows, if There is no stdout,
/// if stdout isn't a TTY, or
/// if it cannot change terminal properties
pub fn enable_ansi() -> io::Result<()> {
    // ANSI is on by default on unix platforms
    // This is here for compatibility with the windows version of this API
    Ok(())
}

/// Disables ANSI support on Windows terminals
///
/// ANSI is always enabled on *nix machines but these function still exist for simpler usage
///
/// # Errors
///
/// Never on *nix
///
/// On Windows, if There is no stdout,
/// if stdout isn't a TTY, or
/// if it cannot change terminal properties
pub fn disable_ansi() -> io::Result<()> {
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
    let mut winsize = Winsize::default();
    let ioctl_result = unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, (&raw mut winsize).cast::<u8>()) };

    if ioctl_result == 0 {
        Ok((winsize.col, winsize.row))
    } else {
        Err(io::Error::last_os_error())
    }
}
