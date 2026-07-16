use std::{marker::PhantomData, ptr::NonNull};

/// Analog of [`Vec<(T, U)>`].
pub struct BiVec<T, U> {
	_t_marker: PhantomData<T>,
	_u_marker: PhantomData<U>,
	ptr: NonNull<u8>,
	capacity: usize,
	t_len: usize,
	u_len: usize,
}
