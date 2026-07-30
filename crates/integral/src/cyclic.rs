//! See [`Cyclic`].

use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// Cyclic ordering for integer primitives.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cyclic<T>(T);

impl<T> Cyclic<T> {
	pub fn new(n: T) -> Self {
		Self(n)
	}

	pub fn get(self) -> T {
		self.0
	}
}

impl<T: Display> Display for Cyclic<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		<T as Display>::fmt(&self.0, f)
	}
}

macro_rules! impl_cyclic {
	( $($ty:ty)+ ) => {
		$(
		impl Cyclic<$ty> {
			const MIDPOINT: $ty = <$ty>::MIN.midpoint(<$ty>::MAX);
			const MIDPOINT_HI: $ty = Self::MIDPOINT + 1;

			pub fn add(self, rhs: Self) -> Self {
				Self(rhs.0.wrapping_add(rhs.0))
			}

			pub fn sub(self, rhs: Self) -> Self {
				Self(rhs.0.wrapping_sub(rhs.0))
			}

			#[inline]
			fn midpoint_distance(self) -> $ty {
				#[allow(overlapping_range_endpoints)]
				match self {
					Self(n @ <$ty>::MIN..=Self::MIDPOINT) => Self::MIDPOINT - n,
					Self(n @ Self::MIDPOINT_HI..=<$ty>::MAX) => n - Self::MIDPOINT,

					//for `usize` and `isize`
					#[allow(unreachable_patterns)]
					Self(..=<$ty>::MIN | <$ty>::MAX..) => unreachable!(),
				}
			}
		}

		impl Add for Cyclic<$ty> {
			type Output = Self;

			fn add(self, rhs: Self) -> Self::Output {
				self.add(rhs)
			}
		}

		impl Add<$ty> for Cyclic<$ty> {
			type Output = Self;

			fn add(self, rhs: $ty) -> Self::Output {
				self.add(Self(rhs))
			}
		}

		impl AddAssign for Cyclic<$ty> {
			fn add_assign(&mut self, rhs: Self) {
				*self = self.add(rhs);
			}
		}

		impl AddAssign<$ty> for Cyclic<$ty> {
			fn add_assign(&mut self, rhs: $ty) {
				*self = self.add(Self(rhs));
			}
		}

		impl Sub for Cyclic<$ty> {
			type Output = Self;

			fn sub(self, rhs: Self) -> Self::Output {
				self.sub(rhs)
			}
		}

		impl Sub<$ty> for Cyclic<$ty> {
			type Output = Self;

			fn sub(self, rhs: $ty) -> Self::Output {
				self.sub(Self(rhs))
			}
		}

		impl SubAssign for Cyclic<$ty> {
			fn sub_assign(&mut self, rhs: Self) {
				*self = self.sub(rhs);
			}
		}

		impl SubAssign<$ty> for Cyclic<$ty> {
			fn sub_assign(&mut self, rhs: $ty) {
				*self = self.sub(Self(rhs));
			}
		}

		impl Ord for Cyclic<$ty> {
			#[inline(always)]
			fn cmp(&self, other: &Self) -> Ordering {
				self.midpoint_distance().cmp(&other.midpoint_distance())
			}
		}

		impl PartialOrd for Cyclic<$ty> {
			#[inline(always)]
			fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
				self.midpoint_distance().partial_cmp(&other.midpoint_distance())
			}
		}

		)+
	};
}

//TODO: overflow-safe impls for i*
impl_cyclic!(u8 u16 u32 u64 u128 usize);

// #[cfg(test)]
#[test]
fn test() {
	type Bruh = u8;

	// for n in (Cyclic::<Bruh>::MIDPOINT - 2)..(Cyclic::<Bruh>::MIDPOINT + 2) {
	// 	let cyclic = Cyclic::new(n);
	// 	let bias = cyclic.midpoint_distance();
	//
	// 	println!("{n} => {bias:?}");
	// }

	// let range = (Cyclic::<Bruh>::MIDPOINT - 1)..=(Cyclic::<Bruh>::MIDPOINT + 2);
	let range = <Bruh>::MIN..=<Bruh>::MAX;

	for alfa in range.clone() {
		let ref alfa_cyclic = Cyclic::new(alfa);

		for bravo in range.clone() {
			let ref bravo_cyclic = Cyclic::new(bravo);
			let cmp = alfa_cyclic.cmp(bravo_cyclic);
			let cmp_rev = bravo_cyclic.cmp(alfa_cyclic);

			assert_eq!(cmp, cmp_rev.reverse());
		}
	}

	for a in Bruh::MIN..=(Bruh::MIN + 2) {
		let ref ac = Cyclic::new(a);

		for b in (Bruh::MAX - 2)..=Bruh::MAX {
			let ref bc = Cyclic::new(b);
			let cmp = ac.cmp(bc);
			let cmp_rev = bc.cmp(ac);

			assert_eq!(cmp, cmp_rev.reverse());
		}
	}

	for a in (Bruh::MAX - 2)..=Bruh::MAX {
		let ref ac = Cyclic::new(a);

		for b in Bruh::MIN..=(Bruh::MIN + 2) {
			let ref bc = Cyclic::new(b);
			let cmp = ac.cmp(bc);
			let cmp_rev = bc.cmp(ac);

			assert_eq!(cmp, cmp_rev.reverse());
		}
	}
}
