extern crate alloc;

use crate::slot_str::SlotStr;
use crate::slot_string::SlotString;
use alloc::boxed::Box;
use alloc::string::String;
use core::ptr;
use itym_slot::Slot;

impl<T> SlotStr<T> {
	pub fn into_boxed_str(self) -> Box<str> {
		let boxed = self.0.into_boxed_slice();
		let alloc = Box::into_raw(boxed);

		unsafe { Box::from_raw(alloc as *mut str) }
	}

	pub fn into_string(self) -> String {
		unsafe { String::from_utf8_unchecked(self.0.into_vec()) }
	}

	pub fn into_string_exact(self) -> String {
		unsafe { String::from_utf8_unchecked(self.0.into_vec_exact()) }
	}
}

impl<T> SlotString<T> {
	pub fn to_boxed_str(&self) -> Box<str> {
		let mut boxed = unsafe { Box::<[u8]>::new_uninit_slice(self.len).assume_init() };

		unsafe { ptr::copy_nonoverlapping(self.slot.as_ptr(), boxed.as_mut_ptr(), self.len) };

		let alloc = Box::into_raw(boxed);

		unsafe { Box::from_raw(alloc as *mut str) }
	}

	pub fn to_string(&self) -> String {
		let boxed = Box::<Slot<T>>::new_uninit();
		let alloc = Box::into_raw(boxed).cast::<u8>();

		unsafe { ptr::copy_nonoverlapping(self.slot.as_ptr(), alloc, self.len) };
		unsafe { String::from_raw_parts(alloc.cast(), self.len, Self::CAPACITY) }
	}
}
