use std::ffi::{c_int, c_uint, c_ulong, c_ushort};
use std::io;

unsafe extern "C" {
    fn ioctl(fd: c_int, request: c_ulong, argp: *mut u8) -> c_int;
    safe fn cfmakeraw(termios: *mut Termios);
    fn tcgetattr(fd: c_int, termios: *mut Termios) -> c_int;
    fn tcsetattr(fd: c_int, optional_actions: c_int, termios: *mut Termios) -> c_int;
}

const STDIN_FILENO: c_int = 0x0;
const STDOUT_FILENO: c_int = 0x1;

#[cfg(target_os = "linux")]
const TIOCGWINSZ: c_ulong = 0x5413;
#[cfg(target_os = "linux")]
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
#[derive(Default, Debug, Clone)]
struct Termios {
    iflag: c_uint,
    oflag: c_uint,
    cflag: c_uint,
    lflag: c_uint,
    cc: [u8; NCCS],
}

/// This struct represents a raw terminal
///
/// This struct will automatically enable raw mode when it is created
/// and disable raw mode when it is destructed
///
/// This insures that you never exit with a terminal still in raw mode which is problematic for
/// users
pub struct RawTerminal {
    orig_termios: Termios
}

impl RawTerminal {
    /// This constructs a terminal, automatically making it raw
    ///
    /// # Errors
    ///
    /// If there is no stdin,
    /// stdin is not a tty,
    /// if it fails to change terminal settings 
    pub fn new() -> io::Result<Self> {
        let mut orig_termios = Termios::default();
        get_attributes(STDIN_FILENO, &mut orig_termios)?;
        let mut termios = orig_termios.clone();
        cfmakeraw(&raw mut termios);
        set_attributes(STDIN_FILENO, &mut termios)?;
        Ok(Self { orig_termios })
    }
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        let mut termios = self.orig_termios.clone();
        set_attributes(STDIN_FILENO, &mut termios).expect("Failed to disable terminal raw mode");
    }
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
    let mut winsize = Winsize::default();
    let ioctl_result = unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, (&raw mut winsize).cast::<u8>()) };

    if ioctl_result == 0 {
        Ok((winsize.col, winsize.row))
    } else {
        Err(io::Error::last_os_error())
    }
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
