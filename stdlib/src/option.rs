//! Optional values

use crate::prelude::*;
use core::fmt;
use core::ops::{Deref, DerefMut};

/// Option type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Option<T> {
    None,
    Some(T),
}

impl<T> Option<T> {
    pub const fn none() -> Self { Option::None }
    pub const fn some(value: T) -> Self { Option::Some(value) }

    pub fn is_some(&self) -> bool { matches!(self, Option::Some(_)) }
    pub fn is_none(&self) -> bool { matches!(self, Option::None) }

    pub fn as_ref(&self) -> Option<&T> {
        match self {
            Option::Some(v) => Option::Some(v),
            Option::None => Option::None,
        }
    }

    pub fn as_mut(&mut self) -> Option<&mut T> {
        match self {
            Option::Some(v) => Option::Some(v),
            Option::None => Option::None,
        }
    }

    pub fn unwrap(self) -> T {
        match self {
            Option::Some(v) => v,
            Option::None => panic!("called `Option::unwrap()` on a `None` value"),
        }
    }

    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Option::Some(v) => v,
            Option::None => default,
        }
    }

    pub fn unwrap_or_else<F: FnOnce() -> T>(self, f: F) -> T {
        match self {
            Option::Some(v) => v,
            Option::None => f(),
        }
    }

    pub fn expect(self, msg: &str) -> T {
        match self {
            Option::Some(v) => v,
            Option::None => panic!("{}: {}", msg, "called `Option::expect()` on a `None` value"),
        }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Option<U> {
        match self {
            Option::Some(v) => Option::Some(f(v)),
            Option::None => Option::None,
        }
    }

    pub fn map_or<U, F: FnOnce(T) -> U>(self, default: U, f: F) -> U {
        match self {
            Option::Some(v) => f(v),
            Option::None => default,
        }
    }

    pub fn map_or_else<U, D: FnOnce() -> U, F: FnOnce(T) -> U>(self, default: D, f: F) -> U {
        match self {
            Option::Some(v) => f(v),
            Option::None => default(),
        }
    }

    pub fn and<U>(self, optb: Option<U>) -> Option<U> {
        match self {
            Option::Some(_) => optb,
            Option::None => Option::None,
        }
    }

    pub fn and_then<U, F: FnOnce(T) -> Option<U>>(self, f: F) -> Option<U> {
        match self {
            Option::Some(v) => f(v),
            Option::None => Option::None,
        }
    }

    pub fn or(self, optb: Option<T>) -> Option<T> {
        match self {
            Option::Some(_) => self,
            Option::None => optb,
        }
    }

    pub fn or_else<F: FnOnce() -> Option<T>>(self, f: F) -> Option<T> {
        match self {
            Option::Some(_) => self,
            Option::None => f(),
        }
    }

    pub fn filter<F: FnOnce(&T) -> bool>(self, f: F) -> Option<T> {
        match self {
            Option::Some(v) if f(&v) => Option::Some(v),
            _ => Option::None,
        }
    }

    pub fn take(&mut self) -> Option<T> {
        core::mem::replace(self, Option::None)
    }

    pub fn replace(&mut self, value: T) -> Option<T> {
        core::mem::replace(self, Option::Some(value))
    }

    pub fn contains<U>(&self, x: &U) -> bool
    where
        T: PartialEq<U>,
    {
        match self {
            Option::Some(v) => v == x,
            Option::None => false,
        }
    }

    pub fn zip<U>(self, other: Option<U>) -> Option<(T, U)> {
        match (self, other) {
            (Option::Some(a), Option::Some(b)) => Option::Some((a, b)),
            _ => Option::None,
        }
    }

    pub fn zip_with<U, R, F: FnOnce(T, U) -> R>(self, other: Option<U>, f: F) -> Option<R> {
        match (self, other) {
            (Option::Some(a), Option::Some(b)) => Option::Some(f(a, b)),
            _ => Option::None,
        }
    }
}

impl<T> Default for Option<T> {
    fn default() -> Self { Option::None }
}

impl<T> From<T> for Option<T> {
    fn from(value: T) -> Self { Option::Some(value) }
}

impl<T> IntoIterator for Option<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { opt: self }
    }
}

pub struct IntoIter<T> { opt: Option<T> }
impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> { self.opt.take() }
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.opt.is_some() { (1, Some(1)) } else { (0, Some(0)) }
    }
}

// Option extensions
pub trait OptionExt<T> {
    fn transpose<U>(self) -> Option<Result<T, U>>;
    fn ok_or<E>(self, err: E) -> Result<T, E>;
    fn ok_or_else<E, F: FnOnce() -> E>(self, err: F) -> Result<T, E>;
}

impl<T> OptionExt<T> for Option<T> {
    fn transpose<U>(self) -> Option<Result<T, U>> {
        match self {
            Option::Some(Ok(v)) => Option::Some(Ok(v)),
            Option::Some(Err(e)) => Option::Some(Err(e)),
            Option::None => Option::None,
        }
    }

    fn ok_or<E>(self, err: E) -> Result<T, E> {
        match self {
            Option::Some(v) => Ok(v),
            Option::None => Err(err),
        }
    }

    fn ok_or_else<E, F: FnOnce() -> E>(self, err: F) -> Result<T, E> {
        match self {
            Option::Some(v) => Ok(v),
            Option::None => Err(err()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option() {
        let some = Option::Some(42);
        assert!(some.is_some());
        assert_eq!(some.unwrap(), 42);

        let none: Option<i32> = Option::None;
        assert!(none.is_none());
        assert_eq!(none.unwrap_or(0), 0);
    }

    #[test]
    fn test_option_map() {
        let some = Option::Some(42);
        let doubled = some.map(|x| x * 2);
        assert_eq!(doubled, Option::Some(84));

        let none: Option<i32> = Option::None;
        let mapped = none.map(|x| x * 2);
        assert_eq!(mapped, Option::None);
    }

    #[test]
    fn test_option_and_then() {
        let some = Option::Some(42);
        let result = some.and_then(|x| if x > 0 { Option::Some(x * 2) } else { Option::None });
        assert_eq!(result, Option::Some(84));
    }
}