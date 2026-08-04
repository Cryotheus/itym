use super::*;

uninhabited! {
	/// Largest representation of `usize` on the stable release channel of Rust.
	pub BigUsize,

	/// Largest representation of `usize` on the stable release channel of Rust.
	pub BigIsize,
}

gen_nsize! {
	/// See [`BigIsize`].
	///
	/// # Panics
	/// Cast of the `VALUE` to usize can overflow, and cause a panic at const-eval time.
	pub BigIsizeValue: i64 as (isize, BigIsize),

	/// See [`BigUsize`].
	///
	/// # Panics
	/// Cast of the `VALUE` to usize can overflow, and cause a panic at const-eval time.
	pub BigUsizeValue: u64 as (usize, BigUsize),
}
