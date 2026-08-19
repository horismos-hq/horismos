//! The typed `horis:Tool` contract model (U3): `horis:takes`/`horis:effect`/
//! `horis:binding`, parsed at [`crate::Ontology`] assembly — before
//! `workflow::model` (U2), whose `ActionStep.tool` cross-validates against
//! what this module parses.
//!
//! **The LLM never sees this.** A `Tool` is a declared contract the runtime
//! resolves arguments into and validates effects against — the sharpest
//! place Horismos inverts the conventional agent stack (doc 07).
//!
//! Only the `rust:` binding scheme is invocable in this build (D1, PRP-004
//! Key decision — Rust-resident loop, Rust-fn binding first). `python:`/
//! `mcp://`/`http(s):` parse (so a profile-valid ontology naming them still
//! loads) but resolve to [`Binding::Unsupported`] — `workflow::agent`
//! refuses to invoke one with a domain-term `ToolError`, gap-noted rather
//! than silently ignored.
//!
//! Spec: `docs/framework/07-tools.md` · Owner: PRP-004

use std::collections::BTreeMap;

use crate::error::OntologyError;
use crate::ontology::schema::Schema;
use crate::ontology::term::{Datatype, Interner, Iri, Value};
use crate::ontology::turtle::{Document, Literal, Object, Triple, xsd_datatype_from_iri};
use crate::reason::rule::{SubjectGraph, node_iri, object_text};

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const HORIS_TOOL: &str = "https://w3id.org/horismos#Tool";
const HORIS_TAKES: &str = "https://w3id.org/horismos#takes";
const HORIS_ARG: &str = "https://w3id.org/horismos#arg";
const HORIS_NAME: &str = "https://w3id.org/horismos#name";
const HORIS_TYPE: &str = "https://w3id.org/horismos#type";
const HORIS_EFFECT: &str = "https://w3id.org/horismos#effect";
const HORIS_ASSERTS: &str = "https://w3id.org/horismos#asserts";
const HORIS_PROPERTY: &str = "https://w3id.org/horismos#property";
const HORIS_VALUE: &str = "https://w3id.org/horismos#value";
const HORIS_BINDING: &str = "https://w3id.org/horismos#binding";
const HORIS_TIMEOUT: &str = "https://w3id.org/horismos#timeout";
const HORIS_IDEMPOTENT: &str = "https://w3id.org/horismos#idempotent";
const HORIS_RETRIES: &str = "https://w3id.org/horismos#retries";
const HORIS_NOW: &str = "https://w3id.org/horismos#now";

// -- Public model --------------------------------------------------------

/// An argument type: either a declared class or one of the six XSD
/// datatypes — the same two shapes a property's `rdfs:range` takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgType {
    Class(Iri),
    Datatype(Datatype),
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub name: String,
    pub ty: ArgType,
}

/// What one `horis:asserts` entry in a tool's `horis:effect` declares.
#[derive(Debug, Clone)]
pub enum EffectValue {
    /// `horis:value horis:now` — the transaction-stamped timestamp (U11),
    /// never a live wall-clock read.
    Now,
    /// `horis:type <…>` with no fixed `horis:value` — the tool's actual
    /// return supplies the value; the runtime checks only that it matches
    /// this declared shape before it may pass the gate.
    TypedPlaceholder(ArgType),
}

#[derive(Debug, Clone)]
pub struct EffectAssertion {
    pub property: Iri,
    pub value: EffectValue,
}

/// A parsed `horis:binding`. Only [`Binding::Rust`] is invocable in this
/// build; see the module doc.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Binding {
    /// `rust:<name>` — a name [`crate::workflow::agent::AgentBuilder::tool`]
    /// registers a [`crate::workflow::agent::Tool`] implementation under.
    Rust(String),
    /// Any other scheme (`python:`, `mcp://`, `http:`/`https:`) — parsed,
    /// not invocable; `workflow::agent` refuses with a domain-term
    /// `ToolError` naming the scheme.
    Unsupported { scheme: String, raw: String },
}

#[derive(Debug, Clone)]
pub struct Tool {
    pub iri: Iri,
    pub takes: Vec<Arg>,
    pub effect: Vec<EffectAssertion>,
    pub binding: Binding,
    pub timeout: Option<String>,
    pub idempotent: bool,
    pub retries: u32,
}

// -- Parsing ---------------------------------------------------------------

/// Parses every `horis:Tool` across `documents`.
pub fn parse_tools(
    documents: &[&Document],
    schema: &Schema,
    interner: &Interner,
) -> Result<BTreeMap<Iri, Tool>, OntologyError> {
    let graph = SubjectGraph::collect(documents.iter().copied(), interner);

    let mut tool_ids = Vec::new();
    for document in documents {
        for Triple {
            subject,
            predicate,
            object,
        } in &document.triples
        {
            if interner.resolve(*predicate) != RDF_TYPE {
                continue;
            }
            if let Object::Node(n) = object
                && interner.resolve(node_iri(*n)) == HORIS_TOOL
            {
                tool_ids.push(node_iri(*subject));
            }
        }
    }
    tool_ids.sort_by_key(|&iri| interner.resolve(iri).to_owned());
    tool_ids.dedup();

    let mut tools = BTreeMap::new();
    for tool_iri in tool_ids {
        let tool = parse_one_tool(tool_iri, &graph, schema, interner)?;
        tools.insert(tool_iri, tool);
    }
    Ok(tools)
}

fn parse_one_tool(
    iri: Iri,
    graph: &SubjectGraph,
    schema: &Schema,
    interner: &Interner,
) -> Result<Tool, OntologyError> {
    let name = || interner.resolve(iri).to_owned();

    let takes_node = graph.object(iri, HORIS_TAKES, interner);
    let mut takes = Vec::new();
    if let Some(Object::Node(n)) = takes_node {
        let takes_iri = node_iri(*n);
        for (predicate, object) in graph.pairs(takes_iri) {
            if interner.resolve(*predicate) != HORIS_ARG {
                continue;
            }
            let Object::Node(arg_node) = object else {
                continue;
            };
            let arg_iri = node_iri(*arg_node);
            let arg_name = match graph.object(arg_iri, HORIS_NAME, interner) {
                Some(Object::Literal(Literal::Typed(Value::String(s)))) => s.clone(),
                _ => {
                    return Err(OntologyError::ToolMissingPart {
                        tool: name(),
                        part: "horis:name on a horis:arg".to_owned(),
                    });
                }
            };
            let ty = parse_arg_type(iri, arg_iri, graph, schema, interner)?;
            takes.push(Arg { name: arg_name, ty });
        }
    }

    let effect_node = graph.object(iri, HORIS_EFFECT, interner);
    let mut effect = Vec::new();
    if let Some(Object::Node(n)) = effect_node {
        let effect_iri = node_iri(*n);
        for (predicate, object) in graph.pairs(effect_iri) {
            if interner.resolve(*predicate) != HORIS_ASSERTS {
                continue;
            }
            let Object::Node(assert_node) = object else {
                continue;
            };
            let assert_iri = node_iri(*assert_node);
            let property = match graph.object(assert_iri, HORIS_PROPERTY, interner) {
                Some(Object::Node(p)) => node_iri(*p),
                _ => {
                    return Err(OntologyError::ToolMissingPart {
                        tool: name(),
                        part: "horis:property on a horis:asserts".to_owned(),
                    });
                }
            };
            if schema.property(property).is_none() {
                return Err(OntologyError::ToolUndeclaredReference {
                    tool: name(),
                    name: interner.resolve(property).to_owned(),
                });
            }
            let value = match graph.object(assert_iri, HORIS_VALUE, interner) {
                Some(Object::Node(v)) if interner.resolve(node_iri(*v)) == HORIS_NOW => {
                    EffectValue::Now
                }
                Some(other) => {
                    return Err(OntologyError::ToolMissingPart {
                        tool: name(),
                        part: format!(
                            "horis:value on horis:asserts to be horis:now (got {}); a fixed non-`horis:now` value is not yet supported",
                            object_text(other, interner)
                        ),
                    });
                }
                None => EffectValue::TypedPlaceholder(parse_arg_type(
                    iri, assert_iri, graph, schema, interner,
                )?),
            };
            effect.push(EffectAssertion { property, value });
        }
    }
    if effect.is_empty() {
        return Err(OntologyError::ToolMissingPart {
            tool: name(),
            part: "horis:effect".to_owned(),
        });
    }

    let binding = match graph.object(iri, HORIS_BINDING, interner) {
        Some(Object::Literal(Literal::Typed(Value::String(s)))) => parse_binding(s),
        _ => {
            return Err(OntologyError::ToolMissingPart {
                tool: name(),
                part: "horis:binding".to_owned(),
            });
        }
    };

    let timeout = match graph.object(iri, HORIS_TIMEOUT, interner) {
        Some(Object::Literal(Literal::Typed(Value::String(s)))) => Some(s.clone()),
        _ => None,
    };

    let idempotent = matches!(
        graph.object(iri, HORIS_IDEMPOTENT, interner),
        Some(Object::Literal(Literal::Typed(Value::Boolean(true))))
    );

    let retries = match graph.object(iri, HORIS_RETRIES, interner) {
        Some(Object::Literal(Literal::Typed(Value::Integer(n)))) => u32::try_from(*n).unwrap_or(0),
        _ => 0,
    };

    Ok(Tool {
        iri,
        takes,
        effect,
        binding,
        timeout,
        idempotent,
        retries,
    })
}

fn parse_arg_type(
    tool: Iri,
    node: Iri,
    graph: &SubjectGraph,
    schema: &Schema,
    interner: &Interner,
) -> Result<ArgType, OntologyError> {
    match graph.object(node, HORIS_TYPE, interner) {
        Some(Object::Node(n)) => {
            let target = node_iri(*n);
            if let Some(datatype) = xsd_datatype_from_iri(interner.resolve(target)) {
                return Ok(ArgType::Datatype(datatype));
            }
            if schema.class_declared(target).is_none() {
                return Err(OntologyError::ToolUndeclaredReference {
                    tool: interner.resolve(tool).to_owned(),
                    name: interner.resolve(target).to_owned(),
                });
            }
            Ok(ArgType::Class(target))
        }
        _ => Err(OntologyError::ToolMissingPart {
            tool: interner.resolve(tool).to_owned(),
            part: "horis:type".to_owned(),
        }),
    }
}

fn parse_binding(raw: &str) -> Binding {
    match raw.strip_prefix("rust:") {
        Some(name) => Binding::Rust(name.to_owned()),
        None => {
            let scheme = raw
                .split_once(':')
                .map(|(s, _)| s)
                .unwrap_or(raw)
                .to_owned();
            Binding::Unsupported {
                scheme,
                raw: raw.to_owned(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ontology::facade::Ontology;

    use super::*;

    fn triage() -> Ontology {
        Ontology::load(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/triage/triage.ttl"
        ))
        .expect("triage.ttl must load, validate, and build a schema")
    }

    /// U3 test target: `:ehr.assign_queue` parses with two args and two
    /// declared effects.
    #[test]
    fn ehr_assign_queue_parses_with_two_args_and_two_effects() {
        let onto = triage();
        let iri = onto
            .iri("https://example.org/triage#ehr.assign_queue")
            .expect(":ehr.assign_queue is named by triage.ttl");
        let tool = onto.tool(iri).expect(":ehr.assign_queue must parse");

        assert_eq!(tool.takes.len(), 2);
        assert_eq!(tool.takes[0].name, "patient");
        assert_eq!(tool.takes[1].name, "queue");

        assert_eq!(tool.effect.len(), 2);
        assert!(matches!(tool.effect[0].value, EffectValue::Now));
        assert!(matches!(
            tool.effect[1].value,
            EffectValue::TypedPlaceholder(ArgType::Datatype(Datatype::String))
        ));

        assert_eq!(tool.binding, Binding::Rust("assign_queue".to_owned()));
        assert_eq!(tool.timeout.as_deref(), Some("5s"));
        assert!(!tool.idempotent);
    }

    #[test]
    fn a_non_rust_binding_parses_as_unsupported_with_its_scheme() {
        assert_eq!(
            parse_binding("python:ehr_client.assign_queue"),
            Binding::Unsupported {
                scheme: "python".to_owned(),
                raw: "python:ehr_client.assign_queue".to_owned(),
            }
        );
    }

    #[test]
    fn a_tool_missing_horis_effect_is_a_domain_term_error() {
        let err = Ontology::load_from_str(
            "@prefix : <https://example.org/t#> .\n\
             @prefix horis: <https://w3id.org/horismos#> .\n\
             @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
             @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
             @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
             :Patient a rdfs:Class .\n\
             :noop a horis:Tool ;\n\
                 horis:binding \"rust:noop\" .\n",
            |_| None,
        )
        .expect_err("a tool with no horis:effect must be rejected at load");
        assert!(err.domain_text().contains("horis:effect"), "{err:?}");
    }
}
