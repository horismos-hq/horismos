---
name: ontology-modeling-conventions
description: Use before writing or editing any Turtle (.ttl) file, ontology fixture, or code that emits/consumes the sigil vocabulary — covers the RDFS-plus profile, the sigil: namespace terms, the 11 condition operators, constraint and workflow vocabulary, IRI and prefix conventions, and what is permanently excluded (OWL DL, SWRL, SPARQL Update).
---

# Ontology modeling conventions

Vocabulary authority: `docs/framework/02-ontology.md` (constraints, rules, imports),
`04-reasoning.md` (operator semantics), `05-workflows.md` (workflow vocabulary).
Never invent a `sigil:` term — if a needed term doesn't exist, that's a **[DECIDE]**
for the owning PRP, not an ad-hoc addition.

## Namespaces & prefixes (canonical order, always these names)

```turtle
@prefix :      <https://example.org/DOMAIN#> .   # the domain being modeled
@prefix sigil: <https://sigil.dev/ns#> .
@prefix sct:   <http://snomed.info/id/> .        # only when anchoring to SNOMED
@prefix rdf:   <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs:  <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:   <http://www.w3.org/2002/07/owl#> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .
```

## The profile in one breath

RDF triples + RDFS (Class, subClassOf, domain, range, label, comment) + XSD
datatypes (decimal, integer, string, boolean, dateTime, duration) + `owl:sameAs`
(the ONLY OWL term, for external anchoring) + the `sigil:` vocabulary.
**Permanently excluded**: OWL DL/Full, SWRL, SPARQL Update, property paths.

## Cheat sheets

- Constraints: `sigil:min/max` (inclusive), `sigil:unit`, `sigil:cardinality`
  ("1" | "0..1" | "1..*" | "0..*"), `sigil:oneOf` (closed enum), `sigil:pattern`,
  `sigil:immutable`, `sigil:assignedBy` (`sigil:RulesOnly` | `sigil:PerceptionAllowed`
  | `sigil:ToolsOnly`). Decision properties an LLM must never set get
  `sigil:assignedBy sigil:RulesOnly`.
- Operators (exactly these 11): `exceeds, below, atLeast, atMost, between, is,
  isNot, isAssigned, isEmpty, contains, inList`. Combinators: `all` (∧), `any` (∨),
  `none` (¬∨). `sigil:is X` matches subclasses via subsumption. Unassigned property
  ⇒ comparisons false, `isNot` true (pragmatic closed world).
- Rules: `a sigil:Rule` with `sigil:priority`, `sigil:when`, `sigil:then`, and a
  human `sigil:explain` — the explain string is **mandatory**; it surfaces in proof
  trees and narration.
- Workflows: every `PerceptionStep` requires `sigil:fallback`; every `ReasoningStep`
  requires `sigil:onUnreachable`; every `ActionStep` requires `sigil:onError`.
  Every run must reach a `sigil:terminal true` step.

## Fixture style

`examples/triage/triage.ttl` is the canonical style reference: section headers
(`## Domain (doc 02)`), one blank line between subjects, aligned predicates,
SNOMED anchors commented. New fixtures follow it; the triage file itself must
exercise every shipped feature.
