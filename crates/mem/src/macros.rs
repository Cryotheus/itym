/// Size-checking version of [`utransmute`].
///
/// [`utransmute`]: crate::utransmute
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

			$crate::utransmute::<Src, Dst>(src)
		}

		_utransmute_macro_expansion::<$src, $dst>($expr)
	}};
}
