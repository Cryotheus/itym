/// `const` compatible [`assert`] returning an `Err(..)` or `None` instead of panicking.
#[macro_export]
macro_rules! ensure {
	($cond:expr, $err:expr $(,)?) => {
		if let false = ($cond) {
			#[allow(unreachable_code)]
			return ::core::result::Result::Err($err);
		}
	};

	($cond:expr $(,)?) => {
		if let false = ($cond) {
			#[allow(unreachable_code)]
			return ::core::option::Option::None;
		}
	};
}

#[macro_export]
macro_rules! ensure_eq {
	($lhs:expr, $rhs:expr $(, $err:expr $(,)?)?) => {
		$crate::ensure!(*(&$lhs) == *(&$rhs) $(, $err)?)
	};
}

#[macro_export]
macro_rules! ensure_ne {
	($lhs:expr, $rhs:expr $(, $err:expr $(,)?)?) => {
		$crate::ensure!(*(&$lhs) != *(&$rhs) $(, $err)?)
	};
}

/// Inversion of [`ensure`].
#[macro_export]
macro_rules! refuse {
	($cond:expr, $err:expr $(,)?) => {
		if let true = ($cond) {
			#[allow(unreachable_code)]
			return ::core::result::Result::Err($err);
		}
	};

	($cond:expr $(,)?) => {
		if let true = ($cond) {
			#[allow(unreachable_code)]
			return ::core::option::Option::None;
		}
	};
}

#[macro_export]
macro_rules! refuse_eq {
	($lhs:expr, $rhs:expr $(, $err:expr $(,)?)?) => {
		$crate::refuse!(*(&$lhs) == *(&$rhs) $(, $err)?)
	};
}

#[macro_export]
macro_rules! refuse_ne {
	($lhs:expr, $rhs:expr $(, $err:expr $(,)?)?) => {
		$crate::refuse!(*(&$lhs) != *(&$rhs) $(, $err)?)
	};
}
