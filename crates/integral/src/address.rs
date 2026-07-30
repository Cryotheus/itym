pub trait Address: Clone + Copy + Eq + Ord + Sized + Send + Sync + 'static {}

macro_rules! impl_addr {
	(
		$($target:ty),+
		$(,)?
	) => {
		$(impl Address for $target {})+
	};
}

impl_addr!(u8, u16, u32, u64, u128, usize);
