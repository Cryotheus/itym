//! Implementations specialized by compilation features.

macro_rules! gated {
	($($module:ident $feature:literal $path:literal),+ $(,)?) => {
		$(#[cfg(feature = $feature)]
		#[path = $path]
		mod $module;)+
	};
}

gated! {
	alloc_impls "alloc" "alloc.rs",
	borsh_impls "borsh" "borsh.rs",
	serde_impls "serde" "serde.rs",
	std_impls "std" "std.rs",
}
