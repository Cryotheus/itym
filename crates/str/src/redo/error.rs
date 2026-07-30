use core::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct CapacityError(());

impl CapacityError {
	pub(crate) const fn new() -> Self {
		Self(())
	}
}

impl Display for CapacityError {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.write_str("Insufficient capacity")
	}
}
