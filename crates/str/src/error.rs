//! General error types.

use core::error::Error;
use core::fmt::{Display, Formatter};
use core::num::NonZero;

/// Emitted if capacity requirements are not met.
///
/// Contains the amount of additional bytes the destination buffer must have allocated to complete the operation.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct CapacityError(pub NonZero<usize>);

impl CapacityError {
	pub(crate) const fn new(space: usize, usage: usize) -> Option<CapacityError> {
		match NonZero::new(usage.saturating_sub(space)) {
			None => None,
			Some(nz) => Some(Self(nz)),
		}
	}
}

impl Display for CapacityError {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.write_fmt(format_args!("Missing {} byte(s) of space in destination buffer", &self.0))
	}
}

impl Error for CapacityError {}
