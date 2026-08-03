use crate::slot_string::SlotString;
use crate::terminated::{AsNulTerminated, AsTerminated};
use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::error::Error;
use core::fmt::{Debug, Display, Formatter};
use core::hash::{Hash, Hasher};
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::str::Utf8Error;
use itym_assert::ensure_eq;
use itym_slot::Slot;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct SlotStr<T>(pub(crate) Slot<T>);

impl<T> SlotStr<T> {
	pub const LEN: usize = Slot::<T>::LEN;

	pub const fn new(slot: Slot<T>) -> Result<Self, Utf8Error> {
		if let Err(error) = str::from_utf8(slot.as_slice()) {
			Err(error)
		} else {
			Ok(unsafe { Self::new_unchecked(slot) })
		}
	}

	/// # Safety
	/// Same as [`str::from_utf8_unchecked`].
	pub const unsafe fn new_unchecked(slot: Slot<T>) -> Self {
		Self(slot)
	}

	pub const fn from_str(str: &str) -> Result<Self, SlotStrError> {
		if let Some(slot) = unsafe { Slot::<T>::try_from_slice(str.as_bytes()) } {
			Ok(Self(slot))
		} else {
			Err(SlotStrError::Size)
		}
	}

	/// # Safety
	/// Same as [`Slot::from_slice_unchecked`].
	pub const unsafe fn from_str_unchecked(str: &str) -> Self {
		unsafe { Self::new_unchecked(Slot::<T>::from_slice_unchecked(str.as_bytes())) }
	}

	pub const fn as_bytes(&self) -> &[u8] {
		self.0.as_slice()
	}

	pub const unsafe fn as_bytes_mut(&mut self) -> &mut [u8] {
		self.0.as_mut_slice()
	}

	pub const fn as_slot(&self) -> &Slot<T> {
		&self.0
	}

	pub const unsafe fn as_mut_slot(&mut self) -> &mut Slot<T> {
		&mut self.0
	}

	pub const fn as_str(&self) -> &str {
		unsafe { str::from_utf8_unchecked(self.0.as_slice()) }
	}

	pub const fn as_mut_str(&mut self) -> &mut str {
		unsafe { str::from_utf8_unchecked_mut(self.0.as_mut_slice()) }
	}

	/// See [`AsTerminated`].
	pub const fn as_terminated(&self) -> &AsTerminated<str> {
		AsTerminated::new_ref(self.as_str())
	}

	/// See [`AsTerminated`].
	pub const fn as_terminated_mut(&mut self) -> &mut AsTerminated<str> {
		AsTerminated::new_mut(self.as_mut_str())
	}

	pub const fn into_slot(self) -> Slot<T> {
		self.0
	}

	/// See [`SlotString`].
	pub const fn into_slot_string(self) -> SlotString<T> {
		unsafe { SlotString::from_raw_parts(self.0, Self::LEN) }
	}

	/// The same as [`Self::LEN`].
	pub const fn len(&self) -> usize {
		Self::LEN
	}

	/// See [`Slot::push`].
	pub const fn push<U>(self, item: SlotStr<U>) -> SlotStr<(T, U)> {
		SlotStr(self.0.push(item.0))
	}

	// INLINE: It's unlikely both `push_terminated` and `push_nul_terminated` are used together in the same binary
	// even if they are, the penalty of the nested function call is worse than a second inlining of this fn body
	#[inline(always)]
	const unsafe fn push_term<U>(self, slot_u: SlotStr<U>, len_t: usize, len_u: usize) -> SlotStr<(T, U)> {
		let slot_t = self;
		let mut slot_str = slot_t.push(slot_u);

		if len_t == Self::LEN || len_u == 0 {
			return slot_str;
		}

		let dst = unsafe { slot_str.0.as_mut_ptr().add(len_t) };
		let src = unsafe { slot_str.0.as_ptr().add(Self::LEN) };
		unsafe { ptr::copy(src, dst, len_u) };

		let tail = unsafe { slot_str.0.as_mut_ptr().add(len_t + len_u + 1) };
		unsafe { ptr::write_bytes(tail, 0x1A, SlotStr::<U>::LEN - len_t) };

		slot_str
	}

	/// Adjusts the contents similar to [`AsNulTerminated::<str>::const_push_str`].
	pub const fn push_nul_terminated<U>(self, item: SlotStr<U>) -> SlotStr<(T, U)> {
		let len_t = AsNulTerminated::new_ref(self.as_bytes()).const_len();
		let len_u = AsNulTerminated::new_ref(item.as_bytes()).const_len();

		unsafe { self.push_term(item, len_t, len_u) }
	}

	/// Adjusts the contents similar to [`AsTerminated::<str>::const_push_str`].
	pub const fn push_terminated<U>(self, item: SlotStr<U>) -> SlotStr<(T, U)> {
		let len_t = AsTerminated::new_ref(self.as_bytes()).const_len();
		let len_u = AsTerminated::new_ref(item.as_bytes()).const_len();

		unsafe { self.push_term(item, len_t, len_u) }
	}
}

impl<T, U> SlotStr<(T, U)> {
	pub const fn pop(self) -> (SlotStr<T>, SlotStr<U>) {
		let (t, u) = self.0.pop();

		(SlotStr(t), SlotStr(u))
	}

	pub const fn popped(self) -> SlotStr<T> {
		SlotStr(self.0.popped())
	}
}

impl<const LEN: usize> SlotStr<[u8; LEN]> {
	/// Fills the buffer with the ASCII substitution byte `0x1A` (also known as the `SUB` control character)
	pub const fn new_terminated() -> Self {
		Self(Slot::<[u8; LEN]>::new_array_fill(0x1a))
	}

	pub const fn from_utf8(bytes: &[u8]) -> Result<Self, SlotStrError> {
		ensure_eq!(bytes.len(), LEN, SlotStrError::Size);

		todo!();
	}

	/// Intended for `const` or `static` values.
	///
	/// ```rust
	/// # use itym_str::array_str::ArrayStr;
	/// const VERSION_NAME: ArrayStr<7> = ArrayStr::lit(b"Poutine");
	/// ```
	///
	/// # Panics
	/// Provided bytes must be valid UTF-8.
	pub const fn lit(byte_literal: &'static [u8; LEN]) -> Self {
		let Ok(slot) = SlotStr::new(Slot::lit(byte_literal)) else {
			panic!("Bytes provided to `ArrayStr::lit` must be valid UTF-8")
		};

		slot
	}
}

impl<T> Deref for SlotStr<T> {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.as_str()
	}
}

impl<T> DerefMut for SlotStr<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.as_mut_str()
	}
}

impl<T> Debug for SlotStr<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.debug_tuple("SlotStr").field(&self.as_str()).finish()
	}
}

impl<T> Display for SlotStr<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.write_str(self.as_str())
	}
}

impl<T> Borrow<str> for SlotStr<T> {
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl<T> BorrowMut<str> for SlotStr<T> {
	fn borrow_mut(&mut self) -> &mut str {
		self.as_mut_str()
	}
}

impl<T, Rhs> PartialEq<Rhs> for SlotStr<T>
where
	str: PartialEq<Rhs>,
{
	fn eq(&self, other: &Rhs) -> bool {
		self.as_str().eq(other)
	}
}

impl<T, U> PartialEq<SlotStr<U>> for SlotStr<T> {
	fn eq(&self, other: &SlotStr<U>) -> bool {
		self.as_bytes().eq(other.as_bytes())
	}
}

impl<T> Eq for SlotStr<T> {}

impl<T, U> PartialOrd<SlotStr<U>> for SlotStr<T> {
	fn partial_cmp(&self, other: &SlotStr<U>) -> Option<Ordering> {
		self.as_bytes().partial_cmp(other.as_bytes())
	}
}

impl<T, Rhs> PartialOrd<Rhs> for SlotStr<T>
where
	str: PartialOrd<Rhs>,
{
	fn partial_cmp(&self, other: &Rhs) -> Option<Ordering> {
		self.as_str().partial_cmp(other)
	}
}

impl<T> Ord for SlotStr<T> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.as_bytes().cmp(other.as_bytes())
	}
}

impl<T> Hash for SlotStr<T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.as_bytes().hash(state)
	}
}

#[derive(Debug, Clone, Copy)]
pub enum SlotStrError {
	Size,
	Utf8(Utf8Error),
}

impl Display for SlotStrError {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.write_str(match self {
			Self::Size => "Length must exactly match buffer capacity",
			Self::Utf8(error) => return <Utf8Error as Display>::fmt(error, f),
		})
	}
}

impl Error for SlotStrError {}

impl From<Utf8Error> for SlotStrError {
	fn from(value: Utf8Error) -> Self {
		Self::Utf8(value)
	}
}
