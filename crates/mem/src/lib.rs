//! Optical perfection.
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod aligned;
pub mod layout;
mod macros;
pub mod pod;

/// Union transmute.
/// Fully unchecked version of [`core::mem::transmute`].
///
/// A union is constructed as `src`, and then indexed as `dst`.
pub const unsafe fn utransmute<Src, Dst>(src: Src) -> Dst {
	use core::mem::ManuallyDrop;

	union Transmute<Src, Dst> {
		src: ManuallyDrop<Src>,
		dst: ManuallyDrop<Dst>,
	}

	ManuallyDrop::into_inner(unsafe { Transmute::<Src, Dst> { src: ManuallyDrop::new(src) }.dst })
}
