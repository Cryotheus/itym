//! # Itym
//! Specialized collections, macros, and monads for your special situations.
//!
//! Explore the crate's re-exports / modules to learn more.
#![cfg_attr(docsrs, feature(doc_cfg))]

//TODO: bake descriptions?
macro_rules! export {
	( $($og:ident as $rename:ident if $feature:literal),+ $(,)? ) => {
		$(#[cfg(feature = $feature)]
		#[doc(inline)]
		pub use $og as $rename;)+
	};
}

export! {
	itym_assert as assert if "itym_assert",
	itym_mem as mem if "itym_mem",
	itym_slot as slot if "itym_slot",
	itym_str as str if "itym_str",
	itym_ts as ts if "itym_ts",
	itym_util as util if "itym_util",
}
