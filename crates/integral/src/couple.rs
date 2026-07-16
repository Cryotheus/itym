//! See [`Couple`].

/// A pair of two distinct values of type `T`.
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Couple<T>(T);

impl<T> Couple<T> {
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
	type Fit;
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

		pub const $max_x: $ty = $eval[0];
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
			use super::{Couple, CouplingError};

			pub const fn seq(x: $ty) -> $ty {
				if x % 2 == 0 { (x / 2) * (x + 1) } else { x * ((x + 1) / 2) }
			}

			pub const fn seq_fast(n: $ty) -> $ty {
				n * (n + 1) / 2
			}

			pub const fn seq_inv(y: $ty) -> $ty {
				((8 * y + 1).isqrt() - 1) / 2
			}

			pub const fn seq_checked(x: $ty) -> Option<$ty> {
				let Some(plus_one) = x.checked_add(1) else { return None };

				if x % 2 == 0 {
					(x / 2).checked_mul(plus_one)
				} else {
					x.checked_mul(plus_one / 2)
				}
			}

			pub const fn seq_fast_checked(x: $ty) -> Option<$ty> {
				let Some(y) = x.checked_add(1) else { return None };
				let Some(y) = x.checked_mul(y) else { return None };

				Some(y / 2)
			}

			pub const fn seq_inv_checked(y: $ty) -> Option<$ty> {
				let Some(y) = y.checked_mul(8) else { return None };
				let Some(y) = y.checked_add(1) else { return None };

				Some((y.isqrt() - 1) / 2)
			}

			eval! {
				for $ty;

				const SAFE, MAX_X, MAX_Y = seq_checked;
				const FAST, MAX_FAST_X, MAX_FAST_Y = seq_fast_checked;
				const INVERSE, INVERSE_MAX_Y, INVERSE_MAX_X = seq_inv_checked;
			}

			impl Couple<$ty> {
				pub const MAX: Self = Self(MAX_Y);
				pub const MIN: Self = Self(0);

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
			}

			#[test]
			fn overflow() {
				//make sure checked functions behave the same
				assert_eq!(seq_checked(MAX_X + 1), None, "[Safe] Overflow");
				assert_eq!(seq_checked(MAX_X), Some(MAX_Y), "[Safe] Maxes (checked)");
				assert_eq!(seq(MAX_X), MAX_Y, "[Safe] Maxes");
				assert_eq!(seq_checked(0), Some(seq(0)), "[Safe] Zeroes");

				//same checks for the fast variant
				assert_eq!(seq_fast_checked(MAX_FAST_X + 1), None, "[Fast] Overflow");
				assert_eq!(seq_fast_checked(MAX_FAST_X), Some(MAX_FAST_Y), "[Fast] Maxes (checked)");
				assert_eq!(seq_fast(MAX_FAST_X), MAX_FAST_Y, "[Fast] Maxes");
				assert_eq!(seq_fast_checked(0), Some(seq_fast(0)), "[Fast] Zeroes");

				// if <$ty>::MAX.ilog2() > 32 $(|| $skip)? { return; }
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

#[cfg(test)]
// #[test]
fn test() {
	dbg!(u16::MAX_X);
	dbg!(u16::MAX_Y);

	for x in 0..10 {
		let y = u16::seq(x);

		println!("x {x} = y {y}");
	}

	print!("\n");

	for y in 0..10 {
		let x = u16::seq_inv(y);

		println!("y {y} = x {x}");
	}

	for alfa in 0..5 {
		for bravo in 0..5 {
			let couple = Couple::<u32>::new(alfa, bravo);

			println!("{alfa}, {bravo} => {couple:?}");
		}
	}
}
