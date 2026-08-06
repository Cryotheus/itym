//! Byte-array supporting type state and `const` mutation.
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use crate::convert::{RawSlotRef, SlotVisPub};
use crate::writer::SlotWriter;
use bytemuck::{Pod, Zeroable};
use convert::RawSlotTransparent;
use core::cmp::Ordering;
use core::fmt::{Debug, Formatter};
use core::hash::{Hash, Hasher};
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::ptr::{copy_nonoverlapping, slice_from_raw_parts, slice_from_raw_parts_mut, write_bytes};
use itym_assert::*;
use itym_mem::utransmute;

macro_rules! size {
	(-: $lhs:ty, $rhs:ty) => {
		const { ::core::mem::size_of::<$lhs>().strict_sub(::core::mem::size_of::<$rhs>()) }
	};

	(+: $lhs:ty, $rhs:ty) => {
		const { ::core::mem::size_of::<$lhs>().strict_add(::core::mem::size_of::<$rhs>()) }
	};

	(>: $lhs:ty, $rhs:ty) => {
		const { ::core::mem::size_of::<$lhs>() > ::core::mem::size_of::<$rhs>() }
	};

	(>=: $lhs:ty, $rhs:ty) => {
		const { ::core::mem::size_of::<$lhs>() >= ::core::mem::size_of::<$rhs>() }
	};

	(!=: $lhs:ty, $rhs:ty) => {
		const { ::core::mem::size_of::<$lhs>() != ::core::mem::size_of::<$rhs>() }
	};

	(==: $lhs:ty, $rhs:ty) => {
		const { ::core::mem::size_of::<$lhs>() == ::core::mem::size_of::<$rhs>() }
	};

	($ty:ty) => {
		const { ::core::mem::size_of::<$ty>() }
	};
}

macro_rules! size_eq {
	($lhs:ty $(, $tail:ty)+ $(,)?) => {
		const { $(::core::mem::size_of::<$lhs>() == ::core::mem::size_of::<$tail>())&&+ }
	};
}

macro_rules! size_ne {
	($lhs:ty, $rhs:ty) => {
		size!(!=: $lhs, $rhs)
	};
}

pub mod convert;
mod impls;
pub mod writer;

/// For use of a [`Pod`] as the buffer of a [`Slot`].
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ByPod<T: Pod>(pub T);

unsafe impl<T: Pod> ForeignSlotInit for ByPod<T> {}

unsafe impl<T: Pod> Pod for ByPod<T> {}

unsafe impl<T: Pod> Zeroable for ByPod<T> {}

/// Byte-array using pod ("plain old data") as the underlying buffer.
/// Supports `const` and offers mutation of the buffer size by type state.
///
/// Use [`Self::new`] or [`Self::new_array_fill`] to safely initialize a slot.
#[repr(transparent)]
pub struct Slot<T>(MaybeUninit<T>);

impl<T: Pod> Slot<ByPod<T>> {
	pub const fn new(pod: T) -> Self {
		const_assert!(const { Self::LEN < isize::MAX as usize });
		const_assert!(const { align_of::<Self>() == 1 });
		Self(MaybeUninit::new(ByPod(pod)))
	}

	pub const fn zeroed() -> Self {
		const_assert!(const { Self::LEN < isize::MAX as usize });
		const_assert!(const { align_of::<Self>() == 1 });
		Self(MaybeUninit::zeroed())
	}
}

impl<T> Slot<T> {
	pub const LEN: usize = size!(Self);

	/// Unrestrained version of [`Self::new_array_fill`]
	///
	/// # Safety
	/// - `T` must have an alignment of 1
	/// - `T` should always be trivial to drop, or otherwise be safe to drop
	/// - Same requirements as [`Pod`]
	pub const unsafe fn filled_unchecked(value: u8) -> Self {
		const_assert!(const { Self::LEN < isize::MAX as usize });
		const_assert_eq!(align_of::<Self>(), 1);
		// const_assert!(!core::mem::needs_drop::<Self>());

		let mut slot = unsafe { Self::uninit() };

		slot.fill(value);

		slot
	}

	/// # Safety
	/// - `T` must have an alignment of 1
	/// - Same requirements as [`Pod`]
	pub const unsafe fn new_unchecked(buffer: MaybeUninit<T>) -> Self {
		const_assert!(const { Self::LEN < isize::MAX as usize });
		const_assert_eq!(align_of::<Self>(), 1);
		Self(buffer)
	}

	/// Unrestrained version of [`Self::new_array_zeroed`]
	///
	/// # Safety
	/// - `T` must have an alignment of 1
	/// - Same requirements as [`Pod`]
	pub const unsafe fn zeroed_unchecked() -> Self {
		const_assert!(const { Self::LEN < isize::MAX as usize });
		const_assert_eq!(align_of::<Self>(), 1);
		Self(MaybeUninit::zeroed())
	}

	/// # Safety
	/// - `T` must have an alignment of 1
	/// - Contents must be filled before read.
	/// - Same requirements as [`Pod`]
	pub const unsafe fn uninit() -> Self {
		const_assert!(const { Self::LEN < isize::MAX as usize });
		const_assert_eq!(align_of::<Self>(), 1);
		Self(MaybeUninit::uninit())
	}

	/// # Safety
	/// - The length of `src` must match the size of `T` exactly.
	/// - `T` must have an alignment of 1
	/// - Same requirements as [`Pod`]
	pub const unsafe fn from_slice_unchecked(src: &[u8]) -> Self {
		let mut slot = unsafe { Self::uninit() };

		slot.write_slice(src);

		slot
	}

	/// Returns `None` if the size does not match.
	///
	/// # Safety
	/// - `T` must have an alignment of 1
	/// - Same requirements as [`Pod`]
	pub const unsafe fn try_from_slice(src: &[u8]) -> Option<Self> {
		if src.len() != Self::LEN {
			return None;
		}

		Some(unsafe { Self::from_slice_unchecked(src) })
	}

	#[inline(always)]
	pub const fn as_ptr(&self) -> *const u8 {
		(self as *const Self).cast::<u8>()
	}

	#[inline(always)]
	pub const fn as_mut_ptr(&mut self) -> *mut u8 {
		(self as *mut Self).cast::<u8>()
	}

	/// # Panics
	/// If `LEN` does not match `self.len`
	pub const fn as_array<const LEN: usize>(&self) -> &[u8; LEN] {
		const_assert!(const { Self::LEN == LEN });
		unsafe { &*self.as_ptr().cast::<[u8; LEN]>() }
	}

	/// # Panics
	/// If `LEN` does not match `self.len`
	pub const fn as_mut_array<const LEN: usize>(&mut self) -> &mut [u8; LEN] {
		const_assert!(const { Self::LEN == LEN });
		unsafe { &mut *self.as_mut_ptr().cast::<[u8; LEN]>() }
	}

	/// Same as [`<[T]>::as_chunks`].
	///
	/// Provided as this is useful in a `const` context where [`Deref::deref`] may not be available.
	pub const fn as_chunks<const N: usize>(&self) -> (&[[u8; N]], &[u8]) {
		self.as_slice().as_chunks()
	}

	/// Same as [`<[T]>::as_chunks_mut`].
	///
	/// Provided as this is useful in a `const` context where [`Deref::deref`] may not be available.
	pub const fn as_chunks_mut<const N: usize>(&mut self) -> (&mut [[u8; N]], &mut [u8]) {
		self.as_mut_slice().as_chunks_mut()
	}

	/// Same as [`<[T]>::as_rchunks`].
	///
	/// Provided as this is useful in a `const` context where [`Deref::deref`] may not be available.
	pub const fn as_rchunks<const N: usize>(&self) -> (&[u8], &[[u8; N]]) {
		self.as_slice().as_rchunks()
	}

	/// Same as [`<[T]>::as_rchunks_mut`].
	///
	/// Provided as this is useful in a `const` context where [`Deref::deref`] may not be available.
	pub const fn as_rchunks_mut<const N: usize>(&mut self) -> (&mut [u8], &mut [[u8; N]]) {
		self.as_mut_slice().as_rchunks_mut()
	}

	#[doc(alias("as_bytes"))]
	pub const fn as_slice(&self) -> &[u8] {
		unsafe { core::slice::from_raw_parts(self.as_ptr(), Self::LEN) }
	}

	#[doc(alias("as_bytes_mut"))]
	pub const fn as_mut_slice(&mut self) -> &mut [u8] {
		unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr(), Self::LEN) }
	}

	pub const fn as_slice_ptr(&self) -> *const [u8] {
		slice_from_raw_parts(self.as_ptr(), Self::LEN)
	}

	pub const fn as_mut_slice_ptr(&mut self) -> *mut [u8] {
		slice_from_raw_parts_mut(self.as_mut_ptr(), Self::LEN)
	}

	/// Allows copying even when `T` does not implement [`Copy`].
	pub const fn copy(&self) -> Self {
		let mut slot = unsafe { Self::uninit() };

		unsafe { copy_nonoverlapping(self as *const Self, &raw mut slot, 1) };

		slot
	}

	/// Same as [`<[T]>::copy_from_slice`].
	///
	/// Provided as this is useful in a `const` context where [`Deref::deref`] may not be available.
	pub const fn copy_from_slice<U>(&mut self, src: &[u8]) {
		const_debug_assert_eq!(Self::LEN, src.len());
		self.as_mut_slice().copy_from_slice(src);
	}

	pub const fn copy_from_slot<U>(&mut self, src: &Slot<U>) {
		const_debug_assert!(size_eq!(Self, Slot<U>));
		//TODO: run this through miri
		unsafe { copy_nonoverlapping((src as *const Slot<U>).cast::<Self>(), self, 1) };
	}

	/// Fills the entire slot with the provided byte.
	///
	/// Initialization safe.
	pub const fn fill(&mut self, value: u8) {
		//TODO: run this through miri
		unsafe { write_bytes(self.0.as_mut_ptr(), value, 1) }
	}

	const fn _fill_char_utf8<const N: usize>(&mut self, char: char) -> &mut [u8] {
		let mut dst = [0u8; N];

		char.encode_utf8(&mut dst);
		self.fill_chunks(&dst)
	}

	/// Initialization safe.
	///
	/// # Panics
	/// If [`Self::len`] is not a multiple of `N`.
	const fn _fill_char_utf8_exact<const N: usize>(&mut self, char: char) {
		let mut dst = [0u8; N];

		char.encode_utf8(&mut dst);
		self.fill_chunks_exact(&dst);
	}

	pub const fn fill_char_utf8(&mut self, char: char) -> &mut [u8] {
		match char.len_utf8() {
			1 => {
				self.fill(char as u8);

				//tail-position empty slice
				self.as_mut_slice().split_at_mut(Self::LEN).1
			}

			2 => self._fill_char_utf8::<2>(char),
			3 => self._fill_char_utf8::<3>(char),
			4 => self._fill_char_utf8::<4>(char),
			_ => unsafe { debug_unreachable!() },
		}
	}

	pub const fn fill_char_utf8_generic<const CHAR: char>(&mut self) -> &mut [u8] {
		match const { CHAR.len_utf8() } {
			1 => {
				self.fill(const { CHAR as u8 });

				//tail-position empty slice
				self.as_mut_slice().split_at_mut(Self::LEN).1
			}

			2 => self._fill_char_utf8::<2>(CHAR),
			3 => self._fill_char_utf8::<3>(CHAR),
			4 => self._fill_char_utf8::<4>(CHAR),
			_ => unsafe { debug_unreachable!() },
		}
	}

	/// Initialization safe.
	///
	/// # Panics
	/// If [`Self::len`] is not a multiple of `N`.
	pub const fn fill_char_utf8_exact(&mut self, char: char) {
		match char.len_utf8() {
			1 => self.fill(char as u8),
			2 => self._fill_char_utf8_exact::<2>(char),
			3 => self._fill_char_utf8_exact::<3>(char),
			4 => self._fill_char_utf8_exact::<4>(char),
			_ => unsafe { debug_unreachable!() },
		}
	}

	/// Initialization safe.
	///
	/// # Panics
	/// If [`Self::len`] is not a multiple of `N`.
	pub const fn fill_char_utf8_exact_generic<const CHAR: char>(&mut self) {
		match const { CHAR.len_utf8() } {
			1 => self.fill(const { CHAR as u8 }),
			2 => self._fill_char_utf8_exact::<2>(CHAR),
			3 => self._fill_char_utf8_exact::<3>(CHAR),
			4 => self._fill_char_utf8_exact::<4>(CHAR),
			_ => unsafe { debug_unreachable!() },
		}
	}

	/// Fills the slot with repetitions of the provided `chunk`.
	pub const fn fill_chunks<const N: usize>(&mut self, chunk: &[u8; N]) -> &mut [u8] {
		//TODO: run this through miri
		let (chunks, tail) = self.as_chunks_mut::<N>();
		let count = chunks.len();

		unsafe { copy_nonoverlapping(chunk, chunks.as_mut_ptr(), count) };

		tail
	}

	/// Fills the slot with repetitions of the provided `chunk`.
	///
	/// Initialization safe.
	///
	/// # Panics
	/// If [`Self::len`] is not a multiple of `N`.
	pub const fn fill_chunks_exact<const N: usize>(&mut self, chunk: &[u8; N]) {
		//TODO: run this through miri
		let (chunks, []) = self.as_chunks_mut::<N>() else { panic!() };
		let count = chunks.len();

		unsafe { copy_nonoverlapping(chunk, chunks.as_mut_ptr(), count) };
	}

	/// Fills the slot with repetitions of the provided `chunk`.
	pub const fn fill_rchunks<const N: usize>(&mut self, chunk: &[u8; N]) -> &mut [u8] {
		//TODO: run this through miri
		let (head, chunks) = self.as_rchunks_mut::<N>();
		let count = chunks.len();

		unsafe { copy_nonoverlapping(chunk, chunks.as_mut_ptr(), count) };

		head
	}

	/// Same as [`<[T]>::first_chunk`].
	///
	/// Provided as this is useful in a `const` context where [`Deref::deref`] may not be available.
	pub const fn first_chunk<const N: usize>(&self) -> Option<&[u8; N]> {
		self.as_slice().first_chunk()
	}

	/// Same as [`<[T]>::first_chunk_mut`].
	///
	/// Provided as this is useful in a `const` context where [`Deref::deref`] may not be available.
	pub const fn first_chunk_mut<const N: usize>(&mut self) -> Option<&mut [u8; N]> {
		self.as_mut_slice().first_chunk_mut()
	}

	/// Same as [`<[T]>::last_chunk`].
	///
	/// Provided as this is useful in a `const` context where [`Deref::deref`] may not be available.
	pub const fn last_chunk<const N: usize>(&self) -> Option<&[u8; N]> {
		self.as_slice().last_chunk()
	}

	/// Same as [`<[T]>::last_chunk_mut`].
	///
	/// Provided as this is useful in a `const` context where [`Deref::deref`] may not be available.
	pub const fn last_chunk_mut<const N: usize>(&mut self) -> Option<&mut [u8; N]> {
		self.as_mut_slice().last_chunk_mut()
	}

	/// # Panics
	/// If the `LEN` does not match [`Self::len`].
	pub const fn into_array<const LEN: usize>(self) -> [u8; LEN] {
		const_assert!(size_eq!(Self, [u8; LEN]));
		unsafe { utransmute(self) }
	}

	/// If the array is larger than the slot, `value` fills the trailing bytes.
	pub const fn into_array_resize<const LEN: usize>(self, value: u8) -> [u8; LEN] {
		let mut resized = unsafe { utransmute::<Self, MaybeUninit<[u8; LEN]>>(self) };

		if const { LEN > Self::LEN } {
			let tail = unsafe { resized.as_mut_ptr().cast::<u8>().byte_add(Self::LEN) };

			unsafe { write_bytes(tail, value, LEN.strict_sub(Self::LEN)) };
		}

		unsafe { resized.assume_init() }
	}

	/// Same as [`Self::resize`], but for targets that are the same or smaller size.
	pub const fn into_array_truncated<const LEN: usize>(self) -> [u8; LEN] {
		const_assert!(LEN <= Self::LEN);
		unsafe { utransmute::<Self, [u8; LEN]>(self) }
	}

	/// Sames as [`Self::LEN`].
	pub const fn len(&self) -> usize {
		Self::LEN
	}

	/// Type state mutation joining `Slot<T>` and `Slot<U>` into `Slot<T, U>`.
	pub const fn push<U>(self, other: Slot<U>) -> Slot<(T, U)> {
		//TODO: use custom struct instead of tuple
		// to ensure offsets and field ordering
		const_assert!(const: Slot::<(T, U)>::LEN < isize::MAX as usize);
		const_assert_eq!(const: Self::LEN + Slot::<U>::LEN, Slot::<(T, U)>::LEN);

		unsafe { utransmute((self, other)) }
	}

	/// If the target slot is larger, `value` fills the trailing bytes.
	pub const fn resize<U>(self, value: u8) -> Slot<U> {
		let mut resized = unsafe { utransmute::<Self, Slot<U>>(self) };

		if size!(>: Slot<U>, Self) {
			let tail = unsafe { resized.0.as_mut_ptr().cast::<u8>().byte_add(Self::LEN) };

			unsafe { write_bytes(tail, value, size!(Slot<U>).strict_sub(Self::LEN)) };
		}

		resized
	}

	/// Writes `src` into the slot, setting trailing bytes to `tail`.
	///
	/// Initialization safe.
	///
	/// Also see: [`Self::write_slice`]
	///
	/// # Panics
	/// If `slice.len()` is greater than [`Self::len`].
	pub const fn set_slice(&mut self, src: &[u8], tail_fill: u8) {
		let (head, tail) = self.as_mut_slice().split_at_mut(src.len());
		let tail_len = tail.len();

		head.copy_from_slice(src);
		unsafe { write_bytes(tail.as_mut_ptr(), tail_fill, tail_len) };
	}

	/// Same as [`Self::resize`], but for targets that are the same or smaller size.
	pub const fn truncated<U>(self) -> Slot<U> {
		const_assert!(const { Slot::<U>::LEN <= Self::LEN });
		self.resize(0)
	}

	/// Convenience for calling [`SlotWriter::new`].
	pub const fn writer(self) -> SlotWriter<T> {
		SlotWriter::new(self)
	}

	/// Initialization safe.
	///
	/// # Panics
	/// If `position >= len`.
	pub const fn write_byte_at(&mut self, position: usize, value: u8) {
		const_assert!(position < Self::LEN);
		unsafe { self.as_mut_ptr().add(position).write(value) };
	}

	/// # Panics
	/// If `count > len`.
	pub const fn write_bytes(&mut self, value: u8, count: usize) {
		const_assert!(count <= Self::LEN);
		unsafe { write_bytes(self.as_mut_ptr(), value, count) }
	}

	/// Writes `src` into the slot, leaving trailing bytes untouched.
	///
	/// Initialization safe.
	///
	/// Also see: [`Self::set_slice`], [`Self::write_slice_at`]
	///
	/// # Panics
	/// If `slice.len()` is greater than [`Self::len`].
	pub const fn write_slice(&mut self, src: &[u8]) {
		self.as_mut_slice().split_at_mut(src.len()).0.copy_from_slice(src);
	}

	/// Initialization safe.
	///
	/// # Panics
	/// If the slice would write out of bounds.
	pub const fn write_slice_at(&mut self, start: usize, src: &[u8]) {
		self.as_mut_slice().split_at_mut(start).1.split_at_mut(src.len()).0.copy_from_slice(src);
	}
}

impl<T, U> Slot<(T, U)> {
	/// Type state mutation splitting `Slot<T, U>` into `Slot<T>` and `Slot<U>`.
	pub const fn pop(self) -> (Slot<T>, Slot<U>) {
		//TODO: use custom struct instead of tuple
		// to ensure offsets and field ordering
		const_assert_eq!(const: Slot::<T>::LEN + Slot::<U>::LEN, Self::LEN);

		unsafe { utransmute(self) }
	}

	/// Same as [`Self::pop`] but discards the popped value.
	pub const fn popped(self) -> Slot<T> {
		self.pop().0
	}
}

impl<const LEN: usize> Slot<[u8; LEN]> {
	/// For byte string literals:
	/// ```rust
	/// # use itym_slot::Slot;
	///
	/// fn main() {
	///     // Usable in `const`
	///     let greeting = const { Slot::lit(b"Hello world!") };
	/// }
	/// ```
	pub const fn lit(byte_literal: &'static [u8; LEN]) -> Self {
		Self(MaybeUninit::new(*byte_literal))
	}

	pub const fn new_array(array: [u8; LEN]) -> Self {
		unsafe { Self::new_unchecked(MaybeUninit::new(array)) }
	}

	pub const fn new_array_zeroed() -> Self {
		unsafe { Slot::zeroed_unchecked() }
	}

	pub const fn new_array_fill(value: u8) -> Self {
		unsafe { Slot::filled_unchecked(value) }
	}
}

unsafe impl<T> RawSlotRef<T> for Slot<T> {
	type Visibility = SlotVisPub;

	const OFFSET: usize = 0;
}

unsafe impl<T> RawSlotTransparent<T> for Slot<T> {}

impl<T> Deref for Slot<T> {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		self.as_slice()
	}
}

impl<T> DerefMut for Slot<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.as_mut_slice()
	}
}

impl<T> Clone for Slot<T> {
	fn clone(&self) -> Self {
		self.copy()
	}
}

impl<T: Copy> Copy for Slot<T> {}

impl<T, U> PartialEq<Slot<U>> for Slot<T> {
	fn eq(&self, other: &Slot<U>) -> bool {
		if size_ne!(Slot<U>, Slot<T>) {
			return false;
		}

		self.as_slice().eq(other.as_slice())
	}
}

impl<T, U> PartialEq<U> for Slot<T>
where
	[u8]: PartialEq<U>,
{
	fn eq(&self, other: &U) -> bool {
		self.as_slice().eq(other)
	}
}

impl<T> Eq for Slot<T> {}

impl<T, U> PartialOrd<Slot<U>> for Slot<T> {
	fn partial_cmp(&self, other: &Slot<U>) -> Option<Ordering> {
		self.as_slice().partial_cmp(other.as_slice())
	}
}

impl<T, U> PartialOrd<U> for Slot<T>
where
	[u8]: PartialOrd<U>,
{
	fn partial_cmp(&self, other: &U) -> Option<Ordering> {
		self.as_slice().partial_cmp(other)
	}
}

impl<T> Ord for Slot<T> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.as_slice().cmp(other.as_slice())
	}
}

impl<T> Hash for Slot<T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.as_slice().hash(state);
	}
}

impl<T> Debug for Slot<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.debug_tuple("Slot").field(&self.as_slice()).finish()
	}
}

unsafe impl<T: Copy + 'static> Pod for Slot<T> {}

unsafe impl<T: Copy> Zeroable for Slot<T> {}

/// Types known to be usable as the buffer of a [`Slot`].
/// Ensures soundness for slots constructed by foreign crates.
pub unsafe trait ForeignSlotInit {}

unsafe impl<const LEN: usize> ForeignSlotInit for [u8; LEN] {}

unsafe impl<T: ForeignSlotInit, U: ForeignSlotInit> ForeignSlotInit for (T, U) {}

#[test]
fn layout() {
	let hello = Slot::lit(b"Hello ");
	let world = Slot::lit(b"world!");
	let mut hello_world = hello.push(world);
	assert_eq!(&hello, b"Hello ");
	assert_eq!(&world, b"world!");
	assert_eq!(&hello_world, b"Hello world!");

	assert_eq!(hello.as_slice(), hello.copy().as_slice());
	assert_eq!(hello_world.as_slice(), hello_world.copy().as_slice());

	let (hello2, world2) = hello_world.pop();
	assert_eq!(hello.as_slice(), hello2.as_slice());
	assert_eq!(world.as_slice(), world2.as_slice());

	let hello_padded = hello.resize::<[u8; 8]>(0x1A);
	let hello_padded2 = hello.into_array_resize::<8>(0x1A);
	assert_eq!(&hello_padded, b"Hello \x1a\x1a");
	assert_eq!(&hello_padded2, b"Hello \x1a\x1a");

	let hello_truncated = hello.truncated::<[u8; 4]>();
	let hello_truncated2 = hello.into_array_truncated::<4>();
	assert_eq!(&hello_truncated, b"Hell");
	assert_eq!(&hello_truncated2, b"Hell");

	let cleared = hello.truncated::<[u8; 0]>();
	assert_eq!(&cleared, b"");

	let array_ref = hello_world.as_array::<12>();
	assert_eq!(array_ref, b"Hello world!");

	let array_mut = hello_world.as_mut_array::<12>();
	assert_eq!(array_mut, b"Hello world!");

	let array_owned = hello_world.into_array::<12>();
	assert_eq!(&array_owned, b"Hello world!");
}

#[test]
fn zero_size() {
	let foo = Slot::lit(b"");
	let bar = Slot::lit(b"");
	let foobar = foo.push(bar);
	assert_eq!(&foo, b"");
	assert_eq!(&bar, b"");
	assert_eq!(&foobar, b"");

	assert_eq!(foo, foo.copy());
	assert_eq!(foobar, foobar.copy());

	let (foo2, bar2) = foobar.pop();
	assert_eq!(foo.as_slice(), foo2.as_slice());
	assert_eq!(bar.as_slice(), bar2.as_slice());

	let mut dst = foobar.copy();
	dst.fill(60);
	assert_eq!(&dst, b"");
}
