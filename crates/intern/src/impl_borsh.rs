use super::Intern;
use borsh::io::{Read, Write};
use borsh::{BorshDeserialize, BorshSerialize};
use std::hash::Hash;

impl<T: ?Sized + BorshSerialize> BorshSerialize for Intern<T> {
	fn serialize<W: Write>(&self, writer: &mut W) -> borsh::io::Result<()> {
		self.0.serialize(writer)
	}
}

impl<T: ?Sized + Eq + Hash + Sync> BorshDeserialize for Intern<T>
where
	Box<T>: BorshDeserialize,
{
	fn deserialize_reader<R: Read>(reader: &mut R) -> borsh::io::Result<Self> {
		Box::<T>::deserialize_reader::<R>(reader).map(Intern::from_box)
	}
}
