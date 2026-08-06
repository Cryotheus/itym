//! See [`Align`].

#[allow(private_bounds)]
pub trait Align: Sealed {
	type Aligned;
}

#[repr(transparent)]
pub struct AlignP<T, const POW: usize>(pub <Self as Align>::Aligned)
where
	Self: Align;

/// 
#[repr(transparent)]
pub struct AlignTo<T, const EXACT: usize>(pub <Self as Align>::Aligned)
where
	Self: Align;

/// Implementations shared between [`AlignP`] and [`AlignTo`].
macro_rules! shared_impls {
	($($name:literal <$t:ident, $a:ident> $target:ty),* $(,)?) => {
		$(
		impl<$t, const $a: usize> $target
		where
			Self: Align
		{
			pub const fn _foo(&self) -> Self {
				todo!()
			}
		}

		impl<$t, const $a: usize> ::core::ops::Deref for $target
		where
			Self: Align,
		{
			type Target = <Self as Align>::Aligned;

			fn deref(&self) -> &Self::Target {
				&self.0
			}
		}

		impl<$t, const $a: usize> ::core::ops::DerefMut for $target
		where
			Self: Align,
		{
			fn deref_mut(&mut self) -> &mut Self::Target {
				&mut self.0
			}
		}

		impl<$t, const $a: usize> Clone for $target
		where
			Self: Align,
			<Self as Align>::Aligned: Clone,
		{
			fn clone(&self) -> Self {
				Self(self.0.clone())
			}
		}

		impl<$t, const $a: usize> Copy for $target
		where
			Self: Align,
			<Self as Align>::Aligned: Copy,
		{
		}

		impl<$t, const $a: usize> ::core::fmt::Debug for $target
		where
			Self: Align,
			<Self as Align>::Aligned: ::core::fmt::Debug,
		{
			fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
				f.debug_tuple($name).field(&self.0).finish()
			}
		})*
	};
}

shared_impls! {
	"AlignTo" <T, A> AlignTo<T, A>,
	"AlignP" <T, A> AlignP<T, A>,
}

/// Types which are self-describing alignments.
macro_rules! align_self_impl {
	($($target:ty),* $(,)?) => {
		$(impl Align for $target { type Aligned = Self; }
		impl Sealed for $target {})*
	};
}

align_self_impl!(u8, u16, u32, u64, u128, usize,);

/// For generating types (new-type pattern :D) with `repr(align(N))`
macro_rules! align_newtype {
	($($(#[$meta:meta])* $ident:ident = $align:literal),* $(,)?) => {
		$(#[repr(align($align))]
		#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
		pub struct $ident<T>(pub T);

		impl<T> Align for AlignTo<T, $align> { type Aligned = $ident<T>; }
		impl<T> Sealed for AlignTo<T, $align> {}

		impl<T> AlignTo<T, $align> {
			pub const fn new(value: T) -> Self {
				Self($ident(value))
			}

			pub const fn as_ref(&self) -> &$ident<T> {
				&self.0
			}

			pub const fn as_mut(&mut self) -> &mut $ident<T> {
				&mut self.0
			}
		}

		impl<T> Align for AlignP<T, { (0usize + $align).trailing_zeros() as usize }> { type Aligned = $ident<T>; }
		impl<T> Sealed for AlignP<T, { (0usize + $align).trailing_zeros() as usize }> {}

		impl<T> AlignP<T, { (0usize + $align).trailing_zeros() as usize }> {
			pub const fn new(value: T) -> Self {
				Self($ident(value))
			}

			pub const fn as_ref(&self) -> &$ident<T> {
				&self.0
			}

			pub const fn as_mut(&mut self) -> &mut $ident<T> {
				&mut self.0
			}
		})*
	};
}

align_newtype! {
	AlignP0 = 1,
	AlignP1 = 2,
	AlignP2 = 4,
	AlignP3 = 8,
	AlignP4 = 16,
	AlignP5 = 32,
	AlignP6 = 64,
	AlignP7 = 128,
	AlignP8 = 256,
	AlignP9 = 512,
	AlignP10 = 1024,
	AlignP11 = 2048,
	AlignP12 = 4096,
	AlignP13 = 8192,
	AlignP14 = 16384,
	AlignP15 = 32768,
}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
align_newtype! {
	AlignP16 = 65536,
	AlignP17 = 131072,
	AlignP18 = 262144,
	AlignP19 = 524288,
	AlignP20 = 1048576,
	AlignP21 = 2097152,
	AlignP22 = 4194304,
	AlignP23 = 8388608,
	AlignP24 = 16777216,
	AlignP25 = 33554432,
	AlignP26 = 67108864,
	AlignP27 = 134217728,
	AlignP28 = 268435456,
	AlignP29 = 536870912,
}

trait Sealed {}
