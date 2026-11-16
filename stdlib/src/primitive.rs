//! Primitive types and their implementations

use core::fmt;
use core::hash::Hash;
use core::ops::*;

// Integer types
macro_rules! impl_int {
    ($($t:ty),*) => {
        $(
            impl fmt::Debug for $t {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, "{}", self)
                }
            }

            impl fmt::Display for $t {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, "{}", self)
                }
            }

            impl Hash for $t {
                fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
                    (*self).hash(state)
                }
            }

            impl PartialEq for $t {
                fn eq(&self, other: &Self) -> bool {
                    *self == *other
                }
            }

            impl Eq for $t {}

            impl PartialOrd for $t {
                fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                    Some(self.cmp(other))
                }
            }

            impl Ord for $t {
                fn cmp(&self, other: &Self) -> core::cmp::Ordering {
                    (*self).cmp(other)
                }
            }
        )*
    };
}

impl_int!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

// Float types
macro_rules! impl_float {
    ($($t:ty),*) => {
        $(
            impl fmt::Debug for $t {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, "{}", self)
                }
            }

            impl fmt::Display for $t {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, "{}", self)
                }
            }

            impl Hash for $t {
                fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
                    // IEEE 754 hash
                    let bits = self.to_bits();
                    bits.hash(state);
                }
            }

            impl PartialEq for $t {
                fn eq(&self, other: &Self) -> bool {
                    *self == *other
                }
            }

            impl PartialOrd for $t {
                fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                    self.partial_cmp(other)
                }
            }
        )*
    };
}

impl_float!(f16, f32, f64, f128);

// Bool
impl fmt::Debug for bool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for bool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

// Char
impl fmt::Debug for char {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "'{}'", self.escape_debug())
    }
}

impl fmt::Display for char {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

// Never type (empty)
pub enum Never {}

// Unit type
impl fmt::Debug for () {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "()")
    }
}

impl fmt::Display for () {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "()")
    }
}

// Pointer types
impl<T: ?Sized> fmt::Debug for *const T {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self)
    }
}

impl<T: ?Sized> fmt::Debug for *mut T {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self)
    }
}

impl<T: ?Sized> fmt::Pointer for *const T {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self)
    }
}

impl<T: ?Sized> fmt::Pointer for *mut T {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self)
    }
}

// References
impl<T: ?Sized + fmt::Debug> fmt::Debug for &T {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (*self).fmt(f)
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for &mut T {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (*self).fmt(f)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_primitive_display() {
        assert_eq!(format!("{}", 42i32), "42");
        assert_eq!(format!("{}", 3.14f64), "3.14");
        assert_eq!(format!("{}", true), "true");
        assert_eq!(format!("{}", 'a'), "a");
    }
}