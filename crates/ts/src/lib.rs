//! Types for using the type state design pattern in Rust.
//!
//! Associated types or generics which are meant to act like a state,
//! but in the form of a type,
//! should add [`TypeState<C>`] as a bound.
//!
//! `C` is the type which bounds, `impl` blocks, and `where` predicates should specify.
//!
//! ```rust
//! use itym_ts::{Bool, TypeState};
//!
//! struct StrangeFruit<A>(A);
//!
//! trait SomeConfig {
//!     type IsAqueous: TypeState<bool>;
//! }
//!
//! // Things that support `SomeConfig` and are allowed to run `fn juice()`
//! impl SomeConfig for StrangeFruit<u32> { type IsAqueous = Bool<true>; }
//! impl SomeConfig for StrangeFruit<u64> { type IsAqueous = Bool<true>; }
//! impl SomeConfig for StrangeFruit<f32> { type IsAqueous = Bool<true>; }
//! impl SomeConfig for StrangeFruit<f64> { type IsAqueous = Bool<true>; }
//!
//! // Things that support `SomeConfig` but are not allowed to run `fn juice()`
//! impl SomeConfig for StrangeFruit<i32> { type IsAqueous = Bool<false>; }
//! impl SomeConfig for StrangeFruit<i64> { type IsAqueous = Bool<false>; }
//!
//! // `Bool<VALUE>` exists to allow some amount of computation before choosing the type
//! // `usize` is juiceable in a... very specific situation?
//! impl SomeConfig for StrangeFruit<usize> { type IsAqueous = Bool<{ align_of::<usize>() == 4 }>; }
//!
//! // Any item in this `impl` block will be available when `SomeSetting` resolves to `True`
//! impl<A> StrangeFruit<A>
//! where
//!     Self: SomeConfig,
//!     <Self as SomeConfig>::IsAqueous: TypeState<bool, State = Bool<true>>,
//! {
//!     fn juice(&mut self) {
//!         // We don't need to assert if the type is juiceable
//!         // `A` types which set `IsAqueous` as any type that resolves to `Bool<true>` will meet the bounds above
//!         // everything else emits a compile error, caught by `cargo check` and the likes
//!         /* code */
//!     }
//! }
//!
//! impl StrangeFruit<u32> {
//!     fn make_raisin(self) -> StrangeFruit<i32> {
//!         StrangeFruit((self.0 / 2) as i32 * -1)
//!     }
//! }
//!
//! fn scrumptuous(mut john: StrangeFruit<u32>) {
//!     //works fine...
//!     john.juice(); //this is `StrangeFruit<u32>`, which meets the bounds above
//!
//!     //type changes into `StrangeFruit<i32>`
//!     let john = john.make_raisin();
//!
//!     // *** COMPILE ERROR! ***
//!     // john.juice();
//! }
//! ```
//!
//! This is mostly an internal for `itym` crates.
//!
//! All exposed types in this crate are not inhabited.
//! Meaning: you cannot construct the types here.
//! Instead, [`TypeValue`] offers an associated `const` for a value that might be represented by the type.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature = "f16", feature(f16))]
#![cfg_attr(feature = "f128", feature(f128))]
#![no_std]

use core::convert::Infallible;
use core::marker::PhantomData;

macro_rules! impl_sealed {
	($($target:ty),+ $(,)?) => {
		$(impl Sealed for $target {})+
	};
}

macro_rules! uninhabited {
	(
		$(
			$(#[$meta:meta])*
			$vis:vis $ident:ident
		),+

		$(,)?
	) => {
		$(
		#[derive(Debug)]
		$(#[$meta])*
		$vis enum $ident {}

		impl_sealed! { $ident }
		)+
	};
}

macro_rules! impl_float {
	(
		macro
		$target:ty,
		$($ts:ty = $val:expr),+
	) => {
		$(
			impl Sealed for FloatTypeNum<($target, $ts)> {}

			impl TypeValue for FloatTypeNum<($target, $ts)> {
				type Value = $target;
				const VALUE: Self::Value = $val;
			}
		)+
	};

	(
		$(
			$(#[$meta:meta])*
			$target:ty
		),+
		$(,)?
	) => {
		$(
		$(#[$meta])*
		impl_float! {
			macro
			$target,
			NumZero = 0.0,
			NumNegZero = -0.0,
			NumOne = 1.0,
			NumNegOne = -1.0,
			NumMin = <$target>::MIN,
			NumMax = <$target>::MAX
		}
		)+
	};
}

macro_rules! gen_unit {
	(
		$(
			$(#[$meta:meta])*
			$vis:vis $divergent:ident: $convergent:ty = $value:expr
		),+

		$(,)?
	) => {
		uninhabited! {
			$(
				$(#[$meta])*
				$vis $divergent
			),+
		}

		$(
		impl TypeState<$convergent> for $divergent {
			type State = $divergent;
		}

		impl TypeValue for $divergent {
			type Value = $convergent;

			const VALUE: Self::Value = $value;
		}
		)+
	};
}

macro_rules! gen_value {
	(
		$(
			$(#[$meta:meta])*
			$vis:vis $divergent:ident: $convergent:ty
		),+

		$(,)?
	) => {
		$(
		#[derive(Debug)]
		$(#[$meta])*
		$vis enum $divergent<const VALUE: $convergent> {}

		impl<const VALUE: $convergent> Sealed for $divergent<VALUE> {}

		impl<const VALUE: $convergent> TypeState<$convergent> for $divergent<VALUE> {
			type State = $divergent<VALUE>;
		}

		impl<const VALUE: $convergent> TypeValue for $divergent<VALUE> {
			type Value = $convergent;

			const VALUE: Self::Value = VALUE;
		}

		impl<const VALUE: $convergent> TypeValue for OptionSome<$divergent<VALUE>> {
			type Value = Option<$convergent>;
			const VALUE: Self::Value = Some(VALUE);
		}
		)+
	};
}

macro_rules! gen_integer {
	(
		macro signed $signed:ident
	) => {
		const _: () = {
			#[allow(unused)]
			#[allow(non_upper_case_globals)]
			const $signed: () = ();
		};
	};

	(
		$(
			$(#[$meta:meta])*
			$vis:vis $divergent:ident $($signed:ident)?: $convergent:ty
		),+

		$(,)?
	) => {
		gen_value! {
			$(
			$(#[$meta])*
			$vis $divergent: $convergent
			),+
		}

		$(
		impl TypeState<$convergent> for NumZero { type State = $divergent<0>; }
		impl TypeState<$convergent> for NumOne { type State = $divergent<1>; }
		impl TypeState<$convergent> for NumMin { type State = $divergent<{ <$convergent>::MIN }>; }
		impl TypeState<$convergent> for NumMax { type State = $divergent<{ <$convergent>::MAX }>; }

		$(
		gen_integer! { macro $signed $signed }
		impl TypeState<$convergent> for NumNegOne { type State = $divergent<{ -1 }>; }
		)?
		)+
	};
}

macro_rules! gen_nsize {
	(
		$(
			$(#[$meta:meta])*
			$vis:vis $ident:ident $($signed:ident)?: $src_ty:ty as ($size_ty:ty, $ts_ty:ty)
		),+

		$(,)?
	) => {
		$(
		#[derive(Debug)]
		$(#[$meta])*
		$vis enum $ident<const VALUE: $src_ty> {}

		impl<const VALUE: $src_ty> Sealed for $ident<VALUE> {}

		impl<const VALUE: $src_ty> TypeState<$size_ty> for $ident<VALUE> {
			type State = Self;
		}

		impl<const VALUE: $src_ty> TypeState<$ts_ty> for $ident<VALUE> {
			type State = Self;
		}

		impl<const VALUE: $src_ty> TypeValue for $ident<VALUE> {
			type Value = $size_ty;

			const VALUE: Self::Value = VALUE as $size_ty;
		}

		impl<const VALUE: $src_ty> TypeValue for OptionSome<$ident<VALUE>> {
			type Value = Option<$size_ty>;

			const VALUE: Self::Value = Some(VALUE as $size_ty);
		}
		)+
	};
}

#[cfg(feature = "size_common")]
mod big;

#[cfg(feature = "size_common")]
mod common;

/// Blocks foreign implementers.
trait Sealed {}

/// Convergent type is `T`, and the divergent (broad) type is `Self`.
/// `State` is the reconverged (narrow) type,
#[allow(private_bounds)]
pub trait TypeState<T>: Sealed {
	/// The re-converged (or "most convergent") type.
	type State: ?Sized + TypeValue;
}

#[allow(private_bounds)]
pub trait TypeValue: Sealed {
	type Value;

	const VALUE: Self::Value;
}

/// Const generics do not support floats.
#[derive(Debug)]
pub struct FloatTypeNum<T> {
	_marker: PhantomData<T>,
	_uninhabited: Infallible,
}

/// [`None`] as a type for `Option<T>` type state.
#[derive(Debug)]
pub enum OptionNone {}

impl Sealed for OptionNone {}

impl<T> TypeState<Option<T>> for OptionNone
where
	OptionSome<T>: TypeState<Option<T>>,
{
	type State = OptionNone;
}

impl TypeValue for OptionNone {
	type Value = ();

	const VALUE: Self::Value = ();
}

/// [`Some(T)`] as a type for `Option<T>` type state.
#[derive(Debug)]
pub struct OptionSome<T> {
	_marker: PhantomData<T>,
	_uninhabited: Infallible,
}

impl<T> Sealed for OptionSome<T> {}

impl<T, U> TypeState<Option<U>> for OptionSome<T>
where
	T: TypeState<U>,
{
	type State = <T as TypeState<U>>::State;
}

uninhabited! {
	pub NumZero,
	pub NumNegZero,
	/// Positive or unsigned.
	pub NumOne,
	/// Negative.
	pub NumNegOne,
	pub NumMin,
	pub NumMax,

	/// Smallest representation of `usize` on the stable release channel of Rust.
	pub SmallUsize,

	/// Smallest representation of `usize` on the stable release channel of Rust.
	pub SmallIsize,
}

gen_unit! {
	pub Unit: () = (),
}

impl_float! {
	#[cfg(feature = "f16")] f16,
	f32,
	f64,
	#[cfg(feature = "f128")] f128,
}

gen_value! {
	pub Bool: bool,
	pub Char: char,
}

gen_integer! {
	pub U8: u8,
	pub U16: u16,
	pub U32: u32,
	pub U64: u64,
	pub U128: u128,
	pub Usize: usize,
	pub I8 signed: i8,
	pub I16 signed: i16,
	pub I32 signed: i32,
	pub I64 signed: i64,
	pub I128 signed: i128,
	pub Isize signed: isize,
}

gen_nsize! {
	/// See [`SmallIsize`].
	pub SmallIsizeValue: i16 as (isize, SmallIsize),

	/// See [`SmallUsize`].
	pub SmallUsizeValue: u16 as (usize, SmallUsize),
}

#[cfg(feature = "size_common")]
pub use big::*;

#[cfg(feature = "size_common")]
pub use common::*;
