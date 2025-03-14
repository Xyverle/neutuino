use std::mem;
use std::os::raw::c_int;

#[derive(Debug)]
pub enum Key {
    String(String),
    Char(char),
    Enter,
    Escape,
    Backspace,
    Tab,
    ShiftTab,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Ctrl(char),
}

#[cfg(unix)]
pub mod platform {
    use super::*;
    use std::fs::File;
    use std::io::Read;
    use std::os::fd::AsRawFd;

    unsafe extern "C" {
        fn tcgetattr(fd: c_int, termios_p: *mut Termios) -> c_int;
        fn tcsetattr(fd: c_int, optional_actions: c_int, termios_p: *const Termios) -> c_int;
        fn fcntl(fd: c_int, cmd: c_int, arg: c_int) -> c_int;
    }

    const TCSAFLUSH: c_int = 2;
    const ICANON: c_int = 0o0002;
    const ECHO: c_int = 0o0010;
    const O_NONBLOCK: c_int = 0x800;

    #[repr(C)]
    #[derive(Clone)]
    struct Termios {
        c_iflag: u32,
        c_oflag: u32,
        c_cflag: u32,
        c_lflag: u32,
        c_cc: [u8; 32],
    }

    pub struct RawInput {
        stdin: File,
        original_termios: Termios,
    }

    impl RawInput {
        pub fn new() -> Self {
            let stdin = File::open("/dev/tty").unwrap();
            let fd = stdin.as_raw_fd();
            let mut termios: Termios = unsafe { mem::zeroed() };

            unsafe {
                tcgetattr(fd, &mut termios);
                let mut new_termios = termios.clone();
                new_termios.c_lflag &= !(ICANON | ECHO) as u32;
                tcsetattr(fd, TCSAFLUSH, &new_termios);
                fcntl(fd, 4, O_NONBLOCK);
            }

            Self {
                stdin,
                original_termios: termios,
            }
        }

        pub fn read_key(&mut self) -> [u8; 6] {
            let mut buf = [0; 6];
            _ = self.stdin.read(&mut buf);
            buf
        }
    }

    impl Drop for RawInput {
        fn drop(&mut self) {
            unsafe { tcsetattr(self.stdin.as_raw_fd(), TCSAFLUSH, &self.original_termios) };
        }
    }
}

#[cfg(windows)]
pub mod platform {
    use super::*;
    use std::ptr;

    extern "system" {
        fn GetStdHandle(nStdHandle: c_int) -> isize;
        fn SetConsoleMode(hConsoleHandle: isize, dwMode: u32) -> i32;
        fn GetConsoleMode(hConsoleHandle: isize, lpMode: *mut u32) -> i32;
        fn ReadConsoleInputW(hConsoleInput: isize, lpBuffer: *mut InputRecord, nLength: u32, lpNumberOfEventsRead: *mut u32) -> i32;
        fn PeekConsoleInputW(hConsoleInput: isize, lpBuffer: *mut InputRecord, nLength: u32, lpNumberOfEventsRead: *mut u32) -> i32;
    }

    const STD_INPUT_HANDLE: c_int = -10;
    const ENABLE_PROCESSED_INPUT: u32 = 0x0001;
    const ENABLE_LINE_INPUT: u32 = 0x0002;
    const ENABLE_ECHO_INPUT: u32 = 0x0004;

    #[repr(C)]
    struct InputRecord {
        event_type: c_ushort,
        event: KeyEventRecord,
    }

    #[repr(C)]
    struct KeyEventRecord {
        key_down: i32,
        repeat_count: c_ushort,
        virtual_key_code: c_ushort,
        virtual_scan_code: c_ushort,
        unicode_char: u16,
        control_key_state: u32,
    }

    pub struct RawInput {
        handle: isize,
        original_mode: u32,
    }

    impl RawInput {
        pub fn new() -> Self {
            unsafe {
                let handle = GetStdHandle(STD_INPUT_HANDLE);
                let mut mode = 0;
                GetConsoleMode(handle, &mut mode);
                SetConsoleMode(handle, mode & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT));

                Self {
                    handle,
                    original_mode: mode,
                }
            }
        }

        pub fn read_key(&mut self) -> Option<Key> {
            let mut records: [InputRecord; 1] = unsafe { mem::zeroed() };
            let mut num_read = 0;

            unsafe {
                PeekConsoleInputW(self.handle, records.as_mut_ptr(), 1, &mut num_read);
                if num_read == 0 {
                    return None;
                }

                ReadConsoleInputW(self.handle, records.as_mut_ptr(), 1, &mut num_read);

                if records[0].event.key_down == 0 {
                    return None;
                }

                match records[0].event.virtual_key_code {
                    0x1B => Some(Key::Escape),
                    0x0D => Some(Key::Enter),
                    0x08 => Some(Key::Backspace),
                    0x09 => Some(Key::Tab),
                    0x2F => Some(Key::ShiftTab),
                    0x2E => Some(Key::Delete),
                    0x24 => Some(Key::Home),
                    0x23 => Some(Key::End),
                    0x21 => Some(Key::PageUp),
                    0x22 => Some(Key::PageDown),
                    0x26 => Some(Key::ArrowUp),
                    0x28 => Some(Key::ArrowDown),
                    0x25 => Some(Key::ArrowLeft),
                    0x27 => Some(Key::ArrowRight),
                    c if c >= 1 && c <= 26 => Some(Key::Ctrl((b'a' + (c - 1) as u8) as char)),
                    c if records[0].event.unicode_char != 0 => Some(Key::Char(records[0].event.unicode_char as u8 as char)),
                    _ => None,
                }
            }
        }
    }

    impl Drop for RawInput {
        fn drop(&mut self) {
            unsafe { SetConsoleMode(self.handle, self.original_mode) };
        }
    }
}

