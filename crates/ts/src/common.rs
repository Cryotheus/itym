use super::*;

uninhabited! {
	/// Casts `usize` to `u32` as a balance between compatible and capable `usize` representations.
	pub CommonUsize,

	/// Casts `isize` to `i32` as a balance between compatible and capable `isize` representations.
	pub CommonIsize,
}

gen_nsize! {
	/// See [`CommonIsize`].
	///
	/// # Panics
	/// Cast of the `VALUE` to usize can overflow, and cause a panic at const-eval time.
	pub CommonIsizeValue: i64 as (isize, CommonIsize),

	/// See [`CommonUsize`].
	///
	/// # Panics
	/// Cast of the `VALUE` to usize can overflow, and cause a panic at const-eval time.
	pub CommonUsizeValue: u64 as (usize, CommonUsize),
}
