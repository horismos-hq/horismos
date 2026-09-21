//! Imports into named graphs (U5): a loaded ontology is the root [`Document`]
//! plus each declared/explicit import in its own named graph.
//! `docs/framework/02-ontology.md` §Imports and anchoring: a subset `.ttl` is
//! an ordinary import — no lazy loading, no I/O beyond reading declared
//! imports (decision D5). This module is the one fs touch-point: the pure,
//! wasm-safe core ([`OntologySet::load_from_str`]) never calls into
//! `std::fs`; only [`OntologySet::load`]/[`OntologySet::with_import`] do,
//! each behind a single small function
//! ([`read_to_string`]/[`file_uri`]/[`path_from_file_uri`]) so a future
//! `wasm` build (`horismos-core::wasm`) can swap the fs touch-point out
//! without touching the resolution logic.
//!
//! `owl:sameAs` anchoring itself — the equivalence relation that lets
//! subsumption cross a graph boundary — lives in
//! [`crate::ontology::schema`] (`Schema::build_all`); this module only
//! resolves *which* graphs exist and hands them, plus a merged prefix map,
//! to that and to [`crate::ontology::profile::validate_named`].
//!
//! Spec: `docs/framework/02-ontology.md`, `docs/framework/12-rust-api.md`
//! (`Ontology::load`/`with_import`) · Owner: PRP-001

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::error::OntologyError;
use crate::ontology::term::Interner;
use crate::ontology::turtle::{self, Document, Node, Object};

/// `horis:imports` — the only predicate this module reads directly.
const HORIS_IMPORTS: &str = "https://w3id.org/horismos#imports";

/// The name [`OntologySet::validate`] attributes a root-document profile
/// violation to. Not a real IRI — the root graph's *actual* identity (its
/// `<>` base, when it has one) isn't needed for naming purposes; a plain,
/// stable label reads better in an error than a `file://…` path.
pub const ROOT_GRAPH_NAME: &str = "root";

/// A loaded ontology's full graph set: the root document plus every import,
/// resolved transitively, each in its own named graph. One [`Interner`] is
/// threaded through every graph that contributes to an `OntologySet`, so
/// `Iri` tokens mean the same thing across all of them — callers must reuse
/// the same `interner` for every method here.
#[derive(Debug, Clone)]
pub struct OntologySet {
    /// The root document.
    pub root: Document,
    /// Imports in resolution order: each graph's own declared imports are
    /// resolved — and appear here — *before* the graph that declared them
    /// (dependency order, not declaration order — see
    /// [`resolve_transitive_imports`] for why). Named by the import's own
    /// resolved IRI ("as declared", per doc 02 — for a relative Turtle IRI
    /// reference that means the absolute IRI it resolves to, exactly as RDF
    /// itself has no notion of an "unresolved" IRI). A diamond import (two
    /// graphs both importing the same third graph) appears once.
    pub imports: Vec<(String, Document)>,
    /// The unified prefix map (brief requirement 3): root's own bindings,
    /// plus every import's, with root winning any conflict. Sorted by
    /// prefix name (deterministic — CLAUDE.md). Root document serialization
    /// (`turtle::serialize(&set.root, &interner)`) is untouched by this: it
    /// keeps using `root.prefixes` exactly as U2 always did.
    pub prefixes: Vec<(String, String)>,
}

impl OntologySet {
    /// The pure, wasm-safe core: parses `root_source` as the root document,
    /// then resolves every `horis:imports` triple it declares — transitively
    /// — via `resolve`, a map or closure from an import's resolved IRI to
    /// its Turtle source text (`None` means "not found"). No filesystem or
    /// network access.
    ///
    /// A relative import IRI (`<./x.ttl>`) only parses if `root_source`
    /// declares its own `@base` — plain Turtle has no default base (RFC
    /// 3986); this function does not inject one, unlike [`OntologySet::load`]
    /// (whose base is the loading file's own location). An absolute import
    /// IRI (`<urn:...>`, `<https://...>`) needs no base at all.
    pub fn load_from_str(
        root_source: &str,
        interner: &mut Interner,
        mut resolve: impl FnMut(&str) -> Option<String>,
    ) -> Result<OntologySet, OntologyError> {
        let root = turtle::parse(root_source, interner)?;

        let mut fetch = |target: &str, interner: &mut Interner| match resolve(target) {
            Some(source) => turtle::parse(&source, interner).map(Some),
            None => Ok(None),
        };

        let imports = resolve_transitive_imports(&root, interner, &mut fetch)?;
        let prefixes = merge_prefixes(&root, &imports)?;
        Ok(OntologySet {
            root,
            imports,
            prefixes,
        })
    }

    /// The fs convenience (doc 12: `Ontology::load("triage.ttl")?`): reads
    /// `path`, then resolves every import it declares — transitively —
    /// relative to *each file's own* location (so a nested import's own
    /// relative imports resolve against its directory, not the root's).
    /// This is the one place the pure core ([`OntologySet::load_from_str`])
    /// doesn't reach: the fs touch-point is kept to the small
    /// [`read_to_string`]/[`file_uri`]/[`path_from_file_uri`] helpers below.
    pub fn load(
        path: impl AsRef<Path>,
        interner: &mut Interner,
    ) -> Result<OntologySet, OntologyError> {
        let path = path.as_ref();
        let root_name = file_uri(path)?;
        let source = read_to_string(path)?;
        let root = turtle::parse_with_base(&source, &root_name, interner)?;

        let mut fetch = fs_fetch();
        let imports = resolve_transitive_imports(&root, interner, &mut fetch)?;
        let prefixes = merge_prefixes(&root, &imports)?;
        Ok(OntologySet {
            root,
            imports,
            prefixes,
        })
    }

    /// Loads and adds one additional, *undeclared* import (doc 12:
    /// `Ontology::load(...)?.with_import(...)?`) — resolved from the
    /// filesystem exactly like a declared import, except no `horis:imports`
    /// triple names it. Its own declared imports (if any) are resolved
    /// transitively too, exactly as [`OntologySet::load`] would. Naming a
    /// graph that's already loaded — the root itself, or an import already
    /// in `self.imports` — is a caller error
    /// ([`OntologyError::DuplicateGraphName`]); a *diamond* import reached
    /// transitively from two different declared imports is not an error
    /// (deduped, "each file loaded once" — brief requirement 1).
    pub fn with_import(
        mut self,
        path: impl AsRef<Path>,
        interner: &mut Interner,
    ) -> Result<OntologySet, OntologyError> {
        let path = path.as_ref();
        let name = file_uri(path)?;

        let root_name = self.root.base.clone().unwrap_or_default();
        if name == root_name || self.imports.iter().any(|(n, _)| *n == name) {
            return Err(OntologyError::DuplicateGraphName { name });
        }

        let source = read_to_string(path)?;
        let doc = turtle::parse_with_base(&source, &name, interner)?;

        let mut loaded: BTreeSet<String> = self.imports.iter().map(|(n, _)| n.clone()).collect();
        loaded.insert(root_name);
        loaded.insert(name.clone());
        let mut in_progress = Vec::new();
        let mut nested = Vec::new();
        let mut fetch = fs_fetch();
        visit_imports(
            &doc,
            interner,
            &mut fetch,
            &mut loaded,
            &mut in_progress,
            &mut nested,
        )?;

        self.imports.push((name, doc));
        self.imports.extend(nested);
        self.prefixes = merge_prefixes(&self.root, &self.imports)?;
        Ok(self)
    }

    /// `root`, then each import in resolution order — the graph set
    /// [`crate::ontology::schema::Schema::build_all`] should consume:
    /// `Schema::build_all(set.documents(), &interner)`.
    pub fn documents(&self) -> impl Iterator<Item = &Document> {
        std::iter::once(&self.root).chain(self.imports.iter().map(|(_, d)| d))
    }

    /// Every graph paired with the name a profile violation inside it should
    /// be attributed to: [`ROOT_GRAPH_NAME`] for the root, then each
    /// import's own resolved graph name.
    pub fn named_documents(&self) -> impl Iterator<Item = (&str, &Document)> {
        std::iter::once((ROOT_GRAPH_NAME, &self.root))
            .chain(self.imports.iter().map(|(name, d)| (name.as_str(), d)))
    }

    /// Runs U3 profile validation over every graph in the set (brief
    /// requirement 2): violations are collected across all of them into one
    /// [`OntologyError::ProfileViolations`], each naming the graph it came
    /// from.
    pub fn validate(&self, interner: &Interner) -> Result<(), OntologyError> {
        crate::ontology::profile::validate_named(self.named_documents(), interner)
    }
}

/// A `fetch` closure for the filesystem loaders ([`OntologySet::load`] /
/// [`OntologySet::with_import`]): resolves an already-absolute `file://`
/// import IRI back to a path, reads it if present, and parses it with
/// *that file's own* location as base (so its own relative imports resolve
/// against its directory, not the root's). `Ok(None)` — "not found" — for
/// anything that isn't a local, existing `file://` path: remote imports
/// (`http://…`) aren't supported by the v1 fs convenience (decision D5: no
/// I/O beyond reading declared *local* imports).
fn fs_fetch() -> impl FnMut(&str, &mut Interner) -> Result<Option<Document>, OntologyError> {
    |target: &str, interner: &mut Interner| {
        let Some(path) = path_from_file_uri(target) else {
            return Ok(None);
        };
        if !path.is_file() {
            return Ok(None);
        }
        let source = read_to_string(&path)?;
        turtle::parse_with_base(&source, target, interner).map(Some)
    }
}

/// Depth-first resolves every `horis:imports` triple reachable from `root`,
/// starting with an empty "already loaded" set — see [`visit_imports`] for
/// the traversal/order contract.
fn resolve_transitive_imports<F>(
    root: &Document,
    interner: &mut Interner,
    fetch: &mut F,
) -> Result<Vec<(String, Document)>, OntologyError>
where
    F: FnMut(&str, &mut Interner) -> Result<Option<Document>, OntologyError>,
{
    let mut loaded = BTreeSet::new();
    let mut in_progress = Vec::new();
    let mut imports = Vec::new();
    visit_imports(
        root,
        interner,
        fetch,
        &mut loaded,
        &mut in_progress,
        &mut imports,
    )?;
    Ok(imports)
}

/// The shared DFS: for each `horis:imports` target `doc` declares, in
/// declaration order —
///
/// - already `loaded` (reached earlier, via any path): skipped (dedupe —
///   "each file loaded once", brief requirement 1; diamond imports are
///   normal, not an error).
/// - already `in_progress` (on the current DFS path): [`OntologyError::ImportCycle`].
/// - otherwise: `fetch`ed and parsed, then *its own* declared imports are
///   resolved first (recursively, depth-first) — only once that finishes
///   does the target itself get appended to `imports`.
///
/// That last point is a deliberate, documented choice: `imports` ends up in
/// **dependency order** (a graph's own imports precede it), not raw
/// declaration order — a graph never appears before everything it depends
/// on has. Nothing downstream (`Schema::build_all`, profile validation)
/// actually depends on the order, but a stable, meaningful order matters for
/// determinism (CLAUDE.md) and for readable output.
fn visit_imports<F>(
    doc: &Document,
    interner: &mut Interner,
    fetch: &mut F,
    loaded: &mut BTreeSet<String>,
    in_progress: &mut Vec<String>,
    imports: &mut Vec<(String, Document)>,
) -> Result<(), OntologyError>
where
    F: FnMut(&str, &mut Interner) -> Result<Option<Document>, OntologyError>,
{
    for target in declared_import_targets(doc, interner) {
        if loaded.contains(&target) {
            continue;
        }
        if in_progress.contains(&target) {
            let mut cycle = in_progress.clone();
            cycle.push(target);
            return Err(OntologyError::ImportCycle {
                cycle: cycle.join(" -> "),
            });
        }

        let imported = fetch(&target, interner)?.ok_or_else(|| OntologyError::ImportNotFound {
            import: target.clone(),
            reason: "no source available for this import".to_owned(),
        })?;

        in_progress.push(target.clone());
        visit_imports(&imported, interner, fetch, loaded, in_progress, imports)?;
        in_progress.pop();

        loaded.insert(target.clone());
        imports.push((target, imported));
    }
    Ok(())
}

/// Every `horis:imports` object `doc` declares, as resolved IRI text, in
/// source (declaration) order. Non-node objects (a malformed import) are
/// skipped — well-formedness of the declaration itself is profile
/// validation's job (U3), not this function's.
fn declared_import_targets(doc: &Document, interner: &Interner) -> Vec<String> {
    doc.triples
        .iter()
        .filter(|t| interner.resolve(t.predicate) == HORIS_IMPORTS)
        .filter_map(|t| match &t.object {
            Object::Node(Node::Named(iri)) => Some(interner.resolve(*iri).to_owned()),
            _ => None,
        })
        .collect()
}

/// The unified prefix map (brief requirement 3): starts from `root`'s own
/// prefixes — root always wins, silently, on any conflict — then folds in
/// each import's prefixes in resolution order. A prefix name introduced by
/// one import and then rebound to a *different* IRI by another import
/// (neither being root — "equal standing") is a load error naming the
/// prefix; rebinding it to the *same* IRI is fine.
fn merge_prefixes(
    root: &Document,
    imports: &[(String, Document)],
) -> Result<Vec<(String, String)>, OntologyError> {
    let root_owned: BTreeSet<&str> = root.prefixes.iter().map(|(n, _)| n.as_str()).collect();
    let mut merged: BTreeMap<String, String> = root.prefixes.iter().cloned().collect();

    for (_, doc) in imports {
        for (name, iri) in &doc.prefixes {
            if root_owned.contains(name.as_str()) {
                continue; // root wins, silently, regardless of what an import declares
            }
            match merged.get(name) {
                Some(existing) if existing != iri => {
                    return Err(OntologyError::PrefixConflict {
                        prefix: name.clone(),
                        a: existing.clone(),
                        b: iri.clone(),
                    });
                }
                Some(_) => {} // same binding declared twice: fine
                None => {
                    merged.insert(name.clone(), iri.clone());
                }
            }
        }
    }

    // `merged` is a `BTreeMap`, so this is already sorted by prefix name —
    // deterministic, matching `Document.prefixes`'s own convention (U2).
    Ok(merged.into_iter().collect())
}

// -- The fs touch-point: kept to these three small functions -------------

/// Reads `path`'s contents as UTF-8 Turtle source. The only place this
/// module actually touches a file's *contents* — everything else here is
/// pure graph-resolution logic over already-read [`Document`]s.
fn read_to_string(path: &Path) -> Result<String, OntologyError> {
    std::fs::read_to_string(path).map_err(|e| OntologyError::ImportNotFound {
        import: path.display().to_string(),
        reason: e.to_string(),
    })
}

/// `path`, canonicalized and expressed as a `file://` IRI — used both as a
/// graph's name and as the base a file parses against, so `<>` inside it
/// denotes the file itself and its own relative imports resolve against its
/// directory. Canonicalizing requires the file to exist, so a missing root
/// path (or an explicit `with_import` target) surfaces here as
/// [`OntologyError::ImportNotFound`], naming the path as given.
///
/// The canonical path is spelled as an IRI by [`file_iri_from_canonical`],
/// which is a byte-identical `format!("file://{}", …)` on Unix and rewrites
/// Windows' backslashes and extended-length prefix into the conventional
/// `file:///C:/…` form.
fn file_uri(path: &Path) -> Result<String, OntologyError> {
    let canonical = std::fs::canonicalize(path).map_err(|e| OntologyError::ImportNotFound {
        import: path.display().to_string(),
        reason: e.to_string(),
    })?;
    Ok(file_iri_from_canonical(&canonical))
}

/// Unix: the canonical path is already absolute and slash-separated, so the
/// IRI is `file://` followed by it verbatim (`/tmp/a.ttl` → `file:///tmp/a.ttl`).
#[cfg(not(windows))]
fn file_iri_from_canonical(canonical: &Path) -> String {
    format!("file://{}", canonical.display())
}

/// Windows: the canonical path is spelled as an IRI by
/// [`windows_file_iri`], which drops the extended-length prefix and turns
/// the backslashes into `/`.
#[cfg(windows)]
fn file_iri_from_canonical(canonical: &Path) -> String {
    windows_file_iri(&canonical.display().to_string())
}

/// A Windows path as a `file:` IRI. `std::fs::canonicalize` returns an
/// extended-length path there (`\\?\D:\a\x.ttl`, or `\\?\UNC\server\share\x.ttl`
/// for a network share), and backslashes are not legal IRI code points, so
/// the prefix is dropped and the separators become `/`:
///
/// - `\\?\D:\a\x.ttl` and a prefix-less `D:\a\x.ttl` → `file:///D:/a/x.ttl`
/// - `\\?\UNC\server\share\x.ttl` and a prefix-less `\\server\share\x.ttl`
///   → `file://server/share/x.ttl`, the authority-carrying form that
///   `\\server\share` means
///
/// Compiled on every platform, and exercised by this module's tests on every
/// platform, so the mapping is checked wherever the suite runs;
/// [`file_iri_from_canonical`] only calls it on Windows.
#[cfg(any(windows, test))]
fn windows_file_iri(text: &str) -> String {
    if let Some(share) = text.strip_prefix(r"\\?\UNC\") {
        format!("file://{}", share.replace('\\', "/"))
    } else if let Some(local) = text.strip_prefix(r"\\?\") {
        format!("file:///{}", local.replace('\\', "/"))
    } else if let Some(share) = text.strip_prefix(r"\\") {
        format!("file://{}", share.replace('\\', "/"))
    } else {
        format!("file:///{}", text.replace('\\', "/"))
    }
}

/// The inverse of [`file_uri`]'s scheme: strips the `file://` prefix. `None`
/// for anything else (a remote import IRI, or a malformed one) — the fs
/// convenience only ever reads local files.
#[cfg(not(windows))]
fn path_from_file_uri(iri: &str) -> Option<PathBuf> {
    iri.strip_prefix("file://").map(PathBuf::from)
}

/// `None` for anything that isn't a `file://` IRI, exactly as on Unix;
/// otherwise the path [`windows_path_from_file_uri`] recovers.
#[cfg(windows)]
fn path_from_file_uri(iri: &str) -> Option<PathBuf> {
    iri.strip_prefix("file://")
        .map(|rest| PathBuf::from(windows_path_from_file_uri(rest)))
}

/// The inverse of [`windows_file_iri`], over the part of the IRI after
/// `file://`: `/D:/a/x.ttl` → `D:\a\x.ttl`, and the authority form
/// `server/share/x.ttl` → `\\server\share\x.ttl`. Compiled and tested on
/// every platform; only called on Windows.
#[cfg(any(windows, test))]
fn windows_path_from_file_uri(rest: &str) -> String {
    match rest.strip_prefix('/') {
        Some(local) => local.replace('/', r"\"),
        None => format!(r"\\{}", rest.replace('/', r"\")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ontology::schema::Schema;
    use std::collections::HashMap;

    fn fixture_path(name: &str) -> String {
        format!(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/imports/{}"),
            name
        )
    }

    // -- pure API: load_from_str --------------------------------------------

    #[test]
    fn load_from_str_resolves_root_plus_one_import_plus_one_transitive_import() {
        let mut interner = Interner::new();
        let root_source = "@prefix horis: <https://w3id.org/horismos#> .\n\
                            <urn:test:self> horis:imports <urn:test:a> .\n";
        let a_source = "@prefix horis: <https://w3id.org/horismos#> .\n\
                         <urn:test:self> horis:imports <urn:test:b> .\n";
        let b_source = "@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
                         <urn:test:B> a rdfs:Class .\n";

        let mut sources: HashMap<&str, &str> = HashMap::new();
        sources.insert("urn:test:a", a_source);
        sources.insert("urn:test:b", b_source);

        let set = OntologySet::load_from_str(root_source, &mut interner, |iri| {
            sources.get(iri).map(|s| s.to_string())
        })
        .expect("root + import + transitive import must load");

        let names: Vec<&str> = set.imports.iter().map(|(n, _)| n.as_str()).collect();
        // Dependency order: b (a's own import) resolves before a itself.
        assert_eq!(names, vec!["urn:test:b", "urn:test:a"]);
    }

    #[test]
    fn load_from_str_dedupes_a_diamond_import() {
        let mut interner = Interner::new();
        let root_source = "@prefix horis: <https://w3id.org/horismos#> .\n\
                            <urn:test:self> horis:imports <urn:test:a>, <urn:test:b2> .\n";
        let a_source = "@prefix horis: <https://w3id.org/horismos#> .\n\
                         <urn:test:self> horis:imports <urn:test:shared> .\n";
        let b_source = "@prefix horis: <https://w3id.org/horismos#> .\n\
                         <urn:test:self> horis:imports <urn:test:shared> .\n";
        let shared_source = "@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
                              <urn:test:Shared> a rdfs:Class .\n";

        let mut sources: HashMap<&str, &str> = HashMap::new();
        sources.insert("urn:test:a", a_source);
        sources.insert("urn:test:b2", b_source);
        sources.insert("urn:test:shared", shared_source);

        let set = OntologySet::load_from_str(root_source, &mut interner, |iri| {
            sources.get(iri).map(|s| s.to_string())
        })
        .expect("a diamond import must load, deduped");

        let shared_count = set
            .imports
            .iter()
            .filter(|(n, _)| n == "urn:test:shared")
            .count();
        assert_eq!(shared_count, 1, "each file must be loaded exactly once");
        assert_eq!(set.imports.len(), 3, "a, b2, shared — shared not doubled");
    }

    #[test]
    fn load_from_str_reports_missing_import() {
        let mut interner = Interner::new();
        let root_source = "@prefix horis: <https://w3id.org/horismos#> .\n\
                            <urn:test:self> horis:imports <urn:test:missing> .\n";

        let err = OntologySet::load_from_str(root_source, &mut interner, |_| None)
            .expect_err("an unresolvable import IRI must be a domain-term error");
        match err {
            OntologyError::ImportNotFound { import, .. } => {
                assert_eq!(import, "urn:test:missing");
            }
            other => panic!("expected ImportNotFound, got {other:?}"),
        }
    }

    #[test]
    fn load_from_str_reports_an_import_cycle() {
        let mut interner = Interner::new();
        let root_source = "@prefix horis: <https://w3id.org/horismos#> .\n\
                            <urn:test:self> horis:imports <urn:test:a> .\n";
        let a_source = "@prefix horis: <https://w3id.org/horismos#> .\n\
                         <urn:test:self> horis:imports <urn:test:b> .\n";
        let b_source = "@prefix horis: <https://w3id.org/horismos#> .\n\
                         <urn:test:self> horis:imports <urn:test:a> .\n";

        let mut sources: HashMap<&str, &str> = HashMap::new();
        sources.insert("urn:test:a", a_source);
        sources.insert("urn:test:b", b_source);

        let err = OntologySet::load_from_str(root_source, &mut interner, |iri| {
            sources.get(iri).map(|s| s.to_string())
        })
        .expect_err("A -> B -> A must be rejected as a cycle");
        match err {
            OntologyError::ImportCycle { cycle } => {
                assert!(cycle.contains("urn:test:a"), "{cycle}");
                assert!(cycle.contains("urn:test:b"), "{cycle}");
            }
            other => panic!("expected ImportCycle, got {other:?}"),
        }
    }

    #[test]
    fn load_from_str_reports_a_prefix_conflict_between_two_imports() {
        let mut interner = Interner::new();
        let root_source = "@prefix horis: <https://w3id.org/horismos#> .\n\
                            <urn:test:self> horis:imports <urn:test:a>, <urn:test:b3> .\n";
        let a_source = "@prefix sct: <http://snomed.info/id/> .\n\
                         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
                         sct:X a rdfs:Class .\n";
        let b_source = "@prefix sct: <http://example.org/not-snomed#> .\n\
                         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
                         sct:Y a rdfs:Class .\n";

        let mut sources: HashMap<&str, &str> = HashMap::new();
        sources.insert("urn:test:a", a_source);
        sources.insert("urn:test:b3", b_source);

        let err = OntologySet::load_from_str(root_source, &mut interner, |iri| {
            sources.get(iri).map(|s| s.to_string())
        })
        .expect_err("two imports binding sct: differently must conflict");
        match err {
            OntologyError::PrefixConflict { prefix, .. } => assert_eq!(prefix, "sct"),
            other => panic!("expected PrefixConflict, got {other:?}"),
        }
    }

    #[test]
    fn root_prefix_always_wins_over_an_import_with_the_same_name() {
        let mut interner = Interner::new();
        let root_source = "@prefix sct: <https://example.org/root-sct#> .\n\
                            @prefix horis: <https://w3id.org/horismos#> .\n\
                            <urn:test:self> horis:imports <urn:test:a4> .\n";
        let a_source = "@prefix sct: <http://snomed.info/id/> .\n\
                         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
                         sct:X a rdfs:Class .\n";

        let mut sources: HashMap<&str, &str> = HashMap::new();
        sources.insert("urn:test:a4", a_source);

        let set = OntologySet::load_from_str(root_source, &mut interner, |iri| {
            sources.get(iri).map(|s| s.to_string())
        })
        .expect("root prefix must win, not conflict");

        let sct = set
            .prefixes
            .iter()
            .find(|(name, _)| name == "sct")
            .expect("sct must be present");
        assert_eq!(sct.1, "https://example.org/root-sct#");
    }

    #[test]
    fn validate_names_the_graph_a_profile_violation_came_from() {
        let mut interner = Interner::new();
        let root_source = "@prefix : <https://example.org/imports-fixture#> .\n\
                            @prefix horis: <https://w3id.org/horismos#> .\n\
                            @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
                            <urn:test:self> horis:imports <urn:test:bad-import> .\n\
                            :Patient a rdfs:Class .\n";
        let bad_import_source = "@prefix owl: <http://www.w3.org/2002/07/owl#> .\n\
                                  @prefix : <urn:test:bad-import#> .\n\
                                  :Foo a owl:Restriction .\n";

        let mut sources: HashMap<&str, &str> = HashMap::new();
        sources.insert("urn:test:bad-import", bad_import_source);

        let set = OntologySet::load_from_str(root_source, &mut interner, |iri| {
            sources.get(iri).map(|s| s.to_string())
        })
        .expect("loading itself must succeed — the violation is a profile issue, not a load issue");

        let err = set
            .validate(&interner)
            .expect_err("owl:Restriction in the import must fail profile validation");
        let text = err.domain_text();
        assert!(
            text.contains("urn:test:bad-import"),
            "the violation must name the graph it came from: {text}"
        );
        assert!(text.contains("owl:Restriction"), "{text}");
    }

    #[test]
    fn documents_and_schema_build_all_see_every_graph() {
        let mut interner = Interner::new();
        let root_source = "@prefix : <https://example.org/imports-fixture#> .\n\
                            @prefix horis: <https://w3id.org/horismos#> .\n\
                            @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
                            <urn:test:self> horis:imports <urn:test:vocab> .\n\
                            :Patient a rdfs:Class .\n";
        let vocab_source = "@prefix sct: <http://snomed.info/id/> .\n\
                             @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
                             sct:29857009 a rdfs:Class .\n";

        let mut sources: HashMap<&str, &str> = HashMap::new();
        sources.insert("urn:test:vocab", vocab_source);

        let set = OntologySet::load_from_str(root_source, &mut interner, |iri| {
            sources.get(iri).map(|s| s.to_string())
        })
        .expect("load must succeed");
        set.validate(&interner)
            .expect("both graphs are profile-valid");

        let schema =
            Schema::build_all(set.documents(), &interner).expect("schema build must succeed");
        let patient = interner.intern("https://example.org/imports-fixture#Patient");
        let chest_pain = interner.intern("http://snomed.info/id/29857009");
        assert!(schema.class(patient).is_some(), "root's own class");
        assert!(
            schema.class(chest_pain).is_some(),
            "the imported graph's class must be queryable too"
        );
    }

    // -- fs convenience: OntologySet::load / with_import --------------------

    #[test]
    fn load_reads_the_root_and_its_declared_import_from_disk() {
        let mut interner = Interner::new();
        let set = OntologySet::load(fixture_path("root.ttl"), &mut interner)
            .expect("the fixture root + its declared import must load");

        assert_eq!(set.imports.len(), 1);
        let (name, _) = &set.imports[0];
        assert!(
            name.starts_with("file://") && name.ends_with("snomed-subset.ttl"),
            "{name}"
        );

        set.validate(&interner)
            .expect("the fixture is profile-valid");
        let schema =
            Schema::build_all(set.documents(), &interner).expect("schema build must succeed");
        let patient = interner.intern("https://example.org/imports-fixture#Patient");
        let chest_pain = interner.intern("http://snomed.info/id/29857009");
        assert!(schema.class(patient).is_some());
        assert!(schema.class(chest_pain).is_some());
    }

    #[test]
    fn with_import_adds_an_undeclared_extra_graph() {
        let mut interner = Interner::new();
        let set = OntologySet::load(fixture_path("root.ttl"), &mut interner)
            .expect("root must load")
            .with_import(fixture_path("extra.ttl"), &mut interner)
            .expect("the explicit extra import must load");

        assert_eq!(
            set.imports.len(),
            2,
            "the declared import plus the extra one"
        );
        let schema =
            Schema::build_all(set.documents(), &interner).expect("schema build must succeed");
        let extra = interner.intern("https://example.org/imports-fixture#ExtraClass");
        assert!(schema.class(extra).is_some());
    }

    #[test]
    fn with_import_rejects_a_graph_that_is_already_loaded() {
        let mut interner = Interner::new();
        let set = OntologySet::load(fixture_path("root.ttl"), &mut interner)
            .expect("root must load")
            .with_import(fixture_path("extra.ttl"), &mut interner)
            .expect("the first with_import must succeed");

        let err = set
            .with_import(fixture_path("extra.ttl"), &mut interner)
            .expect_err("adding the same graph twice must be rejected");
        assert!(
            matches!(err, OntologyError::DuplicateGraphName { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn load_reports_a_missing_root_file() {
        let mut interner = Interner::new();
        let err = OntologySet::load(fixture_path("does-not-exist.ttl"), &mut interner)
            .expect_err("a missing root path must be a domain-term error");
        assert!(
            matches!(err, OntologyError::ImportNotFound { .. }),
            "{err:?}"
        );
    }

    // -- the fs touch-point: `file_uri` / `path_from_file_uri` --------------

    /// Constraint 1: the Unix IRI is unchanged — `file://` followed by the
    /// canonical path verbatim. These strings are graph names and parser
    /// bases, so moving them would move identifiers in snapshots and traces.
    #[test]
    #[cfg(not(windows))]
    fn file_uri_on_unix_is_file_slash_slash_plus_the_canonical_path() {
        let path = fixture_path("root.ttl");
        let canonical = std::fs::canonicalize(&path).expect("fixture must exist");
        assert_eq!(
            file_uri(Path::new(&path)).expect("fixture must canonicalize"),
            format!("file://{}", canonical.display())
        );
    }

    /// Constraint 2, on whatever platform the suite runs: the IRI round-trips
    /// back to a path that opens the very same file.
    #[test]
    fn file_uri_round_trips_back_to_the_same_file() {
        let path = fixture_path("root.ttl");
        let iri = file_uri(Path::new(&path)).expect("fixture must canonicalize");
        let recovered = path_from_file_uri(&iri).expect("a file:// IRI must map back to a path");
        assert!(recovered.is_file(), "{recovered:?} must be a readable file");
        assert_eq!(
            std::fs::canonicalize(&recovered).expect("recovered path must canonicalize"),
            std::fs::canonicalize(&path).expect("fixture must canonicalize"),
            "the recovered path must name the same file"
        );
        assert_eq!(
            std::fs::read_to_string(&recovered).expect("recovered path must read"),
            std::fs::read_to_string(&path).expect("fixture must read"),
        );
    }

    /// Constraint 3: the four Windows path spellings, mapped to IRIs. Pure
    /// string logic, so it is checked on every platform.
    #[test]
    fn windows_paths_become_conventional_file_iris() {
        assert_eq!(
            windows_file_iri(r"\\?\D:\a\horismos\examples\triage\triage.ttl"),
            "file:///D:/a/horismos/examples/triage/triage.ttl"
        );
        assert_eq!(
            windows_file_iri(r"D:\a\horismos\triage.ttl"),
            "file:///D:/a/horismos/triage.ttl"
        );
        assert_eq!(
            windows_file_iri(r"\\?\UNC\server\share\triage.ttl"),
            "file://server/share/triage.ttl"
        );
        assert_eq!(
            windows_file_iri(r"\\server\share\triage.ttl"),
            "file://server/share/triage.ttl"
        );
    }

    /// The bug this fixes: the IRI carried backslashes and the `\\?\`
    /// extended-length prefix, which are not legal IRI code points, so the
    /// Turtle parser rejected the base before reading a byte of the file.
    #[test]
    fn a_windows_file_iri_carries_no_backslash_and_no_extended_length_prefix() {
        let iri = windows_file_iri(r"\\?\D:\a\horismos\horismos\examples\triage\triage.ttl");
        assert!(!iri.contains('\\'), "{iri}");
        assert!(!iri.contains('?'), "{iri}");
        assert!(iri.starts_with("file:///D:/"), "{iri}");
    }

    /// Constraint 2 for the Windows halves specifically, checked on every
    /// platform: the IRI maps back to a path naming the same file. A drive
    /// path loses only its extended-length prefix, which `\\?\D:\…` and
    /// `D:\…` both open through.
    #[test]
    fn windows_file_iris_round_trip_back_to_paths() {
        for (path, expected) in [
            (r"\\?\D:\a\triage.ttl", r"D:\a\triage.ttl"),
            (r"D:\a\triage.ttl", r"D:\a\triage.ttl"),
            (
                r"\\?\UNC\server\share\triage.ttl",
                r"\\server\share\triage.ttl",
            ),
            (r"\\server\share\triage.ttl", r"\\server\share\triage.ttl"),
        ] {
            let iri = windows_file_iri(path);
            let rest = iri
                .strip_prefix("file://")
                .expect("windows_file_iri must emit a file:// IRI");
            assert_eq!(windows_path_from_file_uri(rest), expected, "from {path}");
        }
    }

    /// A relative import resolved against a Windows base comes back as a
    /// sibling path — the property `fs_fetch` depends on.
    #[test]
    fn a_sibling_of_a_windows_base_iri_maps_back_to_a_sibling_path() {
        let base = windows_file_iri(r"\\?\D:\a\triage\triage.ttl");
        assert_eq!(base, "file:///D:/a/triage/triage.ttl");
        let sibling = "file:///D:/a/triage/rules.ttl";
        let rest = sibling.strip_prefix("file://").expect("file:// IRI");
        assert_eq!(windows_path_from_file_uri(rest), r"D:\a\triage\rules.ttl");
    }

    /// Unchanged on both platforms: a non-`file://` IRI has no local path.
    #[test]
    fn path_from_file_uri_rejects_a_remote_iri() {
        assert!(path_from_file_uri("https://example.org/a.ttl").is_none());
        assert!(path_from_file_uri("urn:test:a").is_none());
    }
}
