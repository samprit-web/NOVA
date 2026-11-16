//! NOVA Standard Library
//!
//! This crate provides the core standard library for the NOVA programming language.

// Re-export core modules
pub mod prelude;
pub mod primitive;
pub mod mem;
pub mod sync;
pub mod thread;
pub mod task;
pub mod io;
pub mod fs;
pub mod net;
pub mod time;
pub mod collections;
pub mod string;
pub mod fmt;
pub mod iter;
pub mod ops;
pub mod convert;
pub mod borrow;
pub mod boxed;
pub mod rc;
pub mod cell;
pub mod ptr;
pub mod slice;
pub mod str;
pub mod array;
pub mod tuple;
pub mod option;
pub mod result;
pub mod error;
pub mod any;
pub mod format;
pub mod macros;
pub mod env;
pub mod process;
pub mod path;
pub mod ffi;
pub mod os;
pub mod arch;
pub mod intrinsics;
pub mod panic;
pub mod hint;
pub mod marker;
pub mod num;
pub mod bit;
pub mod hash;
pub mod cmp;

// Re-export commonly used types
pub use prelude::*;

// Platform-specific modules
#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "android")]
pub mod android;
#[cfg(target_os = "freebsd")]
pub mod freebsd;

// Feature-gated modules
#[cfg(feature = "serde")]
pub mod serde_support;

#[cfg(feature = "async")]
pub mod async_support;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the standard library
pub fn init() {
    // Initialize panic hook, allocator, etc.
    crate::panic::set_hook(Box::new(|info| {
        eprintln!("Panic: {}", info);
    }));
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_std_init() {
        init();
    }
}