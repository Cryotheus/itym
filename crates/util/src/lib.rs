//! Dependency of other Itym crates.
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Union transmute.
/// Fully unchecked version of [`core::mem::transmute`].
pub const unsafe fn utransmute<Src, Dst>(src: Src) -> Dst {
	use core::mem::ManuallyDrop;

	union Transmute<Src, Dst> {
		src: ManuallyDrop<Src>,
		dst: ManuallyDrop<Dst>,
	}

	ManuallyDrop::into_inner(unsafe { Transmute::<Src, Dst> { src: ManuallyDrop::new(src) }.dst })
}

/// Size-checking version of [`utransmute`].
#[macro_export]
macro_rules! utransmute {
	(<$src:ty, $dst:ty> $expr:expr) => {{
		const unsafe fn _utransmute_macro_expansion<Src, Dst>(src: Src) -> Dst {
			const {
				if ::core::mem::size_of::<Src>() != ::core::mem::size_of::<Dst>() {
					::core::panic!(
						"{}",
						::core::concat!(
							"utransmute::<",
							::core::stringify!($src),
							", ",
							::core::stringify!($dst),
							"> failed size assertion: `Src == Dst`",
						)
					);
				}
			}

			unsafe { $crate::utransmute::<Src, Dst>(src) }
		}

		_utransmute_macro_expansion::<$src, $dst>($expr)
	}};
}

/// Counts the *pairs* of delimiter tokens. Ignores delimiters that are not free-standing tokes like `"` and `'`.
///
/// Invocations expand into one of the following forms:
/// - `1 + macro!`
/// - `0 + macro!`
/// - `0`
#[macro_export]
macro_rules! delimiter_count {
	(($($nested:tt)*) $($tail:tt)*) => { 1 + $crate::delimiter_count!($($nested)*) $(+ $crate::delimiter_count!($tail))* };
	({$($nested:tt)*} $($tail:tt)*) => { 1 + $crate::delimiter_count!($($nested)*) $(+ $crate::delimiter_count!($tail))* };
	([$($nested:tt)*] $($tail:tt)*) => { 1 + $crate::delimiter_count!($($nested)*) $(+ $crate::delimiter_count!($tail))* };
	($single:tt $($tail:tt)*) => { 0 $(+ $crate::delimiter_count!($tail))* };
	() => { 0 };
}

/// Counts the total amount of tokens, even if they are inside a token tree (`tt`) fragment.
///
/// Invocations expand into one of the following forms:
/// - `2 + macro!`
/// - `1 + macro!`
/// - `1`
/// - `0`
#[macro_export]
macro_rules! token_count {
	(($($nested:tt)*) $($tail:tt)*) => { 2 + $crate::token_count!($($nested)*) $(+ $crate::token_count!($tail))* };
	({$($nested:tt)*} $($tail:tt)*) => { 2 + $crate::token_count!($($nested)*) $(+ $crate::token_count!($tail))* };
	([$($nested:tt)*] $($tail:tt)*) => { 2 + $crate::token_count!($($nested)*) $(+ $crate::token_count!($tail))* };
	($head:tt $($tail:tt)*) => { 1 $(+ $crate::token_count!($tail))* };
	() => { 0 };
}

/// Counts the total amount of token tree (`tt`) fragments.
///
/// Invocations expand into one of the following forms:
/// - `1 + macro!`
/// - `1`
/// - `0`
#[macro_export]
macro_rules! token_tree_count {
	($single:tt $($multiple:tt)+) => { 1 $(+ token_tree_count!($multiple))+ };
	($single:tt) => { 1 };
	() => { 0 };
}
