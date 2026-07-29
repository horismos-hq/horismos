//! Turtle parse/serialize over the U1 term model (decision D1, PRP-001 U2).
//!
//! Consumes `oxttl`/`oxrdf` for grammar-level parsing and serialization;
//! horismos keeps its own interned triple representation (`Document`,
//! `Triple`) rather than depending on the `oxigraph` store.
//!
//! Spec: `docs/framework/02-ontology.md` · Owner: PRP-001

use oxrdf::{BlankNode, NamedNode, NamedOrBlankNode, Term as OxTerm, Triple as OxTriple};
use oxttl::{TurtleParser, TurtleSerializer, TurtleSyntaxError};

use crate::error::OntologyError;
use crate::ontology::term::{Datatype, Interner, Iri, Value};

/// A node: a named IRI or a blank node, both carried as interned [`Iri`]
/// tokens. Blank node ids are interned with a `"_:"`-prefixed string (e.g.
/// `"_:b0"`) — a scheme no absolute IRI can ever collide with — so the same
/// [`Interner`] can hold named and blank identity side by side.
///
/// A blank node's [`Iri`] is only as stable as the parse that produced it:
/// `oxttl` assigns a fresh random id to every anonymous (`[...]`/`(...)`)
/// node it encounters, so parsing the same source twice yields different
/// blank tokens. Giving blank nodes durable, content-addressed identity is
/// skolemization — U6's job, not this module's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Node {
    Named(Iri),
    Blank(Iri),
}

/// A literal object value.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// A literal whose datatype is in the RDFS-plus XSD subset
    /// (`docs/framework/02-ontology.md`) *and* whose lexical form parses
    /// cleanly under U1's [`Value::parse_typed`].
    Typed(Value),
    /// Any other typed literal: datatype outside the XSD subset, or a
    /// lexical form U1's stricter grammar rejects (e.g. an `xsd:dateTime`
    /// without a timezone offset). Kept verbatim — whether it's acceptable
    /// is profile validation's call (U3), not parsing's.
    Other { lexical: String, datatype: Iri },
    /// A language-tagged string (`rdf:langString`). The profile has no
    /// vocabulary for this; kept faithfully so nothing is silently dropped.
    LangString { lexical: String, language: String },
}

/// The object position of a [`Triple`]: a node or a literal.
#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Node(Node),
    Literal(Literal),
}

/// One RDF triple over interned terms.
#[derive(Debug, Clone, PartialEq)]
pub struct Triple {
    pub subject: Node,
    pub predicate: Iri,
    pub object: Object,
}

/// A parsed Turtle document: its prefix map, base IRI, and triples.
///
/// `prefixes` is sorted alphabetically by prefix name rather than kept in
/// source order: `oxttl` only exposes the final prefix set (through a
/// `HashMap`, whose iteration order is randomized per process — CLAUDE.md's
/// no-iteration-order-dependence rule), so a stable canonical order is
/// imposed here rather than trusting source position. `triples` — the part
/// that actually varies per document — preserves source order exactly.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Document {
    pub prefixes: Vec<(String, String)>,
    pub base: Option<String>,
    pub triples: Vec<Triple>,
}

/// Parses `source` as Turtle, interning every IRI (named or blank) through
/// `interner`. Literal objects become a U1 [`Value`] when their datatype is
/// in the XSD subset and their lexical form parses cleanly; anything else is
/// kept as a lexical/datatype (or lexical/language) pair — rejecting it is
/// profile validation's job (U3), not this function's.
///
/// `[ ... ]` blank-node property lists and `( ... )` RDF collections are
/// Turtle grammar sugar: `oxttl` already desugars both into plain triples
/// over synthesized blank nodes before we ever see them, so no special
/// handling is needed here.
pub fn parse(source: &str, interner: &mut Interner) -> Result<Document, OntologyError> {
    let mut parser = TurtleParser::new().for_slice(source);
    let mut triples = Vec::new();
    for result in parser.by_ref() {
        let triple = result.map_err(|e| convert_syntax_error(&e))?;
        triples.push(convert_triple(&triple, interner));
    }

    let mut prefixes: Vec<(String, String)> = parser
        .prefixes()
        .map(|(name, iri)| (name.to_owned(), iri.to_owned()))
        .collect();
    prefixes.sort_by(|a, b| a.0.cmp(&b.0));
    let base = parser.base_iri().map(str::to_owned);

    Ok(Document {
        prefixes,
        base,
        triples,
    })
}

/// Serializes `document` to Turtle text using its own prefix map and base
/// IRI. Triples are written in `document.triples` order — the sole source of
/// ordering; nothing here iterates a hash map — so the output is a pure
/// function of `(document, interner)` (CLAUDE.md determinism rule).
pub fn serialize(document: &Document, interner: &Interner) -> String {
    let mut serializer = TurtleSerializer::new();
    for (name, iri) in &document.prefixes {
        serializer = serializer
            .with_prefix(name.clone(), iri.clone())
            .expect("Document.prefixes only ever holds IRIs oxttl itself already validated");
    }
    if let Some(base) = &document.base {
        serializer = serializer
            .with_base_iri(base.clone())
            .expect("Document.base only ever holds an IRI oxttl itself already validated");
    }

    let mut writer = serializer.for_writer(Vec::new());
    for triple in &document.triples {
        let ox_triple = to_oxrdf_triple(triple, interner);
        writer
            .serialize_triple(ox_triple.as_ref())
            .expect("writing Turtle to an in-memory Vec<u8> cannot fail");
    }
    let bytes = writer
        .finish()
        .expect("writing Turtle to an in-memory Vec<u8> cannot fail");
    String::from_utf8(bytes).expect("oxttl's Turtle serializer always emits valid UTF-8")
}

// -- oxrdf -> our types ------------------------------------------------

fn convert_syntax_error(err: &TurtleSyntaxError) -> OntologyError {
    let start = err.location().start;
    OntologyError::turtle_syntax(start.line + 1, start.column + 1, err.message().to_owned())
}

fn convert_triple(triple: &OxTriple, interner: &mut Interner) -> Triple {
    Triple {
        subject: convert_named_or_blank(&triple.subject, interner),
        predicate: interner.intern(triple.predicate.as_str()),
        object: convert_object(&triple.object, interner),
    }
}

fn convert_named_or_blank(node: &NamedOrBlankNode, interner: &mut Interner) -> Node {
    match node {
        NamedOrBlankNode::NamedNode(n) => Node::Named(interner.intern(n.as_str())),
        NamedOrBlankNode::BlankNode(b) => Node::Blank(intern_blank(interner, b.as_str())),
    }
}

fn intern_blank(interner: &mut Interner, id: &str) -> Iri {
    interner.intern(&format!("_:{id}"))
}

fn convert_object(term: &OxTerm, interner: &mut Interner) -> Object {
    match term {
        OxTerm::NamedNode(n) => Object::Node(Node::Named(interner.intern(n.as_str()))),
        OxTerm::BlankNode(b) => Object::Node(Node::Blank(intern_blank(interner, b.as_str()))),
        OxTerm::Literal(lit) => Object::Literal(convert_literal(lit, interner)),
    }
}

fn convert_literal(lit: &oxrdf::Literal, interner: &mut Interner) -> Literal {
    if let Some(language) = lit.language() {
        return Literal::LangString {
            lexical: lit.value().to_owned(),
            language: language.to_owned(),
        };
    }
    let datatype_iri = lit.datatype().as_str();
    if let Some(datatype) = xsd_datatype_from_iri(datatype_iri)
        && let Ok(value) = Value::parse_typed(lit.value(), datatype)
    {
        return Literal::Typed(value);
    }
    Literal::Other {
        lexical: lit.value().to_owned(),
        datatype: interner.intern(datatype_iri),
    }
}

/// The XSD namespace's `docs/framework/02-ontology.md` subset, both
/// directions: full datatype IRI text <-> U1's [`Datatype`]. The single
/// source of truth both `parse` and `serialize` read from, so the two can
/// never drift apart.
pub(crate) fn xsd_datatype_from_iri(iri: &str) -> Option<Datatype> {
    Some(match iri {
        "http://www.w3.org/2001/XMLSchema#decimal" => Datatype::Decimal,
        "http://www.w3.org/2001/XMLSchema#integer" => Datatype::Integer,
        "http://www.w3.org/2001/XMLSchema#string" => Datatype::String,
        "http://www.w3.org/2001/XMLSchema#boolean" => Datatype::Boolean,
        "http://www.w3.org/2001/XMLSchema#dateTime" => Datatype::DateTime,
        "http://www.w3.org/2001/XMLSchema#duration" => Datatype::Duration,
        _ => return None,
    })
}

fn xsd_datatype_iri(datatype: Datatype) -> &'static str {
    match datatype {
        Datatype::Decimal => "http://www.w3.org/2001/XMLSchema#decimal",
        Datatype::Integer => "http://www.w3.org/2001/XMLSchema#integer",
        Datatype::String => "http://www.w3.org/2001/XMLSchema#string",
        Datatype::Boolean => "http://www.w3.org/2001/XMLSchema#boolean",
        Datatype::DateTime => "http://www.w3.org/2001/XMLSchema#dateTime",
        Datatype::Duration => "http://www.w3.org/2001/XMLSchema#duration",
    }
}

// -- our types -> oxrdf --------------------------------------------------

fn to_oxrdf_triple(triple: &Triple, interner: &Interner) -> OxTriple {
    OxTriple::new(
        to_oxrdf_named_or_blank(triple.subject, interner),
        NamedNode::new_unchecked(interner.resolve(triple.predicate)),
        to_oxrdf_term(&triple.object, interner),
    )
}

fn to_oxrdf_named_or_blank(node: Node, interner: &Interner) -> NamedOrBlankNode {
    match node {
        Node::Named(iri) => {
            NamedOrBlankNode::NamedNode(NamedNode::new_unchecked(interner.resolve(iri)))
        }
        Node::Blank(iri) => {
            let label = interner.resolve(iri);
            let id = label
                .strip_prefix("_:")
                .expect("Node::Blank Iri tokens are always interned with the \"_:\" scheme");
            NamedOrBlankNode::BlankNode(BlankNode::new_unchecked(id))
        }
    }
}

fn to_oxrdf_term(object: &Object, interner: &Interner) -> OxTerm {
    match object {
        Object::Node(node) => match to_oxrdf_named_or_blank(*node, interner) {
            NamedOrBlankNode::NamedNode(n) => OxTerm::NamedNode(n),
            NamedOrBlankNode::BlankNode(b) => OxTerm::BlankNode(b),
        },
        Object::Literal(lit) => OxTerm::Literal(to_oxrdf_literal(lit, interner)),
    }
}

fn to_oxrdf_literal(lit: &Literal, interner: &Interner) -> oxrdf::Literal {
    match lit {
        Literal::Typed(value) => {
            let lexical = value
                .lexical_form()
                .expect("Literal::Typed never wraps Value::Ref — see value_datatype");
            oxrdf::Literal::new_typed_literal(
                lexical,
                NamedNode::new_unchecked(xsd_datatype_iri(value_datatype(value))),
            )
        }
        Literal::Other { lexical, datatype } => oxrdf::Literal::new_typed_literal(
            lexical.clone(),
            NamedNode::new_unchecked(interner.resolve(*datatype)),
        ),
        Literal::LangString { lexical, language } => {
            oxrdf::Literal::new_language_tagged_literal_unchecked(lexical.clone(), language.clone())
        }
    }
}

/// `Literal::Typed` is only ever constructed by [`convert_literal`], which
/// only reaches a [`Value`] through [`Value::parse_typed`] — never producing
/// [`Value::Ref`] (that variant exists for node-valued *properties*, not
/// literal terms). This mirrors [`xsd_datatype_from_iri`] as its inverse.
fn value_datatype(value: &Value) -> Datatype {
    match value {
        Value::Decimal(_) => Datatype::Decimal,
        Value::Integer(_) => Datatype::Integer,
        Value::String(_) => Datatype::String,
        Value::Boolean(_) => Datatype::Boolean,
        Value::DateTime(_) => Datatype::DateTime,
        Value::Duration(_) => Datatype::Duration,
        Value::Ref(_) => unreachable!("a literal term can never carry a Value::Ref"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triple_body(interner: &mut Interner, snippet: &str) -> Document {
        let source = format!("@prefix : <https://example.org/t#> .\n{snippet}\n");
        parse(&source, interner).expect("valid Turtle snippet must parse")
    }

    #[test]
    fn parses_a_single_named_triple() {
        let mut interner = Interner::new();
        let doc = triple_body(&mut interner, ":s :p :o .");
        assert_eq!(doc.triples.len(), 1);
        let t = &doc.triples[0];
        let Node::Named(s) = t.subject else {
            panic!("expected named subject")
        };
        assert_eq!(interner.resolve(s), "https://example.org/t#s");
        assert_eq!(interner.resolve(t.predicate), "https://example.org/t#p");
        let Object::Node(Node::Named(o)) = &t.object else {
            panic!("expected named object")
        };
        assert_eq!(interner.resolve(*o), "https://example.org/t#o");
    }

    #[test]
    fn maps_xsd_subset_literals_to_value() {
        let mut interner = Interner::new();
        let cases: &[(&str, Value)] = &[
            (
                ":s :p 12.5 .",
                Value::Decimal(rust_decimal::Decimal::new(125, 1)),
            ),
            (":s :p 42 .", Value::Integer(42)),
            (":s :p \"hello\" .", Value::String("hello".to_owned())),
            (":s :p true .", Value::Boolean(true)),
        ];
        for (snippet, expected) in cases {
            let doc = triple_body(&mut interner, snippet);
            let Object::Literal(Literal::Typed(value)) = &doc.triples[0].object else {
                panic!("expected a typed literal for {snippet}")
            };
            assert_eq!(value, expected, "mismatch for {snippet}");
        }
    }

    #[test]
    fn keeps_literal_outside_xsd_subset_as_lexical_datatype_pair() {
        let mut interner = Interner::new();
        let doc = triple_body(
            &mut interner,
            ":s :p \"127.0.0.1\"^^<http://www.w3.org/2001/XMLSchema#anyURI> .",
        );
        let Object::Literal(Literal::Other { lexical, datatype }) = &doc.triples[0].object else {
            panic!("expected an out-of-subset literal")
        };
        assert_eq!(lexical, "127.0.0.1");
        assert_eq!(
            interner.resolve(*datatype),
            "http://www.w3.org/2001/XMLSchema#anyURI"
        );
    }

    #[test]
    fn keeps_language_tagged_literal_faithfully() {
        let mut interner = Interner::new();
        let doc = triple_body(&mut interner, ":s :p \"bonjour\"@fr .");
        let Object::Literal(Literal::LangString { lexical, language }) = &doc.triples[0].object
        else {
            panic!("expected a language-tagged literal")
        };
        assert_eq!(lexical, "bonjour");
        assert_eq!(language, "fr");
    }

    #[test]
    fn prefix_map_is_sorted_alphabetically_for_determinism() {
        let mut interner = Interner::new();
        let source = "@prefix z: <urn:z#> .\n@prefix a: <urn:a#> .\nz:s a:p z:o .\n";
        let doc = parse(source, &mut interner).expect("valid Turtle");
        assert_eq!(
            doc.prefixes,
            vec![
                ("a".to_owned(), "urn:a#".to_owned()),
                ("z".to_owned(), "urn:z#".to_owned()),
            ]
        );
    }

    #[test]
    fn base_iri_is_captured_when_present() {
        let mut interner = Interner::new();
        let source = "@base <https://example.org/> .\n<a> <b> <c> .\n";
        let doc = parse(source, &mut interner).expect("valid Turtle");
        assert_eq!(doc.base.as_deref(), Some("https://example.org/"));
    }

    #[test]
    fn base_iri_is_none_when_absent() {
        let mut interner = Interner::new();
        let doc = triple_body(&mut interner, ":s :p :o .");
        assert_eq!(doc.base, None);
    }

    #[test]
    fn blank_node_property_list_shares_one_subject_across_triples() {
        let mut interner = Interner::new();
        let doc = triple_body(&mut interner, ":s :p [ :q :r ; :t :u ] .");
        // The parser resolves the nested `[ ... ]` object (emitting its
        // triples) before it can emit the outer `:s :p _:b .` triple:
        // _:b :q :r .   _:b :t :u .   :s :p _:b .
        assert_eq!(doc.triples.len(), 3);
        let Node::Blank(b1) = doc.triples[0].subject else {
            panic!("expected a blank subject on the first triple")
        };
        let Node::Blank(b2) = doc.triples[1].subject else {
            panic!("expected a blank subject on the second triple")
        };
        let Object::Node(Node::Blank(b)) = doc.triples[2].object else {
            panic!("expected a blank object on the third triple")
        };
        assert_eq!(b, b1);
        assert_eq!(b, b2);
    }

    #[test]
    fn rdf_collection_desugars_to_first_rest_nil_triples() {
        let mut interner = Interner::new();
        let doc = triple_body(&mut interner, ":s :p ( :a :b ) .");
        let first = interner.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#first");
        let rest = interner.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#rest");
        let nil = interner.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#nil");
        let first_count = doc.triples.iter().filter(|t| t.predicate == first).count();
        let rest_count = doc.triples.iter().filter(|t| t.predicate == rest).count();
        assert_eq!(first_count, 2, "one rdf:first per list cell");
        assert_eq!(rest_count, 2, "one rdf:rest per list cell");
        let terminates_in_nil = doc.triples.iter().any(|t| {
            t.predicate == rest && matches!(t.object, Object::Node(Node::Named(n)) if n == nil)
        });
        assert!(terminates_in_nil, "the list must terminate in rdf:nil");
    }

    #[test]
    fn syntax_errors_carry_line_and_column() {
        let mut interner = Interner::new();
        let source = "@prefix : <urn:x#> .\n:s :p\n";
        let err = parse(source, &mut interner).expect_err("missing object must be a syntax error");
        match err {
            OntologyError::TurtleSyntax { line, .. } => {
                assert_eq!(
                    line, 3,
                    "the error should be reported on the line the parser gave up"
                );
            }
            other => panic!("expected OntologyError::TurtleSyntax, got {other:?}"),
        }
    }

    #[test]
    fn serialize_is_a_pure_function_of_the_document() {
        let mut interner = Interner::new();
        let doc = triple_body(&mut interner, ":s :p :o .");
        let a = serialize(&doc, &interner);
        let b = serialize(&doc, &interner);
        assert_eq!(a, b);
    }

    #[test]
    fn round_trips_a_hand_built_document_through_one_interner() {
        let mut interner = Interner::new();
        let doc = triple_body(&mut interner, ":s :p 12.5, 42, \"hi\", true .");
        let text = serialize(&doc, &interner);
        let reparsed = parse(&text, &mut interner).expect("our own output must parse");
        assert_eq!(reparsed, doc);
    }

    /// Canonicalizes a document's triples for isomorphism-safe comparison:
    /// blank nodes are replaced by an index keyed on first appearance in
    /// document order (subject before object, triples in source order).
    /// oxttl assigns blank-node identity via `rand::random()` for every
    /// anonymous (`[...]`/`(...)`) node it parses, so two independent parses
    /// of syntactically-equivalent Turtle are never expected to agree on raw
    /// blank labels — only on graph shape.
    fn canonicalize(doc: &Document, interner: &Interner) -> Vec<String> {
        use std::collections::HashMap;

        let mut blanks: HashMap<Iri, usize> = HashMap::new();
        let mut next = 0usize;
        let mut label = |node: Node| -> String {
            match node {
                Node::Named(iri) => format!("iri:{}", interner.resolve(iri)),
                Node::Blank(iri) => {
                    let idx = *blanks.entry(iri).or_insert_with(|| {
                        let i = next;
                        next += 1;
                        i
                    });
                    format!("blank:{idx}")
                }
            }
        };

        let mut out: Vec<String> = doc
            .triples
            .iter()
            .map(|t| {
                let subject = label(t.subject);
                let predicate = format!("iri:{}", interner.resolve(t.predicate));
                let object = match &t.object {
                    Object::Node(n) => label(*n),
                    Object::Literal(Literal::Typed(v)) => format!("lit:{v:?}"),
                    Object::Literal(Literal::Other { lexical, datatype }) => {
                        format!("lit:{lexical:?}^^{}", interner.resolve(*datatype))
                    }
                    Object::Literal(Literal::LangString { lexical, language }) => {
                        format!("lit:{lexical:?}@{language}")
                    }
                };
                format!("{subject}\u{1}{predicate}\u{1}{object}")
            })
            .collect();
        out.sort();
        out
    }

    #[test]
    fn round_trips_triage_fixture_up_to_blank_node_relabeling() {
        let source = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/triage/triage.ttl"
        ))
        .expect("triage.ttl fixture must exist");

        let mut interner_a = Interner::new();
        let doc_a = parse(&source, &mut interner_a).expect("triage.ttl must parse");

        let text = serialize(&doc_a, &interner_a);

        let mut interner_b = Interner::new();
        let doc_b = parse(&text, &mut interner_b).expect("our own serialized output must parse");

        assert_eq!(doc_a.prefixes, doc_b.prefixes);
        assert_eq!(doc_a.base, doc_b.base);
        assert_eq!(doc_a.triples.len(), doc_b.triples.len());
        assert_eq!(
            canonicalize(&doc_a, &interner_a),
            canonicalize(&doc_b, &interner_b)
        );
    }

    // -- proptest: round trip over generated simple documents -----------

    use proptest::prelude::*;

    fn arb_iri_string() -> impl Strategy<Value = String> {
        // No `#`: a second fragment marker (e.g. two generated IRIs sharing
        // a path could still each be legal, but a single generated string
        // like ".../a#b#c" is not a valid IRI — RFC 3986 allows at most one
        // fragment — and oxiri enforces that on reparse.
        "https://[a-z]{1,10}\\.example/[a-zA-Z0-9_-]{1,20}"
    }

    fn arb_decimal() -> impl Strategy<Value = rust_decimal::Decimal> {
        (-1_000_000i64..=1_000_000i64, 0u32..=6u32)
            .prop_map(|(num, scale)| rust_decimal::Decimal::new(num, scale))
    }

    fn arb_value() -> impl Strategy<Value = Value> {
        prop_oneof![
            arb_decimal().prop_map(Value::Decimal),
            any::<i64>().prop_map(Value::Integer),
            "[a-zA-Z0-9 ]{0,20}".prop_map(Value::String),
            any::<bool>().prop_map(Value::Boolean),
        ]
    }

    #[derive(Debug)]
    enum Either<A, B> {
        Iri(A),
        Literal(B),
    }

    proptest! {
        #[test]
        fn round_trips_simple_generated_documents(
            triples in prop::collection::vec(
                (arb_iri_string(), arb_iri_string(), prop_oneof![
                    arb_iri_string().prop_map(Either::Iri),
                    arb_value().prop_map(Either::Literal),
                ]),
                0..8,
            )
        ) {
            let mut interner = Interner::new();
            let mut document = Document::default();
            for (s, p, o) in triples {
                let subject = Node::Named(interner.intern(&s));
                let predicate = interner.intern(&p);
                let object = match o {
                    Either::Iri(iri) => Object::Node(Node::Named(interner.intern(&iri))),
                    Either::Literal(v) => Object::Literal(Literal::Typed(v)),
                };
                document.triples.push(Triple { subject, predicate, object });
            }

            let text = serialize(&document, &interner);
            let reparsed = parse(&text, &mut interner).expect("generated documents must round-trip");
            prop_assert_eq!(reparsed, document);
        }
    }
}
