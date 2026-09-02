//! Bounded symbolic verification of authored rules.
//!
//! The verifier discretizes only ontology-authored boundaries and values,
//! then represents the resulting finite projection as BDDs. It neither
//! converts units nor claims anything outside the report's assumptions.
//!
//! Spec: `docs/framework/10-verification.md` · Owner: PRP-005

use std::collections::{BTreeMap, BTreeSet};

use biodivine_lib_bdd::{Bdd, BddVariable, BddVariableSet, BddVariableSetBuilder};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use super::{
    AssumptionManifest, Bound, Conflict, FindingKind, Region, RegionPredicate, ReportBuilder,
    VerificationReport, Witness, WorkflowVerification,
};
use crate::Ontology;
use crate::ontology::schema::{AssignedBy, Cardinality, PropertyDecl, PropertyRange};
use crate::ontology::term::{Datatype, Iri, Value};
use crate::reason::rule::{Condition, ConditionTree, Op, Rule, Term};
use crate::workflow::model::{Step, StepBody};

#[derive(Debug, Clone, Copy)]
struct NumericVars {
    greater: BddVariable,
    at_least: BddVariable,
}

#[derive(Debug)]
struct PropertyVars {
    assigned: BddVariable,
    numeric: BTreeMap<Decimal, NumericVars>,
    atoms: BTreeMap<String, BddVariable>,
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

/// Verify all rule guarantees under a caller-supplied bounded-projection
/// assumption manifest.
pub fn verify_ontology(ontology: &Ontology, assumptions: AssumptionManifest) -> VerificationReport {
    let space = SymbolicSpace::new(ontology);
    let formulas: Vec<Bdd> = ontology
        .rules()
        .iter()
        .map(|rule| space.rule_formula(rule))
        .collect();
    let mut report = ReportBuilder::new(assumptions);

    verify_completeness(&space, &formulas, &mut report);
    verify_conflicts(&space, &formulas, &mut report);
    verify_dead_and_vacuous_rules(&space, &formulas, &mut report);
    verify_rule_termination(ontology, &mut report);
    verify_constraint_vacuity(ontology, &mut report);
    verify_workflows(&space, &formulas, &mut report);

    report.finish()
}

impl<'a> SymbolicSpace<'a> {
    fn new(ontology: &'a Ontology) -> Self {
        let mut thresholds: BTreeMap<Iri, BTreeSet<Decimal>> = BTreeMap::new();
        let mut atoms: BTreeMap<Iri, BTreeSet<String>> = BTreeMap::new();
        let mut contains: BTreeMap<Iri, BTreeSet<String>> = BTreeMap::new();
        let mut string_atoms: BTreeMap<Iri, BTreeSet<String>> = BTreeMap::new();

        for property in ontology.schema().properties() {
            if let Some(value) = property.min.as_ref().and_then(as_decimal) {
                thresholds.entry(property.iri).or_default().insert(value);
            }
            if let Some(value) = property.max.as_ref().and_then(as_decimal) {
                thresholds.entry(property.iri).or_default().insert(value);
            }
            if let Some(PropertyRange::Class(range)) = property.range {
                let property_atoms = atoms.entry(property.iri).or_default();
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
        }
        for rule in ontology.rules() {
            for condition in rule.when.leaves() {
                collect_condition_values(
                    ontology,
                    condition,
                    &mut thresholds,
                    &mut atoms,
                    &mut contains,
                    &mut string_atoms,
                );
            }
        }

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
            let mut numeric = BTreeMap::new();
            for (index, threshold) in thresholds
                .get(&property.iri)
                .into_iter()
                .flatten()
                .enumerate()
            {
                numeric.insert(
                    *threshold,
                    NumericVars {
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
                    numeric,
                    atoms: exact,
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

            if !vars.numeric.is_empty() {
                for numeric in vars.numeric.values() {
                    self.reachable = self
                        .reachable
                        .and(&self.variables.mk_var(numeric.greater).imp(&assigned));
                    self.reachable = self
                        .reachable
                        .and(&self.variables.mk_var(numeric.at_least).imp(&assigned));
                }
                let allowed = numeric_samples(property, vars.numeric.keys().copied())
                    .into_iter()
                    .fold(self.variables.mk_false(), |allowed, sample| {
                        allowed.or(&numeric_signature(&self.variables, &vars.numeric, sample))
                    });
                self.reachable = self.reachable.and(&assigned.imp(&allowed));
            }

            let exact: Vec<BddVariable> = vars.atoms.values().copied().collect();
            for atom in &exact {
                self.reachable = self
                    .reachable
                    .and(&self.variables.mk_var(*atom).imp(&assigned));
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
            if matches!(property.range, Some(PropertyRange::Class(_))) && !exact.is_empty() {
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

    fn rule_formula(&self, rule: &Rule) -> Bdd {
        self.reachable
            .and(&self.subject_is(rule.subject_class))
            .and(&self.condition_tree(&rule.when))
    }

    fn condition_tree(&self, tree: &ConditionTree) -> Bdd {
        match tree {
            ConditionTree::Leaf(condition) => self.condition(condition),
            ConditionTree::All(children) => children
                .iter()
                .fold(self.variables.mk_true(), |bdd, child| {
                    bdd.and(&self.condition_tree(child))
                }),
            ConditionTree::Any(children) => children
                .iter()
                .fold(self.variables.mk_false(), |bdd, child| {
                    bdd.or(&self.condition_tree(child))
                }),
            ConditionTree::NoneOf(children) => children
                .iter()
                .fold(self.variables.mk_false(), |bdd, child| {
                    bdd.or(&self.condition_tree(child))
                })
                .not(),
        }
    }

    fn condition_tree_after_rules(&self, tree: &ConditionTree, formulas: &[Bdd]) -> Bdd {
        match tree {
            ConditionTree::Leaf(condition) => {
                let is_derived = self
                    .ontology
                    .schema()
                    .property(condition.property)
                    .is_some_and(|property| property.assigned_by == AssignedBy::RulesOnly);
                if !is_derived {
                    return self.condition(condition);
                }
                let assigning = self
                    .ontology
                    .rules()
                    .iter()
                    .zip(formulas)
                    .filter(|(rule, _)| rule.then.property == condition.property);
                let assigned = assigning
                    .clone()
                    .fold(self.variables.mk_false(), |bdd, (_, formula)| {
                        bdd.or(formula)
                    });
                match &condition.op {
                    Op::IsEmpty => assigned.not(),
                    Op::IsNot(expected) => assigning
                        .filter(|(rule, _)| {
                            term_matches_static(self.ontology, &rule.then.value, expected)
                        })
                        .fold(self.variables.mk_false(), |bdd, (_, formula)| {
                            bdd.or(formula)
                        })
                        .not(),
                    op => assigning
                        .filter(|(rule, _)| {
                            conclusion_satisfies(self.ontology, &rule.then.value, op)
                        })
                        .fold(self.variables.mk_false(), |bdd, (_, formula)| {
                            bdd.or(formula)
                        }),
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
        match &condition.op {
            Op::Exceeds(value) => assigned.and(&self.numeric(vars, value, |v| v.greater)),
            Op::Below(value) => assigned.and(&self.numeric(vars, value, |v| v.at_least).not()),
            Op::AtLeast(value) => assigned.and(&self.numeric(vars, value, |v| v.at_least)),
            Op::AtMost(value) => assigned.and(&self.numeric(vars, value, |v| v.greater).not()),
            Op::Between(low, high) => assigned
                .and(&self.numeric(vars, low, |v| v.at_least))
                .and(&self.numeric(vars, high, |v| v.greater).not()),
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

    fn numeric(
        &self,
        vars: &PropertyVars,
        value: &Value,
        select: impl FnOnce(NumericVars) -> BddVariable,
    ) -> Bdd {
        as_decimal(value)
            .and_then(|number| vars.numeric.get(&number).copied())
            .map_or_else(
                || self.variables.mk_false(),
                |boundary| self.variables.mk_var(select(boundary)),
            )
    }

    fn term_matches(&self, property: Iri, term: &Term) -> Bdd {
        let Some(vars) = self.property_vars.get(&property) else {
            return self.variables.mk_false();
        };
        match term {
            Term::Literal(value) if as_decimal(value).is_some() => as_decimal(value)
                .and_then(|number| vars.numeric.get(&number))
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
            if !vars.numeric.is_empty() {
                let mut lower: Option<(Decimal, bool)> = None;
                let mut upper: Option<(Decimal, bool)> = None;
                for (number, boundary) in &vars.numeric {
                    if forced(&formula, &self.variables, boundary.greater, true) {
                        strongest_lower(&mut lower, *number, false);
                    } else if forced(&formula, &self.variables, boundary.greater, false) {
                        strongest_upper(&mut upper, *number, true);
                    }
                    if forced(&formula, &self.variables, boundary.at_least, true) {
                        strongest_lower(&mut lower, *number, true);
                    } else if forced(&formula, &self.variables, boundary.at_least, false) {
                        strongest_upper(&mut upper, *number, false);
                    }
                }
                if lower.is_some() || upper.is_some() {
                    predicates.push(RegionPredicate::Numeric {
                        property: local(self.ontology, property.iri),
                        lower: lower.map(|(value, inclusive)| Bound {
                            value: value.to_string(),
                            inclusive,
                        }),
                        upper: upper.map(|(value, inclusive)| Bound {
                            value: value.to_string(),
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

fn collect_condition_values(
    ontology: &Ontology,
    condition: &Condition,
    thresholds: &mut BTreeMap<Iri, BTreeSet<Decimal>>,
    atoms: &mut BTreeMap<Iri, BTreeSet<String>>,
    contains: &mut BTreeMap<Iri, BTreeSet<String>>,
    string_atoms: &mut BTreeMap<Iri, BTreeSet<String>>,
) {
    match &condition.op {
        Op::Exceeds(v) | Op::Below(v) | Op::AtLeast(v) | Op::AtMost(v) => {
            if let Some(number) = as_decimal(v) {
                thresholds
                    .entry(condition.property)
                    .or_default()
                    .insert(number);
            }
        }
        Op::Between(a, b) => {
            for value in [a, b] {
                if let Some(number) = as_decimal(value) {
                    thresholds
                        .entry(condition.property)
                        .or_default()
                        .insert(number);
                }
            }
        }
        Op::Is(term) | Op::IsNot(term) => {
            collect_exact_term(
                ontology,
                condition.property,
                term,
                thresholds,
                atoms,
                string_atoms,
            );
        }
        Op::Contains(Term::Literal(Value::String(needle))) => {
            contains
                .entry(condition.property)
                .or_default()
                .insert(needle.clone());
        }
        Op::Contains(term) => {
            collect_exact_term(
                ontology,
                condition.property,
                term,
                thresholds,
                atoms,
                string_atoms,
            );
        }
        Op::InList(terms) => {
            for term in terms {
                collect_exact_term(
                    ontology,
                    condition.property,
                    term,
                    thresholds,
                    atoms,
                    string_atoms,
                );
            }
        }
        Op::IsAssigned | Op::IsEmpty => {}
    }
}

fn collect_exact_term(
    ontology: &Ontology,
    property: Iri,
    term: &Term,
    thresholds: &mut BTreeMap<Iri, BTreeSet<Decimal>>,
    atoms: &mut BTreeMap<Iri, BTreeSet<String>>,
    string_atoms: &mut BTreeMap<Iri, BTreeSet<String>>,
) {
    if let Term::Literal(value) = term
        && let Some(number) = as_decimal(value)
    {
        thresholds.entry(property).or_default().insert(number);
        return;
    }
    let text = term_text(ontology, term);
    atoms.entry(property).or_default().insert(text.clone());
    if matches!(term, Term::Literal(Value::String(_))) {
        string_atoms.entry(property).or_default().insert(text);
    }
}

fn numeric_samples(
    property: &PropertyDecl,
    thresholds: impl Iterator<Item = Decimal>,
) -> BTreeSet<Decimal> {
    let thresholds: Vec<Decimal> = thresholds.collect();
    let mut samples = BTreeSet::new();
    match property.range {
        Some(PropertyRange::Datatype(Datatype::Decimal)) => {
            for threshold in &thresholds {
                samples.insert(*threshold);
            }
            for pair in thresholds.windows(2) {
                if let Some(sum) = pair[0].checked_add(pair[1]) {
                    samples.insert(sum / Decimal::from(2));
                }
            }
            if let Some(first) = thresholds.first().and_then(|v| v.checked_sub(Decimal::ONE)) {
                samples.insert(first);
            }
            if let Some(last) = thresholds.last().and_then(|v| v.checked_add(Decimal::ONE)) {
                samples.insert(last);
            }
        }
        Some(PropertyRange::Datatype(Datatype::Integer)) => {
            for threshold in &thresholds {
                for integer in [
                    threshold.floor().to_i64().and_then(|v| v.checked_sub(1)),
                    threshold.floor().to_i64(),
                    threshold.ceil().to_i64(),
                    threshold.ceil().to_i64().and_then(|v| v.checked_add(1)),
                ]
                .into_iter()
                .flatten()
                {
                    samples.insert(Decimal::from(integer));
                }
            }
        }
        _ => {}
    }
    let minimum = property.min.as_ref().and_then(as_decimal);
    let maximum = property.max.as_ref().and_then(as_decimal);
    samples.retain(|sample| {
        minimum.is_none_or(|bound| *sample >= bound) && maximum.is_none_or(|bound| *sample <= bound)
    });
    samples
}

fn numeric_signature(
    variables: &BddVariableSet,
    boundaries: &BTreeMap<Decimal, NumericVars>,
    sample: Decimal,
) -> Bdd {
    boundaries
        .iter()
        .fold(variables.mk_true(), |signature, (threshold, boundary)| {
            signature
                .and(&variables.mk_literal(boundary.greater, sample > *threshold))
                .and(&variables.mk_literal(boundary.at_least, sample >= *threshold))
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
            let first_value = term_text(space.ontology, &a.then.value);
            let second_value = term_text(space.ontology, &b.then.value);
            report.conflicts.push(Conflict {
                first_rule: first_rule.clone(),
                second_rule: second_rule.clone(),
                property: property_name.clone(),
                first_value: display_atom(space.ontology, &first_value),
                second_value: display_atom(space.ontology, &second_value),
                region: region.clone(),
            });
            report.witnesses.push(Witness {
                kind: FindingKind::Conflict,
                domain_text: format!("{first_rule} and {second_rule} assign conflicting {property_name} values in the same region"),
                region,
                suggestion: Some("make the conditions disjoint or align their conclusions".into()),
            });
        }
    }
}

fn verify_dead_and_vacuous_rules(
    space: &SymbolicSpace<'_>,
    formulas: &[Bdd],
    report: &mut ReportBuilder,
) {
    for (index, rule) in space.ontology.rules().iter().enumerate() {
        let name = compact_iri(space.ontology, rule.iri);
        if formulas[index].is_false() {
            report.dead_rules.push(name.clone());
            report.witnesses.push(Witness {
                kind: FindingKind::DeadRule,
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
        let higher = space
            .ontology
            .rules()
            .iter()
            .zip(formulas)
            .take(index)
            .filter(|(candidate, _)| {
                candidate.priority > rule.priority && candidate.then.property == rule.then.property
            })
            .fold(space.variables.mk_false(), |bdd, (_, formula)| {
                bdd.or(formula)
            });
        if !higher.is_false() && formulas[index].and(&higher.not()).is_false() {
            report.witnesses.push(Witness {
                kind: FindingKind::RuleVacuity,
                domain_text: format!("{name} is satisfiable but its conclusion is inert because higher-priority rules cover every matching input"),
                region: space.region(&formulas[index], Some(rule.subject_class)),
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
                domain_text: format!("{name} participates in a rule dependency cycle, so fixed-point termination is not proven"),
                region: Region { subject_class: Some(compact_iri(ontology, rule.subject_class)), predicates: Vec::new() },
                suggestion: Some("break the derived-property dependency cycle".into()),
            });
        }
    }
}

fn rule_can_enable(ontology: &Ontology, source: &Rule, target: &Rule) -> bool {
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
        Op::Exceeds(bound) => compare_numeric_term(conclusion, bound).is_some_and(|v| v.is_gt()),
        Op::Below(bound) => compare_numeric_term(conclusion, bound).is_some_and(|v| v.is_lt()),
        Op::AtLeast(bound) => compare_numeric_term(conclusion, bound).is_some_and(|v| v.is_ge()),
        Op::AtMost(bound) => compare_numeric_term(conclusion, bound).is_some_and(|v| v.is_le()),
        Op::Between(low, high) => {
            compare_numeric_term(conclusion, low).is_some_and(|v| v.is_ge())
                && compare_numeric_term(conclusion, high).is_some_and(|v| v.is_le())
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

fn compare_numeric_term(conclusion: &Term, bound: &Value) -> Option<std::cmp::Ordering> {
    let Term::Literal(value) = conclusion else {
        return None;
    };
    Some(as_decimal(value)?.cmp(&as_decimal(bound)?))
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

fn verify_constraint_vacuity(ontology: &Ontology, report: &mut ReportBuilder) {
    for property in ontology.schema().properties() {
        let property_name = local(ontology, property.iri);
        let domain = property
            .domain
            .map(|iri| compact_iri(ontology, iri))
            .unwrap_or_else(|| "the declared domain".into());
        if property.cardinality == Some(Cardinality::ZeroOrMany) {
            report.witnesses.push(Witness {
                kind: FindingKind::ConstraintVacuity,
                domain_text: format!(
                    "{property_name} horis:cardinality \"0..*\" excludes no value count on {domain}"
                ),
                region: Region {
                    subject_class: property.domain.map(|iri| compact_iri(ontology, iri)),
                    predicates: Vec::new(),
                },
                suggestion: Some(
                    "remove the vacuous cardinality or declare a restrictive bound".into(),
                ),
            });
        }
        if property.pattern.as_deref().is_some_and(universal_pattern) {
            let datatype = match property.range {
                Some(PropertyRange::Datatype(kind)) => kind.to_string(),
                _ => "value".into(),
            };
            report.witnesses.push(Witness {
                kind: FindingKind::ConstraintVacuity,
                domain_text: format!(
                    "{property_name} horis:pattern {:?} excludes no {datatype} value on {domain}",
                    property.pattern.as_deref().unwrap_or_default()
                ),
                region: Region {
                    subject_class: property.domain.map(|iri| compact_iri(ontology, iri)),
                    predicates: Vec::new(),
                },
                suggestion: Some(
                    "remove the vacuous pattern or declare a restrictive expression".into(),
                ),
            });
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

fn strongest_lower(bound: &mut Option<(Decimal, bool)>, value: Decimal, inclusive: bool) {
    if bound.is_none_or(|(current, current_inclusive)| {
        value > current || (value == current && !inclusive && current_inclusive)
    }) {
        *bound = Some((value, inclusive));
    }
}
fn strongest_upper(bound: &mut Option<(Decimal, bool)>, value: Decimal, inclusive: bool) {
    if bound.is_none_or(|(current, current_inclusive)| {
        value < current || (value == current && !inclusive && current_inclusive)
    }) {
        *bound = Some((value, inclusive));
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
