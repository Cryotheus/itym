use crate::slot_str::SlotStr;
use crate::slot_string::SlotString;
use crate::terminated::{AsNulTerminated, AsTerminated};
use core::fmt::{Display, Formatter};
use itym_assert::*;
use itym_slot::{ForeignSlotInit, Slot};
use serde::de::{Error as SerdeError, Expected};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl<T> Serialize for AsNulTerminated<T>
where
	T: AsRef<str>,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		self.as_str().serialize(serializer)
	}
}

impl<T> Serialize for AsTerminated<T>
where
	T: AsRef<str>,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		self.as_str().serialize(serializer)
	}
}

impl<T> Serialize for SlotStr<T> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		self.as_str().serialize(serializer)
	}
}

impl<'de, T: ForeignSlotInit> Deserialize<'de> for SlotStr<T> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let src = <&str as Deserialize>::deserialize(deserializer)?;
		let slot =
			unsafe { Slot::<T>::try_from_slice(src.as_bytes()) }.ok_or(SerdeError::invalid_length(src.len(), &LenExactErrorFmt(Slot::<T>::LEN)))?;

		Ok(unsafe { SlotStr::new_unchecked(slot) })
	}
}

impl<T> Serialize for SlotString<T> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		self.as_str().serialize(serializer)
	}
}

impl<'de, T: ForeignSlotInit> Deserialize<'de> for SlotString<T> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let src = <&str as Deserialize>::deserialize(deserializer)?;

		refuse!(
			src.len() > Slot::<T>::LEN,
			SerdeError::invalid_length(src.len(), &LenCapErrorFmt(Self::CAPACITY))
		);

		let mut slot = unsafe { Slot::<T>::uninit() };

		slot.write_slice(src.as_bytes());
		Ok(unsafe { SlotString::<T>::from_raw_parts(slot, src.len()) })
	}
}

struct LenCapErrorFmt(usize);

impl Expected for LenCapErrorFmt {
	fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
		f.write_str("expected up to ")?;
		<usize as Display>::fmt(&self.0, f)?;
		f.write_str(" byte(s)")
	}
}

struct LenExactErrorFmt(usize);

impl Expected for LenExactErrorFmt {
	fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
		f.write_str("expected exactly ")?;
		<usize as Display>::fmt(&self.0, f)?;
		f.write_str(" byte(s)")
	}
}
