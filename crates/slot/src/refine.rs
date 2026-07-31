use crate::Slot;
use crate::convert::{RawSlotRef, RawSlotTransparent, SlotVisPriv};
use core::marker::PhantomData;
use itym_assert::*;
use itym_ts::{Bool, TypeState};
use itym_util::utransmute;

/// Monad of a [`Slot<B>`] with contents that have been refined to represent valid bit patterns of `R`.
#[repr(transparent)]
pub struct SlotRefined<R: ?Sized, B> {
	slot: Slot<B>,
	_refined: PhantomData<R>,
}

impl<R: ?Sized, B> SlotRefined<R, B> {
	pub const unsafe fn new_unchecked(slot: Slot<B>) -> Self {
		Self { slot, _refined: PhantomData }
	}

	pub const fn as_slot(&self) -> &Slot<B> {
		&self.slot
	}

	pub const unsafe fn as_mut_slot(&mut self) -> &mut Slot<B> {
		&mut self.slot
	}

	pub const fn into_inner(self) -> Slot<B> {
		self.slot
	}
}

impl<R: ?Sized + SlotRefine, B> SlotRefined<R, B> {
	pub const fn as_ref(&self) -> &R {
		const SIZE_BYTE_PTR: usize = size!(&u8);
		const SIZE_SLICE_PTR: usize = size!(&[u8]);

		// SAFETY: upheld by implementer of `impl SlotRefine for R`
		match size!(&R) {
			SIZE_BYTE_PTR => {
				const_assert!(size_eq!(&R, &u8, &Slot<B>));
				unsafe { utransmute::<&Slot<B>, &R>(&self.slot) }
			}

			SIZE_SLICE_PTR => {
				const_assert!(size_eq!(&R, &[u8]));
				unsafe { utransmute::<&[u8], &R>(self.slot.as_slice()) }
			}

			_ => unsafe { debug_unreachable!() },
		}
	}
}

impl<R: ?Sized + SlotRefine> SlotRefined<R, [u8; 0]>
where
	<R as SlotRefine>::AllowEmptyInit: TypeState<bool, State = Bool<true>>,
{
	pub const fn new_empty() -> SlotRefined<R, [u8; 0]> {
		SlotRefined {
			slot: unsafe { Slot::zeroed_unchecked() },
			_refined: PhantomData,
		}
	}
}

unsafe impl<R, B> RawSlotRef<B> for SlotRefined<R, B> {
	type Visibility = SlotVisPriv;

	const OFFSET: usize = 0;
}

unsafe impl<R, B> RawSlotTransparent<B> for SlotRefined<R, B> {}

pub unsafe trait SlotRefine {
	type AllowEmptyInit: TypeState<bool>;
}

unsafe impl SlotRefine for str {
	type AllowEmptyInit = Bool<true>;
}

#[test]
fn test() {
	SlotRefined::<str, _>::new_empty();
}
