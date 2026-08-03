//! Monads for terminated sequences. See [`AsTerminated`] or [`AsNulTerminated`].

use core::borrow::{Borrow, BorrowMut};
use core::error::Error;
use core::ffi::CStr;
use core::fmt::{Debug, Display, Formatter};
use core::hash::{Hash, Hasher};
use core::ops::{Deref, DerefMut};

macro_rules! gen_monad {
	(
		$($module:ident $ident:ident = $byte:literal),+
		$(,)?
	) => {
		$(
		mod $module {
			use super::*;
			use crate::error::CapacityError;
			use core::cmp::Ordering;
			use core::ptr;

			#[doc = concat!("Trims the ending `", $byte, "` bytes off `T` before usage.")]
			#[derive(Clone, Copy)]
			#[repr(transparent)]
			pub struct $ident<T: ?Sized>(pub T);

			impl<T> $ident<T> {
				pub fn new(sequence: T) -> Self {
					Self(sequence)
				}
			}

			impl<T: ?Sized> $ident<T> {
				pub const fn new_ref(sequence: &T) -> &Self {
					// SAFETY:
					// `Self` is transparent over `T`, and a compound DST has the
					// metadata of its unsized final field.
					unsafe { core::mem::transmute::<&T, &Self>(sequence) }
				}

				pub const fn new_mut(sequence: &mut T) -> &mut Self {
					// SAFETY: Same reasoning as `new_ref`, while preserving uniqueness.
					unsafe { core::mem::transmute::<&mut T, &mut Self>(sequence) }
				}
			}

			impl $ident<[u8]> {
				pub const fn const_as_bytes(&self) -> &[u8] {
					terminated_bytes(&self.0)
				}

				pub const fn const_as_bytes_mut(&mut self) -> &mut [u8] {
					terminated_bytes_mut(&mut self.0)
				}

				pub const fn const_len(&self) -> usize {
					terminated_len(&self.0)
				}

				/// Set the first byte of the terminatation string to `value`.
				pub const fn const_push_byte(&mut self, value: u8) -> Result<(), CapacityError> {
					let (_head, tail) = self.const_split_terminator_mut();

					if let Some(error) = CapacityError::new(tail.len(), 1) {
						return Err(error);
					}

					tail[0] = value;

					Ok(())
				}

				/// Encodes `char` as UTF-8 at the start of the termination string.
				pub const fn const_push_char(&mut self, char: char) -> Result<(), CapacityError> {
					let (_head, tail) = self.const_split_terminator_mut();

					if let Some(error) = CapacityError::new(tail.len(), char.len_utf8()) {
						return Err(error);
					}

					char.encode_utf8(tail);

					Ok(())
				}

				#[doc = concat!("Head never ends in `'\\x", $byte, "'`,")]
				#[doc = concat!("tail is always 0 or more repititions of `'\\x", $byte, "'`.")]
				pub const fn const_split_terminator(&self) -> (&[u8], &[u8]) {
					self.0.split_at(self.const_len())
				}

				/// See [`Self::split_terminator`].
				pub const fn const_split_terminator_mut(&mut self) -> (&mut [u8], &mut [u8]) {
					self.0.split_at_mut(self.const_len())
				}
			}

			impl $ident<str> {
				pub const fn const_as_bytes(&self) -> &[u8] {
					terminated_bytes(self.0.as_bytes())
				}

				/// # Safety
				/// See [`str::as_bytes_mut`].
				pub const unsafe fn const_as_bytes_mut(&mut self) -> &mut [u8] {
					terminated_bytes_mut(unsafe { self.0.as_bytes_mut() })
				}

				pub const fn const_as_str(&self) -> &str {
					terminated(&self.0)
				}

				pub const fn const_as_mut_str(&mut self) -> &mut str {
					terminated_mut(&mut self.0)
				}

				pub const fn const_clear(&mut self) {
					let len = self.0.len();

					// SAFETY:
					// `as_mut_ptr()` points to the first byte of `self.0`, which is writable
					// for exactly `len` bytes through this exclusive borrow. The pointer is
					// correctly aligned for `u8`, and filling the range with the ASCII byte
					// preserves the UTF-8 validity required by `str`.
					unsafe { ptr::write_bytes(self.0.as_mut_ptr(), $byte, len) };
				}

				pub const fn const_len(&self) -> usize {
					terminated_len(self.0.as_bytes())
				}

				pub const fn const_push_char(&mut self, char: char) -> Result<(), CapacityError> {
					let (_head, tail) = self.const_split_terminator_mut();

					if let Some(error) = CapacityError::new(tail.len(), char.len_utf8()) {
						return Err(error);
					}

					// SAFETY: `tail` only contains ASCII characters
					char.encode_utf8(unsafe { tail.as_bytes_mut() });

					Ok(())
				}

				pub const fn const_push_str(&mut self, str: &str) -> Result<(), CapacityError> {
					let (_head, tail) = self.const_split_terminator_mut();

					if let Some(error) = CapacityError::new(tail.len(), str.len()) {
						return Err(error);
					}

					// SAFETY: `tail` only contains ASCII characters
					unsafe { tail.as_bytes_mut() }.copy_from_slice(str.as_bytes());

					Ok(())
				}

				#[doc = concat!("Head never ends in `'\\x", $byte, "'`,")]
				#[doc = concat!("tail is always 0 or more repititions of `'\\x", $byte, "'`.")]
				pub const fn const_split_terminator(&self) -> (&str, &str) {
					self.0.split_at(self.const_len())
				}

				/// See [`Self::split_terminator`].
				pub const fn const_split_terminator_mut(&mut self) -> (&mut str, &mut str) {
					self.0.split_at_mut(self.const_len())
				}
			}

			impl<T: AsRef<[u8]>> $ident<T> {
				pub fn as_bytes(&self) -> &[u8] {
					terminated_bytes(self.0.as_ref())
				}

				pub fn bytes_split_terminator(&self) -> (&[u8], &[u8]) {
					self.0.as_ref().split_at(terminated_len(self.0.as_ref()))
				}

				pub fn len(&self) -> usize {
					terminated_len(self.0.as_ref())
				}
			}

			impl<T: AsMut<[u8]>> $ident<T> {
				pub fn as_bytes_mut(&mut self) -> &[u8] {
					terminated_bytes(self.0.as_mut())
				}

				pub fn bytes_split_terminator_mut(&mut self) -> (&mut [u8], &mut [u8]) {
					$ident::new_mut(self.0.as_mut()).const_split_terminator_mut()
				}

				/// Set the first byte of the terminatation string to `value`.
				pub fn push_byte(&mut self, value: u8) -> Result<(), CapacityError> {
					$ident::new_mut(self.0.as_mut()).const_push_byte(value)
				}

				/// Encodes `char` as UTF-8 at the start of the termination string.
				pub fn push_char(&mut self, char: char) -> Result<(), CapacityError> {
					$ident::new_mut(self.0.as_mut()).const_push_char(char)
				}
			}

			impl<T: AsRef<str>> $ident<T> {
				pub fn as_str(&self) -> &str {
					terminated(self.0.as_ref())
				}

				pub fn split_terminator(&self) -> (&str, &str) {
					$ident::new_ref(self.0.as_ref()).const_split_terminator()
				}
			}

			impl<T: AsMut<str>> $ident<T> {
				pub fn as_mut_str(&mut self) -> &mut str {
					terminated_mut(self.0.as_mut())
				}

				pub fn split_terminator_mut(&mut self) -> (&mut str, &mut str) {
					$ident::new_mut(self.0.as_mut()).const_split_terminator_mut()
				}
			}

			impl<T> AsRef<$ident<str>> for $ident<T>
			where
				T: AsRef<str>,
			{
				fn as_ref(&self) -> &$ident<str> {
					$ident::new_ref(self.0.as_ref())
				}
			}

			impl<T, U> AsRef<U> for $ident<T>
			where
				T: AsRef<str>,
				str: AsRef<U>,
			{
				fn as_ref(&self) -> &U {
					terminated(self.0.as_ref()).as_ref()
				}
			}

			impl<T, U> AsMut<U> for $ident<T>
			where
				T: AsMut<str>,
				str: AsMut<U>,
			{
				fn as_mut(&mut self) -> &mut U {
					terminated_mut(self.0.as_mut()).as_mut()
				}
			}

			impl<T> AsMut<$ident<str>> for $ident<T>
			where
				T: AsMut<str>,
			{
				fn as_mut(&mut self) -> &mut $ident<str> {
					$ident::new_mut(self.0.as_mut())
				}
			}

			impl<T> Deref for $ident<T>
			where
				T: AsRef<str>,
			{
				type Target = str;

				fn deref(&self) -> &Self::Target {
					terminated(self.0.as_ref())
				}
			}

			impl<T> DerefMut for $ident<T>
			where
				T: AsMut<str> + AsRef<str>,
			{
				fn deref_mut(&mut self) -> &mut Self::Target {
					terminated_mut(self.0.as_mut())
				}
			}

			impl<T> Debug for $ident<T>
			where
				T: AsRef<str>,
			{
				fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
					f.debug_tuple(stringify!($ident)).field(&self.0.as_ref()).finish()
				}
			}

			impl Display for $ident<str> {
				fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
					f.write_str(self.const_as_str())
				}
			}

			impl<T> Borrow<str> for $ident<T>
			where
				T: AsRef<str>,
			{
				fn borrow(&self) -> &str {
					terminated(self.0.as_ref())
				}
			}

			impl<T> BorrowMut<str> for $ident<T>
			where
				T: AsMut<str> + AsRef<str>,
			{
				fn borrow_mut(&mut self) -> &mut str {
					terminated_mut(self.0.as_mut())
				}
			}

			impl<T, U> PartialEq<$ident<U>> for $ident<T>
			where
				T: AsRef<[u8]>,
				U: AsRef<[u8]>,
			{
				fn eq(&self, other: &$ident<U>) -> bool {
					terminated_bytes(self.0.as_ref()).eq(terminated_bytes(other.0.as_ref()))
				}
			}

			impl<T, Rhs> PartialEq<Rhs> for $ident<T>
			where
				T: AsRef<str>,
				str: PartialEq<Rhs>,
			{
				fn eq(&self, other: &Rhs) -> bool {
					terminated(self.0.as_ref()).eq(other)
				}
			}

			impl<T> Eq for $ident<T> where T: AsRef<[u8]> {}

			impl<T, U> PartialOrd<$ident<U>> for $ident<T>
			where
				T: AsRef<[u8]>,
				U: AsRef<[u8]>,
			{
				fn partial_cmp(&self, other: &$ident<U>) -> Option<Ordering> {
					terminated_bytes(self.0.as_ref()).partial_cmp(terminated_bytes(other.0.as_ref()))
				}
			}

			impl<T, Rhs> PartialOrd<Rhs> for $ident<T>
			where
				T: AsRef<str>,
				str: PartialOrd<Rhs>,
			{
				fn partial_cmp(&self, other: &Rhs) -> Option<Ordering> {
					terminated(self.0.as_ref()).partial_cmp(other)
				}
			}

			impl<T> Ord for $ident<T>
			where
				T: AsRef<[u8]>
			{
				fn cmp(&self, other: &Self) -> Ordering {
					terminated_bytes(self.0.as_ref()).cmp(terminated_bytes(other.0.as_ref()))
				}
			}

			impl<T> Hash for $ident<T>
			where
				T: AsRef<[u8]>
			{
				fn hash<H: Hasher>(&self, state: &mut H) {
					terminated_bytes(self.0.as_ref()).hash(state)
				}
			}

			const fn terminated(str: &str) -> &str {
				unsafe { str::from_utf8_unchecked(terminated_bytes(str.as_bytes())) }
			}

			const fn terminated_mut(str: &mut str) -> &mut str {
				unsafe { str::from_utf8_unchecked_mut(terminated_bytes_mut(str.as_bytes_mut())) }
			}

			const fn terminated_bytes(mut bytes: &[u8]) -> &[u8] {
				while let [rest @ .., $byte] = bytes {
					bytes = rest;
				}

				bytes
			}

			const fn terminated_bytes_mut(mut bytes: &mut [u8]) -> &mut [u8] {
				while let [rest @ .., $byte] = bytes {
					bytes = rest;
				}

				bytes
			}

			const fn terminated_len(bytes: &[u8]) -> usize {
				terminated_bytes(bytes).len()
			}
		}
		)+
	};
}

gen_monad! {
	sub AsTerminated = 0x1A,
	nul AsNulTerminated = 0x00,
}

pub use nul::*;
pub use sub::*;

impl AsNulTerminated<[u8]> {
	pub const fn as_cstr(&self) -> Result<&CStr, CStrConversionError> {
		let target_len = self.const_len();

		match CStr::from_bytes_until_nul(self.const_as_bytes()) {
			Ok(cstr) if cstr.count_bytes() == target_len => Ok(cstr),
			Ok(..) => Err(CStrConversionError::InteriorNul),
			Err(..) => Err(CStrConversionError::MissingNul),
		}
	}
}

impl AsNulTerminated<str> {
	pub const fn as_cstr(&self) -> Result<&CStr, CStrConversionError> {
		AsNulTerminated::new_ref(self.0.as_bytes()).as_cstr()
	}
}

/// Error states emitted by [`AsNulTerminated::<[u8]>::as_cstr`].
#[derive(Debug, Clone, Copy)]
pub enum CStrConversionError {
	/// [`AsNulTerminated`] did not end in at least 1 nul byte.
	MissingNul,

	/// [`AsNulTerminated`] contains 1 or more nul-byte not at the end of the sequence.
	InteriorNul,
}

impl Display for CStrConversionError {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		match self {
			Self::MissingNul => f.write_str("Missing nul terminator"),
			Self::InteriorNul => f.write_str("Interior nul byte"),
		}
	}
}

impl Error for CStrConversionError {}
