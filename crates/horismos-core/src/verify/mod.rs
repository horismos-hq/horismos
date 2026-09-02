//! Verification: BDD checks with witnesses, run before an ontology executes.
//!
//! Rule-conflict detection, workflow termination and reachability proofs,
//! constraint-domain analysis. Port of the sigil-lang verifier.
//!
//! Spec: `docs/framework/10-verification.md` · Owner: PRP-005 (schema checks
//! land earlier with PRP-002)

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

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
