use crate::array_str::ArrayStr;
use crate::error::CapacityError;
use crate::pod::PodStr;
use crate::util::{const_assert, const_assert_eq, const_debug_assert, transmute2};
use core::cmp::Ordering;
use core::fmt::{Debug, Display, Formatter};
use core::hash::{Hash, Hasher};
use core::ops::{Deref, DerefMut};
use core::slice::{from_raw_parts, from_raw_parts_mut};
use core::str::{from_utf8_unchecked, from_utf8_unchecked_mut};

#[derive(Debug)]
#[cfg_attr(feature = "c_abi", repr(C))]
pub struct ArrayString<const CAP: usize> {
	len: usize,
	pod: PodStr<[u8; CAP]>,
}

impl<const CAP: usize> ArrayString<CAP> {
	#[inline]
	pub const fn new() -> Self {
		Self { pod: PodStr::zeroed(), len: 0 }
	}

	#[inline]
	pub const fn as_bytes(&self) -> &[u8] {
		const_debug_assert!(const: CAP < isize::MAX as usize);
		unsafe { from_raw_parts(self.as_ptr(), self.len) }
	}

	#[inline]
	pub const unsafe fn as_bytes_mut(&mut self) -> &mut [u8] {
		const_debug_assert!(const: CAP < isize::MAX as usize);
		unsafe { from_raw_parts_mut(self.as_mut_ptr(), self.len) }
	}

	#[inline(always)]
	pub const fn as_ptr(&self) -> *const u8 {
		self.pod.as_ptr().cast::<u8>()
	}

	#[inline(always)]
	pub const fn as_mut_ptr(&mut self) -> *mut u8 {
		self.pod.as_mut_ptr().cast::<u8>()
	}

	#[inline]
	pub const fn as_str(&self) -> &str {
		const_debug_assert!(const: CAP < isize::MAX as usize);
		unsafe { from_utf8_unchecked(from_raw_parts(self.as_ptr(), self.len())) }
	}

	#[inline]
	pub const fn as_mut_str(&mut self) -> &mut str {
		const_debug_assert!(const: CAP < isize::MAX as usize);
		unsafe { from_utf8_unchecked_mut(from_raw_parts_mut(self.as_mut_ptr(), self.len)) }
	}

	/// # Panics
	/// If the string's [`len`] is not equal to `TARGET`.
	///
	/// [`len`]: Self::len
	pub const fn into_array_str<const TARGET: usize>(self) -> ArrayStr<TARGET> {
		const_assert_eq!(self.len, TARGET);

		if const { CAP == TARGET } {
			ArrayStr(unsafe { transmute2(self.pod) })
		} else {
			ArrayStr(unsafe { PodStr::from_utf8_array_unchecked(self.pod.into_first_chunk::<TARGET>()) })
		}
	}

	#[inline]
	pub const fn is_empty(&self) -> bool {
		self.len == 0
	}

	#[inline(always)]
	pub const fn capacity(&self) -> usize {
		CAP
	}

	#[inline]
	pub const fn clear(&mut self) {
		self.len = 0;
	}

	#[inline]
	pub const fn len(&self) -> usize {
		self.len
	}

	/// # Panics
	/// If the remaining capacity is insufficient.
	///
	/// Fallible version: [`try_push_char`]
	///
	/// [`try_push_char`]: Self::try_push_char
	pub const fn push_char(&mut self, char: char) {
		const_assert!(self.try_push_char(char).is_ok());
	}

	/// # Panics
	/// If the remaining capacity is insufficient.
	///
	/// Fallible version: [`try_push_str`]
	///
	/// [`try_push_str`]: Self::try_push_str
	pub const fn push_str(&mut self, str: &str) {
		const_assert!(self.try_push_str(str).is_ok());
	}

	/// # Panics
	/// If the string's [`len`] is less than `TARGET`.
	///
	/// Fallible version: [`try_truncate_into_array_str`]
	///
	/// [`len`]: Self::len
	/// [`try_truncate_into_array_str`]: Self::try_truncate_into_array_str
	pub const fn truncate_into_array_str<const TARGET: usize>(self) -> ArrayStr<TARGET> {
		match self.try_truncate_into_array_str() {
			Ok(array_str) => array_str,
			Err(..) => panic!("`TARGET` must be <= `len`"),
		}
	}

	pub const fn try_truncate_into_array_str<const TARGET: usize>(self) -> Result<ArrayStr<TARGET>, Self> {
		if const { CAP == TARGET } && self.len == CAP {
			Ok(ArrayStr(unsafe { transmute2(self.pod) }))
		} else if self.len == TARGET || (self.len > TARGET && self.pod.is_char_boundary(TARGET)) {
			Ok(ArrayStr(unsafe {
				PodStr::from_utf8_array_unchecked(self.pod.into_first_chunk::<TARGET>())
			}))
		} else {
			Err(self)
		}
	}

	/// Returns an error if the remaining capacity is insufficient.
	pub const fn try_push_char(&mut self, char: char) -> Result<(), CapacityError> {
		let size = char.len_utf8();

		if CAP - self.len < size {
			return Err(CapacityError::new());
		}

		char.encode_utf8(unsafe { self.pod.as_bytes_mut() }.split_at_mut(self.len).1.split_at_mut(size).0);
		self.len += size;

		Ok(())
	}

	/// Returns an error if the remaining capacity is insufficient.
	pub const fn try_push_str(&mut self, str: &str) -> Result<(), CapacityError> {
		let size = str.len();

		if CAP - self.len < size {
			return Err(CapacityError::new());
		}

		unsafe { self.pod.as_bytes_mut() }
			.split_at_mut(self.len)
			.1
			.split_at_mut(size)
			.0
			.copy_from_slice(str.as_bytes());
		self.len += size;

		Ok(())
	}
}

impl<const CAP: usize> Deref for ArrayString<CAP> {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.as_str()
	}
}

impl<const CAP: usize> DerefMut for ArrayString<CAP> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.as_mut_str()
	}
}

impl<U, const CAP: usize> AsRef<U> for ArrayString<CAP>
where
	str: AsRef<U>,
{
	fn as_ref(&self) -> &U {
		self.as_str().as_ref()
	}
}

impl<U, const CAP: usize> AsMut<U> for ArrayString<CAP>
where
	str: AsMut<U>,
{
	fn as_mut(&mut self) -> &mut U {
		self.as_mut_str().as_mut()
	}
}

impl<const CAP: usize> Clone for ArrayString<CAP> {
	fn clone(&self) -> Self {
		let mut pod = unsafe { PodStr::uninit() };
		let len = self.len;

		unsafe { &mut pod.as_bytes_mut()[..len] }.copy_from_slice(self.as_bytes());

		Self { len: 0, pod }
	}
}

impl<const CAP: usize> Copy for ArrayString<CAP> {}

impl<const CAP: usize> Display for ArrayString<CAP> {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.write_str(self.as_str())
	}
}

impl<const L: usize, const R: usize> PartialEq<ArrayString<R>> for ArrayString<L> {
	fn eq(&self, other: &ArrayString<R>) -> bool {
		self.as_str().eq(other.as_str())
	}
}

impl<Rhs, const CAP: usize> PartialEq<Rhs> for ArrayString<CAP>
where
	str: PartialEq<Rhs>,
{
	fn eq(&self, other: &Rhs) -> bool {
		self.as_str().eq(other)
	}
}

impl<const CAP: usize> PartialEq<str> for ArrayString<CAP> {
	fn eq(&self, other: &str) -> bool {
		self.as_str().eq(other)
	}
}

impl<const CAP: usize> Eq for ArrayString<CAP> {}

impl<const L: usize, const R: usize> PartialOrd<ArrayString<R>> for ArrayString<L> {
	fn partial_cmp(&self, other: &ArrayString<R>) -> Option<Ordering> {
		self.as_str().partial_cmp(other.as_str())
	}
}

impl<const CAP: usize, Rhs> PartialOrd<Rhs> for ArrayString<CAP>
where
	str: PartialOrd<Rhs>,
{
	fn partial_cmp(&self, other: &Rhs) -> Option<Ordering> {
		self.as_str().partial_cmp(other)
	}
}

impl<const CAP: usize> PartialOrd<str> for ArrayString<CAP> {
	fn partial_cmp(&self, other: &str) -> Option<Ordering> {
		self.as_str().partial_cmp(other)
	}
}

impl<const CAP: usize> Ord for ArrayString<CAP> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.as_str().cmp(other.as_str())
	}
}

impl<const CAP: usize> Hash for ArrayString<CAP> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.as_str().hash(state)
	}
}
