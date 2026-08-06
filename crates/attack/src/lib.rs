//! Somebody scream.
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[derive(Debug)]
pub struct GlobalSortie {
	//
}

#[derive(Debug)]
pub struct LocalSortie {
	//
}

pub trait SortieConfig {
	const VARIANTS: u16;
}

pub trait AttackVariant {
	type Attack: Copy + Sized + Send + Sync + 'static;

	const BYTES_WHOLE: usize = size_of::<Self::Attack>();
	const MAX_INCLUSIVE: Self::Attack;
}

pub trait AttackArchive {

}

pub trait AttackStore {
	
}
