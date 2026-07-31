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
//! 		// everything else emits a compile error, caught by `cargo check` and the likes
//!         /* code */
//!     }
//! }
//!
//! impl StrangeFruit<u32> {
//!     fn make_raisin(self) -> StrangeFruit<i32> {
//!         StrangeFruit((self.0 / 2) as i32 * -1)
//! 	}
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
#![cfg_attr(feature = "f16", feature(f16))]
#![cfg_attr(feature = "f128", feature(f128))]
#![no_std]

use core::convert::Infallible;
use core::marker::PhantomData;

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

uninhabited! {
	pub NumZero,
	pub NumNegZero,
	/// Positive or unsigned.
	pub NumOne,
	/// Negative.
	pub NumNegOne,
	pub NumMin,
	pub NumMax,
}

/// Const generics do not support floats.
#[derive(Debug)]
pub struct FloatTypeNum<T> {
	_marker: PhantomData<T>,
	_uninhabited: Infallible,
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

impl_float! {
	#[cfg(feature = "f16")] f16,
	f32,
	f64,
	#[cfg(feature = "f128")] f128,
}

macro_rules! gen_reconverged {
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

gen_reconverged! {
	pub Unit: () = (),
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

/// Narrowest types over const-generic.
macro_rules! gen_cg_reconverged {
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
		$vis enum $divergent<const CG_VALUE: $convergent> {}

		impl<const CG_VALUE: $convergent> Sealed for $divergent<CG_VALUE> {}

		impl<const CG_VALUE: $convergent> TypeState<$convergent> for $divergent<CG_VALUE> {
			type State = $divergent<CG_VALUE>;
		}

		impl<const CG_VALUE: $convergent> TypeValue for $divergent<CG_VALUE> {
			type Value = $convergent;

			const VALUE: Self::Value = CG_VALUE;
		}

		impl<const CG_VALUE: $convergent> TypeState<Option<$convergent>> for OptionSome<$divergent<CG_VALUE>> {
			type State = OptionSome<$divergent<CG_VALUE>>;
		}

		impl<const CG_VALUE: $convergent> TypeValue for OptionSome<$divergent<CG_VALUE>> {
			type Value = Option<$convergent>;
			const VALUE: Self::Value = Some(CG_VALUE);
		}


		)+
	};
}

gen_cg_reconverged! {
	pub Bool: bool,
	pub Char: char,
}

macro_rules! gen_cg_reconverged_num {
	(
		$(
			$(#[$meta:meta])*
			$vis:vis $divergent:ident: $convergent:ty
		),+

		$(,)?
	) => {
		gen_cg_reconverged! {
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
		)+
	};
}

macro_rules! gen_cg_reconverged_inum {
	(
		$(
			$(#[$meta:meta])*
			$vis:vis $divergent:ident: $convergent:ty
		),+

		$(,)?
	) => {
		gen_cg_reconverged_num! {
			$(
			$(#[$meta])*
			$vis $divergent: $convergent
			),+
		}

		$(
		impl TypeState<$convergent> for NumNegOne { type State = $divergent<{ -1 }>; }
		)+
	};
}

gen_cg_reconverged_num! {
	pub U8: u8,
	pub U16: u16,
	pub U32: u32,
	pub U64: u64,
	pub U128: u128,
	pub Usize: usize,
}

gen_cg_reconverged_inum! {
	pub I8: i8,
	pub I16: i16,
	pub I32: i32,
	pub I64: i64,
	pub I128: i128,
	pub Isize: isize,
}
