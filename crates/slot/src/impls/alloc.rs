extern crate alloc;

use crate::Slot;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::mem::MaybeUninit;
use core::ptr::{NonNull, slice_from_raw_parts_mut, write_bytes};
use itym_assert::*;

impl<T> Slot<T> {
	pub fn new_boxed_fill(value: u8) -> Box<Self> {
		const_assert!(size!(Self) < isize::MAX as usize);
		const_assert_eq!(align_of::<Self>(), 1);

		// `Box` does not allocate for ZSTs
		if const { size!(Self) == 0 } {
			return unsafe { Box::new_zeroed().assume_init() };
		}

		let mut boxed = Box::new_uninit();

		unsafe { write_bytes(boxed.as_mut_ptr(), value, 1) };
		unsafe { boxed.assume_init() }
	}

	pub fn new_boxed_zeroed() -> Box<Self> {
		const_assert!(size!(Self) < isize::MAX as usize);
		const_assert_eq!(align_of::<Self>(), 1);

		unsafe { Box::new_zeroed().assume_init() }
	}

	#[must_use = "Memory leak"]
	fn alloc() -> NonNull<MaybeUninit<T>> {
		NonNull::new(Box::into_raw(Box::<T>::new_uninit())).unwrap()
	}

	#[must_use = "Memory leak"]
	fn into_alloc(self) -> NonNull<MaybeUninit<T>> {
		let alloc = Self::alloc();

		unsafe { core::ptr::write(alloc.as_ptr(), self.0) };

		alloc
	}

	#[must_use]
	pub fn into_boxed_slice(self) -> Box<[u8]> {
		unsafe { Box::from_raw(slice_from_raw_parts_mut(self.into_alloc().cast::<u8>().as_ptr(), size!(Self))) }
	}

	#[must_use]
	pub fn into_vec(self) -> Vec<u8> {
		const_debug_assert_eq!(align_of::<Self>(), 1);

		let mut vec = Vec::<u8>::with_capacity(size!(Self));
		let alloc = vec.as_mut_ptr();

		unsafe { core::ptr::write(alloc.cast::<MaybeUninit<T>>(), self.0) };
		unsafe { vec.set_len(size!(Self)) };

		vec
	}

	/// Same as [`Self::into_vec`], but with exact capacity instead of the allocator's recommendation.
	#[must_use]
	pub fn into_vec_exact(self) -> Vec<u8> {
		let alloc = self.into_alloc();

		unsafe { Vec::from_raw_parts(alloc.cast::<u8>().as_ptr(), size!(Self), size!(Self)) }
	}

	/// Convenice for calling [`into_boxed_slice`] and [`Box::leak`].
	pub fn leak(self) -> &'static mut [u8] {
		Box::leak(self.into_boxed_slice())
	}
}

#[test]
fn alloc_layout() {
	macro_rules! gen_tests {
		($len:literal $static_ident:ident) => {
			let slot = Slot::<[u8; $len]>::new_array_zeroed();

			drop(slot.into_boxed_slice());
			drop(slot.into_vec());
			drop(slot.into_vec_exact());

			// Satisfies MIRI.
			static mut $static_ident: Option<&'static [u8]> = None;

			unsafe { $static_ident = Some(slot.leak()) };
		};
	}

	gen_tests!(64 STATIC_FOO);
	gen_tests!(0 STATIC_BAR);
}
