use core::hash::Hash;

/// Prevent foreign implementations of traits.
trait Sealed {}

/// Const-eval time checked version of [`core::mem::transmute`].
#[inline(always)]
const unsafe fn transmute2<Src, Dst>(src: Src) -> Dst {
	const {
		["transmute2 size assertion"][size_of::<Src>() - size_of::<Dst>()];
		["transmute2 align assertion"][!align_of::<Src>().is_multiple_of(align_of::<Dst>()) as usize];
	};

	unsafe { transmute2_unchecked(src) }
}

/// Const-eval time checked version of [`core::mem::transmute`].
#[inline(always)]
const unsafe fn transmute2_unchecked<Src, Dst>(src: Src) -> Dst {
	use std::mem::ManuallyDrop;

	#[cfg(debug_assertions)]
	if const { size_of::<Src>() != size_of::<Dst>() } {
		panic!();
	}

	union Transmute<Src, Dst> {
		src: ManuallyDrop<Src>,
		dst: ManuallyDrop<Dst>,
	}

	ManuallyDrop::into_inner(unsafe {
		Transmute::<Src, Dst> {
			src: ManuallyDrop::<Src>::new(src),
		}
		.dst
	})
}

/// Contains the primitive unsigned integer type constrained to utilizing a finite amount of bytes.
/// The actual size is equivalent to the smallest primitive that can all fit variants possible for the given amount of bytes.
/// For sizes which exactly match `BYTES`, use [`Unalign`] or [`Unpad`].
pub struct Unsigned<const BYTES: usize>(<Self as Pack>::Canonical)
where
	Self: Pack;

impl<const BYTES: usize> Unsigned<BYTES>
where
	Self: Pack,
{
	pub const fn as_raw(&self) -> &<Self as Pack>::Canonical {
		&self.0
	}

	pub const fn as_raw_mut(&mut self) -> &mut <Self as Pack>::Canonical {
		&mut self.0
	}

	pub const fn into_raw(self) -> <Self as Pack>::Canonical {
		self.0
	}

	#[inline(always)]
	const fn map_generic<const MAPPED: usize>(self) -> Unsigned<MAPPED>
	where
		Unsigned<MAPPED>: Pack,
	{
		#[cfg(debug_assertions)]
		if BYTES != MAPPED {
			panic!();
		}

		unsafe { transmute2_unchecked(self) }
	}

	#[inline(always)]
	const fn map_generic_option<const MAPPED: usize>(option: Option<Self>) -> Option<Unsigned<MAPPED>>
	where
		Unsigned<MAPPED>: Pack,
	{
		match option {
			None => None,
			Some(value) => Some(value.map_generic()),
		}
	}
}

/// A completely unaligned unsigned integer.
/// The size of the integer matches the `BYTES`, and the alignment is always `1`.
#[repr(transparent)]
pub struct Unalign<const BYTES: usize>(<Unsigned<BYTES> as Pack>::UnalignRepr)
where
	Unsigned<BYTES>: Pack;

impl<const BYTES: usize> Unalign<BYTES>
where
	Unsigned<BYTES>: Pack,
{
	#[inline(always)]
	pub const fn new(n: Unsigned<BYTES>) -> Option<Self> {
		Self::from_canon(n.0)
	}

	#[inline(always)]
	pub const fn from_bytes_ne(bytes: [u8; BYTES]) -> Self {
		Self(unsafe { transmute2(bytes) })
	}

	pub const fn as_raw(&self) -> &<Unsigned<BYTES> as Pack>::UnalignRepr {
		&self.0
	}

	pub const fn as_raw_mut(&mut self) -> &mut <Unsigned<BYTES> as Pack>::UnalignRepr {
		&mut self.0
	}

	pub const fn into_raw(self) -> <Unsigned<BYTES> as Pack>::UnalignRepr {
		self.0
	}

	#[inline(always)]
	const fn map_generic<const MAPPED: usize>(self) -> Unalign<MAPPED>
	where
		Unsigned<MAPPED>: Pack,
	{
		#[cfg(debug_assertions)]
		if BYTES != MAPPED {
			panic!();
		}

		unsafe { transmute2_unchecked(self) }
	}

	#[inline(always)]
	const fn map_generic_option<const MAPPED: usize>(option: Option<Self>) -> Option<Unalign<MAPPED>>
	where
		Unsigned<MAPPED>: Pack,
	{
		match option {
			None => None,
			Some(value) => Some(value.map_generic()),
		}
	}
}

impl<const BYTES: usize> Clone for Unalign<BYTES>
where
	Unsigned<BYTES>: Pack,
{
	fn clone(&self) -> Self {
		Self(self.0)
	}
}

impl<const BYTES: usize> Copy for Unalign<BYTES> where Unsigned<BYTES>: Pack {}

/// An unsigned integer using lower alignment to eliminate padding.
/// The size of the integer matches the `BYTES`,
/// and the alignment is the highest it can be without introducting padding.
#[repr(transparent)]
pub struct Unpad<const BYTES: usize>(<Unsigned<BYTES> as Pack>::UnpadRepr)
where
	Unsigned<BYTES>: Pack;

impl<const BYTES: usize> Unpad<BYTES>
where
	Unsigned<BYTES>: Pack,
{
	#[inline(always)]
	pub const fn new(n: Unsigned<BYTES>) -> Option<Self> {
		Self::from_canon(n.0)
	}

	#[inline(always)]
	pub const fn from_bytes_ne(bytes: [u8; BYTES]) -> Self {
		Self(unsafe { transmute2(bytes) })
	}

	pub const fn as_raw(&self) -> &<Unsigned<BYTES> as Pack>::UnpadRepr {
		&self.0
	}

	pub const fn as_raw_mut(&mut self) -> &mut <Unsigned<BYTES> as Pack>::UnpadRepr {
		&mut self.0
	}

	pub const fn into_raw(self) -> <Unsigned<BYTES> as Pack>::UnpadRepr {
		self.0
	}

	pub fn with<F>(&mut self, f: F)
	where
		F: FnMut(&mut <Unsigned<BYTES> as Pack>::Canonical),
	{
		todo!()
	}

	#[inline(always)]
	const fn map_generic<const MAPPED: usize>(self) -> Unpad<MAPPED>
	where
		Unsigned<MAPPED>: Pack,
	{
		#[cfg(debug_assertions)]
		if BYTES != MAPPED {
			panic!();
		}

		unsafe { transmute2_unchecked(self) }
	}

	#[inline(always)]
	const fn map_generic_option<const MAPPED: usize>(option: Option<Self>) -> Option<Unpad<MAPPED>>
	where
		Unsigned<MAPPED>: Pack,
	{
		match option {
			None => None,
			Some(value) => Some(value.map_generic()),
		}
	}
}

impl<const BYTES: usize> Clone for Unpad<BYTES>
where
	Unsigned<BYTES>: Pack,
{
	fn clone(&self) -> Self {
		Self(self.0)
	}
}

impl<const BYTES: usize> Copy for Unpad<BYTES> where Unsigned<BYTES>: Pack {}

#[allow(private_bounds)]
pub trait Pack: Sealed {
	const SIZE: usize;

	const BYTES_MIN: Self::UnalignRepr;
	const BYTES_MAX: Self::UnalignRepr;

	const CANON_MIN: Self::Canonical;
	const CANON_MAX: Self::Canonical;

	type Canonical: Clone + Copy + Eq + Ord + Hash + Sized + Send + Sync + 'static;

	type Unalign;
	type UnalignRepr: Clone + Copy + Eq + Sized + Send + Sync + 'static;
	type UnalignSlot;
	type UnalignTransmute;

	type Unpad;
	type UnpadRepr: Clone + Copy + Eq + Sized + Send + Sync + 'static;
	type UnpadSlot;
	type UnpadTransmute;
}

macro_rules! impl_unsigned {
	(
		$(
		$bytes:literal =>
			unpad: $unpad:ty,
			canon: $canon:ty,
			mod $module:ident,
		)+
	) => {
		$(
		mod $module {
			use super::*;

			const CANON_SIZE: usize = ::core::mem::size_of::<$canon>();
			const OVERFLOW_ZEROES: Overflow = [0; _];

			type Overflow = [u8; CANON_SIZE - $bytes];

			#[doc(hidden)]
			#[derive(Clone, Copy)]
			#[repr(C)]
			pub struct UnalignSlot {
				#[cfg(target_endian = "big")]
				pub(super) overflow: Overflow,

				pub(super) fit: <Unsigned<$bytes> as Pack>::UnalignRepr,

				#[cfg(target_endian = "little")]
				pub(super) overflow: Overflow,
			}

			#[doc(hidden)]
			#[derive(Clone, Copy)]
			#[repr(C)]
			pub union UnalignConstruct {
				pub(super) canon: $canon,
				pub(super) slot: UnalignSlot,
				pub(super) bytes: [u8; CANON_SIZE],
			}

			const _: () = {
				["Construct union size"][CANON_SIZE - ::core::mem::size_of::<UnalignConstruct>()];
				["UnalignRepr struct alignment"][1 - ::core::mem::align_of::<UnalignSlot>()];
				["Construct union alignment"][::core::mem::align_of::<$canon>() - ::core::mem::align_of::<UnalignConstruct>()];

				()
			};

			#[doc(hidden)]
			#[derive(Clone, Copy)]
			#[repr(C)]
			pub struct UnpadSlot {
				#[cfg(target_endian = "big")]
				pub(super) overflow: Overflow,

				pub(super) fit: <Unsigned<$bytes> as Pack>::UnpadRepr,

				#[cfg(target_endian = "little")]
				pub(super) overflow: Overflow,
			}

			#[doc(hidden)]
			#[derive(Clone, Copy)]
			#[repr(C)]
			pub union UnpadConstruct {
				pub(super) canon: $canon,
				pub(super) slot: UnpadSlot,
				pub(super) bytes: [u8; CANON_SIZE],
			}

			const _: () = {
				["Construct union size"][CANON_SIZE - ::core::mem::size_of::<UnpadConstruct>()];
				["UnalignRepr struct alignment"][::core::mem::align_of::<$unpad>() - ::core::mem::align_of::<UnpadSlot>()];
				["Construct union alignment"][::core::mem::align_of::<$canon>() - ::core::mem::align_of::<UnpadConstruct>()];

				()
			};

			impl Sealed for Unsigned<$bytes> {}

			impl Pack for Unsigned<$bytes> {
				const SIZE: usize = $bytes;

				const BYTES_MIN: Self::UnalignRepr = [0u8; $bytes];
				const BYTES_MAX: Self::UnalignRepr = [255u8; $bytes];

				const CANON_MIN: Self::Canonical = <$canon>::from_ne_bytes([0; _]);

				const CANON_MAX: Self::Canonical = {
					let mut bytes = [0u8; _];
					let mut index = 0usize;

					while index < $bytes {
						bytes[index] = 255u8;
						index += 1;
					}

					<$canon>::from_le_bytes(bytes)
				};

				type Canonical = $canon;

				type Unalign = Unalign<$bytes>;
				type UnalignRepr = [u8; $bytes];
				type UnalignSlot = UnalignSlot;
				type UnalignTransmute = UnalignConstruct;

				type Unpad = Unpad<$bytes>;
				type UnpadRepr = [$unpad; {$bytes / ::core::mem::size_of::<$unpad>()}];
				type UnpadSlot = UnpadSlot;
				type UnpadTransmute = UnpadConstruct;
			}

			impl Unsigned<$bytes>
			where
				Self: Pack,
			{
				pub(super) const fn _from_canon(canon: $canon) -> Option<Self> {
					match canon {
						<Self as Pack>::CANON_MIN ..= <Self as Pack>::CANON_MAX => Some(Self(canon)),

						#[allow(unreachable_patterns)]
						_ => None,
					}
				}
			}

			impl Unalign<$bytes> where Unsigned<$bytes>: Pack {
				pub(super) const fn _from_canon(canon: $canon) -> Option<Self> {
					match unsafe { UnalignConstruct { canon }.slot } {
						UnalignSlot { fit, overflow: OVERFLOW_ZEROES } => Some(Self(fit)),

						#[allow(unreachable_patterns)]
						_ => None,
					}
				}

				pub(super) const fn _get(self) -> $canon {
					<$canon>::from_ne_bytes(unsafe { UnalignConstruct { slot: UnalignSlot { fit: self.0, overflow: OVERFLOW_ZEROES } }.bytes })
				}
			}

			impl Unpad<$bytes> where Unsigned<$bytes>: Pack {
				pub(super) const fn _from_canon(canon: $canon) -> Option<Self> {
					match unsafe { UnpadConstruct { canon }.slot } {
						UnpadSlot { fit, overflow: OVERFLOW_ZEROES } => Some(Self(fit)),

						#[allow(unreachable_patterns)]
						_ => None,
					}
				}

				pub(super) const fn _get(self) -> $canon {
					<$canon>::from_ne_bytes(unsafe { UnpadConstruct { slot: UnpadSlot { fit: self.0, overflow: OVERFLOW_ZEROES } }.bytes })
				}
			}

			/// Compile-time assertion, so I can sleep at night.
			const _: () = {
				const UNPAD_TY_ALIGN: usize = ::core::mem::align_of::<$unpad>();
				const UNPAD_IMPL_ALIGN: usize = ::core::mem::align_of::<<Unsigned<$bytes> as Pack>::UnpadRepr>();
				const UNPAD_IMPL_SIZE: usize = ::core::mem::size_of::<<Unsigned<$bytes> as Pack>::UnpadRepr>();

				["Alignment of unaligned type failed assertion"][1 - ::core::mem::align_of::<<Unsigned<$bytes> as Pack>::UnalignRepr>()];
				["Alignment of unpadded type failed assertion"][UNPAD_TY_ALIGN - UNPAD_IMPL_ALIGN];
				["Padding of unpadded type failed assertion"][UNPAD_IMPL_SIZE % UNPAD_IMPL_ALIGN];

				()
			};

			#[test]
			fn pack_layout() -> ::std::io::Result<()> {
				// use ::std::io::Write;
				//
				// let mut stdout = std::io::stdout().lock();
				//
				// ::core::writeln!(stdout, "impl Pack for {} {{", ::core::stringify!(Unsigned<$bytes>))?;
				// ::core::writeln!(stdout, "\tconst SIZE = {};\n", <Unsigned<$bytes> as Pack>::SIZE)?;
				// ::core::writeln!(stdout, "\tconst BYTES_MIN = {:?};", <Unsigned<$bytes> as Pack>::BYTES_MIN)?;
				// ::core::writeln!(stdout, "\tconst BYTES_MAX = {:?};\n", <Unsigned<$bytes> as Pack>::BYTES_MAX)?;
				// ::core::writeln!(stdout, "\tconst CANON_MIN = {};", <Unsigned<$bytes> as Pack>::CANON_MIN)?;
				// ::core::writeln!(stdout, "\tconst CANON_MAX = {};\n", <Unsigned<$bytes> as Pack>::CANON_MAX)?;
				// ::core::writeln!(stdout, "\ttype Canonical = {};", ::core::any::type_name::<<Unsigned<$bytes> as Pack>::Canonical>())?;
				// ::core::writeln!(stdout, "\ttype UnalignRepr = {};", ::core::any::type_name::<<Unsigned<$bytes> as Pack>::UnalignRepr>())?;
				// ::core::writeln!(stdout, "\ttype UnpadRepr = {};", ::core::any::type_name::<<Unsigned<$bytes> as Pack>::UnpadRepr>())?;
				// ::core::writeln!(stdout, "}}")?;
				// ::core::mem::drop(stdout);

				// let canon = Unsigned::<$bytes>::CANON_MAX;
				// let unalign = Unalign::<$bytes>::from_canon(canon);

				Ok(())
			}
		}
		)+

		impl<const BYTES: usize> Unsigned<BYTES>
		where
			Self: Pack,
		{
			pub const fn from_canon(canon: <Self as Pack>::Canonical) -> Option<Self> {
				match const { BYTES } {
					$( $bytes => <Unsigned<$bytes>>::map_generic_option::<BYTES>(<Unsigned<$bytes>>::_from_canon(unsafe { transmute2_unchecked(canon) })), )+
					_ => unreachable!(),
				}
			}

			pub const fn get(self) -> <Self as Pack>::Canonical {
				match const { BYTES } {
					$( $bytes => unsafe { transmute2_unchecked(self.map_generic::<$bytes>().get()) }, )+
					_ => unreachable!(),
				}
			}
		}

		impl<const BYTES: usize> Unalign<BYTES>
		where
			Unsigned<BYTES>: Pack,
		{
			pub const fn from_canon(canon: <Unsigned<BYTES> as Pack>::Canonical) -> Option<Self> {
				match const { BYTES } {
					$( $bytes => <Unalign<$bytes>>::map_generic_option::<BYTES>(<Unalign<$bytes>>::_from_canon(unsafe { transmute2_unchecked(canon) })), )+
					_ => unreachable!(),
				}
			}

			pub const fn get(self) -> <Unsigned<BYTES> as Pack>::Canonical {
				match const { BYTES } {
					$( $bytes => unsafe { transmute2_unchecked(self.map_generic::<$bytes>()._get()) }, )+
					_ => unreachable!(),
				}
			}
		}

		impl<const BYTES: usize> Unpad<BYTES>
		where
			Unsigned<BYTES>: Pack,
		{
			pub const fn from_canon(canon: <Unsigned<BYTES> as Pack>::Canonical) -> Option<Self> {
				match const { BYTES } {
					$( $bytes => <Unpad<$bytes>>::map_generic_option::<BYTES>(<Unpad<$bytes>>::_from_canon(unsafe { transmute2_unchecked(canon) })), )+
					_ => unreachable!(),
				}
			}

			pub const fn get(self) -> <Unsigned<BYTES> as Pack>::Canonical {
				match const { BYTES } {
					$( $bytes => unsafe { transmute2_unchecked(self.map_generic::<$bytes>()._get()) }, )+
					_ => unreachable!(),
				}
			}
		}
	};
}

impl_unsigned! {
	1 => unpad: u8, canon: u8, mod pack_1,
	2 => unpad: u16, canon: u16, mod pack_2,
	3 => unpad: u8, canon: u32, mod pack_3,
	4 => unpad: u32, canon: u32, mod pack_4,
	5 => unpad: u8, canon: u64, mod pack_5,
	6 => unpad: u16, canon: u64, mod pack_6,
	7 => unpad: u8, canon: u64, mod pack_7,
	8 => unpad: u64, canon: u64, mod pack_8,
	9 => unpad: u8, canon: u128, mod pack_9,
	10 => unpad: u16, canon: u128, mod pack_10,
	11 => unpad: u8, canon: u128, mod pack_11,
	12 => unpad: u32, canon: u128, mod pack_12,
	13 => unpad: u8, canon: u128, mod pack_13,
	14 => unpad: u16, canon: u128, mod pack_14,
	15 => unpad: u8, canon: u128, mod pack_15,
	16 => unpad: u128, canon: u128, mod pack_16,
}

#[test]
fn samples() {
	use core::fmt::{Debug, Display};
	use core::iter::from_fn;
	use core::ops::RangeToInclusive;
	use rand::distr::StandardUniform;
	use rand::distr::uniform::{SampleRange, SampleUniform};
	use rand::prelude::*;

	const SAMPLES: usize = 10_000;

	fn sampled<const BYTES: usize>()
	where
		Unsigned<BYTES>: Pack,
		<Unsigned<BYTES> as Pack>::Canonical: Debug + Clone + Copy + Display + SampleUniform + TryInto<usize>,
		<<Unsigned<BYTES> as Pack>::Canonical as TryInto<usize>>::Error: core::error::Error,
		RangeToInclusive<<Unsigned<BYTES> as Pack>::Canonical>: SampleRange<<Unsigned<BYTES> as Pack>::Canonical>,
		StandardUniform: Distribution<<Unsigned<BYTES> as Pack>::Canonical>,
	{
		let mut rng = rand::rng();

		let samples = [<Unsigned<BYTES> as Pack>::CANON_MIN, <Unsigned<BYTES> as Pack>::CANON_MAX]
			.into_iter()
			.chain(
				from_fn(|| {
					let range: RangeToInclusive<<Unsigned<BYTES> as Pack>::Canonical> = ..=<Unsigned<BYTES> as Pack>::CANON_MAX;
					let canon: <Unsigned<BYTES> as Pack>::Canonical = rng.random_range::<<Unsigned<BYTES> as Pack>::Canonical, _>(range);

					Some(canon)
				})
				.take(<Unsigned<BYTES> as Pack>::CANON_MAX.try_into().unwrap_or(usize::MAX).min(SAMPLES)),
			);

		for canon in samples {
			let unalign = Unalign::<BYTES>::from_canon(canon).unwrap();
			let unpad = Unpad::<BYTES>::from_canon(canon).unwrap();

			assert_eq!(canon, unalign.get());
			assert_eq!(canon, unpad.get());
		}
	}

	sampled::<1>();
	sampled::<2>();
	sampled::<3>();
	sampled::<4>();
	sampled::<5>();
	sampled::<6>();
	sampled::<7>();
	sampled::<8>();
	sampled::<9>();
	sampled::<10>();
	sampled::<11>();
	sampled::<12>();
	sampled::<13>();
	sampled::<14>();
	sampled::<15>();
	sampled::<16>();
}
