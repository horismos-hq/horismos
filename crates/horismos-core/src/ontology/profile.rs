//! RDFS-plus profile validation (U3): accepts exactly the vocabulary
//! documented in `docs/framework/02-ontology.md` (RDFS base + the XSD
//! subset + `owl:sameAs` + the `horis:` vocabulary) and rejects everything
//! else with a domain-term [`OntologyError`].
//!
//! `validate` runs over one parsed [`Document`] (U2); `validate_documents`
//! runs the same pass over several, unnamed; `validate_named` (U5) does the
//! same over several *named* graphs, attributing each violation to the graph
//! it came from — what `ontology::imports::OntologySet::validate` calls.
//! Every violation in the document is collected — the first failing term
//! does not short-circuit the pass, so an authoring loop sees the full list
//! in one round trip (CLAUDE.md: errors speak domain terms; nothing here is
//! fail-fast).
//!
//! Spec: `docs/framework/02-ontology.md`, `docs/framework/05-workflows.md` ·
//! Owner: PRP-001

use crate::error::OntologyError;
use crate::ontology::term::Interner;
use crate::ontology::turtle::{Document, Literal, Node, Object, Triple};

// -- Namespaces (shared with `schema.rs`) --------------------------------

pub(crate) const RDF_NS: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
pub(crate) const RDFS_NS: &str = "http://www.w3.org/2000/01/rdf-schema#";
pub(crate) const OWL_NS: &str = "http://www.w3.org/2002/07/owl#";
pub(crate) const XSD_NS: &str = "http://www.w3.org/2001/XMLSchema#";
pub(crate) const HORIS_NS: &str = "https://w3id.org/horismos#";

pub(crate) const HORIS_CARDINALITY: &str = "https://w3id.org/horismos#cardinality";
pub(crate) const HORIS_ASSIGNED_BY: &str = "https://w3id.org/horismos#assignedBy";
pub(crate) const HORIS_RULES_ONLY: &str = "https://w3id.org/horismos#RulesOnly";
pub(crate) const HORIS_PERCEPTION_ALLOWED: &str = "https://w3id.org/horismos#PerceptionAllowed";
pub(crate) const HORIS_TOOLS_ONLY: &str = "https://w3id.org/horismos#ToolsOnly";

// -- Allowed local names per reserved namespace --------------------------

const ALLOWED_RDF: &[&str] = &["type", "Property", "first", "rest", "nil"];
const ALLOWED_RDFS: &[&str] = &["Class", "subClassOf", "domain", "range", "label", "comment"];
const ALLOWED_XSD: &[&str] = &[
    "decimal", "integer", "string", "boolean", "dateTime", "duration",
];

/// The exact `horis:` vocabulary the RDFS-plus profile accepts
/// (`docs/framework/02-ontology.md` §Constraint/Rule vocabulary,
/// `docs/framework/05-workflows.md` for the workflow terms, `docs/framework/
/// 07-tools.md` for the `horis:Tool` contract and `ProposalStep` terms — doc
/// 02's own workflow list is an explicit summary of doc 05's fuller
/// vocabulary, and `examples/triage/triage.ttl`'s Workflow and Tool sections
/// exercise every term listed here).
const ALLOWED_HORIS: &[&str] = &[
    // Constraints
    "min",
    "max",
    "unit",
    "cardinality",
    "oneOf",
    "pattern",
    "immutable",
    "assignedBy",
    "RulesOnly",
    "PerceptionAllowed",
    "ToolsOnly",
    // Rules
    "Rule",
    "priority",
    "when",
    "then",
    "explain",
    "all",
    "any",
    "none",
    "exceeds",
    "below",
    "atLeast",
    "atMost",
    "between",
    "is",
    "isNot",
    "isAssigned",
    "isEmpty",
    "contains",
    "inList",
    "subject",
    "property",
    "value",
    // Workflows (doc 05, full vocabulary — triage.ttl's Workflow section is
    // the acceptance fixture)
    "Workflow",
    "startsAt",
    "PerceptionStep",
    "ReasoningStep",
    "ActionStep",
    "NarrationStep",
    "ProposalStep",
    "WaitStep",
    "ChoiceStep",
    "SubflowStep",
    "Step",
    "next",
    "goal",
    "requires",
    "fallback",
    "terminal",
    "onUnreachable",
    "onError",
    "onUnsatisfied",
    "extracts",
    "model",
    "confidence",
    "tool",
    "audience",
    "narrates",
    "branches",
    "otherwise",
    "workflow",
    "decides",
    "required",
    // Tool contracts (doc 07)
    "Tool",
    "takes",
    "arg",
    "name",
    "type",
    "effect",
    "asserts",
    "binding",
    "timeout",
    "idempotent",
    "retries",
    "now",
    // ProposalStep (doc 05/07)
    "candidates",
    "rankBy",
    // Candidate-artifact runtime evidence (PRP-008). PRP-007 owns the
    // authoring-side envelope vocabulary; these terms state only what the
    // deterministic policy/sandbox/authorization boundary established.
    "CandidatePolicyEvidence",
    "SandboxEvidence",
    "AuthorizationEvidence",
    "policyAccepted",
    "sandboxPassed",
    "authorizationGranted",
    "evidenceFor",
    // Imports/versioning
    "imports",
    "version",
    "profileVersion",
];

/// Checks one IRI (already resolved to its full string) against the
/// reserved-namespace allowlists. Returns `None` for anything outside the
/// five reserved namespaces (`rdf:`, `rdfs:`, `owl:`, `xsd:`, `horis:`) —
/// unknown non-`horis:` namespaces are the user's domain (brief: "domain
/// IRIs, sct:, etc. are fine").
fn check_iri_in_profile(iri_str: &str) -> Option<OntologyError> {
    if let Some(local) = iri_str.strip_prefix(RDF_NS) {
        (!ALLOWED_RDF.contains(&local)).then(|| OntologyError::OutOfProfileTerm {
            term: format!("rdf:{local}"),
        })
    } else if let Some(local) = iri_str.strip_prefix(RDFS_NS) {
        (!ALLOWED_RDFS.contains(&local)).then(|| OntologyError::OutOfProfileTerm {
            term: format!("rdfs:{local}"),
        })
    } else if let Some(local) = iri_str.strip_prefix(OWL_NS) {
        (local != "sameAs").then(|| OntologyError::OutOfProfileTerm {
            term: format!("owl:{local}"),
        })
    } else if let Some(local) = iri_str.strip_prefix(XSD_NS) {
        (!ALLOWED_XSD.contains(&local)).then(|| OntologyError::DatatypeNotInProfile {
            datatype: format!("xsd:{local}"),
        })
    } else if let Some(local) = iri_str.strip_prefix(HORIS_NS) {
        (!ALLOWED_HORIS.contains(&local)).then(|| OntologyError::UnknownHorisTerm {
            term: format!("horis:{local}"),
        })
    } else {
        None
    }
}

fn node_display(interner: &Interner, node: Node) -> String {
    match node {
        Node::Named(iri) => interner.resolve(iri).to_owned(),
        Node::Blank(iri) => interner.resolve(iri).to_owned(),
    }
}

fn check_literal(interner: &Interner, lit: &Literal) -> Option<OntologyError> {
    match lit {
        Literal::Typed(_) => None,
        Literal::Other { datatype, .. } => check_iri_in_profile(interner.resolve(*datatype)),
        Literal::LangString { .. } => Some(OntologyError::DatatypeNotInProfile {
            datatype: "rdf:langString".to_owned(),
        }),
    }
}

/// `horis:cardinality`'s object must be exactly one of the four documented
/// strings. Only called when the object already passed the generic literal
/// check (so this never double-reports an out-of-subset datatype).
fn check_cardinality(interner: &Interner, object: &Object) -> Option<OntologyError> {
    use crate::ontology::term::Value;
    let text = match object {
        Object::Literal(Literal::Typed(Value::String(s)))
            if matches!(s.as_str(), "1" | "0..1" | "1..*" | "0..*") =>
        {
            return None;
        }
        Object::Literal(Literal::Typed(v)) => v.lexical_form().unwrap_or_default(),
        Object::Literal(Literal::Other { lexical, .. }) => lexical.clone(),
        Object::Literal(Literal::LangString { lexical, .. }) => lexical.clone(),
        Object::Node(n) => node_display(interner, *n),
    };
    Some(OntologyError::InvalidCardinality { value: text })
}

/// `horis:assignedBy`'s object must be exactly one of the three enum IRIs.
/// Only called when the object already passed the generic term check (so an
/// unknown `horis:` term is reported once, as `UnknownHorisTerm`, not twice).
fn check_assigned_by(interner: &Interner, object: &Object) -> Option<OntologyError> {
    match object {
        Object::Node(Node::Named(iri)) => {
            let s = interner.resolve(*iri);
            if matches!(
                s,
                HORIS_RULES_ONLY | HORIS_PERCEPTION_ALLOWED | HORIS_TOOLS_ONLY
            ) {
                None
            } else {
                Some(OntologyError::InvalidAssignedBy {
                    value: s.to_owned(),
                })
            }
        }
        Object::Node(Node::Blank(iri)) => Some(OntologyError::InvalidAssignedBy {
            value: interner.resolve(*iri).to_owned(),
        }),
        Object::Literal(_) => Some(OntologyError::InvalidAssignedBy {
            value: "a literal".to_owned(),
        }),
    }
}

fn validate_triple(interner: &Interner, triple: &Triple, violations: &mut Vec<OntologyError>) {
    // The subject's term validity doesn't gate anything else below (no
    // structural check keys on it), so it's checked but not tracked.
    if let Node::Named(iri) = triple.subject
        && let Some(v) = check_iri_in_profile(interner.resolve(iri))
    {
        violations.push(v);
    }

    let predicate_str = interner.resolve(triple.predicate);
    let predicate_ok = match check_iri_in_profile(predicate_str) {
        Some(v) => {
            violations.push(v);
            false
        }
        None => true,
    };

    let object_ok = match &triple.object {
        Object::Node(Node::Named(iri)) => match check_iri_in_profile(interner.resolve(*iri)) {
            Some(v) => {
                violations.push(v);
                false
            }
            None => true,
        },
        Object::Node(Node::Blank(_)) => true,
        Object::Literal(lit) => match check_literal(interner, lit) {
            Some(v) => {
                violations.push(v);
                false
            }
            None => true,
        },
    };

    if predicate_ok
        && object_ok
        && predicate_str == HORIS_CARDINALITY
        && let Some(v) = check_cardinality(interner, &triple.object)
    {
        violations.push(v);
    }
    if predicate_ok
        && object_ok
        && predicate_str == HORIS_ASSIGNED_BY
        && let Some(v) = check_assigned_by(interner, &triple.object)
    {
        violations.push(v);
    }
}

/// Validates `document` against the RDFS-plus profile, interning already
/// done by `interner`. Collects every violation in one pass rather than
/// stopping at the first (the authoring loop needs the full list).
pub fn validate(document: &Document, interner: &Interner) -> Result<(), OntologyError> {
    let mut violations = Vec::new();
    for triple in &document.triples {
        validate_triple(interner, triple, &mut violations);
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(OntologyError::ProfileViolations(violations))
    }
}

/// Validates several documents as one pass (U5's named-graph imports will
/// hand one `Document` per graph; violations across all of them are
/// collected into a single [`OntologyError::ProfileViolations`]).
pub fn validate_documents<'a>(
    documents: impl IntoIterator<Item = &'a Document>,
    interner: &Interner,
) -> Result<(), OntologyError> {
    let mut violations = Vec::new();
    for document in documents {
        if let Err(OntologyError::ProfileViolations(v)) = validate(document, interner) {
            violations.extend(v);
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(OntologyError::ProfileViolations(violations))
    }
}

/// Validates several *named* graphs as one pass (U5): each violation is
/// wrapped in [`OntologyError::ProfileViolationInGraph`] naming the graph it
/// came from, so a caller sees exactly which import — or the root — failed.
/// This is what [`crate::ontology::imports::OntologySet::validate`] calls;
/// [`validate_documents`] stays as-is (unnamed) for callers that don't need
/// per-graph attribution.
pub fn validate_named<'a>(
    documents: impl IntoIterator<Item = (&'a str, &'a Document)>,
    interner: &Interner,
) -> Result<(), OntologyError> {
    let mut violations = Vec::new();
    for (graph, document) in documents {
        if let Err(OntologyError::ProfileViolations(v)) = validate(document, interner) {
            violations.extend(
                v.into_iter()
                    .map(|inner| OntologyError::ProfileViolationInGraph {
                        graph: graph.to_owned(),
                        inner: Box::new(inner),
                    }),
            );
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(OntologyError::ProfileViolations(violations))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ontology::turtle::parse;

    fn parse_snippet(source: &str) -> (Document, Interner) {
        let mut interner = Interner::new();
        let full = format!(
            "@prefix : <https://example.org/t#> .\n\
             @prefix horis: <https://w3id.org/horismos#> .\n\
             @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
             @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
             @prefix owl: <http://www.w3.org/2002/07/owl#> .\n\
             @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
             {source}\n"
        );
        let doc = parse(&full, &mut interner).expect("valid Turtle snippet must parse");
        (doc, interner)
    }

    fn violations_of(source: &str) -> Vec<OntologyError> {
        let (doc, interner) = parse_snippet(source);
        match validate(&doc, &interner) {
            Ok(()) => Vec::new(),
            Err(OntologyError::ProfileViolations(v)) => v,
            Err(other) => panic!("expected ProfileViolations, got {other:?}"),
        }
    }

    #[test]
    fn rejects_owl_term_other_than_same_as() {
        let violations = violations_of(":Foo a owl:Restriction .");
        assert_eq!(violations.len(), 1, "{violations:?}");
        let text = violations[0].domain_text();
        assert_eq!(text, "owl:Restriction is outside the RDFS-plus profile");
    }

    #[test]
    fn accepts_owl_same_as() {
        let violations = violations_of(":ChestPain owl:sameAs :SomeExternalConcept .");
        assert!(violations.is_empty(), "{violations:?}");
    }

    #[test]
    fn rejects_unknown_horis_term() {
        let violations = violations_of(":Foo horis:frobnicate 1 .");
        assert_eq!(violations.len(), 1, "{violations:?}");
        let text = violations[0].domain_text();
        assert!(
            text.contains("no such term in the horis: vocabulary"),
            "{text}"
        );
        assert!(text.contains("horis:frobnicate"), "{text}");
    }

    #[test]
    fn rejects_out_of_subset_datatype() {
        let violations =
            violations_of(":s :p \"127.0.0.1\"^^<http://www.w3.org/2001/XMLSchema#anyURI> .");
        assert_eq!(violations.len(), 1, "{violations:?}");
        let text = violations[0].domain_text();
        assert!(text.contains("xsd:anyURI"), "{text}");
    }

    #[test]
    fn rejects_lang_tagged_string() {
        let violations = violations_of(":s :p \"bonjour\"@fr .");
        assert_eq!(violations.len(), 1, "{violations:?}");
        let text = violations[0].domain_text();
        assert!(text.contains("rdf:langString"), "{text}");
    }

    #[test]
    fn rejects_bad_cardinality_string() {
        let violations = violations_of(":hasFoo horis:cardinality \"lots\" .");
        assert_eq!(violations.len(), 1, "{violations:?}");
        let text = violations[0].domain_text();
        assert!(text.contains("lots"), "{text}");
        assert!(text.contains("horis:cardinality"), "{text}");
    }

    #[test]
    fn rejects_non_string_cardinality() {
        let violations = violations_of(":hasFoo horis:cardinality 1 .");
        assert_eq!(violations.len(), 1, "{violations:?}");
    }

    #[test]
    fn accepts_every_documented_cardinality_value() {
        for value in ["1", "0..1", "1..*", "0..*"] {
            let violations = violations_of(&format!(":hasFoo horis:cardinality \"{value}\" ."));
            assert!(violations.is_empty(), "{value}: {violations:?}");
        }
    }

    #[test]
    fn rejects_bad_assigned_by_object() {
        let violations = violations_of(":acuity horis:assignedBy :NotAPolicy .");
        assert_eq!(violations.len(), 1, "{violations:?}");
        let text = violations[0].domain_text();
        assert!(text.contains("horis:assignedBy"), "{text}");
    }

    #[test]
    fn accepts_every_documented_assigned_by_value() {
        for value in ["RulesOnly", "PerceptionAllowed", "ToolsOnly"] {
            let violations = violations_of(&format!(":acuity horis:assignedBy horis:{value} ."));
            assert!(violations.is_empty(), "{value}: {violations:?}");
        }
    }

    #[test]
    fn unknown_non_horis_namespace_is_fine() {
        let violations = violations_of(":Patient :favoriteColor \"blue\" .");
        assert!(violations.is_empty(), "{violations:?}");
    }

    #[test]
    fn collects_all_violations_in_one_pass_not_fail_fast() {
        let violations = violations_of(
            ":Foo a owl:Restriction .\n:Bar horis:frobnicate 1 .\n:Baz :p \"x\"@fr .",
        );
        assert_eq!(violations.len(), 3, "{violations:?}");
    }

    #[test]
    fn accepts_tool_contract_vocabulary() {
        let violations = violations_of(
            ":ehr.assign_queue a horis:Tool ;\n\
             \x20   horis:takes [\n\
             \x20       horis:arg [ horis:name \"patient\" ; horis:type :Patient ] ;\n\
             \x20       horis:arg [ horis:name \"queue\" ; horis:type :AcuityLevel ]\n\
             \x20   ] ;\n\
             \x20   horis:effect [\n\
             \x20       horis:asserts [ horis:property :queuedAt ; horis:value horis:now ] ;\n\
             \x20       horis:asserts [ horis:property :queueRef ; horis:type xsd:string ]\n\
             \x20   ] ;\n\
             \x20   horis:binding \"python:ehr_client.assign_queue\" ;\n\
             \x20   horis:timeout \"5s\" ;\n\
             \x20   horis:retries 2 ;\n\
             \x20   horis:idempotent false .",
        );
        assert!(violations.is_empty(), "{violations:?}");
    }

    #[test]
    fn accepts_proposal_step_vocabulary() {
        let violations = violations_of(
            ":PickOption a horis:ProposalStep ;\n\
             \x20   horis:candidates [ horis:property :option ] ;\n\
             \x20   horis:rankBy \"customer preference\" ;\n\
             \x20   horis:model \"claude:claude-fable-5\" ;\n\
             \x20   horis:confidence 0.8 ;\n\
             \x20   horis:fallback :AskHuman ;\n\
             \x20   horis:next :ExecuteOption .",
        );
        assert!(violations.is_empty(), "{violations:?}");
    }

    #[test]
    fn rejects_tool_vocabulary_term_not_in_profile() {
        let violations = violations_of(":Foo horis:query \"…\" .");
        assert_eq!(violations.len(), 1, "{violations:?}");
        let text = violations[0].domain_text();
        assert!(text.contains("horis:query"), "{text}");
    }

    #[test]
    fn triage_fixture_is_accepted_untouched() {
        let source = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/triage/triage.ttl"
        ))
        .expect("triage.ttl fixture must exist");
        let mut interner = Interner::new();
        let doc = parse(&source, &mut interner).expect("triage.ttl must parse");
        let result = validate(&doc, &interner);
        assert!(
            result.is_ok(),
            "triage.ttl must pass profile validation: {:?}",
            result.err().map(|e| e.domain_text())
        );
    }

    #[test]
    fn validate_documents_aggregates_violations_across_documents() {
        let (doc_b, mut interner_b) = parse_snippet(":Bar horis:frobnicate 1 .");
        // Both documents must share one interner, as a real multi-document
        // caller (U5) would.
        let doc_a = parse(
            "@prefix : <https://example.org/t#> .\n\
             @prefix owl: <http://www.w3.org/2002/07/owl#> .\n\
             :Foo a owl:Restriction .\n",
            &mut interner_b,
        )
        .expect("valid Turtle");
        let result = validate_documents([&doc_a, &doc_b], &interner_b);
        match result {
            Err(OntologyError::ProfileViolations(v)) => assert_eq!(v.len(), 2, "{v:?}"),
            other => panic!("expected ProfileViolations with 2 entries, got {other:?}"),
        }
    }

    #[test]
    fn validate_named_attributes_each_violation_to_its_graph() {
        let (doc_import, mut interner) = parse_snippet(":Bar horis:frobnicate 1 .");
        let doc_root = parse(
            "@prefix : <https://example.org/t#> .\n\
             @prefix owl: <http://www.w3.org/2002/07/owl#> .\n\
             :Foo a owl:Restriction .\n",
            &mut interner,
        )
        .expect("valid Turtle");

        let result = validate_named(
            [("root", &doc_root), ("urn:test:import", &doc_import)],
            &interner,
        );
        let violations = match result {
            Err(OntologyError::ProfileViolations(v)) => v,
            other => panic!("expected ProfileViolations, got {other:?}"),
        };
        assert_eq!(violations.len(), 2, "{violations:?}");
        for v in &violations {
            assert!(
                matches!(v, OntologyError::ProfileViolationInGraph { .. }),
                "{v:?}"
            );
        }
        let texts: Vec<String> = violations.iter().map(OntologyError::domain_text).collect();
        assert!(
            texts
                .iter()
                .any(|t| t.starts_with("root: ") && t.contains("owl:Restriction")),
            "{texts:?}"
        );
        assert!(
            texts
                .iter()
                .any(|t| t.starts_with("urn:test:import: ") && t.contains("horis:frobnicate")),
            "{texts:?}"
        );
    }
}
