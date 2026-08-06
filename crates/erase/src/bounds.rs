use core::any::Any;

/// Use by the crate to enforce bounds which are required for soundness.
#[diagnostic::on_unimplemented(
	message = "`{Self}` must implement bounds of `{S}` in `ErasedBounds<{S}>`",
	note = "Fix: restrict `{Self}` to the bounds of `{S}`",
	note = "Fix: relax bounds of `{S}` as such: `ErasedBounds<dyn Any>`"
)]
pub unsafe trait ErasedBounds<S: ?Sized + Erased>: Any {}

unsafe impl<T: Any> ErasedBounds<dyn Any> for T {}
unsafe impl<T: Any + Send> ErasedBounds<dyn Any + Send> for T {}
unsafe impl<T: Any + Sync> ErasedBounds<dyn Any + Sync> for T {}
unsafe impl<T: Any + Send + Sync> ErasedBounds<dyn Any + Send + Sync> for T {}

#[allow(private_bounds)]
pub trait Erased: Sealed {}

impl Erased for dyn Any {}
impl Erased for dyn Any + Send {}
impl Erased for dyn Any + Sync {}
impl Erased for dyn Any + Send + Sync {}

trait Sealed {}

impl Sealed for dyn Any {}
impl Sealed for dyn Any + Send {}
impl Sealed for dyn Any + Sync {}
impl Sealed for dyn Any + Send + Sync {}
