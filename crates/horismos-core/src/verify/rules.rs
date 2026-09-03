//! Bounded symbolic verification of authored rules.
//!
//! The verifier discretizes only ontology-authored boundaries and values,
//! then represents the resulting finite projection as BDDs. It neither
//! converts units nor claims anything outside the report's assumptions.
//!
//! Spec: `docs/framework/10-verification.md` · Owner: PRP-005

use std::collections::{BTreeMap, BTreeSet};

use biodivine_lib_bdd::{Bdd, BddVariable, BddVariableSet, BddVariableSetBuilder};
use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;

use super::{
    AssumptionManifest, Bound, Conflict, FindingKind, Region, RegionPredicate, ReportBuilder,
    Severity, VerificationReport, Witness, WorkflowVerification,
};
use crate::Ontology;
use crate::ontology::schema::{AssignedBy, Cardinality, PropertyDecl, PropertyRange};
use crate::ontology::term::{Datatype, Iri, Value};
use crate::reason::eval::compare as compare_values;
use crate::reason::rule::{Condition, ConditionTree, Op, Rule, Term};
use crate::workflow::model::{Step, StepBody};

/// An authored constant that the runtime orders with `compare`: numbers
/// cross-type, `xsd:dateTime` by instant, and strings lexicographically.
/// Each kind is discretized in its own opaque order; nothing is converted.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Threshold {
    Number(Decimal),
    DateTime(DateTime<FixedOffset>),
    Text(String),
}

impl Threshold {
    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Decimal(value) => Some(Self::Number(*value)),
            Value::Integer(value) => Some(Self::Number(Decimal::from(*value))),
            Value::DateTime(value) => Some(Self::DateTime(*value)),
            Value::String(value) => Some(Self::Text(value.clone())),
            Value::Boolean(_) | Value::Duration(_) | Value::Ref(_) => None,
        }
    }

    fn lexical(&self) -> String {
        match self {
            Self::Number(value) => value.to_string(),
            Self::DateTime(value) => value.to_rfc3339(),
            Self::Text(value) => value.clone(),
        }
    }
}

/// The threshold an authored constant contributes to `property`, if the
/// runtime could ever order a stored value of that property against it.
fn threshold_for(range: Option<PropertyRange>, value: &Value) -> Option<Threshold> {
    let threshold = Threshold::from_value(value)?;
    let comparable = match range {
        None => true,
        Some(PropertyRange::Class(_)) => false,
        Some(PropertyRange::Datatype(datatype)) => matches!(
            (&threshold, datatype),
            (Threshold::Number(_), Datatype::Decimal | Datatype::Integer)
                | (Threshold::DateTime(_), Datatype::DateTime)
                | (Threshold::Text(_), Datatype::String)
        ),
    };
    comparable.then_some(threshold)
}

/// `horis:min`/`horis:max` as the gate applies them: numeric bounds on a
/// numerically comparable property; anything else constrains no value.
fn gate_bound(property: &PropertyDecl, bound: Option<&Value>) -> Option<Threshold> {
    threshold_for(property.range, bound?)
        .filter(|threshold| matches!(threshold, Threshold::Number(_)))
}

#[derive(Debug, Clone, Copy)]
struct OrderedVars {
    greater: BddVariable,
    at_least: BddVariable,
}

#[derive(Debug)]
struct PropertyVars {
    assigned: BddVariable,
    ordered: BTreeMap<Threshold, OrderedVars>,
    atoms: BTreeMap<String, BddVariable>,
    atom_positions: BTreeMap<String, Threshold>,
    contains: BTreeMap<String, BddVariable>,
    string_atoms: BTreeSet<String>,
}

#[derive(Debug)]
struct SymbolicSpace<'a> {
    ontology: &'a Ontology,
    variables: BddVariableSet,
    class_vars: BTreeMap<Iri, BddVariable>,
    property_vars: BTreeMap<Iri, PropertyVars>,
    reachable: Bdd,
}

/// Three distinct runtime moments for each rule. A satisfiable condition may
/// still be rejected by the validation gate, and an accepted conclusion may
/// later lose a single-valued conflict. Keeping those states separate avoids
/// certifying a fact that is not standing at the runtime fixed point.
struct RuleFormulas {
    candidates: Vec<Bdd>,
    accepted: Vec<Bdd>,
    standing: Vec<Bdd>,
}

/// Verify all rule guarantees under a caller-supplied bounded-projection
/// assumption manifest.
pub fn verify_ontology(ontology: &Ontology, assumptions: AssumptionManifest) -> VerificationReport {
    let has_rules_only_property = ontology
        .schema()
        .properties()
        .any(|property| property.assigned_by == AssignedBy::RulesOnly);
    if ontology.rules().is_empty() && ontology.workflows().is_empty() && !has_rules_only_property {
        let mut report = ReportBuilder::new(assumptions);
        verify_constraint_vacuity(ontology, &mut report);
        return report.finish();
    }
    let space = SymbolicSpace::new(ontology);
    let formulas = space.established_rule_formulas();
    let mut report = ReportBuilder::new(assumptions);

    verify_completeness(&space, &formulas.standing, &mut report);
    verify_conflicts(&space, &formulas.accepted, &mut report);
    verify_dead_and_vacuous_rules(
        &space,
        &formulas.candidates,
        &formulas.accepted,
        &mut report,
    );
    verify_rule_termination(ontology, &mut report);
    verify_constraint_vacuity(ontology, &mut report);
    verify_workflows(&space, &formulas.standing, &mut report);

    report.finish()
}

impl<'a> SymbolicSpace<'a> {
    fn new(ontology: &'a Ontology) -> Self {
        let mut collected = CollectedValues::default();

        for property in ontology.schema().properties() {
            for bound in [property.min.as_ref(), property.max.as_ref()] {
                if let Some(threshold) = gate_bound(property, bound) {
                    collected
                        .thresholds
                        .entry(property.iri)
                        .or_default()
                        .insert(threshold);
                }
            }
            match property.range {
                Some(PropertyRange::Class(range)) => {
                    let property_atoms = collected.atoms.entry(property.iri).or_default();
                    if let Some(members) = ontology
                        .schema()
                        .class(range)
                        .and_then(|c| c.one_of.as_ref())
                    {
                        for &member in members {
                            property_atoms.insert(ontology.resolve(member).to_owned());
                        }
                    } else {
                        for subclass in ontology.schema().subclasses(range) {
                            property_atoms.insert(ontology.resolve(subclass).to_owned());
                        }
                    }
                }
                Some(PropertyRange::Datatype(Datatype::Boolean)) => {
                    let property_atoms = collected.atoms.entry(property.iri).or_default();
                    for value in [true, false] {
                        property_atoms
                            .insert(term_text(ontology, &Term::Literal(Value::Boolean(value))));
                    }
                }
                _ => {}
            }
        }
        for rule in ontology.rules() {
            for condition in rule.when.leaves() {
                collect_condition_values(ontology, condition, &mut collected);
            }
        }
        // Workflow predicates share the same bounded input projection as
        // rule predicates. Collect their authored constants before creating
        // BDD variables; otherwise a value used only by `horis:requires`, a
        // reasoning goal, or a choice branch would be mistaken for an
        // impossible value.
        for workflow in ontology.workflows().values() {
            for step in workflow.steps.values() {
                let trees: Vec<&ConditionTree> = match &step.body {
                    StepBody::Reasoning(body) => vec![&body.goal],
                    StepBody::Action(body) => body.requires.iter().collect(),
                    StepBody::Choice(body) => {
                        body.branches.iter().map(|branch| &branch.when).collect()
                    }
                    _ => Vec::new(),
                };
                for tree in trees {
                    for condition in tree.leaves() {
                        collect_condition_values(ontology, condition, &mut collected);
                    }
                }
            }
        }
        let CollectedValues {
            thresholds,
            atoms,
            mut atom_positions,
            contains,
            mut string_atoms,
        } = collected;

        let mut builder = BddVariableSetBuilder::new();
        let mut class_vars = BTreeMap::new();
        for (class_index, class) in ontology.schema().classes().enumerate() {
            class_vars.insert(
                class.iri,
                builder.make_variable(&format!("class_{class_index}")),
            );
        }
        let mut property_vars = BTreeMap::new();
        for (property_index, property) in ontology.schema().properties().enumerate() {
            let assigned = builder.make_variable(&format!("p{property_index}_assigned"));
            let mut ordered = BTreeMap::new();
            for (index, threshold) in thresholds
                .get(&property.iri)
                .into_iter()
                .flatten()
                .enumerate()
            {
                ordered.insert(
                    threshold.clone(),
                    OrderedVars {
                        greater: builder.make_variable(&format!("p{property_index}_n{index}_gt")),
                        at_least: builder.make_variable(&format!("p{property_index}_n{index}_ge")),
                    },
                );
            }
            let mut exact = BTreeMap::new();
            for (index, atom) in atoms.get(&property.iri).into_iter().flatten().enumerate() {
                exact.insert(
                    atom.clone(),
                    builder.make_variable(&format!("p{property_index}_a{index}")),
                );
            }
            let mut substring = BTreeMap::new();
            for (index, needle) in contains
                .get(&property.iri)
                .into_iter()
                .flatten()
                .enumerate()
            {
                substring.insert(
                    needle.clone(),
                    builder.make_variable(&format!("p{property_index}_c{index}")),
                );
            }
            property_vars.insert(
                property.iri,
                PropertyVars {
                    assigned,
                    ordered,
                    atoms: exact,
                    atom_positions: atom_positions.remove(&property.iri).unwrap_or_default(),
                    contains: substring,
                    string_atoms: string_atoms.remove(&property.iri).unwrap_or_default(),
                },
            );
        }

        let variables = builder.build();
        let mut reachable = variables.mk_true();
        let all_classes: Vec<BddVariable> = class_vars.values().copied().collect();
        if !all_classes.is_empty() {
            reachable = reachable.and(&variables.mk_sat_exactly_k(1, &all_classes));
        }

        let mut space = Self {
            ontology,
            variables,
            class_vars,
            property_vars,
            reachable,
        };
        space.add_schema_constraints();
        space
    }

    fn add_schema_constraints(&mut self) {
        for property in self.ontology.schema().properties() {
            let Some(vars) = self.property_vars.get(&property.iri) else {
                continue;
            };
            let assigned = self.variables.mk_var(vars.assigned);
            let subject_in_domain = property.domain.map_or_else(
                || self.variables.mk_true(),
                |domain| self.subject_is(domain),
            );
            self.reachable = self.reachable.and(&assigned.imp(&subject_in_domain));
            if property
                .cardinality
                .is_some_and(Cardinality::requires_a_value)
            {
                self.reachable = self.reachable.and(&subject_in_domain.imp(&assigned));
            }

            if !vars.ordered.is_empty() {
                for ordered in vars.ordered.values() {
                    self.reachable = self
                        .reachable
                        .and(&self.variables.mk_var(ordered.greater).imp(&assigned));
                    self.reachable = self
                        .reachable
                        .and(&self.variables.mk_var(ordered.at_least).imp(&assigned));
                }
                let allowed = allowed_signatures(property, &vars.ordered, &self.variables);
                self.reachable = self.reachable.and(&assigned.imp(&allowed));
            }

            let exact: Vec<BddVariable> = vars.atoms.values().copied().collect();
            for atom in &exact {
                self.reachable = self
                    .reachable
                    .and(&self.variables.mk_var(*atom).imp(&assigned));
            }
            for (atom, position) in &vars.atom_positions {
                let Some(atom_variable) = vars.atoms.get(atom) else {
                    continue;
                };
                let atom_bdd = self.variables.mk_var(*atom_variable);
                let single_valued = property
                    .cardinality
                    .is_some_and(Cardinality::is_single_valued);
                for (threshold, ordered) in &vars.ordered {
                    for (holds, variable) in [
                        (position > threshold, ordered.greater),
                        (position >= threshold, ordered.at_least),
                    ] {
                        if holds || single_valued {
                            self.reachable = self
                                .reachable
                                .and(&atom_bdd.imp(&self.variables.mk_literal(variable, holds)));
                        }
                    }
                }
            }
            for variable in vars.contains.values() {
                self.reachable = self
                    .reachable
                    .and(&self.variables.mk_var(*variable).imp(&assigned));
            }
            for (needle, variable) in &vars.contains {
                for (other_needle, other_variable) in &vars.contains {
                    if needle.contains(other_needle) {
                        self.reachable = self.reachable.and(
                            &self
                                .variables
                                .mk_var(*variable)
                                .imp(&self.variables.mk_var(*other_variable)),
                        );
                    }
                }
                for atom in &vars.string_atoms {
                    let Some(atom_variable) = vars.atoms.get(atom) else {
                        continue;
                    };
                    let atom_bdd = self.variables.mk_var(*atom_variable);
                    let contains_bdd = self.variables.mk_var(*variable);
                    let relation = if atom.contains(needle) {
                        atom_bdd.imp(&contains_bdd)
                    } else if property
                        .cardinality
                        .is_some_and(Cardinality::is_single_valued)
                    {
                        atom_bdd.imp(&contains_bdd.not())
                    } else {
                        self.variables.mk_true()
                    };
                    self.reachable = self.reachable.and(&relation);
                }
            }
            if property
                .cardinality
                .is_some_and(Cardinality::is_single_valued)
                && exact.len() > 1
            {
                self.reachable = self
                    .reachable
                    .and(&self.variables.mk_sat_up_to_k(1, &exact));
            }
            let closed_value_space = matches!(
                property.range,
                Some(PropertyRange::Class(_) | PropertyRange::Datatype(Datatype::Boolean))
            );
            if closed_value_space && !exact.is_empty() {
                let some_atom = exact
                    .iter()
                    .fold(self.variables.mk_false(), |bdd, variable| {
                        bdd.or(&self.variables.mk_var(*variable))
                    });
                self.reachable = self.reachable.and(&assigned.imp(&some_atom));
            }
        }
    }

    fn subject_is(&self, class: Iri) -> Bdd {
        self.class_vars
            .iter()
            .filter(|(actual, _)| self.ontology.schema().is_subclass_of(**actual, class))
            .fold(self.variables.mk_false(), |bdd, (_, var)| {
                bdd.or(&self.variables.mk_var(*var))
            })
    }

    fn established_rule_formulas(&self) -> RuleFormulas {
        let mut candidates = vec![self.variables.mk_false(); self.ontology.rules().len()];
        let mut accepted = vec![self.variables.mk_false(); self.ontology.rules().len()];
        // Accepted rule conclusions only add support while chaining. Starting
        // with every RulesOnly fact absent and unioning each round computes
        // the least firing fixed point, including derived chains. Candidate
        // conditions are retained separately because the gate can make a
        // satisfiable rule inert.
        let rule_count = self.ontology.rules().len();
        let max_updates = rule_count
            .saturating_mul(rule_count)
            .saturating_add(rule_count)
            .saturating_add(1);
        let mut updates = 0;
        let mut index = 0;
        while index < rule_count && updates < max_updates {
            // The runtime fires one candidate, then restarts its priority
            // scan. Mirror that restart whenever any symbolic support grows.
            let rule = &self.ontology.rules()[index];
            let standing_now = self.standing_rule_formulas(&accepted);
            let candidate = self
                .reachable
                .and(&self.subject_is(rule.subject_class))
                .and(&self.condition_tree_after_rules(&rule.when, &standing_now));
            let next_candidate = candidates[index].or(&candidate);
            let admitted = self.admitted_candidate(index, candidate, &standing_now);
            let next_accepted = accepted[index].or(&admitted);
            if next_candidate != candidates[index] || next_accepted != accepted[index] {
                candidates[index] = next_candidate;
                accepted[index] = next_accepted;
                updates += 1;
                index = 0;
            } else {
                index += 1;
            }
        }

        // The firing fixed point above is monotone, like the runtime's
        // no-re-fire history. Workflow conditions observe the final store,
        // though: for a single-valued property, an earlier rule in the
        // deterministic priority order suppresses every later conflicting
        // conclusion in the region where both were accepted. A
        // higher-priority rule enabled late is already present in `accepted`
        // and therefore removes the lower value here, matching override.
        let standing = self.standing_rule_formulas(&accepted);

        RuleFormulas {
            candidates,
            accepted,
            standing,
        }
    }

    fn standing_rule_formulas(&self, accepted: &[Bdd]) -> Vec<Bdd> {
        self.ontology
            .rules()
            .iter()
            .enumerate()
            .map(|(index, rule)| {
                let property = self.ontology.schema().property(rule.then.property);
                let replaceable_single_value = property.is_some_and(|property| {
                    !property.immutable
                        && property
                            .cardinality
                            .is_some_and(Cardinality::is_single_valued)
                });
                if !replaceable_single_value {
                    return accepted[index].clone();
                }
                let winning_conflict = self
                    .ontology
                    .rules()
                    .iter()
                    .zip(accepted)
                    .take(index)
                    .filter(|(winner, _)| {
                        winner.then.property == rule.then.property
                            && !terms_equal(self.ontology, &winner.then.value, &rule.then.value)
                    })
                    .fold(self.variables.mk_false(), |bdd, (_, formula)| {
                        bdd.or(formula)
                    });
                accepted[index].and(&winning_conflict.not())
            })
            .collect()
    }

    /// Restrict a satisfiable rule condition to the region where the
    /// conclusion is actually admitted by the runtime gate. Most checks are
    /// static, but `horis:immutable` depends on the value already standing
    /// when this rule is considered. In particular, a higher-priority rule
    /// enabled after a lower-priority immutable assignment is rejected; it
    /// does not replace the earlier value.
    fn admitted_candidate(&self, index: usize, candidate: Bdd, standing: &[Bdd]) -> Bdd {
        let rule = &self.ontology.rules()[index];
        if !conclusion_passes_gate(self.ontology, rule) {
            return self.variables.mk_false();
        }
        let Some(property) = self.ontology.schema().property(rule.then.property) else {
            return self.variables.mk_false();
        };
        if !property.immutable {
            return candidate;
        }

        let (mut held_any, mut held_same) = if property.assigned_by == AssignedBy::PerceptionAllowed
        {
            let vars = self
                .property_vars
                .get(&rule.then.property)
                .expect("every declared rule property has symbolic variables");
            let assigned = self.variables.mk_var(vars.assigned);
            (
                assigned.clone(),
                assigned.and(&self.term_matches(rule.then.property, &rule.then.value)),
            )
        } else {
            (self.variables.mk_false(), self.variables.mk_false())
        };

        for (other, formula) in self.ontology.rules().iter().zip(standing) {
            if other.then.property != rule.then.property {
                continue;
            }
            held_any = held_any.or(formula);
            if terms_equal(self.ontology, &other.then.value, &rule.then.value) {
                held_same = held_same.or(formula);
            }
        }

        // Re-asserting a value already held is a no-op even on a
        // multi-valued immutable property. Only a non-empty store without
        // the proposed value rejects the conclusion.
        let immutable_conflict = held_any.and(&held_same.not());
        candidate.and(&immutable_conflict.not())
    }

    /// A condition over the store at the rule fixed point: facts the input
    /// may carry (unless the property is `horis:RulesOnly`) together with
    /// every standing rule conclusion on the same property.
    fn condition_tree_after_rules(&self, tree: &ConditionTree, formulas: &[Bdd]) -> Bdd {
        match tree {
            ConditionTree::Leaf(condition) => {
                let rules_only = self
                    .ontology
                    .schema()
                    .property(condition.property)
                    .is_some_and(|property| property.assigned_by == AssignedBy::RulesOnly);
                let assigning = self
                    .ontology
                    .rules()
                    .iter()
                    .zip(formulas)
                    .filter(|(rule, _)| rule.then.property == condition.property);
                let derived_where = |keep: &dyn Fn(&Rule) -> bool| {
                    assigning
                        .clone()
                        .filter(|(rule, _)| keep(rule))
                        .fold(self.variables.mk_false(), |bdd, (_, formula)| {
                            bdd.or(formula)
                        })
                };
                let input = (!rules_only).then(|| self.condition(condition));
                match &condition.op {
                    Op::IsEmpty => {
                        let holds = derived_where(&|_| true).not();
                        input.map_or(holds.clone(), |input| holds.and(&input))
                    }
                    Op::IsNot(expected) => {
                        let holds = derived_where(&|rule| {
                            term_matches_static(self.ontology, &rule.then.value, expected)
                        })
                        .not();
                        input.map_or(holds.clone(), |input| holds.and(&input))
                    }
                    op => {
                        let holds = derived_where(&|rule| {
                            conclusion_satisfies(self.ontology, &rule.then.value, op)
                        });
                        input.map_or(holds.clone(), |input| holds.or(&input))
                    }
                }
            }
            ConditionTree::All(children) => children
                .iter()
                .fold(self.variables.mk_true(), |bdd, child| {
                    bdd.and(&self.condition_tree_after_rules(child, formulas))
                }),
            ConditionTree::Any(children) => children
                .iter()
                .fold(self.variables.mk_false(), |bdd, child| {
                    bdd.or(&self.condition_tree_after_rules(child, formulas))
                }),
            ConditionTree::NoneOf(children) => children
                .iter()
                .fold(self.variables.mk_false(), |bdd, child| {
                    bdd.or(&self.condition_tree_after_rules(child, formulas))
                })
                .not(),
        }
    }

    fn condition_subject_scope(&self, tree: &ConditionTree) -> Bdd {
        let domains: Vec<Iri> = tree
            .leaves()
            .iter()
            .filter_map(|condition| {
                self.ontology
                    .schema()
                    .property(condition.property)
                    .and_then(|property| property.domain)
            })
            .collect();
        if domains.is_empty() {
            return self.variables.mk_true();
        }
        domains
            .into_iter()
            .fold(self.variables.mk_false(), |scope, domain| {
                scope.or(&self.subject_is(domain))
            })
    }

    fn condition(&self, condition: &Condition) -> Bdd {
        let Some(vars) = self.property_vars.get(&condition.property) else {
            return self.variables.mk_false();
        };
        let assigned = self.variables.mk_var(vars.assigned);
        let property = condition.property;
        match &condition.op {
            Op::Exceeds(value) => assigned.and(&self.ordered(property, value, |v| v.greater)),
            Op::Below(value) => assigned.and(&self.ordered(property, value, |v| v.at_least).not()),
            Op::AtLeast(value) => assigned.and(&self.ordered(property, value, |v| v.at_least)),
            Op::AtMost(value) => assigned.and(&self.ordered(property, value, |v| v.greater).not()),
            Op::Between(low, high) => assigned
                .and(&self.ordered(property, low, |v| v.at_least))
                .and(&self.ordered(property, high, |v| v.greater).not()),
            Op::Is(term) => assigned.and(&self.term_matches(condition.property, term)),
            Op::Contains(Term::Literal(Value::String(needle)))
                if matches!(
                    self.ontology
                        .schema()
                        .property(condition.property)
                        .and_then(|p| p.range),
                    Some(PropertyRange::Datatype(Datatype::String))
                ) =>
            {
                assigned.and(&vars.contains.get(needle).map_or_else(
                    || self.variables.mk_false(),
                    |variable| self.variables.mk_var(*variable),
                ))
            }
            Op::Contains(term) => assigned.and(&self.term_matches(condition.property, term)),
            Op::IsNot(term) => self.term_matches(condition.property, term).not(),
            Op::IsAssigned => assigned,
            Op::IsEmpty => assigned.not(),
            Op::InList(terms) => {
                assigned.and(&terms.iter().fold(self.variables.mk_false(), |bdd, term| {
                    bdd.or(&self.term_matches(condition.property, term))
                }))
            }
        }
    }

    /// The boundary variable an ordered comparison against `value` reads.
    /// A constant the runtime cannot order against this property (a class
    /// reference, a duration, or a mismatched datatype) never holds.
    fn ordered(
        &self,
        property: Iri,
        value: &Value,
        select: impl FnOnce(OrderedVars) -> BddVariable,
    ) -> Bdd {
        let Some(vars) = self.property_vars.get(&property) else {
            return self.variables.mk_false();
        };
        let range = self
            .ontology
            .schema()
            .property(property)
            .and_then(|declared| declared.range);
        threshold_for(range, value)
            .and_then(|threshold| vars.ordered.get(&threshold).copied())
            .map_or_else(
                || self.variables.mk_false(),
                |boundary| self.variables.mk_var(select(boundary)),
            )
    }

    fn term_matches(&self, property: Iri, term: &Term) -> Bdd {
        let Some(vars) = self.property_vars.get(&property) else {
            return self.variables.mk_false();
        };
        let range = self
            .ontology
            .schema()
            .property(property)
            .and_then(|declared| declared.range);
        match term {
            Term::Literal(value) if as_decimal(value).is_some() => threshold_for(range, value)
                .and_then(|threshold| vars.ordered.get(&threshold))
                .map_or_else(
                    || self.variables.mk_false(),
                    |boundary| {
                        self.variables
                            .mk_var(boundary.at_least)
                            .and(&self.variables.mk_not_var(boundary.greater))
                    },
                ),
            Term::Iri(target) if self.ontology.schema().class_declared(*target).is_some() => vars
                .atoms
                .iter()
                .filter_map(|(atom, var)| {
                    let iri = self.ontology.iri(atom)?;
                    self.ontology
                        .schema()
                        .is_subclass_of(iri, *target)
                        .then_some(*var)
                })
                .fold(self.variables.mk_false(), |bdd, var| {
                    bdd.or(&self.variables.mk_var(var))
                }),
            _ => {
                let key = term_text(self.ontology, term);
                vars.atoms.get(&key).map_or_else(
                    || self.variables.mk_false(),
                    |var| self.variables.mk_var(*var),
                )
            }
        }
    }

    fn region(&self, formula: &Bdd, subject: Option<Iri>) -> Region {
        // Select the deterministic first disconnected component, but
        // generalize its total valuation before rendering. This avoids both
        // erasing predicates across a union of gaps and reporting arbitrary
        // don't-care bits from a raw valuation.
        let formula = first_prime_region(formula, &self.variables);
        let mut predicates = Vec::new();
        for property in self.ontology.schema().properties() {
            // Rules-only properties are symbolic outputs, not dimensions of
            // the externally reachable input region being explained.
            if property.assigned_by == AssignedBy::RulesOnly {
                continue;
            }
            let Some(vars) = self.property_vars.get(&property.iri) else {
                continue;
            };
            if !vars.ordered.is_empty() {
                let mut lower: Option<(Threshold, bool)> = None;
                let mut upper: Option<(Threshold, bool)> = None;
                for (threshold, boundary) in &vars.ordered {
                    if forced(&formula, &self.variables, boundary.greater, true) {
                        strongest_lower(&mut lower, threshold, false);
                    } else if forced(&formula, &self.variables, boundary.greater, false) {
                        strongest_upper(&mut upper, threshold, true);
                    }
                    if forced(&formula, &self.variables, boundary.at_least, true) {
                        strongest_lower(&mut lower, threshold, true);
                    } else if forced(&formula, &self.variables, boundary.at_least, false) {
                        strongest_upper(&mut upper, threshold, false);
                    }
                }
                if lower.is_some() || upper.is_some() {
                    predicates.push(RegionPredicate::Numeric {
                        property: local(self.ontology, property.iri),
                        lower: lower.map(|(value, inclusive)| Bound {
                            value: value.lexical(),
                            inclusive,
                        }),
                        upper: upper.map(|(value, inclusive)| Bound {
                            value: value.lexical(),
                            inclusive,
                        }),
                        unit: property.unit.clone(),
                    });
                    continue;
                }
            }

            let forced_true: Vec<&str> = vars
                .atoms
                .iter()
                .filter(|(_, var)| forced(&formula, &self.variables, **var, true))
                .map(|(atom, _)| atom.as_str())
                .collect();
            if let Some(atom) = forced_true.first() {
                predicates.push(RegionPredicate::Value {
                    property: local(self.ontology, property.iri),
                    value: display_atom(self.ontology, atom),
                });
            } else if !vars.atoms.is_empty()
                && vars
                    .atoms
                    .values()
                    .all(|var| forced(&formula, &self.variables, *var, false))
            {
                let value = match property.range {
                    Some(PropertyRange::Class(range)) => Some(compact_iri(self.ontology, range)),
                    _ => None,
                };
                predicates.push(RegionPredicate::Absent {
                    property: local(self.ontology, property.iri),
                    value,
                });
            } else if forced(&formula, &self.variables, vars.assigned, false) {
                predicates.push(RegionPredicate::Assigned {
                    property: local(self.ontology, property.iri),
                    assigned: false,
                });
            }
        }
        Region {
            subject_class: subject.map(|class| compact_iri(self.ontology, class)),
            predicates,
        }
    }
}

#[derive(Debug, Default)]
struct CollectedValues {
    thresholds: BTreeMap<Iri, BTreeSet<Threshold>>,
    atoms: BTreeMap<Iri, BTreeSet<String>>,
    atom_positions: BTreeMap<Iri, BTreeMap<String, Threshold>>,
    contains: BTreeMap<Iri, BTreeSet<String>>,
    string_atoms: BTreeMap<Iri, BTreeSet<String>>,
}

fn collect_condition_values(
    ontology: &Ontology,
    condition: &Condition,
    collected: &mut CollectedValues,
) {
    let range = ontology
        .schema()
        .property(condition.property)
        .and_then(|property| property.range);
    match &condition.op {
        Op::Exceeds(v) | Op::Below(v) | Op::AtLeast(v) | Op::AtMost(v) => {
            if let Some(threshold) = threshold_for(range, v) {
                collected
                    .thresholds
                    .entry(condition.property)
                    .or_default()
                    .insert(threshold);
            }
        }
        Op::Between(a, b) => {
            for value in [a, b] {
                if let Some(threshold) = threshold_for(range, value) {
                    collected
                        .thresholds
                        .entry(condition.property)
                        .or_default()
                        .insert(threshold);
                }
            }
        }
        Op::Is(term) | Op::IsNot(term) => {
            collect_exact_term(ontology, condition.property, range, term, collected);
        }
        Op::Contains(Term::Literal(Value::String(needle))) => {
            collected
                .contains
                .entry(condition.property)
                .or_default()
                .insert(needle.clone());
        }
        Op::Contains(term) => {
            collect_exact_term(ontology, condition.property, range, term, collected);
        }
        Op::InList(terms) => {
            for term in terms {
                collect_exact_term(ontology, condition.property, range, term, collected);
            }
        }
        Op::IsAssigned | Op::IsEmpty => {}
    }
}

fn collect_exact_term(
    ontology: &Ontology,
    property: Iri,
    range: Option<PropertyRange>,
    term: &Term,
    collected: &mut CollectedValues,
) {
    if let Term::Literal(value) = term
        && as_decimal(value).is_some()
    {
        if let Some(threshold) = threshold_for(range, value) {
            collected
                .thresholds
                .entry(property)
                .or_default()
                .insert(threshold);
        }
        return;
    }
    let text = term_text(ontology, term);
    collected
        .atoms
        .entry(property)
        .or_default()
        .insert(text.clone());
    if let Term::Literal(value) = term
        && let Some(position) = threshold_for(range, value)
    {
        collected
            .atom_positions
            .entry(property)
            .or_default()
            .insert(text.clone(), position);
    }
    if matches!(term, Term::Literal(Value::String(_))) {
        collected
            .string_atoms
            .entry(property)
            .or_default()
            .insert(text);
    }
}

/// One cell of the order induced by a property's authored thresholds.
enum Cell<'a> {
    Below,
    At(&'a Threshold),
    Between(&'a Threshold, &'a Threshold),
    Above,
}

impl Cell<'_> {
    /// Whether a stored value of `property` can fall in this cell: inside
    /// the gate's `horis:min`/`horis:max`, and for `xsd:integer` only where
    /// an integer actually exists.
    fn is_inhabited(&self, property: &PropertyDecl) -> bool {
        let minimum = gate_bound(property, property.min.as_ref());
        let maximum = gate_bound(property, property.max.as_ref());
        let within_bounds = match self {
            Cell::Below => minimum.is_none(),
            Cell::At(point) => {
                minimum.as_ref().is_none_or(|bound| *point >= bound)
                    && maximum.as_ref().is_none_or(|bound| *point <= bound)
            }
            Cell::Between(low, high) => {
                minimum.as_ref().is_none_or(|bound| *low >= bound)
                    && maximum.as_ref().is_none_or(|bound| *high <= bound)
            }
            Cell::Above => maximum.is_none(),
        };
        if !within_bounds {
            return false;
        }
        if !matches!(
            property.range,
            Some(PropertyRange::Datatype(Datatype::Integer))
        ) {
            return true;
        }
        match self {
            Cell::At(Threshold::Number(point)) => point.fract().is_zero(),
            Cell::Between(Threshold::Number(low), Threshold::Number(high)) => {
                low.floor() + Decimal::ONE < *high
            }
            _ => true,
        }
    }

    fn signature(
        &self,
        boundaries: &BTreeMap<Threshold, OrderedVars>,
        variables: &BddVariableSet,
    ) -> Bdd {
        boundaries
            .iter()
            .fold(variables.mk_true(), |signature, (threshold, boundary)| {
                let (greater, at_least) = match self {
                    Cell::Below => (false, false),
                    Cell::At(point) => (*point > threshold, *point >= threshold),
                    Cell::Between(low, _) => (*low >= threshold, *low >= threshold),
                    Cell::Above => (true, true),
                };
                signature
                    .and(&variables.mk_literal(boundary.greater, greater))
                    .and(&variables.mk_literal(boundary.at_least, at_least))
            })
    }
}

/// Every threshold signature some stored value of `property` can have.
fn allowed_signatures(
    property: &PropertyDecl,
    boundaries: &BTreeMap<Threshold, OrderedVars>,
    variables: &BddVariableSet,
) -> Bdd {
    let thresholds: Vec<&Threshold> = boundaries.keys().collect();
    let mut cells = Vec::new();
    if !thresholds.is_empty() {
        cells.push(Cell::Below);
        for (index, point) in thresholds.iter().enumerate() {
            cells.push(Cell::At(point));
            if let Some(next) = thresholds.get(index + 1) {
                cells.push(Cell::Between(point, next));
            }
        }
        cells.push(Cell::Above);
    }
    cells
        .iter()
        .filter(|cell| cell.is_inhabited(property))
        .fold(variables.mk_false(), |allowed, cell| {
            allowed.or(&cell.signature(boundaries, variables))
        })
}

fn verify_completeness(space: &SymbolicSpace<'_>, formulas: &[Bdd], report: &mut ReportBuilder) {
    for property in space
        .ontology
        .schema()
        .properties()
        .filter(|property| property.assigned_by == AssignedBy::RulesOnly)
    {
        let Some(domain) = property.domain else {
            continue;
        };
        let target = space.reachable.and(&space.subject_is(domain));
        let covered = space
            .ontology
            .rules()
            .iter()
            .zip(formulas)
            .filter(|(rule, _)| rule.then.property == property.iri)
            .fold(space.variables.mk_false(), |bdd, (_, formula)| {
                bdd.or(formula)
            });
        let gap = target.and(&covered.not());
        let key = compact_iri(space.ontology, property.iri);
        report.complete.insert(key, gap.is_false());
        if !gap.is_false() {
            let region = space.region(&gap, Some(domain));
            let conditions = render_region_conditions(&region);
            report.witnesses.push(Witness {
                kind: FindingKind::Completeness,
                severity: Severity::Error,
                domain_text: format!(
                    "no {} assigned when {conditions}",
                    local(space.ontology, property.iri)
                ),
                region,
                suggestion: Some(format!(
                    "add a rule assigning {} for this region",
                    local(space.ontology, property.iri)
                )),
            });
        }
    }
}

fn verify_conflicts(space: &SymbolicSpace<'_>, formulas: &[Bdd], report: &mut ReportBuilder) {
    for first in 0..space.ontology.rules().len() {
        let a = &space.ontology.rules()[first];
        for second in first + 1..space.ontology.rules().len() {
            let b = &space.ontology.rules()[second];
            let Some(property) = space.ontology.schema().property(a.then.property) else {
                continue;
            };
            if a.then.property != b.then.property
                || !property
                    .cardinality
                    .is_some_and(Cardinality::is_single_valued)
                || terms_equal(space.ontology, &a.then.value, &b.then.value)
            {
                continue;
            }
            let overlap = formulas[first].and(&formulas[second]);
            if overlap.is_false() {
                continue;
            }
            let region = space.region(&overlap, Some(a.subject_class));
            let property_name = compact_iri(space.ontology, a.then.property);
            let first_rule = compact_iri(space.ontology, a.iri);
            let second_rule = compact_iri(space.ontology, b.iri);
            let first_value =
                display_atom(space.ontology, &term_text(space.ontology, &a.then.value));
            let second_value =
                display_atom(space.ontology, &term_text(space.ontology, &b.then.value));
            let winner = first_rule.clone();
            let conditions = render_region_conditions(&region);
            report.conflicts.push(Conflict {
                first_rule: first_rule.clone(),
                second_rule: second_rule.clone(),
                property: property_name.clone(),
                first_value: first_value.clone(),
                second_value: second_value.clone(),
                first_priority: a.priority,
                second_priority: b.priority,
                winner: winner.clone(),
                region: region.clone(),
            });
            report.witnesses.push(Witness {
                kind: FindingKind::Conflict,
                severity: Severity::Error,
                domain_text: format!(
                    "{first_rule} → {first_value} but {second_rule} → {second_value} for {property_name} when {conditions} — {winner} wins by priority; confirm intended"
                ),
                region,
                suggestion: Some("make the conditions disjoint or align their conclusions".into()),
            });
        }
    }
}

/// Whether this rule's constant conclusion can pass the same schema-level
/// checks as [`crate::world::gate`]. Subject existence/cardinality-minimum
/// are properties of the already-admitted input instance; one conclusion
/// contributes exactly one value. The remaining checks are decidable from
/// the immutable ontology and constant `horis:then` term.
fn conclusion_passes_gate(ontology: &Ontology, rule: &Rule) -> bool {
    let Some(property) = ontology.schema().property(rule.then.property) else {
        return false;
    };
    if property.assigned_by == AssignedBy::ToolsOnly
        || property
            .domain
            .is_some_and(|domain| !ontology.schema().is_subclass_of(rule.subject_class, domain))
    {
        return false;
    }

    let value = match &rule.then.value {
        Term::Iri(iri) => Value::Ref(crate::ontology::term::Ref::new(*iri)),
        Term::Literal(value) => value.clone(),
    };
    match property.range {
        None => {}
        Some(PropertyRange::Datatype(datatype)) if !literal_matches(&value, datatype) => {
            return false;
        }
        Some(PropertyRange::Datatype(_)) => {}
        Some(PropertyRange::Class(range)) => {
            let Value::Ref(referent) = &value else {
                return false;
            };
            if let Some(members) = ontology
                .schema()
                .class(range)
                .and_then(|class| class.one_of.as_ref())
            {
                if !members.iter().any(|&member| {
                    ontology.schema().canonical_iri(member)
                        == ontology.schema().canonical_iri(referent.iri())
                        || (ontology.schema().class_declared(referent.iri()).is_some()
                            && ontology.schema().is_subclass_of(referent.iri(), member))
                }) {
                    return false;
                }
            } else if ontology
                .schema()
                .class_declared(referent.iri())
                .is_none_or(|class| !ontology.schema().is_subclass_of(class, range))
            {
                return false;
            }
        }
    }

    if let (Some(minimum), Some(actual)) = (
        property.min.as_ref().and_then(as_decimal),
        as_decimal(&value),
    ) && actual < minimum
    {
        return false;
    }
    if let (Some(maximum), Some(actual)) = (
        property.max.as_ref().and_then(as_decimal),
        as_decimal(&value),
    ) && actual > maximum
    {
        return false;
    }
    if let Value::String(text) = &value
        && ontology
            .pattern(property.iri)
            .is_some_and(|pattern| !pattern.is_match(text))
    {
        return false;
    }
    true
}

fn literal_matches(value: &Value, datatype: Datatype) -> bool {
    matches!(
        (value, datatype),
        (Value::Decimal(_), Datatype::Decimal)
            | (Value::Integer(_), Datatype::Decimal | Datatype::Integer)
            | (Value::String(_), Datatype::String)
            | (Value::Boolean(_), Datatype::Boolean)
            | (Value::DateTime(_), Datatype::DateTime)
            | (Value::Duration(_), Datatype::Duration)
    )
}

fn verify_dead_and_vacuous_rules(
    space: &SymbolicSpace<'_>,
    candidates: &[Bdd],
    accepted: &[Bdd],
    report: &mut ReportBuilder,
) {
    for (index, rule) in space.ontology.rules().iter().enumerate() {
        let name = compact_iri(space.ontology, rule.iri);
        if candidates[index].is_false() {
            report.dead_rules.push(name.clone());
            report.witnesses.push(Witness {
                kind: FindingKind::DeadRule,
                severity: Severity::Error,
                domain_text: format!(
                    "{name} has no satisfiable input under the declared schema bounds"
                ),
                region: Region {
                    subject_class: Some(compact_iri(space.ontology, rule.subject_class)),
                    predicates: Vec::new(),
                },
                suggestion: Some("repair or remove the unreachable condition".into()),
            });
            continue;
        }
        if accepted[index].is_false() {
            let property = compact_iri(space.ontology, rule.then.property);
            let value = display_atom(space.ontology, &term_text(space.ontology, &rule.then.value));
            report.witnesses.push(Witness {
                kind: FindingKind::RuleVacuity,
                severity: Severity::Error,
                domain_text: format!(
                    "{name} is satisfiable but its conclusion {property} {value} cannot pass the declared validation constraints"
                ),
                region: space.region(&candidates[index], Some(rule.subject_class)),
                suggestion: Some(
                    "change the conclusion to a gate-valid value or repair the property constraints"
                        .into(),
                ),
            });
            continue;
        }
        let higher = space
            .ontology
            .rules()
            .iter()
            .zip(accepted)
            .take(index)
            .filter(|(candidate, _)| {
                if candidate.then.property != rule.then.property {
                    return false;
                }
                let property = space
                    .ontology
                    .schema()
                    .property(rule.then.property)
                    .expect("a parsed rule conclusion names a declared property");
                property.immutable
                    || property
                        .cardinality
                        .is_some_and(Cardinality::is_single_valued)
                    || terms_equal(space.ontology, &candidate.then.value, &rule.then.value)
            })
            .fold(space.variables.mk_false(), |bdd, (_, formula)| {
                bdd.or(formula)
            });
        if !higher.is_false() && accepted[index].and(&higher.not()).is_false() {
            report.witnesses.push(Witness {
                kind: FindingKind::RuleVacuity,
                severity: Severity::Error,
                domain_text: format!("{name} is satisfiable but its conclusion is inert because higher-priority rules cover every matching input"),
                region: space.region(&accepted[index], Some(rule.subject_class)),
                suggestion: Some("narrow the higher-priority rules or remove the inert rule".into()),
            });
        }
    }
}

fn verify_rule_termination(ontology: &Ontology, report: &mut ReportBuilder) {
    let rules = ontology.rules();
    let mut edges = vec![Vec::new(); rules.len()];
    for (source_index, source) in rules.iter().enumerate() {
        for (target_index, target) in rules.iter().enumerate() {
            if rule_can_enable(ontology, source, target) {
                edges[source_index].push(target_index);
            }
        }
    }
    for (start, rule) in rules.iter().enumerate() {
        let mut seen = BTreeSet::new();
        if reaches(start, start, &edges, &mut seen, true) {
            report.terminating = false;
            let name = compact_iri(ontology, rule.iri);
            report.witnesses.push(Witness {
                kind: FindingKind::RuleTermination,
                severity: Severity::Error,
                domain_text: format!("{name} participates in a rule dependency cycle, so fixed-point termination is not proven"),
                region: Region { subject_class: Some(compact_iri(ontology, rule.subject_class)), predicates: Vec::new() },
                suggestion: Some("break the derived-property dependency cycle".into()),
            });
        }
    }
}

fn rule_can_enable(ontology: &Ontology, source: &Rule, target: &Rule) -> bool {
    if !conclusion_passes_gate(ontology, source) {
        return false;
    }
    if !target
        .when
        .leaves()
        .iter()
        .any(|condition| condition.property == source.then.property)
    {
        return false;
    }
    let mut authored_values: Vec<&Term> = Vec::new();
    for value in ontology
        .rules()
        .iter()
        .filter(|rule| rule.then.property == source.then.property)
        .map(|rule| &rule.then.value)
    {
        if !authored_values
            .iter()
            .any(|known| terms_equal(ontology, known, value))
        {
            authored_values.push(value);
        }
    }
    let leaf_count = target.when.leaves().len();
    let variable_count = leaf_count + authored_values.len();
    if variable_count >= (u16::MAX - 1) as usize {
        return true;
    }
    let variables = BddVariableSet::new_anonymous(variable_count as u16);
    let value_variables: Vec<BddVariable> = (leaf_count..variable_count)
        .map(BddVariable::from_index)
        .collect();
    let single_valued = ontology
        .schema()
        .property(source.then.property)
        .and_then(|property| property.cardinality)
        .is_some_and(Cardinality::is_single_valued);

    let mut leaf_index = 0;
    let (before, after) = condition_transition(
        ontology,
        &target.when,
        source.then.property,
        &source.then.value,
        single_valued,
        &authored_values,
        &value_variables,
        &variables,
        &mut leaf_index,
    );
    let source_absent = authored_values
        .iter()
        .position(|value| terms_equal(ontology, value, &source.then.value))
        .map_or_else(
            || variables.mk_true(),
            |index| variables.mk_not_var(value_variables[index]),
        );
    !before.not().and(&after).and(&source_absent).is_false()
}

#[allow(clippy::too_many_arguments)]
fn condition_transition(
    ontology: &Ontology,
    tree: &ConditionTree,
    property: Iri,
    conclusion: &Term,
    single_valued: bool,
    authored_values: &[&Term],
    value_variables: &[BddVariable],
    variables: &BddVariableSet,
    leaf_index: &mut usize,
) -> (Bdd, Bdd) {
    match tree {
        ConditionTree::Leaf(condition) => {
            let variable = BddVariable::from_index(*leaf_index);
            *leaf_index += 1;
            if condition.property == property {
                let before = condition_for_authored_values(
                    ontology,
                    &condition.op,
                    authored_values,
                    value_variables,
                    variables,
                );
                let after_holds = conclusion_satisfies_full(ontology, conclusion, &condition.op);
                let after = if single_valued {
                    if after_holds {
                        variables.mk_true()
                    } else {
                        variables.mk_false()
                    }
                } else {
                    match condition.op {
                        Op::IsNot(_) => {
                            if after_holds {
                                before.clone()
                            } else {
                                variables.mk_false()
                            }
                        }
                        Op::IsEmpty => variables.mk_false(),
                        _ if after_holds => variables.mk_true(),
                        _ => before.clone(),
                    }
                };
                (before, after)
            } else {
                let unchanged = variables.mk_var(variable);
                (unchanged.clone(), unchanged)
            }
        }
        ConditionTree::All(children) => children.iter().fold(
            (variables.mk_true(), variables.mk_true()),
            |(before, after), child| {
                let (child_before, child_after) = condition_transition(
                    ontology,
                    child,
                    property,
                    conclusion,
                    single_valued,
                    authored_values,
                    value_variables,
                    variables,
                    leaf_index,
                );
                (before.and(&child_before), after.and(&child_after))
            },
        ),
        ConditionTree::Any(children) => children.iter().fold(
            (variables.mk_false(), variables.mk_false()),
            |(before, after), child| {
                let (child_before, child_after) = condition_transition(
                    ontology,
                    child,
                    property,
                    conclusion,
                    single_valued,
                    authored_values,
                    value_variables,
                    variables,
                    leaf_index,
                );
                (before.or(&child_before), after.or(&child_after))
            },
        ),
        ConditionTree::NoneOf(children) => {
            let (before, after) = children.iter().fold(
                (variables.mk_false(), variables.mk_false()),
                |(before, after), child| {
                    let (child_before, child_after) = condition_transition(
                        ontology,
                        child,
                        property,
                        conclusion,
                        single_valued,
                        authored_values,
                        value_variables,
                        variables,
                        leaf_index,
                    );
                    (before.or(&child_before), after.or(&child_after))
                },
            );
            (before.not(), after.not())
        }
    }
}

fn condition_for_authored_values(
    ontology: &Ontology,
    op: &Op,
    authored_values: &[&Term],
    value_variables: &[BddVariable],
    variables: &BddVariableSet,
) -> Bdd {
    let assigned = value_variables
        .iter()
        .fold(variables.mk_false(), |bdd, variable| {
            bdd.or(&variables.mk_var(*variable))
        });
    match op {
        Op::IsNot(expected) => authored_values
            .iter()
            .zip(value_variables)
            .filter(|(value, _)| term_matches_static(ontology, value, expected))
            .fold(variables.mk_false(), |bdd, (_, variable)| {
                bdd.or(&variables.mk_var(*variable))
            })
            .not(),
        Op::IsEmpty => assigned.not(),
        _ => authored_values
            .iter()
            .zip(value_variables)
            .filter(|(value, _)| conclusion_satisfies(ontology, value, op))
            .fold(variables.mk_false(), |bdd, (_, variable)| {
                bdd.or(&variables.mk_var(*variable))
            }),
    }
}

fn conclusion_satisfies_full(ontology: &Ontology, conclusion: &Term, op: &Op) -> bool {
    match op {
        Op::IsNot(expected) => !term_matches_static(ontology, conclusion, expected),
        Op::IsEmpty => false,
        _ => conclusion_satisfies(ontology, conclusion, op),
    }
}

fn conclusion_satisfies(ontology: &Ontology, conclusion: &Term, op: &Op) -> bool {
    match op {
        Op::Exceeds(bound) => compare_term(conclusion, bound).is_some_and(|v| v.is_gt()),
        Op::Below(bound) => compare_term(conclusion, bound).is_some_and(|v| v.is_lt()),
        Op::AtLeast(bound) => compare_term(conclusion, bound).is_some_and(|v| v.is_ge()),
        Op::AtMost(bound) => compare_term(conclusion, bound).is_some_and(|v| v.is_le()),
        Op::Between(low, high) => {
            compare_term(conclusion, low).is_some_and(|v| v.is_ge())
                && compare_term(conclusion, high).is_some_and(|v| v.is_le())
        }
        Op::Is(expected) | Op::Contains(expected) => {
            if let (Term::Literal(Value::String(value)), Term::Literal(Value::String(needle))) =
                (conclusion, expected)
                && matches!(op, Op::Contains(_))
            {
                value.contains(needle)
            } else {
                term_matches_static(ontology, conclusion, expected)
            }
        }
        Op::InList(expected) => expected
            .iter()
            .any(|term| term_matches_static(ontology, conclusion, term)),
        Op::IsAssigned => true,
        Op::IsNot(_) | Op::IsEmpty => false,
    }
}

fn compare_term(conclusion: &Term, bound: &Value) -> Option<std::cmp::Ordering> {
    let Term::Literal(value) = conclusion else {
        return None;
    };
    compare_values(value, bound)
}

fn reaches(
    current: usize,
    target: usize,
    edges: &[Vec<usize>],
    seen: &mut BTreeSet<usize>,
    initial: bool,
) -> bool {
    if !initial && current == target {
        return true;
    }
    if !seen.insert(current) {
        return false;
    }
    edges[current]
        .iter()
        .any(|next| reaches(*next, target, edges, seen, false))
}

/// Declared constraints that exclude nothing. An explicit `0..*` is a
/// documented authored value (doc 02) that merely restates the default, so
/// it is reported at info level; a bound that can never reject a value of
/// the property's own type is a genuine authoring error.
fn verify_constraint_vacuity(ontology: &Ontology, report: &mut ReportBuilder) {
    for property in ontology.schema().properties() {
        let property_name = local(ontology, property.iri);
        let domain = property
            .domain
            .map(|iri| compact_iri(ontology, iri))
            .unwrap_or_else(|| "the declared domain".into());
        let region = Region {
            subject_class: property.domain.map(|iri| compact_iri(ontology, iri)),
            predicates: Vec::new(),
        };
        let typed = match property.range {
            Some(PropertyRange::Datatype(kind)) => kind.to_string(),
            Some(PropertyRange::Class(class)) => compact_iri(ontology, class),
            None => "value".into(),
        };
        let mut push = |severity: Severity, domain_text: String, suggestion: &str| {
            report.witnesses.push(Witness {
                kind: FindingKind::ConstraintVacuity,
                severity,
                domain_text,
                region: region.clone(),
                suggestion: Some(suggestion.into()),
            });
        };
        if property.cardinality == Some(Cardinality::ZeroOrMany) {
            push(
                Severity::Info,
                format!(
                    "{property_name} horis:cardinality \"0..*\" excludes no value count on {domain}"
                ),
                "omit the default cardinality or declare a restrictive bound",
            );
        }
        for (name, bound) in [("horis:min", &property.min), ("horis:max", &property.max)] {
            if let Some(bound) = bound
                && gate_bound(property, Some(bound)).is_none()
            {
                push(
                    Severity::Error,
                    format!(
                        "{property_name} {name} {} excludes no {typed} value on {domain}",
                        bound.lexical_form().unwrap_or_default()
                    ),
                    "declare a numeric bound on a numeric property or remove it",
                );
            }
        }
        if let Some(pattern) = property.pattern.as_deref() {
            let applies_to_range = matches!(
                property.range,
                None | Some(PropertyRange::Datatype(Datatype::String))
            );
            if !applies_to_range || universal_pattern(pattern) {
                push(
                    Severity::Error,
                    format!(
                        "{property_name} horis:pattern {pattern:?} excludes no {typed} value on {domain}"
                    ),
                    "remove the vacuous pattern or declare a restrictive expression",
                );
            }
        }
    }
}

fn universal_pattern(pattern: &str) -> bool {
    matches!(pattern, ".*" | "^.*$" | "(?:.*)" | "^(?:.*)$")
}

fn verify_workflows(space: &SymbolicSpace<'_>, formulas: &[Bdd], report: &mut ReportBuilder) {
    for workflow in space.ontology.workflows().values() {
        let workflow_name = compact_iri(space.ontology, workflow.iri);
        let mut result = WorkflowVerification::default();

        let edges: BTreeMap<Iri, Vec<Iri>> = workflow
            .steps
            .iter()
            .map(|(&iri, step)| (iri, verification_step_edges(step)))
            .collect();
        let cycle = workflow.steps.keys().copied().any(|start| {
            let mut seen = BTreeSet::new();
            reaches_step(start, start, &edges, &mut seen, true)
        });
        let dead_end = workflow.steps.values().any(|step| {
            !step.terminal
                && edges
                    .get(&step.iri)
                    .is_none_or(|targets| targets.is_empty())
        });
        if cycle || dead_end {
            result.terminating = false;
            report.witnesses.push(Witness {
                kind: FindingKind::WorkflowTermination,
                severity: Severity::Error,
                domain_text: format!(
                    "{workflow_name} has a reachable {} and not every path reaches horis:terminal",
                    if cycle {
                        "step cycle"
                    } else {
                        "non-terminal dead end"
                    }
                ),
                region: Region::default(),
                suggestion: Some("route every acyclic path to a terminal step".into()),
            });
        }

        for step in workflow.steps.values() {
            let step_name = compact_iri(space.ontology, step.iri);
            let missing_edge = match &step.body {
                StepBody::Perception(body) if body.fallback.is_none() => Some("horis:fallback"),
                StepBody::Reasoning(body) if body.on_unreachable.is_none() => {
                    Some("horis:onUnreachable")
                }
                StepBody::Action(body) if body.on_error.is_none() => Some("horis:onError"),
                StepBody::Proposal(body) if body.fallback.is_none() => Some("horis:fallback"),
                _ => None,
            };
            if let Some(edge) = missing_edge {
                result.fallbacks_total = false;
                report.witnesses.push(Witness {
                    kind: FindingKind::WorkflowFallback,
                    severity: Severity::Error,
                    domain_text: format!("{step_name} can fail but declares no {edge} edge"),
                    region: Region::default(),
                    suggestion: Some(format!("declare {edge} on {step_name}")),
                });
            }

            if let StepBody::Action(action) = &step.body
                && let Some(requires) = &action.requires
                && space
                    .reachable
                    .and(&space.condition_tree_after_rules(requires, formulas))
                    .is_false()
            {
                result.requires_satisfiable = false;
                report.witnesses.push(Witness {
                    kind: FindingKind::WorkflowRequires,
                    severity: Severity::Error,
                    domain_text: format!(
                        "{step_name} horis:requires has no satisfiable value under the declared schema bounds"
                    ),
                    region: Region::default(),
                    suggestion: Some("repair the precondition or its establishing rules".into()),
                });
            }

            if let StepBody::Choice(choice) = &step.body
                && choice.otherwise.is_none()
            {
                let covered = choice
                    .branches
                    .iter()
                    .fold(space.variables.mk_false(), |bdd, branch| {
                        bdd.or(&space.condition_tree_after_rules(&branch.when, formulas))
                    });
                let subject_scope = choice
                    .branches
                    .iter()
                    .fold(space.variables.mk_false(), |scope, branch| {
                        scope.or(&space.condition_subject_scope(&branch.when))
                    });
                let target = space.reachable.and(&subject_scope);
                if !target.and(&covered.not()).is_false() {
                    result.choices_total = false;
                    report.witnesses.push(Witness {
                        kind: FindingKind::WorkflowChoice,
                        severity: Severity::Error,
                        domain_text: format!(
                            "{step_name} horis:branches do not cover every reachable case and no horis:otherwise is declared"
                        ),
                        region: space.region(&target.and(&covered.not()), None),
                        suggestion: Some("add a covering branch or horis:otherwise".into()),
                    });
                }
            }
        }

        report.workflows.insert(workflow_name, result);
    }

    for step in space.ontology.unreachable_workflow_steps() {
        let step_name = compact_iri(space.ontology, *step);
        report.witnesses.push(Witness {
            kind: FindingKind::WorkflowReachability,
            severity: Severity::Error,
            domain_text: format!(
                "{step_name} is not reachable from any horis:Workflow horis:startsAt"
            ),
            region: Region::default(),
            suggestion: Some("connect the step to a workflow or remove its declaration".into()),
        });
    }
}

fn verification_step_edges(step: &Step) -> Vec<Iri> {
    if step.terminal {
        return Vec::new();
    }
    match &step.body {
        StepBody::Bare => Vec::new(),
        StepBody::Perception(body) => body.fallback.into_iter().chain([body.next]).collect(),
        StepBody::Reasoning(body) => body.on_unreachable.into_iter().chain([body.next]).collect(),
        StepBody::Action(body) => [body.on_error, body.on_unsatisfied, body.next]
            .into_iter()
            .flatten()
            .collect(),
        StepBody::Narration(body) => vec![body.next],
        StepBody::Proposal(body) => body.fallback.into_iter().chain([body.next]).collect(),
        StepBody::Choice(body) => body
            .branches
            .iter()
            .map(|branch| branch.next)
            .chain(body.otherwise)
            .collect(),
        StepBody::Wait(body) => body.next.into_iter().collect(),
        StepBody::Subflow(body) => vec![body.next],
    }
}

fn reaches_step(
    current: Iri,
    target: Iri,
    edges: &BTreeMap<Iri, Vec<Iri>>,
    seen: &mut BTreeSet<Iri>,
    initial: bool,
) -> bool {
    if !initial && current == target {
        return true;
    }
    if !seen.insert(current) {
        return false;
    }
    edges
        .get(&current)
        .into_iter()
        .flatten()
        .any(|next| reaches_step(*next, target, edges, seen, false))
}

fn as_decimal(value: &Value) -> Option<Decimal> {
    match value {
        Value::Decimal(value) => Some(*value),
        Value::Integer(value) => Some(Decimal::from(*value)),
        _ => None,
    }
}

fn term_text(ontology: &Ontology, term: &Term) -> String {
    match term {
        Term::Iri(iri) => ontology.resolve(*iri).to_owned(),
        Term::Literal(Value::Ref(value)) => ontology.resolve(value.iri()).to_owned(),
        Term::Literal(value) => value.lexical_form().unwrap_or_default(),
    }
}

fn terms_equal(ontology: &Ontology, a: &Term, b: &Term) -> bool {
    match (a, b) {
        (Term::Iri(a), Term::Iri(b)) => canonical_iris_equal(ontology, *a, *b),
        (Term::Literal(Value::Ref(a)), Term::Literal(Value::Ref(b))) => {
            canonical_iris_equal(ontology, a.iri(), b.iri())
        }
        (Term::Iri(a), Term::Literal(Value::Ref(b)))
        | (Term::Literal(Value::Ref(b)), Term::Iri(a)) => {
            canonical_iris_equal(ontology, *a, b.iri())
        }
        (Term::Literal(a), Term::Literal(b)) => values_equal(a, b),
        _ => false,
    }
}

fn term_matches_static(ontology: &Ontology, value: &Term, expected: &Term) -> bool {
    match (value, expected) {
        (Term::Iri(value), Term::Iri(expected)) => {
            canonical_iris_equal(ontology, *value, *expected)
                || ontology
                    .schema()
                    .class_declared(*value)
                    .zip(ontology.schema().class_declared(*expected))
                    .is_some_and(|(actual, target)| {
                        ontology.schema().is_subclass_of(actual, target)
                    })
        }
        (Term::Literal(value), Term::Literal(expected)) => values_equal(value, expected),
        (Term::Literal(Value::Ref(value)), Term::Iri(expected)) => {
            canonical_iris_equal(ontology, value.iri(), *expected)
                || ontology
                    .schema()
                    .class_declared(value.iri())
                    .zip(ontology.schema().class_declared(*expected))
                    .is_some_and(|(actual, target)| {
                        ontology.schema().is_subclass_of(actual, target)
                    })
        }
        _ => false,
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (as_decimal(a), as_decimal(b)) {
        (Some(a), Some(b)) => a == b,
        _ => a == b,
    }
}

fn canonical_iris_equal(ontology: &Ontology, a: Iri, b: Iri) -> bool {
    ontology.schema().canonical_iri(a) == ontology.schema().canonical_iri(b)
}

fn compact_iri(ontology: &Ontology, iri: Iri) -> String {
    ontology.compact(ontology.resolve(iri))
}
fn local(ontology: &Ontology, iri: Iri) -> String {
    compact_iri(ontology, iri)
        .rsplit_once(':')
        .map_or_else(|| compact_iri(ontology, iri), |(_, local)| local.to_owned())
}
fn display_atom(ontology: &Ontology, atom: &str) -> String {
    ontology
        .iri(atom)
        .map_or_else(|| atom.to_owned(), |iri| compact_iri(ontology, iri))
}

fn forced(formula: &Bdd, variables: &BddVariableSet, variable: BddVariable, value: bool) -> bool {
    formula
        .and(&variables.mk_literal(variable, !value))
        .is_false()
}

fn first_prime_region(formula: &Bdd, variables: &BddVariableSet) -> Bdd {
    let Some(valuation) = formula.first_valuation() else {
        return variables.mk_false();
    };
    let all_variables = variables.variables();
    let mut literals: Vec<Option<bool>> = all_variables
        .iter()
        .map(|variable| Some(valuation.value(*variable)))
        .collect();
    loop {
        let mut removed = false;
        for index in 0..literals.len() {
            let previous = literals[index].take();
            let candidate = literal_cube(variables, &all_variables, &literals);
            if candidate.and(&formula.not()).is_false() {
                removed |= previous.is_some();
            } else {
                literals[index] = previous;
            }
        }
        if !removed {
            break;
        }
    }
    literal_cube(variables, &all_variables, &literals)
}

fn literal_cube(
    variables: &BddVariableSet,
    all_variables: &[BddVariable],
    literals: &[Option<bool>],
) -> Bdd {
    all_variables
        .iter()
        .zip(literals)
        .filter_map(|(variable, value)| value.map(|value| (*variable, value)))
        .fold(variables.mk_true(), |bdd, (variable, value)| {
            bdd.and(&variables.mk_literal(variable, value))
        })
}

fn strongest_lower(bound: &mut Option<(Threshold, bool)>, value: &Threshold, inclusive: bool) {
    if bound.as_ref().is_none_or(|(current, current_inclusive)| {
        value > current || (value == current && !inclusive && *current_inclusive)
    }) {
        *bound = Some((value.clone(), inclusive));
    }
}
fn strongest_upper(bound: &mut Option<(Threshold, bool)>, value: &Threshold, inclusive: bool) {
    if bound.as_ref().is_none_or(|(current, current_inclusive)| {
        value < current || (value == current && !inclusive && *current_inclusive)
    }) {
        *bound = Some((value.clone(), inclusive));
    }
}

fn render_region_conditions(region: &Region) -> String {
    let mut conditions = Vec::new();
    for predicate in &region.predicates {
        match predicate {
            RegionPredicate::Numeric {
                property,
                lower,
                upper,
                ..
            } => conditions.push(render_interval(property, lower.as_ref(), upper.as_ref())),
            RegionPredicate::Absent {
                property: _,
                value: Some(value),
            } => conditions.push(format!("no {} present", value.trim_start_matches(':'))),
            RegionPredicate::Absent {
                property,
                value: None,
            } => conditions.push(format!("no {property} assigned")),
            RegionPredicate::Present { property, value }
            | RegionPredicate::Value { property, value } => {
                conditions.push(format!("{property} is {value}"))
            }
            RegionPredicate::Assigned { property, assigned } => conditions.push(if *assigned {
                format!("{property} is assigned")
            } else {
                format!("no {property} assigned")
            }),
        }
    }
    if conditions.is_empty() {
        "the declared input domain holds".into()
    } else {
        conditions.join(" and ")
    }
}

fn render_interval(property: &str, lower: Option<&Bound>, upper: Option<&Bound>) -> String {
    match (lower, upper) {
        (Some(lower), Some(upper)) => format!(
            "{property} ∈ {}{}, {}{}",
            if lower.inclusive { "[" } else { "(" },
            lower.value,
            upper.value,
            if upper.inclusive { "]" } else { ")" }
        ),
        (Some(lower), None) => format!(
            "{property} {} {}",
            if lower.inclusive { "≥" } else { ">" },
            lower.value
        ),
        (None, Some(upper)) => format!(
            "{property} {} {}",
            if upper.inclusive { "≤" } else { "<" },
            upper.value
        ),
        (None, None) => format!("{property} has a numeric value"),
    }
}
