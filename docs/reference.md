# NOVA Language Reference

Complete language reference for the NOVA programming language.

## Table of Contents

1. [Lexical Structure](#lexical-structure)
2. [Keywords](#keywords)
3. [Types](#types)
4. [Expressions](#expressions)
5. [Statements](#statements)
6. [Items](#items)
7. [Patterns](#patterns)
8. [Attributes](#attributes)
9. [Macros](#macros)
10. [Built-in Traits](#built-in-traits)

---

## Lexical Structure

### Character Set

NOVA source files are UTF-8 encoded. Any Unicode character is allowed in string literals, comments, and identifiers (with restrictions).

### Whitespace

Whitespace includes spaces, tabs, newlines, and carriage returns. It separates tokens but is otherwise insignificant.

### Comments

```nova
// Line comment - extends to end of line

/* Block comment - can be nested */

/// Outer doc comment - documents the following item

//! Inner doc comment - documents the enclosing item
```

### Identifiers

```
identifier = (letter | '_') (letter | digit | '_')*
letter = Unicode XID_Start
digit = Unicode XID_Continue
```

Identifiers cannot be keywords (unless prefixed with `r#`). They are case-sensitive.

```nova
let variable = 1
let _private = 2
let r#type = 3  // raw identifier
let 变量 = 4     // Unicode allowed
```

### Literals

#### Integer Literals

```
decimal    = digit (digit | '_')*
hex        = '0x' hex_digit (hex_digit | '_')*
octal      = '0o' oct_digit (oct_digit | '_')*
binary     = '0b' bin_digit (bin_digit | '_')*

suffix = 'i8' | 'i16' | 'i32' | 'i64' | 'i128' | 'isize'
       | 'u8' | 'u16' | 'u32' | 'u64' | 'u128' | 'usize'
```

Examples: `42`, `1_000_000`, `0xFF`, `0o77`, `0b1010`, `42i64`, `100u32`

#### Float Literals

```
float = (digit+ '.' digit* | digit+ '.' digit* exponent | '.' digit+ exponent?) suffix?

exponent = ('e' | 'E') ('+' | '-')? digit+

suffix = 'f16' | 'f32' | 'f64' | 'f128'
```

Examples: `3.14`, `1.5e2`, `2e-3`, `0x1.8p3`, `3.14f32`

#### Boolean Literals

```
'true' | 'false'
```

#### Character Literals

```
''' (escape | non_quote_char) '''
```

Escape sequences: `\n`, `\r`, `\t`, `\\`, `\'`, `\"`, `\0`, `\xHH`, `\u{HHHHHH}`

Examples: `'a'`, `'\n'`, `'🚀'`, `'\u{1F680}'`

#### String Literals

```
'"' (escape | non_quote_char)* '"'          // regular string
'r' '#'* '"' (any char except ending) '"' '#'*  // raw string
'b' '"' (byte_escape | ascii_printable)* '"' // byte string
'br' '#'* '"' (any char except ending) '"' '#'* // raw byte string
```

Examples:
```nova
"hello\nworld"
r#"raw "string""#
br"bytes"
```

---

## Keywords

### Reserved Keywords (Cannot be used as identifiers)

```
as        async     await     break     const     continue
crate     do        else      enum      extern    false
fn        for       if        impl      in        let
loop      match     mod       move      mut       pub
ref       return    self      Self      static    struct
super     trait     true      type      unsafe    use
where     while     yield     try       catch     throw
spawn     actor     channel   select    defer     drop
box       dyn       abstract  final     override  virtual
sealed    open      inline    noinline  const_eval comptime
effect    handle    resume    linear    unique    shared
atomic    volatile  unaligned packed    transparent repr
align     cfg       cfg_attr  derive    macro     macro_rules
include   include_str include_bytes concat  stringify
file      line      column    module_path compile_error compile_warning
todo      unimplemented unreachable panic   assert  assert_eq
assert_ne debug_assert debug_assert_eq debug_assert_ne
```

### Contextual Keywords (Valid in specific positions)

```
async     await     move      ref       mut       box
dyn       impl      super     self      Self      crate
const     static    let       auto      _         ..
..=       =>        ->        :         ;         ,
.         ..        ...       =         ==        !=
<         >         <=        >=        +         -
*         /         %         &&        ||        !
&         |         ^         <<        >>        +=
-=        *=        /=        %=        &=        |=
^=        <<=       >>=       @         ?         #
```

---

## Types

### Type Grammar

```
type = primitive
     | path
     | tuple
     | array
     | slice
     | reference
     | pointer
     | function
     | never
     | trait_object
     | impl_trait
     | parenthesized

primitive = 'bool' | 'char' | 'str' | '()' 
          | 'i8' | 'i16' | 'i32' | 'i64' | 'i128' | 'isize'
          | 'u8' | 'u16' | 'u32' | 'u64' | 'u128' | 'usize'
          | 'f16' | 'f32' | 'f64' | 'f128'

tuple = '(' (type (',' type)*)? ')'
array = '[' type ';' expr ']'
slice = '[' type ']'
reference = '&' ('mut')? type
pointer = '*' ('mut' | 'const') type
function = 'fn' '(' (type (',' type)*)? ')' '->' type
never = '!'
trait_object = 'dyn' type ('+' type)* ('+' 'Send')? ('+' 'Sync')?
impl_trait = 'impl' type ('+' type)*
```

### Type Aliases

```nova
type Result<T> = std::result::Result<T, MyError>
type Callback = fn(i32) -> i32
```

### Type Parameters

```nova
struct Wrapper<T, U = String> { ... }
fn func<T: Clone + Send>(x: T) -> T { ... }
```

### Const Generics

```nova
struct Array<T, const N: usize> { data: [T; N] }
fn take_array<const N: usize>(arr: [i32; N]) { ... }
```

---

## Expressions

### Expression Precedence (Highest to Lowest)

1. **Paths/Variables/Literals**: `x`, `42`, `"hello"`
2. **Parenthesized**: `(expr)`
3. **Field/Index Access**: `expr.field`, `expr[index]`
4. **Method Calls**: `expr.method()`
5. **Function Calls**: `func()`, `func(args)`
6. **Type Ascription**: `expr as Type`
7. **Unary**: `-expr`, `!expr`, `*expr`, `&expr`, `&mut expr`
8. **Multiplicative**: `*`, `/`, `%`
9. **Additive**: `+`, `-`
10. **Shift**: `<<`, `>>`
11. **Bitwise AND**: `&`
12. **Bitwise XOR**: `^`
13. **Bitwise OR**: `|`
14. **Comparison**: `==`, `!=`, `<`, `>`, `<=`, `>=`
14. **Logical AND**: `&&`
15. **Logical OR**: `||`
16. **Range**: `..`, `..=`
17. **Assignment**: `=`, `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=`
18. **Blocks/Control Flow**: `{}`, `if`, `match`, `loop`, `for`, `while`, `return`, `break`, `continue`

### Literal Expressions

```nova
42              // integer
3.14            // float
true            // boolean
'c'             // character
"hello"         // string
b"bytes"        // byte string
()              // unit
```

### Path Expressions

```nova
x                    // variable
std::vec::Vec        // absolute path
crate::module::Type  // crate root
super::parent        // parent module
self::current        // current module
Type::method         // associated function
<Type as Trait>::method  // fully qualified
```

### Tuple Expressions

```nova
(1, 2, 3)      // tuple
(42,)          // single-element tuple (trailing comma required)
()             // empty tuple (unit)
```

### Array/Slice Expressions

```nova
[1, 2, 3]        // array
[0; 100]         // repeated element
&[1, 2, 3]       // slice reference
```

### Block Expressions

```nova
{
    let x = 1;
    let y = 2;
    x + y  // implicit return (no semicolon)
}
```

### Control Flow Expressions

```nova
// If expression
if condition { then_expr } else { else_expr }

// Match expression
match value {
    pattern => expr,
    pattern => { block_expr },
}

// Loop expressions
loop { break expr }
while condition { body }
for pattern in iterable { body }
```

### Operator Expressions

```nova
// Arithmetic
a + b, a - b, a * b, a / b, a % b

// Comparison
a == b, a != b, a < b, a > b, a <= b, a >= b

// Logical
a && b, a || b, !a

// Bitwise
a & b, a | b, a ^ b, !a, a << b, a >> b

// Assignment
a = b
a += b
a -= b
// ... etc.

// Range
a..b      // exclusive
a..=b     // inclusive

// Type cast
expr as Type
```

### Call Expressions

```nova
func()                    // no args
func(1, 2, 3)             // positional
func { x: 1, y: 2 }       // struct-like
method(&self, args)       // method
<Type as Trait>::func()   // fully qualified
```

### Field/Index Access

```nova
expr.field              // struct field
expr.0                  // tuple index
expr[index]             // index
expr[start..end]        // slice range
expr[..]                // full slice
```

### Await/Async

```nova
future.await            // await future
async { expr }          // async block
async fn name() { }     // async function
```

### Try Operator

```nova
expr?    // propagate error
```

### Macros

```nova
macro!()                  // macro invocation
macro! { ... }            // block macro
macro! [ ... ]            // bracket macro
```

---

## Statements

### Let Statements

```nova
let pattern = expr;           // immutable
let mut pattern = expr;       // mutable
let pattern: Type = expr;     // typed
const PATTERN: Type = expr;   // constant
static PATTERN: Type = expr;  // static
```

### Expression Statements

```nova
expr;    // any expression followed by semicolon
```

### Control Flow Statements

```nova
return expr;          // return from function
return;               // return unit

break;                // break loop
break 'label;         // break labeled loop
break expr;           // break with value (loop only)

continue;             // continue loop
continue 'label;      // continue labeled loop
```

### Item Statements

Items can appear as statements in blocks:

```nova
fn foo() { }          // function
struct Bar { }        // struct
enum Baz { }          // enum
mod qux { }           // module
use std::vec::Vec;    // use declaration
```

---

## Items

### Functions

```nova
fn name(parameters) -> return_type {
    body
}

// Async
async fn name() -> Type { }

// Const fn (compile-time)
const fn name() -> i32 { 42 }

// Unsafe
unsafe fn name() { }

// Extern
extern "C" fn name() { }

// Generic
fn name<T: Trait>(x: T) -> T { }

// With where clause
fn name<T>(x: T) where T: Clone + Send { }
```

### Structs

```nova
struct Name {
    field: Type,
    field: Type,
}

// Tuple struct
struct Name(Type, Type);

// Unit struct
struct Name;

// With generics
struct Name<T, U> { ... }

// With where clause
struct Name<T> where T: Clone { ... }

// Attributes
#[derive(Debug, Clone)]
#[repr(C)]
struct Name { ... }
```

### Enums

```nova
enum Name {
    Variant,
    Variant(Type),
    Variant(Type, Type),
    Variant { field: Type, field: Type },
}

// With generics
enum Option<T> {
    Some(T),
    None,
}

// With discriminant
enum Enum {
    A = 1,
    B = 2,
    C = 3,
}

// Attributes
#[derive(Debug)]
#[repr(u8)]
enum Name { ... }
```

### Traits

```nova
trait Name {
    fn method(&self) -> Type;
    fn method(&mut self);
    fn method(self);
    const CONST: Type = value;
    type Associated;
    
    // Default implementation
    fn default_method(&self) { ... }
}

// Supertraits
trait Name: Supertrait + OtherTrait { }

// Auto traits
auto trait Name { }

// Unsafe traits
unsafe trait Name { }
```

### Implementations

```nova
impl Type {
    fn method(&self) { }
}

impl<T> Type<T> {
    fn method(&self) { }
}

impl Trait for Type {
    fn method(&self) { }
}

impl<T: Trait> OtherTrait for Type<T> { }

// Negative impl (nightly)
impl !Trait for Type { }
```

### Type Aliases

```nova
type Name = Type;
type Name<T> = Type<T>;
```

### Modules

```nova
mod name { }                    // inline module
mod name;                       // file module (name.nova or name/mod.nova)
mod name { pub mod inner { } }  // nested

// Re-exports
pub use crate::module::name;
pub use crate::module::name as alias;
```

### Use Declarations

```nova
use path::to::item;
use path::to::{item1, item2};
use path::to::item as alias;
use path::to::*;  // glob import
use path::to::item as _;  // unused import
```

### Constants and Statics

```nova
const NAME: Type = expr;      // compile-time constant
static NAME: Type = expr;     // static variable
static mut NAME: Type = expr; // mutable static (unsafe)
```

### Extern Crates

```nova
extern crate name;
extern crate name as alias;
```

### Attributes on Items

```nova
#[attribute]
#[attribute(arg)]
#[attribute = "value"]
item
```

---

## Patterns

### Pattern Grammar

```
pattern = literal
        | identifier
        | wildcard
        | range
        | reference
        | struct
        | tuple
        | slice
        | or
        | binding
        | guard
        | group
```

### Pattern Types

```nova
// Literal
42, "hello", true, 'c'

// Wildcard
_                       // matches anything, doesn't bind

// Identifier (binding)
x                       // binds variable
ref x                   // binds by reference
ref mut x               // binds by mutable reference

// Range
1..=10                  // inclusive range
'a'..='z'               // char range

// Reference
&pattern                // matches reference
&mut pattern            // matches mutable reference

// Struct
Struct { field, .. }    // with rest
Struct { field: pattern }  // rename
Struct { field: pattern, .. }

// Tuple
(Pattern, Pattern, ...)
(Pattern, ..)           // rest

// Slice
[Pattern, Pattern, ..]  // prefix + rest
[.., Pattern]           // suffix
[Pattern, .., Pattern]  // prefix + suffix
[Pattern; N]            // exact length

// Or-pattern
Pattern | Pattern | ...

// Binding with @
x @ Pattern             // binds entire match

// Guard
Pattern if condition

// Group
(Pattern)
```

### Pattern Matching Contexts

```nova
// match arms
match expr {
    pattern => expr,
    pattern => { block },
}

// if let
if let pattern = expr { }

// while let
while let pattern = expr { }

// for loops
for pattern in iter { }

// let statements
let pattern = expr;

// function parameters
fn func(pattern: Type) { }

// closures
|pattern| expr

// try blocks (experimental)
try { pattern? }
```

---

## Attributes

### Attribute Syntax

```nova
#[attribute]                    // outer attribute (applies to following item)
#![attribute]                   // inner attribute (applies to enclosing item)
#[attribute = "value"]          // key-value
#[attribute(arg1, arg2)]        // with arguments
#[attribute(key = "value")]     // meta items
```

### Built-in Attributes

#### Conditional Compilation

```nova
#[cfg(target_os = "linux")]
#[cfg(feature = "serde")]
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
```

#### Testing

```nova
#[test]
#[bench]
#[ignore = "reason"]
#[should_panic(expected = "msg")]
```

#### Documentation

```nova
/// Doc comment
#[doc = "Custom doc"]
#[doc(alias = "name")]
#[doc(cfg(feature = "name"))]
#[doc(hidden)]
#[doc(include = "path")]
```

#### Lint Control

```nova
#[allow(lint_name)]
#[warn(lint_name)]
#[deny(lint_name)]
#[forbid(lint_name)]

// Examples
#[allow(dead_code)]
#[warn(unused_variables)]
#[deny(clippy::unwrap_used)]
```

#### Optimization

```nova
#[inline]
#[inline(always)]
#[inline(never)]
#[cold]
```

#### Linking/FFI

```nova
#[link(name = "lib")]
#[link(name = "lib", kind = "static")]
#[link(name = "lib", kind = "dylib")]
#[no_mangle]
#[link_section = "section"]
#[export_name = "name"]
```

#### Representation

```nova
#[repr(C)]
#[repr(packed)]
#[repr(packed(N))]
#[repr(transparent)]
#[repr(u8)]           // enum discriminant
#[repr(i32)]
#[repr(align(N))]
```

#### Derive Macros

```nova
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
```

#### Must Use

```nova
#[must_use]
#[must_use = "message"]
```

#### Deprecation

```nova
#[deprecated]
#[deprecated(since = "1.0", note = "use foo instead")]
```

#### Non-exhaustive

```nova
#[non_exhaustive]
```

### Attribute Targets

Attributes can be applied to any item, statement, or expression depending on the attribute.

---

## Macros

### Declarative Macros (`macro_rules!`)

```nova
macro_rules! name {
    // Matchers
    ($($token:tt)*) => { expansion };
    ($($x:expr),*) => { vec![$($x),*] };
    ($($x:ident)*) => { ... };
    ($($x:ty)*) => { ... };
    ($($x:path)*) => { ... };
    ($($x:pat)*) => { ... };
    ($($x:stmt)*) => { ... };
    ($($x:block)*) => { ... };
    ($($x:meta)*) => { ... };
    ($($x:tt)*) => { ... };  // token trees
}
```

### Fragment Specifiers

| Specifier | Matches |
|-----------|---------|
| `expr` | Expression |
| `ty` | Type |
| `pat` | Pattern |
| `stmt` | Statement |
| `block` | Block |
| `ident` | Identifier |
| `path` | Path |
| `tt` | Token tree |
| `meta` | Meta item |
| `lifetime` | Lifetime |
| `vis` | Visibility |

### Repetition

```nova
$($x:expr),*     // zero or more, separated by comma
$($x:expr),+     // one or more, separated by comma
$($x:expr)*      // zero or more, no separator
$($x:expr)+      // one or more, no separator
```

### Hygiene

Macros are hygienic by default. Use `$crate` for crate root, `macro_export` for export.

### Procedural Macros

```nova
// Derive macro
#[proc_macro_derive(TraitName)]
pub fn derive(input: TokenStream) -> TokenStream { ... }

// Attribute macro
#[proc_macro_attribute]
pub fn attr(args: TokenStream, item: TokenStream) -> TokenStream { ... }

// Function-like macro
#[proc_macro]
pub fn func(input: TokenStream) -> TokenStream { ... }
```

---

## Built-in Traits

### Marker Traits

| Trait | Meaning |
|-------|---------|
| `Send` | Safe to transfer across threads |
| `Sync` | Safe to share across threads (`&T: Send`) |
| `Copy` | Bitwise copyable |
| `Clone` | Explicitly cloneable |
| `Sized` | Known size at compile time |
| `Unpin` | Safe to move after pinning |

### Operator Traits

| Trait | Operator |
|-------|----------|
| `Add` | `+` |
| `Sub` | `-` |
| `Mul` | `*` |
| `Div` | `/` |
| `Rem` | `%` |
| `Neg` | `-` (unary) |
| `Not` | `!` |
| `BitAnd` | `&` |
| `BitOr` | `|` |
| `BitXor` | `^` |
| `Shl` | `<<` |
| `Shr` | `>>` |
| `AddAssign` | `+=` |
| `SubAssign` | `-=` |
| `MulAssign` | `*=` |
| `DivAssign` | `/=` |
| `RemAssign` | `%=` |

### Comparison Traits

| Trait | Methods |
|-------|---------|
| `PartialEq` | `==`, `!=` |
| `Eq` | (marker) |
| `PartialOrd` | `<`, `>`, `<=`, `>=` |
| `Ord` | `cmp` |

### Conversion Traits

| Trait | Method |
|-------|--------|
| `From<T>` | `from(T) -> Self` |
| `Into<T>` | `into(self) -> T` |
| `TryFrom<T>` | `try_from(T) -> Result<Self, Error>` |
| `TryInto<T>` | `try_into(self) -> Result<T, Error>` |
| `AsRef<T>` | `as_ref(&self) -> &T` |
| `AsMut<T>` | `as_mut(&mut self) -> &mut T` |

### Iterator Traits

| Trait | Methods |
|-------|---------|
| `Iterator` | `next(&mut self) -> Option<Item>` |
| `IntoIterator` | `into_iter(self) -> Iterator` |
| `DoubleEndedIterator` | `next_back(&mut self) -> Option<Item>` |
| `ExactSizeIterator` | `len(&self) -> usize` |
| `FusedIterator` | (marker) |

### Future/Async Traits

| Trait | Methods |
|-------|---------|
| `Future` | `poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Output>` |
| `IntoFuture` | `into_future(self) -> Future` |

### Error Traits

| Trait | Methods |
|-------|---------|
| `Error` | `source(&self) -> Option<&dyn Error>` |
| `Display` | `fmt(&self, &mut Formatter) -> Result` |
| `Debug` | `fmt(&self, &mut Formatter) -> Result` |

### Function Traits

| Trait | Call Style |
|-------|------------|
| `Fn` | `fn(&self, args) -> Ret` |
| `FnMut` | `fn(&mut self, args) -> Ret` |
| `FnOnce` | `fn(self, args) -> Ret` |

### Default/Zero Traits

| Trait | Method |
|-------|--------|
| `Default` | `default() -> Self` |
| `Zero` | `zero() -> Self` |
| `One` | `one() -> Self` |

---

## Appendix: Grammar Summary

### Full Grammar (EBNF)

```
source_file = item*

item = function
     | struct
     | enum
     | trait
     | impl
     | type_alias
     | module
     | use_decl
     | const_item
     | static_item
     | extern_crate
     | extern_block

function = 'fn' ident generics? '(' params? ')' ('->' type)? where_clause? block
struct = 'struct' ident generics? where_clause? '{' fields? '}'
      | 'struct' ident generics? where_clause? '(' types? ')'
      | 'struct' ident generics? where_clause? ';'

enum = 'enum' ident generics? where_clause? '{' variants? '}'

trait = 'trait' ident generics? (':' supertraits)? where_clause? '{' items? '}'

impl = 'impl' generics? (trait 'for')? type where_clause? '{' items? '}'

block = '{' statements? '}'

statement = let_stmt | expr_stmt | item_stmt | control_stmt

expr = literal | path | tuple | array | block | if | match | loop | while
     | for | call | method | field | index | cast | unary | binary
     | range | await | try | closure | macro | continue | break | return
```

---

## Version Information

This reference documents NOVA 1.0 (2026 Edition).

For the latest updates, see: https://github.com/samprit-web/NOVA