# Itym

Specialized collections, macros, and monads for your special situations.

## Soundness

Items with unresolved soundness concerns are marked `unsafe`.
That is to say: some sound items may be marked `unsafe`, and all unsound items are marked `unsafe`.
Exact crate goals are tentative

## Features

The default feature set enables default features for other enabled sub-crates.

E.g.

- `itym_ts` has `size_common` in its default feature set
- if the features `itym_ts` and `default` are enabled, `itym_ts/size_common` is enabled

| Feature         | Purpose                                                                                                    |
|-----------------|------------------------------------------------------------------------------------------------------------|
| `itym_assert`   | Re-exports `itym_assert`                                                                                   |
| `itym_slot`     | Re-exports `itym_slot`                                                                                     |
| `itym_str`      | Re-exports `itym_str`                                                                                      |
| `itym_ts`       | Re-exports `itym_ts`                                                                                       |
| `itym_util`     | Re-exports `itym_util`                                                                                     |
|                 |                                                                                                            |
| Sub-crate       |                                                                                                            |
| `enforce_32bit` | Emits a compile error if the target pointer width is less than 32 bits, and all 32-bit+ dependent features |
| `enforce_64bit` | Emits a compile error if the target pointer width is less than 64 bits, and all 64-bit dependent features  |
|                 |                                                                                                            |
| `size_big`      | Enables `itym_ts/size_big`                                                                                 |
| `size_common`   | Enables `itym_ts/size_common`                                                                              |
|                 |                                                                                                            |
| Rust            |                                                                                                            |
| `alloc`         | Enable `liballoc` dependent features                                                                       |
| `std`           | Enable `libstd` dependent features                                                                         |
|                 |                                                                                                            |
| Nightly         |                                                                                                            |
| `f16`           |                                                                                                            |
| `f128`          |                                                                                                            |
|                 |                                                                                                            |
| Crates (3p)     |                                                                                                            |
| `borsh`         | `borsh` impls for enabled `itym` crates                                                                    |
| `serde`         | `serde` impls for enabled `itym` crates                                                                    |

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
