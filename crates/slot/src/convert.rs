use crate::Slot;

///
/// For types which contain a primary `Slot<T>`.
/// This is most similar to a `const unsafe` variant of [`AsRef<Slot<T>>`] and [`AsMut<Slot<T>>`].
///
/// Due to language limitations, the trait does not supply methods.
/// Use the module's [`as_slot_ref`] and [`as_slot_mut`] functions instead.
///
/// Expect a major version bump impacting this trait upon stabilization of the `const_trait_impl` feature.
///
/// # Safety
/// For any valid `*mut Self` or `*const Self`, a valid `Slot<T>` must exist at the given `OFFSET`.
#[diagnostic::on_unimplemented(
	message = "`{Self}` does not offer borrows of `Slot<{T}>` at `const`",
	label = "`&Slot<{T}>` not accessible at `const`",
	note = "An `unsafe impl RawSlotRef<{T}> for {Self}` can be made as a remedy",
	note = "`RawSlotRef<{T}>` is a stable release channel implmentation of `[const] AsRef<Slot<T>>`"
)]
pub unsafe trait RawSlotRef<T> {
	/// Set this to [`SlotVisPub`].
	type Visibility: SlotVisibility;

	/// The byte offset at which a `Slot<T>` exists behind a raw pointer of `Self`.
	const OFFSET: usize;
}

/// For types which have the same layout as [`Slot<T>`].
/// Utilized by `unsafe` code for transmutation.
///
/// Expect a major version bump impacting this trait upon stabilization of the `const_trait_impl` feature.
///
/// # Safety
/// Type must have the exact layout as `Slot<T>` and safely support transmutation into `Slot<T>`.
pub unsafe trait RawSlotTransparent<T>: RawSlotRef<T> {}

/// Controls access to usage of [`as_slot_ref`] and [`as_slot_mut`].
#[allow(private_bounds)]
pub trait SlotVisibility: Sealed {}
trait Sealed {}

/// Prevents usage of [`as_slot_ref`] and [`as_slot_mut`].
#[derive(Debug)]
pub enum SlotVisPriv {}

impl Sealed for SlotVisPriv {}
impl SlotVisibility for SlotVisPriv {}

/// Allows usage of [`as_slot_ref`] and [`as_slot_mut`].
#[derive(Debug)]
pub enum SlotVisPub {}

impl Sealed for SlotVisPub {}
impl SlotVisibility for SlotVisPub {}

pub(crate) const fn private_as_slot_ptr<T, C: RawSlotRef<T, Visibility = V>, V>(container: *const C) -> *const Slot<T> {
	// SAFETY: contracts upheld by implementer of `RawSlotRef`
	unsafe { container.byte_add(<C as RawSlotRef<T>>::OFFSET).cast::<Slot<T>>() }
}

pub(crate) const fn private_as_slot_mut_ptr<T, C: RawSlotRef<T, Visibility = V>, V>(container: *mut C) -> *mut Slot<T> {
	// SAFETY: contracts upheld by implementer of `RawSlotRef`
	unsafe { container.byte_add(<C as RawSlotRef<T>>::OFFSET).cast::<Slot<T>>() }
}

/// Safety concerns should be upheld by the implementation of the [`RawSlotRef`] trait.
#[inline]
pub(crate) const fn private_as_slot_ref<T, C: RawSlotRef<T, Visibility = V>, V>(container: &C) -> &Slot<T> {
	// SAFETY: contracts upheld by implementer of `RawSlotRef`
	unsafe { &*private_as_slot_ptr::<T, C, V>(container) }
}

/// Safety concerns should be upheld by the implementation of the [`RawSlotRef`] trait.
#[inline]
pub(crate) const fn private_as_slot_mut<T, C: RawSlotRef<T, Visibility = V>, V>(container: &mut C) -> &mut Slot<T> {
	// SAFETY: contracts upheld by implementer of `RawSlotRef`
	unsafe { &mut *private_as_slot_mut_ptr::<T, C, V>(container) }
}

/// Opt to use [`as_slot_ref`] instead.
///
/// This is provided for use in unsafe code, which may not abide by visibility rules.
/// Breaking changes may appear in minor (or patch) version bumps.
///
/// Safety concerns should be upheld by the implementation of the [`RawSlotRef`] trait.
#[inline]
pub const fn as_slot_ptr<T>(container: *const impl RawSlotRef<T>) -> *const Slot<T> {
	private_as_slot_ptr(container)
}

/// Opt to use [`as_slot_mut`] instead.
///
/// This is provided for use in unsafe code, which may not abide by visibility rules.
/// Breaking changes may appear in minor (or patch) version bumps.
///
/// Safety concerns should be upheld by the implementation of the [`RawSlotRef`] trait.
#[inline]
pub const fn as_slot_mut_ptr<T>(container: *mut impl RawSlotRef<T>) -> *mut Slot<T> {
	private_as_slot_mut_ptr(container)
}

/// Const version of [`AsRef<Slot<T>>`].
#[inline]
pub const fn as_slot_ref<T>(container: &impl RawSlotRef<T, Visibility = SlotVisPub>) -> &Slot<T> {
	private_as_slot_ref(container)
}

/// Const version of [`AsMut<Slot<T>>`].
#[inline]
pub const fn as_slot_mut<T>(container: &mut impl RawSlotRef<T, Visibility = SlotVisPub>) -> &mut Slot<T> {
	private_as_slot_mut(container)
}
