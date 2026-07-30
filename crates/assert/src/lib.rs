#![no_std]

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

/// `const`-compatible version of [`assert`].
#[macro_export]
macro_rules! const_assert {
	(macro $(#[$meta:meta])* [$left:expr]) => {
		$crate::const_assert!(
			macro $(#[$meta])* [
				$left,
				"{}",
				::core::concat!(
					"Assertion failed ",
					::core::stringify!($left)
				)
			]
		)
	};

	(macro $(#[$meta:meta])* [$left:expr, $($message:tt)*]) => {
		$(#[$meta])*
		if let false = ($left) { ::core::panic!(
			$($message)*
		); }
	};

	(macro unsafe $(#[$meta:meta])* [$left:expr $(, $($tt:tt)*)?]) => {
		$(#[$meta])*
		unsafe { ::core::hint::assert_unchecked($left) }
	};

	//user-facing
	(const: $left:expr $(, $($message:tt)*)?) => { const { $crate::const_assert!(macro [$left $(, $($message)*)?]) } };
	// (const unsafe: $left:expr $(, $($message:tt)*)?) => { const { $crate::const_assert!(macro unsafe [$left $(, $($message)*)?]) } };
	// (unsafe: $left:expr $(, $($message:tt)*)?) => { $crate::const_assert!(macro unsafe [$left $(, $($message)*)?]) };
	($left:expr $(, $($message:tt)*)?) => { $crate::const_assert!(macro [$left $(, $($message)*)?]) };
}

/// `const`-compatible version of [`assert_eq`].
#[macro_export]
macro_rules! const_assert_eq {
	(macro $(#[$meta:meta])* [$left:expr, $right:expr]) => {
		$crate::const_assert_eq!(
			macro $(#[$meta])* [
				$left,
				$right,
				"{}",
				::core::concat!(
					"Assertion failed ",
					::core::stringify!($left),
					" == ",
					::core::stringify!($right)
				)
			]
		)
	};

	(macro $(#[$meta:meta])* [$left:expr, $right:expr, $($message:tt)*]) => {
		$(#[$meta])*
		if ($left) != ($right) { ::core::panic!($($message)*); }
	};

	(macro unsafe $(#[$meta:meta])* [$left:expr, $right:expr $(, $($tt:tt)*)?]) => {
		$(#[$meta])*
		unsafe { ::core::hint::assert_unchecked($left == $right) }
	};

	//user-facing
	(const: $left:expr, $right:expr $(, $($message:tt)*)?) => { const { $crate::const_assert_eq!(macro [$left, $right $(, $($message)*)?]) } };
	// (const unsafe: $left:expr, $right:expr $(, $($message:tt)*)?) => { const { $crate::const_assert_eq!(macro unsafe [$left, $right $(, $($message)*)?]) } };
	// (unsafe: $left:expr, $right:expr $(, $($message:tt)*)?) => { $crate::const_assert_eq!(macro unsafe [$left, $right $(, $($message)*)?]) };
	($left:expr, $right:expr $(, $($message:tt)*)?) => { $crate::const_assert_eq!(macro [$left, $right $(, $($message)*)?]) };
}

/// `const`-compatible version of [`assert_ne`].
#[macro_export]
macro_rules! const_assert_ne {
	(macro $(#[$meta:meta])* [$left:expr, $right:expr]) => {
		$crate::const_assert_ne!(
			macro $(#[$meta])* [
				$left,
				$right,
				"{}",
				::core::concat!(
					"Assertion failed ",
					::core::stringify!($left),
					" != ",
					::core::stringify!($right)
				)
			]
		)
	};

	(macro $(#[$meta:meta])* [$left:expr, $right:expr, $($message:tt)*]) => {
		$(#[$meta])*
		if ($left) == ($right) { ::core::panic!($($message)*); }
	};

	(macro unsafe $(#[$meta:meta])* [$left:expr, $right:expr $(, $($tt:tt)*)?]) => {
		$(#[$meta])*
		unsafe { ::core::hint::assert_unchecked($left != $right) }
	};

	//user-facing
	(const: $left:expr, $right:expr $(, $($message:tt)*)?) => { const { $crate::const_assert_ne!(macro [$left, $right $(, $($message)*)?]) } };
	// (const unsafe: $left:expr, $right:expr $(, $($message:tt)*)?) => { const { $crate::const_assert_ne!(macro unsafe [$left, $right $(, $($message)*)?]) } };
	// (unsafe: $left:expr, $right:expr $(, $($message:tt)*)?) => { $crate::const_assert_ne!(macro unsafe [$left, $right $(, $($message)*)?]) };
	($left:expr, $right:expr $(, $($message:tt)*)?) => { $crate::const_assert_ne!(macro [$left, $right $(, $($message)*)?]) };
}

/// `const`-compatible version of [`debug_assert`].
///
/// If the assertion should not panic but instead be used as a compiler hint with [`core::hint::assert_unchecked`],
/// prefix the macro invocation's input tokens with `unsafe: `
#[macro_export]
macro_rules! const_debug_assert {
	//user-facing
	(const: $expr:expr $(, $($message:tt)*)?) => {
		const { $crate::const_assert!(macro #[cfg(debug_assertions)] [$expr $(, $($message)*)?]) }
	};

	(const unsafe: $expr:expr $(, $($message:tt)*)?) => {
		const {
			$crate::const_assert!(macro #[cfg(debug_assertions)] [$expr $(, $($message)*)?]);
			// $crate::const_assert!(macro unsafe #[cfg(not(debug_assertions))] [$expr $(, $($message)*)?]);
		}
	};

	(unsafe: $expr:expr $(, $($message:tt)*)?) => {
		{
			$crate::const_assert!(macro #[cfg(debug_assertions)] [$expr $(, $($message)*)?]);
			$crate::const_assert!(macro unsafe #[cfg(not(debug_assertions))] [$expr $(, $($message)*)?]);
		}
	};

	($expr:expr $(, $($message:tt)*)?) => {
		$crate::const_assert!(macro #[cfg(debug_assertions)] [$expr $(, $($message)*)?])
	};
}

/// `const`-compatible version of [`debug_assert_eq`].
///
/// Supports an `unsafe` flag, see [`const_debug_assert`] for details.
#[macro_export]
macro_rules! const_debug_assert_eq {
	(const: $left:expr, $right:expr $(, $($message:tt)*)?) => {
		const { $crate::const_assert_eq!(macro [$left, $right $(, $($message)*)?]) }
	};

	(const unsafe: $left:expr, $right:expr $(, $($message:tt)*)?) => {
		const {
			$crate::const_assert_eq!(macro #[cfg(debug_assertions)] [$left, $right $(, $($message)*)?]);
			//$crate::const_assert_eq!(macro unsafe #[cfg(not(debug_assertions))] [$left, $right $(, $($message)*)?]);
		}
	};

	(unsafe: $left:expr, $right:expr $(, $($message:tt)*)?) => {
		{
			$crate::const_assert_eq!(macro #[cfg(debug_assertions)] [$left, $right $(, $($message)*)?]);
			$crate::const_assert_eq!(macro unsafe #[cfg(not(debug_assertions))] [$left, $right $(, $($message)*)?]);
		}
	};

	($left:expr, $right:expr $(, $($message:tt)*)?) => {
		$crate::const_assert_eq!(macro [$left, $right $(, $($message)*)?])
	};
}

/// `const`-compatible version of [`debug_assert_ne`].
///
/// Supports an `unsafe` flag, see [`const_debug_assert`] for details.
#[macro_export]
macro_rules! const_debug_assert_ne {
	(const: $left:expr, $right:expr $(, $($message:tt)*)?) => {
		const { $crate::const_assert_ne!(macro [$left, $right $(, $($message)*)?]) }
	};

	(const unsafe: $left:expr, $right:expr $(, $($message:tt)*)?) => {
		const {
			$crate::const_assert_ne!(macro #[cfg(debug_assertions)] [$left, $right $(, $($message)*)?]);
			//$crate::const_assert_ne!(macro unsafe #[cfg(not(debug_assertions))] [$left, $right $(, $($message)*)?]);
		}
	};

	(unsafe: $left:expr, $right:expr $(, $($message:tt)*)?) => {
		{
			$crate::const_assert_ne!(macro #[cfg(debug_assertions)] [$left, $right $(, $($message)*)?]);
			$crate::const_assert_ne!(macro unsafe #[cfg(not(debug_assertions))] [$left, $right $(, $($message)*)?]);
		}
	};

	($left:expr, $right:expr $(, $($message:tt)*)?) => {
		$crate::const_assert_ne!(macro [$left, $right $(, $($message)*)?])
	};
}

/// An alternative to the unstable compile option `-Zub-checks`.
///
/// If `debug_assertions` are enabled, acts as [`unreachable`], otherwise behaves like [`core::hint::unreachable_unchecked`].
#[macro_export]
macro_rules! debug_unreachable {
	() => {{
		#[cfg(debug_assertions)]
		{
			unsafe fn debug_unreachable_requires_unsafe() {} //force a lint
			core::unreachable!();
			#[allow(unreachable_code)]
			debug_unreachable_requires_unsafe();
		}

		#[cfg(not(debug_assertions))]
		core::hint::unreachable_unchecked();
	}};
}
