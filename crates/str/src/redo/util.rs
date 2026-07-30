use core::mem::ManuallyDrop;

/// Transmutation for depentendly-sized types which is not supported by [`core::mem::transmute`].
///
/// # Safety
/// Same concerns as [`transmute`].
/// Constant evaluations in the body of the function will fail
/// if size or alignment requirements are not satisfied.
///
/// [`transmute`]: core::mem::transmute
#[inline(always)]
pub(crate) const unsafe fn transmute2<Src, Dst>(src: Src) -> Dst {
	unsafe { transmute_lenient::<Src, Dst, true>(src) }
}

/// Transmutation for depentendly-sized types which is not supported by [`core::mem::transmute`].
///
/// # Safety
/// Same concerns as [`transmute`].
///
/// [`transmute`]: core::mem::transmute
#[inline(always)]
pub(crate) const unsafe fn transmute_lenient<Src, Dst, const ASSERTIONS: bool>(src: Src) -> Dst {
	/// For transmutation which is not supported by [`core::mem::transmute`].
	union Transmute<Src, Dst> {
		src: ManuallyDrop<Src>,
		dst: ManuallyDrop<Dst>,
	}

	const {
		["size: Src and Dst must be equal"][(size_of::<Src>().wrapping_sub(size_of::<Dst>())) * if ASSERTIONS { 1 } else { 0 }];
		["align: Dst must be a multiple of Src"][(!align_of::<Src>().is_multiple_of(align_of::<Dst>())) as usize * if ASSERTIONS { 1 } else { 0 }];
	};

	impl<Src, Dst> Transmute<Src, Dst> {
		#[inline(always)]
		pub const fn new(src: Src) -> Self {
			Self { src: ManuallyDrop::new(src) }
		}
	}

	ManuallyDrop::into_inner(unsafe { Transmute::<Src, Dst>::new(src).dst })
}

macro_rules! const_assert {
	(macro $(#[$meta:meta])* [$left:expr]) => {
		$crate::util::const_assert!(
			macro $(#[$meta])* [
				$left,
				"{}",
				::core::concat!(
					"Assertion failed ",
					::core::stringify!($left)
				)
			]
		)
	};

	(macro $(#[$meta:meta])* [$left:expr, $($message:tt)*]) => {
		$(#[$meta])*
		if let false = ($left) { ::core::panic!(
			$($message)*
		); }
	};

	(macro unsafe $(#[$meta:meta])* [$left:expr $(, $($tt:tt)*)?]) => {
		$(#[$meta])*
		unsafe { ::core::hint::assert_unchecked($left) }
	};

	//user-facing
	(const: $left:expr $(, $($message:tt)*)?) => { const { $crate::util::const_assert!(macro [$left $(, $($message)*)?]) } };
	// (const unsafe: $left:expr $(, $($message:tt)*)?) => { const { $crate::util::const_assert!(macro unsafe [$left $(, $($message)*)?]) } };
	// (unsafe: $left:expr $(, $($message:tt)*)?) => { $crate::util::const_assert!(macro unsafe [$left $(, $($message)*)?]) };
	($left:expr $(, $($message:tt)*)?) => { $crate::util::const_assert!(macro [$left $(, $($message)*)?]) };
}

macro_rules! const_assert_eq {
	(macro $(#[$meta:meta])* [$left:expr, $right:expr]) => {
		$crate::util::const_assert_eq!(
			macro $(#[$meta])* [
				$left,
				$right,
				"{}",
				::core::concat!(
					"Assertion failed ",
					::core::stringify!($left),
					" == ",
					::core::stringify!($right)
				)
			]
		)
	};

	(macro $(#[$meta:meta])* [$left:expr, $right:expr, $($message:tt)*]) => {
		$(#[$meta])*
		if ($left) != ($right) { ::core::panic!($($message)*); }
	};

	(macro unsafe $(#[$meta:meta])* [$left:expr, $right:expr $(, $($tt:tt)*)?]) => {
		$(#[$meta])*
		unsafe { ::core::hint::assert_unchecked($left == $right) }
	};

	//user-facing
	(const: $left:expr, $right:expr $(, $($message:tt)*)?) => { const { $crate::util::const_assert_eq!(macro [$left, $right $(, $($message)*)?]) } };
	// (const unsafe: $left:expr, $right:expr $(, $($message:tt)*)?) => { const { $crate::util::const_assert_eq!(macro unsafe [$left, $right $(, $($message)*)?]) } };
	// (unsafe: $left:expr, $right:expr $(, $($message:tt)*)?) => { $crate::util::const_assert_eq!(macro unsafe [$left, $right $(, $($message)*)?]) };
	($left:expr, $right:expr $(, $($message:tt)*)?) => { $crate::util::const_assert_eq!(macro [$left, $right $(, $($message)*)?]) };
}

macro_rules! const_assert_ne {
	(macro $(#[$meta:meta])* [$left:expr, $right:expr]) => {
		$crate::util::const_assert_ne!(
			macro $(#[$meta])* [
				$left,
				$right,
				"{}",
				::core::concat!(
					"Assertion failed ",
					::core::stringify!($left),
					" != ",
					::core::stringify!($right)
				)
			]
		)
	};

	(macro $(#[$meta:meta])* [$left:expr, $right:expr, $($message:tt)*]) => {
		$(#[$meta])*
		if ($left) == ($right) { ::core::panic!($($message)*); }
	};

	(macro unsafe $(#[$meta:meta])* [$left:expr, $right:expr $(, $($tt:tt)*)?]) => {
		$(#[$meta])*
		unsafe { ::core::hint::assert_unchecked($left != $right) }
	};

	//user-facing
	(const: $left:expr, $right:expr $(, $($message:tt)*)?) => { const { $crate::util::const_assert_ne!(macro [$left, $right $(, $($message)*)?]) } };
	// (const unsafe: $left:expr, $right:expr $(, $($message:tt)*)?) => { const { $crate::util::const_assert_ne!(macro unsafe [$left, $right $(, $($message)*)?]) } };
	// (unsafe: $left:expr, $right:expr $(, $($message:tt)*)?) => { $crate::util::const_assert_ne!(macro unsafe [$left, $right $(, $($message)*)?]) };
	($left:expr, $right:expr $(, $($message:tt)*)?) => { $crate::util::const_assert_ne!(macro [$left, $right $(, $($message)*)?]) };
}

macro_rules! const_debug_assert {
	//user-facing
	(const: $expr:expr $(, $($message:tt)*)?) => {
		const { $crate::util::const_assert!(macro #[cfg(debug_assertions)] [$expr $(, $($message)*)?]) }
	};

	(const unsafe: $expr:expr $(, $($message:tt)*)?) => {
		const {
			$crate::util::const_assert!(macro #[cfg(debug_assertions)] [$expr $(, $($message)*)?]);
			$crate::util::const_assert!(macro unsafe #[cfg(not(debug_assertions))] [$expr $(, $($message)*)?]);
		}
	};

	(unsafe: $expr:expr $(, $($message:tt)*)?) => {
		{
			$crate::util::const_assert!(macro #[cfg(debug_assertions)] [$expr $(, $($message)*)?]);
			$crate::util::const_assert!(macro unsafe #[cfg(not(debug_assertions))] [$expr $(, $($message)*)?]);
		}
	};

	($expr:expr $(, $($message:tt)*)?) => {
		$crate::util::const_assert!(macro #[cfg(debug_assertions)] [$expr $(, $($message)*)?])
	};
}

macro_rules! const_debug_assert_eq {
	(const: $left:expr, $right:expr $(, $($message:tt)*)?) => {
		const { $crate::util::const_assert_eq!(macro [$left, $right $(, $($message)*)?]) }
	};

	(const unsafe: $left:expr, $right:expr $(, $($message:tt)*)?) => {
		const {
			$crate::util::const_assert_eq!(macro #[cfg(debug_assertions)] [$left, $right $(, $($message)*)?]);
			$crate::util::const_assert_eq!(macro unsafe #[cfg(not(debug_assertions))] [$left, $right $(, $($message)*)?]);
		}
	};

	(unsafe: $left:expr, $right:expr $(, $($message:tt)*)?) => {
		{
			$crate::util::const_assert_eq!(macro #[cfg(debug_assertions)] [$left, $right $(, $($message)*)?]);
			$crate::util::const_assert_eq!(macro unsafe #[cfg(not(debug_assertions))] [$left, $right $(, $($message)*)?]);
		}
	};

	($left:expr, $right:expr $(, $($message:tt)*)?) => {
		$crate::util::const_assert_eq!(macro [$left, $right $(, $($message)*)?])
	};
}

macro_rules! const_debug_assert_ne {
	(const: $left:expr, $right:expr $(, $($message:tt)*)?) => {
		const { $crate::util::const_assert_ne!(macro [$left, $right $(, $($message)*)?]) }
	};

	(const unsafe: $left:expr, $right:expr $(, $($message:tt)*)?) => {
		const {
			$crate::util::const_assert_ne!(macro #[cfg(debug_assertions)] [$left, $right $(, $($message)*)?]);
			$crate::util::const_assert_ne!(macro unsafe #[cfg(not(debug_assertions))] [$left, $right $(, $($message)*)?]);
		}
	};

	(unsafe: $left:expr, $right:expr $(, $($message:tt)*)?) => {
		{
			$crate::util::const_assert_ne!(macro #[cfg(debug_assertions)] [$left, $right $(, $($message)*)?]);
			$crate::util::const_assert_ne!(macro unsafe #[cfg(not(debug_assertions))] [$left, $right $(, $($message)*)?]);
		}
	};

	($left:expr, $right:expr $(, $($message:tt)*)?) => {
		$crate::util::const_assert_ne!(macro [$left, $right $(, $($message)*)?])
	};
}

macro_rules! debug_unreachable {
	() => {{
		#[cfg(debug_assertions)]
		{
			unsafe fn debug_unreachable_requires_unsafe() {} //force a lint
			core::unreachable!();
			#[allow(unreachable_code)]
			debug_unreachable_requires_unsafe();
		}

		#[cfg(not(debug_assertions))]
		core::hint::unreachable_unchecked();
	}};
}

#[allow(unused_imports)]
pub(crate) use {
	const_assert, const_assert_eq, const_assert_ne, const_debug_assert, const_debug_assert_eq, const_debug_assert_ne, debug_unreachable,
};
