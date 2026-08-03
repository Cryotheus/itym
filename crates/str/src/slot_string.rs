use crate::error::CapacityError;
use crate::slot_str::SlotStr;
use crate::terminated::AsTerminated;
use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::error::Error;
use core::fmt::{Debug, Display, Formatter};
use core::hash::{Hash, Hasher};
use core::mem::MaybeUninit;
use core::ops::Deref;
use core::str::{FromStr, Utf8Error};
use core::{ptr, slice};
use itym_assert::*;
use itym_slot::Slot;

/// [`SlotStr`] of adjustable length, up to [`Self::CAPACITY`].
#[derive(Clone, Copy)]
pub struct SlotString<T> {
	/// # Safety
	/// Bytes in the range `0..len`
	pub(crate) slot: Slot<T>,

	/// # Safety
	/// Must uphold: `len <= Self::LEN`
	pub(crate) len: usize,
}

impl<T> SlotString<T> {
	pub const CAPACITY: usize = Slot::<T>::LEN;

	pub const unsafe fn from_raw_parts(slot: Slot<T>, len: usize) -> Self {
		Self { slot, len }
	}

	pub const fn as_bytes(&self) -> &[u8] {
		self.slot.as_slice().split_at(self.len).0
	}

	pub const unsafe fn as_bytes_mut(&mut self) -> &mut [u8] {
		self.slot.as_mut_slice().split_at_mut(self.len).0
	}

	pub const fn as_ptr(&self) -> *const u8 {
		self.slot.as_ptr()
	}

	pub const fn as_mut_ptr(&mut self) -> *mut u8 {
		self.slot.as_mut_ptr()
	}

	pub const fn as_str(&self) -> &str {
		unsafe { str::from_utf8_unchecked(self.as_bytes()) }
	}

	pub const fn as_mut_str(&mut self) -> &mut str {
		unsafe { str::from_utf8_unchecked_mut(self.as_bytes_mut()) }
	}

	/// See [`AsTerminated`].
	pub const fn as_terminated(&self) -> &AsTerminated<str> {
		AsTerminated::new_ref(self.as_str())
	}

	/// See [`AsTerminated`].
	pub const fn as_terminated_mut(&mut self) -> &mut AsTerminated<str> {
		AsTerminated::new_mut(self.as_mut_str())
	}

	/// The same as [`Self::CAPACITY`].
	pub const fn capacity(&self) -> usize {
		Self::CAPACITY
	}

	pub const fn clear(&mut self) {
		self.len = 0;
	}

	/// Fills the spare capacity with the terminator byte used by [`AsTerminated`].
	pub const fn into_slot_str(mut self) -> SlotStr<T> {
		let space = Self::CAPACITY - self.len;

		unsafe { ptr::write_bytes(self.spare_capacity_ptr(), 0x1A, space) };
		unsafe { SlotStr::new_unchecked(self.slot) }
	}

	pub const fn is_empty(&self) -> bool {
		self.len == 0
	}

	pub const fn len(&self) -> usize {
		self.len
	}

	pub const fn pop_char(&mut self) -> Option<char> {
		if self.len == 0 {
			return None;
		}

		let char = {
			let bytes = self.as_bytes();
			let end = bytes.len();
			let mut start = end - 1;

			// Find the leading byte of the final UTF-8 code point.
			while let 0b1000_0000 = bytes[start] & 0b1100_0000 {
				start -= 1;
			}

			let code_point = match end - start {
				1 => bytes[start] as u32,

				2 => ((bytes[start] & 0b0001_1111) as u32) << 6 | ((bytes[start + 1] & 0b0011_1111) as u32),

				3 => {
					((bytes[start] & 0b0000_1111) as u32) << 12
						| ((bytes[start + 1] & 0b0011_1111) as u32) << 6
						| ((bytes[start + 2] & 0b0011_1111) as u32)
				}

				4 => {
					((bytes[start] & 0b0000_0111) as u32) << 18
						| ((bytes[start + 1] & 0b0011_1111) as u32) << 12
						| ((bytes[start + 2] & 0b0011_1111) as u32) << 6
						| ((bytes[start + 3] & 0b0011_1111) as u32)
				}

				_ => unsafe { debug_unreachable!() },
			};

			unsafe { char::from_u32_unchecked(code_point) }
		};

		self.len -= char.len_utf8();

		Some(char)
	}

	/// Merges the buffers of the two strings, placing the contents of `SlotString<U>` immediately after the contents of `Self`.
	pub const fn push<U>(self, slot_string: SlotString<U>) -> SlotString<(T, U)> {
		let SlotString { slot: slot_t, len: len_t } = self;
		let SlotString { slot: slot_u, len: len_u } = slot_string;
		let mut slot = slot_t.push(slot_u);

		if len_t == Self::CAPACITY {
			//no shifting needed, we're done
			//this also handles the ZST case where we would have a `*const u8` and `*mut u8` sharing an address
			return SlotString { slot, len: len_t + len_u };
		}

		//TODO: miri this.
		let slot_u_src = unsafe { slot.as_ptr().add(Self::CAPACITY) };
		let slot_u_dst = unsafe { slot.as_mut_ptr().add(len_t) };

		unsafe { ptr::copy(slot_u_src, slot_u_dst, len_u) };
		SlotString { slot, len: len_t + len_u }
	}

	pub const fn push_char(&mut self, char: char) -> Result<(), CapacityError> {
		let space = Self::CAPACITY - self.len;
		let usage = char.len_utf8();

		if let Some(error) = CapacityError::new(space, usage) {
			return Err(error);
		}

		char.encode_utf8(unsafe { self.spare_capacity_mut().assume_init_mut() });

		self.len += char.len_utf8();

		Ok(())
	}

	pub const fn push_slot_str<U>(&mut self, slot_str: SlotStr<U>) -> Result<(), CapacityError> {
		let space = Self::CAPACITY - self.len;
		let usage = SlotStr::<U>::LEN;

		if let Some(error) = CapacityError::new(space, usage) {
			return Err(error);
		}

		unsafe { ptr::write(self.spare_capacity_ptr().cast::<SlotStr<U>>(), slot_str) };

		self.len += SlotStr::<U>::LEN;

		Ok(())
	}

	pub const fn push_str(&mut self, str: &str) -> Result<(), CapacityError> {
		let space = Self::CAPACITY - self.len;
		let usage = str.len();

		if let Some(error) = CapacityError::new(space, usage) {
			return Err(error);
		}

		unsafe { self.spare_capacity_mut().assume_init_mut() }.copy_from_slice(str.as_bytes());

		self.len += usage;

		Ok(())
	}

	pub const fn resize(&mut self, new_len: usize, value: u8) {
		unsafe { self.resize_unchecked(if new_len > Self::CAPACITY { Self::CAPACITY } else { new_len }, value) }
	}

	/// # Safety
	/// Must uphold `new_len <= CAPACITY`
	pub const unsafe fn resize_unchecked(&mut self, new_len: usize, value: u8) {
		let count = new_len.saturating_sub(self.len);

		if count == 0 {
			return;
		}

		unsafe { ptr::write_bytes(self.spare_capacity_ptr(), value, count) }

		self.len = new_len;
	}

	pub const unsafe fn set_len(&mut self, new_len: usize) {
		self.len = new_len;
	}

	pub const fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
		let space = Self::CAPACITY - self.len;

		unsafe { slice::from_raw_parts_mut(self.spare_capacity_ptr().cast::<MaybeUninit<u8>>(), space) }
	}

	const fn spare_capacity_ptr(&mut self) -> *mut u8 {
		unsafe { self.as_mut_ptr().add(self.len) }
	}
}

impl<T, U> SlotString<(T, U)> {
	pub const fn pop(self) -> Result<(SlotString<T>, SlotString<U>), SlotStringError<(T, U)>> {
		//
		todo!()
	}

	pub const fn pop_at(self, mid: usize) -> Result<(SlotString<T>, SlotString<U>), SlotStringError<(T, U)>> {
		ensure!(
			self.as_str().is_char_boundary(mid),
			SlotStringError::new(self, SlotStringErrorKind::CharBoundary(mid))
		);

		todo!()
	}
}

impl<const CAP: usize> SlotString<[u8; CAP]> {
	pub const fn new() -> Self {
		Self {
			slot: unsafe { Slot::uninit() },
			len: 0,
		}
	}

	pub const fn from_str(str: &str) -> Result<Self, CapacityError> {
		let mut string = SlotString::new();
		if let Err(error) = string.push_str(str) { Err(error) } else { Ok(string) }
	}

	pub const fn lit(str: &str) -> Self {
		let Ok(string) = Self::from_str(str) else {
			panic!("SlotString::lit `str` must fit `CAP`")
		};

		string
	}
}

impl<T> Deref for SlotString<T> {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.as_str()
	}
}

impl<T> Debug for SlotString<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.debug_tuple("SlotString").field(&self.as_str()).finish()
	}
}

impl<T> Display for SlotString<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.write_str(self.as_str())
	}
}

impl<const CAP: usize> FromStr for SlotString<[u8; CAP]> {
	type Err = CapacityError;

	fn from_str(str: &str) -> Result<Self, Self::Err> {
		Self::from_str(str)
	}
}

impl<T> AsRef<str> for SlotString<T> {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl<T> AsMut<str> for SlotString<T> {
	fn as_mut(&mut self) -> &mut str {
		self.as_mut_str()
	}
}

impl<T, U> AsRef<U> for SlotString<T>
where
	str: AsRef<U>,
{
	fn as_ref(&self) -> &U {
		self.as_str().as_ref()
	}
}

impl<T, U> AsMut<U> for SlotString<T>
where
	str: AsMut<U>,
{
	fn as_mut(&mut self) -> &mut U {
		self.as_mut_str().as_mut()
	}
}

impl<T> Borrow<str> for SlotString<T> {
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl<T> BorrowMut<str> for SlotString<T> {
	fn borrow_mut(&mut self) -> &mut str {
		self.as_mut_str()
	}
}

impl<T, Rhs> PartialEq<Rhs> for SlotString<T>
where
	str: PartialEq<Rhs>,
{
	fn eq(&self, other: &Rhs) -> bool {
		self.as_str().eq(other)
	}
}

impl<T, U> PartialEq<SlotString<U>> for SlotString<T> {
	fn eq(&self, other: &SlotString<U>) -> bool {
		self.as_bytes().eq(other.as_bytes())
	}
}

impl<T> Eq for SlotString<T> {}

impl<T, U> PartialOrd<SlotString<U>> for SlotString<T> {
	fn partial_cmp(&self, other: &SlotString<U>) -> Option<Ordering> {
		self.as_bytes().partial_cmp(other.as_bytes())
	}
}

impl<T, Rhs> PartialOrd<Rhs> for SlotString<T>
where
	str: PartialOrd<Rhs>,
{
	fn partial_cmp(&self, other: &Rhs) -> Option<Ordering> {
		self.as_str().partial_cmp(other)
	}
}

impl<T> Ord for SlotString<T> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.as_bytes().cmp(other.as_bytes())
	}
}

impl<T> Hash for SlotString<T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.as_bytes().hash(state)
	}
}

/// Returned by fallible functions owning a [`SlotString`].
#[derive(Debug, Clone, Copy)]
pub struct SlotStringError<T> {
	pub kind: SlotStringErrorKind,
	pub slot_string: SlotString<T>,
}

impl<T> SlotStringError<T> {
	pub const fn new(slot_string: SlotString<T>, kind: SlotStringErrorKind) -> Self {
		Self { kind, slot_string }
	}
}

impl<T> Display for SlotStringError<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		Display::fmt(&self.kind, f)
	}
}

impl<T: Debug> Error for SlotStringError<T> {}

#[derive(Debug, Clone, Copy)]
pub enum SlotStringErrorKind {
	/// See [`CapacityError`].
	Capacity(CapacityError),

	/// The specified index was not a character boundary and cannot be split.
	CharBoundary(usize),

	/// See [`Utf8Error`].
	Utf8(Utf8Error),
}

impl Display for SlotStringErrorKind {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		match self {
			Self::Capacity(error) => Display::fmt(error, f),
			Self::CharBoundary(index) => f.write_fmt(format_args!("Split at index {index} not on a character boundary")),
			Self::Utf8(error) => Display::fmt(error, f),
		}
	}
}

impl Error for SlotStringErrorKind {}

impl From<CapacityError> for SlotStringErrorKind {
	fn from(value: CapacityError) -> Self {
		Self::Capacity(value)
	}
}
