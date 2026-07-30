use crate::{ForeignSlotInit, Slot};
use core::fmt::{Display, Formatter};
use core::marker::PhantomData;
use serde::de::{Error as SerdeError, Expected};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl<'de, T: ForeignSlotInit> Deserialize<'de> for Slot<T> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let src = <&[u8] as Deserialize>::deserialize(deserializer)?;
		unsafe { Self::try_from_slice(src) }.ok_or(SerdeError::invalid_length(src.len(), &LenErrorFmt::<Slot<T>>(PhantomData)))
	}
}

impl<T> Serialize for Slot<T> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_bytes(self.as_slice())
	}
}

struct LenErrorFmt<T>(PhantomData<T>);

impl<T> Expected for LenErrorFmt<T> {
	fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
		f.write_str("expected exactly ")?;
		<usize as Display>::fmt(&size!(Slot<T>), f)?;
		f.write_str(" byte(s)")
	}
}
