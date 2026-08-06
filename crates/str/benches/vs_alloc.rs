#![allow(unused)]
#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use core::hint::black_box;
use core::iter;
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use itym_str::ArrayStr;

macro_rules! gen_bench_fns {
	(macro type $literal:literal) => {
		&[u8; $literal.len()]
	};

	(
		macro 1 [$($literal:literal)*] $literal2:tt $($tt:tt)*
	) => {};

	(
		macro 0 $literals_0:tt $($fn_bench:ident $cg_base:ident $cg_push:ident $base:ident $push:ident $body:tt)*
	) => {
		
	};

	(
		const $name:ident = [$($literal:literal),* $(,)?];

		$(
		$(#[$meta:meta])*
		fn $fn_bench:ident <$cg_base:ident, $cg_push:ident> ($base:ident, $push:ident) {
			$($body:tt)*
		}
		)*
	) => {
		gen_bench_fns! {
			macro 0
			[$($literal)*]

			$(
			$fn_bench
			$cg_base
			$cg_push
			$base
			$push
			{$($body)*}
			)+
		}

		// $(
		// $(#[$meta])*
		//
		// fn $fn_bench<const $cg_base: usize, const $cg_push: usize>($base: &str, $push: &str) {
		// 	// gen_bench_fns! { macro 0 [$($literal)*] [$($literal)*] }
		//
		//
		// 	// $($body)*
		// }
		// )*

		// const $name: ($(literals!(macro type $literal)),*) = (
		// 	$($literal),*
		// );
	};
}

gen_bench_fns! {
	const STRINGS = [
		b"How is ",
		b"Let us meet again, ",
		b"In that general direction I hear *malicious skeleton riff*, could it be ",
		b"Should that wire be glowing, ",
		b"Pickles just attacked the box, ",
		b"Did you put the bag on the box, ",
		b"Grease sold our ",
		b"Where's our ",
		b"... ",
		b"This is a very long sentence, I know it doesn't look like it yet but trust me it is. Also, I ask you give my greetings to ",
		b"Here's a bunch of UUIDv4 strings to make a really long literal: \
		11eb4062-2729-4419-a5a8-d2edd79be2d1 \
		311e069f-f113-42ee-abcd-ab24512aaf14 \
		4e5f4c34-ffaa-48d6-8676-a7cb1d73f884 \
		1b2cb8dc-2f3b-435a-926e-6f0ef4ca4383 \
		c654ef23-06ff-4b17-ab5c-8cfbace836b1 \
		f42f442f-fc4c-4dcb-a1e1-100ca417583c \
		af7d474e-8567-4097-b6d4-5ad2cc738c2e \
		3a7bf005-cc6f-498e-9d43-994c50bb1ac9 \
		556e9c81-4d44-4698-8de8-9bbf8b9874bc \
		83659f51-35f1-4071-afcf-8fee07135ced \
		b63bbbc5-410f-414c-996c-aff51aa98829 \
		328f03cb-d997-49b4-8c48-654f94871553 \
		bc36c979-0c06-4247-8f60-3ac181bc4afa \
		dc0512e8-db7a-419d-a3a7-2b0b7f620733 \
		a1f92e36-c351-4dfa-b1f0-f302cd17672d \
		90f95a75-e7b4-429b-98c7-b9b6b536d254 \
		ee2d3013-4c1d-47b3-9de5-a362adab95f0 \
		612397c8-3593-4947-b90d-288789b845dc \
		fc392a8e-5bd2-43e5-ae70-a98079a1a811 \
		af3078cd-5476-4554-ade8-cd361fc9981f \
		f8f41f86-f19a-4af5-8bb5-a4b8792082c0 \
		a0ed5331-59e2-4fe7-8b25-9d5d54213f1f \
		d25a96b7-ab91-45bd-a726-832bf2256be6 \
		45a2442b-4bb4-4738-a3ac-8d75647902b1 \
		f3a37bea-25dc-4bcf-87f9-b85fd424c63e \
		ef85c3ae-17da-4c7c-a57b-56f7b5780947 \
		da2d71e6-2143-4968-9698-12add892ed76 \
		55b62bcb-60aa-49a4-a58a-346baf3e90d2 \
		97ba059b-c008-4252-8de6-e0a6d279b0dd \
		53442b0c-e932-4c92-8e1c-7940050b9bc6 \
		0ba246b2-46de-48f9-87d1-92d26e05911a \
		8e43714b-c1c7-4b83-84a8-ecceaf6881af \
		46abf952-3d38-4cd1-b425-550bacfdfa98 \
		d2ad26bb-1bd3-49bf-a925-5ece6c5eec0d \
		ed8210da-b89d-4a00-89d8-4d5f61e22823 \
		a1f69b86-56bb-4de6-800b-8cfb8113d189 \
		24bbb80f-2c89-4080-a0bf-e84c997a179d \
		8739f5bb-78c5-48b9-972a-8b2c33fc8d2e \
		79e91753-1308-4774-b947-93fa665d267a \
		a7642c9e-185a-4fb2-a801-4da54ae864e6 \
		23d74540-4c11-4d93-a490-d3986321c3e7",
		b"the world",
		b"Jon",
		b"Sinclair",
		b"home",
		b"tic-tac-toe project",
		b"pickles, please get off the desk, thank you",
		b"ope, she's going for the eggs again",
		b"now that I look at it, why is the window wrapped up for winter",
		b"what's a question mark",
	];

	fn array_str_concat<BASE, PUSH>(base, push) {
		let base = ArrayStr::<BASE>::from_str(base).unwrap();
		let push = ArrayStr::<PUSH>::from_str(push).unwrap();

		todo!("{} {}", base, push)
	}
}

fn bench(c: &mut Criterion) {
	let mut group = c.benchmark_group("sample-size-example");

	group.bench_function("my-function", |b| {
		b.iter_batched(
			move || (),
			|argument| {
				//bench mark
			},
			BatchSize::SmallInput,
		)

		// b.iter(|| array_str_concat())
	});

	group.finish();
}

criterion_group! {
	name = benches;
	config = Criterion::default().significance_level(0.05).sample_size(500);
	targets = bench
}

criterion_main!(benches);
