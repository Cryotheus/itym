//! Assertion and hint macros usable in `const` blocks and functions.
//!
//! ```rust
//! use itym_assert::*;
//!
//! const fn check(byte: u8) -> u8 {
//!     // Just like `assert_ne!`, but usable in `const`
//!     const_assert_ne!(byte, 0);
//!
//! 	byte - 1
//! }
//!
//! fn main() {
//!     const {
//!         let byte = 62;
//!         let checked = check(byte);
//!
//!         // Computes the condition within a `const` block
//!         const_assert_ne!(byte, checked, "Check value unexpectedly matched");
//! 	}
//!
//!     // The `const: ` written here wraps the assertion's condition inside a `const` block.
//!     // Useful for assertions against generic types, such as memory layout checks.
//!     // Failed assertions properly prevent a compile, but lints may not show including with `cargo check`.
//!     const_assert_eq!(const: 61, check(62));
//!
//!     // Equivalent of `debug_assert!`
//!     const_debug_assert!();
//! }
//! ```
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod assert;
mod hint;

#[cfg(feature = "shorting")]
mod shorting;
