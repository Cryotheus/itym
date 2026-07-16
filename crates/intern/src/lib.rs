//! See [`ByteInterner`] for internalizing most string types.
//! See [`Interner`] for all other types.
//!
//! ```rust
//! # use itym_intern::ByteInterner;
//! # use std::sync::LazyLock;
//!
//! static BYTES: LazyLock<ByteInterner> = ByteInterner::new();
//!
//! fn main() {
//!     // Returns `Intern<[u8]>` usable as `&'static [u8]`
//!     BYTES.intern_static("This is a literal");
//!
//!     // Offers a `str` equivalent
//!     BYTES.intern_string(String::from("Smart pointers lose their destructors"));
//!
//!     let greet = {
//!         let greet_1 = BYTES.intern_box(String::from("Hi john").into_boxed_str());
//!         let greet_2 = BYTES.intern_static("Hi john"); //duplicate
//!
//!         // No duplicates
//!         assert!(std::ptr::eq(&*greet_1, &*greet_2));
//!
//!         // Lifetime extension
//!         greet_1.get_static_ref()
//!     };
//!
//!     assert_eq!(greet, b"Hi john");
//! }
//! ```
//!
//!

#[cfg(feature = "borsh")]
mod impl_borsh;

#[cfg(feature = "serde")]
mod impl_serde;

use std::any::{Any, TypeId};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Index};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex, MutexGuard, TryLockError};

type AnySend = dyn Any + Send + 'static;
type Erased = &'static Mutex<AnySend>;
type ErasedGuard = MutexGuard<'static, AnySend>;

static REGISTRY: LazyLock<Mutex<HashMap<TypeId, Erased>>> = LazyLock::new(Mutex::default);

/// An [`Interner<[u8]>`] all interns which can be represented by `[u8]`.
#[derive(Debug, Clone, Copy)]
pub struct ByteInterner(pub Interner<[u8]>);

impl ByteInterner {
	pub const fn new() -> LazyLock<Self> {
		LazyLock::new(Self::new_raw)
	}

	pub fn new_raw() -> Self {
		Self(Interner::new_raw())
	}

	#[inline(always)]
	pub fn lock(&'static self) -> ByteInternerGuard {
		ByteInternerGuard(self.0.lock())
	}

	#[inline(always)]
	pub fn intern_box<T: ?Sized>(&'static self, bytes: Box<T>) -> Intern<[u8]>
	where
		Box<T>: Into<Box<[u8]>>,
	{
		self.lock().intern_box(bytes)
	}

	#[inline(always)]
	pub fn intern_static<T: ?Sized + AsRef<[u8]>>(&'static self, bytes: &'static T) -> Intern<[u8]> {
		self.lock().intern_static(bytes)
	}

	#[inline(always)]
	pub fn intern_os_string(&'static self, os_string: OsString) -> Intern<OsStr> {
		self.lock().intern_os_string(os_string)
	}

	#[inline(always)]
	pub fn intern_path_buf(&'static self, path_buf: PathBuf) -> Intern<Path> {
		self.lock().intern_path_buf(path_buf)
	}

	#[inline(always)]
	pub fn intern_string(&'static self, string: String) -> Intern<str> {
		self.lock().intern_string(string)
	}
}

impl Deref for ByteInterner {
	type Target = Interner<[u8]>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for ByteInterner {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

/// `Interner<[u8]>` all interns which can be represented by `[u8]`.
#[derive(Debug)]
pub struct ByteInternerGuard(pub InternerGuard<[u8]>);

impl ByteInternerGuard {
	pub fn intern_box<T: ?Sized>(&mut self, bytes: Box<T>) -> Intern<[u8]>
	where
		Box<T>: Into<Box<[u8]>>,
	{
		self.0.intern_box(bytes.into())
	}

	pub fn intern_static<T: ?Sized + AsRef<[u8]>>(&mut self, bytes: &'static T) -> Intern<[u8]> {
		self.0.intern_static(bytes.as_ref())
	}

	pub fn intern_os_string(&mut self, os_string: OsString) -> Intern<OsStr> {
		let intern = self.0.intern_vec(os_string.into_encoded_bytes());

		unsafe { intern.map(OsStr::from_encoded_bytes_unchecked) }
	}

	pub fn intern_path_buf(&mut self, path_buf: PathBuf) -> Intern<Path> {
		let intern = self.0.intern_vec(path_buf.into_os_string().into_encoded_bytes());

		unsafe { intern.map(|os_str| Path::new(OsStr::from_encoded_bytes_unchecked(os_str))) }
	}

	pub fn intern_string(&mut self, string: String) -> Intern<str> {
		let intern = self.0.intern_vec(string.into_bytes());

		unsafe { intern.map_res(std::str::from_utf8) }
	}
}

impl Deref for ByteInternerGuard {
	type Target = InternerGuard<[u8]>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for ByteInternerGuard {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

#[derive(Debug)]
struct Cache<T: ?Sized + 'static>(HashSet<&'static T>);

impl<T: ?Sized> Cache<T> {
	fn new() -> Self {
		Self(HashSet::new())
	}
}

/// Reference to an internalized value of `T`.
/// Values are managed by an [`Interner`].
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Intern<T: ?Sized + 'static>(&'static T);

impl<T: ?Sized> Intern<T> {
	pub fn get_static_ref(self) -> &'static T {
		self.0
	}

	#[inline]
	pub fn map_intern<U: ?Sized, F>(self, f: F) -> Intern<U>
	where
		F: FnOnce(&'static T) -> &'static U,
	{
		Intern(f(self.0))
	}

	/// Same as `AsRef::as_ref` but keeps the reference wrapped in the [`Intern`] newtype.
	#[inline]
	pub fn map_intern_auto<U: ?Sized>(self) -> Intern<U>
	where
		T: AsRef<U>,
	{
		Intern(<T as AsRef<U>>::as_ref(self.0))
	}

	/// Map unchecked.
	/// Internal convenience.
	#[inline(always)]
	unsafe fn map<U: ?Sized>(self, f: unsafe fn(&'static T) -> &'static U) -> Intern<U> {
		Intern(unsafe { f(self.0) })
	}

	/// Map result unchecked.
	/// Internal convenience.
	#[inline(always)]
	unsafe fn map_res<U: ?Sized, E>(self, f: unsafe fn(&'static T) -> Result<&'static U, E>) -> Intern<U> {
		Intern(unsafe { f(self.0).unwrap_unchecked() })
	}
}

impl<T: Eq + Hash + Sync> Intern<T> {
	/// You should use a `static` [`Interner`] instead,
	/// as this is significantly slower than [`InternerGuard::intern`].
	pub fn new(value: T) -> Self {
		Interner::<T>::new_raw()._lock().intern(value)
	}
}

impl<T: ?Sized + Eq + Hash + Sync> Intern<T> {
	/// You should use a `static` [`Interner`] instead,
	/// as this is significantly slower than [`InternerGuard::intern_box`].
	pub fn from_box(value: Box<T>) -> Self {
		Interner::<T>::new_raw()._lock().intern_box(value)
	}
}

impl<T: ?Sized, U: ?Sized> AsRef<U> for Intern<T>
where
	T: AsRef<U>,
{
	fn as_ref(&self) -> &U {
		<T as AsRef<U>>::as_ref(self.0)
	}
}

impl<T: ?Sized> Borrow<T> for Intern<T> {
	fn borrow(&self) -> &T {
		self.0
	}
}

impl<T: ?Sized> Clone for Intern<T> {
	fn clone(&self) -> Self {
		Self(self.0)
	}
}

impl<T: ?Sized> Copy for Intern<T> {}

impl<T: ?Sized> Deref for Intern<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.0
	}
}

macro_rules! impl_fmt {
	(
		$($path:path),+
		$(,)?
	) => {
		$(impl<T: ?Sized + $path> $path for Intern<T> {
			fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
				<T as $path>::fmt(self.0, f)
			}
		})+
	};
}

impl_fmt! {
	core::fmt::Binary,
	core::fmt::Display,
	core::fmt::LowerExp,
	core::fmt::LowerHex,
	core::fmt::Octal,
	core::fmt::Pointer,
	core::fmt::UpperExp,
	core::fmt::UpperHex,
}

impl<T: ?Sized + Error> Error for Intern<T> {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		self.0.source()
	}
}

impl<T: ?Sized + PartialEq> PartialEq<T> for Intern<T> {
	fn eq(&self, other: &T) -> bool {
		self.0.eq(other)
	}
}

impl<T: ?Sized + PartialOrd> PartialOrd<T> for Intern<T> {
	fn partial_cmp(&self, other: &T) -> Option<Ordering> {
		self.0.partial_cmp(other)
	}
}

impl<T: ?Sized + Index<I>, I> Index<I> for Intern<T> {
	type Output = T::Output;

	fn index(&self, index: I) -> &Self::Output {
		self.0.index(index)
	}
}

/// Trivially copy-able reference to a cache of [internalized] `T` values.
///
/// [internalized]: [`Intern`]
pub struct Interner<T: ?Sized + 'static> {
	_marker: PhantomData<fn() -> T>,
	container: Erased,
}

impl<T: ?Sized> Interner<T> {
	#[inline(always)]
	pub fn lock(&'static self) -> InternerGuard<T> {
		self._lock()
	}

	fn _lock(&self) -> InternerGuard<T> {
		InternerGuard {
			_marker: PhantomData,
			guard: self.container.lock().unwrap(),
		}
	}

	fn try_lock(&self) -> Option<InternerGuard<T>> {
		match self.container.try_lock() {
			Ok(guard) => Some(InternerGuard { _marker: PhantomData, guard }),
			Err(TryLockError::WouldBlock) => None,
			Err(TryLockError::Poisoned(..)) => panic!(),
		}
	}
}

impl<T: ?Sized + Sync> Interner<T> {
	pub const fn new() -> LazyLock<Self> {
		LazyLock::new(Self::new_raw)
	}

	pub fn new_raw() -> Self {
		let mut registry_guard = REGISTRY.lock().unwrap();

		let erased: Erased = registry_guard.entry(TypeId::of::<T>()).or_insert_with(
			const {
				|| {
					let mutex: Mutex<Cache<T>> = Mutex::new(Cache::<T>::new());
					let boxed: Box<Mutex<AnySend>> = Box::<Mutex<Cache<T>>>::new(mutex);

					Box::leak(boxed)
				}
			},
		);

		drop(registry_guard);

		Self {
			_marker: PhantomData,
			container: erased,
		}
	}
}

impl<T: Eq + Hash> Interner<T> {
	#[inline(always)]
	pub fn intern(&'static self, value: T) -> Intern<T> {
		self.lock().intern(value)
	}
}

impl<T: ?Sized + Eq + Hash> Interner<T> {
	/// You should probably use [`intern`] or [`is_interned`] instead.
	#[inline(always)]
	#[must_use]
	pub fn get(&'static self, value: &T) -> Option<Intern<T>> {
		self.lock().get(value)
	}

	#[inline(always)]
	pub fn intern_box(&'static self, value: Box<T>) -> Intern<T> {
		self.lock().intern_box(value)
	}

	#[inline(always)]
	pub fn intern_static(&'static self, value: &'static T) -> Intern<T> {
		self.lock().intern_static(value)
	}

	#[inline(always)]
	#[must_use]
	pub fn is_interned(&'static self, value: &T) -> bool {
		self.lock().is_interned(value)
	}
}

impl<T: Eq + Hash> Interner<[T]> {
	#[inline(always)]
	pub fn intern_array<const LEN: usize>(&'static self, array: [T; LEN]) -> Intern<[T]> {
		self.lock().intern_array(array)
	}

	#[inline(always)]
	pub fn intern_vec(&'static self, vec: Vec<T>) -> Intern<[T]> {
		self.lock().intern_vec(vec)
	}

	#[inline(always)]
	pub fn intern_vec_deque(&'static self, queue: VecDeque<T>) -> Intern<[T]> {
		self.lock().intern_vec_deque(queue)
	}
}

impl<T: ?Sized> Clone for Interner<T> {
	fn clone(&self) -> Self {
		Self {
			_marker: PhantomData,
			container: self.container,
		}
	}
}

impl<T: ?Sized> Copy for Interner<T> {}

impl<T: ?Sized + Debug> Debug for Interner<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		use std::fmt::from_fn;

		let alt = f.alternate();
		let guard = self.try_lock();

		f.debug_struct("Interner")
			.field(
				"container",
				match (guard.as_ref(), alt) {
					(None, true) => const { &from_fn(|f| f.debug_struct("#[locked]\nInternerGuard").finish_non_exhaustive()) },
					(None, false) => const { &from_fn(|f| f.debug_struct("#[locked] InternerGuard").finish_non_exhaustive()) },
					(Some(guard), ..) => guard,
				},
			)
			.finish()
	}
}

impl<T: ?Sized, U: ?Sized> PartialEq<Interner<U>> for Interner<T> {
	fn eq(&self, _: &Interner<U>) -> bool {
		TypeId::of::<T>().eq(&TypeId::of::<U>())
	}
}

impl<T: ?Sized> Eq for Interner<T> {}

impl<T: ?Sized> Hash for Interner<T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		<TypeId as Hash>::hash::<H>(&TypeId::of::<T>(), state)
	}
}

impl<T: ?Sized, U: ?Sized> PartialOrd<Interner<U>> for Interner<T> {
	fn partial_cmp(&self, _: &Interner<U>) -> Option<Ordering> {
		TypeId::of::<T>().partial_cmp(&TypeId::of::<U>())
	}
}

impl<T: ?Sized> Ord for Interner<T> {
	fn cmp(&self, _: &Self) -> Ordering {
		Ordering::Equal
	}
}

pub struct InternerGuard<T: ?Sized + 'static> {
	_marker: PhantomData<fn() -> T>,
	guard: ErasedGuard,
}

impl<T: Eq + Hash> InternerGuard<T> {
	pub fn intern(&mut self, value: T) -> Intern<T> {
		self.intern_box(Box::new(value))
	}
}

impl<T: ?Sized> InternerGuard<T> {
	fn cache(&self) -> &Cache<T> {
		self.guard.downcast_ref::<Cache<T>>().unwrap()
	}

	fn cache_mut(&mut self) -> &mut Cache<T> {
		self.guard.downcast_mut::<Cache<T>>().unwrap()
	}
}

impl<T: ?Sized + Eq + Hash> InternerGuard<T> {
	pub fn is_interned(&self, value: &T) -> bool {
		self.cache().0.contains(value)
	}

	/// You should probably use [`intern`] or [`is_interned`] instead.
	pub fn get(&self, value: &T) -> Option<Intern<T>> {
		self.cache().0.get(value).copied().map(Intern)
	}

	#[inline(always)]
	pub fn intern_box(&mut self, value: Box<T>) -> Intern<T> {
		self.intern_with(value, Deref::deref, |boxed, _| Box::leak(boxed))
	}

	#[inline(always)]
	pub fn intern_static(&mut self, value: &'static T) -> Intern<T> {
		self.intern_with::<&'static T, &'static T>(value, |pass| pass, |pass, _| pass)
	}

	fn intern_with<U, Q>(&mut self, value: U, borrow: for<'a> fn(&'a U) -> &'a Q, internalizer: fn(U, &mut Cache<T>) -> &'static T) -> Intern<T>
	where
		&'static T: Borrow<Q>,
		Q: Hash + Eq + ?Sized,
	{
		let cache = self.cache_mut();
		let borrow = borrow(&value);

		//we should return the static ref we already have if we have one
		Intern(match cache.0.get::<Q>(borrow) {
			None => {
				let intern = internalizer(value, cache);

				cache.0.insert(intern);

				intern
			}

			Some(&intern) => intern,
		})
	}
}

/// Slices.
impl<T: Eq + Hash> InternerGuard<[T]> {
	/// Attempts to create the [`Intern`] from an existing slice instead of using [`Vec::leak`].
	#[inline(always)]
	pub fn agressive_intern_vec(&mut self, vec: Vec<T>) -> Intern<[T]> {
		self.intern_with(vec, <Vec<T>>::as_slice, |vec, cache| {
			let len = vec.len();

			for &intern in &cache.0 {
				for start in 0..=intern.len() - len {
					let offset = &intern[start..];

					if offset.starts_with(&vec) {
						return &offset[..len];
					}
				}
			}

			vec.leak()
		})
	}

	/// Searches all internalized values in the cache for one containing `array`.
	/// If found, the contained slice is used as the intern avoiding a new memory leak.
	pub fn agressive_intern_array<const LEN: usize>(&mut self, array: [T; LEN]) -> Intern<[T]> {
		self.intern_with(array, <[T; LEN]>::as_slice, |array, cache| {
			for &intern in &cache.0 {
				for start in 0..=intern.len() - LEN {
					let offset = &intern[start..];

					if offset.starts_with(&array) {
						return &offset[..LEN];
					}
				}
			}

			Box::leak(Box::new(array))
		})
	}

	#[inline(always)]
	pub fn intern_array<const LEN: usize>(&mut self, array: [T; LEN]) -> Intern<[T]> {
		self.intern_with(array, <[T; LEN]>::as_slice, |array, _| Box::leak(Box::new(array)))
	}

	#[inline(always)]
	pub fn intern_vec(&mut self, vec: Vec<T>) -> Intern<[T]> {
		self.intern_with(vec, <Vec<T>>::as_slice, |vec, _| vec.leak())
	}

	pub fn intern_vec_deque(&mut self, mut queue: VecDeque<T>) -> Intern<[T]> {
		queue.make_contiguous();

		self.intern_with(
			queue,
			|queue| queue.as_slices().0,
			//`VecDeque` doesn't have a leak function
			//but it does have a specialized conversion to `Vec`
			|queue, _| Vec::from(queue).leak(),
		)
	}
}

impl InternerGuard<str> {
	#[inline(always)]
	pub fn agressive_intern_string(&mut self, string: String) -> Intern<str> {
		self.intern_with(
			string,
			|string| string.deref(),
			|string, cache| {
				for &intern in &cache.0 {
					if let Some(found) = intern.find(&string) {
						let sub: &'static str = &intern[found..][..string.len()];

						return sub;
					}
				}

				//
				string.leak()
			},
		)
	}
}

macro_rules! flat_impl {
	(macro $func:ident $param:ident $param_ty:ty; $target:ty; self $self:ident $body:expr) => {
		impl InternerGuard<$target> {
			#[inline(always)]
			pub fn $func(&mut $self, $param: $param_ty) -> Intern<$target> {
				$body
			}
		}
	};

	(macro $func:ident $param:ident $param_ty:ty; $target:ty;) => {
		impl InternerGuard<$target> {
			#[inline(always)]
			pub fn $func(&mut self, $param: $param_ty) -> Intern<$target> {
				self.intern_with($param, |$param| $param.deref(), |$param, _| $param.leak())
			}
		}
	};

	(
		$($func:ident($param:ident: $param_ty:ty) -> $target:ty $( { $self:ident => $($leak:expr);+ })?),+
		$(,)?
	) => {
		$(
			flat_impl! { macro $func $param $param_ty; $target; $( $self $self { $($leak);+ } )? }

			impl Interner<$target> {
				#[inline(always)]
				pub fn $func(&'static self, $param: $param_ty) -> Intern<$target> {
					self.lock().$func($param)
				}
			}
		)+
	};
}

flat_impl! {
	intern_string(string: String) -> str,
	intern_c_string(c_string: std::ffi::CString) -> std::ffi::CStr { self => self.intern_box(c_string.into_boxed_c_str()) },
	intern_os_string(os_string: std::ffi::OsString) -> std::ffi::OsStr,
	intern_path_buf(path_buf: std::path::PathBuf) -> std::path::Path,
}

impl<T: ?Sized + Debug> Debug for InternerGuard<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("InternerGuard").field("cache", self.cache()).finish()
	}
}

static STRINGS: LazyLock<ByteInterner> = ByteInterner::new();
