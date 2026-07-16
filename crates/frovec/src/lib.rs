#![allow(unused)]

use std::marker::PhantomData;
use std::mem::MaybeUninit;

/// Field re-ordering [`Vec`].
pub struct FroVec<T: FieldMap> {
	_marker: PhantomData<T>,
	chunks: Vec<T::Fro>,
	len: usize,
}

pub trait FieldMap {
	type Fro;

	const SIZE: usize;
}

#[derive(Debug)]
struct Juicy {
	balance: u64,
	state: u8,
	zip: u16,
}

impl FieldMap for Juicy {
	type Fro = (
		[MaybeUninit<u64>; Self::SIZE],
		[MaybeUninit<u8>; Self::SIZE],
		[MaybeUninit<u16>; Self::SIZE],
	);

	const SIZE: usize = 8;
}
