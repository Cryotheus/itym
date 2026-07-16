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
	pub fn from_canon(canon: <Self as Pack>::Canonical) -> Option<Self> {
		if <Self as Pack>::CANON_MIN <= canon && canon <= <Self as Pack>::CANON_MAX {
			Some(Self(canon))
		} else {
			None
		}
	}
}

/// A completely unaligned unsigned integer.
/// The size of the integer matches the `BYTES`, and the alignment is always `1`.
pub struct Unalign<const BYTES: usize>(<Unsigned<BYTES> as Pack>::Bytes)
where
	Unsigned<BYTES>: Pack;

impl<const BYTES: usize> Unalign<BYTES> where Unsigned<BYTES>: Pack {}

/// An unsigned integer using lower alignment to eliminate padding.
/// The size of the integer matches the `BYTES`,
/// and the alignment is the highest it can be without introducting padding.
pub struct Unpad<const BYTES: usize>(<Unsigned<BYTES> as Pack>::UnpadRepr)
where
	Unsigned<BYTES>: Pack;

impl<const BYTES: usize> Unpad<BYTES> where Unsigned<BYTES>: Pack {}

pub trait Pack {
	const SIZE: usize;

	const BYTES_MIN: Self::Bytes;
	const BYTES_MAX: Self::Bytes;

	const CANON_MIN: Self::Canonical;
	const CANON_MAX: Self::Canonical;

	type Canonical: Copy + Ord;
	type Bytes;
	type UnpadRepr;
}

macro_rules! impl_unsigned {
	(
		$(
		$bytes:literal =>
			unpad: $unpad:ty,
			canon: $canon:ty,
			$(test: $test:ident,)?
		)+
	) => {
		$(
		impl Pack for Unsigned<$bytes> {
			const SIZE: usize = $bytes;

			const BYTES_MIN: Self::Bytes = [0u8; $bytes];
			const BYTES_MAX: Self::Bytes = [255u8; $bytes];

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
			type Bytes = [u8; $bytes];
			type UnpadRepr = [$unpad; {$bytes / ::core::mem::size_of::<$unpad>()}];
		}

		impl Unsigned<$bytes>
		where
			Self: Pack,
		{
			pub const fn from_canon_const(canon: $canon) -> Option<Self> {
				match canon {
					<Self as Pack>::CANON_MIN ..= <Self as Pack>::CANON_MAX => Some(Self(canon)),

					#[allow(unreachable_patterns)]
					_ => None,
				}
			}
		}

		impl Unalign<$bytes> where Unsigned<$bytes>: Pack {
			pub const fn from_canon(canon: $canon) -> Option<Self> {
				const CANON_SIZE: usize = ::core::mem::size_of::<$canon>();
				const OVERFLOW: Overflow = [0; _];

				type Overflow = [u8; CANON_SIZE - $bytes];

				#[derive(Clone, Copy)]
				#[repr(C)]
				struct Bytes {
					#[cfg(target_endian = "big")]
					overflow: Overflow,

					fit: [u8; $bytes],

					#[cfg(target_endian = "little")]
					overflow: Overflow,
				}

				#[derive(Clone, Copy)]
				#[repr(C)]
				union Construct {
					canon: $canon,
					bytes: Bytes,
				}

				const _: () = {
					["Construct union size"][CANON_SIZE - ::core::mem::size_of::<Construct>()];
					["Bytes struct alignment"][1 - ::core::mem::align_of::<Bytes>()];
					["Construct union alignment"][::core::mem::align_of::<$canon>() - ::core::mem::align_of::<Construct>()];

					()
				};

				match unsafe { Construct { canon }.bytes } {
					Bytes { fit, overflow: OVERFLOW } => Some(Self(fit)),

					#[allow(unreachable_patterns)]
					_ => None,
				}
			}
		}

		impl Unpad<$bytes> where Unsigned<$bytes>: Pack {
			pub fn from_canon(_canon: $canon) -> Option<Self> {
				todo!()
			}
		}



		/// Compile-time assertion, so I can sleep at night.
		const _: () = {
			const UNPAD_TY_ALIGN: usize = ::core::mem::align_of::<$unpad>();
			const UNPAD_IMPL_ALIGN: usize = ::core::mem::align_of::<<Unsigned<$bytes> as Pack>::UnpadRepr>();
			const UNPAD_IMPL_SIZE: usize = ::core::mem::size_of::<<Unsigned<$bytes> as Pack>::UnpadRepr>();

			["Alignment of unaligned type failed assertion"][1 - ::core::mem::align_of::<<Unsigned<$bytes> as Pack>::Bytes>()];
			["Alignment of unpadded type failed assertion"][UNPAD_TY_ALIGN - UNPAD_IMPL_ALIGN];
			["Padding of unpadded type failed assertion"][UNPAD_IMPL_SIZE % UNPAD_IMPL_ALIGN];

			()
		};

		$(
		#[test]
		fn $test() -> ::std::io::Result<()> {
			use ::std::io::Write;

			let mut stdout = std::io::stdout().lock();

			::core::writeln!(stdout, "impl Pack for {} {{", ::core::stringify!(Unsigned<$bytes>))?;
			::core::writeln!(stdout, "\tconst SIZE = {};\n", <Unsigned<$bytes> as Pack>::SIZE)?;
			::core::writeln!(stdout, "\tconst BYTES_MIN = {:?};", <Unsigned<$bytes> as Pack>::BYTES_MIN)?;
			::core::writeln!(stdout, "\tconst BYTES_MAX = {:?};\n", <Unsigned<$bytes> as Pack>::BYTES_MAX)?;
			::core::writeln!(stdout, "\tconst CANON_MIN = {};", <Unsigned<$bytes> as Pack>::CANON_MIN)?;
			::core::writeln!(stdout, "\tconst CANON_MAX = {};\n", <Unsigned<$bytes> as Pack>::CANON_MAX)?;
			::core::writeln!(stdout, "\ttype Canonical = {};", ::core::any::type_name::<<Unsigned<$bytes> as Pack>::Canonical>())?;
			::core::writeln!(stdout, "\ttype Bytes = {};", ::core::any::type_name::<<Unsigned<$bytes> as Pack>::Bytes>())?;
			::core::writeln!(stdout, "\ttype UnpadRepr = {};", ::core::any::type_name::<<Unsigned<$bytes> as Pack>::UnpadRepr>())?;
			::core::writeln!(stdout, "}}")?;
			::core::mem::drop(stdout);

			Ok(())
		}
		)?

		)+
	};
}

impl_unsigned! {
	1 => unpad: u8, canon: u8, test: pack_1,
	2 => unpad: u16, canon: u16, test: pack_2,
	3 => unpad: u8, canon: u32, test: pack_3,
	4 => unpad: u32, canon: u32, test: pack_4,
	5 => unpad: u8, canon: u64, test: pack_5,
	6 => unpad: u16, canon: u64, test: pack_6,
	7 => unpad: u8, canon: u64, test: pack_7,
	8 => unpad: u64, canon: u64, test: pack_8,
	9 => unpad: u8, canon: u128, test: pack_9,
	10 => unpad: u16, canon: u128, test: pack_10,
	11 => unpad: u8, canon: u128, test: pack_11,
	12 => unpad: u32, canon: u128, test: pack_12,
	13 => unpad: u8, canon: u128, test: pack_13,
	14 => unpad: u16, canon: u128, test: pack_14,
	15 => unpad: u8, canon: u128, test: pack_15,
	16 => unpad: u128, canon: u128, test: pack_16,
}
