use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::error::Error;
use core::fmt::{Debug, Display, Formatter};
use core::hash::{Hash, Hasher};
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};
use core::{mem, slice};

pub mod byte_array;
pub mod byte_slice;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Aligned<T, A>(AlignedManuallyDrop<T, A>);

impl<T, A> Aligned<T, A> {
	pub const TRANSPARENT: bool = AlignedManuallyDrop::<T, A>::TRANSPARENT;

	pub const fn new(value: T) -> Self {
		Self(AlignedManuallyDrop::new(value))
	}

	pub const fn as_ref(&self) -> &T {
		self.0.as_ref()
	}

	pub const fn as_mut(&mut self) -> &mut T {
		self.0.as_mut()
	}

	pub const fn as_ptr(&self) -> *const T {
		self.0.as_ptr()
	}

	pub const fn as_mut_ptr(&mut self) -> *mut T {
		self.0.as_mut_ptr()
	}

	pub fn copy(&self) -> Self
	where
		T: Copy,
	{
		Self(self.0.copy())
	}

	/// Dropping [`AlignedManuallyDrop`] will not run destructors.
	/// This probably means
	pub const fn into_manually_drop(self) -> AlignedManuallyDrop<T, A> {
		let md = unsafe { (&raw const self.0).cast::<AlignedManuallyDrop<T, A>>().read() };

		mem::forget(self);
		md
	}

	pub const fn into_inner(self) -> T {
		let value = unsafe { (&raw const self.0.value).read() };

		mem::forget(self);
		ManuallyDrop::into_inner(value)
	}
}

impl<T, A> Drop for Aligned<T, A> {
	fn drop(&mut self) {
		unsafe { ManuallyDrop::drop(&mut self.0.value) }
	}
}

/// Wrapper of type `T` with layout requirements (reflexively) increased to accommodate of `A`.
pub union AlignedManuallyDrop<T, A> {
	_alignment: ManuallyDrop<A>,
	value: ManuallyDrop<T>,
}

impl<T, A> AlignedManuallyDrop<T, A> {
	pub const TRANSPARENT: bool = size_of::<Self>() == size_of::<T>();

	pub const fn new(value: T) -> Self {
		Self {
			value: ManuallyDrop::new(value),
		}
	}

	pub const fn as_ref(&self) -> &T {
		unsafe { &*self.as_ptr() }
	}

	pub const fn as_mut(&mut self) -> &mut T {
		unsafe { &mut *self.as_mut_ptr() }
	}

	pub const fn as_ptr(&self) -> *const T {
		unsafe { &self.value as *const ManuallyDrop<T> }.cast::<T>()
	}

	pub const fn as_mut_ptr(&mut self) -> *mut T {
		unsafe { &mut self.value as *mut ManuallyDrop<T> }.cast::<T>()
	}

	pub const fn copy(&self) -> Self
	where
		T: Copy,
	{
		Self::new(ManuallyDrop::into_inner(*unsafe { &self.value }))
	}

	pub const fn into_inner(self) -> T {
		ManuallyDrop::into_inner(unsafe { self.value })
	}
}

unsafe impl<T: Send, A> Send for AlignedManuallyDrop<T, A> {}
unsafe impl<T: Sync, A> Sync for AlignedManuallyDrop<T, A> {}

impl<T: Clone, A> Clone for AlignedManuallyDrop<T, A> {
	fn clone(&self) -> Self {
		Self::new(self.as_ref().clone())
	}
}

// It's frustrating that `A: Copy` is required here
// but any workarounds I came up with are... excruciating to use
impl<T: Copy, A: Copy> Copy for AlignedManuallyDrop<T, A> {}

impl<T: PartialEq, A0, A1> PartialEq<AlignedManuallyDrop<T, A1>> for AlignedManuallyDrop<T, A0> {
	fn eq(&self, other: &AlignedManuallyDrop<T, A1>) -> bool {
		self.as_ref().eq(other.as_ref())
	}
}

impl<T: Eq, A> Eq for AlignedManuallyDrop<T, A> {}

impl<T: PartialOrd, A0, A1> PartialOrd<AlignedManuallyDrop<T, A1>> for AlignedManuallyDrop<T, A0> {
	fn partial_cmp(&self, other: &AlignedManuallyDrop<T, A1>) -> Option<Ordering> {
		self.as_ref().partial_cmp(other.as_ref())
	}
}

impl<T: Ord, A> Ord for AlignedManuallyDrop<T, A> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.as_ref().cmp(other.as_ref())
	}
}

impl<T: Hash, A> Hash for AlignedManuallyDrop<T, A> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		<T as Hash>::hash::<H>(self.as_ref(), state)
	}

	fn hash_slice<H: Hasher>(data: &[Self], state: &mut H)
	where
		Self: Sized,
	{
		if const { Self::TRANSPARENT } {
			<T as Hash>::hash_slice::<H>(unsafe { slice::from_raw_parts(data.as_ptr().cast::<T>(), data.len()) }, state)
		} else {
			for piece in data {
				<T as Hash>::hash(piece, state)
			}
		}
	}
}

macro_rules! shared_impls {
	($($name:literal <$t:ident, $a:ident> $target:ty),* $(,)?) => {
		$(impl<$t, $a> Deref for $target {
			type Target = $t;

			fn deref(&self) -> &Self::Target {
				self.as_ref()
			}
		}

		impl<$t, $a> DerefMut for $target {
			fn deref_mut(&mut self) -> &mut Self::Target {
				self.as_mut()
			}
		}

		impl<$t, $a, U> AsRef<U> for $target
		where
			$t: AsRef<U>,
		{
			fn as_ref(&self) -> &U {
				<$t as AsRef<U>>::as_ref(Self::as_ref(self))
			}
		}

		impl<$t, $a, U> AsMut<U> for $target
		where
			$t: AsMut<U>,
		{
			fn as_mut(&mut self) -> &mut U {
				<$t as AsMut<U>>::as_mut(Self::as_mut(self))
			}
		}

		impl<$t, $a> Borrow<$t> for $target {
			fn borrow(&self) -> &$t {
				self.as_ref()
			}
		}

		impl<$t, $a> BorrowMut<$t> for $target {
			fn borrow_mut(&mut self) -> &mut $t {
				self.as_mut()
			}
		}

		impl<$t: Debug, $a> Debug for $target {
			fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
				if const { align_of::<$a>() > align_of::<$t>() } {
					f.write_fmt(format_args!("#[repr(align({}))]", align_of::<$a>()))?;
					f.write_str(if f.alternate() { "\n" } else { " " })?;
				}

				f.debug_tuple($name).field(self.as_ref()).finish()
			}
		}

		impl<$t: Display, $a> Display for $target {
			fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
				<$t as Display>::fmt(self.as_ref(), f)
			}
		}

		impl<$t: Error, $a> Error for $target {
			fn source(&self) -> Option<&(dyn Error + 'static)> {
				<$t as Error>::source(self.as_ref())
			}
		})*
	};
}

shared_impls! {
	"AlignedManuallyDrop" <T, A> AlignedManuallyDrop<T, A>,
	"Aligned" <T, A> Aligned<T, A>,
}
