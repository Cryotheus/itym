//! # Itym
//! Specialized collections, macros, and monads for your special situations.
//!
//! Explore the crate's modules to learn more.
#![allow(unexpected_cfgs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//TODO: bake descriptions?
macro_rules! export {
	( $($og:ident as $rename:ident if $feature:literal),+ $(,)? ) => {
		$(#[doc = include_str!(concat!("../crates/", stringify!($rename), "/description.md"))]
		#[cfg(feature = $feature)]
		pub mod $rename { pub use $og::*; })+
	};
}

export! {
	itym_assert as assert if "itym_assert",
	itym_slot as slot if "itym_slot",
	itym_util as util if "itym_util",
}
