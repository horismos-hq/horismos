//! PRP-009 P1: the motion-command shield's per-decision cost, criterion
//! form (means and confidence intervals; `shield_percentiles.rs` reports the
//! per-decision p50/p99/max the physical-AI question actually asks).
//!
//! Groups:
//! - `shield_decision/<shape>/<case>`: one steady-state decision —
//!   assert → fixed point → read `:verdict` — for the three verdict paths.
//!   `replace_standing` updates the one standing `:cmd` (the world size is
//!   constant; every tick pays truth maintenance for its own verdict).
//!   `fresh_subject` asserts a new anonymous command into a cloned world
//!   (the clone is set-up, outside the timed section). `cold_world` is the
//!   very first transaction of a brand-new `World`, the shape PRP-002's
//!   `triage_transaction` measures — for comparison, not the steady state.
//! - `shield_explain`: reading the recorded derivation (`explain`) and the
//!   goal-directed search (`prove`) for the standing verdict, timed apart
//!   from the decision as PRP-009 asks.
//! - `shield_world_size/<N>`: the next `replace_standing` decision with N
//!   further standing commands in the world, each with its own derived
//!   verdict that truth maintenance resets and re-derives every tick. Sizes
//!   the iteration cap refuses (100 firings per transaction, so 100 or more
//!   standing verdicts) are reported on stderr and skipped, not hidden.
//!
//! Numbers, hardware and commands: `crates/horismos-core/benches/README.md`.
//! These are measurements; the published latency claim is PRP-009 decision 2.

mod shield_common;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use shield_common::{
    Case, WORLD_SIZES, decide_fresh, decide_standing, ontology, standing_world, try_standing_world,
};

fn shield_decision(c: &mut Criterion) {
    let onto = ontology();
    let mut group = c.benchmark_group("shield_decision");
    for case in Case::ALL {
        group.bench_with_input(
            BenchmarkId::new("replace_standing", case.name()),
            &case,
            |b, &case| {
                let (mut world, t, _) = standing_world(&onto, case, 0);
                b.iter(|| decide_standing(&mut world, &t, case));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("fresh_subject", case.name()),
            &case,
            |b, &case| {
                let (world, t, _) = standing_world(&onto, case, 0);
                b.iter_batched(
                    || world.clone(),
                    |mut world| decide_fresh(&mut world, &t, case),
                    BatchSize::SmallInput,
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("cold_world", case.name()),
            &case,
            |b, &case| {
                b.iter_batched(
                    || {
                        let mut world = horismos_core::World::new(&onto);
                        let t = shield_common::Terms::intern(&mut world);
                        (world, t)
                    },
                    |(mut world, t)| decide_standing(&mut world, &t, case),
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn shield_explain(c: &mut Criterion) {
    let onto = ontology();
    let (world, t, subject) = standing_world(&onto, Case::Attenuated, 0);
    let mut group = c.benchmark_group("shield_explain");
    group.bench_function("explain", |b| {
        b.iter(|| {
            world
                .explain(subject, t.verdict)
                .map(|proof| proof.children.len())
        })
    });
    let goal = Case::Attenuated.expected(&t);
    group.bench_function("prove", |b| {
        b.iter(|| {
            world
                .prove(subject, t.verdict, &goal)
                .map(|proof| proof.children.len())
        })
    });
    group.finish();
}

fn shield_world_size(c: &mut Criterion) {
    let onto = ontology();
    let mut group = c.benchmark_group("shield_world_size");
    for padding in WORLD_SIZES {
        let (mut world, t, _) = match try_standing_world(&onto, Case::Attenuated, padding) {
            Ok(fixture) => fixture,
            Err(error) => {
                // Past the iteration cap the world refuses every transaction
                // (see `shield_common::WORLD_SIZES`); that is the result.
                eprintln!("shield_world_size/{padding}: refused — {error}");
                continue;
            }
        };
        group.bench_with_input(BenchmarkId::from_parameter(padding), &padding, |b, _| {
            b.iter(|| decide_standing(&mut world, &t, Case::Attenuated));
        });
    }
    group.finish();
}

criterion_group!(benches, shield_decision, shield_explain, shield_world_size);
criterion_main!(benches);
