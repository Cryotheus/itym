//! # Itym: str
#![doc = include_str!("../description.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod array_byte_str;
pub mod array_str;
pub mod array_string;
pub mod error;
pub mod pod;
pub(crate) mod util;

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc as liballoc;

#[cfg(all(feature = "alloc", not(feature = "std")))]
pub(crate) use liballoc as alloc;

#[cfg(feature = "std")]
pub(crate) use ::std as alloc;
