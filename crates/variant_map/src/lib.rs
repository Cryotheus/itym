mod example;

use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::RandomState;

pub unsafe trait VariantMapKey: Sized {
	type Entry: Sized;
	type EntryMut<'a>: Sized;
	type EntryRef<'a>: Sized;
	type Union: Sized;

	unsafe fn drop_entry(&self, entry: &mut Self::Union);
}

pub struct VariantMap<K: VariantMapKey, S = RandomState>(HashMap<K, K::Union, S>);

impl<K, S> Debug for VariantMap<K, S>
where
	K: VariantMapKey + Debug,
	<K as VariantMapKey>::Entry: Debug,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("VariantMap").finish_non_exhaustive()
	}
}

impl<K: VariantMapKey, S> Drop for VariantMap<K, S> {
	fn drop(&mut self) {
		for (key, entry) in self.0.iter_mut() {
			unsafe { key.drop_entry(entry) };
		}

		todo!()
	}
}
