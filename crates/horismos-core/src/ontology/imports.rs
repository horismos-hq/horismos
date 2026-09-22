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
//! Ontology loading is a trust boundary (`docs/framework/15-security.md`
//! §6): the ontology is the program, so a root document does not get to
//! decide by its own text which files on the host join it. Every declared
//! import is therefore confined to [`ImportRoots`] — by default the root
//! document's own canonical directory — and a `file:` IRI that names a
//! host is refused outright rather than mapped to a path.
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

/// The deepest transitive `horis:imports` chain a filesystem load will
/// follow. The same figure as [`crate::reason::rule::MAX_CONDITION_DEPTH`]
/// and [`crate::reason::backward::MAX_DEPTH`]: the traversal is recursive,
/// cycles alone do not bound it, and a stack overflow aborts the process
/// rather than raising anything a caller could catch.
///
/// Headroom, not a crash threshold: no ontology in `examples/` declares a
/// `horis:imports` chain at all (`examples/triage/` supplies its SNOMED CT
/// subset as a caller-named `with_import` instead), and the deepest declared
/// chain in the repository is the one level this module's own fixtures use.
/// Reaching 50 means an import chain no reader can hold in their head, not a
/// deployment this bound is squeezing.
pub const MAX_IMPORT_DEPTH: usize = 50;

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
    ///
    /// Every import resolved here is confined to `path`'s own canonical
    /// directory — see [`ImportRoots`], and
    /// [`OntologySet::load_with_roots`] for widening that.
    pub fn load(
        path: impl AsRef<Path>,
        interner: &mut Interner,
    ) -> Result<OntologySet, OntologyError> {
        let path = path.as_ref();
        let roots = ImportRoots::for_root_document(path, &[])?;
        Self::load_with_roots(path, interner, &roots)
    }

    /// [`OntologySet::load`] against an explicit containment policy — the
    /// shape a deployment that shares an ontology directory across several
    /// root documents needs.
    pub fn load_with_roots(
        path: impl AsRef<Path>,
        interner: &mut Interner,
        roots: &ImportRoots,
    ) -> Result<OntologySet, OntologyError> {
        let path = path.as_ref();
        let root_name = file_uri(path)?;
        let source = read_to_string(path)?;
        let root = turtle::parse_with_base(&source, &root_name, interner)?;

        let mut fetch = fs_fetch(roots);
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
    ///
    /// `path` itself is a caller act, so it is read wherever it points; the
    /// graphs *it* goes on to declare are confined to `roots` widened by
    /// `path`'s own directory ([`ImportRoots::with_directory_of`]).
    pub fn with_import(
        mut self,
        path: impl AsRef<Path>,
        interner: &mut Interner,
        roots: &ImportRoots,
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
        let nested_roots = roots.with_directory_of(path)?;
        let mut fetch = fs_fetch(&nested_roots);
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

/// The directories a filesystem ontology load may read imports from.
///
/// Ontology loading is a trust boundary: the ontology *is* the program (doc
/// 14), so a `horis:imports` target that escapes the document's own
/// directory pulls decision logic in from somewhere nobody signed off on.
/// By default a load is therefore confined to the canonical parent
/// directory of its root document, and a deployment that legitimately
/// shares an ontology directory declares that intent explicitly
/// ([`crate::LoadOptions::permitted_import_roots`]) rather than getting it
/// by accident.
///
/// Every root here is canonical (symlinks resolved), and so is every path
/// tested against it, so containment is a component-wise path comparison
/// and not a string prefix: `/srv/ontology-evil` is not under
/// `/srv/ontology`, and a symlink inside a permitted root cannot walk out
/// of it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportRoots {
    roots: Vec<PathBuf>,
}

impl ImportRoots {
    /// The default policy for a load of `root_document`: its own canonical
    /// parent directory, plus each of `extra` (a deployment's explicitly
    /// permitted shared ontology directories). An `extra` that does not
    /// exist, or is not a directory, is a caller configuration error
    /// ([`OntologyError::PermittedRootInvalid`]) rather than a silently
    /// dropped permission.
    pub fn for_root_document(
        root_document: &Path,
        extra: &[PathBuf],
    ) -> Result<ImportRoots, OntologyError> {
        let canonical =
            std::fs::canonicalize(root_document).map_err(|e| OntologyError::ImportNotFound {
                import: root_document.display().to_string(),
                reason: e.to_string(),
            })?;
        // A canonicalized file always has a parent; a canonicalized root
        // directory ("/") is its own containment root.
        let mut roots = vec![canonical.parent().unwrap_or(&canonical).to_path_buf()];
        for root in extra {
            roots.push(Self::canonical_directory(root)?);
        }
        roots.sort();
        roots.dedup();
        Ok(ImportRoots { roots })
    }

    /// The same set widened by the directory of `path` — what an explicit,
    /// caller-named [`OntologySet::with_import`] target contributes. The
    /// caller naming a file in its own source is a deliberate act; the
    /// graphs *that* file goes on to declare are not, so they stay confined.
    pub fn with_directory_of(&self, path: &Path) -> Result<ImportRoots, OntologyError> {
        let canonical = std::fs::canonicalize(path).map_err(|e| OntologyError::ImportNotFound {
            import: path.display().to_string(),
            reason: e.to_string(),
        })?;
        let mut roots = self.roots.clone();
        roots.push(canonical.parent().unwrap_or(&canonical).to_path_buf());
        roots.sort();
        roots.dedup();
        Ok(ImportRoots { roots })
    }

    /// Whether `path` — already canonical, or normalized as far as the
    /// filesystem allowed — lies under one of the permitted roots.
    fn permits(&self, path: &Path) -> bool {
        self.roots.iter().any(|root| path.starts_with(root))
    }

    /// The roots as an error message names them, in sorted order.
    fn describe(&self) -> String {
        if self.roots.is_empty() {
            return "none".to_owned();
        }
        self.roots
            .iter()
            .map(|r| r.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn canonical_directory(root: &Path) -> Result<PathBuf, OntologyError> {
        let canonical =
            std::fs::canonicalize(root).map_err(|e| OntologyError::PermittedRootInvalid {
                root: root.display().to_string(),
                reason: e.to_string(),
            })?;
        if !canonical.is_dir() {
            return Err(OntologyError::PermittedRootInvalid {
                root: root.display().to_string(),
                reason: "not a directory".to_owned(),
            });
        }
        Ok(canonical)
    }
}

/// A `fetch` closure for the filesystem loaders ([`OntologySet::load`] /
/// [`OntologySet::with_import`]): resolves an already-absolute `file://`
/// import IRI back to a path, checks it against `roots`, reads it if
/// present, and parses it with *that file's own* location as base (so its
/// own relative imports resolve against its directory, not the root's).
/// `Ok(None)` — "not found" — for anything that isn't a local, existing
/// `file://` path: remote imports (`http://…`) aren't supported by the v1
/// fs convenience (decision D5: no I/O beyond reading declared *local*
/// imports).
///
/// Containment is decided before existence, and its refusal names only the
/// import IRI the document already chose and the roots the deployment
/// already configured. Neither depends on whether the target is there, so
/// an out-of-root import cannot be used to probe the filesystem.
fn fs_fetch(
    roots: &ImportRoots,
) -> impl FnMut(&str, &mut Interner) -> Result<Option<Document>, OntologyError> + '_ {
    move |target: &str, interner: &mut Interner| {
        let Some(path) = path_from_file_uri(target) else {
            return Ok(None);
        };
        // Resolve symlinks where the filesystem lets us — a link inside a
        // permitted root that points outside it is an escape. A target that
        // does not exist has no canonical form, so fall back to its deepest
        // existing ancestor's (which keeps the comparison in the same
        // spelling the roots are in — on Windows, canonicalization adds the
        // extended-length prefix, and a lexical path would compare against
        // it as an escape rather than as the missing file it is), and to
        // plain lexical normalization only if even that fails. Every branch
        // produces the identical refusal, so none of them leaks.
        let resolved = std::fs::canonicalize(&path).unwrap_or_else(|_| {
            match (path.parent(), path.file_name()) {
                (Some(parent), Some(name)) => std::fs::canonicalize(parent)
                    .map(|dir| dir.join(name))
                    .unwrap_or_else(|_| lexically_normalize(&path)),
                _ => lexically_normalize(&path),
            }
        });
        if !roots.permits(&resolved) {
            return Err(OntologyError::ImportOutsideRoots {
                import: target.to_owned(),
                roots: roots.describe(),
            });
        }
        if !resolved.is_file() {
            return Ok(None);
        }
        let source = read_to_string(&resolved)?;
        turtle::parse_with_base(&source, target, interner).map(Some)
    }
}

/// `path` with `.` and `..` segments folded out textually — no filesystem
/// access, so no symlink resolution. Only ever a fallback for a path
/// [`std::fs::canonicalize`] refused (which, for an import, means it does
/// not exist): a real target is judged canonically.
fn lexically_normalize(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push(component.as_os_str());
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
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
        // `in_progress` is pushed before each recursive call and popped
        // after, so its length *is* the current depth. Checked before the
        // fetch, so an over-deep chain is refused without reading the file.
        if in_progress.len() >= MAX_IMPORT_DEPTH {
            return Err(OntologyError::ImportChainTooDeep {
                import: target.clone(),
                max: MAX_IMPORT_DEPTH,
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

/// The local-path body of the part of a `file:` IRI after `file://`:
/// everything past the authority component, and `None` unless that
/// authority names *this* host.
///
/// A `file:` IRI that carries a host is not a local path. Mapping one to a
/// path is how `file://attacker.example.com/share/evil.ttl` becomes the UNC
/// path `\\attacker.example.com\share\evil.ttl`, which Windows resolves
/// over the network — turning an ontology load into an outbound fetch, and
/// an outbound NTLM authentication with it. That is exactly what this
/// module's header and doc 02's decision D5 say never happens, so the
/// authority form is refused here, once, for every platform.
///
/// The empty authority (`file:///path`, what [`file_uri`] itself emits) is
/// the local form. `file://localhost/path` is accepted as equivalent to it,
/// which is the one conventional exception (RFC 8089 §2 defines exactly
/// that equivalence, and a `localhost` target resolves locally rather than
/// over the network, so accepting it costs nothing). Every other authority
/// is refused.
fn file_uri_local_body(rest: &str) -> Option<&str> {
    if rest.is_empty() || rest.starts_with('/') {
        return Some(rest); // empty authority: file:///path
    }
    let split = rest.find('/')?;
    let (authority, body) = rest.split_at(split);
    authority.eq_ignore_ascii_case("localhost").then_some(body)
}

/// The inverse of [`file_uri`]'s scheme: the local path a `file:` IRI
/// names. `None` for anything else — a remote import IRI, a malformed one,
/// or (see [`file_uri_local_body`]) a `file:` IRI naming a host. The fs
/// convenience only ever reads local files.
#[cfg(not(windows))]
fn path_from_file_uri(iri: &str) -> Option<PathBuf> {
    file_uri_local_body(iri.strip_prefix("file://")?).map(PathBuf::from)
}

/// `None` for anything that isn't a local `file://` IRI, exactly as on
/// Unix; otherwise the path [`windows_path_from_file_uri`] recovers.
#[cfg(windows)]
fn path_from_file_uri(iri: &str) -> Option<PathBuf> {
    windows_path_from_file_uri(iri.strip_prefix("file://")?).map(PathBuf::from)
}

/// The inverse of [`windows_file_iri`], over the part of the IRI after
/// `file://`: `/D:/a/x.ttl` → `D:\a\x.ttl`. `None` for the authority form
/// (`server/share/x.ttl`), which would otherwise become the UNC path
/// `\\server\share\x.ttl` — see [`file_uri_local_body`] for why that is
/// refused rather than mapped.
///
/// [`windows_file_iri`] still *emits* `file://server/share/…` for a UNC
/// path handed to it as the root document, because that is the honest IRI
/// for such a path; the mapping is deliberately one-way. A root document
/// opened from a UNC share therefore loads, and its relative imports do
/// not. Reading files off a named network share is a deliberate act that
/// belongs to whoever opens the root, not to whatever the root's text asks
/// for next.
///
/// Compiled and tested on every platform; only called on Windows.
#[cfg(any(windows, test))]
fn windows_path_from_file_uri(rest: &str) -> Option<String> {
    let body = file_uri_local_body(rest)?;
    let local = body.strip_prefix('/').unwrap_or(body);
    Some(local.replace('/', r"\"))
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

    /// The default containment policy for a fixture-rooted load.
    fn fixture_roots(root: &str) -> ImportRoots {
        ImportRoots::for_root_document(Path::new(root), &[])
            .expect("the fixture root must canonicalize")
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
        let roots = fixture_roots(&fixture_path("root.ttl"));
        let set = OntologySet::load(fixture_path("root.ttl"), &mut interner)
            .expect("root must load")
            .with_import(fixture_path("extra.ttl"), &mut interner, &roots)
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
        let roots = fixture_roots(&fixture_path("root.ttl"));
        let set = OntologySet::load(fixture_path("root.ttl"), &mut interner)
            .expect("root must load")
            .with_import(fixture_path("extra.ttl"), &mut interner, &roots)
            .expect("the first with_import must succeed");

        let err = set
            .with_import(fixture_path("extra.ttl"), &mut interner, &roots)
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
    /// platform: a *drive* IRI maps back to a path naming the same file. A
    /// drive path loses only its extended-length prefix, which `\\?\D:\…`
    /// and `D:\…` both open through.
    ///
    /// The UNC spellings are deliberately not here: see
    /// [`unc_file_iris_do_not_map_back_to_a_unc_path`] for why the mapping
    /// is one-way.
    #[test]
    fn windows_drive_file_iris_round_trip_back_to_paths() {
        for (path, expected) in [
            (r"\\?\D:\a\triage.ttl", r"D:\a\triage.ttl"),
            (r"D:\a\triage.ttl", r"D:\a\triage.ttl"),
        ] {
            let iri = windows_file_iri(path);
            let rest = iri
                .strip_prefix("file://")
                .expect("windows_file_iri must emit a file:// IRI");
            assert_eq!(
                windows_path_from_file_uri(rest).as_deref(),
                Some(expected),
                "from {path}"
            );
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
        assert_eq!(
            windows_path_from_file_uri(rest).as_deref(),
            Some(r"D:\a\triage\rules.ttl")
        );
    }

    /// Unchanged on both platforms: a non-`file://` IRI has no local path.
    #[test]
    fn path_from_file_uri_rejects_a_remote_iri() {
        assert!(path_from_file_uri("https://example.org/a.ttl").is_none());
        assert!(path_from_file_uri("urn:test:a").is_none());
    }

    // -- B4: a `file:` IRI naming a host is not a local path ----------------

    /// The defect: `file://attacker.example.com/share/evil.ttl` mapped to
    /// the UNC path `\\attacker.example.com\share\evil.ttl`, which Windows
    /// resolves over the network — so a declared import became an outbound
    /// SMB fetch (and an outbound NTLM authentication), which decision D5
    /// and this module's own header say never happens.
    ///
    /// `windows_path_from_file_uri` is `#[cfg(any(windows, test))]`
    /// precisely so this is checked wherever the suite runs, not only on
    /// Windows.
    #[test]
    fn a_file_iri_naming_a_host_never_becomes_a_windows_unc_path() {
        for iri in [
            "file://attacker.example.com/share/evil.ttl",
            "file://10.1.2.3/pwn/x.ttl",
            "file://server/share/triage.ttl",
            "file://LOCALHOST.attacker.example.com/share/x.ttl",
        ] {
            let rest = iri.strip_prefix("file://").expect("file:// IRI");
            assert_eq!(
                windows_path_from_file_uri(rest),
                None,
                "{iri} must not map to any local path"
            );
            assert_eq!(
                path_from_file_uri(iri),
                None,
                "{iri} must not map to any local path"
            );
        }
    }

    /// The forward direction from commit `a40a496` is untouched: a UNC path
    /// handed in as the *root document* still gets an honest IRI. Only the
    /// inverse is refused, so the root a caller opens still loads while the
    /// IRIs its text asks for next cannot name a host.
    #[test]
    fn unc_file_iris_do_not_map_back_to_a_unc_path() {
        for path in [
            r"\\?\UNC\server\share\triage.ttl",
            r"\\server\share\triage.ttl",
        ] {
            let iri = windows_file_iri(path);
            assert_eq!(iri, "file://server/share/triage.ttl", "from {path}");
            let rest = iri.strip_prefix("file://").expect("file:// IRI");
            assert_eq!(
                windows_path_from_file_uri(rest),
                None,
                "the authority form must not map back to a UNC path"
            );
        }
    }

    // -- B2/B3: depth and containment on filesystem loads -------------------

    /// A private scratch directory for one test, named after it so parallel
    /// tests never collide. Removed on entry and on exit.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Scratch {
            let dir = std::env::temp_dir().join(format!("horismos-imports-test-{name}"));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).expect("scratch directory must be creatable");
            Scratch(dir)
        }

        fn write(&self, relative: &str, source: &str) -> PathBuf {
            let path = self.0.join(relative);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("scratch subdirectory must be creatable");
            }
            std::fs::write(&path, source).expect("scratch file must be writable");
            path
        }

        fn dir(&self, relative: &str) -> PathBuf {
            let path = self.0.join(relative);
            std::fs::create_dir_all(&path).expect("scratch subdirectory must be creatable");
            path
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    const HORIS_PREFIX: &str = "@prefix horis: <https://w3id.org/horismos#> .\n";

    /// A chain `f0 → f1 → … → fn`, each file importing the next; the last
    /// one declares a class instead. Built programmatically — the depth that
    /// actually overflows the stack is thousands of files, which is not a
    /// fixture anyone should commit.
    fn write_import_chain(scratch: &Scratch, links: usize) -> PathBuf {
        for i in 0..links {
            scratch.write(
                &format!("f{i}.ttl"),
                &format!("{HORIS_PREFIX}<> horis:imports <./f{}.ttl> .\n", i + 1),
            );
        }
        scratch.write(
            &format!("f{links}.ttl"),
            "@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
             <https://example.org/t#C> a rdfs:Class .\n",
        );
        scratch.0.join("f0.ttl")
    }

    /// B2: cycles were caught, depth was not — and an acyclic chain of a few
    /// thousand files overflowed the stack, which aborts the process rather
    /// than raising anything a caller could catch.
    #[test]
    fn a_transitive_import_chain_deeper_than_the_cap_is_refused() {
        let scratch = Scratch::new("deep-import-chain");
        let root = write_import_chain(&scratch, MAX_IMPORT_DEPTH + 1);
        let mut interner = Interner::new();
        let err = OntologySet::load(&root, &mut interner)
            .expect_err("a chain deeper than the cap must be refused");
        assert!(
            matches!(
                &err,
                OntologyError::ImportChainTooDeep { max, .. } if *max == MAX_IMPORT_DEPTH
            ),
            "{err:?}"
        );
    }

    /// The bound is a bound, not a ban: a chain right at the cap still
    /// loads, so the refusal above is about depth and nothing else.
    #[test]
    fn a_transitive_import_chain_at_the_cap_still_loads() {
        let scratch = Scratch::new("import-chain-at-cap");
        let root = write_import_chain(&scratch, MAX_IMPORT_DEPTH);
        let mut interner = Interner::new();
        let set =
            OntologySet::load(&root, &mut interner).expect("a chain at the cap must still load");
        assert_eq!(set.imports.len(), MAX_IMPORT_DEPTH);
    }

    /// B3, consequence 1: `<> horis:imports <../secret.ttl>` walked out of
    /// the root document's directory and merged whatever it found — up to
    /// and including foreign `horis:Rule` declarations. The ontology is the
    /// program, so that is decision logic arriving from off the record.
    #[test]
    fn an_import_that_escapes_the_root_directory_is_refused() {
        let scratch = Scratch::new("escape-root-directory");
        scratch.write(
            "secret.ttl",
            "@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
             <https://example.org/t#Foreign> a rdfs:Class .\n",
        );
        let root = scratch.write(
            "project/root.ttl",
            &format!("{HORIS_PREFIX}<> horis:imports <../secret.ttl> .\n"),
        );

        let mut interner = Interner::new();
        let err = OntologySet::load(&root, &mut interner)
            .expect_err("an import outside the root document's directory must be refused");
        assert!(
            matches!(&err, OntologyError::ImportOutsideRoots { .. }),
            "{err:?}"
        );
        assert!(
            err.to_string()
                .contains("outside the permitted ontology roots"),
            "the refusal must name the escape: {err}"
        );
    }

    /// B3, consequences 2 and 3: an absolute import read any file the
    /// process could read, echoed a fragment of it back through the parse
    /// error, and distinguished "exists but is not Turtle" from "does not
    /// exist" — a filesystem oracle, one load at a time.
    ///
    /// Both spellings must now produce the *same* refusal, because a
    /// refusal that varies with existence is still an oracle.
    #[test]
    fn an_absolute_import_outside_the_roots_is_refused_without_an_existence_oracle() {
        let scratch = Scratch::new("absolute-import-oracle");
        let existing = scratch.write("outside/passwd-like.txt", "root:x:0:0:root:/root:/bin/sh\n");
        let missing = scratch.0.join("outside/definitely-not-here.txt");

        let mut refusals = Vec::new();
        for target in [&existing, &missing] {
            let root = scratch.write(
                "project/root.ttl",
                &format!(
                    "{HORIS_PREFIX}<> horis:imports <file://{}> .\n",
                    target.display()
                ),
            );
            let mut interner = Interner::new();
            let err = OntologySet::load(&root, &mut interner)
                .expect_err("an absolute import outside the roots must be refused");
            let OntologyError::ImportOutsideRoots { import, roots } = err else {
                panic!("expected an out-of-roots refusal, got {err:?}");
            };
            assert!(
                !import.contains("root:x:0:0"),
                "the refusal must not echo the file's contents: {import}"
            );
            refusals.push((import, roots));
        }
        // The two refusals differ only in the import IRI the *document*
        // chose, which the author already knows; everything the loader adds
        // is identical, so the refusal reveals nothing about what is on
        // disk.
        assert_eq!(
            refusals[0].1, refusals[1].1,
            "an existing and a missing out-of-root target must be refused identically"
        );
        for (import, _) in &refusals {
            assert!(import.starts_with("file://"), "{import}");
        }
    }

    /// The captain-mandated escape hatch: a deployment that legitimately
    /// shares an ontology directory declares that intent, and the same
    /// import then resolves.
    #[test]
    fn an_explicitly_permitted_root_lets_a_shared_ontology_directory_import() {
        let scratch = Scratch::new("permitted-extra-root");
        let shared = scratch.dir("shared");
        scratch.write(
            "shared/vocab.ttl",
            "@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
             <https://example.org/t#Shared> a rdfs:Class .\n",
        );
        let root = scratch.write(
            "project/root.ttl",
            &format!("{HORIS_PREFIX}<> horis:imports <../shared/vocab.ttl> .\n"),
        );

        let mut interner = Interner::new();
        OntologySet::load(&root, &mut interner)
            .expect_err("without the declaration the import stays refused");

        let mut interner = Interner::new();
        let roots = ImportRoots::for_root_document(&root, std::slice::from_ref(&shared))
            .expect("the declared root must canonicalize");
        let set = OntologySet::load_with_roots(&root, &mut interner, &roots)
            .expect("a declared shared ontology directory must import");
        assert_eq!(set.imports.len(), 1);
    }

    /// Containment is a canonical-path comparison, not a string prefix: a
    /// symlink inside a permitted root that points outside it is an escape.
    #[test]
    #[cfg(unix)]
    fn a_symlink_cannot_walk_out_of_a_permitted_root() {
        let scratch = Scratch::new("symlink-escape");
        scratch.write(
            "secret.ttl",
            "@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
             <https://example.org/t#Foreign> a rdfs:Class .\n",
        );
        let project = scratch.dir("project");
        std::os::unix::fs::symlink(scratch.0.join("secret.ttl"), project.join("link.ttl"))
            .expect("the scratch symlink must be creatable");
        let root = scratch.write(
            "project/root.ttl",
            &format!("{HORIS_PREFIX}<> horis:imports <./link.ttl> .\n"),
        );

        let mut interner = Interner::new();
        let err = OntologySet::load(&root, &mut interner)
            .expect_err("a symlink out of the permitted roots must be refused");
        assert!(
            matches!(&err, OntologyError::ImportOutsideRoots { .. }),
            "{err:?}"
        );
    }

    /// A sibling directory whose name merely *starts with* a permitted
    /// root's name is not under it — the property raw prefix matching would
    /// get wrong.
    #[test]
    fn a_sibling_directory_sharing_a_name_prefix_is_not_contained() {
        let scratch = Scratch::new("name-prefix-sibling");
        scratch.write(
            "project-evil/evil.ttl",
            "@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
             <https://example.org/t#Foreign> a rdfs:Class .\n",
        );
        let root = scratch.write(
            "project/root.ttl",
            &format!("{HORIS_PREFIX}<> horis:imports <../project-evil/evil.ttl> .\n"),
        );

        let mut interner = Interner::new();
        let err = OntologySet::load(&root, &mut interner)
            .expect_err("a name-prefix sibling must not count as contained");
        assert!(
            matches!(&err, OntologyError::ImportOutsideRoots { .. }),
            "{err:?}"
        );
    }

    /// A missing import *inside* the permitted roots still reads as missing,
    /// not as an escape — the two facts stay distinguishable where the
    /// reader is entitled to both.
    #[test]
    fn a_missing_import_inside_the_roots_is_still_reported_as_missing() {
        let scratch = Scratch::new("missing-inside-roots");
        let root = scratch.write(
            "root.ttl",
            &format!("{HORIS_PREFIX}<> horis:imports <./absent.ttl> .\n"),
        );
        let mut interner = Interner::new();
        let err = OntologySet::load(&root, &mut interner)
            .expect_err("a missing import must still be refused");
        assert!(
            matches!(&err, OntologyError::ImportNotFound { .. }),
            "{err:?}"
        );
    }

    /// The local spellings still resolve, including the one conventional
    /// exception: RFC 8089 §2 makes `file://localhost/p` equivalent to
    /// `file:///p`, and a `localhost` target resolves locally.
    #[test]
    fn the_empty_and_localhost_authorities_are_the_local_forms() {
        assert_eq!(
            path_from_file_uri("file:///tmp/a.ttl"),
            Some(PathBuf::from("/tmp/a.ttl"))
        );
        assert_eq!(file_uri_local_body("/tmp/a.ttl"), Some("/tmp/a.ttl"));
        assert_eq!(
            file_uri_local_body("localhost/tmp/a.ttl"),
            Some("/tmp/a.ttl")
        );
        assert_eq!(
            file_uri_local_body("LocalHost/tmp/a.ttl"),
            Some("/tmp/a.ttl")
        );
        assert_eq!(file_uri_local_body("evil.example.com/tmp/a.ttl"), None);
        assert_eq!(
            windows_path_from_file_uri("localhost/D:/a/x.ttl").as_deref(),
            Some(r"D:\a\x.ttl")
        );
    }
}
