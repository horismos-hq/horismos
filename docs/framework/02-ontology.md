# 02 · The Ontology (RDFS-plus)

The ontology is the single authored artifact of a Horismos deployment. It is standard
Turtle (`.ttl`), readable by any RDF tool, extended with the `horis:` vocabulary that
makes it *executable*.

## The RDFS-plus profile

Horismos deliberately takes a small, tractable slice of the semantic-web stack:

**Included (base):**
- RDF triples; Turtle syntax in/out; named graphs for imports.
- RDFS: `rdfs:Class`, `rdfs:subClassOf`, `rdf:Property`, `rdfs:domain`, `rdfs:range`,
  `rdfs:label`, `rdfs:comment`.
- Datatypes: `xsd:decimal`, `xsd:integer`, `xsd:string`, `xsd:boolean`,
  `xsd:dateTime`, `xsd:duration`.
- `owl:sameAs` for anchoring to external vocabularies (that is the *only* OWL term).

**Included (the "plus" — `horis:` namespace, `https://w3id.org/horismos#`):**
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
@prefix horis: <https://w3id.org/horismos#> .
@prefix sct:   <http://snomed.info/id/> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .

:Patient a rdfs:Class ;
    rdfs:label "Patient" .

:heartRate a rdf:Property ;
    rdfs:domain :Patient ;
    rdfs:range  xsd:decimal ;
    horis:unit  "bpm" ;
    horis:min   0 ;
    horis:max   300 ;
    horis:cardinality "0..1" .

:Symptom a rdfs:Class .
:ChestPain rdfs:subClassOf :Symptom ;
    owl:sameAs sct:29857009 .          # SNOMED CT anchor
:CrushingChestPain rdfs:subClassOf :ChestPain .

:hasSymptom a rdf:Property ;
    rdfs:domain :Patient ;
    rdfs:range  :Symptom ;
    horis:cardinality "0..*" .

:AcuityLevel a rdfs:Class ;
    horis:oneOf ( :Immediate :Urgent :Standard ) .   # closed enumeration
:acuity a rdf:Property ;
    rdfs:domain :Patient ; rdfs:range :AcuityLevel ;
    horis:cardinality "0..1" ;
    horis:assignedBy horis:RulesOnly .   # perception may NEVER set this field
```

`horis:assignedBy horis:RulesOnly` is the property-level enforcement of "the LLM never
decides": the validation gate rejects any perception output that tries to set it.

## Constraint vocabulary

| Term | Applies to | Meaning |
|------|-----------|---------|
| `horis:min` / `horis:max` | numeric property | Inclusive bounds; gate rejects violations |
| `horis:unit` | numeric property | Opaque unit label; exact match required between declarations and rule conditions — mismatches are load-time verification errors. No auto-conversion (decided, PRP-001) |
| `horis:cardinality` | property | `"1"`, `"0..1"`, `"1..*"`, `"0..*"` |
| `horis:oneOf` | class | Closed instance enumeration (enums) |
| `horis:pattern` | string property | Regex the value must match |
| `horis:assignedBy` | property | `horis:RulesOnly` \| `horis:PerceptionAllowed` (default) \| `horis:ToolsOnly` |
| `horis:immutable` | property | Once set, cannot be re-asserted with a different value |

`horis:immutable` binds assertion **and retraction** (decided, PRP-002): once set,
an immutable fact can be neither re-asserted with a different value nor retracted —
the two-step retract-then-assert bypass is closed at the gate, from every
provenance.

Constraints are enforced at two moments: at the **gate** (every assertion, doc 03) and
at **verification** (they bound the BDD variable domains, doc 10).

## Rule vocabulary

```turtle
:r1 a horis:Rule ;
    horis:priority 10 ;
    horis:when [
        horis:all (
            [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 120 ]
            [ horis:subject :Patient ; horis:property :hasSymptom ; horis:is :ChestPain ]
        )
    ] ;
    horis:then [ horis:property :acuity ; horis:value :Immediate ] ;
    horis:explain "Tachycardia with chest pain — immediate track" .
```

Condition operators (the 11 inherited from sigil-lang, unchanged semantics):
`horis:exceeds`, `horis:below`, `horis:atLeast`, `horis:atMost`, `horis:between`,
`horis:is`, `horis:isNot`, `horis:isAssigned`, `horis:isEmpty`, `horis:contains`,
`horis:inList`. Combinators: `horis:all` (∧), `horis:any` (∨), `horis:none` (¬∨).

Two semantics notes (full detail in doc 04):
- `horis:is :ChestPain` matches **any subclass** via subsumption — that is the point
  of having an ontology.
- Conditions over unassigned properties are false (`horis:isNot` true) — Horismos's
  pragmatic closed-world stance, inherited from sigil-lang and re-validated for the
  graph world (decided, PRP-002): `isNot` is the ∀ negation of `is`, including over
  multi-valued properties (doc 04's table).

## Workflow vocabulary

Summarized here, specified in doc 05:
`horis:Workflow`, `horis:startsAt`, `horis:PerceptionStep`, `horis:ReasoningStep`,
`horis:ActionStep`, `horis:NarrationStep`, `horis:ProposalStep`, `horis:next`,
`horis:goal`, `horis:requires`, `horis:fallback`, `horis:terminal`.

## Standard-artifact anchoring

The pivot to a bounded projection kernel keeps this profile exactly as specified above —
no new `horis:` terms are minted for external terminology anchoring itself, and the
governing design questions are resolved inside PRP-008, per this project's rule that
vocabulary is never invented ad hoc. What changes is the
*honesty* of the anchoring story: `owl:sameAs` alone is necessary but not sufficient for
real interoperability. A single-parent, unversioned anchor (`:ChestPain owl:sameAs
sct:29857009`, no edition) loses SNOMED CT's polyhierarchy and cannot express "which
release this anchor was checked against" — the current triage fixture demonstrates exactly
this gap: its SNOMED subset is ~50 concepts, single-parent only, with no edition
recorded. PRP-008 extends anchoring with:

- **versioned mappings** — which standard-vocabulary edition/version an anchor was
  verified against, not just the bare IRI;
- **polyhierarchy-preserving anchors** — more than one `owl:sameAs`-style edge where the
  external system itself is polyhierarchical, rather than collapsing to one parent;
- **profile identity** — which FHIR profile, PlanDefinition version, or tool-schema
  version a term or mapping targets;
- **compiled mappings** — FHIR JSON/RDF resources are mapped to ontology facts by
  deterministic adapter code, compiled/validated before runtime, never inferred ad hoc by
  an LLM (the perception/narration boundary discipline of doc 06 extends unchanged to
  standards mapping: it is not a transducer boundary, it is ordinary deterministic code).

None of this weakens the "compatible with standards" claim below into "the ontology is
FHIR" — Horismos's rule, workflow, provenance, and gate semantics remain Horismos's own;
standards supply domain identity and terminology, not Horismos's execution model.

## Candidate-artifact runtime evidence

PRP-008 mints only the runtime half of the joint PRP-007/008 boundary. The authoring
envelope remains PRP-007's vocabulary; the runtime consumes it as opaque typed data and
emits these evidence terms:

| Term | Meaning |
|------|---------|
| `horis:CandidatePolicyEvidence` | Deterministic path/base-hash/syntax/permission result |
| `horis:SandboxEvidence` | Ordered results of the proposal's declared checks |
| `horis:AuthorizationEvidence` | Symbolic authorization result over the two prior stages |
| `horis:policyAccepted` | Boolean policy verdict |
| `horis:sandboxPassed` | Boolean declared-check verdict |
| `horis:authorizationGranted` | Boolean authorization verdict |
| `horis:evidenceFor` | Proposal identifier the evidence concerns |

These terms prove policy legality and declared-check outcomes only. They never state that
generated content is semantically correct.

## Imports and anchoring

```turtle
@prefix imports: <https://w3id.org/horismos#> .
<> horis:imports <./snomed-ct-subset.ttl> , <./fhir-vitals.ttl> .
```

- Imports load into named graphs; their class hierarchies participate in subsumption.
- Large vocabularies (SNOMED CT ≈ 350k concepts) are consumed as **extracted subsets**
  — a subset `.ttl` is an ordinary import; there is no lazy loading, and the core does
  no I/O beyond reading declared imports (decided, PRP-001). `horismos author` assists
  extraction (doc 13).
- An import may only name a file under the root document's own directory. Ontology
  loading is a trust boundary (doc 15): the ontology is the program, so a document
  does not get to decide by its own text which files on the host join it. A deployment
  that shares an ontology directory across root documents declares those directories
  explicitly at load. A `file:` IRI naming a host is not a local path and is refused.
- A transitive import chain is bounded, as is `horis:all`/`any`/`none` nesting inside
  a `horis:when`. Both caps sit far above anything readable — no shipped example
  declares an import chain at all, and the deepest condition body across them is four
  levels — and a document past either is refused with the depth named.
- `owl:sameAs` anchors local classes to external IRIs. Anchoring buys: shared meaning
  across deployments, imported hierarchy (subsumption you didn't write), and interop
  with existing RDF datasets and editors.

## Versioning

An ontology is code: it lives in git, changes by diff, and every released version
carries `horis:version` and `horis:profileVersion` (the RDFS-plus profile it targets).
The runtime refuses to load an ontology whose profile version it doesn't support.
Rendered domain-term diffs between versions are produced by the authoring tooling
(doc 13).
