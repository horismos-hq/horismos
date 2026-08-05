//! The [`Ontology`] facade (U6 prerequisite): one loaded, profile-valid,
//! schema-built artifact — the aggregate `World::new(&onto)`
//! (`docs/framework/03-world.md`) and `Ontology::load(path)`
//! (`docs/framework/12-rust-api.md`) are written against.
//!
//! It wraps U5's [`OntologySet`] (root graph + imports), U4's [`Schema`], and
//! the [`Interner`] every `Iri` token in both is relative to, plus two
//! derived pieces the validation gate needs on every assertion: the compiled
//! `horis:pattern` regexes (compiled once here so a bad pattern is a *load*
//! error, never a per-assertion one) and the compaction table behind
//! [`Ontology::compact`] (domain-term errors name classes in prefixed form —
//! `:Patient`, not the full IRI).
//!
//! Immutable after construction and cheap to clone (`Arc` internals, doc 12
//! § Core types), so a `World` can hold one by value.
//!
//! Spec: `docs/framework/02-ontology.md`, `docs/framework/12-rust-api.md`
//! · Owner: PRP-001

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use regex::Regex;

use crate::error::OntologyError;
use crate::ontology::imports::OntologySet;
use crate::ontology::schema::Schema;
use crate::ontology::term::{Interner, Iri};

/// A loaded ontology: graphs, schema, interner. Construct with
/// [`Ontology::load`] (filesystem) or [`Ontology::load_from_str`] (pure);
/// both run RDFS-plus profile validation (U3) and build the schema (U4)
/// before returning, so an `Ontology` value is always profile-valid.
///
/// Cloning is an `Arc` bump. `Send + Sync` — share one across threads (doc
/// 12 § Threading & determinism).
#[derive(Debug, Clone)]
pub struct Ontology {
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    set: OntologySet,
    schema: Schema,
    interner: Interner,
    /// The unified prefix map (`OntologySet::prefixes`), re-sorted by
    /// *descending namespace length* — so [`Ontology::compact`] always picks
    /// the most specific binding — with the prefix name as a tie-break, which
    /// keeps the choice deterministic when two prefixes bind the same-length
    /// namespace (CLAUDE.md).
    compaction: Vec<(String, String)>,
    /// One compiled regex per property that declares `horis:pattern`. Built
    /// at load so the gate is a lookup, and an unparseable pattern surfaces
    /// as [`OntologyError::InvalidPattern`] before any world exists.
    patterns: BTreeMap<Iri, Regex>,
}

impl Ontology {
    /// The filesystem convenience (doc 12: `Ontology::load("triage.ttl")?`):
    /// reads `path` and every import it declares, transitively.
    pub fn load(path: impl AsRef<Path>) -> Result<Ontology, OntologyError> {
        let mut interner = Interner::new();
        let set = OntologySet::load(path, &mut interner)?;
        Ontology::assemble(set, interner)
    }

    /// The pure, wasm-safe constructor: parses `source` as the root graph and
    /// resolves every `horis:imports` target through `resolve` (an import IRI
    /// → its Turtle source; `None` means "not found"). No filesystem, no
    /// network. See [`OntologySet::load_from_str`] for the resolution rules.
    pub fn load_from_str(
        source: &str,
        resolve: impl FnMut(&str) -> Option<String>,
    ) -> Result<Ontology, OntologyError> {
        let mut interner = Interner::new();
        let set = OntologySet::load_from_str(source, &mut interner, resolve)?;
        Ontology::assemble(set, interner)
    }

    /// Adds one *undeclared* import from the filesystem (doc 12:
    /// `Ontology::load(...)?.with_import("snomed-ct-subset.ttl")?`), then
    /// re-validates and re-builds the schema over the widened graph set —
    /// the result is a new, independent `Ontology`; `self` is consumed.
    pub fn with_import(self, path: impl AsRef<Path>) -> Result<Ontology, OntologyError> {
        let inner = Arc::try_unwrap(self.inner).unwrap_or_else(|shared| Inner {
            set: shared.set.clone(),
            schema: shared.schema.clone(),
            interner: shared.interner.clone(),
            compaction: shared.compaction.clone(),
            patterns: shared.patterns.clone(),
        });
        let mut interner = inner.interner;
        let set = inner.set.with_import(path, &mut interner)?;
        Ontology::assemble(set, interner)
    }

    /// Validates, builds, and derives — the one construction path every
    /// constructor above funnels through.
    fn assemble(set: OntologySet, interner: Interner) -> Result<Ontology, OntologyError> {
        set.validate(&interner)?;
        let schema = Schema::build_all(set.documents(), &interner)?;
        let patterns = compile_patterns(&schema, &interner)?;

        let mut compaction = set.prefixes.clone();
        compaction.sort_by(|(a_name, a_ns), (b_name, b_ns)| {
            b_ns.len().cmp(&a_ns.len()).then_with(|| a_name.cmp(b_name))
        });

        Ok(Ontology {
            inner: Arc::new(Inner {
                set,
                schema,
                interner,
                compaction,
                patterns,
            }),
        })
    }

    /// The queryable schema (U4) — classes, the `subClassOf` closure,
    /// property domains/ranges/constraints. The validation gate's input.
    pub fn schema(&self) -> &Schema {
        &self.inner.schema
    }

    /// The interner every `Iri` in this ontology's schema is relative to.
    /// A `World` clones it at construction and continues interning instance
    /// IRIs into its own copy (see [`crate::world::World::new`]).
    pub fn interner(&self) -> &Interner {
        &self.inner.interner
    }

    /// The `Iri` token for `iri`, if this ontology's graphs name it. Lookup
    /// only — an ontology is immutable, so an IRI it never mentions has no
    /// token here; mint one against a world instead
    /// ([`crate::world::World::intern`]).
    pub fn iri(&self, iri: &str) -> Option<Iri> {
        self.inner.interner.get(iri)
    }

    /// The loaded graph set: root document plus imports (U5).
    pub fn graphs(&self) -> &OntologySet {
        &self.inner.set
    }

    /// The unified prefix map, sorted by prefix name (U5).
    pub fn prefixes(&self) -> &[(String, String)] {
        &self.inner.set.prefixes
    }

    /// The compiled `horis:pattern` for `property`, if it declares one.
    pub fn pattern(&self, property: Iri) -> Option<&Regex> {
        self.inner.patterns.get(&property)
    }

    /// Resolves an `Iri` token interned by *this* ontology back to its text.
    ///
    /// # Panics
    /// Panics if `iri` did not come from this ontology's interner — use
    /// [`crate::world::World::resolve`] for tokens minted by a world.
    pub fn resolve(&self, iri: Iri) -> &str {
        self.inner.interner.resolve(iri)
    }

    /// `iri` in Turtle prefixed form (`:Patient`, `sct:29857009`) using the
    /// most specific binding in the unified prefix map, for domain-term
    /// errors (`docs/framework/03-world.md`). Falls back to the full IRI in
    /// angle brackets (`<urn:x:y>` — Turtle's own form for an unprefixable
    /// IRI) when no binding matches, or when the local part would itself
    /// contain a `/` or `#` and so could not be written prefixed.
    pub fn compact(&self, iri: &str) -> String {
        match self.split_namespace(iri) {
            Some((name, local)) => format!("{name}:{local}"),
            None => format!("<{iri}>"),
        }
    }

    /// `iri`'s local name (`heartRate`) — how domain-term errors name a
    /// *property*, per the doc-03 template ("heartRate 1280 exceeds
    /// horis:max 300 (bpm) on :Patient": properties bare, classes prefixed).
    /// Falls back to the segment after the last `#` or `/`, then to the whole
    /// IRI.
    pub fn local_name(&self, iri: &str) -> String {
        if let Some((_, local)) = self.split_namespace(iri) {
            return local.to_owned();
        }
        match iri.rfind(['#', '/']) {
            Some(cut) if cut + 1 < iri.len() => iri[cut + 1..].to_owned(),
            _ => iri.to_owned(),
        }
    }

    /// Splits `iri` into `(prefix name, local part)` against the most
    /// specific bound namespace, if one applies and the local part is
    /// writable as a Turtle prefixed-name local part.
    fn split_namespace<'a>(&'a self, iri: &'a str) -> Option<(&'a str, &'a str)> {
        self.inner.compaction.iter().find_map(|(name, namespace)| {
            let local = iri.strip_prefix(namespace.as_str())?;
            (!local.is_empty() && !local.contains(['/', '#'])).then_some((name.as_str(), local))
        })
    }
}

/// Compiles every declared `horis:pattern` once. Patterns are **anchored**:
/// `horis:pattern` states the form the *whole* value must take
/// (`docs/framework/02-ontology.md`: "Regex the value must match"), so the
/// declared expression is wrapped in `^(?:…)$` rather than matched as a
/// substring search.
fn compile_patterns(
    schema: &Schema,
    interner: &Interner,
) -> Result<BTreeMap<Iri, Regex>, OntologyError> {
    let mut compiled = BTreeMap::new();
    for property in schema.properties() {
        let Some(pattern) = &property.pattern else {
            continue;
        };
        let regex =
            Regex::new(&format!("^(?:{pattern})$")).map_err(|e| OntologyError::InvalidPattern {
                property: interner.resolve(property.iri).to_owned(),
                pattern: pattern.clone(),
                reason: e.to_string(),
            })?;
        compiled.insert(property.iri, regex);
    }
    Ok(compiled)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ontology::schema::{AssignedBy, Cardinality};

    const PRELUDE: &str = "@prefix : <https://example.org/t#> .\n\
         @prefix horis: <https://w3id.org/horismos#> .\n\
         @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n";

    fn load(source: &str) -> Result<Ontology, OntologyError> {
        Ontology::load_from_str(&format!("{PRELUDE}{source}\n"), |_| None)
    }

    fn triage() -> Ontology {
        Ontology::load(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/triage/triage.ttl"
        ))
        .expect("triage.ttl must load, validate, and build a schema")
    }

    #[test]
    fn load_from_str_validates_the_profile_and_builds_the_schema() {
        let onto = load(
            ":Patient a rdfs:Class .\n\
             :heartRate a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:decimal ;\n\
             \x20   horis:cardinality \"0..1\" .",
        )
        .expect("a profile-valid document must load");

        let heart_rate = onto
            .iri("https://example.org/t#heartRate")
            .expect("heartRate is named by the loaded document");
        let decl = onto
            .schema()
            .property(heart_rate)
            .expect("the schema must be built and queryable");
        assert_eq!(decl.cardinality, Some(Cardinality::ZeroOrOne));
    }

    #[test]
    fn load_from_str_rejects_an_out_of_profile_document() {
        let err = Ontology::load_from_str(
            "@prefix owl: <http://www.w3.org/2002/07/owl#> .\n\
             @prefix : <https://example.org/t#> .\n\
             :Foo a owl:Restriction .\n",
            |_| None,
        )
        .expect_err("owl:Restriction is outside the RDFS-plus profile");
        assert!(err.domain_text().contains("owl:Restriction"), "{err:?}");
    }

    #[test]
    fn load_reads_triage_ttl_from_disk() {
        let onto = triage();
        let acuity = onto
            .iri("https://example.org/triage#acuity")
            .expect("acuity is named by triage.ttl");
        assert_eq!(
            onto.schema()
                .property(acuity)
                .expect("acuity must be a property")
                .assigned_by,
            AssignedBy::RulesOnly
        );
    }

    #[test]
    fn compact_uses_the_most_specific_prefix_binding() {
        let onto = triage();
        assert_eq!(
            onto.compact("https://example.org/triage#Patient"),
            ":Patient"
        );
        assert_eq!(
            onto.compact("http://snomed.info/id/29857009"),
            "sct:29857009"
        );
    }

    #[test]
    fn compact_falls_back_to_the_full_iri_in_angle_brackets() {
        let onto = triage();
        assert_eq!(onto.compact("urn:nowhere:x"), "<urn:nowhere:x>");
    }

    #[test]
    fn local_name_strips_the_namespace() {
        let onto = triage();
        assert_eq!(
            onto.local_name("https://example.org/triage#heartRate"),
            "heartRate"
        );
        assert_eq!(onto.local_name("urn:nowhere:x"), "urn:nowhere:x");
        assert_eq!(onto.local_name("http://example.org/vocab/thing"), "thing");
    }

    #[test]
    fn patterns_are_compiled_at_load_and_anchored() {
        let onto = load(
            ":Patient a rdfs:Class .\n\
             :mrn a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string ;\n\
             \x20   horis:pattern \"[A-Z]{2}-[0-9]{4}\" .",
        )
        .expect("a valid pattern must load");
        let mrn = onto
            .iri("https://example.org/t#mrn")
            .expect("mrn is declared");
        let regex = onto.pattern(mrn).expect("the pattern must be compiled");
        assert!(regex.is_match("AB-1234"));
        assert!(
            !regex.is_match("xxAB-1234yy"),
            "horis:pattern must match the whole value, not a substring"
        );
    }

    #[test]
    fn an_unparseable_pattern_is_a_load_error() {
        let err = load(
            ":Patient a rdfs:Class .\n\
             :mrn a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string ;\n\
             \x20   horis:pattern \"[unterminated\" .",
        )
        .expect_err("an unparseable horis:pattern must fail at load, not at the gate");
        match err {
            OntologyError::InvalidPattern {
                property, pattern, ..
            } => {
                assert!(property.contains("t#mrn"), "{property}");
                assert_eq!(pattern, "[unterminated");
            }
            other => panic!("expected InvalidPattern, got {other:?}"),
        }
    }

    #[test]
    fn with_import_widens_the_schema() {
        let fixture = |name: &str| {
            format!(
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/imports/{}"),
                name
            )
        };
        let onto = Ontology::load(fixture("root.ttl"))
            .expect("root must load")
            .with_import(fixture("extra.ttl"))
            .expect("the extra import must load");
        let extra = onto
            .iri("https://example.org/imports-fixture#ExtraClass")
            .expect("ExtraClass is named by the extra import");
        assert!(onto.schema().class(extra).is_some());
    }
}
