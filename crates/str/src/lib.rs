//! # Itym: str
#![doc = include_str!("../description.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

pub mod error;
mod impls;
pub mod slot_str;
pub mod slot_string;
pub mod terminated;

use crate::slot_str::SlotStr;
use crate::slot_string::SlotString;

pub type ArrayStr<const LEN: usize> = SlotStr<[u8; LEN]>;
pub type ArrayString<const LEN: usize> = SlotString<[u8; LEN]>;
