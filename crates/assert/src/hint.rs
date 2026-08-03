/// An alternative to the unstable compile option `-Zub-checks`.
///
/// If `debug_assertions` are enabled, acts as [`unreachable`], otherwise behaves like [`core::hint::unreachable_unchecked`].
#[macro_export]
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
