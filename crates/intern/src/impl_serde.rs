use super::Intern;
use serde::{Deserialize, Deserializer, Serialize};
use std::hash::Hash;

impl<T: ?Sized + Serialize> Serialize for Intern<T> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.0.serialize(serializer)
	}
}

impl<'de, T: ?Sized + Eq + Hash> Deserialize<'de> for Intern<T>
where
	T: Deserialize<'de> + Sync,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		Box::<T>::deserialize(deserializer).map(Intern::from_box)
	}
}
