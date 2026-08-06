use crate::utransmute;
use core::mem::ManuallyDrop;
use itym_assert::*;

/// Plain old data.
///
/// # Safety
/// - `Copy + Send + Sync + Sized + 'static`
/// - Inhabited (these do not qualify: `!`, `Infallible`)
/// - Not as ZST (these do not qualify: `()`, `PhantomData`, `PhantomPinned`)
///   - ZSTs defeat the point of a "plan old data"
/// - No padding
/// - Allows all bit patterns (these do not qualify: `bool`, `char`)
/// - Does not contain references
/// - Does not contain types with niches (these do not qualify: `NonZero`, `NonNull`, `Box<T>`)
///   - `Option<NonZero<T>>` may implement `Pod` in the future
///
/// ## Notes
/// Will never be implemented for `f16`, `f32`, `f64`, or `f128`
/// as not all systems potentially targetable by nightly Rust support any bit pattern for these types.
///
/// As of right now, `usize` and `isize` implement `Pod` but this may change in the future.
pub unsafe trait Pod: Copy + Sized + Send + Sync + 'static {
	type AlignPod: Copy + Sized + Send + Sync + 'static;
	type Bytes: Copy + Sized + Send + Sync + 'static;

	const ASSERTIONS: () = {
		const_assert_eq!(const: align_of::<<Self as Pod>::Bytes>(), 1, "Pod types must be unaligned");
		const_assert_ne!(const: size_of::<<Self as Pod>::Bytes>(), 0, "Pod types cannot be zero-sized");
		const_assert!(const: size_of::<<Self as Pod>::Bytes>() <= isize::MAX as usize, "Pod types must be of a safe size to allocate");
	};
}

unsafe impl<const LEN: usize> Pod for [u8; LEN] {
	type AlignPod = u8;
	type Bytes = [u8; LEN];
}

unsafe impl<T: Pod> Pod for ManuallyDrop<T> {
	type AlignPod = T::AlignPod;
	type Bytes = T::Bytes;
}

macro_rules! impl_pod {
	(macro $first:ty $(, $tail:ty)* $(,)?) => { $first };

	($($target:ty $(= $align:ty)?),* $(,)?) => {
		$(unsafe impl Pod for $target {
			type AlignPod = impl_pod!(macro $($align, )? $target);
			type Bytes = [u8; size_of::<Self>()];
		})*
	};
}

impl_pod! {
	u8,
	u16,
	u32,
	u64,
	u128,
	usize,
	i8 = u8,
	i16 = u16,
	i32 = u32,
	i64 = u64,
	i128 = u128,
	isize = usize,
}

/// Safe.
pub const fn pod_into_bytes<Src: Pod>(src: Src) -> Src::Bytes {
	// SAFETY: contracts upheld by implementer of `Pod`
	unsafe { utransmute::<Src, Src::Bytes>(src) }
}

/// Safe.
///
/// # Panics
/// If the size of `Src` and `Dst` are different.
pub const fn transmute_pod<Src: Pod, Dst: Pod>(src: Src) -> Dst {
	// SAFETY: contracts upheld by implementer of `Pod`
	if const { size_of::<Src>() == size_of::<Dst>() } {
		unsafe { utransmute::<Src, Dst>(src) }
	} else {
		panic!()
	}
}
