use crate::Slot;
use crate::convert::{RawSlotRef, SlotVisPub};
use core::error::Error;
use core::fmt::{Display, Formatter};
use core::mem::offset_of;
use itym_assert::*;

/// Does not support seeking beyond [`Slot::len`].
#[derive(Debug, Copy, Clone)]
pub struct SlotWriter<T> {
	/// Should never be set larger than `slot.len`.
	cursor: usize,
	slot: Slot<T>,
}

impl<T> SlotWriter<T> {
	pub const fn new(slot: Slot<T>) -> Self {
		Self { cursor: 0, slot }
	}

	pub const fn as_mut_ptr(&mut self) -> *mut u8 {
		unsafe { self.slot.as_mut_ptr().add(self.cursor) }
	}

	pub const fn as_mut_slice(&mut self) -> &mut [u8] {
		self.slot.as_mut_slice().split_at_mut(self.cursor).1
	}

	pub const fn as_slice(&self) -> &[u8] {
		self.slot.as_slice().split_at(self.cursor).1
	}

	#[inline(always)]
	const fn clamp_cursor(&mut self) {
		if self.cursor > size!(Slot<T>) {
			self.cursor = size!(Slot<T>);
		}
	}

	/// Cursor maintains same position relative to the start.
	pub const fn buffer_push<U>(self, item: Slot<U>) -> SlotWriter<(T, U)> {
		SlotWriter {
			cursor: self.cursor,
			slot: self.slot.push(item),
		}
	}

	#[doc(alias("finish", "slot"))]
	pub const fn into_inner(self) -> Slot<T> {
		self.slot
	}

	pub const fn position(&self) -> usize {
		self.cursor
	}

	/// Infallible version of [`Self::try_seek`].
	pub const fn seek(&mut self, offset: isize) {
		self.cursor = self.cursor.saturating_add_signed(offset);
		self.clamp_cursor();
	}

	/// As the cursor does not move beyond [`Slot::len`],
	/// the offset is unsigned and only functions as the negative values of [`Self::try_seek_from_end`].
	pub const fn seek_from_end(&mut self, offset: usize) {
		self.cursor = size!(Slot<T>).saturating_sub(offset);
	}

	/// Infallible version of [`Self::try_seek_from_start`].
	pub const fn seek_from_start(&mut self, position: usize) {
		self.cursor = position;
		self.clamp_cursor();
	}

	#[doc(alias("get_ref"))]
	pub const fn slot(&self) -> &Slot<T> {
		&self.slot
	}

	#[doc(alias("get_mut"))]
	pub const fn slot_mut(&mut self) -> &mut Slot<T> {
		&mut self.slot
	}

	/// Functionally similar to [`Seek::seek`] using [`SeekFrom::Current`].
	///
	/// [`Seek::seek`]: [`std::io::Seek::seek`]
	/// [`SeekFrom::Current`]: [`std::io::SeekFrom::Current`]
	pub const fn try_seek(&mut self, offset: i64) -> Result<(), SlotSeekError> {
		let Some(new) = (self.cursor as u64).checked_add_signed(offset) else {
			return Err(if offset.is_negative() {
				SlotSeekError::CursorUnderflow
			} else {
				SlotSeekError::CursorOverflow
			});
		};

		self.try_seek_from_start(new)
	}

	/// Functionally similar to [`Seek::seek`] using [`SeekFrom::End`].
	///
	/// [`Seek::seek`]: [`std::io::Seek::seek`]
	/// [`SeekFrom::End`]: [`std::io::SeekFrom::End`]
	pub const fn try_seek_from_end(&mut self, offset: i64) -> Result<(), SlotSeekError> {
		let Some(new) = { size!(Slot<T>) as u64 }.checked_add_signed(offset) else {
			return Err(if offset.is_negative() {
				SlotSeekError::CursorUnderflow
			} else {
				SlotSeekError::CursorOverflow
			});
		};

		self.try_seek_from_start(new)
	}

	/// Functionally similar to [`Seek::seek`] using [`SeekFrom::Start`].
	///
	/// [`Seek::seek`]: [`std::io::Seek::seek`]
	/// [`SeekFrom::Start`]: [`std::io::SeekFrom::Start`]
	pub const fn try_seek_from_start(&mut self, position: u64) -> Result<(), SlotSeekError> {
		if position > (isize::MAX as u64) {
			return Err(SlotSeekError::IndexOverflow);
		}

		let position = position as usize;

		if position > size!(Slot<T>) {
			return Err(SlotSeekError::CursorOverflow);
		}

		self.cursor = position;

		Ok(())
	}

	/// # Panics
	/// If the buffer would overflow.
	pub const fn write_byte(&mut self, value: u8) {
		self.slot.write_byte_at(self.cursor, value);
		self.cursor += 1;
	}

	/// # Panics
	/// If the buffer overflowed.
	pub const fn write_char(&mut self, char: char) {
		let mut encoded = [0u8; char::MAX_LEN_UTF8];
		let written = self.write_slice(char.encode_utf8(&mut encoded).as_bytes());

		const_assert_eq!(written, char.len_utf8());
	}

	/// Returns how many bytes were written.
	/// Never panics, but can write nothing if the cursor has reached the end of the buffer.
	pub const fn write_slice(&mut self, src: &[u8]) -> usize {
		let space = size!(Slot<T>) - self.cursor;

		if src.len() > space {
			self.slot.write_slice_at(self.cursor, src.split_at(space).0);
			self.cursor = size!(Slot<T>);

			return space;
		}

		self.slot.write_slice_at(self.cursor, src);
		self.cursor += src.len();
		src.len()
	}
}

impl<T, U> SlotWriter<(T, U)> {
	/// If the cursor would rest beyond [`Slot<T>::len`], the cursor is set to the end of the buffer.
	pub const fn buffer_pop(self) -> (SlotWriter<T>, Slot<U>) {
		let Self { cursor, slot } = self;
		let (t, u) = slot.pop();
		let mut writer = SlotWriter { cursor, slot: t };

		if size_ne!(Self, Slot<T>) {
			writer.clamp_cursor();
		}

		(writer, u)
	}

	pub const fn buffer_popped(self) -> SlotWriter<T> {
		let mut writer = SlotWriter {
			cursor: self.cursor,
			slot: self.slot.popped(),
		};

		if size_ne!(Self, Slot<T>) {
			writer.clamp_cursor();
		}

		writer
	}
}

unsafe impl<T> RawSlotRef<T> for SlotWriter<T> {
	type Visibility = SlotVisPub;

	const OFFSET: usize = offset_of!(SlotWriter<T>, slot);
}

#[derive(Debug, Clone, Copy)]
pub enum SlotSeekError {
	/// Cursor set to a position beyond [`Slot::len`].
	CursorOverflow,

	/// Cursor set to a position before byte 0.
	CursorUnderflow,

	/// When the cursor is set beyond [`isize::MAX`].
	IndexOverflow,
}

impl Display for SlotSeekError {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.write_str(match self {
			Self::CursorOverflow => "Cursor overflows past buffer which is not allowed for a SlotWriter",
			Self::CursorUnderflow => "Cursor position is before byte 0 which is not allowed for writeable objects",
			Self::IndexOverflow => "Position provided cannot be used with host system's addressing space",
		})
	}
}

impl Error for SlotSeekError {}

#[derive(Debug, Clone, Copy)]
pub enum SlotWriteError {
	EndOfBuffer,

	/// See [`SlotSeekError`].
	Seek(SlotSeekError),
}

impl Display for SlotWriteError {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.write_str(match self {
			SlotWriteError::EndOfBuffer => "End of buffer",
			SlotWriteError::Seek(error) => return <SlotSeekError as Display>::fmt(error, f),
		})
	}
}

impl Error for SlotWriteError {}

impl From<SlotSeekError> for SlotWriteError {
	fn from(error: SlotSeekError) -> Self {
		Self::Seek(error)
	}
}
