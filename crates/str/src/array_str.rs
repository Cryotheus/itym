//! See [`ArrayStr`].

use core::str::Utf8Error;
use itym_slot::Slot;
use itym_slot::convert::{RawSlotTransparent, RawSlotRef, SlotVisPub};
use itym_util::utransmute;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct ArrayStr<const LEN: usize>(Slot<[u8; LEN]>);

impl<const LEN: usize> ArrayStr<LEN> {
	/// Creates a new string exactly `LEN` bytes in size filled with the ASCII substitution character `\x01A`.
	pub const fn new() -> Self {
		Self(Slot::new_array_fill(0x1A))
	}

	pub const fn from_fill(char: char) -> Self {
		// Self(Slot::fill_char_utf8(char))
		todo!()
	}

	pub const fn into_slot(self) -> Slot<[u8; LEN]> {
		self.0
	}

	pub const fn into_slot_str(self) -> SlotStr<[u8; LEN]> {
		SlotStr(self.0)
	}

	pub const fn push<U>(self, slot: impl RawSlotTransparent<U>) -> SlotStr<([u8; LEN], U)> {
		self.into_slot_str().push(slot)
	}

	pub const fn write_str(&mut self, str: &str) {
		todo!()
	}
}

unsafe impl<const LEN: usize> RawSlotRef<[u8; LEN]> for ArrayStr<LEN> {
	type Visibility = SlotVisPub;

	const OFFSET: usize = 0;
}

unsafe impl<const LEN: usize> RawSlotTransparent<[u8; LEN]> for ArrayStr<LEN> {}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct SlotStr<T>(Slot<T>);

impl<T> SlotStr<T> {
	pub const fn from_slot(slot: Slot<T>) -> Result<Self, Utf8Error> {
		match str::from_utf8(slot.as_slice()) {
			Ok(_) => Ok(Self(slot)),
			Err(error) => Err(error),
		}
	}

	pub const fn push<U>(self, slot: impl RawSlotTransparent<U>) -> SlotStr<(T, U)> {
		let slot_u = unsafe { utransmute!(<_, Slot<U>> slot) };

		SlotStr(self.0.push(slot_u))
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

unsafe impl<T> RawSlotRef<T> for SlotStr<T> {
	type Visibility = SlotVisPub;

	const OFFSET: usize = 0;
}

unsafe impl<T> RawSlotTransparent<T> for SlotStr<T> {}
