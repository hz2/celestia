//! Benchmarks `celestia::series::evaluate_cos` (a single truncated-series
//! coordinate evaluation) against two existing crates' full moon-position
//! calls, following the single-call-latency + throughput pattern used by
//! `solar-positioning`'s own `performance_scenarios.rs` benchmark.
//!
//! This is not an apples-to-apples comparison of equivalent work --
//! `pracstro`/`astro` compute a full moon position (multiple summed
//! series plus coordinate conversion), while `evaluate_cos` is one
//! 6-term series fold, the building block such a position is made of.
//! The intent is to establish a per-term-evaluation cost baseline this
//! crate's own future full-ephemeris functions can be checked against,
//! not to claim `celestia` is faster at moon positions it doesn't yet
//! compute.

use astro::lunar;
use celestia::series::{GAPPA_PROOF_AMPLITUDES, Term, evaluate_cos};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use pracstro::{moon::MOON, time::Date};
use std::hint::black_box;

const ARGUMENTS: [f64; 6] = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6];

fn terms_at(scale: f64) -> [Term; 6] {
    core::array::from_fn(|i| Term::new(GAPPA_PROOF_AMPLITUDES[i], ARGUMENTS[i] * scale))
}

const J2000: f64 = 2451545.0;

fn bench_single_call(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_call");
    let terms = terms_at(1.0);

    group.bench_function("celestia/evaluate_cos_6_terms", |b| {
        b.iter(|| evaluate_cos(black_box(&terms)))
    });

    group.bench_function("pracstro/moon_phaseage", |b| {
        let date = Date::from_julian(black_box(J2000));
        b.iter(|| MOON.phaseage(date))
    });

    group.bench_function("astro/lunar_geocent_ecl_pos", |b| {
        b.iter(|| lunar::geocent_ecl_pos(black_box(J2000)))
    });

    group.finish();
}

fn bench_time_series(c: &mut Criterion) {
    let mut group = c.benchmark_group("time_series");
    group.sample_size(20);

    for count in [10_000_u32, 100_000] {
        group.throughput(Throughput::Elements(u64::from(count)));

        let jds: Vec<f64> = (0..count).map(|i| J2000 + f64::from(i) * 0.5).collect();

        group.bench_with_input(
            BenchmarkId::new("celestia/evaluate_cos_6_terms", count),
            &jds,
            |b, jds| {
                b.iter(|| {
                    for &jd in jds {
                        black_box(evaluate_cos(&terms_at(jd * 0.01)));
                    }
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("pracstro/moon_phaseage", count),
            &jds,
            |b, jds| {
                b.iter(|| {
                    for &jd in jds {
                        black_box(MOON.phaseage(Date::from_julian(jd)));
                    }
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("astro/lunar_geocent_ecl_pos", count),
            &jds,
            |b, jds| {
                b.iter(|| {
                    for &jd in jds {
                        black_box(lunar::geocent_ecl_pos(jd));
                    }
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_single_call, bench_time_series);
criterion_main!(benches);
