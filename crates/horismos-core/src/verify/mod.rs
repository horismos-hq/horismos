//! Verification: BDD checks with witnesses, run before an ontology executes.
//!
//! Rule-conflict detection, workflow termination and reachability proofs,
//! constraint-domain analysis. Adapts the BDD-verifier design described by
//! PRP-005 to the ontology model; no prior-art source was consulted.
//!
//! Spec: `docs/framework/10-verification.md` · Owner: PRP-005 (schema checks
//! land earlier with PRP-002)

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

mod rules;

pub use rules::verify_ontology;

/// The explicit scope under which a [`VerificationReport`] holds.
///
/// These five fields are fixed by the bounded-projection pivot. Their
/// string/list value types are provisional until PRP-008 introduces the
/// core-native `ExternalSnapshot` and input-closure types; no field may be
/// removed or made implicit during that migration.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssumptionManifest {
    /// Facts/resources treated as the complete input space.
    pub input_closure: Vec<String>,
    /// External vocabulary/profile name to exact version.
    pub external_profile_versions: BTreeMap<String, String>,
    /// Version of the mapping that anchored discretized terms.
    pub mapping_version: Option<String>,
    /// Imported artifact constructs formalized by this verification.
    pub supported_subset: Vec<String>,
    /// Tool/adapter effects checked only at their declared shape boundary.
    pub unverified_external_effects: Vec<String>,
}

/// Stable machine category for a verification witness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    Completeness,
    Conflict,
    DeadRule,
    RuleTermination,
    RuleVacuity,
    ConstraintVacuity,
    WorkflowReachability,
    WorkflowTermination,
    WorkflowRequires,
    WorkflowFallback,
    WorkflowChoice,
}

/// Whether a witness fails the report or merely informs the author.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// The finding makes `ok` false and the default-hard load fail.
    Error,
    /// An advisory note; the report still passes.
    Info,
}

/// One endpoint of an ordered witness interval. Numbers, `xsd:dateTime`,
/// and strings are each ordered as the runtime orders them.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Bound {
    /// Exact authored lexical value; never rounded through `f64`.
    pub value: String,
    pub inclusive: bool,
}

/// One machine-readable predicate in a counterexample region.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RegionPredicate {
    Numeric {
        property: String,
        lower: Option<Bound>,
        upper: Option<Bound>,
        unit: Option<String>,
    },
    Present {
        property: String,
        value: String,
    },
    Absent {
        property: String,
        value: Option<String>,
    },
    Value {
        property: String,
        value: String,
    },
    Assigned {
        property: String,
        assigned: bool,
    },
}

/// A symbolic input region decoded from a BDD valuation.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Region {
    pub subject_class: Option<String>,
    pub predicates: Vec<RegionPredicate>,
}

/// A finding in both SME-readable and machine-readable form.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Witness {
    pub kind: FindingKind,
    pub severity: Severity,
    pub domain_text: String,
    pub region: Region,
    pub suggestion: Option<String>,
}

/// One overlapping pair of contradictory rule conclusions, with the rule
/// whose value stands on the overlap under the runtime's priority order.
/// `winner` is `None` when both rules share a priority: the standing value
/// then depends on firing order rather than on an authored resolution.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Conflict {
    pub first_rule: String,
    pub second_rule: String,
    pub property: String,
    pub first_value: String,
    pub second_value: String,
    pub first_priority: i64,
    pub second_priority: i64,
    pub winner: Option<String>,
    pub region: Region,
}

/// Per-workflow summary; detail remains in [`VerificationReport::witnesses`].
/// Step reachability is reported globally through
/// [`FindingKind::WorkflowReachability`] witnesses because an orphan step
/// belongs to no workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowVerification {
    pub terminating: bool,
    pub requires_satisfiable: bool,
    pub fallbacks_total: bool,
    pub choices_total: bool,
}

impl Default for WorkflowVerification {
    fn default() -> Self {
        Self {
            terminating: true,
            requires_satisfiable: true,
            fallbacks_total: true,
            choices_total: true,
        }
    }
}

/// The complete deterministic result of verifying one ontology.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerificationReport {
    ok: bool,
    complete: BTreeMap<String, bool>,
    conflicts: Vec<Conflict>,
    dead_rules: Vec<String>,
    terminating: bool,
    workflows: BTreeMap<String, WorkflowVerification>,
    witnesses: Vec<Witness>,
    assumptions: AssumptionManifest,
}

impl VerificationReport {
    /// A successful report for an ontology with no verifiable rules or
    /// workflows, still carrying all five explicit assumption fields.
    pub fn empty(assumptions: AssumptionManifest) -> Self {
        ReportBuilder::new(assumptions).finish()
    }

    pub fn ok(&self) -> bool {
        self.ok
    }

    pub fn complete(&self) -> &BTreeMap<String, bool> {
        &self.complete
    }

    pub fn conflicts(&self) -> &[Conflict] {
        &self.conflicts
    }

    pub fn dead_rules(&self) -> &[String] {
        &self.dead_rules
    }

    pub fn terminating(&self) -> bool {
        self.terminating
    }

    pub fn workflows(&self) -> &BTreeMap<String, WorkflowVerification> {
        &self.workflows
    }

    pub fn witnesses(&self) -> &[Witness] {
        &self.witnesses
    }

    /// The witnesses that make the report fail; info-level notes are excluded.
    pub fn failures(&self) -> impl Iterator<Item = &Witness> {
        self.witnesses
            .iter()
            .filter(|witness| witness.severity == Severity::Error)
    }

    pub fn assumptions(&self) -> &AssumptionManifest {
        &self.assumptions
    }

    /// Stable JSON for CI and authoring tooling.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// CI process status: zero only when every verification check passed.
    pub fn exit_code(&self) -> i32 {
        i32::from(!self.ok)
    }
}

/// Mutable construction state kept private to the verifier. Only
/// [`ReportBuilder::finish`] yields a public report, so `ok`, JSON, and exit
/// status cannot observe a partially accumulated finding set.
#[derive(Debug)]
pub(crate) struct ReportBuilder {
    pub(crate) complete: BTreeMap<String, bool>,
    pub(crate) conflicts: Vec<Conflict>,
    pub(crate) dead_rules: Vec<String>,
    pub(crate) terminating: bool,
    pub(crate) workflows: BTreeMap<String, WorkflowVerification>,
    pub(crate) witnesses: Vec<Witness>,
    assumptions: AssumptionManifest,
}

impl ReportBuilder {
    pub(crate) fn new(assumptions: AssumptionManifest) -> Self {
        Self {
            complete: BTreeMap::new(),
            conflicts: Vec::new(),
            dead_rules: Vec::new(),
            terminating: true,
            workflows: BTreeMap::new(),
            witnesses: Vec::new(),
            assumptions,
        }
    }

    /// Canonicalizes every serialized sequence and derives `ok` from all
    /// checks before the report becomes observable.
    pub(crate) fn finish(mut self) -> VerificationReport {
        self.assumptions.input_closure.sort();
        self.assumptions.input_closure.dedup();
        self.assumptions.supported_subset.sort();
        self.assumptions.supported_subset.dedup();
        self.assumptions.unverified_external_effects.sort();
        self.assumptions.unverified_external_effects.dedup();
        self.dead_rules.sort();
        self.dead_rules.dedup();
        for conflict in &mut self.conflicts {
            conflict.region.predicates.sort();
            conflict.region.predicates.dedup();
        }
        self.conflicts.sort();
        self.conflicts.dedup();
        for witness in &mut self.witnesses {
            witness.region.predicates.sort();
            witness.region.predicates.dedup();
        }
        self.witnesses.sort();
        self.witnesses.dedup();
        let ok = self
            .witnesses
            .iter()
            .all(|witness| witness.severity == Severity::Info)
            && self.complete.values().all(|complete| *complete)
            && self.conflicts.is_empty()
            && self.dead_rules.is_empty()
            && self.terminating
            && self.workflows.values().all(|workflow| {
                workflow.terminating
                    && workflow.requires_satisfiable
                    && workflow.fallbacks_total
                    && workflow.choices_total
            });
        VerificationReport {
            ok,
            complete: self.complete,
            conflicts: self.conflicts,
            dead_rules: self.dead_rules,
            terminating: self.terminating,
            workflows: self.workflows,
            witnesses: self.witnesses,
            assumptions: self.assumptions,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use biodivine_lib_bdd::BddVariableSet;

    use super::{
        AssumptionManifest, Bound, FindingKind, Region, RegionPredicate, ReportBuilder, Severity,
        VerificationReport, Witness,
    };

    #[test]
    fn every_report_carries_five_explicitly_empty_assumption_fields() {
        let report = VerificationReport::empty(AssumptionManifest::default());
        assert!(report.assumptions.input_closure.is_empty());
        assert!(report.assumptions.external_profile_versions.is_empty());
        assert_eq!(report.assumptions.mapping_version, None);
        assert!(report.assumptions.supported_subset.is_empty());
        assert!(report.assumptions.unverified_external_effects.is_empty());

        let json = report.to_json().expect("a report is JSON serializable");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&json).expect("valid JSON")["assumptions"],
            serde_json::json!({
                "input_closure": [],
                "external_profile_versions": {},
                "mapping_version": null,
                "supported_subset": [],
                "unverified_external_effects": []
            })
        );
    }

    #[test]
    fn populated_manifest_serializes_without_losing_scope() {
        let assumptions = AssumptionManifest {
            input_closure: vec!["Patient/123: Observation.code=heart-rate".into()],
            external_profile_versions: BTreeMap::from([("SNOMED CT".into(), "2026-07-31".into())]),
            mapping_version: Some("triage-map/4".into()),
            supported_subset: vec!["PlanDefinition.action.condition".into()],
            unverified_external_effects: vec!["ehr.assign_queue implementation".into()],
        };
        let report = VerificationReport::empty(assumptions.clone());
        assert_eq!(report.assumptions(), &assumptions);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(
                &report.to_json().expect("a report is JSON serializable")
            )
            .expect("valid JSON"),
            serde_json::json!({
                "ok": true,
                "complete": {},
                "conflicts": [],
                "dead_rules": [],
                "terminating": true,
                "workflows": {},
                "witnesses": [],
                "assumptions": {
                    "input_closure": ["Patient/123: Observation.code=heart-rate"],
                    "external_profile_versions": {"SNOMED CT": "2026-07-31"},
                    "mapping_version": "triage-map/4",
                    "supported_subset": ["PlanDefinition.action.condition"],
                    "unverified_external_effects": ["ehr.assign_queue implementation"]
                }
            })
        );
    }

    #[test]
    fn findings_are_sorted_and_make_the_exit_code_fail() {
        let interval = RegionPredicate::Numeric {
            property: "heartRate".into(),
            lower: Some(Bound {
                value: "100".into(),
                inclusive: false,
            }),
            upper: Some(Bound {
                value: "120".into(),
                inclusive: true,
            }),
            unit: Some("bpm".into()),
        };
        let mut builder = ReportBuilder::new(AssumptionManifest::default());
        builder.witnesses.push(Witness {
            kind: FindingKind::WorkflowReachability,
            severity: Severity::Error,
            domain_text: ":Orphan is unreachable from :Intake".into(),
            region: Region::default(),
            suggestion: None,
        });
        builder.witnesses.push(Witness {
            kind: FindingKind::Completeness,
            severity: Severity::Error,
            domain_text: "no acuity assigned when heartRate ∈ (100, 120] and no Symptom present"
                .into(),
            region: Region {
                subject_class: Some(":Patient".into()),
                predicates: vec![interval],
            },
            suggestion: Some("add a rule covering this region".into()),
        });
        let report = builder.finish();

        assert!(!report.ok());
        assert_eq!(report.exit_code(), 1);
        assert_eq!(report.witnesses()[0].kind, FindingKind::Completeness);
        assert_eq!(
            report.witnesses()[0].domain_text,
            "no acuity assigned when heartRate ∈ (100, 120] and no Symptom present"
        );
    }

    #[test]
    fn info_witnesses_are_reported_without_failing_the_report() {
        let mut builder = ReportBuilder::new(AssumptionManifest::default());
        builder.witnesses.push(Witness {
            kind: FindingKind::ConstraintVacuity,
            severity: Severity::Info,
            domain_text: "tags horis:cardinality \"0..*\" excludes no value count on :Patient"
                .into(),
            region: Region::default(),
            suggestion: None,
        });
        let report = builder.finish();

        assert!(report.ok());
        assert_eq!(report.exit_code(), 0);
        assert_eq!(report.witnesses().len(), 1);
        assert_eq!(report.failures().count(), 0);
        let json: serde_json::Value =
            serde_json::from_str(&report.to_json().expect("JSON")).expect("valid JSON");
        assert_eq!(json["ok"], serde_json::json!(true));
        assert_eq!(json["witnesses"][0]["severity"], serde_json::json!("info"));
    }

    #[test]
    fn equivalent_insertion_orders_produce_byte_identical_json() {
        let numeric = RegionPredicate::Numeric {
            property: "heartRate".into(),
            lower: Some(Bound {
                value: "100".into(),
                inclusive: false,
            }),
            upper: Some(Bound {
                value: "120".into(),
                inclusive: true,
            }),
            unit: Some("bpm".into()),
        };
        let absent = RegionPredicate::Absent {
            property: "hasSymptom".into(),
            value: Some(":Symptom".into()),
        };
        let make = |reverse: bool| {
            let mut builder = ReportBuilder::new(AssumptionManifest::default());
            let predicates = if reverse {
                vec![absent.clone(), numeric.clone()]
            } else {
                vec![numeric.clone(), absent.clone()]
            };
            let mut witnesses = vec![
                Witness {
                    kind: FindingKind::Completeness,
                    severity: Severity::Error,
                    domain_text: "same text".into(),
                    region: Region {
                        subject_class: Some(":Patient".into()),
                        predicates,
                    },
                    suggestion: Some("repair b".into()),
                },
                Witness {
                    kind: FindingKind::Completeness,
                    severity: Severity::Error,
                    domain_text: "same text".into(),
                    region: Region {
                        subject_class: Some(":Patient".into()),
                        predicates: vec![RegionPredicate::Assigned {
                            property: "heartRate".into(),
                            assigned: false,
                        }],
                    },
                    suggestion: Some("repair a".into()),
                },
            ];
            if reverse {
                witnesses.reverse();
            }
            builder.witnesses = witnesses;
            builder.finish()
        };

        let first = make(false);
        let second = make(true);
        assert_eq!(first, second);
        assert_eq!(
            first.to_json().expect("JSON"),
            second.to_json().expect("JSON")
        );
    }

    #[test]
    fn bdd_dependency_supports_deterministic_first_witness_selection() {
        let variables = BddVariableSet::new(&["a", "b"]);
        let region = variables.mk_var_by_name("b");
        let witness = region.first_valuation().expect("b is satisfiable");
        assert!(!witness.value(variables.var_by_name("a").expect("a exists")));
        assert!(witness.value(variables.var_by_name("b").expect("b exists")));
    }
}

#[cfg(test)]
mod rule_verification_tests {
    use super::{AssumptionManifest, FindingKind, RegionPredicate, Severity, verify_ontology};
    use crate::{Ontology, Value, World};

    const PRELUDE: &str = "@prefix : <https://example.org/verify#> .\n\
        @prefix horis: <https://w3id.org/horismos#> .\n\
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n";

    fn load(body: &str) -> Ontology {
        Ontology::load_from_str_allow_incomplete(&format!("{PRELUDE}{body}"), |_| None)
            .expect("verification fixture must parse")
    }

    fn report(body: &str) -> super::VerificationReport {
        verify_ontology(&load(body), AssumptionManifest::default())
    }

    const TRIAGE_DOMAIN: &str = r#"
        :Patient a rdfs:Class .
        :Symptom a rdfs:Class .
        :ChestPain rdfs:subClassOf :Symptom .
        :CrushingChestPain rdfs:subClassOf :ChestPain .
        :heartRate a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:decimal ;
            horis:unit "bpm" ; horis:min 0 ; horis:max 300 ;
            horis:cardinality "0..1" .
        :hasSymptom a rdf:Property ; rdfs:domain :Patient ; rdfs:range :Symptom .
        :AcuityLevel a rdfs:Class ; horis:oneOf ( :Immediate :Urgent :Standard ) .
        :acuity a rdf:Property ; rdfs:domain :Patient ; rdfs:range :AcuityLevel ;
            horis:cardinality "0..1" ; horis:assignedBy horis:RulesOnly .
    "#;

    const TRIAGE_RULES_WITH_GAP: &str = r#"
        :r1 a horis:Rule ; horis:priority 50 ;
            horis:when [ horis:all (
                [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 120 ]
                [ horis:subject :Patient ; horis:property :hasSymptom ; horis:is :ChestPain ]
            ) ] ; horis:then [ horis:property :acuity ; horis:value :Immediate ] ;
            horis:explain "immediate" .
        :r2 a horis:Rule ; horis:priority 40 ;
            horis:when [ horis:all (
                [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 100 ]
                [ horis:subject :Patient ; horis:property :heartRate ; horis:atMost 120 ]
                [ horis:subject :Patient ; horis:property :hasSymptom ; horis:is :Symptom ]
            ) ] ; horis:then [ horis:property :acuity ; horis:value :Urgent ] ;
            horis:explain "moderate with symptom" .
        :r3 a horis:Rule ; horis:priority 30 ;
            horis:when [ horis:subject :Patient ; horis:property :heartRate ; horis:atMost 100 ] ;
            horis:then [ horis:property :acuity ; horis:value :Standard ] ;
            horis:explain "standard" .
        :r4 a horis:Rule ; horis:priority 20 ;
            horis:when [ horis:all (
                [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 120 ]
                [ horis:subject :Patient ; horis:property :hasSymptom ; horis:isNot :ChestPain ]
            ) ] ; horis:then [ horis:property :acuity ; horis:value :Urgent ] ;
            horis:explain "urgent" .
        :r5 a horis:Rule ; horis:priority 10 ;
            horis:when [ horis:subject :Patient ; horis:property :heartRate ; horis:isEmpty true ] ;
            horis:then [ horis:property :acuity ; horis:value :Standard ] ;
            horis:explain "no vital" .
    "#;

    #[test]
    fn seeded_triage_gap_has_the_exact_domain_sentence_and_region() {
        let source = include_str!("../../../../examples/triage/triage.ttl");
        let start = source
            .find("# PRP-005 completeness-cover:start")
            .expect("canonical triage marks the covering rule");
        let end_marker = "# PRP-005 completeness-cover:end";
        let end = source
            .find(end_marker)
            .map(|index| index + end_marker.len())
            .expect("canonical triage closes the covering-rule marker");
        let seeded = format!("{}{}", &source[..start], &source[end..]);
        let ontology = Ontology::load_from_str_allow_incomplete(&seeded, |_| None)
            .expect("the seeded canonical triage ontology remains structurally valid");
        let result = verify_ontology(&ontology, AssumptionManifest::default());
        let witness = result
            .witnesses()
            .iter()
            .find(|w| w.kind == FindingKind::Completeness)
            .expect("the seeded completeness gap must be found");
        assert_eq!(
            witness.domain_text,
            "no acuity assigned when heartRate ∈ (100, 120] and no Symptom present"
        );
        assert!(witness.region.predicates.iter().any(|predicate| matches!(
            predicate,
            RegionPredicate::Numeric { property, lower: Some(lower), upper: Some(upper), unit: Some(unit) }
                if property == "heartRate" && lower.value == "100" && !lower.inclusive
                    && upper.value == "120" && upper.inclusive && unit == "bpm"
        )));
        assert!(witness.region.predicates.iter().any(|predicate| matches!(
            predicate,
            RegionPredicate::Absent { property, value: Some(value) }
                if property == "hasSymptom" && value == ":Symptom"
        )));
    }

    #[test]
    fn conflicts_dead_rules_and_cycles_are_reported() {
        let conflict = report(&format!(
            "{TRIAGE_DOMAIN}\n\
             :a a horis:Rule ; horis:priority 2 ; horis:when [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 100 ] ; horis:then [ horis:property :acuity ; horis:value :Immediate ] .\n\
             :b a horis:Rule ; horis:priority 1 ; horis:when [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 120 ] ; horis:then [ horis:property :acuity ; horis:value :Urgent ] ."
        ));
        assert_eq!(conflict.conflicts().len(), 1);
        let found = &conflict.conflicts()[0];
        assert_eq!(found.first_rule, ":a");
        assert_eq!(found.second_rule, ":b");
        assert_eq!(found.first_priority, 2);
        assert_eq!(found.second_priority, 1);
        assert_eq!(found.winner.as_deref(), Some(":a"));
        let witness = conflict
            .witnesses()
            .iter()
            .find(|w| w.kind == FindingKind::Conflict)
            .expect("the overlap carries a witness");
        assert_eq!(
            witness.domain_text,
            ":a → :Immediate but :b → :Urgent for :acuity when heartRate ∈ (120, 300] and no Symptom present — :a wins by priority; confirm intended"
        );

        let tie = report(&format!(
            "{TRIAGE_DOMAIN}\n\
             :a a horis:Rule ; horis:priority 1 ; horis:when [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 100 ] ; horis:then [ horis:property :acuity ; horis:value :Immediate ] .\n\
             :b a horis:Rule ; horis:priority 1 ; horis:when [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 120 ] ; horis:then [ horis:property :acuity ; horis:value :Urgent ] ."
        ));
        assert_eq!(tie.conflicts().len(), 1);
        assert_eq!(tie.conflicts()[0].winner, None);
        assert_eq!(tie.conflicts()[0].first_priority, 1);
        assert_eq!(tie.conflicts()[0].second_priority, 1);
        let tie_witness = tie
            .witnesses()
            .iter()
            .find(|w| w.kind == FindingKind::Conflict)
            .expect("the tied overlap carries a witness");
        assert_eq!(
            tie_witness.domain_text,
            ":a → :Immediate but :b → :Urgent for :acuity when heartRate ∈ (120, 300] and no Symptom present — both share priority 1, so the standing value is order-dependent; raise one rule's priority to make the resolution explicit"
        );
        assert!(!tie_witness.domain_text.contains("wins by priority"));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&tie.to_json().expect("JSON"))
                .expect("valid JSON")["conflicts"][0]["winner"],
            serde_json::Value::Null
        );

        let dead = report(&format!(
            "{TRIAGE_DOMAIN}\n\
             :dead a horis:Rule ; horis:when [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 350 ] ; horis:then [ horis:property :acuity ; horis:value :Immediate ] ."
        ));
        assert_eq!(dead.dead_rules(), &[":dead"]);

        let cycle = report(
            r#"
            :Patient a rdfs:Class .
            :flagA a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
                horis:assignedBy horis:RulesOnly .
            :flagB a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
                horis:assignedBy horis:RulesOnly .
            :a a horis:Rule ; horis:when [ horis:subject :Patient ; horis:property :flagB ; horis:isAssigned true ] ;
                horis:then [ horis:property :flagA ; horis:value true ] .
            :b a horis:Rule ; horis:when [ horis:subject :Patient ; horis:property :flagA ; horis:isAssigned true ] ;
                horis:then [ horis:property :flagB ; horis:value true ] .
            "#,
        );
        assert!(!cycle.terminating());
        assert!(
            cycle
                .witnesses()
                .iter()
                .any(|w| w.kind == FindingKind::RuleTermination)
        );
    }

    #[test]
    fn satisfiable_fully_shadowed_rule_is_vacuous_not_dead() {
        let result = report(&format!(
            "{TRIAGE_DOMAIN}\n\
             :high a horis:Rule ; horis:priority 10 ; horis:when [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 0 ] ; horis:then [ horis:property :acuity ; horis:value :Immediate ] .\n\
             :low a horis:Rule ; horis:priority 1 ; horis:when [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 100 ] ; horis:then [ horis:property :acuity ; horis:value :Urgent ] ."
        ));
        assert!(!result.dead_rules().iter().any(|rule| rule == ":low"));
        let witness = result
            .witnesses()
            .iter()
            .find(|w| w.kind == FindingKind::RuleVacuity)
            .expect("the inert lower-priority rule must be caught");
        assert!(witness.domain_text.contains(":low"));
        assert!(witness.domain_text.contains("higher-priority"));
    }

    #[test]
    fn distinct_multi_valued_conclusions_are_not_vacuously_shadowed() {
        let body = r#"
            :Patient a rdfs:Class .
            :Decision a rdfs:Class ; horis:oneOf ( :A :B ) .
            :flag a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ; horis:cardinality "1" .
            :decision a rdf:Property ; rdfs:domain :Patient ; rdfs:range :Decision ;
                horis:assignedBy horis:RulesOnly .
            :high a horis:Rule ; horis:priority 10 ;
                horis:when [ horis:subject :Patient ; horis:property :flag ; horis:isAssigned true ] ;
                horis:then [ horis:property :decision ; horis:value :A ] .
            :low a horis:Rule ; horis:priority 1 ;
                horis:when [ horis:subject :Patient ; horis:property :flag ; horis:isAssigned true ] ;
                horis:then [ horis:property :decision ; horis:value :B ] .
        "#;
        let source = format!("{PRELUDE}{body}");
        let ontology = Ontology::load_from_str(&source, |_| None)
            .expect("distinct multi-valued conclusions are both effective");
        assert!(ontology.verify().ok());

        let patient = ontology.node_ref(":Patient").expect(":Patient");
        let flag = ontology.node_ref(":flag").expect(":flag");
        let decision = ontology.node_ref(":decision").expect(":decision");
        let mut world = World::new(&ontology);
        let subject = world
            .assert_instance(patient, [(flag, Value::Boolean(true))])
            .expect("both rule conclusions pass the gate");
        assert_eq!(world.values(subject, decision).count(), 2);
    }

    #[test]
    fn universal_pattern_and_zero_or_many_are_vacuous_constraints() {
        let result = report(
            r#"
            :Patient a rdfs:Class .
            :code a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string ;
                horis:pattern ".*" .
            :tags a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string ;
                horis:cardinality "0..*" .
            :restricted a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string ;
                horis:pattern "A+" ; horis:cardinality "0..1" .
            :spaced a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string ;
                horis:pattern " .* " .
            "#,
        );
        let findings: Vec<(&str, Severity)> = result
            .witnesses()
            .iter()
            .filter(|w| w.kind == FindingKind::ConstraintVacuity)
            .map(|w| (w.domain_text.as_str(), w.severity))
            .collect();
        assert_eq!(findings.len(), 2);
        assert!(
            findings
                .iter()
                .any(|(text, severity)| text.contains("code") && *severity == Severity::Error)
        );
        assert!(
            findings
                .iter()
                .any(|(text, severity)| text.contains("tags") && *severity == Severity::Info)
        );
        assert!(!findings.iter().any(|(text, _)| text.contains("restricted")));
        assert!(!findings.iter().any(|(text, _)| text.contains("spaced")));
        assert!(!result.ok());
    }

    #[test]
    fn explicit_zero_or_many_is_an_informational_note_that_still_loads() {
        let body = r#"
            :Patient a rdfs:Class .
            :tags a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string ;
                horis:cardinality "0..*" .
        "#;
        let ontology = Ontology::load_from_str(&format!("{PRELUDE}{body}"), |_| None)
            .expect("the documented default cardinality loads under the fail-closed default");
        let report = ontology.verify();
        assert!(report.ok());
        assert_eq!(report.exit_code(), 0);
        assert_eq!(report.failures().count(), 0);
        let note = &report.witnesses()[0];
        assert_eq!(note.kind, FindingKind::ConstraintVacuity);
        assert_eq!(note.severity, Severity::Info);
        assert_eq!(
            note.domain_text,
            "tags horis:cardinality \"0..*\" excludes no value count on :Patient"
        );

        let canonical = Ontology::load(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/triage/triage.ttl"
        ))
        .expect("canonical triage declares 0..* on hasSymptom and still loads");
        let canonical_report = canonical.verify();
        assert!(canonical_report.ok());
        assert!(canonical_report.witnesses().iter().all(|witness| {
            witness.severity == Severity::Info && witness.domain_text.starts_with("hasSymptom")
        }));
    }

    #[test]
    fn bounds_that_cannot_reject_a_typed_value_are_vacuous() {
        let result = report(
            r#"
            :Patient a rdfs:Class .
            :label a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string ;
                horis:max 10 .
            :flag a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
                horis:pattern "yes|no" .
            :heartRate a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:decimal ;
                horis:min 0 ; horis:max 300 .
            :note a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string ;
                horis:pattern "A+" .
            "#,
        );
        let texts: Vec<&str> = result
            .witnesses()
            .iter()
            .filter(|w| w.kind == FindingKind::ConstraintVacuity && w.severity == Severity::Error)
            .map(|w| w.domain_text.as_str())
            .collect();
        assert_eq!(
            texts,
            vec![
                "flag horis:pattern \"yes|no\" excludes no xsd:boolean value on :Patient",
                "label horis:max 10 excludes no xsd:string value on :Patient",
            ]
        );
        assert!(!result.ok());
    }

    #[test]
    fn datetime_and_string_orderings_follow_the_runtime() {
        let datetime_domain = r#"
            :Patient a rdfs:Class .
            :queuedAt a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:dateTime ;
                horis:cardinality "0..1" .
            :decision a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
                horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
            :late a horis:Rule ; horis:priority 2 ;
                horis:when [ horis:subject :Patient ; horis:property :queuedAt ; horis:exceeds "2026-01-01T00:00:00Z"^^xsd:dateTime ] ;
                horis:then [ horis:property :decision ; horis:value true ] .
            :none a horis:Rule ; horis:priority 0 ;
                horis:when [ horis:subject :Patient ; horis:property :queuedAt ; horis:isEmpty true ] ;
                horis:then [ horis:property :decision ; horis:value false ] .
        "#;
        let early_rule = r#"
            :early a horis:Rule ; horis:priority 1 ;
                horis:when [ horis:subject :Patient ; horis:property :queuedAt ; horis:atMost "2026-01-01T00:00:00Z"^^xsd:dateTime ] ;
                horis:then [ horis:property :decision ; horis:value false ] .
        "#;
        let gap = report(datetime_domain);
        assert!(gap.dead_rules().is_empty());
        assert_eq!(gap.complete().get(":decision"), Some(&false));
        let witness = gap
            .witnesses()
            .iter()
            .find(|w| w.kind == FindingKind::Completeness)
            .expect("the early region is uncovered");
        assert_eq!(
            witness.domain_text,
            "no decision assigned when queuedAt ≤ 2026-01-01T00:00:00+00:00"
        );

        let source = format!("{PRELUDE}{datetime_domain}{early_rule}");
        let ontology = Ontology::load_from_str(&source, |_| None)
            .expect("a dateTime partition verifies under the fail-closed default");
        let complete = ontology.verify();
        assert!(complete.ok());
        assert!(complete.dead_rules().is_empty());
        let patient = ontology.node_ref(":Patient").expect(":Patient");
        let queued_at = ontology.node_ref(":queuedAt").expect(":queuedAt");
        let decision = ontology.node_ref(":decision").expect(":decision");
        let mut world = World::new(&ontology);
        let subject = world
            .assert_instance(
                patient,
                [(
                    queued_at,
                    Value::parse_typed(
                        "2026-02-01T00:00:00Z",
                        crate::ontology::term::Datatype::DateTime,
                    )
                    .expect("valid dateTime"),
                )],
            )
            .expect("the base input is valid");
        assert_eq!(world.value(subject, decision), Some(&Value::Boolean(true)));

        let strings = report(
            r#"
            :Patient a rdfs:Class .
            :name a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string ;
                horis:cardinality "1" .
            :decision a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
                horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
            :upper a horis:Rule ; horis:priority 3 ;
                horis:when [ horis:subject :Patient ; horis:property :name ; horis:atLeast "M" ] ;
                horis:then [ horis:property :decision ; horis:value true ] .
            :lower a horis:Rule ; horis:priority 2 ;
                horis:when [ horis:subject :Patient ; horis:property :name ; horis:below "M" ] ;
                horis:then [ horis:property :decision ; horis:value false ] .
            :contradictory a horis:Rule ; horis:priority 1 ; horis:when [ horis:all (
                [ horis:subject :Patient ; horis:property :name ; horis:is "ready" ]
                [ horis:subject :Patient ; horis:property :name ; horis:below "M" ]
            ) ] ; horis:then [ horis:property :decision ; horis:value true ] .
            "#,
        );
        assert_eq!(strings.complete().get(":decision"), Some(&true));
        assert!(strings.conflicts().is_empty());
        assert_eq!(strings.dead_rules(), &[":contradictory"]);

        let incomparable = report(
            r#"
            :Patient a rdfs:Class .
            :flag a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
                horis:cardinality "1" .
            :tag a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string .
            :never a horis:Rule ;
                horis:when [ horis:subject :Patient ; horis:property :flag ; horis:exceeds 0 ] ;
                horis:then [ horis:property :tag ; horis:value "x" ] .
            "#,
        );
        assert_eq!(incomparable.dead_rules(), &[":never"]);
    }

    #[test]
    fn a_rule_replacing_a_single_valued_input_value_hides_the_input_from_later_rules() {
        let body = r#"
            :Patient a rdfs:Class .
            :Decision a rdfs:Class ; horis:oneOf ( :X :Y ) .
            :input a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
                horis:cardinality "1" .
            :score a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:decimal ;
                horis:min 0 ; horis:max 1 ; horis:cardinality "1" .
            :decision a rdf:Property ; rdfs:domain :Patient ; rdfs:range :Decision ;
                horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
            :rescore a horis:Rule ; horis:priority 3 ;
                horis:when [ horis:subject :Patient ; horis:property :input ; horis:is true ] ;
                horis:then [ horis:property :score ; horis:value 1 ] .
            :half a horis:Rule ; horis:priority 2 ;
                horis:when [ horis:subject :Patient ; horis:property :score ; horis:is 0.5 ] ;
                horis:then [ horis:property :decision ; horis:value :X ] .
            :full a horis:Rule ; horis:priority 1 ;
                horis:when [ horis:subject :Patient ; horis:property :score ; horis:is 1 ] ;
                horis:then [ horis:property :decision ; horis:value :Y ] .
            :other a horis:Rule ; horis:priority 0 ;
                horis:when [ horis:subject :Patient ; horis:property :score ; horis:isNot 0.5 ] ;
                horis:then [ horis:property :decision ; horis:value :Y ] .
        "#;
        let ontology = Ontology::load_from_str(&format!("{PRELUDE}{body}"), |_| None)
            .expect("the replaced score never reaches the half rule, so nothing conflicts");
        let report = ontology.verify();
        assert!(report.ok());
        assert!(report.conflicts().is_empty());
        assert_eq!(report.complete().get(":decision"), Some(&true));

        let patient = ontology.node_ref(":Patient").expect(":Patient");
        let input = ontology.node_ref(":input").expect(":input");
        let score = ontology.node_ref(":score").expect(":score");
        let decision = ontology.node_ref(":decision").expect(":decision");
        let expected = Value::Ref(crate::ontology::term::Ref::new(
            ontology.node_ref(":Y").expect(":Y"),
        ));
        let mut world = World::new(&ontology);
        let subject = world
            .assert_instance(
                patient,
                [
                    (input, Value::Boolean(true)),
                    (score, Value::Decimal("0.5".parse().expect("decimal"))),
                ],
            )
            .expect("the base input is valid");
        assert_eq!(world.value(subject, score), Some(&Value::Integer(1)));
        assert_eq!(world.value(subject, decision), Some(&expected));
    }

    #[test]
    fn a_reader_outranking_the_replacing_rule_fires_on_the_input_value() {
        let body = r#"
            :Patient a rdfs:Class .
            :Decision a rdfs:Class ; horis:oneOf ( :X :Y ) .
            :input a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
                horis:cardinality "1" .
            :score a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:decimal ;
                horis:min 0 ; horis:max 1 ; horis:cardinality "1" .
            :decision a rdf:Property ; rdfs:domain :Patient ; rdfs:range :Decision ;
                horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
            :half a horis:Rule ; horis:priority 4 ;
                horis:when [ horis:subject :Patient ; horis:property :score ; horis:is 0.5 ] ;
                horis:then [ horis:property :decision ; horis:value :X ] .
            :rescore a horis:Rule ; horis:priority 3 ;
                horis:when [ horis:subject :Patient ; horis:property :input ; horis:is true ] ;
                horis:then [ horis:property :score ; horis:value 1 ] .
            :full a horis:Rule ; horis:priority 1 ;
                horis:when [ horis:subject :Patient ; horis:property :score ; horis:is 1 ] ;
                horis:then [ horis:property :decision ; horis:value :Y ] .
        "#;
        let source = format!("{PRELUDE}{body}");
        let error = Ontology::load_from_str(&source, |_| None)
            .expect_err("the half rule reads the input score before rescore replaces it");
        let crate::error::OntologyError::VerificationFailed { report } = error else {
            panic!("expected the complete verification report");
        };
        assert_eq!(report.conflicts().len(), 1);
        let conflict = &report.conflicts()[0];
        assert_eq!(
            (conflict.first_rule.as_str(), conflict.second_rule.as_str()),
            (":half", ":full")
        );
        assert_eq!(conflict.winner.as_deref(), Some(":half"));
        assert!(!report.dead_rules().iter().any(|rule| rule == ":half"));

        let ontology = Ontology::load_from_str_allow_incomplete(&source, |_| None)
            .expect("the explicit escape keeps the conflicting ontology runnable");
        let patient = ontology.node_ref(":Patient").expect(":Patient");
        let input = ontology.node_ref(":input").expect(":input");
        let score = ontology.node_ref(":score").expect(":score");
        let decision = ontology.node_ref(":decision").expect(":decision");
        let expected = Value::Ref(crate::ontology::term::Ref::new(
            ontology.node_ref(":X").expect(":X"),
        ));
        let mut world = World::new(&ontology);
        let subject = world
            .assert_instance(
                patient,
                [
                    (input, Value::Boolean(true)),
                    (score, Value::Decimal("0.5".parse().expect("decimal"))),
                ],
            )
            .expect("the base input is valid");
        assert_eq!(world.value(subject, score), Some(&Value::Integer(1)));
        assert_eq!(world.value(subject, decision), Some(&expected));
    }

    #[test]
    fn a_reader_outranking_the_writer_still_sees_the_empty_input() {
        let body = r#"
            :Patient a rdfs:Class .
            :input a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
                horis:cardinality "1" .
            :score a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:decimal ;
                horis:min 0 ; horis:max 1 ; horis:cardinality "0..1" .
            :decision a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
                horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
            :unscored a horis:Rule ; horis:priority 4 ;
                horis:when [ horis:subject :Patient ; horis:property :score ; horis:isEmpty true ] ;
                horis:then [ horis:property :decision ; horis:value true ] .
            :rescore a horis:Rule ; horis:priority 3 ;
                horis:when [ horis:subject :Patient ; horis:property :input ; horis:is true ] ;
                horis:then [ horis:property :score ; horis:value 1 ] .
            :scored a horis:Rule ; horis:priority 1 ;
                horis:when [ horis:subject :Patient ; horis:property :score ; horis:isAssigned true ] ;
                horis:then [ horis:property :decision ; horis:value false ] .
        "#;
        let source = format!("{PRELUDE}{body}");
        let ontology =
            Ontology::load_from_str_allow_incomplete(&source, |_| None).expect("fixture parses");
        let report = ontology.verify();
        assert!(!report.dead_rules().iter().any(|rule| rule == ":unscored"));
        assert_eq!(report.conflicts().len(), 1);
        assert_eq!(report.conflicts()[0].winner.as_deref(), Some(":unscored"));

        let patient = ontology.node_ref(":Patient").expect(":Patient");
        let input = ontology.node_ref(":input").expect(":input");
        let score = ontology.node_ref(":score").expect(":score");
        let decision = ontology.node_ref(":decision").expect(":decision");
        let mut world = World::new(&ontology);
        let subject = world
            .assert_instance(patient, [(input, Value::Boolean(true))])
            .expect("the base input is valid");
        assert_eq!(world.value(subject, score), Some(&Value::Integer(1)));
        assert_eq!(world.value(subject, decision), Some(&Value::Boolean(true)));
    }

    #[test]
    fn thresholds_of_different_kinds_never_share_a_cell() {
        let body = r#"
            :Patient a rdfs:Class .
            :x a rdf:Property ; rdfs:domain :Patient .
            :tag a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string .
            :numeric a horis:Rule ;
                horis:when [ horis:subject :Patient ; horis:property :x ; horis:exceeds 5 ] ;
                horis:then [ horis:property :tag ; horis:value "n" ] .
            :textual a horis:Rule ;
                horis:when [ horis:subject :Patient ; horis:property :x ; horis:below "M" ] ;
                horis:then [ horis:property :tag ; horis:value "t" ] .
            :mixed a horis:Rule ; horis:when [ horis:all (
                [ horis:subject :Patient ; horis:property :x ; horis:exceeds 5 ]
                [ horis:subject :Patient ; horis:property :x ; horis:below "M" ]
            ) ] ; horis:then [ horis:property :tag ; horis:value "impossible" ] .
        "#;
        let result = report(body);
        assert_eq!(result.dead_rules(), &[":mixed"]);
    }

    #[test]
    fn boolean_partitions_are_closed_over_true_and_false() {
        let body = r#"
            :Patient a rdfs:Class .
            :Decision a rdfs:Class ; horis:oneOf ( :A :B ) .
            :flag a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
                horis:cardinality "1" .
            :decision a rdf:Property ; rdfs:domain :Patient ; rdfs:range :Decision ;
                horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
            :yes a horis:Rule ; horis:priority 2 ;
                horis:when [ horis:subject :Patient ; horis:property :flag ; horis:is true ] ;
                horis:then [ horis:property :decision ; horis:value :A ] .
            :no a horis:Rule ; horis:priority 1 ;
                horis:when [ horis:subject :Patient ; horis:property :flag ; horis:is false ] ;
                horis:then [ horis:property :decision ; horis:value :B ] .
            :Flow a horis:Workflow ; horis:startsAt :Choose .
            :Choose a horis:ChoiceStep ; horis:branches (
                [ horis:when [ horis:property :flag ; horis:is true ] ; horis:next :Done ]
                [ horis:when [ horis:property :flag ; horis:is false ] ; horis:next :Done ]
            ) .
            :Done a horis:Step ; horis:terminal true .
        "#;
        let ontology = Ontology::load_from_str(&format!("{PRELUDE}{body}"), |_| None)
            .expect("a total true/false partition verifies under the fail-closed default");
        let report = ontology.verify();
        assert!(report.ok());
        assert_eq!(report.complete().get(":decision"), Some(&true));
        assert!(report.workflows()[":Flow"].choices_total);

        let patient = ontology.node_ref(":Patient").expect(":Patient");
        let flag = ontology.node_ref(":flag").expect(":flag");
        let decision = ontology.node_ref(":decision").expect(":decision");
        let expected = Value::Ref(crate::ontology::term::Ref::new(
            ontology.node_ref(":B").expect(":B"),
        ));
        let mut world = World::new(&ontology);
        let subject = world
            .assert_instance(patient, [(flag, Value::Boolean(false))])
            .expect("the base input is valid");
        assert_eq!(world.value(subject, decision), Some(&expected));
    }

    #[test]
    fn rule_conclusions_on_perception_allowed_properties_support_downstream_rules() {
        let body = r#"
            :Patient a rdfs:Class .
            :input a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
                horis:cardinality "1" .
            :score a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:decimal ;
                horis:min 0 ; horis:max 1 ; horis:cardinality "0..1" .
            :decision a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
                horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
            :scoreIt a horis:Rule ; horis:priority 2 ;
                horis:when [ horis:subject :Patient ; horis:property :input ; horis:isAssigned true ] ;
                horis:then [ horis:property :score ; horis:value 1 ] .
            :decide a horis:Rule ; horis:priority 1 ;
                horis:when [ horis:subject :Patient ; horis:property :score ; horis:isAssigned true ] ;
                horis:then [ horis:property :decision ; horis:value true ] .
        "#;
        let unscored_rule = r#"
            :unscored a horis:Rule ; horis:priority 0 ;
                horis:when [ horis:subject :Patient ; horis:property :score ; horis:isEmpty true ] ;
                horis:then [ horis:property :decision ; horis:value false ] .
        "#;
        let with_unscored = report(&format!("{body}{unscored_rule}"));
        assert_eq!(with_unscored.complete().get(":decision"), Some(&true));
        assert_eq!(with_unscored.dead_rules(), &[":unscored"]);

        let ontology = Ontology::load_from_str(&format!("{PRELUDE}{body}"), |_| None)
            .expect("a rule-established perception property completes the partition");
        assert!(ontology.verify().ok());
        let patient = ontology.node_ref(":Patient").expect(":Patient");
        let input = ontology.node_ref(":input").expect(":input");
        let decision = ontology.node_ref(":decision").expect(":decision");
        let mut world = World::new(&ontology);
        let subject = world
            .assert_instance(patient, [(input, Value::Boolean(true))])
            .expect("the base input is valid");
        assert_eq!(world.value(subject, decision), Some(&Value::Boolean(true)));
    }

    #[test]
    fn integer_cells_exact_numbers_and_contains_follow_runtime_semantics() {
        let integer_partition = report(
            r#"
            :Patient a rdfs:Class .
            :count a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:integer ;
                horis:cardinality "1" .
            :decision a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
                horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
            :low a horis:Rule ; horis:when [ horis:subject :Patient ; horis:property :count ; horis:atMost 0 ] ;
                horis:then [ horis:property :decision ; horis:value false ] .
            :high a horis:Rule ; horis:when [ horis:subject :Patient ; horis:property :count ; horis:atLeast 1 ] ;
                horis:then [ horis:property :decision ; horis:value true ] .
            :impossible a horis:Rule ; horis:when [ horis:all (
                [ horis:subject :Patient ; horis:property :count ; horis:is 50 ]
                [ horis:subject :Patient ; horis:property :count ; horis:exceeds 120 ]
            ) ] ; horis:then [ horis:property :decision ; horis:value true ] .
            "#,
        );
        assert_eq!(integer_partition.complete().get(":decision"), Some(&true));
        assert!(
            integer_partition
                .dead_rules()
                .iter()
                .any(|rule| rule == ":impossible")
        );

        let contains = report(
            r#"
            :Patient a rdfs:Class .
            :code a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string .
            :tag a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string .
            :substring a horis:Rule ; horis:when [ horis:all (
                [ horis:subject :Patient ; horis:property :code ; horis:contains "x" ]
                [ horis:subject :Patient ; horis:property :code ; horis:isNot "x" ]
            ) ] ; horis:then [ horis:property :tag ; horis:value "matched" ] .
            "#,
        );
        assert!(
            !contains
                .dead_rules()
                .iter()
                .any(|rule| rule == ":substring")
        );
    }

    #[test]
    fn semantic_conclusion_equality_and_enabling_cycles_avoid_false_findings() {
        let equivalent_numbers = report(
            r#"
            :Patient a rdfs:Class .
            :input a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean .
            :score a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:decimal ;
                horis:cardinality "0..1" .
            :a a horis:Rule ; horis:when [ horis:subject :Patient ; horis:property :input ; horis:isAssigned true ] ;
                horis:then [ horis:property :score ; horis:value 1 ] .
            :b a horis:Rule ; horis:when [ horis:subject :Patient ; horis:property :input ; horis:isAssigned true ] ;
                horis:then [ horis:property :score ; horis:value 1.0 ] .
            "#,
        );
        assert!(equivalent_numbers.conflicts().is_empty());

        let disabling_cycle = report(
            r#"
            :Patient a rdfs:Class .
            :flagA a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ; horis:assignedBy horis:RulesOnly .
            :flagB a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ; horis:assignedBy horis:RulesOnly .
            :a a horis:Rule ; horis:when [ horis:subject :Patient ; horis:property :flagB ; horis:isEmpty true ] ;
                horis:then [ horis:property :flagA ; horis:value true ] .
            :b a horis:Rule ; horis:when [ horis:subject :Patient ; horis:property :flagA ; horis:isEmpty true ] ;
                horis:then [ horis:property :flagB ; horis:value true ] .
            "#,
        );
        assert!(disabling_cycle.terminating());

        let compound_non_enabling = report(
            r#"
            :Patient a rdfs:Class .
            :x a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string ; horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
            :y a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ; horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
            :source a horis:Rule ; horis:when [ horis:subject :Patient ; horis:property :y ; horis:isAssigned true ] ;
                horis:then [ horis:property :x ; horis:value "Y" ] .
            :noneTarget a horis:Rule ; horis:when [ horis:none (
                [ horis:subject :Patient ; horis:property :x ; horis:isEmpty true ]
                [ horis:subject :Patient ; horis:property :x ; horis:isNot "X" ]
            ) ] ; horis:then [ horis:property :y ; horis:value true ] .
            :allTarget a horis:Rule ; horis:when [ horis:all (
                [ horis:subject :Patient ; horis:property :x ; horis:is "X" ]
                [ horis:subject :Patient ; horis:property :x ; horis:is "Y" ]
            ) ] ; horis:then [ horis:property :y ; horis:value true ] .
            "#,
        );
        assert!(compound_non_enabling.terminating());

        let multi_value_enabling = report(
            r#"
            :Patient a rdfs:Class .
            :seedA a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean .
            :seedB a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean .
            :x a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string ; horis:assignedBy horis:RulesOnly ; horis:cardinality "0..*" .
            :y a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ; horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
            :priorA a horis:Rule ; horis:when [ horis:subject :Patient ; horis:property :seedA ; horis:isAssigned true ] ;
                horis:then [ horis:property :x ; horis:value "A" ] .
            :priorB a horis:Rule ; horis:when [ horis:subject :Patient ; horis:property :seedB ; horis:isAssigned true ] ;
                horis:then [ horis:property :x ; horis:value "B" ] .
            :source a horis:Rule ; horis:when [ horis:subject :Patient ; horis:property :y ; horis:isAssigned true ] ;
                horis:then [ horis:property :x ; horis:value "C" ] .
            :target a horis:Rule ; horis:when [ horis:all (
                [ horis:subject :Patient ; horis:property :x ; horis:is "A" ]
                [ horis:subject :Patient ; horis:property :x ; horis:is "B" ]
                [ horis:subject :Patient ; horis:property :x ; horis:is "C" ]
            ) ] ; horis:then [ horis:property :y ; horis:value true ] .
            "#,
        );
        assert!(!multi_value_enabling.terminating());
    }

    #[test]
    fn full_bdd_reports_repeat_byte_identically() {
        let onto = load(&format!("{TRIAGE_DOMAIN}{TRIAGE_RULES_WITH_GAP}"));
        let first = verify_ontology(&onto, AssumptionManifest::default());
        let second = verify_ontology(&onto, AssumptionManifest::default());
        assert_eq!(
            first.to_json().expect("JSON"),
            second.to_json().expect("JSON")
        );
    }
}

#[cfg(test)]
mod workflow_verification_tests {
    use super::{AssumptionManifest, FindingKind, verify_ontology};
    use crate::error::OntologyError;
    use crate::{Ontology, Value, World};

    const PRELUDE: &str = "@prefix : <https://example.org/workflow-verify#> .\n\
        @prefix horis: <https://w3id.org/horismos#> .\n\
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n";

    fn report(body: &str) -> super::VerificationReport {
        let ontology =
            Ontology::load_from_str_allow_incomplete(&format!("{PRELUDE}{body}"), |_| None)
                .expect("workflow verification fixture must parse");
        verify_ontology(&ontology, AssumptionManifest::default())
    }

    #[test]
    fn workflow_reachability_and_termination_pass_and_fail() {
        let passing = report(
            r#"
            :Patient a rdfs:Class .
            :Flow a horis:Workflow ; horis:startsAt :Start .
            :Start a horis:PerceptionStep ; horis:extracts :Patient ;
                horis:confidence 0.8 ; horis:fallback :Done ; horis:next :Done .
            :Done a horis:Step ; horis:terminal true .
            "#,
        );
        let flow = &passing.workflows()[":Flow"];
        assert!(flow.terminating && flow.fallbacks_total);
        assert!(passing.ok());

        let unreachable = report(
            r#"
            :Patient a rdfs:Class .
            :Flow a horis:Workflow ; horis:startsAt :Done .
            :Done a horis:Step ; horis:terminal true .
            :Orphan a horis:Step ; horis:terminal true .
            "#,
        );
        assert!(!unreachable.ok());
        assert_eq!(
            unreachable.workflows()[":Flow"],
            super::WorkflowVerification::default()
        );
        assert!(unreachable.witnesses().iter().any(|w| {
            w.kind == FindingKind::WorkflowReachability
                && w.domain_text
                    == ":Orphan is not reachable from any horis:Workflow horis:startsAt"
        }));

        let isolated = report(
            r#"
            :Patient a rdfs:Class .
            :Alpha a horis:Workflow ; horis:startsAt :AlphaDone .
            :AlphaDone a horis:Step ; horis:terminal true .
            :Beta a horis:Workflow ; horis:startsAt :BetaDone .
            :BetaDone a horis:Step ; horis:terminal true .
            :GlobalOrphan a horis:Step ; horis:terminal true .
            "#,
        );
        assert!(
            isolated
                .workflows()
                .values()
                .all(|workflow| *workflow == super::WorkflowVerification::default())
        );
        assert_eq!(
            isolated
                .witnesses()
                .iter()
                .filter(|w| w.kind == FindingKind::WorkflowReachability)
                .count(),
            1
        );

        let cycle = report(
            r#"
            :Patient a rdfs:Class .
            :result a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string .
            :Flow a horis:Workflow ; horis:startsAt :A .
            :A a horis:NarrationStep ; horis:narrates [ horis:property :result ] ; horis:next :B .
            :B a horis:NarrationStep ; horis:narrates [ horis:property :result ] ; horis:next :A .
            "#,
        );
        assert!(!cycle.workflows()[":Flow"].terminating);
    }

    #[test]
    fn workflow_requires_fallback_and_choice_checks_fail_independently() {
        let missing_fallback = report(
            r#"
            :Patient a rdfs:Class .
            :Flow a horis:Workflow ; horis:startsAt :Start .
            :Start a horis:PerceptionStep ; horis:extracts :Patient ;
                horis:confidence 0.8 ; horis:next :Done .
            :Done a horis:Step ; horis:terminal true .
            "#,
        );
        assert!(!missing_fallback.workflows()[":Flow"].fallbacks_total);

        let unsatisfied = report(
            r#"
            :Patient a rdfs:Class .
            :score a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:integer ; horis:min 0 ; horis:max 5 .
            :effect a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string .
            :tool a horis:Tool ; horis:effect [ horis:asserts [ horis:property :effect ; horis:type xsd:string ] ] ; horis:binding "rust:test" .
            :Flow a horis:Workflow ; horis:startsAt :Act .
            :Act a horis:ActionStep ; horis:tool :tool ;
                horis:requires [ horis:property :score ; horis:exceeds 10 ] ;
                horis:onError :Done ; horis:onUnsatisfied :Done ; horis:next :Done .
            :Done a horis:Step ; horis:terminal true .
            "#,
        );
        assert!(!unsatisfied.workflows()[":Flow"].requires_satisfiable);

        let choice = report(
            r#"
            :Patient a rdfs:Class .
            :flag a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean .
            :Flow a horis:Workflow ; horis:startsAt :Choose .
            :Choose a horis:ChoiceStep ; horis:branches (
                [ horis:when [ horis:property :flag ; horis:is true ] ; horis:next :Done ]
            ) .
            :Done a horis:Step ; horis:terminal true .
            "#,
        );
        assert!(!choice.workflows()[":Flow"].choices_total);
    }

    #[test]
    fn workflow_only_string_and_numeric_values_are_discretized() {
        let source = format!(
            "{PRELUDE}{}",
            r#"
            :Patient a rdfs:Class .
            :status a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string ; horis:cardinality "1" .
            :score a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:integer ;
                horis:min 0 ; horis:max 10 ; horis:cardinality "1" .
            :effect a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string .
            :tool a horis:Tool ; horis:effect [ horis:asserts [ horis:property :effect ; horis:type xsd:string ] ] ; horis:binding "rust:test" .
            :StringFlow a horis:Workflow ; horis:startsAt :StringAct .
            :StringAct a horis:ActionStep ; horis:tool :tool ;
                horis:requires [ horis:property :status ; horis:is "ready" ] ;
                horis:onError :StringDone ; horis:onUnsatisfied :StringDone ; horis:next :StringDone .
            :StringDone a horis:Step ; horis:terminal true .
            :NumericFlow a horis:Workflow ; horis:startsAt :NumericAct .
            :NumericAct a horis:ActionStep ; horis:tool :tool ;
                horis:requires [ horis:property :score ; horis:exceeds 5 ] ;
                horis:onError :NumericDone ; horis:onUnsatisfied :NumericDone ; horis:next :NumericDone .
            :NumericDone a horis:Step ; horis:terminal true .
            :ChoiceFlow a horis:Workflow ; horis:startsAt :Choose .
            :Choose a horis:ChoiceStep ; horis:branches (
                [ horis:when [ horis:property :status ; horis:is "ready" ] ; horis:next :ChoiceDone ]
                [ horis:when [ horis:property :status ; horis:isNot "ready" ] ; horis:next :ChoiceDone ]
            ) .
            :ChoiceDone a horis:Step ; horis:terminal true .
            "#
        );
        let ontology = Ontology::load_from_str(&source, |_| None)
            .expect("workflow-only authored values are valid projection boundaries");
        let report = ontology.verify();
        assert!(report.workflows()[":StringFlow"].requires_satisfiable);
        assert!(report.workflows()[":NumericFlow"].requires_satisfiable);
        assert!(report.workflows()[":ChoiceFlow"].choices_total);
    }

    #[test]
    fn workflow_conditions_use_only_rule_established_derived_facts() {
        let domain = r#"
            :Patient a rdfs:Class .
            :Decision a rdfs:Class ; horis:oneOf ( :Yes :No ) .
            :flag a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
                horis:cardinality "1" .
            :decision a rdf:Property ; rdfs:domain :Patient ; rdfs:range :Decision ;
                horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
            :effect a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string .
            :tool a horis:Tool ; horis:effect [ horis:asserts [ horis:property :effect ; horis:type xsd:string ] ] ; horis:binding "rust:test" .
            :Flow a horis:Workflow ; horis:startsAt :Act .
            :Act a horis:ActionStep ; horis:tool :tool ;
                horis:requires [ horis:property :decision ; horis:isAssigned true ] ;
                horis:onError :Choose ; horis:onUnsatisfied :Choose ; horis:next :Choose .
            :Choose a horis:ChoiceStep ; horis:branches (
                [ horis:when [ horis:property :decision ; horis:is :Yes ] ; horis:next :Done ]
            ) .
            :Done a horis:Step ; horis:terminal true .
        "#;
        let missing = report(domain);
        assert!(!missing.workflows()[":Flow"].requires_satisfiable);
        assert!(!missing.workflows()[":Flow"].choices_total);

        let establishing_rule = r#"
            :establish a horis:Rule ;
                horis:when [ horis:subject :Patient ; horis:property :flag ; horis:isAssigned true ] ;
                horis:then [ horis:property :decision ; horis:value :Yes ] .
        "#;
        let established = report(&format!("{domain}{establishing_rule}"));
        assert!(established.workflows()[":Flow"].requires_satisfiable);
        assert!(established.workflows()[":Flow"].choices_total);
    }

    #[test]
    fn workflow_conditions_follow_derived_from_derived_support() {
        let domain = r#"
            :Patient a rdfs:Class .
            :Decision a rdfs:Class ; horis:oneOf ( :Yes :No ) .
            :flag a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
                horis:cardinality "1" .
            :intermediate a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
                horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
            :decision a rdf:Property ; rdfs:domain :Patient ; rdfs:range :Decision ;
                horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
            :deriveDecision a horis:Rule ;
                horis:when [ horis:subject :Patient ; horis:property :intermediate ; horis:isAssigned true ] ;
                horis:then [ horis:property :decision ; horis:value :Yes ] .
            :effect a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string .
            :tool a horis:Tool ; horis:effect [ horis:asserts [ horis:property :effect ; horis:type xsd:string ] ] ; horis:binding "rust:test" .
            :Flow a horis:Workflow ; horis:startsAt :Act .
            :Act a horis:ActionStep ; horis:tool :tool ;
                horis:requires [ horis:property :decision ; horis:isAssigned true ] ;
                horis:onError :Done ; horis:onUnsatisfied :Done ; horis:next :Done .
            :Done a horis:Step ; horis:terminal true .
        "#;
        let missing_upstream = report(domain);
        assert!(!missing_upstream.workflows()[":Flow"].requires_satisfiable);

        let upstream = r#"
            :deriveIntermediate a horis:Rule ;
                horis:when [ horis:subject :Patient ; horis:property :flag ; horis:isAssigned true ] ;
                horis:then [ horis:property :intermediate ; horis:value true ] .
        "#;
        let established = report(&format!("{domain}{upstream}"));
        assert!(established.workflows()[":Flow"].requires_satisfiable);
    }

    #[test]
    fn higher_priority_support_blocks_a_lower_priority_negative_rule() {
        let report = report(
            r#"
            :Patient a rdfs:Class .
            :flag a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ; horis:cardinality "1" .
            :intermediate a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
                horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
            :decision a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
                horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
            :establishFirst a horis:Rule ; horis:priority 10 ;
                horis:when [ horis:subject :Patient ; horis:property :flag ; horis:isAssigned true ] ;
                horis:then [ horis:property :intermediate ; horis:value true ] .
            :negativeLater a horis:Rule ; horis:priority 0 ;
                horis:when [ horis:subject :Patient ; horis:property :intermediate ; horis:isEmpty true ] ;
                horis:then [ horis:property :decision ; horis:value true ] .
            :effect a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string .
            :tool a horis:Tool ; horis:effect [ horis:asserts [ horis:property :effect ; horis:type xsd:string ] ] ; horis:binding "rust:test" .
            :Flow a horis:Workflow ; horis:startsAt :Act .
            :Act a horis:ActionStep ; horis:tool :tool ;
                horis:requires [ horis:property :decision ; horis:isAssigned true ] ;
                horis:onError :Done ; horis:onUnsatisfied :Done ; horis:next :Done .
            :Done a horis:Step ; horis:terminal true .
            "#,
        );
        assert!(!report.workflows()[":Flow"].requires_satisfiable);
        assert!(report.dead_rules().contains(&":negativeLater".to_owned()));
    }

    #[test]
    fn newly_enabled_higher_priority_support_restarts_before_negative_rules() {
        let report = report(
            r#"
            :Patient a rdfs:Class .
            :flag a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ; horis:cardinality "1" .
            :x a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ; horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
            :y a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ; horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
            :decision a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ; horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
            :high a horis:Rule ; horis:priority 10 ;
                horis:when [ horis:subject :Patient ; horis:property :x ; horis:isAssigned true ] ;
                horis:then [ horis:property :y ; horis:value true ] .
            :middle a horis:Rule ; horis:priority 5 ;
                horis:when [ horis:subject :Patient ; horis:property :flag ; horis:isAssigned true ] ;
                horis:then [ horis:property :x ; horis:value true ] .
            :negative a horis:Rule ; horis:priority 0 ;
                horis:when [ horis:subject :Patient ; horis:property :y ; horis:isEmpty true ] ;
                horis:then [ horis:property :decision ; horis:value true ] .
            :effect a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string .
            :tool a horis:Tool ; horis:effect [ horis:asserts [ horis:property :effect ; horis:type xsd:string ] ] ; horis:binding "rust:test" .
            :Flow a horis:Workflow ; horis:startsAt :Act .
            :Act a horis:ActionStep ; horis:tool :tool ; horis:requires [ horis:property :decision ; horis:isAssigned true ] ;
                horis:onError :Done ; horis:onUnsatisfied :Done ; horis:next :Done .
            :Done a horis:Step ; horis:terminal true .
            "#,
        );
        assert!(!report.workflows()[":Flow"].requires_satisfiable);
        assert!(report.dead_rules().contains(&":negative".to_owned()));
    }

    #[test]
    fn workflow_requires_observes_only_the_standing_conflict_winner() {
        let report = report(
            r#"
            :Patient a rdfs:Class .
            :Decision a rdfs:Class ; horis:oneOf ( :A :B ) .
            :flag a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ; horis:cardinality "1" .
            :decision a rdf:Property ; rdfs:domain :Patient ; rdfs:range :Decision ;
                horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
            :downstream a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
                horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
            :high a horis:Rule ; horis:priority 10 ;
                horis:when [ horis:subject :Patient ; horis:property :flag ; horis:isAssigned true ] ;
                horis:then [ horis:property :decision ; horis:value :A ] .
            :low a horis:Rule ; horis:priority 5 ;
                horis:when [ horis:subject :Patient ; horis:property :flag ; horis:isAssigned true ] ;
                horis:then [ horis:property :decision ; horis:value :B ] .
            :usesLoser a horis:Rule ; horis:priority 0 ;
                horis:when [ horis:subject :Patient ; horis:property :decision ; horis:is :B ] ;
                horis:then [ horis:property :downstream ; horis:value true ] .
            :effect a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string .
            :tool a horis:Tool ; horis:effect [ horis:asserts [ horis:property :effect ; horis:type xsd:string ] ] ; horis:binding "rust:test" .
            :DirectFlow a horis:Workflow ; horis:startsAt :DirectAct .
            :DirectAct a horis:ActionStep ; horis:tool :tool ;
                horis:requires [ horis:property :decision ; horis:is :B ] ;
                horis:onError :DirectDone ; horis:onUnsatisfied :DirectDone ; horis:next :DirectDone .
            :DirectDone a horis:Step ; horis:terminal true .
            :ChainedFlow a horis:Workflow ; horis:startsAt :ChainedAct .
            :ChainedAct a horis:ActionStep ; horis:tool :tool ;
                horis:requires [ horis:property :downstream ; horis:isAssigned true ] ;
                horis:onError :ChainedDone ; horis:onUnsatisfied :ChainedDone ; horis:next :ChainedDone .
            :ChainedDone a horis:Step ; horis:terminal true .
            "#,
        );
        assert_eq!(report.conflicts().len(), 1);
        assert!(report.complete()[":decision"]);
        assert!(!report.workflows()[":DirectFlow"].requires_satisfiable);
        assert!(!report.workflows()[":ChainedFlow"].requires_satisfiable);
        assert!(report.dead_rules().contains(&":usesLoser".to_owned()));
    }

    #[test]
    fn immutable_value_set_before_a_late_high_priority_rule_remains_standing() {
        let body = r#"
            :Patient a rdfs:Class .
            :Decision a rdfs:Class ; horis:oneOf ( :A :B ) .
            :flag a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ; horis:cardinality "1" .
            :trigger a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
                horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
            :decision a rdf:Property ; rdfs:domain :Patient ; rdfs:range :Decision ;
                horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" ; horis:immutable true .
            :lateHigh a horis:Rule ; horis:priority 20 ;
                horis:when [ horis:subject :Patient ; horis:property :trigger ; horis:isAssigned true ] ;
                horis:then [ horis:property :decision ; horis:value :A ] .
            :firstLow a horis:Rule ; horis:priority 10 ;
                horis:when [ horis:subject :Patient ; horis:property :flag ; horis:isAssigned true ] ;
                horis:then [ horis:property :decision ; horis:value :B ] .
            :enableHigh a horis:Rule ; horis:priority 5 ;
                horis:when [ horis:subject :Patient ; horis:property :flag ; horis:isAssigned true ] ;
                horis:then [ horis:property :trigger ; horis:value true ] .
            :effect a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string .
            :tool a horis:Tool ; horis:effect [ horis:asserts [ horis:property :effect ; horis:type xsd:string ] ] ; horis:binding "rust:test" .
            :BFlow a horis:Workflow ; horis:startsAt :BAct .
            :BAct a horis:ActionStep ; horis:tool :tool ; horis:requires [ horis:property :decision ; horis:is :B ] ;
                horis:onError :BDone ; horis:onUnsatisfied :BDone ; horis:next :BDone .
            :BDone a horis:Step ; horis:terminal true .
            :AFlow a horis:Workflow ; horis:startsAt :AAct .
            :AAct a horis:ActionStep ; horis:tool :tool ; horis:requires [ horis:property :decision ; horis:is :A ] ;
                horis:onError :ADone ; horis:onUnsatisfied :ADone ; horis:next :ADone .
            :ADone a horis:Step ; horis:terminal true .
        "#;
        let source = format!("{PRELUDE}{body}");
        let ontology = Ontology::load_from_str_allow_incomplete(&source, |_| None)
            .expect("the explicit escape keeps both workflow reports inspectable");
        let report = ontology.verify();
        assert!(report.workflows()[":BFlow"].requires_satisfiable);
        assert!(!report.workflows()[":AFlow"].requires_satisfiable);
        assert!(report.witnesses().iter().any(|witness| {
            witness.kind == FindingKind::RuleVacuity
                && witness.domain_text.contains(":lateHigh is satisfiable")
        }));

        let patient = ontology.node_ref(":Patient").expect(":Patient");
        let flag = ontology.node_ref(":flag").expect(":flag");
        let decision = ontology.node_ref(":decision").expect(":decision");
        let expected = Value::Ref(crate::ontology::term::Ref::new(
            ontology.node_ref(":B").expect(":B"),
        ));
        let mut world = World::new(&ontology);
        let subject = world
            .assert_instance(patient, [(flag, Value::Boolean(true))])
            .expect("the base input and first immutable conclusion remain valid");
        assert_eq!(world.value(subject, decision), Some(&expected));
    }

    #[test]
    fn gate_rejected_conclusion_cannot_certify_completeness() {
        let body = r#"
            :Patient a rdfs:Class .
            :Acuity a rdfs:Class ; horis:oneOf ( :Good ) .
            :flag a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ; horis:cardinality "1" .
            :acuity a rdf:Property ; rdfs:domain :Patient ; rdfs:range :Acuity ;
                horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
            :invalid a horis:Rule ;
                horis:when [ horis:subject :Patient ; horis:property :flag ; horis:isAssigned true ] ;
                horis:then [ horis:property :acuity ; horis:value :Bad ] .
        "#;
        let source = format!("{PRELUDE}{body}");
        let error = Ontology::load_from_str(&source, |_| None)
            .expect_err("default-hard load must reject unrealizable coverage");
        let OntologyError::VerificationFailed { report } = error else {
            panic!("expected the complete verification report");
        };
        assert!(!report.complete()[":acuity"]);
        assert!(report.witnesses().iter().any(|witness| {
            witness.kind == FindingKind::RuleVacuity
                && witness.domain_text.contains(":invalid is satisfiable")
                && witness.domain_text.contains(":acuity :Bad")
        }));

        let ontology = Ontology::load_from_str_allow_incomplete(&source, |_| None)
            .expect("the explicit escape keeps the invalid ontology inspectable");
        let patient = ontology.node_ref(":Patient").expect(":Patient");
        let flag = ontology.node_ref(":flag").expect(":flag");
        let acuity = ontology.node_ref(":acuity").expect(":acuity");
        let mut world = World::new(&ontology);
        let subject = world
            .assert_instance(patient, [(flag, Value::Boolean(true))])
            .expect("the base input remains valid");
        assert_eq!(world.value(subject, acuity), None);
    }
}

#[cfg(test)]
mod load_verification_tests {
    use crate::error::OntologyError;
    use crate::{LoadOptions, Ontology};

    const INCOMPLETE: &str = "@prefix : <https://example.org/load-verify#> .\n\
        @prefix horis: <https://w3id.org/horismos#> .\n\
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
        :Patient a rdfs:Class .\n\
        :decision a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;\n\
            horis:assignedBy horis:RulesOnly .\n";

    #[test]
    fn ordinary_load_fails_closed_with_the_complete_report() {
        let error = Ontology::load_from_str(INCOMPLETE, |_| None)
            .expect_err("an incomplete rules-only property must block ordinary load");
        let OntologyError::VerificationFailed { report } = error else {
            panic!("expected VerificationFailed");
        };
        assert!(!report.ok());
        assert_eq!(report.exit_code(), 1);
        assert_eq!(report.complete().get(":decision"), Some(&false));
        assert_eq!(report.assumptions().input_closure, Vec::<String>::new());
    }

    #[test]
    fn explicit_allow_incomplete_keeps_the_same_report_inspectable() {
        let ontology = Ontology::load_from_str_with_options(
            INCOMPLETE,
            |_| None,
            LoadOptions {
                allow_incomplete: true,
            },
        )
        .expect("the explicit authoring escape returns the ontology");
        let first = ontology.verify();
        let second = ontology.verify();
        assert!(!first.ok());
        assert_eq!(
            first.to_json().expect("JSON"),
            second.to_json().expect("JSON")
        );
    }
}
