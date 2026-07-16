//! TODO: module-level docs?

macro_rules! export {
	( $($og:ident as $rename:ident if $feature:literal),+ $(,)? ) => {
		$(
		#[cfg(feature = $feature)]
		pub use $og as $rename;
		)+
	};
}

export! {
	itym_bivec as bivec if "bivec",
	itym_frovec as frovec if "frovec",
	itym_integral as integral if "integral",
}
