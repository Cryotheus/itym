# Itym

Specialized collections, macros, and monads for your special situations.

## Soundness

Items with unresolved soundness concerns are marked `unsafe`.
That is to say: some sound items may be marked `unsafe`, and all unsound items are marked `unsafe`.
Exact crate goals are tentative

## Features

Any enabled `itym_*` feature also enables the features of their dependencies,
unless doing so increases burden on:

- The build system.
	- in the form of increased compile times
- The developer.
	- In the form of overlapping identifiers causing erroneous auto-imports

Exceptions are in place for crates which are nominally undesired.

## Goals

Incomplete and tentative list.

- Developers using `#![deny(unsafe_code)]` gain soundness guarantees
- `#![no_std]` in the form of `std` and `alloc` features where possible
- Maximum `const` coverage, without forsaking runtime performance
	- Even if runtime-optimized and `const` variants are
- Error as early as possible, roughly:
	1. Refuse construction in which error is possible
		- By `impl` predicates and trait bounds
	2. Assert using errors caught by `cargo check`
		- Such as `const _: () = panic!();`
	3. Assert using errors caught by `cargo build`
		- Such as `const { panic!(); }`
	4. Assert at runtime

## MSRV Policy

Bumping MSRV is not considered a semver-breaking change.

# License

This project is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
  https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `itym` by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
