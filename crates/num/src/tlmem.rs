//! See [`TlmLayout`].

use core::alloc::Layout;
use core::fmt::{Display, Formatter};
use core::hint::assert_unchecked;
use core::num::NonZero;
use itym_assert::*;

/// Top-level memory layout, in the form of an encoded [`TlmAlign`] and [`TlmSize`] pair.
///
/// Alignment will always be less or equal to size,
/// as padding padding is considered to be part of the layout's size.
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct TlmLayout(usize);

impl TlmLayout {
	/// # Panics
	/// On overflow.
	pub const fn new(size: TlmSize, align: TlmAlign) -> Self {
		let Ok(encoded) = (unsafe { encode_unchecked(size.0.get(), align.0.get()) }) else {
			panic!()
		};

		Self(encoded)
	}

	pub const fn of<T: Sized>() -> Self {
		let Ok(layout) = Self::try_from_size_align(size_of::<T>(), align_of::<T>()) else {
			panic!()
		};

		layout
	}

	pub const fn try_from_layout(layout: Layout) -> Result<Self, TlmError> {
		let align = layout.align();
		let size = layout.size();

		ensure_ne!(size, 0, TlmError::ZeroSize);
		ensure!(align <= size, TlmError::OddAlign);

		// SAFETY: power of 2 rule is upheld by `Layout`
		match unsafe { encode_unchecked(size, align) } {
			Ok(packed) => Ok(Self(packed)),
			Err(error) => Err(error),
		}
	}

	pub const fn try_from_size_align(size: usize, align: usize) -> Result<Self, TlmError> {
		ensure_eq!(align.count_ones(), 1, TlmError::OddAlign);
		ensure_ne!(size, 0, TlmError::ZeroSize);
		ensure!(align <= size, TlmError::OddAlign);

		match unsafe { encode_unchecked(size, align) } {
			Ok(packed) => Ok(Self(packed)),
			Err(error) => Err(error),
		}
	}

	pub const fn into_raw(self) -> usize {
		self.0
	}

	pub const fn size_align(&self) -> (TlmSize, TlmAlign) {
		let (size, align) = decode(self.0);

		unsafe { (TlmSize::new_unchecked(size), TlmAlign::new_unchecked(align)) }
	}

	pub const fn align(&self) -> TlmAlign {
		self.size_align().1
	}

	pub const fn size(&self) -> TlmSize {
		self.size_align().0
	}
}

/// Byte alignment in a top-level memory layout.
/// Always a power of two.
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct TlmAlign(NonZero<usize>);

impl TlmAlign {
	pub const fn new(align: usize) -> Result<Self, TlmError> {
		ensure!(align.count_ones() == 1, TlmError::OddAlign);

		match NonZero::new(align) {
			None => Err(TlmError::ZeroAlign),
			Some(nz) => Ok(Self(nz)),
		}
	}

	pub const unsafe fn new_unchecked(align: usize) -> Self {
		TlmAlign(unsafe { NonZero::new_unchecked(align) })
	}

	pub const fn into_raw(self) -> NonZero<usize> {
		self.0
	}

	pub const fn lower(&self) -> Option<Self> {
		unsafe { assert_unchecked(self.0.get().count_ones() == 1) };

		match self.0.get() {
			0 => unreachable!(),
			1 => None,
			pow => Some(unsafe { TlmAlign::new_unchecked(pow >> 1) }),
		}
	}
}

/// Size in a top-level memory layout.
/// Never zero.
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct TlmSize(pub NonZero<usize>);

impl TlmSize {
	pub const fn new(size: usize) -> Result<Self, TlmError> {
		match NonZero::new(size) {
			None => Err(TlmError::ZeroSize),
			Some(nz) => Ok(Self(nz)),
		}
	}

	pub const unsafe fn new_unchecked(size: usize) -> Self {
		Self(unsafe { NonZero::new_unchecked(size) })
	}

	pub const fn into_raw(self) -> NonZero<usize> {
		self.0
	}

	/// Maximum alignment that does not require a change of size.
	pub const fn max_align(&self) -> TlmAlign {
		unsafe { TlmAlign::new_unchecked(1usize << self.0.get().trailing_zeros()) }
	}
}

#[derive(Debug, Clone, Copy)]
pub enum TlmError {
	OddAlign,
	ExcessiveAlign,
	ZeroAlign,
	ZeroSize,
	EncodeOverflow,
}

impl Display for TlmError {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.write_str(match self {
			Self::OddAlign => "Alignment must a power of 2",
			Self::ExcessiveAlign => "Alignment can not be larger than size",
			Self::ZeroAlign => "Alignment cannot be zero",
			Self::ZeroSize => "Size cannot be zero",
			Self::EncodeOverflow => "Overflow during encode computation",
		})
	}
}

/// Number of encoded entries occupied by majors `1..=major`.
const fn entries_through(major: usize) -> Option<usize> {
	// Equivalent to `2 * major - major.count_ones()`, but permits
	// detecting overflow without first calculating `2 * major`.
	major.checked_add(major - major.count_ones() as usize)
}

/// # Panics
/// On overflow.
const fn decode(encoded: usize) -> (usize, usize) {
	// Find the number of complete major groups preceding `encoded`.
	let mut low = 0usize;
	let mut high = encoded;

	while low < high {
		let distance = high - low;
		let middle = low + distance / 2 + distance % 2;

		match entries_through(middle) {
			Some(end) if end <= encoded => low = middle,
			Some(..) | None => high = middle - 1,
		}
	}

	let completed_majors = low;

	let Some(start) = entries_through(completed_majors) else {
		unreachable!()
	};

	let exponent = encoded - start;

	let Some(major) = completed_majors.checked_add(1) else { unreachable!() };

	if exponent >= usize::BITS as usize {
		panic!("decoded minor overflow");
	}

	(major, 1usize << exponent)
}

/// # Safety
/// See assertions inside `fn` body.
const unsafe fn encode_unchecked(size: usize, align: usize) -> Result<usize, TlmError> {
	unsafe { assert_unchecked(align.count_ones() == 1) };
	unsafe { assert_unchecked(size != 0) };
	unsafe { assert_unchecked(align <= size) };

	let Some(start) = entries_through(size - 1) else {
		return Err(TlmError::EncodeOverflow);
	};

	let Some(encoded) = start.checked_add(align.trailing_zeros() as usize) else {
		return Err(TlmError::EncodeOverflow);
	};

	Ok(encoded)
}
