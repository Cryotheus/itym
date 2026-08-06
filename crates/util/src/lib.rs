//! Dependency of other Itym crates.
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

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
