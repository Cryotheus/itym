use crate::bounds::{Erased, ErasedBounds};
use alloc::vec::Vec;
use core::any::{Any, TypeId, type_name};
use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;
use core::mem::{MaybeUninit, transmute};
use core::ops::{Deref, DerefMut};
use itym_assert::*;
use itym_mem::utransmute;

/// Type erasure over two vecs of distance types `Vec<T>` and `Vec<U>` is typically
/// implemented by wrapping both in a [`Box`] and performing coercion to `Box<dyn Any>`.
///
/// This solutio
pub struct ErasedVec<S: ?Sized = dyn Any + Send + Sync> {
	vec: Inner,
	virtuals: &'static Virtuals,
	_bounds: PhantomData<fn() -> S>,
	_not_send_sync: PhantomData<*mut ()>,
}

unsafe impl Send for ErasedVec<dyn Any + Send> {}

unsafe impl Sync for ErasedVec<dyn Any + Sync> {}

unsafe impl Send for ErasedVec<dyn Any + Send + Sync> {}
unsafe impl Sync for ErasedVec<dyn Any + Send + Sync> {}

impl<S: ?Sized + Erased> ErasedVec<S> {
	pub fn new<T: ErasedBounds<S>>() -> Self {
		Self::from_vec::<T>(Vec::<T>::new())
	}

	pub fn from_vec<T: ErasedBounds<S>>(vec: Vec<T>) -> Self {
		Self {
			vec: Inner::new(vec),
			virtuals: const { &Virtuals::new::<T>() },
			_bounds: PhantomData,
			_not_send_sync: PhantomData,
		}
	}

	pub fn downcast<T: ErasedBounds<S>>(self) -> Result<Vec<T>, Self> {
		if const { TypeId::of::<T>() } == self.virtuals.type_id {
			Ok(unsafe { self.vec.downcast() })
		} else {
			Err(self)
		}
	}

	pub fn downcast_ref<T: ErasedBounds<S>>(&self) -> Option<&Vec<T>> {
		ensure!(const { TypeId::of::<T>() } == self.virtuals.type_id);
		Some(unsafe { self.vec.downcast_ref::<T>() })
	}

	pub fn downcast_mut<T: ErasedBounds<S>>(&mut self) -> Option<&mut Vec<T>> {
		ensure!(const { TypeId::of::<T>() } == self.virtuals.type_id);
		Some(unsafe { self.vec.downcast_mut::<T>() })
	}

	pub fn with_clone<T: ErasedBounds<S> + Clone>(self) -> Result<ErasedVecClone<S>, Self> {
		if const { TypeId::of::<T>() } != self.virtuals.type_id {
			return Err(self);
		}

		Ok(ErasedVecClone::<S>(self))
	}

	pub fn as_ptr(&self) -> *const u8 {
		unsafe { (self.virtuals.as_ptr)(&self.vec) }
	}

	pub fn as_mut_ptr(&mut self) -> *mut u8 {
		unsafe { (self.virtuals.as_mut_ptr)(&mut self.vec) }
	}

	pub fn capacity(&self) -> usize {
		unsafe { (self.virtuals.capacity)(&self.vec) }
	}

	pub fn clear(&mut self) {
		unsafe { (self.virtuals.clear)(&mut self.vec) }
	}

	pub fn len(&self) -> usize {
		unsafe { (self.virtuals.len)(&self.vec) }
	}

	pub fn get(&self, index: usize) -> Option<&S> {
		if index < self.len() {
			Some(unsafe { self.get_unchecked(index) })
		} else {
			None
		}
	}

	pub fn get_mut(&mut self, index: usize) -> Option<&mut S> {
		if index < self.len() {
			Some(unsafe { self.get_unchecked_mut(index) })
		} else {
			None
		}
	}

	pub unsafe fn get_unchecked(&self, index: usize) -> &S {
		unsafe { utransmute!(<&dyn Any, &S> (self.virtuals.get_unchecked)(&self.vec, index)) }
	}

	pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut S {
		unsafe { utransmute!(<&mut dyn Any, &mut S> (self.virtuals.get_unchecked_mut)(&mut self.vec, index)) }
	}

	pub fn is<T: Any>(&self) -> bool {
		self.virtuals.type_id == const { TypeId::of::<T>() }
	}

	pub fn type_id(&self) -> &'static TypeId {
		&self.virtuals.type_id
	}
}

impl<S: ?Sized> Drop for ErasedVec<S> {
	fn drop(&mut self) {
		unsafe { (self.virtuals.drop)(self.vec) };
	}
}

impl<S: ?Sized + Erased> Debug for ErasedVec<S> {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.debug_struct((self.virtuals.type_name)())
			.field("ptr", &self.as_ptr())
			.field("cap", &self.capacity())
			.field("len", &self.len())
			.finish()
	}
}

/// Functions similar to [`ErasedVec`], but also implements [`Clone`].
#[repr(transparent)]
pub struct ErasedVecClone<S: ?Sized = dyn Any + Send + Sync>(
	/// # Safety
	/// Must uphold: `T: ErasedBounds<S> + Clone`
	ErasedVec<S>,
);

impl<S: ?Sized + Erased> ErasedVecClone<S> {
	pub fn new<T: ErasedBounds<S> + Clone>() -> Self {
		Self::from_vec::<T>(Vec::new())
	}

	pub fn from_vec<T: ErasedBounds<S> + Clone>(vec: Vec<T>) -> Self {
		Self(ErasedVec {
			vec: Inner::new(vec),
			virtuals: const { &Virtuals::new_clone::<T>() },
			_bounds: PhantomData,
			_not_send_sync: PhantomData,
		})
	}

	pub fn without_clone(self) -> ErasedVec<S> {
		self.0
	}
}

impl<S: ?Sized> Clone for ErasedVecClone<S> {
	fn clone(&self) -> Self {
		ErasedVecClone(ErasedVec {
			vec: unsafe { (self.0.virtuals.clone)(&self.0.vec) },
			virtuals: self.0.virtuals,
			_bounds: PhantomData,
			_not_send_sync: PhantomData,
		})
	}
}

impl<S: ?Sized> Deref for ErasedVecClone<S> {
	type Target = ErasedVec<S>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<S: ?Sized> DerefMut for ErasedVecClone<S> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<S: ?Sized + Erased> Debug for ErasedVecClone<S> {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.debug_tuple("ErasedVecClone").field(&self.0).finish()
	}
}

#[derive(Clone, Copy)]
#[repr(transparent)]
struct Inner(MaybeUninit<[usize; 3]>);

impl Inner {
	#[inline(always)]
	const fn new<T>(vec: Vec<T>) -> Self {
		unsafe { transmute::<Vec<T>, Self>(vec) }
	}

	#[inline(always)]
	const unsafe fn downcast<T>(self) -> Vec<T> {
		unsafe { transmute::<Self, Vec<T>>(self) }
	}

	#[inline(always)]
	const unsafe fn downcast_ref<T>(&self) -> &Vec<T> {
		unsafe { &*(self as *const Self).cast::<Vec<T>>() }
	}

	#[inline(always)]
	const unsafe fn downcast_mut<T>(&mut self) -> &mut Vec<T> {
		unsafe { &mut *(self as *mut Self).cast::<Vec<T>>() }
	}

	#[inline(always)]
	unsafe fn as_ptr<T>(&self) -> *const u8 {
		unsafe { self.downcast_ref::<T>() }.as_ptr().cast::<u8>()
	}

	#[inline(always)]
	unsafe fn as_mut_ptr<T>(&mut self) -> *mut u8 {
		unsafe { self.downcast_mut::<T>() }.as_mut_ptr().cast::<u8>()
	}

	#[inline(always)]
	unsafe fn capacity<T>(&self) -> usize {
		unsafe { self.downcast_ref::<T>() }.capacity()
	}

	#[inline(always)]
	unsafe fn clear<T>(&mut self) {
		unsafe { self.downcast_mut::<T>() }.clear()
	}

	#[inline(always)]
	unsafe fn len<T>(&self) -> usize {
		unsafe { self.downcast_ref::<T>() }.len()
	}

	unsafe fn get_unchecked<T: Any>(&self, index: usize) -> &dyn Any {
		unsafe { self.downcast_ref::<T>().get_unchecked(index) }
	}

	unsafe fn get_unchecked_mut<T: Any>(&mut self, index: usize) -> &mut dyn Any {
		unsafe { self.downcast_mut::<T>().get_unchecked_mut(index) }
	}
}

impl Debug for Inner {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("Inner").finish_non_exhaustive()
	}
}

#[derive(Debug, Clone, Copy)]
pub enum ErasedVecPopError {
	Empty,
	TypeMismatch,
}

#[derive(Debug, Clone, Copy)]
pub enum ErasedVecPushError {
	TypeMismatch,
}

#[derive(Debug)]
struct Virtuals {
	as_ptr: unsafe fn(&Inner) -> *const u8,
	as_mut_ptr: unsafe fn(&mut Inner) -> *mut u8,
	capacity: unsafe fn(&Inner) -> usize,
	clear: unsafe fn(&mut Inner),
	clone: unsafe fn(&Inner) -> Inner,
	get_unchecked: for<'a> unsafe fn(&'a Inner, usize) -> &'a dyn Any,
	get_unchecked_mut: for<'a> unsafe fn(&'a mut Inner, usize) -> &'a mut dyn Any,
	len: unsafe fn(&Inner) -> usize,
	drop: unsafe fn(Inner),
	type_id: TypeId,
	type_name: fn() -> &'static str,
}

impl Virtuals {
	const fn new<T: Any>() -> Self {
		Self {
			as_ptr: Inner::as_ptr::<T>,
			as_mut_ptr: Inner::as_mut_ptr::<T>,
			capacity: Inner::capacity::<T>,
			clear: Inner::clear::<T>,
			clone: |_| unimplemented!("Virtual function table for Vec<{}> unimplements clone", type_name::<T>()),
			get_unchecked: Inner::get_unchecked::<T>,
			get_unchecked_mut: Inner::get_unchecked_mut::<T>,
			len: Inner::len::<T>,

			drop: |vec| {
				let vec = unsafe { vec.downcast::<T>() };

				drop(vec);
			},

			type_id: TypeId::of::<T>(),
			type_name: type_name::<Vec<T>>,
		}
	}

	const fn new_clone<T: Any + Clone>() -> Self {
		Self {
			clone: |vec| Inner::new(unsafe { vec.downcast_ref::<T>() }.clone()),
			..Self::new::<T>()
		}
	}
}
