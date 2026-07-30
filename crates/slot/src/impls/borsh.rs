use crate::{ForeignSlotInit, Slot};
use borsh::io::{Read, Write};
use borsh::{BorshDeserialize, BorshSerialize};

impl<T: ForeignSlotInit> BorshDeserialize for Slot<T> {
	fn deserialize_reader<R: Read>(reader: &mut R) -> borsh::io::Result<Self> {
		// There is not a guarantee that the writer will actually write any bytes,
		// even if it returns `Ok(())`
		// we use `zeroed_unchecked` instead of `uninit` to ensure all bytes are indeed initialized
		let mut slot = unsafe { Slot::<T>::zeroed_unchecked() };

		reader.read_exact(slot.as_mut_slice())?;

		Ok(slot)
	}
}

impl<T> BorshSerialize for Slot<T> {
	fn serialize<W: Write>(&self, writer: &mut W) -> borsh::io::Result<()> {
		writer.write_all(self.as_slice())
	}
}
