//! Deferred step kinds (U12): `ChoiceStep`/`ProposalStep`/`WaitStep`/
//! `SubflowStep` parse (`workflow::model`, U2 — already done, including
//! `SubflowStep`'s D6 depth-1 check) but do not execute in this PRP (01-
//! objective.md's Non-goals; D3 for `WaitStep`, D6 for `SubflowStep`).
//! `ProposalStep`'s candidate ranking is deferred the same way — trace/2
//! gives it a real emission builder (`trace::emit::proposal_step`, U9) for
//! when ranking does land, but nothing in this PRP calls `Bridge::rank`.
//!
//! Each function here returns the documented `WorkflowError::StepDeferred`,
//! naming the concrete step and its kind in domain terms — never a panic,
//! never a silent no-op (U12's own test target). `workflow::agent` (U7,
//! landing on a sibling branch) calls one of these at the point its step
//! dispatch reaches the matching `StepBody` variant; until then these are
//! free functions with no caller in this crate, exercised directly by the
//! tests below.
//!
//! Spec: `PRPs/PRP-004-agent-loop/02-tasks-and-gaps.md` U12 · Owner: PRP-004

use crate::World;
use crate::ontology::term::Iri;
use crate::workflow::WorkflowError;

fn deferred(world: &World, step: Iri, kind: &str) -> WorkflowError {
    WorkflowError::StepDeferred {
        step: world.ontology().compact(world.resolve(step)),
        kind: kind.to_owned(),
    }
}

/// A `horis:ChoiceStep` is not yet executed — see module doc.
pub fn choice_deferred(world: &World, step: Iri) -> WorkflowError {
    deferred(world, step, "ChoiceStep")
}

/// A `horis:ProposalStep` is not yet executed — see module doc.
pub fn proposal_deferred(world: &World, step: Iri) -> WorkflowError {
    deferred(world, step, "ProposalStep")
}

/// A `horis:WaitStep` is not yet executed (D3) — see module doc.
pub fn wait_deferred(world: &World, step: Iri) -> WorkflowError {
    deferred(world, step, "WaitStep")
}

/// A `horis:SubflowStep` is not yet executed — see module doc.
pub fn subflow_deferred(world: &World, step: Iri) -> WorkflowError {
    deferred(world, step, "SubflowStep")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ontology;
    use crate::workflow::model::StepBody;

    const PRELUDE: &str = "@prefix : <https://example.org/t#> .\n\
         @prefix horis: <https://w3id.org/horismos#> .\n\
         @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
         :flag a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean .\n\
         :Patient a rdfs:Class .\n";

    fn load(source: &str) -> Ontology {
        let full = format!("{PRELUDE}{source}");
        Ontology::load_from_str_allow_incomplete(&full, |_| None)
            .expect("a valid deferred-step fixture loads")
    }

    fn iri(onto: &Ontology, local: &str) -> Iri {
        onto.iri(&format!("https://example.org/t#{local}"))
            .unwrap_or_else(|| panic!("{local} must be named by the fixture"))
    }

    /// U12 test target: a workflow using `ChoiceStep` loads cleanly (U2's
    /// parsing already covers this) and the deferred stub names it in
    /// domain terms, not a panic, not a silent no-op.
    #[test]
    fn choice_step_parses_and_defers_by_domain_term_error() {
        let onto = load(
            ":Wf a horis:Workflow ; horis:startsAt :A .\n\
             :A a horis:ChoiceStep ;\n\
             \x20  horis:branches ( [ horis:when [ horis:property :flag ; horis:isAssigned true ] ; horis:next :B ] ) ;\n\
             \x20  horis:otherwise :B .\n\
             :B a horis:Step ; horis:terminal true .\n",
        );
        let step = iri(&onto, "A");
        let wf = onto.workflow(iri(&onto, "Wf")).expect("parses");
        assert!(matches!(
            wf.step(step).expect("declared step").body,
            StepBody::Choice(_)
        ));

        let world = World::new(&onto);
        let err = choice_deferred(&world, step);
        assert_eq!(
            err.domain_text(),
            "step :A (ChoiceStep) is not yet executable — deferred past v1"
        );
    }

    #[test]
    fn proposal_step_parses_and_defers_by_domain_term_error() {
        let onto = load(
            ":Wf a horis:Workflow ; horis:startsAt :A .\n\
             :A a horis:ProposalStep ;\n\
             \x20  horis:candidates ( :OptionA :OptionB ) ;\n\
             \x20  horis:confidence 0.7 ;\n\
             \x20  horis:fallback :B ;\n\
             \x20  horis:next :B .\n\
             :B a horis:Step ; horis:terminal true .\n",
        );
        let step = iri(&onto, "A");
        let wf = onto.workflow(iri(&onto, "Wf")).expect("parses");
        assert!(matches!(
            wf.step(step).expect("declared step").body,
            StepBody::Proposal(_)
        ));

        let world = World::new(&onto);
        let err = proposal_deferred(&world, step);
        assert_eq!(
            err.domain_text(),
            "step :A (ProposalStep) is not yet executable — deferred past v1"
        );
    }

    #[test]
    fn wait_step_parses_and_defers_by_domain_term_error() {
        let onto = load(":Wf a horis:Workflow ; horis:startsAt :A .\n:A a horis:WaitStep .\n");
        let step = iri(&onto, "A");
        let wf = onto.workflow(iri(&onto, "Wf")).expect("parses");
        assert!(matches!(
            wf.step(step).expect("declared step").body,
            StepBody::Wait(_)
        ));

        let world = World::new(&onto);
        let err = wait_deferred(&world, step);
        assert_eq!(
            err.domain_text(),
            "step :A (WaitStep) is not yet executable — deferred past v1"
        );
    }

    /// Also exercises D6's depth-1 subflow check via `workflow::model`
    /// (U2) — the fixture is deliberately two workflows, the target
    /// nesting no `SubflowStep` of its own.
    #[test]
    fn subflow_step_parses_and_defers_by_domain_term_error() {
        let onto = load(
            ":Sub a horis:Workflow ; horis:startsAt :SubStart .\n\
             :SubStart a horis:Step ; horis:terminal true .\n\
             :Main a horis:Workflow ; horis:startsAt :A .\n\
             :A a horis:SubflowStep ; horis:workflow :Sub ; horis:next :B .\n\
             :B a horis:Step ; horis:terminal true .\n",
        );
        let step = iri(&onto, "A");
        let wf = onto.workflow(iri(&onto, "Main")).expect("parses");
        assert!(matches!(
            wf.step(step).expect("declared step").body,
            StepBody::Subflow(_)
        ));

        let world = World::new(&onto);
        let err = subflow_deferred(&world, step);
        assert_eq!(
            err.domain_text(),
            "step :A (SubflowStep) is not yet executable — deferred past v1"
        );
    }
}
