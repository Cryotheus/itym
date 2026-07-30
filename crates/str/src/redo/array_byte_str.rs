use crate::pod::PodStr;

/// An exact length [`ByteStr`].
///
/// [`ByteStr`]: core::bstr::ByteStr
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(not(feature = "c_abi"), repr(transparent))]
#[cfg_attr(feature = "c_abi", repr(C))]
pub struct ArrayByteStr<const LEN: usize>(PodStr<[u8; LEN]>);