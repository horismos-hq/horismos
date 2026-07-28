# 02 · The Ontology (RDFS-plus)

The ontology is the single authored artifact of a Sigil deployment. It is standard
Turtle (`.ttl`), readable by any RDF tool, extended with the `sigil:` vocabulary that
makes it *executable*.

## The RDFS-plus profile

Sigil deliberately takes a small, tractable slice of the semantic-web stack:

**Included (base):**
- RDF triples; Turtle syntax in/out; named graphs for imports.
- RDFS: `rdfs:Class`, `rdfs:subClassOf`, `rdf:Property`, `rdfs:domain`, `rdfs:range`,
  `rdfs:label`, `rdfs:comment`.
- Datatypes: `xsd:decimal`, `xsd:integer`, `xsd:string`, `xsd:boolean`,
  `xsd:dateTime`, `xsd:duration`.
- `owl:sameAs` for anchoring to external vocabularies (that is the *only* OWL term).

**Included (the "plus" — `sigil:` namespace, `https://sigil.dev/ns#`):**
- Constraint vocabulary (SHACL-inspired, executable at the validation gate).
- Rule vocabulary (Datalog-shaped conditions and conclusions).
- Workflow vocabulary (steps, transitions, goals — executed by the Agent).
- Tool contracts.

**Excluded, permanently:** OWL DL/Full (restrictions, complex class expressions,
property chains), open-ended SWRL, SPARQL Update. If inference over it can't be
deterministic and fast, it doesn't go in the profile.

## Declaring a domain

```turtle
@prefix :      <https://example.org/triage#> .
@prefix sigil: <https://sigil.dev/ns#> .
@prefix sct:   <http://snomed.info/id/> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .

:Patient a rdfs:Class ;
    rdfs:label "Patient" .

:heartRate a rdf:Property ;
    rdfs:domain :Patient ;
    rdfs:range  xsd:decimal ;
    sigil:unit  "bpm" ;
    sigil:min   0 ;
    sigil:max   300 ;
    sigil:cardinality "0..1" .

:Symptom a rdfs:Class .
:ChestPain rdfs:subClassOf :Symptom ;
    owl:sameAs sct:29857009 .          # SNOMED CT anchor
:CrushingChestPain rdfs:subClassOf :ChestPain .

:hasSymptom a rdf:Property ;
    rdfs:domain :Patient ;
    rdfs:range  :Symptom ;
    sigil:cardinality "0..*" .

:AcuityLevel a rdfs:Class ;
    sigil:oneOf ( :Immediate :Urgent :Standard ) .   # closed enumeration
:acuity a rdf:Property ;
    rdfs:domain :Patient ; rdfs:range :AcuityLevel ;
    sigil:cardinality "0..1" ;
    sigil:assignedBy sigil:RulesOnly .   # perception may NEVER set this field
```

`sigil:assignedBy sigil:RulesOnly` is the property-level enforcement of "the LLM never
decides": the validation gate rejects any perception output that tries to set it.

## Constraint vocabulary

| Term | Applies to | Meaning |
|------|-----------|---------|
| `sigil:min` / `sigil:max` | numeric property | Inclusive bounds; gate rejects violations |
| `sigil:unit` | numeric property | Unit label; conversions are explicit, mismatches rejected **[DECIDE: auto-convert compatible units? → PRP-001]** |
| `sigil:cardinality` | property | `"1"`, `"0..1"`, `"1..*"`, `"0..*"` |
| `sigil:oneOf` | class | Closed instance enumeration (enums) |
| `sigil:pattern` | string property | Regex the value must match |
| `sigil:assignedBy` | property | `sigil:RulesOnly` \| `sigil:PerceptionAllowed` (default) \| `sigil:ToolsOnly` |
| `sigil:immutable` | property | Once set, cannot be re-asserted with a different value |

Constraints are enforced at two moments: at the **gate** (every assertion, doc 03) and
at **verification** (they bound the BDD variable domains, doc 10).

## Rule vocabulary

```turtle
:r1 a sigil:Rule ;
    sigil:priority 10 ;
    sigil:when [
        sigil:all (
            [ sigil:subject :Patient ; sigil:property :heartRate ; sigil:exceeds 120 ]
            [ sigil:subject :Patient ; sigil:property :hasSymptom ; sigil:is :ChestPain ]
        )
    ] ;
    sigil:then [ sigil:property :acuity ; sigil:value :Immediate ] ;
    sigil:explain "Tachycardia with chest pain — immediate track" .
```

Condition operators (the 11 inherited from sigil-lang, unchanged semantics):
`sigil:exceeds`, `sigil:below`, `sigil:atLeast`, `sigil:atMost`, `sigil:between`,
`sigil:is`, `sigil:isNot`, `sigil:isAssigned`, `sigil:isEmpty`, `sigil:contains`,
`sigil:inList`. Combinators: `sigil:all` (∧), `sigil:any` (∨), `sigil:none` (¬∨).

Two semantics notes (full detail in doc 04):
- `sigil:is :ChestPain` matches **any subclass** via subsumption — that is the point
  of having an ontology.
- Conditions over unassigned properties are false (`sigil:isNot` true) — Sigil's
  pragmatic closed-world stance, inherited and re-validated **[DECIDE: PRP-002]**.

## Workflow vocabulary

Summarized here, specified in doc 05:
`sigil:Workflow`, `sigil:startsAt`, `sigil:PerceptionStep`, `sigil:ReasoningStep`,
`sigil:ActionStep`, `sigil:NarrationStep`, `sigil:ProposalStep`, `sigil:next`,
`sigil:goal`, `sigil:requires`, `sigil:fallback`, `sigil:terminal`.

## Imports and anchoring

```turtle
@prefix imports: <https://sigil.dev/ns#> .
<> sigil:imports <./snomed-ct-subset.ttl> , <./fhir-vitals.ttl> .
```

- Imports load into named graphs; their class hierarchies participate in subsumption.
- Large vocabularies (SNOMED CT ≈ 350k concepts) are consumed as **extracted subsets**;
  `sigil author` assists extraction (doc 13). **[DECIDE: lazy loading — PRP-001]**
- `owl:sameAs` anchors local classes to external IRIs. Anchoring buys: shared meaning
  across deployments, imported hierarchy (subsumption you didn't write), and interop
  with existing RDF datasets and editors.

## Versioning

An ontology is code: it lives in git, changes by diff, and every released version
carries `sigil:version` and `sigil:profileVersion` (the RDFS-plus profile it targets).
The runtime refuses to load an ontology whose profile version it doesn't support.
Rendered domain-term diffs between versions are produced by the authoring tooling
(doc 13).
