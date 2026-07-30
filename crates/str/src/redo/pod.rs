use crate::util::*;
use core::cmp::Ordering;
use core::convert::Infallible;
use core::fmt::{Debug, Display, Formatter};
use core::hash::{Hash, Hasher};
use core::mem::{ManuallyDrop, MaybeUninit, offset_of};
use core::ops::{Deref, DerefMut};
use core::ptr::{copy_nonoverlapping, write_bytes};
use core::slice::{from_raw_parts, from_raw_parts_mut};
use core::str::Utf8Error;
use core::str::{from_utf8_unchecked, from_utf8_unchecked_mut};
use zerocopy::{Unalign, Unaligned};

/// Simple use cases are possible with [`ArrayStr`] and [`ArrayString`].
///
/// [`ArrayStr`]: crate::array_str::ArrayStr
/// [`ArrayString`]: crate::array_string::ArrayString
///
/// Any piece-of-data utilized as a `str`.
/// Behaves like an inlined `str` buffer where `T` is utilized as a type-level state.
#[derive(Unaligned)]
#[cfg_attr(not(feature = "c_abi"), repr(transparent))]
#[cfg_attr(feature = "c_abi", repr(C))]
pub struct PodStr<T: PodStrBuffer>(MaybeUninit<T::Buffer>);

impl<T: PodStrBuffer> PodStr<T> {
	///
	pub const DEFAULT: Self = Self::from_fill_ascii(0x1A).unwrap();
	pub const SPACE: Self = Self::from_fill_ascii(b' ').unwrap();
	pub const NUL: Self = Self::zeroed();

	/// Creates a string of nul-bytes using an unaligned layout of `T` as a buffer.
	#[inline]
	pub const fn new() -> Self {
		const { Self::from_fill_ascii(0x1A).unwrap() }
	}

	/// Creates a string of nul-bytes using an unaligned layout of `T` as a buffer.
	#[inline]
	pub const fn zeroed() -> Self {
		const_debug_assert_eq!(const: align_of::<T::Buffer>(), 1);
		const_debug_assert!(const: size_of::<T>() < isize::MAX as usize);

		Self(MaybeUninit::zeroed())
	}

	// /// Creates a string of nul-bytes using an unaligned layout of `T` as a buffer.
	// #[inline]
	// #[doc(alias("new_nul", "new_zeroed"))]
	// pub const fn zeroed() -> Self {
	// 	const_debug_assert_eq!(const: align_of::<T::Buffer>(), 1);
	//
	// 	Self(MaybeUninit::zeroed())
	// }

	/// Builds a `PodStr` by filling the buffer with the specified `char`.
	/// Returns `None` if the buffer's size is not a multiple of [`char::len_utf8`].
	pub const fn from_fill(char: char) -> Option<Self> {
		const_debug_assert!(const: size_of::<T>() < isize::MAX as usize);

		if const { size_of::<T>() == 0 } {
			return Some(Self::zeroed());
		}

		const {
			FillPod::<MaybeUninit<T::Buffer>, 1>::assert::<T>();
			FillPod::<MaybeUninit<T::Buffer>, 2>::assert::<T>();
			FillPod::<MaybeUninit<T::Buffer>, 3>::assert::<T>();
			FillPod::<MaybeUninit<T::Buffer>, 4>::assert::<T>();
		};

		match [const { size_of::<T>() }, char.len_utf8()] {
			[_, 1] if char == '\x00' => Some(Self::zeroed()),

			[1, 1] => {
				const_assert!(char.is_ascii());
				Some(Self(unsafe { FillPod::<MaybeUninit<T::Buffer>, 1>::single_ascii(char as u8) }))
			}

			[_, 1] => {
				const_assert!(char.is_ascii());
				Some(Self(unsafe { FillPod::<MaybeUninit<T::Buffer>, 1>::filled_ascii(char as u8) }))
			}

			[2, 2] => Some(Self(unsafe { FillPod::<MaybeUninit<T::Buffer>, 2>::single(char) })),
			[len, 2] if len.is_multiple_of(2) => Some(Self(unsafe { FillPod::<MaybeUninit<T::Buffer>, 2>::filled(char) })),

			[3, 3] => Some(Self(unsafe { FillPod::<MaybeUninit<T::Buffer>, 3>::single(char) })),
			[len, 3] if len.is_multiple_of(3) => Some(Self(unsafe { FillPod::<MaybeUninit<T::Buffer>, 3>::filled(char) })),

			[4, 4] => Some(Self(unsafe { FillPod::<MaybeUninit<T::Buffer>, 4>::single(char) })),
			[len, 4] if len.is_multiple_of(4) => Some(Self(unsafe { FillPod::<MaybeUninit<T::Buffer>, 4>::filled(char) })),

			// UTF-8 boundary violations
			[(1..), (1..=4)] => None,
			_ => unreachable!(),
		}
	}

	/// Generic version of [`from_fill`]
	///
	/// [`from_fill`]: Self::from_fill
	/// # Panics
	/// If the buffer's size is not a multiple of [`char::len_utf8`] for `CHAR`.
	pub const fn from_fill_const<const CHAR: char>() -> Self {
		const { Self::from_fill(CHAR).unwrap() }
	}

	/// Returns `None` if `byte` is not a valid ASCII 8-bit bit pattern.
	pub const fn from_fill_ascii(byte: u8) -> Option<Self> {
		const_debug_assert!(const: size_of::<T>() < isize::MAX as usize);

		if !byte.is_ascii() {
			return None;
		}

		let mut slot = MaybeUninit::uninit();

		unsafe { write_bytes(slot.as_mut_ptr(), 0x1A, 1) };

		Some(Self(slot))
	}

	/// Generic version of [`from_fill_ascii`]
	///
	/// [`from_fill_ascii`]: Self::from_fill_ascii
	/// # Panics
	/// If `BYTE` is not a valid ASCII 8-bit bit pattern.
	pub const fn from_fill_ascii_const<const BYTE: u8>() -> Self {
		const { Self::from_fill_ascii(BYTE).unwrap() }
	}

	/// Errors if the provided bytes are not valid UTF-8.
	///
	/// Fails constant evaluation if `LEN` is not the exact same size as `T`.
	pub const fn from_utf8_array<const LEN: usize>(bytes: [u8; LEN]) -> Result<Self, Utf8Error> {
		const_assert_eq!(const: LEN, size_of::<T>());

		match str::from_utf8(&bytes) {
			Ok(_) => Ok(unsafe { Self::from_utf8_array_unchecked(bytes) }),
			Err(error) => Err(error),
		}
	}

	/// Errors if the provided bytes are not valid UTF-8.
	///
	/// # Panics
	/// If `len` of slice does not exactly match the buffer's size.
	pub const fn from_utf8_slice(bytes: &[u8]) -> Result<Self, Utf8Error> {
		match bytes.split_at(const { size_of::<T>() }) {
			(bytes, []) => match str::from_utf8(&bytes) {
				Ok(_) => Ok(unsafe { Self::from_utf8_slice_unchecked(bytes) }),
				Err(error) => Err(error),
			},

			(_, [..]) => {
				panic!("Slice too large for destination buffer")
			}
		}
	}

	/// # Safety
	/// - Bytes must be valid UTF-8.
	/// - `LEN` should match the size of `T`,
	///   - This is checked at compile time, but `cargo check` may miss it.
	/// - The size of `T` must be below [`isize::MAX`]
	///   - For conversion to `[u8]`
	#[inline]
	pub const unsafe fn from_utf8_array_unchecked<const LEN: usize>(bytes: [u8; LEN]) -> Self {
		const_debug_assert!(const: LEN < isize::MAX as usize);
		const_debug_assert_eq!(const: LEN, size_of::<T>());

		Self(unsafe { transmute2(bytes) })
	}

	/// # Safety
	/// - Bytes must be valid UTF-8.
	/// - The `len` of `bytes` must exactly match the size of `T`
	/// - The size of `T` must be below [`isize::MAX`]
	///   - For conversion to `[u8]`
	#[inline]
	pub const unsafe fn from_utf8_slice_unchecked(bytes: &[u8]) -> Self {
		const_debug_assert!(const: size_of::<T>() < isize::MAX as usize);
		const_debug_assert_eq!(bytes.len(), size_of::<T>());

		let mut buffer = MaybeUninit::<T::Buffer>::uninit();
		let dst = unsafe { from_raw_parts_mut(buffer.as_mut_ptr().cast::<u8>(), const { size_of::<T>() }) };

		dst.copy_from_slice(bytes);

		Self(buffer)
	}

	/// # Safety
	/// Contents must be initialized before accessing as a `str`.
	#[inline]
	pub const unsafe fn uninit() -> Self {
		const_debug_assert_eq!(const: align_of::<T::Buffer>(), 1);

		Self(MaybeUninit::<T::Buffer>::uninit())
	}

	#[inline]
	pub const fn as_bytes(&self) -> &[u8] {
		const_debug_assert!(const: size_of::<T>() < isize::MAX as usize);
		unsafe { from_raw_parts(self.as_ptr(), const { size_of::<T>() }) }
	}

	#[inline]
	pub const unsafe fn as_bytes_mut(&mut self) -> &mut [u8] {
		const_debug_assert!(const: size_of::<T>() < isize::MAX as usize);
		unsafe { from_raw_parts_mut(self.as_mut_ptr(), const { size_of::<T>() }) }
	}

	#[inline(always)]
	const unsafe fn as_fill<const N: usize>(&mut self) -> &mut FillPod<MaybeUninit<T::Buffer>, N> {
		unsafe { transmute2::<&mut Self, &mut FillPod<MaybeUninit<T::Buffer>, N>>(self) }
	}

	#[inline(always)]
	pub const fn as_ptr(&self) -> *const u8 {
		self.0.as_ptr().cast::<u8>()
	}

	#[inline(always)]
	pub const fn as_mut_ptr(&mut self) -> *mut u8 {
		self.0.as_mut_ptr().cast::<u8>()
	}

	#[inline]
	pub const fn as_str(&self) -> &str {
		const_debug_assert!(const: size_of::<T>() < isize::MAX as usize);
		unsafe { from_utf8_unchecked(from_raw_parts(self.as_ptr(), const { size_of::<T>() })) }
	}

	#[inline]
	pub const fn as_mut_str(&mut self) -> &mut str {
		const_debug_assert!(const: size_of::<T>() < isize::MAX as usize);
		unsafe { from_utf8_unchecked_mut(from_raw_parts_mut(self.as_mut_ptr(), const { size_of::<T>() })) }
	}

	/// Provides mutable access to the underlying buffer.
	/// See [`LossyMutBytes`] for details.
	pub const fn bytes_mut_lossy(&mut self) -> LossyMutBytes<'_, T> {
		LossyMutBytes { pod: self }
	}

	/// Provides mutable access to the underlying buffer.
	/// A copy is immediately made upon creation.
	/// See [`ReversibleMutBytes`] for details.
	pub const fn bytes_mut_reversible(&mut self) -> ReversibleMutBytes<'_, T> {
		ReversibleMutBytes {
			edited: *self,
			original: self,
		}
	}

	/// Provides mutable access to the underlying buffer.
	/// See [`StrictMutBytes`] for details.
	pub const fn bytes_mut_strict(&mut self) -> StrictMutBytes<'_, T> {
		StrictMutBytes { pod: self }
	}

	#[inline(always)]
	pub const fn clear(&mut self) {
		self.0 = MaybeUninit::zeroed();
	}

	/// Alias of [`Self::push`].
	#[inline(always)]
	pub const fn concat<U: PodStrBuffer>(self, item: PodStr<U>) -> PodStr<T::Push<U>>
	where
		T::Push<U>: PodStrBuffer,
	{
		self.push(item)
	}

	#[inline]
	pub const fn fill(&mut self, char: char) {
		*self = Self::from_fill(char).unwrap();
	}

	#[inline]
	pub const fn fill_ascii(&mut self, byte: u8) {
		const_assert!(byte.is_ascii());

		match const { size_of::<T>() } {
			0 => { /* do nothing */ }
			1 => {
				self.0.write(unsafe { transmute2(byte) });
			}
			_ => unsafe { self.as_fill::<1>().fill_ascii(byte) },
		}
	}

	pub const fn first_char(&self) -> char {
		todo!()
	}

	/// # Safety
	/// - Ensure `N <= LEN`
	/// - Bytes must be kept as valid UTF-8.
	#[inline]
	pub const unsafe fn first_chunk_bytes_mut<const N: usize>(&mut self) -> &mut [u8; N] {
		unsafe { self.as_bytes_mut().first_chunk_mut::<N>().unwrap_unchecked() }
	}

	pub const fn into_first_chunk<const N: usize>(self) -> [u8; N] {
		const { ["Chunk size must be equal or smaller to the buffer size"][N.saturating_sub(size_of::<T>())] };
		unsafe { self.into_first_chunk_unchecked() }
	}

	/// # Safety
	/// The following must be upheld: `N <= size_of::<T>()`,
	/// otherwise the tail portion of the array will be constructed from uninitialized memory.
	pub const unsafe fn into_first_chunk_unchecked<const N: usize>(self) -> [u8; N] {
		union FirstChunkTransmute<T: PodStrBuffer, const N: usize> {
			whole: PodStr<T>,
			chunk: [u8; N],
		}

		const { ["Offset assertion"][offset_of!(FirstChunkTransmute<T, N>, whole) + offset_of!(FirstChunkTransmute<T, N>, chunk)] };
		unsafe { FirstChunkTransmute { whole: self }.chunk }
	}

	/// Same as [`str::is_ascii`].
	#[inline]
	pub const fn is_ascii(&self) -> bool {
		self.as_bytes().is_ascii()
	}

	/// Same as [`str::is_char_boundary`].
	#[inline]
	pub const fn is_char_boundary(&self, index: usize) -> bool {
		self.as_str().is_char_boundary(index)
	}

	/// Same as [`str::is_empty`].
	#[inline]
	pub const fn is_empty(&self) -> bool {
		const { size_of::<T>() == 0 }
	}

	/// # Safety
	/// Same as [`first_chunk_bytes_mut`].
	///
	/// [`first_chunk_bytes_mut`]: Self::first_chunk_bytes_mut
	#[inline]
	pub const unsafe fn last_chunk_bytes_mut<const N: usize>(&mut self) -> &mut [u8; N] {
		unsafe { self.as_bytes_mut().last_chunk_mut::<N>().unwrap_unchecked() }
	}

	#[inline]
	pub const fn len(&self) -> usize {
		const { size_of::<T>() }
	}

	#[inline(always)]
	pub const fn last(self) -> PodStr<T::Pop>
	where
		T::Residual: PodStrBuffer,
		T::Pop: PodStrBuffer,
	{
		self.pop().1
	}

	/// # Panics
	/// If the `PodStr` is empty.
	pub const fn last_char(&self) -> char {
		const_assert!(const: const { size_of::<T>() != 0 });

		let bytes = self.as_bytes();
		let mut start = { size_of::<T>().saturating_sub(1) };

		while start != 0 && bytes[start] & 0b1100_0000 == 0b1000_0000 {
			start -= 1;
		}

		let code_point = match const { size_of::<T>() } - start {
			1 => bytes[start] as u32,

			2 => ((bytes[start] & 0b0001_1111) as u32) << 6 | ((bytes[start + 1] & 0b0011_1111) as u32),

			3 => {
				((bytes[start] & 0b0000_1111) as u32) << 12
					| ((bytes[start + 1] & 0b0011_1111) as u32) << 6
					| ((bytes[start + 2] & 0b0011_1111) as u32)
			}

			4 => {
				((bytes[start] & 0b0000_0111) as u32) << 18
					| ((bytes[start + 1] & 0b0011_1111) as u32) << 12
					| ((bytes[start + 2] & 0b0011_1111) as u32) << 6
					| ((bytes[start + 3] & 0b0011_1111) as u32)
			}

			// SAFETY: valid UTF-8 code points occupy at most four bytes,
			//  `PodStr` should only ever contain valid UTF-8 when this function is accessible
			_ => unsafe { debug_unreachable!() },
		};

		unsafe { char::from_u32_unchecked(code_point) }
	}

	#[inline(always)]
	pub const fn pop(self) -> (PodStr<T::Residual>, PodStr<T::Pop>)
	where
		T::Residual: PodStrBuffer,
		T::Pop: PodStrBuffer,
	{
		unsafe { transmute2::<Self, (PodStr<T::Residual>, PodStr<T::Pop>)>(self) }
	}

	#[inline(always)]
	pub const fn pop_discard(self) -> PodStr<T::Residual>
	where
		T::Residual: PodStrBuffer,
		T::Pop: PodStrBuffer,
	{
		self.pop().0
	}

	#[inline(always)]
	pub const fn pop_into(self, slot: &mut Option<PodStr<T::Pop>>) -> PodStr<T::Residual>
	where
		T::Residual: PodStrBuffer,
		T::Pop: PodStrBuffer,
	{
		let (residual, pop) = self.pop();
		*slot = Some(pop);

		residual
	}

	#[inline(always)]
	pub const fn push<U: PodStrBuffer>(self, item: PodStr<U>) -> PodStr<T::Push<U>>
	where
		T::Push<U>: PodStrBuffer,
	{
		unsafe { transmute2::<(PodStr<T>, PodStr<U>), PodStr<T::Push<U>>>((self, item)) }
	}

	#[inline(always)]
	pub const fn push_from<U: PodStrBuffer>(self, item: &mut Option<PodStr<U>>) -> Result<PodStr<T::Push<U>>, Self>
	where
		T::Push<U>: PodStrBuffer,
	{
		match item.take() {
			None => Err(self),
			Some(item) => Ok(self.push(item)),
		}
	}

	/// Writes `src` into the buffer, overwriting an existing contents.
	///
	/// # Panics
	/// If the end of `src` splits a UTF-8 code point.
	///
	/// ```rust
	/// # use itym_str::pod::PodStr;
	/// # fn main() {
	/// let mut buffer = PodStr::<[u8; 26]>::zeroed();
	///
	/// buffer.write_str("Meet me at the simulacrum.");
	/// buffer.write_str("Hello world!");
	///
	/// assert_eq!(&*buffer, "Hello world!he simulacrum.");
	/// # }
	/// ```
	pub const fn write_str(&mut self, src: &str) {
		let end = src.len();

		if !self.is_char_boundary(end) {
			panic!("`src` splits character boundary");
		}

		unsafe { self.as_bytes_mut() }.split_at_mut(end).0.copy_from_slice(src.as_bytes());
	}

	/// Writes `src` into the buffer at the specified position, overwriting an existing contents.
	///
	/// # Panics
	/// If the start or end of the written slice splits a UTF-8 code point.
	///
	/// ```rust
	/// # use itym_str::pod::PodStr;
	/// # fn main() {
	/// let mut buffer = PodStr::<[u8; 26]>::zeroed();
	///
	/// buffer.write_str("Meet me at the simulacrum.");
	/// buffer.write_str_at(5, "Hello world!");
	///
	/// assert_eq!(&*buffer, "Meet Hello world!mulacrum.");
	/// # }
	/// ```
	pub const fn write_str_at(&mut self, start: usize, src: &str) {
		let end = src.len() + start;

		if !self.is_char_boundary(start) {
			panic!("start splits character boundary");
		} else if !self.is_char_boundary(end) {
			panic!("end splits character boundary");
		}

		unsafe { self.as_bytes_mut() }
			.split_at_mut(end)
			.0
			.split_at_mut(start)
			.1
			.copy_from_slice(src.as_bytes());
	}
}

impl<const LEN: usize> PodStr<[u8; LEN]> {
	/// # Panics
	/// If the given `byte_str_literal` is not valid UTF-8.
	pub const fn lit(byte_str_literal: &'static [u8; LEN]) -> Self {
		match Self::from_utf8_array(*byte_str_literal) {
			Ok(pod) => pod,
			Err(..) => panic!("PodStr::lit given invalid UTF-8 byte sequence"),
		}
	}
}

impl<T: PodStrBuffer> Deref for PodStr<T> {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.as_str()
	}
}

impl<T: PodStrBuffer> DerefMut for PodStr<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.as_mut_str()
	}
}

impl<T: PodStrBuffer, U> AsRef<U> for PodStr<T>
where
	str: AsRef<U>,
{
	fn as_ref(&self) -> &U {
		self.as_str().as_ref()
	}
}

impl<T: PodStrBuffer, U> AsMut<U> for PodStr<T>
where
	str: AsMut<U>,
{
	fn as_mut(&mut self) -> &mut U {
		self.as_mut_str().as_mut()
	}
}

impl<T: PodStrBuffer> Clone for PodStr<T> {
	fn clone(&self) -> Self {
		Self(self.0)
	}
}

impl<T: PodStrBuffer> Copy for PodStr<T> {}

#[cfg(not(feature = "alloc"))]
impl<T: PodStrBuffer> Debug for PodStr<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		match str::from_utf8(self.as_bytes()) {
			Ok(str) => f.debug_tuple("PodStr").field(&str).finish(),
			Err(..) => f
				.debug_tuple("#[lossy]\nPodStr")
				.field(&core::fmt::from_fn(|f| {
					for chunk in self.as_bytes().utf8_chunks() {
						let valid = chunk.valid();

						if !valid.is_empty() {
							f.write_str(chunk.valid())?;
						}

						if !chunk.invalid().is_empty() {
							f.write_str("\u{FFFD}")?; //char::REPLACEMENT_CHARACTER
						}
					}

					Ok(())
				}))
				.finish(),
		}
	}
}

impl<T: PodStrBuffer> Display for PodStr<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.write_str(self.as_str())
	}
}

impl<T: PodStrBuffer, U: PodStrBuffer> PartialEq<PodStr<U>> for PodStr<T> {
	fn eq(&self, other: &PodStr<U>) -> bool {
		self.as_str().eq(other.as_str())
	}
}

impl<T: PodStrBuffer, Rhs> PartialEq<Rhs> for PodStr<T>
where
	str: PartialEq<Rhs>,
{
	fn eq(&self, other: &Rhs) -> bool {
		self.as_str().eq(other)
	}
}

impl<T: PodStrBuffer> PartialEq<str> for PodStr<T> {
	fn eq(&self, other: &str) -> bool {
		self.as_str().eq(other)
	}
}

impl<T: PodStrBuffer> Eq for PodStr<T> {}

impl<T: PodStrBuffer, U: PodStrBuffer> PartialOrd<PodStr<U>> for PodStr<T> {
	fn partial_cmp(&self, other: &PodStr<U>) -> Option<Ordering> {
		self.as_str().partial_cmp(other.as_str())
	}
}

impl<T: PodStrBuffer, Rhs> PartialOrd<Rhs> for PodStr<T>
where
	str: PartialOrd<Rhs>,
{
	fn partial_cmp(&self, other: &Rhs) -> Option<Ordering> {
		self.as_str().partial_cmp(other)
	}
}

impl<T: PodStrBuffer> PartialOrd<str> for PodStr<T> {
	fn partial_cmp(&self, other: &str) -> Option<Ordering> {
		self.as_str().partial_cmp(other)
	}
}

impl<T: PodStrBuffer> Ord for PodStr<T> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.as_bytes().cmp(other.as_bytes())
	}
}

impl<T: PodStrBuffer> Hash for PodStr<T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.as_str().hash(state)
	}
}

/// Implemented by types which act as the state of a [`PodStr`].
pub unsafe trait PodStrBuffer: Copy + Sized {
	type Buffer: Copy + zerocopy::Unaligned;
	type Pop;
	type Residual;
	type Push<A>;
}

unsafe impl PodStrBuffer for () {
	type Buffer = ();
	type Pop = ();
	type Residual = ();
	type Push<A> = Single<A>;
}

unsafe impl<T: Copy> PodStrBuffer for Single<T> {
	type Buffer = Unalign<T>;
	type Pop = Self;
	type Residual = ();
	type Push<A> = (T, A);
}

unsafe impl<const LEN: usize> PodStrBuffer for [u8; LEN] {
	type Buffer = Self;
	type Pop = Self;
	type Residual = ();
	type Push<A> = (Self, A);
}

unsafe impl<T0: Copy, T1: Copy> PodStrBuffer for (T0, T1) {
	type Buffer = Unalign<(Unalign<T0>, Unalign<T1>)>;
	type Pop = Single<T1>;
	type Residual = Single<T0>;
	type Push<A> = (T0, T1, A);
}

//TODO: macro expansion for PodStrBuffer tuples N=3 and above
unsafe impl<T0: Copy, T1: Copy, T2: Copy> PodStrBuffer for (T0, T1, T2) {
	type Buffer = Unalign<(Unalign<T0>, Unalign<T1>, Unalign<T2>)>;
	type Pop = Single<T2>;
	type Residual = (T0, T1);
	type Push<A> = (T0, T1, T2, A);
}

unsafe impl<T0: Copy, T1: Copy, T2: Copy, T3: Copy> PodStrBuffer for (T0, T1, T2, T3) {
	type Buffer = Unalign<(Unalign<T0>, Unalign<T1>, Unalign<T2>, Unalign<T3>)>;
	type Pop = Single<T3>;
	type Residual = (T0, T1, T2);
	// type Push<A: PodStrBuffer> = (T0, T1, T2, T3, A);
	type Push<A> = Infallible;
}

/// Type-level workaround for implementing traits on tuples.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Single<T>(T);

/// Replaces invalid UTF-8 sequences with nul (`\x00`) bytes on drop.
///
/// To use this in `const`, call [`finish`].
///
/// [`finish`]: Self::finish
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(not(feature = "c_abi"), repr(transparent))]
#[cfg_attr(feature = "c_abi", repr(C))]
pub struct LossyMutBytes<'a, T: PodStrBuffer> {
	pod: &'a mut PodStr<T>,
}

impl<'a, T: PodStrBuffer> LossyMutBytes<'a, T> {
	#[inline(always)]
	const fn complete(&mut self) {
		self.repair_lossy();
	}

	const unsafe fn get(&self) -> &PodStr<T> {
		&self.pod
	}

	const unsafe fn get_mut(&mut self) -> &mut PodStr<T> {
		&mut self.pod
	}

	/// Same as [`repair_lossy`]
	///
	/// [`repair_lossy`]: Self::repair_lossy
	pub const fn repair(&mut self) -> &mut str {
		self.repair_lossy()
	}
}

/// Only updates the [`PodStr`] if the underlying edits are valid UTF-8.
///
/// To use this in `const`, call [`finish`].
///
/// [`finish`]: Self::finish
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "c_abi", repr(C))]
pub struct ReversibleMutBytes<'a, T: PodStrBuffer> {
	original: &'a mut PodStr<T>,
	edited: PodStr<T>,
}

impl<'a, T: PodStrBuffer> ReversibleMutBytes<'a, T> {
	const fn complete(&mut self) {
		if !self.is_damaged() {
			*self.original = self.edited;
		}
	}

	const unsafe fn get(&self) -> &PodStr<T> {
		&self.edited
	}

	const unsafe fn get_mut(&mut self) -> &mut PodStr<T> {
		&mut self.edited
	}

	pub const fn original(&self) -> &PodStr<T> {
		&*self.original
	}

	/// Restores the buffer back to its original content.
	///
	/// [`repair_lossy`]: Self::repair_lossy
	pub const fn repair(&mut self) -> &mut str {
		self.edited = *self.original;

		self.edited.as_mut_str()
	}
}

/// Panics if the buffer is not valid UTF-8 when dropped.
///
/// To use this in `const`, call [`finish`].
///
/// [`finish`]: Self::finish
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(not(feature = "c_abi"), repr(transparent))]
#[cfg_attr(feature = "c_abi", repr(C))]
pub struct StrictMutBytes<'a, T: PodStrBuffer> {
	pod: &'a mut PodStr<T>,
}

impl<'a, T: PodStrBuffer> StrictMutBytes<'a, T> {
	#[inline(always)]
	const fn complete(&mut self) {
		self.repair();
	}

	const unsafe fn get(&self) -> &PodStr<T> {
		&self.pod
	}

	const unsafe fn get_mut(&mut self) -> &mut PodStr<T> {
		&mut self.pod
	}

	pub const fn repair(&mut self) -> &mut str {
		if self.is_damaged() {
			panic!("StrictMutBytes dropped while containing invalid UTF-8 byte sequence");
		} else {
			self.pod.as_mut_str()
		}
	}
}

macro_rules! impl_mut_bytes {
	(macro $g_lt:lifetime $g_gen:ident $target:ty as $rename:ident $($item:item)* $($next:ty $(, $tail:ty)*)?) => {
		const _: () = {
			type $rename<$g_lt, $g_gen> = $target;

			$($item)*

			()
		};
	};

	(
		impl <$g_lt:lifetime, $g_gen:ident> $target:ty $(, $tail:ty)+ as $rename:ident;

		$($item:item)*
	) => {
		impl_mut_bytes! { macro $g_lt $g_gen $target as $rename $($item)* }
		impl_mut_bytes! { impl <$g_lt, $g_gen> $($tail),+ as $rename; $($item)* }
	};

	(
		impl <$g_lt:lifetime, $g_gen:ident> $target:ty as $rename:ident;

		$($item:item)*
	) => {
		impl_mut_bytes! { macro $g_lt $g_gen $target as $rename $($item)* }
	};
}

impl_mut_bytes! {
	impl<'a, T> LossyMutBytes<'a, T>, StrictMutBytes<'a, T>, ReversibleMutBytes<'a, T> as MutBytes;

	impl<'a, T: PodStrBuffer> MutBytes<'a, T> {
		pub const fn as_bytes(&self) -> &[u8] {
			unsafe { self.get() }.as_bytes()
		}

		#[inline(always)]
		pub const fn as_bytes_mut(&mut self) -> &mut [u8] {
			unsafe { self.get_mut().as_bytes_mut() }
		}

		#[inline(always)]
		pub const fn as_str(&self) -> Result<&str, Utf8Error> {
			str::from_utf8(unsafe { self.get() }.as_bytes())
		}

		#[inline(always)]
		pub const fn as_str_mut(&mut self) -> Result<&mut str, Utf8Error> {
			str::from_utf8_mut(unsafe { self.get_mut().as_bytes_mut() })
		}

		pub const fn finish(mut self) {
			if self.is_damaged() {
				self.repair();
			} else {
				self.complete();
			}

			core::mem::forget(self);
		}

		#[inline(always)]
		pub const fn is_damaged(&self) -> bool {
			str::from_utf8(unsafe { self.get() }.as_bytes()).is_err()
		}

		/// Replaces invalid sequences with the nul-byte `'\x00'`.
		pub const fn repair_lossy(&mut self) -> &mut str {
			let mut march = self.as_bytes_mut();

			while let Err(error) = str::from_utf8(march) {
				let (_valid, tail) = march.split_at_mut(error.valid_up_to());
				let step = match error.error_len() {
					None => 1,
					Some(len) => len,
				};

				let (invalid, tail) = tail.split_at_mut(step);

				unsafe { write_bytes(invalid.as_mut_ptr().cast::<u8>(), 0x1A, invalid.len()) };

				march = tail;
			}

			unsafe { self.get_mut() }.as_mut_str()
		}
	}

	impl<'a, T: PodStrBuffer> Deref for MutBytes<'a, T> {
		type Target = [u8];

		fn deref(&self) -> &Self::Target {
			self.as_bytes()
		}
	}

	impl<'a, T: PodStrBuffer> DerefMut for MutBytes<'a, T> {
		fn deref_mut(&mut self) -> &mut Self::Target {
			self.as_bytes_mut()
		}
	}

	impl<'a, T: PodStrBuffer> Drop for MutBytes<'a, T> {
		fn drop(&mut self) {
			if self.is_damaged() {
				self.repair();
			}
		}
	}
}

/// Utility for filling a [`PodStr`].
union FillPod<B, const N: usize> {
	uninit: (),
	byte: u8,
	array: [u8; N],
	buffer: ManuallyDrop<B>,
}

impl<B, const N: usize> FillPod<B, N> {
	const fn assert<T: PodStrBuffer>() {
		["offset assertion"][const {
			(
				//1
				offset_of!(FillPod<MaybeUninit<T::Buffer>, 1>, byte)
					+ offset_of!(FillPod<MaybeUninit<T::Buffer>, 1>, array)
					+ offset_of!(FillPod<MaybeUninit<T::Buffer>, 1>, buffer)
			) + (
				//2
				offset_of!(FillPod<MaybeUninit<T::Buffer>, 2>, byte)
					+ offset_of!(FillPod<MaybeUninit<T::Buffer>, 2>, array)
					+ offset_of!(FillPod<MaybeUninit<T::Buffer>, 2>, buffer)
			) + (
				//3
				offset_of!(FillPod<MaybeUninit<T::Buffer>, 3>, byte)
					+ offset_of!(FillPod<MaybeUninit<T::Buffer>, 3>, array)
					+ offset_of!(FillPod<MaybeUninit<T::Buffer>, 3>, buffer)
			) + (
				//4
				offset_of!(FillPod<MaybeUninit<T::Buffer>, 4>, byte)
					+ offset_of!(FillPod<MaybeUninit<T::Buffer>, 4>, array)
					+ offset_of!(FillPod<MaybeUninit<T::Buffer>, 4>, buffer)
			)
		}];
	}

	#[inline(always)]
	const unsafe fn single(char: char) -> B {
		const_debug_assert_eq!(char.len_utf8(), size_of::<B>());

		let mut slot = Self { uninit: () };

		unsafe { slot.set_char(char) };
		ManuallyDrop::into_inner(unsafe { slot.buffer })
	}

	#[inline(always)]
	const unsafe fn fill(&mut self, char: char) {
		const_debug_assert_ne!(size_of::<B>(), 0);

		unsafe { self.set_char(char) };

		let (src, dst) = self.as_chunk_thin_ptrs();

		if const { size_of::<B>() / N == 0 } {
			return;
		}

		unsafe { copy_nonoverlapping(src, dst, const { (size_of::<B>() / N).saturating_sub(1) }) };
	}

	#[inline(always)]
	const fn as_chunk_thin_ptrs(&mut self) -> (*mut [MaybeUninit<u8>; N], *mut [MaybeUninit<u8>; N]) {
		let ptr: *mut [MaybeUninit<u8>; N] = (self as *mut Self).cast();

		(ptr, unsafe { ptr.add(1) })
	}

	#[inline(always)]
	const unsafe fn filled(char: char) -> B {
		let mut slot = Self { uninit: () };

		unsafe { slot.fill(char) };
		ManuallyDrop::into_inner(unsafe { slot.buffer })
	}

	/// # Safety
	/// `char::len_utf8` must equal `N`
	#[inline(always)]
	const unsafe fn set_char(&mut self, char: char) {
		const_debug_assert_eq!(unsafe: char.len_utf8(), N);
		char.encode_utf8(unsafe { &mut self.array });
	}
}

impl<B> FillPod<B, 1> {
	const unsafe fn single_ascii(byte: u8) -> B {
		const_debug_assert_eq!(unsafe: 1, size_of::<B>());
		const_debug_assert!(unsafe: byte.is_ascii());

		ManuallyDrop::into_inner(unsafe { Self { byte }.buffer })
	}

	#[inline(always)]
	const unsafe fn fill_ascii(&mut self, byte: u8) {
		const_debug_assert_ne!(unsafe: size_of::<B>(), 0);
		const_debug_assert!(unsafe: byte.is_ascii());

		unsafe { write_bytes(&raw mut self.buffer, byte, 1) };
	}

	#[inline(always)]
	const unsafe fn filled_ascii(byte: u8) -> B {
		let mut slot = Self { uninit: () };

		unsafe { slot.fill_ascii(byte) }
		ManuallyDrop::into_inner(unsafe { slot.buffer })
	}
}

#[cfg(feature = "alloc")]
mod requires_alloc {
	use super::*;
	use crate::alloc;
	use alloc::borrow::Cow;
	use alloc::boxed::Box;
	use alloc::string::String;
	use core::ptr::slice_from_raw_parts_mut;

	impl<T: PodStrBuffer> PodStr<T> {
		#[inline]
		pub fn into_alloc_thin_ptr(self) -> *mut u8 {
			debug_assert_eq!(align_of::<T::Buffer>(), 1);
			Box::into_raw(Box::new(self.0)).cast::<u8>()
		}

		#[inline]
		pub fn into_alloc_wide_ptr(self) -> *mut [u8] {
			const_debug_assert!(const: size_of::<T>() < isize::MAX as usize);
			slice_from_raw_parts_mut(self.into_alloc_thin_ptr(), const { size_of::<T>() })
		}

		pub fn into_boxed_str(self) -> Box<str> {
			const_debug_assert_eq!(const: align_of::<T>(), 1);
			const_debug_assert!(const: size_of::<T>() < isize::MAX as usize);

			let data = Box::into_raw(Box::new(self.0)).cast::<u8>();
			let ptr = slice_from_raw_parts_mut(data, const { size_of::<T>() });

			// SAFETY:
			// - `Box<T>` and `Box<[u8]>` use the same allocation size;
			// - the caller guarantees that T's alignment is 1;
			// - T: Copy means no destructor is being skipped.
			let boxed: Box<[u8]> = unsafe { Box::from_raw(ptr) };

			// SAFETY: safety contract is presented during construction
			//  the entire buffer is assumed to be a valid `str` slice
			unsafe { alloc::str::from_boxed_utf8_unchecked(boxed) }
		}

		pub fn into_string(self) -> String {
			self.into_boxed_str().into_string()
		}
	}

	impl<T: PodStrBuffer> Debug for PodStr<T> {
		fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
			match String::from_utf8_lossy(self.as_bytes()) {
				Cow::Borrowed(str) => f.debug_tuple("PodStr").field(&str).finish(),
				Cow::Owned(string) => f.debug_tuple("#[lossy]\nPodStr").field(&string.as_str()).finish(),
			}
		}
	}

	impl_mut_bytes! {
		impl<'a, T> LossyMutBytes<'a, T>, StrictMutBytes<'a, T>, ReversibleMutBytes<'a, T> as MutBytes;

		impl<'a, T: PodStrBuffer> MutBytes<'a, T> {
			pub fn to_string_lossy(&self) -> Cow<'_, str> {
				String::from_utf8_lossy(unsafe { self.get() }.as_bytes())
			}
		}
	}

	#[test]
	fn fills() {
		use alloc::string::ToString;

		fn assert<const LEN: usize>(char: char) {
			let pod = PodStr::<[u8; LEN]>::from_fill(char);

			if LEN == 0 {
				assert_eq!(&*pod.unwrap(), "");

				return;
			}

			let char_len = char.len_utf8();

			match (pod, LEN.is_multiple_of(char_len)) {
				(None, false) => { /* pass */ }
				(Some(pod), true) => assert_eq!(&pod, &char.to_string().repeat(LEN / char_len)),
				_ => panic!(),
			}
		}

		for char in ['\x00', 'A', 'ß', 'ℝ', '💣'] {
			// 0 up to the max char len + 1
			assert::<0>(char);
			assert::<1>(char);
			assert::<2>(char);
			assert::<3>(char);
			assert::<4>(char);
			assert::<5>(char);
		}
	}
}

#[test]
fn push() {
	const ALPHA: &[u8; 6] = b"Hello ";
	const BRAVO: &[u8; 6] = b"world!";

	const ALPHA_STR: &str = {
		let Ok(str) = str::from_utf8(ALPHA) else { panic!() };

		str
	};

	const BRAVO_STR: &str = {
		let Ok(str) = str::from_utf8(BRAVO) else { panic!() };

		str
	};

	let alpha = PodStr::lit(ALPHA);
	let bravo = PodStr::lit(BRAVO);
	let charlie = alpha.push(bravo);

	assert_eq!(&charlie, "Hello world!");
	// assert_eq!(charlie.pop(), "Hello world!");

	let (alpha, bravo) = charlie.pop();

	assert_eq!(&alpha, ALPHA_STR);
	assert_eq!(&bravo, BRAVO_STR);
}
