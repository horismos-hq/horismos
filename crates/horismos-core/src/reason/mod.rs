//! The Reasoner: deterministic inference over the World.
//!
//! Subsumption (always on), forward chaining to fixed point (Rete-lite),
//! backward chaining (`prove`), proof trees with provenance down to perception
//! confidence (`explain`). No wall-clock, no randomness, no iteration-order
//! dependence — same inputs, same conclusions, always.
//!
//! Spec: `docs/framework/04-reasoning.md` · Owner: PRP-002

pub mod backward;
pub mod eval;
pub mod forward;
pub mod matcher;
pub mod proof;
pub mod rule;

pub use forward::{Firing, FiringOutcome, ReasoningRecord};
pub use proof::{ConditionVia, ProofKind, ProofNode};
pub use rule::{Conclusion, Condition, ConditionTree, Op, Rule, Term};

/// Shared test fixture: the PRP's worked Example 1 — triage.ttl's domain
/// plus the illustrative `:r2`/`:r3`/`:needsCardiacMonitor` declared inline
/// (they are teaching material, not fixture content — the PRP marks them
/// "illustrative only").
#[cfg(test)]
pub(crate) mod testing {
    use crate::{Ontology, World};

    pub(crate) const PRELUDE: &str = "@prefix : <https://example.org/t#> .\n\
         @prefix horis: <https://w3id.org/horismos#> .\n\
         @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n";

    pub(crate) const EXAMPLE_1: &str = ":Patient a rdfs:Class .\n\
         :Symptom a rdfs:Class .\n\
         :ChestPain rdfs:subClassOf :Symptom .\n\
         :CrushingChestPain rdfs:subClassOf :ChestPain .\n\
         :AcuityLevel a rdfs:Class ; horis:oneOf ( :Immediate :Urgent :Standard ) .\n\
         :heartRate a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:decimal ;\n\
         \x20   horis:unit \"bpm\" ; horis:min 0 ; horis:max 300 ; horis:cardinality \"0..1\" .\n\
         :hasSymptom a rdf:Property ; rdfs:domain :Patient ; rdfs:range :Symptom ;\n\
         \x20   horis:cardinality \"0..*\" .\n\
         :acuity a rdf:Property ; rdfs:domain :Patient ; rdfs:range :AcuityLevel ;\n\
         \x20   horis:cardinality \"0..1\" ; horis:assignedBy horis:RulesOnly .\n\
         :needsCardiacMonitor a rdf:Property ; rdfs:domain :Patient ;\n\
         \x20   rdfs:range xsd:boolean ; horis:cardinality \"0..1\" ;\n\
         \x20   horis:assignedBy horis:RulesOnly .\n\
         :r1 a horis:Rule ; horis:priority 10 ;\n\
         \x20 horis:when [ horis:all (\n\
         \x20   [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 120 ]\n\
         \x20   [ horis:subject :Patient ; horis:property :hasSymptom ; horis:is :ChestPain ]\n\
         \x20 ) ] ;\n\
         \x20 horis:then [ horis:property :acuity ; horis:value :Immediate ] ;\n\
         \x20 horis:explain \"Tachycardia with chest pain — immediate track\" .\n\
         :r2 a horis:Rule ; horis:priority 5 ;\n\
         \x20 horis:when [ horis:all (\n\
         \x20   [ horis:subject :Patient ; horis:property :hasSymptom ; horis:is :ChestPain ]\n\
         \x20 ) ] ;\n\
         \x20 horis:then [ horis:property :acuity ; horis:value :Urgent ] ;\n\
         \x20 horis:explain \"Chest pain — at least urgent\" .\n\
         :r3 a horis:Rule ; horis:priority 8 ;\n\
         \x20 horis:when [ horis:all (\n\
         \x20   [ horis:subject :Patient ; horis:property :acuity ; horis:is :Immediate ]\n\
         \x20 ) ] ;\n\
         \x20 horis:then [ horis:property :needsCardiacMonitor ; horis:value true ] ;\n\
         \x20 horis:explain \"Immediate track — continuous cardiac monitoring\" .\n";

    pub(crate) fn example_world() -> World {
        let onto =
            Ontology::load_from_str_allow_incomplete(&format!("{PRELUDE}{EXAMPLE_1}"), |_| None)
                .expect("the Example 1 ontology must load");
        World::new(&onto)
    }
}
