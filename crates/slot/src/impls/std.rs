extern crate std;

use crate::Slot;
use crate::writer::{SlotWriteError, SlotWriter};
use std::io::Cursor;
use std::io::Write;
use std::io::{Error as IoError, ErrorKind};

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
		}

		if written == 0 {
			return Err(IoError::new(ErrorKind::WriteZero, SlotWriteError::EndOfBuffer));
		}

		todo!()
	}

	fn flush(&mut self) -> std::io::Result<()> {
		Ok(())
	}
}
