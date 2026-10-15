# celestia

`no_std` sun and moon position/phase calculations in Rust.

MSRV 1.86 (edition 2024; the library itself builds on 1.85, the edition's
floor, but `criterion` as a dev-dependency needs 1.86 to run tests/benches).

The Rust ecosystem has solid solar-position crates (`solar-positioning`,
`sunrise`), but nothing comparable for the Moon that is `no_std` and
permissively licensed (`pracstro` is `std`-only; `siderust` is
full-precision but AGPL-3.0). `celestia` fills that gap using the
truncated-series strategy `cosinekitty/astronomy` and `pracstro` both use:
keep only the VSOP87/ELP2000-82B terms needed to stay within a stated
accuracy budget, rather than shipping full-precision series or JPL binary
ephemeris kernels.

## Status

Early scaffolding: angle normalization, Julian date conversion, and a
truncated trigonometric series evaluator. Not yet a usable ephemeris.

## Features

```toml
celestia = "0.1"                                                     # std
celestia = { version = "0.1", default-features = false, features = ["libm"] } # no_std
```

Exactly one of `std`/`libm` must be enabled; `std` is the default.

## Correctness

A series term is $s = \sum_{i=1}^{n} a_i \cos(x_i)$, evaluated in `f64`.
Two tools check two disjoint classes of property, since neither covers
both:

**[Gappa](https://gappa.gitlabpages.inria.fr/gappa/)** (`gappa/`) proves a
rounding-error bound on the series arithmetic itself, for a representative
6-term series. With $e = s_{f64} - s_{exact}$:

| cosine implementation         | proved bound on $\lvert e \rvert$ |
|--------------------------------|------------------------------------|
| correctly rounded (<=0.5 ulp)  | $2^{-53.98}$ rad                  |
| typical libm (<=1 ulp)         | $2^{-53.67}$ rad                  |

Both sit ~13 orders of magnitude below the ~1 arcminute truncation error a
series this short carries by construction -- rounding is not the accuracy
bottleneck here, term count is. See `gappa/README.md` for the full proof
writeup.

**[Kani](https://github.com/model-checking/kani)** (`#[cfg(kani)] mod
proofs` per source file, `cargo kani`) exhaustively proves the invariants
Gappa doesn't reach:

- `angle::normalize` always returns a value in $[0, 2\pi)$ for finite
  input, and never panics.
- `time::JulianDate::from_gregorian` never overflows its `i64` arithmetic,
  for any `year: i32`/`month: u32`/`day: u32`.
- `series::evaluate_cos`/`evaluate_sin` never panic, and stay finite for
  bounded amplitudes.

Kani over-approximates `sin`/`cos`/`sqrt` (any value in $[-1, 1]$
regardless of input), so it can't check accuracy -- that split with Gappa
is deliberate. CI runs `cargo kani` on every push/PR.

## Benchmarks

```
cargo bench --features std
```

`benches/series.rs` compares the series evaluator against full
moon-position calls in `pracstro` and `astro` (not apples-to-apples --
`evaluate_cos` is one 6-term fold, the primitive a full position is built
from):

| call                                        | time     |
|----------------------------------------------|----------|
| `celestia::series::evaluate_cos` (6 terms)   | 14.75 ns |
| `pracstro::moon::MOON.phaseage`              | 267 ns   |
| `astro::lunar::geocent_ecl_pos`              | 788 ns   |

## Nix

```
nix build .                 # library
nix flake check             # tests, clippy, fmt, docs, MSRV, no_std cross-compile
nix develop                 # dev shell with gappa, rust-analyzer, etc.
```

`no-std-verify` cross-compiles for `thumbv7em-none-eabi` (a real Cortex-M4
target) to catch accidental `std` leakage.

## License

Licensed under either of [MIT](LICENSE-MIT) or
[Apache-2.0](LICENSE-APACHE), at your option.
