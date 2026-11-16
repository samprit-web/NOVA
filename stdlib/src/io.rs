//! I/O traits and implementations

use crate::prelude::*;
use core::fmt;
use core::pin::Pin;
use core::task::{Context, Poll};

// Read trait
pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let mut offset = 0;
        while offset < buf.len() {
            match self.read(&mut buf[offset..]) {
                Ok(0) => return Err(Error::new(ErrorKind::UnexpectedEof, "failed to fill whole buffer")),
                Ok(n) => offset += n,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        let mut total = 0;
        let mut buf = Vec::with_capacity(1024);
        loop {
            let len = buf.len();
            buf.reserve(1024);
            unsafe { buf.set_len(len + 1024); }
            match self.read(&mut buf[len..]) {
                Ok(0) => {
                    unsafe { buf.set_len(len); }
                    total += len;
                    break;
                }
                Ok(n) => {
                    unsafe { buf.set_len(len + n); }
                }
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        *buf = buf;
        Ok(total)
    }

    fn read_to_string(&mut self, buf: &mut String) -> Result<usize> {
        let mut bytes = Vec::new();
        let n = self.read_to_end(&mut bytes)?;
        *buf = String::from_utf8_lossy(&bytes).to_string();
        Ok(n)
    }

    fn bytes(&self) -> Bytes<&Self> where Self: Sized { Bytes { inner: self } }
    fn chars(&self) -> Chars<&Self> where Self: Sized { Chars { inner: self } }
}

pub struct Bytes<'a, R: Read + ?Sized> { inner: &'a R }
impl<'a, R: Read + ?Sized> Iterator for Bytes<'a, R> {
    type Item = Result<u8>;
    fn next(&mut self) -> Option<Result<u8>> {
        let mut buf = [0];
        match self.inner.read(&mut buf) {
            Ok(0) => None,
            Ok(_) => Some(Ok(buf[0])),
            Err(e) => Some(Err(e)),
        }
    }
}

pub struct Chars<'a, R: Read + ?Sized> { inner: &'a R }
impl<'a, R: Read + ?Sized> Iterator for Chars<'a, R> {
    type Item = Result<char>;
    fn next(&mut self) -> Option<Result<char>> { None }
}

// Write trait
pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        let mut offset = 0;
        while offset < buf.len() {
            match self.write(&buf[offset..]) {
                Ok(0) => return Err(Error::new(ErrorKind::WriteZero, "failed to write whole buffer")),
                Ok(n) => offset += n,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> Result<()> {
        struct Adapter<'a, W: Write + ?Sized> { inner: &'a mut W }
        impl<W: Write + ?Sized> fmt::Write for Adapter<'_, W> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                self.inner.write_all(s.as_bytes()).map_err(|_| fmt::Error)
            }
        }
        fmt::write(&mut Adapter { inner: self }, fmt)
    }

    fn by_ref(&mut self) -> &mut Self where Self: Sized { self }
}

// Seek trait
pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>;
    fn stream_position(&mut self) -> Result<u64> { self.seek(SeekFrom::Current(0)) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekFrom {
    Start(u64),
    End(i64),
    Current(i64),
}

// BufRead trait
pub trait BufRead: Read {
    fn fill_buf(&mut self) -> Result<&[u8]>;
    fn consume(&mut self, amt: usize);

    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> Result<usize> {
        let mut total = 0;
        loop {
            let buf_ = self.fill_buf()?;
            if let Some(i) = buf_.iter().position(|&b| b == byte) {
                buf.extend_from_slice(&buf_[..=i]);
                self.consume(i + 1);
                return Ok(total + i + 1);
            }
            if buf_.is_empty() { return Ok(total); }
            buf.extend_from_slice(buf_);
            let len = buf_.len();
            self.consume(len);
            total += len;
        }
    }

    fn read_line(&mut self, buf: &mut String) -> Result<usize> {
        let mut bytes = Vec::new();
        let n = self.read_until(b'\n', &mut bytes)?;
        *buf = String::from_utf8_lossy(&bytes).to_string();
        Ok(n)
    }

    fn lines(&self) -> Lines<&Self> where Self: Sized { Lines { inner: self } }
    fn split(&self, byte: u8) -> Split<&Self> where Self: Sized { Split { inner: self, byte } }
}

pub struct Lines<'a, R: BufRead + ?Sized> { inner: &'a R }
impl<'a, R: BufRead + ?Sized> Iterator for Lines<'a, R> {
    type Item = Result<String>;
    fn next(&mut self) -> Option<Result<String>> {
        let mut buf = String::new();
        match self.inner.read_line(&mut buf) {
            Ok(0) if buf.is_empty() => None,
            Ok(_) => Some(Ok(buf)),
            Err(e) => Some(Err(e)),
        }
    }
}

pub struct Split<'a, R: BufRead + ?Sized> { inner: &'a R, byte: u8 }
impl<'a, R: BufRead + ?Sized> Iterator for Split<'a, R> {
    type Item = Result<Vec<u8>>;
    fn next(&mut self) -> Option<Result<Vec<u8>>> {
        let mut buf = Vec::new();
        match self.inner.read_until(self.byte, &mut buf) {
            Ok(0) if buf.is_empty() => None,
            Ok(_) => Some(Ok(buf)),
            Err(e) => Some(Err(e)),
        }
    }
}

// Error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    ConnectionAborted,
    NotConnected,
    AddrInUse,
    AddrNotAvailable,
    BrokenPipe,
    AlreadyExists,
    WouldBlock,
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    Interrupted,
    UnexpectedEof,
    OutOfMemory,
    Other,
}

pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self { kind, message: message.into() }
    }

    pub fn kind(&self) -> ErrorKind { self.kind }
    pub fn message(&self) -> &str { &self.message }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.kind)
            .field("message", &self.message)
            .finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;

// Stdin/Stdout/Stderr
pub fn stdin() -> Stdin { Stdin }
pub fn stdout() -> Stdout { Stdout }
pub fn stderr() -> Stderr { Stderr }

pub struct Stdin;
impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        use std::io::Read;
        std::io::stdin().read(buf).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
    }
}

pub struct Stdout;
impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        use std::io::Write;
        std::io::stdout().write(buf).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
    }
    fn flush(&mut self) -> Result<()> {
        use std::io::Write;
        std::io::stdout().flush().map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
    }
}

pub struct Stderr;
impl Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        use std::io::Write;
        std::io::stderr().write(buf).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
    }
    fn flush(&mut self) -> Result<()> {
        use std::io::Write;
        std::io::stderr().flush().map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
    }
}

// BufReader
pub struct BufReader<R: Read> {
    inner: R,
    buf: Vec<u8>,
    pos: usize,
    cap: usize,
}

impl<R: Read> BufReader<R> {
    pub fn new(inner: R) -> Self {
        Self { inner, buf: Vec::with_capacity(8192), pos: 0, cap: 0 }
    }

    pub fn with_capacity(capacity: usize, inner: R) -> Self {
        Self { inner, buf: Vec::with_capacity(capacity), pos: 0, cap: 0 }
    }

    pub fn get_ref(&self) -> &R { &self.inner }
    pub fn get_mut(&mut self) -> &mut R { &mut self.inner }
    pub fn into_inner(self) -> R { self.inner }
    pub fn buffer(&self) -> &[u8] { &self.buf[self.pos..self.cap] }
}

impl<R: Read> Read for BufReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.pos >= self.cap {
            self.cap = self.inner.read(&mut self.buf)?;
            self.pos = 0;
            if self.cap == 0 { return Ok(0); }
        }
        let amt = core::cmp::min(buf.len(), self.cap - self.pos);
        buf[..amt].copy_from_slice(&self.buf[self.pos..self.pos + amt]);
        self.pos += amt;
        Ok(amt)
    }
}

impl<R: Read> BufRead for BufReader<R> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        if self.pos >= self.cap {
            self.cap = self.inner.read(&mut self.buf)?;
            self.pos = 0;
        }
        Ok(&self.buf[self.pos..self.cap])
    }
    fn consume(&mut self, amt: usize) {
        self.pos = core::cmp::min(self.pos + amt, self.cap);
    }
}

// BufWriter
pub struct BufWriter<W: Write> {
    inner: W,
    buf: Vec<u8>,
    pos: usize,
}

impl<W: Write> BufWriter<W> {
    pub fn new(inner: W) -> Self {
        Self { inner, buf: Vec::with_capacity(8192), pos: 0 }
    }

    pub fn with_capacity(capacity: usize, inner: W) -> Self {
        Self { inner, buf: Vec::with_capacity(capacity), pos: 0 }
    }

    pub fn get_ref(&self) -> &W { &self.inner }
    pub fn get_mut(&mut self) -> &mut W { &mut self.inner }
    pub fn into_inner(self) -> Result<W> {
        let mut this = self;
        this.flush()?;
        Ok(this.inner)
    }
    pub fn buffer(&self) -> &[u8] { &self.buf[..this.pos] }
}

impl<W: Write> Write for BufWriter<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if self.pos + buf.len() > self.buf.len() {
            self.flush()?;
        }
        if buf.len() >= self.buf.len() {
            return self.inner.write(buf);
        }
        self.buf[self.pos..self.pos + buf.len()].copy_from_slice(buf);
        self.pos += buf.len();
        Ok(buf.len())
    }
    fn flush(&mut self) -> Result<()> {
        if self.pos > 0 {
            self.inner.write_all(&self.buf[..self.pos])?;
            self.pos = 0;
        }
        self.inner.flush()
    }
}

// Cursor
pub struct Cursor<T> {
    inner: T,
    pos: usize,
}

impl<T: AsRef<[u8]>> Cursor<T> {
    pub fn new(inner: T) -> Self { Self { inner, pos: 0 } }
    pub fn position(&self) -> usize { self.pos }
    pub fn set_position(&mut self, pos: usize) { self.pos = pos; }
    pub fn get_ref(&self) -> &T { &self.inner }
    pub fn get_mut(&mut self) -> &mut T { &mut self.inner }
    pub fn into_inner(self) -> T { self.inner }
}

impl<T: AsRef<[u8]>> Read for Cursor<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let remaining = self.inner.as_ref().len().saturating_sub(self.pos);
        let amt = core::cmp::min(buf.len(), remaining);
        buf[..amt].copy_from_slice(&self.inner.as_ref()[self.pos..self.pos + amt]);
        self.pos += amt;
        Ok(amt)
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> Write for Cursor<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let remaining = self.inner.as_mut().len().saturating_sub(self.pos);
        let amt = core::cmp::min(buf.len(), remaining);
        self.inner.as_mut()[self.pos..self.pos + amt].copy_from_slice(&buf[..amt]);
        self.pos += amt;
        Ok(amt)
    }
    fn flush(&mut self) -> Result<()> { Ok(()) }
}

impl<T: AsRef<[u8]>> Seek for Cursor<T> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let len = self.inner.as_ref().len() as u64;
        let new_pos = match pos {
            SeekFrom::Start(p) => p,
            SeekFrom::End(p) => len.saturating_add_signed(p),
            SeekFrom::Current(p) => self.pos as u64 + p as u64,
        };
        if new_pos > len { return Err(Error::new(ErrorKind::InvalidInput, "seek out of bounds")); }
        self.pos = new_pos as usize;
        Ok(new_pos)
    }
}

// Print macros
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use $crate::io::Write;
        let _ = $crate::io::stdout().write_fmt(format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => {{
        $crate::print!("{}\n", format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => {{
        use $crate::io::Write;
        let _ = $crate::io::stderr().write_fmt(format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! eprintln {
    () => { $crate::eprint!("\n") };
    ($($arg:tt)*) => {{
        $crate::eprint!("{}\n", format_args!($($arg)*));
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buf_reader() {
        let data = b"Hello, World!";
        let mut reader = BufReader::new(&data[..]);
        let mut buf = [0; 5];
        assert_eq!(reader.read(&mut buf).unwrap(), 5);
        assert_eq!(&buf, b"Hello");
    }

    #[test]
    fn test_buf_writer() {
        let mut buf = Vec::new();
        let mut writer = BufWriter::new(&mut buf);
        writer.write_all(b"Hello").unwrap();
        writer.flush().unwrap();
        assert_eq!(buf, b"Hello");
    }

    #[test]
    fn test_cursor() {
        let mut cursor = Cursor::new(vec![1, 2, 3, 4, 5]);
        let mut buf = [0; 3];
        assert_eq!(cursor.read(&mut buf).unwrap(), 3);
        assert_eq!(buf, [1, 2, 3]);
        cursor.set_position(3);
        assert_eq!(cursor.read(&mut buf).unwrap(), 2);
        assert_eq!(buf, [4, 5]);
    }
}