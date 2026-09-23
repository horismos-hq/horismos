//! The shield fixture shared by `shield.rs` (criterion) and
//! `shield_percentiles.rs` (per-decision tail): the motion-command overlay
//! `examples/gallery/safety-envelope.ttl`, loaded fail-closed exactly as a
//! deployment loads it, and the three verdict paths the Physical AI page
//! walks — an open-floor `:Permitted`, a shared-cell `:Attenuated`, and the
//! no-reading `:Blocked` that PRP-005's completeness check forced.
//!
//! One *decision* is one transaction (doc 03): assert the command, let the
//! gate and forward chaining run to the fixed point inside that same call,
//! then read `:verdict`. `explain` is a separate read of the recorded
//! derivation and is timed apart, never inside a decision.
//!
//! Every decision after the first pays truth maintenance (doc 03 § Truth
//! maintenance): the transaction resets *all* standing derivations and
//! re-chains to the fixed point of the resulting base facts, so a world
//! holding N standing commands re-derives N verdicts per tick. The
//! world-size sweep measures exactly that.

#![allow(dead_code)]

use horismos_core::error::HorismosError;
use horismos_core::{Iri, Ontology, Ref, Value, World};
use rust_decimal::Decimal;

pub const NS: &str = "https://example.org/shield#";

pub fn ontology() -> Ontology {
    Ontology::load(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/gallery/safety-envelope.ttl"
    ))
    .expect("safety-envelope.ttl loads and verifies")
}

/// The interned vocabulary a decision needs.
#[derive(Clone, Copy)]
pub struct Terms {
    pub motion_command: Iri,
    pub requested_speed: Iri,
    pub nearest_human_distance: Iri,
    pub e_stop_engaged: Iri,
    pub zone: Iri,
    pub verdict: Iri,
    pub open_floor: Iri,
    pub shared_cell: Iri,
    pub permitted: Iri,
    pub attenuated: Iri,
    pub blocked: Iri,
    /// The standing command a steady-state tick updates.
    pub cmd: Iri,
}

impl Terms {
    pub fn intern(world: &mut World) -> Terms {
        let mut t = |local: &str| world.intern(&format!("{NS}{local}"));
        Terms {
            motion_command: t("MotionCommand"),
            requested_speed: t("requestedSpeed"),
            nearest_human_distance: t("nearestHumanDistance"),
            e_stop_engaged: t("eStopEngaged"),
            zone: t("zone"),
            verdict: t("verdict"),
            open_floor: t("OpenFloor"),
            shared_cell: t("SharedCell"),
            permitted: t("Permitted"),
            attenuated: t("Attenuated"),
            blocked: t("Blocked"),
            cmd: t("cmd"),
        }
    }
}

/// The three verdict paths, as the Physical AI page exercises them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Case {
    /// Open floor, 2.4 m clearance, 1.8 m/s → `:Permitted` via `permitOpenFloor`.
    Permitted,
    /// Shared cell, 0.8 m, 1.2 m/s → `:Attenuated` via `attenuateInSharedCell`.
    Attenuated,
    /// Open floor, 1.0 m/s, *no* proximity reading → `:Blocked` via
    /// `noProximityReadingBlocks` (closed world: the missing measurement).
    BlockedNoReading,
}

impl Case {
    pub const ALL: [Case; 3] = [Case::Permitted, Case::Attenuated, Case::BlockedNoReading];

    pub fn name(self) -> &'static str {
        match self {
            Case::Permitted => "permitted_open_floor",
            Case::Attenuated => "attenuated_shared_cell",
            Case::BlockedNoReading => "blocked_no_reading",
        }
    }

    /// The command's state for this case. A tenth of a metre / metre per
    /// second is exact in `Decimal`, so the values are the same lexical
    /// forms the docs page asserts.
    pub fn state(self, t: &Terms) -> Vec<(Iri, Value)> {
        let mut props = vec![(t.e_stop_engaged, Value::Boolean(false))];
        match self {
            Case::Permitted => {
                props.push((t.zone, Value::Ref(Ref::new(t.open_floor))));
                props.push((t.requested_speed, Value::Decimal(Decimal::new(18, 1))));
                props.push((
                    t.nearest_human_distance,
                    Value::Decimal(Decimal::new(24, 1)),
                ));
            }
            Case::Attenuated => {
                props.push((t.zone, Value::Ref(Ref::new(t.shared_cell))));
                props.push((t.requested_speed, Value::Decimal(Decimal::new(12, 1))));
                props.push((t.nearest_human_distance, Value::Decimal(Decimal::new(8, 1))));
            }
            Case::BlockedNoReading => {
                props.push((t.zone, Value::Ref(Ref::new(t.open_floor))));
                props.push((t.requested_speed, Value::Decimal(Decimal::new(10, 1))));
            }
        }
        props
    }

    pub fn expected(self, t: &Terms) -> Value {
        Value::Ref(Ref::new(match self {
            Case::Permitted => t.permitted,
            Case::Attenuated => t.attenuated,
            Case::BlockedNoReading => t.blocked,
        }))
    }
}

/// One decision on the standing command: assert (the gate and chaining to
/// the fixed point run inside), then read `:verdict`. Returns the verdict so
/// the caller can keep the read observable.
pub fn decide_standing(world: &mut World, t: &Terms, case: Case) -> Option<Value> {
    let subject = world
        .assert_named_instance(t.cmd, t.motion_command, case.state(t))
        .expect("the command passes the gate");
    world.value(subject, t.verdict).cloned()
}

/// A clone of `world` whose interner is already its own. `World::clone` is
/// a refcount bump that leaves the two worlds sharing one `Arc<Interner>`,
/// and the first `intern` on the clone (which `assert_instance` performs
/// for its skolem) would deep-copy every ontology IRI through
/// `Arc::make_mut`. A real loop never clones per tick, so the harnesses
/// force that copy here, in set-up, by interning a throwaway IRI — the
/// timed decision then pays only for the decision.
pub fn detached_clone(world: &World) -> World {
    let mut clone = world.clone();
    clone.intern(&format!("{NS}detached"));
    clone
}

/// One decision on a fresh, anonymous command (the alternative per-tick
/// shape PRP-009 decision 7 weighs): the world grows by one subject.
pub fn decide_fresh(world: &mut World, t: &Terms, case: Case) -> Option<Value> {
    let subject = world
        .assert_instance(t.motion_command, case.state(t))
        .expect("the command passes the gate");
    world.value(subject, t.verdict).cloned()
}

/// The world-size axis: `padding` further standing commands beside `:cmd`.
/// Every transaction resets and re-derives *all* standing derivations, and
/// `reason::forward::ITERATION_CAP` (100) counts rule firings per
/// transaction and admits exactly 100, so a world holding more than 100
/// standing verdicts in all (more than 99 padding verdicts beside `:cmd`)
/// refuses every transaction with `IterationCapExceeded` — the sweep records
/// that ceiling as a result rather than avoiding it.
pub const WORLD_SIZES: [usize; 6] = [1, 10, 50, 99, 100, 1000];

/// A world at its fixed point with the standing command `:cmd` already
/// decided for `case`, plus `padding` further standing commands (each
/// holding a `:Permitted` derivation of its own). `Err` is the gate's own
/// refusal of the world — at and past the iteration cap, see
/// [`WORLD_SIZES`].
pub fn try_standing_world(
    onto: &Ontology,
    case: Case,
    padding: usize,
) -> Result<(World, Terms, Ref), HorismosError> {
    let mut world = World::new(onto);
    let t = Terms::intern(&mut world);
    for i in 0..padding {
        let iri = world.intern(&format!("{NS}standing-{i}"));
        world.assert_named_instance(iri, t.motion_command, Case::Permitted.state(&t))?;
    }
    let subject = world.assert_named_instance(t.cmd, t.motion_command, case.state(&t))?;
    assert_eq!(
        world.value(subject, t.verdict),
        Some(&case.expected(&t)),
        "{}: the fixture must settle on its documented verdict",
        case.name()
    );
    Ok((world, t, subject))
}

/// [`try_standing_world`] for sizes the cap admits.
pub fn standing_world(onto: &Ontology, case: Case, padding: usize) -> (World, Terms, Ref) {
    try_standing_world(onto, case, padding).expect("the world fits under the iteration cap")
}
