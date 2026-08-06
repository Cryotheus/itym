use itym_assert::{const_assert, const_assert_eq, const_assert_ne};
use std::marker::PhantomData;
use std::num::NonZero;
use std::ptr::NonNull;

pub struct ErasedVec {
	ptr: NonNull<u8>,
	virtuals: &'static ErasedVecVirtuals,
}

struct ErasedVecVirtuals {
	
}

pub struct RainbowSlab<const RAINBOW: u128, const BANDS: usize> {
	bands: [ErasedVec; BANDS],
}

impl<const RAINBOW: u128, const BANDS: usize> RainbowSlab<RAINBOW, BANDS> {
	pub const fn new() -> Self {
		const_assert_eq!(RAINBOW.count_ones() as usize, BANDS);

		todo!()
	}
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct RainbowSlabId<T> {
	index: NonZero<usize>,
	_marker: PhantomData<fn() -> T>,
}

#[derive(Debug, Clone)]
pub struct RainbowBuilder<T> {
	_marker: PhantomData<fn() -> T>,
}

impl<T> RainbowBuilder<T> {
	pub const fn new() -> RainbowBuilder<(T,)> {
		const_assert!(const { size_of::<T>() != 0 });
		const_assert!(const { size_of::<T>() <= 128 });

		RainbowBuilder { _marker: PhantomData }
	}

	pub const fn register<U>(self) -> RainbowBuilder<(T, U)> {
		const_assert!(const { size_of::<T>() != 0 });
		const_assert!(const { size_of::<T>() <= 128 });

		RainbowBuilder { _marker: PhantomData }
	}

	pub const fn bands(&self) -> usize {
		<(T,) as Rainbow>::RAINBOW.count_ones() as usize
	}

	pub const fn rainbow(&self) -> u128 {
		<(T,) as Rainbow>::RAINBOW
	}
}

pub trait Rainbow {
	const RAINBOW: u128;
}

impl<T> Rainbow for (T,) {
	const RAINBOW: u128 = {
		if let size @ 1..=128 = size_of::<T>() {
			1u128 << (size - 1) as u32
		} else {
			0
		}
	};
}

impl<T, U> Rainbow for (T, U) {
	const RAINBOW: u128 = {
		let t = <(T,) as Rainbow>::RAINBOW;
		let u = <(U,) as Rainbow>::RAINBOW;

		if let 0 = t & u { 0 } else { t | u }
	};
}

impl<T0, T1, T2> Rainbow for (T0, T1, T2) {
	const RAINBOW: u128 = {
		let t0 = <(T0,) as Rainbow>::RAINBOW;
		let t1 = <(T1,) as Rainbow>::RAINBOW;
		let t2 = <(T2,) as Rainbow>::RAINBOW;

		if let 0 = t0 & t1 & t2 { 0 } else { t0 | t1 | t2 }
	};
}
