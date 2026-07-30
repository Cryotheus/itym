use crate::Slot;
use crate::writer::{SlotSeekError, SlotWriteError, SlotWriter};
use std::io::{Cursor, Seek};
use std::io::{Error as IoError, ErrorKind};
use std::io::{SeekFrom, Write};

impl<T> Slot<T> {
	/// Convenience for calling [`Cursor::new`] on [`Slot::as_mut_slice`].
	pub fn cursor(&mut self) -> Cursor<&mut [u8]> {
		Cursor::new(self.as_mut_slice())
	}
}

impl<const LEN: usize> Slot<[u8; LEN]> {
	/// Convenience for calling [`Cursor::new`] on [`Slot::as_mut_array`].
	pub fn cursor_array(&mut self) -> Cursor<&mut [u8; LEN]> {
		Cursor::new(self.as_mut_array())
	}
}

impl<T> Write for SlotWriter<T> {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		let written = self.write_slice(buf);

		if written == buf.len() {
			return Ok(written);
		} else if written == 0 {
			return Err(IoError::new(ErrorKind::WriteZero, SlotWriteError::EndOfBuffer));
		}

		Ok(written)
	}

	fn flush(&mut self) -> std::io::Result<()> {
		Ok(())
	}
}

impl<T> Seek for SlotWriter<T> {
	fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
		match match pos {
			SeekFrom::Start(position) => self.try_seek_from_start(position),
			SeekFrom::End(offset) => self.try_seek_from_end(offset),
			SeekFrom::Current(offset) => self.try_seek(offset),
		} {
			Ok(()) => Ok(self.position() as u64),
			Err(error) => Err(error.into_std_io_error()),
		}
	}
}

impl SlotSeekError {
	fn into_std_io_error(self) -> std::io::Error {
		IoError::new(self.std_io_error_kind(), self)
	}

	fn std_io_error_kind(&self) -> ErrorKind {
		match self {
			Self::CursorOverflow => ErrorKind::UnexpectedEof,
			Self::CursorUnderflow => ErrorKind::InvalidInput,
			Self::IndexOverflow => ErrorKind::Other,
		}
	}
}
