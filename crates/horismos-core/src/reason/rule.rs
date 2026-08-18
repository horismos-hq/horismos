//! The typed rule model (U1): `horis:Rule` parsed from the ontology's graphs
//! into data the reasoner interprets — a rule is a row in a rules table, not
//! a function.
//!
//! Parsing happens once, at [`crate::Ontology`] assembly, so every rule error
//! is a *load* error in domain terms (the same posture as `horis:pattern`
//! compilation): an undeclared property in a condition, a `horis:unit` that
//! contradicts the property's declaration (PRP-001 D6's rule-side
//! follow-through), or conditions that bind more than one subject class can
//! never surprise a running world.
//!
//! Spec: `docs/framework/02-ontology.md` § Rule vocabulary,
//! `docs/framework/04-reasoning.md` · Owner: PRP-002

use std::collections::{BTreeMap, BTreeSet};

use crate::error::OntologyError;
use crate::ontology::schema::Schema;
use crate::ontology::term::{Interner, Iri, Value};
use crate::ontology::turtle::{Document, Literal, Node, Object, Triple};

// -- Vocabulary this module reads directly -------------------------------

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const RDF_FIRST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#first";
const RDF_REST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#rest";
const RDF_NIL: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#nil";
const HORIS_RULE: &str = "https://w3id.org/horismos#Rule";
const HORIS_PRIORITY: &str = "https://w3id.org/horismos#priority";
const HORIS_WHEN: &str = "https://w3id.org/horismos#when";
const HORIS_THEN: &str = "https://w3id.org/horismos#then";
const HORIS_EXPLAIN: &str = "https://w3id.org/horismos#explain";
const HORIS_ALL: &str = "https://w3id.org/horismos#all";
const HORIS_ANY: &str = "https://w3id.org/horismos#any";
const HORIS_NONE: &str = "https://w3id.org/horismos#none";
const HORIS_SUBJECT: &str = "https://w3id.org/horismos#subject";
const HORIS_PROPERTY: &str = "https://w3id.org/horismos#property";
const HORIS_VALUE: &str = "https://w3id.org/horismos#value";
const HORIS_UNIT: &str = "https://w3id.org/horismos#unit";
const HORIS_EXCEEDS: &str = "https://w3id.org/horismos#exceeds";
const HORIS_BELOW: &str = "https://w3id.org/horismos#below";
const HORIS_AT_LEAST: &str = "https://w3id.org/horismos#atLeast";
const HORIS_AT_MOST: &str = "https://w3id.org/horismos#atMost";
const HORIS_BETWEEN: &str = "https://w3id.org/horismos#between";
const HORIS_IS: &str = "https://w3id.org/horismos#is";
const HORIS_IS_NOT: &str = "https://w3id.org/horismos#isNot";
const HORIS_IS_ASSIGNED: &str = "https://w3id.org/horismos#isAssigned";
const HORIS_IS_EMPTY: &str = "https://w3id.org/horismos#isEmpty";
const HORIS_CONTAINS: &str = "https://w3id.org/horismos#contains";
const HORIS_IN_LIST: &str = "https://w3id.org/horismos#inList";

// -- Public model ----------------------------------------------------------

/// The value position of `horis:is`/`isNot`/`inList`/`horis:value`: either a
/// node (a class, enum member, or instance IRI) or a typed literal.
#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Iri(Iri),
    Literal(Value),
}

/// One of the 11 condition operators (`docs/framework/02-ontology.md`),
/// carrying its declared operand(s). Operand *type* coherence is evaluation
/// semantics (U2) and verification (PRP-005); only structure is checked here.
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Exceeds(Value),
    Below(Value),
    AtLeast(Value),
    AtMost(Value),
    Between(Value, Value),
    Is(Term),
    IsNot(Term),
    IsAssigned,
    IsEmpty,
    Contains(Term),
    InList(Vec<Term>),
}

impl Op {
    /// The `horis:` local name of the operator, for domain-term messages and
    /// proof rendering ("exceeds", "is", …).
    pub fn name(&self) -> &'static str {
        match self {
            Op::Exceeds(_) => "exceeds",
            Op::Below(_) => "below",
            Op::AtLeast(_) => "atLeast",
            Op::AtMost(_) => "atMost",
            Op::Between(_, _) => "between",
            Op::Is(_) => "is",
            Op::IsNot(_) => "isNot",
            Op::IsAssigned => "isAssigned",
            Op::IsEmpty => "isEmpty",
            Op::Contains(_) => "contains",
            Op::InList(_) => "inList",
        }
    }
}

/// One leaf condition: a property of the rule's bound subject, tested by one
/// operator. `unit`, when stated, was already checked against the property's
/// declaration at load (D6: strict units).
#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    pub property: Iri,
    pub op: Op,
    pub unit: Option<String>,
}

/// The `horis:when` body: leaf conditions under the three combinators
/// (`horis:all` ∧, `horis:any` ∨, `horis:none` ¬∨), arbitrarily nested.
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionTree {
    Leaf(Condition),
    All(Vec<ConditionTree>),
    Any(Vec<ConditionTree>),
    NoneOf(Vec<ConditionTree>),
}

/// The `horis:then` body: assert `property = value` on the bound subject,
/// through the gate, with `Provenance::Rule`.
#[derive(Debug, Clone, PartialEq)]
pub struct Conclusion {
    pub property: Iri,
    pub value: Term,
}

/// One parsed `horis:Rule`. `subject_class` is the single class every
/// condition's `horis:subject` names (v1 rules bind one subject; see
/// [`OntologyError::RuleMixedSubjects`]).
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub iri: Iri,
    /// `horis:priority`, defaulting to 0 when undeclared. Orders candidate
    /// selection during forward chaining (doc 04) — higher fires first.
    pub priority: i64,
    pub subject_class: Iri,
    pub when: ConditionTree,
    pub then: Conclusion,
    pub explain: Option<String>,
}

impl ConditionTree {
    /// Every leaf condition, in declaration order.
    pub fn leaves(&self) -> Vec<&Condition> {
        let mut out = Vec::new();
        self.collect_leaves(&mut out);
        out
    }

    fn collect_leaves<'a>(&'a self, out: &mut Vec<&'a Condition>) {
        match self {
            ConditionTree::Leaf(c) => out.push(c),
            ConditionTree::All(children)
            | ConditionTree::Any(children)
            | ConditionTree::NoneOf(children) => {
                for child in children {
                    child.collect_leaves(out);
                }
            }
        }
    }
}

// -- Parsing ----------------------------------------------------------------

/// Parses every `horis:Rule` across `documents` against the built [`Schema`],
/// returning rules sorted by (priority descending, rule IRI lexicographic on
/// the resolved string — doc 04's documented tie-break, never load order).
///
/// Every failure is an [`OntologyError`] naming the rule in domain terms;
/// callers surface it as a *load* error.
pub fn parse_rules<'a>(
    documents: impl IntoIterator<Item = &'a Document>,
    schema: &Schema,
    interner: &Interner,
) -> Result<Vec<Rule>, OntologyError> {
    let graph = SubjectGraph::collect(documents, interner);
    if graph.anonymous_rule {
        return Err(OntologyError::RuleAnonymous);
    }

    let mut rules = Vec::new();
    for &rule_iri in &graph.rule_ids {
        rules.push(parse_rule(rule_iri, &graph, schema, interner)?);
    }

    rules.sort_by(|a, b| {
        b.priority.cmp(&a.priority).then_with(|| {
            let a_iri = interner.resolve(a.iri);
            let b_iri = interner.resolve(b.iri);
            a_iri.cmp(b_iri)
        })
    });
    Ok(rules)
}

/// All triples regrouped subject-major, plus the set of `a horis:Rule`
/// subjects — the shape rule parsing walks. Predicate/object pairs stay in
/// source order per subject, so parsing is deterministic.
struct SubjectGraph {
    by_subject: BTreeMap<Iri, Vec<(Iri, Object)>>,
    rule_ids: Vec<Iri>,
    /// True when some rule was declared on an anonymous (blank) subject —
    /// refused at load: oxttl mints blank ids *randomly per parse*, so an
    /// anonymous rule's identity would differ on every load, leaking
    /// nondeterminism into priority tie-breaks, error text, firing records,
    /// and traces ("same ontology → same trace", doc 04).
    anonymous_rule: bool,
}

impl SubjectGraph {
    fn collect<'a>(documents: impl IntoIterator<Item = &'a Document>, interner: &Interner) -> Self {
        let mut by_subject: BTreeMap<Iri, Vec<(Iri, Object)>> = BTreeMap::new();
        let mut rule_ids = Vec::new();
        let mut anonymous_rule = false;
        for document in documents {
            for Triple {
                subject,
                predicate,
                object,
            } in &document.triples
            {
                let subject_node = *subject;
                let subject = node_iri(subject_node);
                if interner.resolve(*predicate) == RDF_TYPE
                    && let Object::Node(node) = object
                    && interner.resolve(node_iri(*node)) == HORIS_RULE
                {
                    if matches!(subject_node, Node::Blank(_)) {
                        anonymous_rule = true;
                    } else if !rule_ids.contains(&subject) {
                        rule_ids.push(subject);
                    }
                }
                by_subject
                    .entry(subject)
                    .or_default()
                    .push((*predicate, object.clone()));
            }
        }
        // Rule order must not depend on document load order for error
        // reporting either: sort by resolved IRI.
        rule_ids.sort_by(|&a, &b| interner.resolve(a).cmp(interner.resolve(b)));
        Self {
            by_subject,
            rule_ids,
            anonymous_rule,
        }
    }

    fn pairs(&self, subject: Iri) -> &[(Iri, Object)] {
        self.by_subject
            .get(&subject)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// The first object for `predicate` on `subject`, in source order.
    fn object<'a>(
        &'a self,
        subject: Iri,
        predicate: &str,
        interner: &Interner,
    ) -> Option<&'a Object> {
        self.pairs(subject)
            .iter()
            .find(|(p, _)| interner.resolve(*p) == predicate)
            .map(|(_, o)| o)
    }

    /// Walks the `rdf:first`/`rdf:rest` chain from `head`, yielding each
    /// `rdf:first` object in list order. A cell revisited (a hand-authored
    /// cyclic list — `_:l rdf:rest _:l`) ends the walk instead of hanging
    /// the load; the truncation surfaces downstream as the arity/operand
    /// error the malformed list deserves.
    fn list(&self, head: Node, interner: &Interner) -> Vec<Object> {
        let mut out = Vec::new();
        let mut visited = std::collections::BTreeSet::new();
        let mut cursor = head;
        loop {
            let cell = node_iri(cursor);
            if interner.resolve(cell) == RDF_NIL || !visited.insert(cell) {
                break;
            }
            match self.object(cell, RDF_FIRST, interner) {
                Some(first) => out.push(first.clone()),
                None => break,
            }
            match self.object(cell, RDF_REST, interner) {
                Some(Object::Node(next)) => cursor = *next,
                _ => break,
            }
        }
        out
    }
}

/// The running "one subject class per rule" state while walking conditions.
struct RuleCx<'a> {
    rule: Iri,
    schema: &'a Schema,
    interner: &'a Interner,
    subject_class: Option<Iri>,
}

impl RuleCx<'_> {
    fn rule_name(&self) -> String {
        self.interner.resolve(self.rule).to_owned()
    }
}

fn parse_rule(
    rule_iri: Iri,
    graph: &SubjectGraph,
    schema: &Schema,
    interner: &Interner,
) -> Result<Rule, OntologyError> {
    let rule_name = || interner.resolve(rule_iri).to_owned();

    let priority = match graph.object(rule_iri, HORIS_PRIORITY, interner) {
        Some(Object::Literal(Literal::Typed(Value::Integer(p)))) => *p,
        Some(other) => {
            return Err(OntologyError::RuleInvalidPriority {
                rule: rule_name(),
                value: object_text(other, interner),
            });
        }
        None => 0,
    };

    let explain = match graph.object(rule_iri, HORIS_EXPLAIN, interner) {
        Some(Object::Literal(Literal::Typed(Value::String(s)))) => Some(s.clone()),
        _ => None,
    };

    let mut cx = RuleCx {
        rule: rule_iri,
        schema,
        interner,
        subject_class: None,
    };

    let when_node = match graph.object(rule_iri, HORIS_WHEN, interner) {
        Some(Object::Node(node)) => *node,
        _ => {
            return Err(OntologyError::RuleMissingPart {
                rule: rule_name(),
                part: "horis:when".to_owned(),
            });
        }
    };
    let when = parse_condition_node(when_node, graph, &mut cx, &mut BTreeSet::new())?;

    let then_node = match graph.object(rule_iri, HORIS_THEN, interner) {
        Some(Object::Node(node)) => *node,
        _ => {
            return Err(OntologyError::RuleMissingPart {
                rule: rule_name(),
                part: "horis:then".to_owned(),
            });
        }
    };
    let then = parse_conclusion(then_node, graph, &mut cx)?;

    let subject_class = cx
        .subject_class
        .ok_or_else(|| OntologyError::RuleMissingPart {
            rule: rule_name(),
            part: "horis:subject on its conditions".to_owned(),
        })?;

    Ok(Rule {
        iri: rule_iri,
        priority,
        subject_class,
        when,
        then,
        explain,
    })
}

/// Parses a node that is either a combinator (`horis:all/any/none` over a
/// list) or a leaf condition. `on_path` carries the condition nodes above
/// this one — a node reachable from itself (`_:a horis:all ( _:a )`, valid
/// Turtle) is a load error, not unbounded recursion.
fn parse_condition_node(
    node: Node,
    graph: &SubjectGraph,
    cx: &mut RuleCx<'_>,
    on_path: &mut BTreeSet<Iri>,
) -> Result<ConditionTree, OntologyError> {
    let subject = node_iri(node);
    if !on_path.insert(subject) {
        return Err(OntologyError::RuleMissingPart {
            rule: cx.rule_name(),
            part: "an acyclic horis:when body (a condition node contains itself)".to_owned(),
        });
    }

    type CombinatorMatch = (&'static str, fn(Vec<ConditionTree>) -> ConditionTree, Node);
    let mut found: Option<CombinatorMatch> = None;
    for (combinator, name, build) in [
        (HORIS_ALL, "all", ConditionTree::All as fn(_) -> _),
        (HORIS_ANY, "any", ConditionTree::Any as fn(_) -> _),
        (HORIS_NONE, "none", ConditionTree::NoneOf as fn(_) -> _),
    ] {
        if let Some(Object::Node(head)) = graph.object(subject, combinator, cx.interner) {
            if let Some((existing, _, _)) = found {
                return Err(OntologyError::RuleMultipleCombinators {
                    rule: cx.rule_name(),
                    a: existing.to_owned(),
                    b: name.to_owned(),
                });
            }
            found = Some((name, build, *head));
        }
    }

    let result = if let Some((_, build, head)) = found {
        let mut children = Vec::new();
        for item in graph.list(head, cx.interner) {
            let Object::Node(child) = item else {
                return Err(OntologyError::RuleMissingPart {
                    rule: cx.rule_name(),
                    part: "a condition node inside its combinator list".to_owned(),
                });
            };
            children.push(parse_condition_node(child, graph, cx, on_path)?);
        }
        Ok(build(children))
    } else {
        Ok(ConditionTree::Leaf(parse_leaf_condition(node, graph, cx)?))
    };
    on_path.remove(&subject);
    result
}

fn parse_leaf_condition(
    node: Node,
    graph: &SubjectGraph,
    cx: &mut RuleCx<'_>,
) -> Result<Condition, OntologyError> {
    let subject = node_iri(node);

    // horis:subject — bind (and cross-check) the rule's one subject class.
    if let Some(Object::Node(class_node)) = graph.object(subject, HORIS_SUBJECT, cx.interner) {
        let class = node_iri(*class_node);
        if cx.schema.class_declared(class).is_none() {
            return Err(OntologyError::RuleUndeclaredSubject {
                rule: cx.rule_name(),
                class: cx.interner.resolve(class).to_owned(),
            });
        }
        match cx.subject_class {
            None => cx.subject_class = Some(class),
            Some(existing) if canonically_equal(cx.schema, existing, class) => {}
            Some(existing) => {
                return Err(OntologyError::RuleMixedSubjects {
                    rule: cx.rule_name(),
                    a: cx.interner.resolve(existing).to_owned(),
                    b: cx.interner.resolve(class).to_owned(),
                });
            }
        }
    }

    let property = match graph.object(subject, HORIS_PROPERTY, cx.interner) {
        Some(Object::Node(p)) => node_iri(*p),
        _ => {
            return Err(OntologyError::RuleMissingPart {
                rule: cx.rule_name(),
                part: "horis:property on a condition".to_owned(),
            });
        }
    };
    let property_decl =
        cx.schema
            .property(property)
            .ok_or_else(|| OntologyError::RuleUndeclaredProperty {
                rule: cx.rule_name(),
                property: cx.interner.resolve(property).to_owned(),
            })?;

    // D6 follow-through: a condition may state horis:unit; stated units must
    // exactly equal the property's declared unit (strict, opaque labels).
    // A stated unit must be a plain string — a mistyped one silently
    // skipping the D6 check would defeat "mismatch is a load-time error".
    let unit = match graph.object(subject, HORIS_UNIT, cx.interner) {
        Some(Object::Literal(Literal::Typed(Value::String(u)))) => Some(u.clone()),
        Some(other) => {
            return Err(OntologyError::RuleInvalidOperand {
                rule: cx.rule_name(),
                operator: "horis:unit".to_owned(),
                value: object_text(other, cx.interner),
                expected: "a string literal".to_owned(),
            });
        }
        None => None,
    };
    if let Some(stated) = &unit {
        match &property_decl.unit {
            Some(declared) if declared == stated => {}
            Some(declared) => {
                return Err(OntologyError::RuleUnitMismatch {
                    rule: cx.rule_name(),
                    property: cx.interner.resolve(property).to_owned(),
                    stated: stated.clone(),
                    declared: declared.clone(),
                });
            }
            None => {
                return Err(OntologyError::RuleUnitWithoutDeclaration {
                    rule: cx.rule_name(),
                    property: cx.interner.resolve(property).to_owned(),
                    stated: stated.clone(),
                });
            }
        }
    }

    let op = parse_operator(subject, graph, cx, property)?;

    Ok(Condition { property, op, unit })
}

fn parse_operator(
    subject: Iri,
    graph: &SubjectGraph,
    cx: &mut RuleCx<'_>,
    property: Iri,
) -> Result<Op, OntologyError> {
    let mut found: Option<Op> = None;
    for (predicate, object) in graph.pairs(subject) {
        let predicate = cx.interner.resolve(*predicate);
        let op = match predicate {
            HORIS_EXCEEDS => Some(Op::Exceeds(literal_operand(object, predicate, cx)?)),
            HORIS_BELOW => Some(Op::Below(literal_operand(object, predicate, cx)?)),
            HORIS_AT_LEAST => Some(Op::AtLeast(literal_operand(object, predicate, cx)?)),
            HORIS_AT_MOST => Some(Op::AtMost(literal_operand(object, predicate, cx)?)),
            HORIS_BETWEEN => {
                let Object::Node(head) = object else {
                    return Err(OntologyError::RuleBetweenArity {
                        rule: cx.rule_name(),
                        found: 0,
                    });
                };
                let items = graph.list(*head, cx.interner);
                // Every member must be a typed literal — silently dropping
                // a bad member would load `between ( 100 :oops 200 )` as
                // `between 100 and 200`, changing semantics without a word.
                let mut values = Vec::new();
                for item in &items {
                    match item {
                        Object::Literal(Literal::Typed(v)) => values.push(v.clone()),
                        other => {
                            return Err(OntologyError::RuleInvalidOperand {
                                rule: cx.rule_name(),
                                operator: "horis:between".to_owned(),
                                value: object_text(other, cx.interner),
                                expected: "a typed literal list member".to_owned(),
                            });
                        }
                    }
                }
                if values.len() != 2 {
                    return Err(OntologyError::RuleBetweenArity {
                        rule: cx.rule_name(),
                        found: values.len(),
                    });
                }
                Some(Op::Between(values[0].clone(), values[1].clone()))
            }
            HORIS_IS => Some(Op::Is(term_operand(object))),
            HORIS_IS_NOT => Some(Op::IsNot(term_operand(object))),
            HORIS_IS_ASSIGNED => match object {
                Object::Literal(Literal::Typed(Value::Boolean(true))) => Some(Op::IsAssigned),
                other => {
                    return Err(OntologyError::RuleInvalidOperand {
                        rule: cx.rule_name(),
                        operator: "horis:isAssigned".to_owned(),
                        value: object_text(other, cx.interner),
                        expected: "the literal true".to_owned(),
                    });
                }
            },
            HORIS_IS_EMPTY => match object {
                Object::Literal(Literal::Typed(Value::Boolean(true))) => Some(Op::IsEmpty),
                other => {
                    return Err(OntologyError::RuleInvalidOperand {
                        rule: cx.rule_name(),
                        operator: "horis:isEmpty".to_owned(),
                        value: object_text(other, cx.interner),
                        expected: "the literal true".to_owned(),
                    });
                }
            },
            HORIS_CONTAINS => Some(Op::Contains(term_operand(object))),
            HORIS_IN_LIST => {
                let Object::Node(head) = object else {
                    return Err(OntologyError::RuleInvalidOperand {
                        rule: cx.rule_name(),
                        operator: "horis:inList".to_owned(),
                        value: object_text(object, cx.interner),
                        expected: "a list".to_owned(),
                    });
                };
                let items = graph
                    .list(*head, cx.interner)
                    .iter()
                    .map(term_operand)
                    .collect();
                Some(Op::InList(items))
            }
            _ => None,
        };
        if let Some(op) = op {
            if let Some(existing) = &found {
                return Err(OntologyError::RuleMultipleOperators {
                    rule: cx.rule_name(),
                    property: cx.interner.resolve(property).to_owned(),
                    a: existing.name().to_owned(),
                    b: op.name().to_owned(),
                });
            }
            found = Some(op);
        }
    }
    found.ok_or_else(|| OntologyError::RuleMissingOperator {
        rule: cx.rule_name(),
        property: cx.interner.resolve(property).to_owned(),
    })
}

fn parse_conclusion(
    node: Node,
    graph: &SubjectGraph,
    cx: &mut RuleCx<'_>,
) -> Result<Conclusion, OntologyError> {
    let subject = node_iri(node);
    let property = match graph.object(subject, HORIS_PROPERTY, cx.interner) {
        Some(Object::Node(p)) => node_iri(*p),
        _ => {
            return Err(OntologyError::RuleMissingPart {
                rule: cx.rule_name(),
                part: "horis:property on its conclusion".to_owned(),
            });
        }
    };
    if cx.schema.property(property).is_none() {
        return Err(OntologyError::RuleUndeclaredProperty {
            rule: cx.rule_name(),
            property: cx.interner.resolve(property).to_owned(),
        });
    }
    let value = match graph.object(subject, HORIS_VALUE, cx.interner) {
        Some(object) => term_operand(object),
        None => {
            return Err(OntologyError::RuleMissingPart {
                rule: cx.rule_name(),
                part: "horis:value on its conclusion".to_owned(),
            });
        }
    };
    Ok(Conclusion { property, value })
}

/// True when `a` and `b` name the same class through the `owl:sameAs`
/// equivalence (D-7: class positions are canonical-aware). Compares the
/// canonical representatives directly — `class_declared` would answer each
/// *declared* twin with itself, wrongly splitting an anchored pair in which
/// both sides are declared classes.
fn canonically_equal(schema: &Schema, a: Iri, b: Iri) -> bool {
    a == b || schema.canonical_iri(a) == schema.canonical_iri(b)
}

fn literal_operand(
    object: &Object,
    operator: &str,
    cx: &RuleCx<'_>,
) -> Result<Value, OntologyError> {
    match object {
        Object::Literal(Literal::Typed(v)) => Ok(v.clone()),
        other => Err(OntologyError::RuleInvalidOperand {
            rule: cx.rule_name(),
            // Rendered as the authored `horis:` prefixed form, matching the
            // isAssigned/isEmpty/inList error paths.
            operator: operator
                .strip_prefix("https://w3id.org/horismos#")
                .map(|local| format!("horis:{local}"))
                .unwrap_or_else(|| operator.to_owned()),
            value: object_text(other, cx.interner),
            expected: "a typed literal".to_owned(),
        }),
    }
}

fn term_operand(object: &Object) -> Term {
    match object {
        Object::Node(node) => Term::Iri(node_iri(*node)),
        Object::Literal(Literal::Typed(v)) => Term::Literal(v.clone()),
        Object::Literal(Literal::Other { lexical, .. })
        | Object::Literal(Literal::LangString { lexical, .. }) => {
            Term::Literal(Value::String(lexical.clone()))
        }
    }
}

fn object_text(object: &Object, interner: &Interner) -> String {
    match object {
        Object::Node(node) => interner.resolve(node_iri(*node)).to_owned(),
        Object::Literal(Literal::Typed(v)) => v.lexical_form().unwrap_or_default(),
        Object::Literal(Literal::Other { lexical, .. })
        | Object::Literal(Literal::LangString { lexical, .. }) => lexical.clone(),
    }
}

fn node_iri(node: Node) -> Iri {
    match node {
        Node::Named(iri) | Node::Blank(iri) => iri,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ontology;
    use crate::error::OntologyError;

    const PRELUDE: &str = "@prefix : <https://example.org/t#> .\n\
         @prefix horis: <https://w3id.org/horismos#> .\n\
         @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n";

    /// A minimal patient-shaped schema for inline rule fixtures.
    const DOMAIN: &str = ":Patient a rdfs:Class .\n\
         :Symptom a rdfs:Class .\n\
         :ChestPain rdfs:subClassOf :Symptom .\n\
         :AcuityLevel a rdfs:Class ; horis:oneOf ( :Immediate :Urgent :Standard ) .\n\
         :heartRate a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:decimal ;\n\
         \x20   horis:unit \"bpm\" ; horis:min 0 ; horis:max 300 ; horis:cardinality \"0..1\" .\n\
         :hasSymptom a rdf:Property ; rdfs:domain :Patient ; rdfs:range :Symptom ;\n\
         \x20   horis:cardinality \"0..*\" .\n\
         :acuity a rdf:Property ; rdfs:domain :Patient ; rdfs:range :AcuityLevel ;\n\
         \x20   horis:cardinality \"0..1\" ; horis:assignedBy horis:RulesOnly .\n";

    pub(super) fn load(rules: &str) -> Result<Ontology, OntologyError> {
        Ontology::load_from_str(&format!("{PRELUDE}{DOMAIN}{rules}\n"), |_| None)
    }

    fn triage() -> Ontology {
        Ontology::load(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/triage/triage.ttl"
        ))
        .expect("triage.ttl must load")
    }

    #[test]
    fn triage_ttl_yields_exactly_r1_with_its_documented_shape() {
        let onto = triage();
        let rules = onto.rules();
        assert_eq!(rules.len(), 1, "triage.ttl declares exactly :r1");

        let r1 = &rules[0];
        assert_eq!(onto.resolve(r1.iri), "https://example.org/triage#r1");
        assert_eq!(r1.priority, 10);
        assert_eq!(
            r1.explain.as_deref(),
            Some("Tachycardia with chest pain — immediate track")
        );
        assert_eq!(
            onto.resolve(r1.subject_class),
            "https://example.org/triage#Patient"
        );

        // when: all( heartRate exceeds 120, hasSymptom is :ChestPain )
        let ConditionTree::All(children) = &r1.when else {
            panic!(
                "r1's when must be a horis:all combinator, got {:?}",
                r1.when
            );
        };
        assert_eq!(children.len(), 2);
        let ConditionTree::Leaf(first) = &children[0] else {
            panic!("expected a leaf condition");
        };
        assert_eq!(
            onto.resolve(first.property),
            "https://example.org/triage#heartRate"
        );
        assert_eq!(first.op, Op::Exceeds(Value::Integer(120)));
        let ConditionTree::Leaf(second) = &children[1] else {
            panic!("expected a leaf condition");
        };
        assert_eq!(
            onto.resolve(second.property),
            "https://example.org/triage#hasSymptom"
        );
        let chest_pain = onto
            .iri("https://example.org/triage#ChestPain")
            .expect("ChestPain");
        assert_eq!(second.op, Op::Is(Term::Iri(chest_pain)));

        // then: acuity = :Immediate
        assert_eq!(
            onto.resolve(r1.then.property),
            "https://example.org/triage#acuity"
        );
        let immediate = onto
            .iri("https://example.org/triage#Immediate")
            .expect("Immediate");
        assert_eq!(r1.then.value, Term::Iri(immediate));
    }

    #[test]
    fn rules_are_sorted_by_priority_desc_then_iri_lexicographic() {
        let onto = load(
            ":rB a horis:Rule ; horis:priority 5 ;\n\
             \x20 horis:when [ horis:all ( [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 100 ] ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Urgent ] .\n\
             :rA a horis:Rule ; horis:priority 5 ;\n\
             \x20 horis:when [ horis:all ( [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 110 ] ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Urgent ] .\n\
             :rTop a horis:Rule ; horis:priority 10 ;\n\
             \x20 horis:when [ horis:all ( [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 120 ] ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Immediate ] .",
        )
        .expect("three valid rules must load");
        let names: Vec<&str> = onto.rules().iter().map(|r| onto.resolve(r.iri)).collect();
        assert_eq!(
            names,
            vec![
                "https://example.org/t#rTop",
                "https://example.org/t#rA",
                "https://example.org/t#rB",
            ],
            "priority desc, ties by rule IRI lexicographic"
        );
    }

    #[test]
    fn a_rule_without_priority_defaults_to_zero() {
        let onto = load(
            ":r a horis:Rule ;\n\
             \x20 horis:when [ horis:all ( [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 100 ] ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Urgent ] .",
        )
        .expect("valid");
        assert_eq!(onto.rules()[0].priority, 0);
    }

    #[test]
    fn an_undeclared_condition_property_is_a_load_error_naming_the_rule() {
        let err = load(
            ":r9 a horis:Rule ;\n\
             \x20 horis:when [ horis:all ( [ horis:subject :Patient ; horis:property :pulse ; horis:exceeds 100 ] ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Urgent ] .",
        )
        .expect_err("pulse is not declared");
        let text = err.domain_text();
        assert!(text.contains("t#r9"), "{text}");
        assert!(text.contains("t#pulse"), "{text}");
        assert!(text.contains("not a declared property"), "{text}");
    }

    #[test]
    fn an_undeclared_conclusion_property_is_a_load_error() {
        let err = load(
            ":r9 a horis:Rule ;\n\
             \x20 horis:when [ horis:all ( [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 100 ] ) ] ;\n\
             \x20 horis:then [ horis:property :disposition ; horis:value :Urgent ] .",
        )
        .expect_err("disposition is not declared");
        assert!(
            err.domain_text().contains("t#disposition"),
            "{}",
            err.domain_text()
        );
    }

    #[test]
    fn an_undeclared_subject_class_is_a_load_error() {
        let err = load(
            ":r9 a horis:Rule ;\n\
             \x20 horis:when [ horis:all ( [ horis:subject :Ghost ; horis:property :heartRate ; horis:exceeds 100 ] ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Urgent ] .",
        )
        .expect_err(":Ghost is not declared");
        let text = err.domain_text();
        assert!(text.contains("t#r9"), "{text}");
        assert!(text.contains("t#Ghost"), "{text}");
    }

    #[test]
    fn mixed_subject_classes_are_a_load_error() {
        let err = load(
            ":r9 a horis:Rule ;\n\
             \x20 horis:when [ horis:all (\n\
             \x20   [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 100 ]\n\
             \x20   [ horis:subject :Symptom ; horis:property :hasSymptom ; horis:isAssigned true ]\n\
             \x20 ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Urgent ] .",
        )
        .expect_err("v1 rules bind one subject class");
        let text = err.domain_text();
        assert!(text.contains("t#Patient"), "{text}");
        assert!(text.contains("t#Symptom"), "{text}");
        assert!(text.contains("one subject"), "{text}");
    }

    #[test]
    fn a_missing_when_is_a_load_error() {
        let err = load(
            ":r9 a horis:Rule ; horis:then [ horis:property :acuity ; horis:value :Urgent ] .",
        )
        .expect_err("no when");
        let text = err.domain_text();
        assert!(text.contains("t#r9"), "{text}");
        assert!(text.contains("horis:when"), "{text}");
    }

    #[test]
    fn a_missing_then_is_a_load_error() {
        let err = load(
            ":r9 a horis:Rule ;\n\
             \x20 horis:when [ horis:all ( [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 100 ] ) ] .",
        )
        .expect_err("no then");
        assert!(
            err.domain_text().contains("horis:then"),
            "{}",
            err.domain_text()
        );
    }

    #[test]
    fn a_condition_without_an_operator_is_a_load_error() {
        let err = load(
            ":r9 a horis:Rule ;\n\
             \x20 horis:when [ horis:all ( [ horis:subject :Patient ; horis:property :heartRate ] ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Urgent ] .",
        )
        .expect_err("no operator stated");
        let text = err.domain_text();
        assert!(text.contains("t#heartRate"), "{text}");
        assert!(text.contains("operator"), "{text}");
    }

    #[test]
    fn a_rule_side_unit_mismatch_is_a_load_error_in_domain_terms() {
        // D6 (strict units), rule-side follow-through: heartRate is declared
        // in "bpm"; a condition stating "mmHg" is a load error, never a
        // runtime surprise.
        let err = load(
            ":r9 a horis:Rule ;\n\
             \x20 horis:when [ horis:all ( [ horis:subject :Patient ; horis:property :heartRate ;\n\
             \x20   horis:unit \"mmHg\" ; horis:exceeds 100 ] ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Urgent ] .",
        )
        .expect_err("mmHg contradicts the declared bpm");
        assert_eq!(
            err.domain_text(),
            "rule https://example.org/t#r9 https://example.org/t#heartRate condition states \
             horis:unit \"mmHg\" but https://example.org/t#heartRate is declared in \"bpm\"",
            "the exact D6 wording, naming rule, property, and both units"
        );
    }

    #[test]
    fn a_matching_rule_side_unit_is_accepted() {
        load(
            ":r a horis:Rule ;\n\
             \x20 horis:when [ horis:all ( [ horis:subject :Patient ; horis:property :heartRate ;\n\
             \x20   horis:unit \"bpm\" ; horis:exceeds 100 ] ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Urgent ] .",
        )
        .expect("an exactly-matching stated unit is fine");
    }

    #[test]
    fn a_stated_unit_on_a_unitless_property_is_a_load_error() {
        let err = load(
            ":r9 a horis:Rule ;\n\
             \x20 horis:when [ horis:all ( [ horis:subject :Patient ; horis:property :hasSymptom ;\n\
             \x20   horis:unit \"bpm\" ; horis:isAssigned true ] ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Urgent ] .",
        )
        .expect_err("hasSymptom declares no unit");
        let text = err.domain_text();
        assert!(text.contains("hasSymptom"), "{text}");
        assert!(text.contains("no unit"), "{text}");
    }

    #[test]
    fn between_requires_exactly_two_values() {
        let err = load(
            ":r9 a horis:Rule ;\n\
             \x20 horis:when [ horis:all ( [ horis:subject :Patient ; horis:property :heartRate ;\n\
             \x20   horis:between ( 100 ) ] ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Urgent ] .",
        )
        .expect_err("between wants two bounds");
        assert!(
            err.domain_text().contains("two values"),
            "{}",
            err.domain_text()
        );
    }

    #[test]
    fn between_with_two_values_parses() {
        let onto = load(
            ":r a horis:Rule ;\n\
             \x20 horis:when [ horis:all ( [ horis:subject :Patient ; horis:property :heartRate ;\n\
             \x20   horis:between ( 100 120 ) ] ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Urgent ] .",
        )
        .expect("valid");
        let leaves = onto.rules()[0].when.leaves();
        assert_eq!(
            leaves[0].op,
            Op::Between(Value::Integer(100), Value::Integer(120))
        );
    }

    #[test]
    fn two_operators_on_one_condition_are_a_load_error() {
        let err = load(
            ":r9 a horis:Rule ;\n\
             \x20 horis:when [ horis:all ( [ horis:subject :Patient ; horis:property :heartRate ;\n\
             \x20   horis:exceeds 100 ; horis:below 200 ] ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Urgent ] .",
        )
        .expect_err("one operator per condition");
        let text = err.domain_text();
        assert!(text.contains("exceeds"), "{text}");
        assert!(text.contains("below"), "{text}");
    }

    #[test]
    fn nested_combinators_parse() {
        let onto = load(
            ":r a horis:Rule ;\n\
             \x20 horis:when [ horis:any (\n\
             \x20   [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 150 ]\n\
             \x20   [ horis:all (\n\
             \x20     [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 120 ]\n\
             \x20     [ horis:subject :Patient ; horis:property :hasSymptom ; horis:is :ChestPain ]\n\
             \x20   ) ]\n\
             \x20 ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Immediate ] .",
        )
        .expect("nesting is part of the grammar");
        let ConditionTree::Any(children) = &onto.rules()[0].when else {
            panic!("outer combinator must be Any");
        };
        assert_eq!(children.len(), 2);
        assert!(matches!(children[1], ConditionTree::All(_)));
    }

    #[test]
    fn in_list_parses_its_members_in_order() {
        let onto = load(
            ":r a horis:Rule ;\n\
             \x20 horis:when [ horis:all ( [ horis:subject :Patient ; horis:property :acuity ;\n\
             \x20   horis:inList ( :Immediate :Urgent ) ] ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Immediate ] .",
        )
        .expect("valid");
        let leaves = onto.rules()[0].when.leaves();
        let immediate = onto
            .iri("https://example.org/t#Immediate")
            .expect("interned");
        let urgent = onto.iri("https://example.org/t#Urgent").expect("interned");
        assert_eq!(
            leaves[0].op,
            Op::InList(vec![Term::Iri(immediate), Term::Iri(urgent)])
        );
    }

    #[test]
    fn an_ontology_without_rules_has_an_empty_rule_slice() {
        let onto = load("").expect("no rules is fine");
        assert!(onto.rules().is_empty());
    }
}

#[cfg(test)]
mod strictness_tests {
    use super::tests::load;
    use crate::error::OntologyError;

    #[test]
    fn an_anonymous_rule_is_a_load_error() {
        // Blank ids are minted per parse — an anonymous rule would have a
        // different identity on every load (PRP-002 review).
        let err = load(
            "[] a horis:Rule ; horis:priority 1 ;\n\
             \x20 horis:when [ horis:all ( [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 100 ] ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Urgent ] .",
        )
        .expect_err("rules must be named");
        assert!(matches!(err, OntologyError::RuleAnonymous));
        assert!(
            err.domain_text().contains("must be named"),
            "{}",
            err.domain_text()
        );
    }

    #[test]
    fn a_cyclic_between_list_terminates_as_an_arity_error() {
        // `_:l rdf:rest _:l` — a hand-authored cyclic list must not hang
        // the load; the truncated walk surfaces as the arity error.
        let err = load(
            ":r a horis:Rule ; horis:priority 1 ;\n\
             \x20 horis:when [ horis:all ( [ horis:subject :Patient ; horis:property :heartRate ;\n\
             \x20   horis:between _:cyc ] ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Urgent ] .\n\
             _:cyc rdf:first 100 .\n\
             _:cyc rdf:rest _:cyc .",
        )
        .expect_err("a cyclic list is malformed, not an infinite loop");
        assert!(
            matches!(err, OntologyError::RuleBetweenArity { .. }),
            "got {err:?}"
        );
    }

    #[test]
    fn a_self_containing_condition_node_is_a_load_error() {
        // `_:a horis:all ( _:a )` is valid Turtle; without the path guard
        // it recursed to stack overflow (PRP-002 review).
        let err = load(
            ":r a horis:Rule ; horis:priority 1 ;\n\
             \x20 horis:when _:a ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Urgent ] .\n\
             _:a horis:all ( _:a ) .",
        )
        .expect_err("a condition node containing itself is malformed");
        assert!(
            err.domain_text().contains("acyclic"),
            "{}",
            err.domain_text()
        );
    }

    #[test]
    fn two_combinators_on_one_node_are_a_load_error() {
        let err = load(
            ":r a horis:Rule ; horis:priority 1 ;\n\
             \x20 horis:when [\n\
             \x20   horis:all ( [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 100 ] ) ;\n\
             \x20   horis:any ( [ horis:subject :Patient ; horis:property :heartRate ; horis:below 50 ] )\n\
             \x20 ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Urgent ] .",
        )
        .expect_err("a node is one combinator or one leaf");
        let text = err.domain_text();
        assert!(text.contains("horis:all"), "{text}");
        assert!(text.contains("horis:any"), "{text}");
    }

    #[test]
    fn a_non_literal_between_member_is_a_load_error_not_a_silent_drop() {
        // Without the strictness, `between ( 100 :oops 200 )` loaded as
        // `between 100 and 200` — semantics changed without a word.
        let err = load(
            ":r a horis:Rule ; horis:priority 1 ;\n\
             \x20 horis:when [ horis:all ( [ horis:subject :Patient ; horis:property :heartRate ;\n\
             \x20   horis:between ( 100 :oops 200 ) ] ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Urgent ] .",
        )
        .expect_err("every between member must be a typed literal");
        let text = err.domain_text();
        assert!(text.contains("horis:between"), "{text}");
        assert!(text.contains("t#oops"), "{text}");
    }

    #[test]
    fn a_non_string_unit_is_a_load_error_not_a_skipped_check() {
        // A mistyped unit silently skipping D6 would defeat "mismatch is a
        // load-time error".
        let err = load(
            ":r a horis:Rule ; horis:priority 1 ;\n\
             \x20 horis:when [ horis:all ( [ horis:subject :Patient ; horis:property :heartRate ;\n\
             \x20   horis:unit 42 ; horis:exceeds 100 ] ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Urgent ] .",
        )
        .expect_err("horis:unit must be a string");
        let text = err.domain_text();
        assert!(text.contains("horis:unit"), "{text}");
        assert!(text.contains("string"), "{text}");
    }
}
