//! See [`UniqueCouple`].

use std::ops::{Deref, Index};

const fn byte_occupancy(max_value: u128) -> usize {
	let bits = max_value.ilog2() + 1;
	let bytes = bits.div_ceil(8);

	bytes as usize
}

/// A pair of two distinct values of type `T`, or a single value of `T`.
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct OptionalCouple<T>(T);

impl<T> OptionalCouple<T> {
	pub fn as_raw(&self) -> &T {
		&self.0
	}

	pub fn into_raw(self) -> T {
		self.0
	}
}

/// A pair of two distinct values of type `T`.
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct UniqueCouple<T>(T);

impl<T> UniqueCouple<T> {
	pub fn as_raw(&self) -> &T {
		&self.0
	}

	pub fn into_raw(self) -> T {
		self.0
	}
}

#[derive(Debug, thiserror::Error)]
pub enum CouplingError {
	#[error("Cannot couple equivalent values")]
	Equal,

	#[error("High-part encoding overflowed")]
	OverflowHi,

	#[error("Low-part encoding overflowed")]
	OverflowLo,
}

pub trait Coupling {
	type Couple;
}

impl Coupling for (u8, Option<u8>) {
	type Couple = OptionalCouple<u16>;
}

impl Coupling for (u16, Option<u16>) {
	type Couple = OptionalCouple<u32>;
}

impl Coupling for (u32, Option<u32>) {
	type Couple = OptionalCouple<u64>;
}

impl Coupling for (u64, Option<u64>) {
	type Couple = OptionalCouple<u64>;
}

impl Coupling for [u8; 2] {
	type Couple = UniqueCouple<u16>;
}

impl Coupling for [u16; 2] {
	type Couple = UniqueCouple<u32>;
}

impl Coupling for [u32; 2] {
	type Couple = UniqueCouple<u64>;
}

impl Coupling for [u64; 2] {
	type Couple = UniqueCouple<u128>;
}

#[cfg(target_pointer_width = "16")]
impl Coupling for [usize; 2] {
	type Couple = UniqueCouple<u32>;
}

#[cfg(target_pointer_width = "32")]
impl Coupling for [usize; 2] {
	type Couple = UniqueCouple<u64>;
}

#[cfg(target_pointer_width = "64")]
impl Coupling for [usize; 2] {
	type Couple = UniqueCouple<u128>;
}

macro_rules! eval {
	(
		for $ty:ty;

		$(
		const $eval:ident, $max_x:ident, $max_y:ident = $f:ident;
		)+
	) => {
		$(
		const $eval: [$ty; 2] = {
			let mut max_x = 0;
			let mut overflow_x = <$ty>::MAX;

			while overflow_x - max_x != 1 {
				let x = max_x.midpoint(overflow_x);

				match $f(x) {
					Some(_) => max_x = x,
					None => overflow_x = x,
				};
			}

			[
				max_x,
				$f(max_x).unwrap(),
			]
		};

		#[allow(unused)]
		pub const $max_x: $ty = $eval[0];

		#[allow(unused)]
		pub const $max_y: $ty = $eval[1];
		)+
	};
}

macro_rules! gen_seq {
	(
		$(
		$module:ident $($skip:literal)? $ty:ty),+
		$(,)?
	) => {
		$(
		pub(crate) mod $module {
			use super::{CouplingError, UniqueCouple};

			#[inline(always)]
			pub(super) const fn seq(x: $ty) -> $ty {
				if x % 2 == 0 { (x / 2) * (x + 1) } else { x * ((x + 1) / 2) }
			}

			#[inline(always)]
			pub(super) const fn seq_inv(y: $ty) -> $ty {
				((8 * y + 1).isqrt() - 1) / 2
			}

			#[inline]
			pub(super) const fn seq_checked(x: $ty) -> Option<$ty> {
				let Some(plus_one) = x.checked_add(1) else { return None };

				if x % 2 == 0 {
					(x / 2).checked_mul(plus_one)
				} else {
					x.checked_mul(plus_one / 2)
				}
			}

			#[inline]
			pub(super) const fn seq_inv_checked(y: $ty) -> Option<$ty> {
				let Some(y) = y.checked_mul(8) else { return None };
				let Some(y) = y.checked_add(1) else { return None };

				Some((y.isqrt() - 1) / 2)
			}

			eval! {
				for $ty;

				const SAFE, MAX_X, MAX_Y = seq_checked;
				const INVERSE, INVERSE_MAX_Y, INVERSE_MAX_X = seq_inv_checked;
			}

			impl UniqueCouple<$ty> {
				pub const MAX: Self = Self(MAX_Y);
				pub const MIN: Self = Self(0);

				/// Creates a new coupling of unique values.
				/// # Errors
				/// Returns an error if `alfa == bravo` or the computation overflows.
				pub const fn new(mut alfa: $ty, mut bravo: $ty) -> Result<Self, CouplingError> {
					if alfa > bravo {
						core::mem::swap(&mut alfa, &mut bravo);
					} else if alfa == bravo {
						return Err(CouplingError::Equal);
					}

					let Some(pack) = seq_checked(bravo) else { return Err(CouplingError::OverflowHi); };
					let Some(pack) = pack.checked_add(alfa) else { return Err(CouplingError::OverflowLo); };

					Ok(Self(pack))
				}

				/// # Safety
				/// Ordering `greater > lesser` must be upheld.
				/// Resulting coupled value must not overflow.
				#[inline(always)]
				pub const unsafe fn new_unchecked(lesser: $ty, greater: $ty) -> Self {
					Self(seq(greater) + lesser)
				}

				#[doc(alias("uncouple", "unpack"))]
				#[inline]
				pub const fn get(self) -> [$ty; 2] {
					let greater = seq_inv(self.0);
					let lesser = self.0 - greater;

					[lesser, greater]
				}

				#[doc(alias("get_max"))]
				#[inline]
				pub const fn get_greater(self) -> $ty {
					seq_inv(self.0)
				}

				#[doc(alias("get_min"))]
				#[inline]
				pub const fn get_lesser(self) -> $ty {
					self.0 - seq_inv(self.0)
				}
			}

			#[test]
			fn overflow() {
				//make sure checked functions behave the same
				assert_eq!(seq_checked(MAX_X + 1), None, "[Safe] Overflow");
				assert_eq!(seq_checked(MAX_X), Some(MAX_Y), "[Safe] Maxes (checked)");
				assert_eq!(seq(MAX_X), MAX_Y, "[Safe] Maxes");
				assert_eq!(seq_checked(0), Some(seq(0)), "[Safe] Zeroes");

				println!("{} f({MAX_X}) = {MAX_Y}", stringify!($ty));

				let ideal = UniqueCouple::<u128>::new(<$ty>::MAX as u128 - 1, <$ty>::MAX as u128);

				if let Ok(ideal) = ideal {
					println!("\t{ideal:?}");
					println!("\tbytes = {}", super::byte_occupancy(ideal.0 as _));
					println!("\t{}", ideal.0 + <$ty>::MAX as u128);
					println!("\tbytes = {}", super::byte_occupancy(ideal.0 + <$ty>::MAX as u128));
				}

				print!("\n");

				$(if $skip { return; })?

				for x in 0..INVERSE_MAX_X {
					let y = seq(x);
					let x2 = seq_inv(y);

					assert_eq!(x, x2, "Inverse mismatch");
				}
			}
		}
		)+
	};
}

gen_seq! {
	u8 u8,
	u16 u16,
	u32 u32,
	u64 true u64,
	u128 true u128,
	usize true usize,
}

macro_rules! gen_signed {
	(
		$($i:ty = $u:ty),+
		$(,)?
	) => {
		$(
		impl UniqueCouple<$i> {

		}
		)+
	};
}

gen_signed! {
	i8 = u8,
	i16 = u16,
	i32 = u32,
	i64 = u64,
	i128 = u128,
	isize = usize,
}

//
//
// MAX_X u8 22
// MAX_Y u8 253

//
//
// MAX_X u16 361
// MAX_Y u16 65341

//
//
// MAX_X u32 92681
// MAX_Y u32 4294930221

//
//
// MAX_X u64 6074000999
// MAX_Y u64 18446744070963499500

//
//
// MAX_X u128 26087635650665564424
// MAX_Y u128 340282366920938463458179421426580008100

//
//
// MAX_X usize 6074000999
// MAX_Y usize 18446744070963499500
