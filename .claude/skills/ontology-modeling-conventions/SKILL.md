---
name: ontology-modeling-conventions
description: Use before writing or editing any Turtle (.ttl) file, ontology fixture, or code that emits/consumes the horismos vocabulary — covers the RDFS-plus profile, the horis: namespace terms, the 11 condition operators, constraint and workflow vocabulary, IRI and prefix conventions, and what is permanently excluded (OWL DL, SWRL, SPARQL Update).
---

# Ontology modeling conventions

Vocabulary authority: `docs/framework/02-ontology.md` (constraints, rules, imports),
`04-reasoning.md` (operator semantics), `05-workflows.md` (workflow vocabulary).
Never invent a `horis:` term — if a needed term doesn't exist, that's a **[DECIDE]**
for the owning PRP, not an ad-hoc addition.

## Namespaces & prefixes (canonical order, always these names)

```turtle
@prefix :      <https://example.org/DOMAIN#> .   # the domain being modeled
@prefix horis: <https://w3id.org/horismos#> .
@prefix sct:   <http://snomed.info/id/> .        # only when anchoring to SNOMED
@prefix rdf:   <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs:  <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:   <http://www.w3.org/2002/07/owl#> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .
```

## The profile in one breath

RDF triples + RDFS (Class, subClassOf, domain, range, label, comment) + XSD
datatypes (decimal, integer, string, boolean, dateTime, duration) + `owl:sameAs`
(the ONLY OWL term, for external anchoring) + the `horis:` vocabulary.
**Permanently excluded**: OWL DL/Full, SWRL, SPARQL Update, property paths.

## Cheat sheets

- Constraints: `horis:min/max` (inclusive), `horis:unit`, `horis:cardinality`
  ("1" | "0..1" | "1..*" | "0..*"), `horis:oneOf` (closed enum), `horis:pattern`,
  `horis:immutable`, `horis:assignedBy` (`horis:RulesOnly` | `horis:PerceptionAllowed`
  | `horis:ToolsOnly`). Decision properties an LLM must never set get
  `horis:assignedBy horis:RulesOnly`.
- Operators (exactly these 11): `exceeds, below, atLeast, atMost, between, is,
  isNot, isAssigned, isEmpty, contains, inList`. Combinators: `all` (∧), `any` (∨),
  `none` (¬∨). `horis:is X` matches subclasses via subsumption. Unassigned property
  ⇒ comparisons false, `isNot` true (pragmatic closed world).
- Rules: `a horis:Rule` with `horis:priority`, `horis:when`, `horis:then`, and a
  human `horis:explain` — the explain string is **mandatory**; it surfaces in proof
  trees and narration.
- Workflows: every `PerceptionStep` requires `horis:fallback`; every `ReasoningStep`
  requires `horis:onUnreachable`; every `ActionStep` requires `horis:onError`.
  Every run must reach a `horis:terminal true` step.

## Fixture style

`examples/triage/triage.ttl` is the canonical style reference: section headers
(`## Domain (doc 02)`), one blank line between subjects, aligned predicates,
SNOMED anchors commented. New fixtures follow it; the triage file itself must
exercise every shipped feature.
