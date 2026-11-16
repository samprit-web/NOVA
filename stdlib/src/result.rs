//! Result type for error handling

use crate::prelude::*;
use core::fmt;
use core::ops::{Deref, DerefMut};

/// Result type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> Result<T, E> {
    pub const fn ok(value: T) -> Self { Result::Ok(value) }
    pub const fn err(error: E) -> Self { Result::Err(error) }

    pub fn is_ok(&self) -> bool { matches!(self, Result::Ok(_)) }
    pub fn is_err(&self) -> bool { matches!(self, Result::Err(_)) }

    pub fn as_ref(&self) -> Result<&T, &E> {
        match self {
            Result::Ok(v) => Result::Ok(v),
            Result::Err(e) => Result::Err(e),
        }
    }

    pub fn as_mut(&mut self) -> Result<&mut T, &mut E> {
        match self {
            Result::Ok(v) => Result::Ok(v),
            Result::Err(e) => Result::Err(e),
        }
    }

    pub fn unwrap(self) -> T {
        match self {
            Result::Ok(v) => v,
            Result::Err(e) => panic!("called `Result::unwrap()` on an `Err` value: {:?}", e),
        }
    }

    pub fn unwrap_err(self) -> E {
        match self {
            Result::Ok(v) => panic!("called `Result::unwrap_err()` on an `Ok` value: {:?}", v),
            Result::Err(e) => e,
        }
    }

    pub fn expect(self, msg: &str) -> T {
        match self {
            Result::Ok(v) => v,
            Result::Err(e) => panic!("{}: {:?}", msg, e),
        }
    }

    pub fn expect_err(self, msg: &str) -> E {
        match self {
            Result::Ok(v) => panic!("{}: {:?}", msg, v),
            Result::Err(e) => e,
        }
    }

    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Result::Ok(v) => v,
            Result::Err(_) => default,
        }
    }

    pub fn unwrap_or_else<F: FnOnce(E) -> T>(self, f: F) -> T {
        match self {
            Result::Ok(v) => v,
            Result::Err(e) => f(e),
        }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Result<U, E> {
        match self {
            Result::Ok(v) => Result::Ok(f(v)),
            Result::Err(e) => Result::Err(e),
        }
    }

    pub fn map_err<F, O: FnOnce(E) -> F>(self, f: O) -> Result<T, F> {
        match self {
            Result::Ok(v) => Result::Ok(v),
            Result::Err(e) => Result::Err(f(e)),
        }
    }

    pub fn map_or<U, F: FnOnce(T) -> U>(self, default: U, f: F) -> U {
        match self {
            Result::Ok(v) => f(v),
            Result::Err(_) => default,
        }
    }

    pub fn map_or_else<U, D: FnOnce(E) -> U, F: FnOnce(T) -> U>(self, default: D, f: F) -> U {
        match self {
            Result::Ok(v) => f(v),
            Result::Err(e) => default(e),
        }
    }

    pub fn and<U>(self, res: Result<U, E>) -> Result<U, E> {
        match self {
            Result::Ok(_) => res,
            Result::Err(e) => Result::Err(e),
        }
    }

    pub fn and_then<U, F: FnOnce(T) -> Result<U, E>>(self, f: F) -> Result<U, E> {
        match self {
            Result::Ok(v) => f(v),
            Result::Err(e) => Result::Err(e),
        }
    }

    pub fn or<F>(self, res: Result<T, F>) -> Result<T, F> {
        match self {
            Result::Ok(v) => Result::Ok(v),
            Result::Err(_) => res,
        }
    }

    pub fn or_else<F, O: FnOnce(E) -> Result<T, F>>(self, f: O) -> Result<T, F> {
        match self {
            Result::Ok(v) => Result::Ok(v),
            Result::Err(e) => f(e),
        }
    }

    pub fn transpose<U, F>(self) -> Option<Result<T, F>>
    where
        E: Into<F>,
    {
        match self {
            Result::Ok(v) => Some(Result::Ok(v)),
            Result::Err(e) => Some(Result::Err(e.into())),
        }
    }

    pub fn flatten<U>(self) -> Result<U, E>
    where
        T: Into<Result<U, E>>,
    {
        match self {
            Result::Ok(v) => v.into(),
            Result::Err(e) => Result::Err(e),
        }
    }

    pub fn contains<U>(&self, x: &U) -> bool
    where
        T: PartialEq<U>,
    {
        match self {
            Result::Ok(v) => v == x,
            Result::Err(_) => false,
        }
    }

    pub fn contains_err<F>(&self, x: &F) -> bool
    where
        E: PartialEq<F>,
    {
        match self {
            Result::Ok(_) => false,
            Result::Err(e) => e == x,
        }
    }
}

impl<T, E> Default for Result<T, E>
where
    T: Default,
{
    fn default() -> Self { Result::Ok(T::default()) }
}

impl<T, E> From<T> for Result<T, E> {
    fn from(value: T) -> Self { Result::Ok(value) }
}

impl<T, E> From<Option<T>> for Result<T, E>
where
    E: Default,
{
    fn from(opt: Option<T>) -> Self {
        match opt {
            Option::Some(v) => Result::Ok(v),
            Option::None => Result::Err(E::default()),
        }
    }
}

impl<T, E> IntoIterator for Result<T, E> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { res: self }
    }
}

pub struct IntoIter<T> { res: Result<T, !> }
impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        match core::mem::replace(&mut self.res, Result::Err(core::panic!())) {
            Result::Ok(v) => Some(v),
            Result::Err(_) => None,
        }
    }
}

// Result extensions
pub trait ResultExt<T, E> {
    fn ok(self) -> Option<T>;
    fn err(self) -> Option<E>;
    fn transpose<U>(self) -> Option<Result<T, U>>;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn ok(self) -> Option<T> {
        match self {
            Result::Ok(v) => Option::Some(v),
            Result::Err(_) => Option::None,
        }
    }

    fn err(self) -> Option<E> {
        match self {
            Result::Ok(_) => Option::None,
            Result::Err(e) => Option::Some(e),
        }
    }

    fn transpose<U>(self) -> Option<Result<T, U>> {
        match self {
            Result::Ok(Some(v)) => Some(Result::Ok(v)),
            Result::Ok(None) => None,
            Result::Err(e) => Some(Result::Err(e)),
        }
    }
}

// Try trait for ? operator
pub trait Try {
    type Output;
    type Residual;
    fn from_output(output: Self::Output) -> Self;
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
}

pub enum ControlFlow<B, C> {
    Continue(C),
    Break(B),
}

impl<T, E> Try for Result<T, E> {
    type Output = T;
    type Residual = Result<core::convert::Infallible, E>;

    fn from_output(output: T) -> Self { Result::Ok(output) }

    fn branch(self) -> ControlFlow<Self::Residual, T> {
        match self {
            Result::Ok(v) => ControlFlow::Continue(v),
            Result::Err(e) => ControlFlow::Break(Result::Err(e)),
        }
    }
}

impl<T> Try for Option<T> {
    type Output = T;
    type Residual = Option<core::convert::Infallible>;

    fn from_output(output: T) -> Self { Option::Some(output) }

    fn branch(self) -> ControlFlow<Self::Residual, T> {
        match self {
            Option::Some(v) => ControlFlow::Continue(v),
            Option::None => ControlFlow::Break(None),
        }
    }
}

// Convenience macros
#[macro_export]
macro_rules! try {
    ($expr:expr) => {
        match $crate::result::Try::branch($expr) {
            $crate::result::ControlFlow::Continue(val) => val,
            $crate::result::ControlFlow::Break(err) => return $crate::result::Try::from_output(err),
        }
    };
}

#[macro_export]
macro_rules! unwrap {
    ($expr:expr) => {
        match $expr {
            $crate::result::Result::Ok(val) => val,
            $crate::result::Result::Err(err) => panic!("unwrap failed: {:?}", err),
        }
    };
}

#[macro_export]
macro_rules! expect {
    ($expr:expr, $msg:expr) => {
        match $expr {
            $crate::result::Result::Ok(val) => val,
            $crate::result::Result::Err(err) => panic!("{}: {:?}", $msg, err),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result() {
        let ok: Result<i32, &str> = Result::Ok(42);
        assert!(ok.is_ok());
        assert_eq!(ok.unwrap(), 42);

        let err: Result<i32, &str> = Result::Err("error");
        assert!(err.is_err());
        assert_eq!(err.unwrap_err(), "error");
    }

    #[test]
    fn test_result_map() {
        let ok = Result::Ok(42);
        let doubled = ok.map(|x| x * 2);
        assert_eq!(doubled, Result::Ok(84));

        let err: Result<i32, &str> = Result::Err("error");
        let mapped = err.map(|x| x * 2);
        assert_eq!(mapped, Result::Err("error"));
    }

    #[test]
    fn test_result_and_then() {
        let ok = Result::Ok(42);
        let result = ok.and_then(|x| if x > 0 { Result::Ok(x * 2) } else { Result::Err("negative") });
        assert_eq!(result, Result::Ok(84));
    }
}