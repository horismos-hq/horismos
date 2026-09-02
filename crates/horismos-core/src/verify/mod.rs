//! Verification: BDD checks with witnesses, run before an ontology executes.
//!
//! Rule-conflict detection, workflow termination and reachability proofs,
//! constraint-domain analysis. Port of the sigil-lang verifier.
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

/// One endpoint of a numeric witness interval.
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
    pub domain_text: String,
    pub region: Region,
    pub suggestion: Option<String>,
}

/// One overlapping pair of contradictory rule conclusions.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Conflict {
    pub first_rule: String,
    pub second_rule: String,
    pub property: String,
    pub first_value: String,
    pub second_value: String,
    pub region: Region,
}

/// Per-workflow summary; detail remains in [`VerificationReport::witnesses`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowVerification {
    pub reachable: bool,
    pub terminating: bool,
    pub requires_satisfiable: bool,
    pub fallbacks_total: bool,
    pub choices_total: bool,
}

impl Default for WorkflowVerification {
    fn default() -> Self {
        Self {
            reachable: true,
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
        let ok = self.witnesses.is_empty()
            && self.complete.values().all(|complete| *complete)
            && self.conflicts.is_empty()
            && self.dead_rules.is_empty()
            && self.terminating
            && self.workflows.values().all(|workflow| {
                workflow.reachable
                    && workflow.terminating
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
        AssumptionManifest, Bound, FindingKind, Region, RegionPredicate, ReportBuilder,
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
            domain_text: ":Orphan is unreachable from :Intake".into(),
            region: Region::default(),
            suggestion: None,
        });
        builder.witnesses.push(Witness {
            kind: FindingKind::Completeness,
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
                    domain_text: "same text".into(),
                    region: Region {
                        subject_class: Some(":Patient".into()),
                        predicates,
                    },
                    suggestion: Some("repair b".into()),
                },
                Witness {
                    kind: FindingKind::Completeness,
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
    use super::{AssumptionManifest, FindingKind, RegionPredicate, verify_ontology};
    use crate::Ontology;

    const PRELUDE: &str = "@prefix : <https://example.org/verify#> .\n\
        @prefix horis: <https://w3id.org/horismos#> .\n\
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n";

    fn load(body: &str) -> Ontology {
        Ontology::load_from_str(&format!("{PRELUDE}{body}"), |_| None)
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
        let result = report(&format!("{TRIAGE_DOMAIN}{TRIAGE_RULES_WITH_GAP}"));
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
        let texts: Vec<&str> = result
            .witnesses()
            .iter()
            .filter(|w| w.kind == FindingKind::ConstraintVacuity)
            .map(|w| w.domain_text.as_str())
            .collect();
        assert_eq!(texts.len(), 2);
        assert!(texts.iter().any(|text| text.contains("code")));
        assert!(texts.iter().any(|text| text.contains("tags")));
        assert!(!texts.iter().any(|text| text.contains("restricted")));
        assert!(!texts.iter().any(|text| text.contains("spaced")));
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
