#![no_std]
//! # Itym: Utilities
//! Dependency of other Itym crates.

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
