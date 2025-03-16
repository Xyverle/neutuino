//! Various input functions, structs, etc.
//!
//! Very incomplete currently

use std::io::{self, Read};
use std::sync::mpsc;
use std::thread;

/// An asynchronous reader.
///
/// This acts as any other stream, with the exception that reading from it won't block. Instead,
/// the buffer will only be partially updated based on how much the internal buffer holds.
///
/// Taken from the Termion crate
pub struct AsyncReader {
    recv: mpsc::Receiver<io::Result<u8>>,
}

impl Read for AsyncReader {
    /// Read from the byte stream.
    ///
    /// This will never block, but try to drain the event queue until empty. If the total number of
    /// bytes written is lower than the buffer's length, the event queue is empty or that the event
    /// stream halted.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut total = 0;
        loop {
            if total >= buf.len() {
                break;
            }

            match self.recv.try_recv() {
                Ok(Ok(b)) => {
                    buf[total] = b;
                    total += 1;
                }
                Ok(Err(e)) => return Err(e),
                Err(_) => break,
            }
        }

        Ok(total)
    }
}

impl AsyncReader {
    pub fn new<R: Read + Send + 'static>(reader: R) -> Self {
        let (send, recv) = mpsc::channel();

        thread::spawn(move || {
            for i in reader.bytes() {
                if send.send(i).is_err() {
                    return;
                }
            }
        });

        Self { recv }
    }
}
