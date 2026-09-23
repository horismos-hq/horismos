//! PRP-009 P1: per-decision latency percentiles for the motion-command
//! shield — the tail a control loop cares about, which criterion's
//! mean-of-batches reporting does not expose.
//!
//! This is a `harness = false` bench target with its own `main`, so
//! `cargo bench` builds it under the bench profile (release optimisation)
//! and runs it alongside the criterion benches. Every decision is timed on
//! its own with `Instant`, and each row reports p50 / p99 / max over the
//! sample count printed beside it, after an untimed warm-up.
//!
//! ```text
//! cargo bench -p horismos-core --bench shield_percentiles -- [--samples N] [--sweep-samples N] [--warmup N]
//! ```
//!
//! Rows:
//! - `replace_standing/<case>`: one steady-state decision on the standing
//!   `:cmd` — assert → fixed point → read `:verdict` — per verdict path.
//! - `fresh_subject/<case>`: the same decision as a new anonymous command
//!   into a world cloned outside the timed section.
//! - `explain`: the recorded derivation of the standing verdict, apart.
//! - `world_size/<N>`: the next `replace_standing` decision with N further
//!   standing commands, each carrying a derivation truth maintenance resets
//!   and re-derives on every transaction. A size the iteration cap refuses
//!   (100 firings per transaction) is printed as a refused row with the
//!   gate's own error text.
//!
//! Output is a markdown table on stdout; `README.md` beside this file holds
//! the recorded runs. Measurement only — the published claim is PRP-009
//! decision 2.

mod shield_common;

use std::time::{Duration, Instant};

use shield_common::{
    Case, WORLD_SIZES, decide_fresh, decide_standing, ontology, standing_world, try_standing_world,
};

struct Config {
    samples: usize,
    sweep_samples: usize,
    warmup: usize,
}

impl Config {
    fn from_args() -> Config {
        let mut config = Config {
            samples: 20_000,
            sweep_samples: 2_000,
            warmup: 1_000,
        };
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            let mut take = |field: &mut usize| {
                let value = args.next().and_then(|v| v.parse().ok());
                match value {
                    Some(n) => *field = n,
                    None => eprintln!("shield_percentiles: {arg} needs a positive integer"),
                }
            };
            match arg.as_str() {
                "--samples" => take(&mut config.samples),
                "--sweep-samples" => take(&mut config.sweep_samples),
                "--warmup" => take(&mut config.warmup),
                // `cargo bench` forwards criterion's flags (`--bench`,
                // `--save-baseline …`) to every target; they mean nothing here.
                other => eprintln!("shield_percentiles: ignoring argument {other}"),
            }
        }
        config
    }
}

struct Row {
    name: String,
    samples: usize,
    p50: Duration,
    p99: Duration,
    max: Duration,
}

/// Times `f` once per sample after `warmup` untimed calls. `prepare` runs
/// before each call, outside the timed section (a clone, or nothing).
fn measure<S>(
    name: &str,
    warmup: usize,
    samples: usize,
    mut prepare: impl FnMut() -> S,
    mut f: impl FnMut(S),
) -> Row {
    for _ in 0..warmup {
        f(prepare());
    }
    let mut times = Vec::with_capacity(samples);
    for _ in 0..samples {
        let input = prepare();
        let start = Instant::now();
        f(input);
        times.push(start.elapsed());
    }
    times.sort_unstable();
    let at = |q: f64| times[((times.len() - 1) as f64 * q).round() as usize];
    Row {
        name: name.to_owned(),
        samples,
        p50: at(0.50),
        p99: at(0.99),
        max: times[times.len() - 1],
    }
}

fn fmt(d: Duration) -> String {
    let us = d.as_secs_f64() * 1e6;
    if us >= 1000.0 {
        format!("{:.3} ms", us / 1000.0)
    } else {
        format!("{us:.2} µs")
    }
}

fn main() {
    let config = Config::from_args();
    let onto = ontology();
    let mut rows = Vec::new();

    for case in Case::ALL {
        let (mut world, t, _) = standing_world(&onto, case, 0);
        rows.push(measure(
            &format!("replace_standing/{}", case.name()),
            config.warmup,
            config.samples,
            || (),
            |()| {
                let verdict = decide_standing(&mut world, &t, case);
                assert_eq!(verdict.as_ref(), Some(&case.expected(&t)));
            },
        ));
    }

    for case in Case::ALL {
        let (world, t, _) = standing_world(&onto, case, 0);
        rows.push(measure(
            &format!("fresh_subject/{}", case.name()),
            config.warmup,
            config.samples,
            || world.clone(),
            |mut world| {
                let verdict = decide_fresh(&mut world, &t, case);
                assert_eq!(verdict.as_ref(), Some(&case.expected(&t)));
            },
        ));
    }

    {
        let (world, t, subject) = standing_world(&onto, Case::Attenuated, 0);
        rows.push(measure(
            "explain/attenuated_shared_cell",
            config.warmup,
            config.samples,
            || (),
            |()| {
                let proof = world.explain(subject, t.verdict);
                assert!(proof.is_some_and(|p| !p.children.is_empty()));
            },
        ));
    }

    let mut refused = Vec::new();
    for padding in WORLD_SIZES {
        let (mut world, t, _) = match try_standing_world(&onto, Case::Attenuated, padding) {
            Ok(fixture) => fixture,
            Err(error) => {
                refused.push((padding, error));
                continue;
            }
        };
        rows.push(measure(
            &format!("world_size/{padding}"),
            config.warmup.min(config.sweep_samples),
            config.sweep_samples,
            || (),
            |()| {
                let verdict = decide_standing(&mut world, &t, Case::Attenuated);
                assert_eq!(verdict.as_ref(), Some(&Case::Attenuated.expected(&t)));
            },
        ));
    }

    println!("| case | samples | p50 | p99 | max |");
    println!("|---|---:|---:|---:|---:|");
    for row in &rows {
        println!(
            "| `{}` | {} | {} | {} | {} |",
            row.name,
            row.samples,
            fmt(row.p50),
            fmt(row.p99),
            fmt(row.max)
        );
    }
    for (padding, _) in &refused {
        println!("| `world_size/{padding}` | — | refused | refused | refused |");
    }
    for (padding, error) in &refused {
        println!();
        println!("`world_size/{padding}`: {error}");
    }
}
