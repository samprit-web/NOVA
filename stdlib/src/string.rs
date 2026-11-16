//! String types

use crate::prelude::*;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::{Deref, DerefMut, Index, IndexMut, RangeBounds};
use core::str::{from_utf8, from_utf8_unchecked, Utf8Error};
use core::slice::SliceIndex;

// String
pub struct String {
    vec: Vec<u8>,
}

impl String {
    pub const fn new() -> Self {
        Self { vec: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { vec: Vec::with_capacity(capacity) }
    }

    pub fn from_utf8(vec: Vec<u8>) -> Result<Self, FromUtf8Error> {
        match from_utf8(&vec) {
            Ok(_) => Ok(Self { vec }),
            Err(e) => Err(FromUtf8Error { bytes: vec, error: e }),
        }
    }

    pub fn from_utf8_lossy(bytes: &[u8]) -> Self {
        String { vec: bytes.to_vec() }
    }

    pub fn as_str(&self) -> &str {
        unsafe { from_utf8_unchecked(&self.vec) }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.vec
    }

    pub fn push_str(&mut self, s: &str) {
        self.vec.extend_from_slice(s.as_bytes());
    }

    pub fn push(&mut self, ch: char) {
        let mut buf = [0; 4];
        self.vec.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
    }

    pub fn pop(&mut self) -> Option<char> {
        if self.vec.is_empty() { return None; }
        let ch = self.as_str().chars().last().unwrap();
        let len = ch.len_utf8();
        self.vec.truncate(self.vec.len() - len);
        Some(ch)
    }

    pub fn len(&self) -> usize { self.vec.len() }
    pub fn is_empty(&self) -> bool { self.vec.is_empty() }
    pub fn capacity(&self) -> usize { self.vec.capacity() }

    pub fn clear(&mut self) { self.vec.clear(); }
    pub fn truncate(&mut self, len: usize) { self.vec.truncate(len); }
    pub fn reserve(&mut self, additional: usize) { self.vec.reserve(additional); }
    pub fn shrink_to_fit(&mut self) { self.vec.shrink_to_fit(); }

    pub fn split_off(&mut self, at: usize) -> String {
        String { vec: self.vec.split_off(at) }
    }

    pub fn insert(&mut self, idx: usize, ch: char) {
        let mut buf = [0; 4];
        let encoded = ch.encode_utf8(&mut buf);
        self.vec.insert(idx, 0); // Placeholder
        self.vec.splice(idx..idx + 1, encoded.bytes());
    }

    pub fn remove(&mut self, idx: usize) -> char {
        let ch = self.as_str().chars().nth(idx).unwrap();
        let len = ch.len_utf8();
        self.vec.drain(idx..idx + len);
        ch
    }
}

impl Deref for String {
    type Target = str;
    fn deref(&self) -> &str { self.as_str() }
}

impl DerefMut for String {
    fn deref_mut(&mut self) -> &mut str {
        unsafe { core::str::from_utf8_unchecked_mut(&mut self.vec) }
    }
}

impl fmt::Debug for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl PartialEq for String {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for String {}

impl PartialOrd for String {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.as_str().cmp(other.as_str()))
    }
}

impl Ord for String {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl Hash for String {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl Default for String {
    fn default() -> Self { Self::new() }
}

impl Clone for String {
    fn clone(&self) -> Self {
        Self { vec: self.vec.clone() }
    }
}

impl From<&str> for String {
    fn from(s: &str) -> Self {
        Self { vec: s.as_bytes().to_vec() }
    }
}

impl From<String> for Vec<u8> {
    fn from(s: String) -> Self { s.vec }
}

impl Extend<char> for String {
    fn extend<I: IntoIterator<Item = char>>(&mut self, iter: I) {
        for ch in iter { self.push(ch); }
    }
}

impl Extend<&str> for String {
    fn extend<I: IntoIterator<Item = &str>>(&mut self, iter: I) {
        for s in iter { self.push_str(s); }
    }
}

impl<'a> Extend<&'a str> for String {
    fn extend<I: IntoIterator<Item = &'a str>>(&mut self, iter: I) {
        for s in iter { self.push_str(s); }
    }
}

// FromUtf8Error
pub struct FromUtf8Error {
    bytes: Vec<u8>,
    error: Utf8Error,
}

impl FromUtf8Error {
    pub fn bytes(&self) -> &[u8] { &self.bytes }
    pub fn into_bytes(self) -> Vec<u8> { self.bytes }
    pub fn utf8_error(&self) -> Utf8Error { self.error }
}

impl fmt::Debug for FromUtf8Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FromUtf8Error")
            .field("bytes", &self.bytes.len())
            .field("error", &self.error)
            .finish()
    }
}

impl fmt::Display for FromUtf8Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl Error for FromUtf8Error {}

// ToString trait
pub trait ToString {
    fn to_string(&self) -> String;
}

impl<T: fmt::Display + ?Sized> ToString for T {
    fn to_string(&self) -> String {
        use core::fmt::Write;
        let mut buf = String::new();
        write!(&mut buf, "{}", self).unwrap();
        buf
    }
}

// String methods (from str)
impl str {
    pub fn len(&self) -> usize { self.as_bytes().len() }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
    pub fn as_bytes(&self) -> &[u8] { self.as_bytes() }
    pub fn as_ptr(&self) -> *const u8 { self.as_bytes().as_ptr() }

    pub fn chars(&self) -> Chars { Chars { iter: self.as_bytes().iter() } }
    pub fn bytes(&self) -> Bytes { Bytes { iter: self.as_bytes().iter() } }

    pub fn split(&self, pat: &str) -> Split<&str> { Split { haystack: self, pat, index: 0 } }
    pub fn split_whitespace(&self) -> SplitWhitespace { SplitWhitespace { haystack: self, index: 0 } }
    pub fn lines(&self) -> Lines { Lines { haystack: self, index: 0 } }

    pub fn trim(&self) -> &str {
        let bytes = self.as_bytes();
        let start = bytes.iter().position(|&b| !b.is_ascii_whitespace()).unwrap_or(bytes.len());
        let end = bytes.iter().rposition(|&b| !b.is_ascii_whitespace()).map(|i| i + 1).unwrap_or(0);
        if start >= end { "" } else { unsafe { from_utf8_unchecked(&bytes[start..end]) } }
    }

    pub fn trim_start(&self) -> &str {
        let start = self.as_bytes().iter().position(|&b| !b.is_ascii_whitespace()).unwrap_or(self.len());
        unsafe { from_utf8_unchecked(&self.as_bytes()[start..]) }
    }

    pub fn trim_end(&self) -> &str {
        let end = self.as_bytes().iter().rposition(|&b| !b.is_ascii_whitespace()).map(|i| i + 1).unwrap_or(0);
        unsafe { from_utf8_unchecked(&self.as_bytes()[..end]) }
    }

    pub fn to_lowercase(&self) -> String { String::from(self.to_lowercase()) }
    pub fn to_uppercase(&self) -> String { String::from(self.to_uppercase()) }

    pub fn contains(&self, pat: &str) -> bool { self.find(pat).is_some() }
    pub fn starts_with(&self, pat: &str) -> bool { self.as_bytes().starts_with(pat.as_bytes()) }
    pub fn ends_with(&self, pat: &str) -> bool { self.as_bytes().ends_with(pat.as_bytes()) }

    pub fn find(&self, pat: &str) -> Option<usize> {
        self.as_bytes().windows(pat.len()).position(|w| w == pat.as_bytes())
    }

    pub fn rfind(&self, pat: &str) -> Option<usize> {
        let bytes = self.as_bytes();
        bytes.windows(pat.len()).rposition(|w| w == pat.as_bytes())
    }

    pub fn replace(&self, from: &str, to: &str) -> String {
        let mut result = String::with_capacity(self.len());
        let mut last = 0;
        while let Some(pos) = self[last..].find(from) {
            let pos = last + pos;
            result.push_str(&self[last..pos]);
            result.push_str(to);
            last = pos + from.len();
        }
        result.push_str(&self[last..]);
        result
    }
}

// Iterator types
pub struct Chars<'a> { iter: core::slice::Iter<'a, u8> }
impl<'a> Iterator for Chars<'a> { type Item = char; fn next(&mut self) -> Option<char> { None } }

pub struct Bytes<'a> { iter: core::slice::Iter<'a, u8> }
impl<'a> Iterator for Bytes<'a> { type Item = u8; fn next(&mut self) -> Option<u8> { self.iter.next().copied() } }

pub struct Split<'a, P> { haystack: &'a str, pat: P, index: usize }
impl<'a, P: AsRef<str>> Iterator for Split<'a, P> { type Item = &'a str; fn next(&mut self) -> Option<&'a str> { None } }

pub struct SplitWhitespace<'a> { haystack: &'a str, index: usize }
impl<'a> Iterator for SplitWhitespace<'a> { type Item = &'a str; fn next(&mut self) -> Option<&'a str> { None } }

pub struct Lines<'a> { haystack: &'a str, index: usize }
impl<'a> Iterator for Lines<'a> { type Item = &'a str; fn next(&mut self) -> Option<&'a str> { None } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string() {
        let mut s = String::new();
        s.push_str("Hello");
        s.push(' ');
        s.push_str("World");
        assert_eq!(s.as_str(), "Hello World");
    }

    #[test]
    fn test_string_from() {
        let s = String::from("test");
        assert_eq!(s.as_str(), "test");
    }

    #[test]
    fn test_to_string() {
        let s = 42.to_string();
        assert_eq!(s, "42");
    }
}