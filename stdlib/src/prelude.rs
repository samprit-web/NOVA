//! NOVA Prelude - Automatically imported items

// Primitive types
pub use crate::primitive::{
    bool, char,
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize,
    f16, f32, f64, f128,
    Never,
};

// Core traits
pub use crate::marker::{
    Copy, Clone, Send, Sync, Sized, Unpin,
    Drop, PhantomData, PhantomPinned,
};

pub use crate::convert::{
    From, Into, TryFrom, TryInto, AsRef, AsMut, IntoIterator,
};

pub use crate::ops::{
    Add, Sub, Mul, Div, Rem,
    AddAssign, SubAssign, MulAssign, DivAssign, RemAssign,
    Neg, Not,
    BitAnd, BitOr, BitXor,
    BitAndAssign, BitOrAssign, BitXorAssign,
    Shl, Shr, ShlAssign, ShrAssign,
    Deref, DerefMut,
    Index, IndexMut,
    Fn, FnMut, FnOnce,
};

pub use crate::cmp::{
    Eq, PartialEq, Ord, PartialOrd,
};

pub use crate::hash::{
    Hash, Hasher, BuildHasher,
};

pub use crate::fmt::{
    Debug, Display, Binary, Octal, LowerHex, UpperHex,
    Pointer, Formatter, Write,
};

pub use crate::borrow::{
    Borrow, BorrowMut, Cow, ToOwned,
};

pub use crate::boxed::Box;
pub use crate::rc::Rc;
pub use crate::sync::Arc;
pub use crate::cell::{Cell, RefCell, UnsafeCell, OnceCell, Lazy};

pub use crate::ptr::{
    NonNull, const_ptr, mut_ptr,
};

pub use crate::slice::{
    SliceExt, SliceIndex,
};

pub use crate::str::{
    StrExt, FromStr, ParseError,
};

pub use crate::array::{
    ArrayExt,
};

pub use crate::tuple::{
    TupleExt,
};

pub use crate::option::{
    Option, OptionExt,
};

pub use crate::result::{
    Result, ResultExt,
};

pub use crate::error::{
    Error, ErrorExt,
};

pub use crate::any::{
    Any, TypeId, type_name, type_name_of_val,
};

pub use crate::iter::{
    Iterator, IteratorExt, DoubleEndedIterator, ExactSizeIterator,
    FusedIterator, TrustedLen,
    IntoIterator, FromIterator,
};

pub use crate::collections::{
    Vec, VecDeque, LinkedList,
    HashMap, HashSet, BTreeMap, BTreeSet,
    BinaryHeap,
};

pub use crate::string::{
    String, ToString,
};

pub use crate::format::{
    format, format_args, write, writeln,
};

pub use crate::macros::{
    vec, format_args, println, eprintln, print, eprint,
    todo, unimplemented, unreachable, panic, assert, assert_eq, assert_ne,
    debug_assert, debug_assert_eq, debug_assert_ne,
    cfg, cfg_attr, compile_error, compile_warning,
    concat, stringify, file, line, column, module_path,
    include, include_str, include_bytes,
    try, try_from, try_into,
};

pub use crate::mem::{
    size_of, size_of_val, align_of, align_of_val,
    forget, replace, swap, take, transmute, transmute_copy,
    zeroed, uninitialized, maybe_uninit, MaybeUninit,
    drop, needs_drop, ManuallyDrop,
};

pub use crate::sync::{
    Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
    Condvar, WaitTimeoutResult,
    Once, OnceState,
    Barrier, BarrierWaitResult,
    atomic::*,
};

pub use crate::thread::{
    Thread, ThreadId, Builder, JoinHandle,
    scope, current, sleep, park, park_timeout,
    yield_now, available_parallelism,
};

pub use crate::task::{
    Future, Poll, Context, Waker,
    spawn, spawn_blocking, join, try_join, select, race,
    yield_now as task_yield, sleep as task_sleep,
    timeout, LocalExecutor, ThreadPoolExecutor,
};

pub use crate::io::{
    Read, Write, Seek, BufRead,
    BufReader, BufWriter,
    Cursor, Lines, Split,
    stdin, stdout, stderr,
    Error, ErrorKind, Result,
};

pub use crate::fs::{
    File, OpenOptions, Metadata, DirEntry, ReadDir,
    FileType, Permissions,
    read, write, read_to_string, read_dir,
    create_dir, create_dir_all, remove_file, remove_dir, remove_dir_all,
    rename, copy, hard_link, soft_link, read_link,
    canonicalize, exists, is_file, is_dir,
};

pub use crate::net::{
    TcpListener, TcpStream, UdpSocket,
    ToSocketAddrs, SocketAddr, IpAddr, Ipv4Addr, Ipv6Addr,
    Shutdown,
};

pub use crate::time::{
    Instant, Duration, SystemTime, UNIX_EPOCH,
    sleep as time_sleep, timeout as time_timeout,
};

pub use crate::env::{
    args, args_os, var, var_os, vars, vars_os,
    set_var, remove_var, current_dir, set_current_dir,
    home_dir, temp_dir,
};

pub use crate::process::{
    Command, Child, ExitStatus, Output, Stdio,
    spawn, exit, abort,
};

pub use crate::path::{
    Path, PathBuf, Component, Components, Prefix, Iter,
    MAIN_SEPARATOR, MAIN_SEPARATOR_STR,
};

pub use crate::ffi::{
    CString, CStr, OsString, OsStr,
    c_void, c_char, c_schar, c_uchar,
    c_short, c_ushort, c_int, c_uint,
    c_long, c_ulong, c_longlong, c_ulonglong,
    c_float, c_double,
    intptr_t, uintptr_t,
    size_t, ssize_t,
    off_t, dev_t, ino_t, mode_t, nlink_t, uid_t, gid_t,
    time_t, clock_t, pid_t,
};

pub use crate::num::{
    NonZero, NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize,
    NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize,
    Wrapping, Saturating,
    CheckedAdd, CheckedSub, CheckedMul, CheckedDiv, CheckedRem,
    CheckedNeg, CheckedShl, CheckedShr,
    SaturatingAdd, SaturatingSub, SaturatingMul, SaturatingDiv, SaturatingRem,
    WrappingAdd, WrappingSub, WrappingMul, WrappingDiv, WrappingRem,
};

pub use crate::bit::{
    reverse_bits, count_ones, count_zeros,
    leading_zeros, trailing_zeros, rotate_left, rotate_right,
    swap_bytes, to_be, to_le, from_be, from_le,
};

pub use crate::hash::{
    DefaultHasher, RandomState, BuildHasherDefault,
    SipHasher, FxHasher, AhashHasher,
};

pub use crate::marker::{
    Unsize, CoerceUnsized, DispatchFromDyn,
    Pointee, Thin, DynMetadata,
};

// Re-export std::prelude for compatibility
#[doc(hidden)]
pub use std::prelude::v1::*;