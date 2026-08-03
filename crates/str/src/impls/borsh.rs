use crate::slot_str::SlotStr;
use crate::slot_string::SlotString;
use crate::terminated::{AsNulTerminated, AsTerminated};
use borsh::io::{Error as IoError, ErrorKind as IoErrorKind};
use borsh::io::{Read, Write};
use borsh::{BorshDeserialize, BorshSerialize};
use itym_slot::{ForeignSlotInit, Slot};

impl<T> BorshSerialize for AsNulTerminated<T>
where
	T: AsRef<str>,
{
	fn serialize<W: Write>(&self, writer: &mut W) -> borsh::io::Result<()> {
		self.as_str().serialize(writer)
	}
}

impl<T> BorshSerialize for AsTerminated<T>
where
	T: AsRef<str>,
{
	fn serialize<W: Write>(&self, writer: &mut W) -> borsh::io::Result<()> {
		self.as_str().serialize(writer)
	}
}

impl<T> BorshSerialize for SlotStr<T> {
	fn serialize<W: Write>(&self, writer: &mut W) -> borsh::io::Result<()> {
		self.as_str().serialize(writer)
	}
}

impl<T: ForeignSlotInit> BorshDeserialize for SlotStr<T> {
	fn deserialize_reader<R: Read>(reader: &mut R) -> borsh::io::Result<Self> {
		let mut slot = unsafe { Slot::<T>::filled_unchecked(0x1A) };

		reader.read_exact(slot.as_mut_slice())?;
		SlotStr::new(slot).map_err(|_| IoError::new(IoErrorKind::InvalidData, "Expected UTF-8 encoded byte sequence"))
	}
}

impl<T> BorshSerialize for SlotString<T> {
	fn serialize<W: Write>(&self, writer: &mut W) -> borsh::io::Result<()> {
		self.as_str().serialize(writer)
	}
}

impl<T: ForeignSlotInit> BorshDeserialize for SlotString<T> {
	fn deserialize_reader<R: Read>(reader: &mut R) -> borsh::io::Result<Self> {
		let len: usize = u32::deserialize_reader(reader)?
			.try_into()
			.map_err(|_| IoError::new(IoErrorKind::InvalidData, "Casting u32 -> usize"))?;

		if len > SlotString::<T>::CAPACITY {
			return Err(IoError::new(IoErrorKind::InvalidData, "Length exceeds SlotString capacity"));
		}

		let mut slot = unsafe { Slot::<T>::uninit() };

		slot.write_bytes(0x1A, len);
		reader.read_exact(&mut slot.as_mut_slice()[..len])?;

		if str::from_utf8(&slot.as_slice()[..len]).is_err() {
			return Err(IoError::new(IoErrorKind::InvalidData, "Expected UTF-8 encoded byte sequence"));
		}

		Ok(unsafe { SlotString::from_raw_parts(slot, len) })
	}
}
