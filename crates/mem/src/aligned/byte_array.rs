use crate::aligned::AlignedManuallyDrop;
use crate::pod::Pod;
use core::ops::Deref;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ByteArray<const LEN: usize, A = usize>(AlignedManuallyDrop<[u8; LEN], A>);

impl<const LEN: usize, A> ByteArray<LEN, A> {}

impl<const LEN: usize, A> ByteArray<LEN, A> {
	pub const fn as_slice(&self) -> &[u8] {
		self.0.as_ref()
	}

	pub const fn as_mut_slice(&mut self) -> &mut [u8] {
		self.0.as_mut()
	}

	pub const fn as_ref(&self) -> &[u8; LEN] {
		self.0.as_ref()
	}

	pub const fn as_mut(&mut self) -> &mut [u8; LEN] {
		self.0.as_mut()
	}

	pub const fn copy(&self) -> Self {
		Self(self.0.copy())
	}
}

impl<const LEN: usize, A> Deref for ByteArray<LEN, A> {
	type Target = [u8; LEN];

	fn deref(&self) -> &Self::Target {
		Self::as_ref(self)
	}
}

impl<const LEN: usize, A: Copy> Copy for ByteArray<LEN, A> {}

// unsafe impl<const LEN: usize> Pod for ByteArray<LEN, u8> {}
