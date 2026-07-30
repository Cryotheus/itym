use crate::pod::PodStr;

/// An exact length [`str`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(not(feature = "c_abi"), repr(transparent))]
#[cfg_attr(feature = "c_abi", repr(C))]
pub struct ArrayStr<const LEN: usize>(pub PodStr<[u8; LEN]>);

impl<const LEN: usize> ArrayStr<LEN> {
	pub const fn lit(byte_str_literal: &'static [u8; LEN]) -> Self {
		Self(PodStr::lit(byte_str_literal))
	}
}