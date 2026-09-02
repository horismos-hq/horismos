# CQL / CDS Hooks / FHIR PlanDefinition

## Claim

CQL, CDS Hooks, and FHIR PlanDefinition/ActivityDefinition are the formalisms that
actually reached real, regulatory-mandated, national scale — and they won by being
thin, composable layers on top of FHIR and the already-dominant vendor CIS, not by
trying to own the whole domain+rules+workflow stack as one artifact the way GLIF,
Asbru, EON, and SAGE did.

## Evidence

**[measured]** CDS Hooks has broad EHR-vendor support (Epic, Cerner, Allscripts,
athenahealth); spec v2.0.1 STU2 with a normative ballot planned for 2025
([cds-hooks.hl7.org](https://cds-hooks.hl7.org/); implementer list:
https://www.trisotech.com/cds-hooks/).

**[measured]** The FHIR Clinical Reasoning Module (PlanDefinition/ActivityDefinition)
is targeting FHIR R6 normative and is the substrate for HL7's Da Vinci
prior-authorization implementation guides
([hl7.org/fhir/clinicalreasoning-module.html](https://hl7.org/fhir/clinicalreasoning-module.html)).

**[measured]** A 2025 assessment of 3 standards-based CDS tools (CQL, CDS Hooks, FHIR)
found 96–100% accuracy; the real barrier was EHR data-documentation gaps, not the
representation language itself
([PMC12309839](https://pmc.ncbi.nlm.nih.gov/articles/PMC12309839/)).

**[measured]** The 2024 CAREPATH study needed 296 hand-authored CDS rules / 326 CDS
Hooks cards from just 25 source guidelines, requiring a bespoke co-creation
methodology
([PMC11163046](https://pmc.ncbi.nlm.nih.gov/articles/PMC11163046/)) — evidence that
even the "winning" formalisms don't eliminate authoring burden, they just don't
require owning the whole stack to get adopted.

**[measured]** BPM+ Health / the HL7 BPM Community of Practice represents a related
OMG-to-HL7 transition reusing generic BPMN/DMN/CMMN rather than a bespoke clinical
formalism ([bpm-plus.org](https://www.bpm-plus.org/)).

## Sweep #2 delta (2026-09-02): the host surface in motion

**[measured] Production scale.** Epic's Coverage Requirements Discovery API (HL7 FHIR
Da Vinci CRD) went live 2026-08-17 at Ochsner, Froedtert ThedaCare, Denver Health and
Summit Health with UnitedHealthcare, Aetna and Network Health, "16 additional health
insurers undergoing sandbox integration testing," surfacing requirements "at the exact
moment a clinician enters an order," "more than five months ahead of" the CMS 2027-01-01
deadline ([Epic newsroom](https://www.epic.com/epic/newsroom/);
[HIT Consultant](https://hitconsultant.net/2026/08/17/epic-deploys-real-time-crd-prior-authorization-apis-ochsner-froedtert/);
[TechTarget](https://www.techtarget.com/revcyclemanagement/news/366649207/Health-systems-test-real-time-prior-auths-through-Epic)).
The coverage does not name CDS Hooks; that Da Vinci CRD is a CDS Hooks profile is the
sweep's knowledge of the IG, not a statement in those pieces.

**[measured] CDS Hooks 3.0.0.** The `HL7/cds-hooks` publication request reads
`"version": "3.0.0-ballot"`, `"desc": "R3 Normative Ballot"`, `fhirVersion: 4.0.1`,
status `ballot`; `changes.md`: "The 3.0 release is a largely normative release … Not
all of CDS Hooks 3.0 is Normative." Breaking: "Use SMART App Launch STU 2, replacing use
of SMART App Launch STU 1 … this impacts the contents of the `scope` element, within the
optional `fhirAuthorization` object" (FHIR-37609). New: prefetch tokens may contain "a
limited subset of FHIRPath" (FHIR-35999), `suggestion.actionSelectionBehavior`, an
example override-reason ValueSet; recommended: errors as OperationOutcome, JWKS key
rotation, semver `version` and `hookVersion` in discovery. Reconciliation merges ran
through 2026-06-24; the May 2026 publication target ("Earliest publication timeframe is
May," Rhodes, ASTP/ONC 2026 Annual Meeting deck) was missed and 2.0.1 remains current
as of 2026-09-02; the ballot close "comments due by September 21" is from a search
snippet → [unverified] ([build.fhir.org/ig/HL7/cds-hooks](https://build.fhir.org/ig/HL7/cds-hooks/);
[Rhodes deck](https://healthit.gov/wp-content/uploads/2025/09/2026AnnualMeeting_Beyond-Traditional-APIs-The-Future-of-Health-Data-Interoperability.pdf)).
No AI/LLM-specific hook, card guidance, or agent language appears anywhere in the
changelog or hook catalog.

**[measured] FHIR R6, second full Normative ballot (v6.0.0-ballot5).** "ALL of the
specification is being balloted as a full Normative ANSI Standard"; "no draft or
trial-use content remains"; ~35 resources moved to incubator IGs, including
StructureMap, GraphDefinition, Contract, TestPlan, Transport; "hoping that we won't
require additional full ballots." PlanDefinition, ActivityDefinition and
RequestOrchestration are Standards Status Normative; the Clinical Reasoning module
records "All Changes: None" this cycle; the CDS-on-FHIR page still describes "$apply
against the specified PlanDefinition, and transforming the resulting
RequestOrchestration into a CDS Hooks response." The FHIR Mapping Language is "not
ready for normative (moved to mapping-language IG with StructureMap)"
([ballot intro](https://hl7.org/fhir/6.0.0-ballot5/ballot-intro.html);
[plandefinition](https://build.fhir.org/plandefinition.html);
[requestorchestration](https://build.fhir.org/requestorchestration.html)). FHIR
RDF/Turtle, ShEx and the OWL ontology become Normative "except for modifier
extensions," omitted "because modifier extensions could introduce non-monotonicity"
([rdf.html](https://build.fhir.org/rdf.html);
[W3C HCLS #176](https://github.com/w3c-cg/hcls-fhir-rdf/issues/176)).

**[measured] Version-aware update.** R6 HTTP (Normative): "If the version id given in
the `If-Match` header does not match, the server returns a `412 Precondition Failed`";
servers "SHOULD provide full version support"; "Servers can require that clients
provide an `If-Match` header by returning `400 Bad Request`"; transactions "SHALL
either accept all actions … or reject all resources," and overlapping identities
"SHALL fail" ([http.html](https://build.fhir.org/http.html)).
`Bundle.entry.request.ifMatch`: "Only perform the operation if the Etag value matches"
([bundle-definitions](https://build.fhir.org/bundle-definitions.html)).
`CapabilityStatement.rest.resource.versioning` ∈ `no-version | versioned |
versioned-update`, the last meaning "using e-tags for version integrity in the API"
([capabilitystatement-definitions](https://build.fhir.org/capabilitystatement-definitions.html)).
Oracle Health Millennium R4 makes `If-Match` required on update, with `W/""` ETags
([docs.oracle.com](https://docs.oracle.com/en/industries/health/millennium-platform-apis/mfrap/op-condition-id-put.html));
its 412-when-missing / 409-on-mismatch codes are from a snippet → [unverified]. No
server-support survey of `If-Match`/`versioned-update` exists; Epic's update specs are
login-walled.

**[measured] CQL.** Release 2 published as STU 2026-08-12: "introduces substantive new
features … and no breaking changes"; v1.5.3 "was reaffirmed as an ANSI standard in
September 2026"; R2 adds directives (`#DefaultComparisonPrecision`), model qualifiers
(`using … called USC`), and in-language `define type / context / conversion`
([standups.hl7.org](https://standups.hl7.org/2026/08/12/stu-publication-of-hl7-specification-clinical-quality-language-cql-release-2/);
[cql.hl7.org](https://cql.hl7.org/)). Common CQL Artifacts for FHIR Ed. 2 published
Informative 2026-08-21 (R4). The Using-CQL-with-FHIR IG's STU3 ballot adds
`CQLGroupDefinition` and `$evaluate` (FHIR-56396), `evaluatedResource` references in
results (FHIR-57261), and "how to interpret a data requirement in terms of a FHIR API
request" (FHIR-56804). Engines: cqframework 5.0.0 (2026-06-30) represents all values as
a sealed `Value` interface, 5.2.0 (2026-08-19) adds a Kotlin-multiplatform FHIR JSON →
CQL parser and UCUM units; Firely CQL SDK 2.9.2 → 2.14.0 (2026-07-08 → 2026-08-20) with
exact opaque version matching, "closed-world semantics when a value set is resolved
locally," and frozen expansions — both R4
([cqframework releases](https://github.com/cqframework/clinical_quality_language/releases);
[Firely releases](https://github.com/FirelyTeam/firely-cql-sdk/releases)).

**[measured] AI Transparency on FHIR IG** (`hl7.fhir.uv.aitransparency` 1.0.0-ballot,
FHIR 4.0.1, reconciliation commits to 2026-08-20): "AI algorithm is defined broadly to
include any computer-based logic that touches health data … This IG does not make a
strict distinction between clearly AI, and non-AI algorithms"; assertions "are
declarative statements made by the producing system"; per the 2026-08-06 Agents-on-FHIR
notes, "normative content largely unchanged from the ballot," Zulip "almost universally
for making the label required," cascading models out of scope
([Health Samurai notes](https://www.health-samurai.io/events/agents-on-fhir/2026-08-06-meeting)).
**[measured] Bulk Data STU4 ballot** adds Bulk Submit/Publish and migrates the async
pattern into the IG; kick-off is POST-only; no version preconditions
([build.fhir.org/ig/HL7/bulk-data](https://build.fhir.org/ig/HL7/bulk-data/)).

**[inference] Bearing on PRP-008's D-items (evidence, not resolution):** D1 — the host
surface is R4 (CDS Hooks 3.0, both CQL engines, the transparency IG) while the
PlanDefinition shape is about to freeze normative in R6, so the baseline is really "R4
wire + R6-shaped PlanDefinition import"; D2 — StructureMap/FML lost core standing while
FHIR RDF/OWL/ShEx became normative, favouring an RDF bridge with modifier-extension
rejection; D3 — probe `versioning == versioned-update` and transaction support per
resource type at adapter init and treat 409/412/weak ETags as equivalent drift; D4/D5 —
prefer prefetch-declared closures while `fhirAuthorization` sits at STU; D7 — compose
(release velocity settles it); D9 — the transparency label plus a Provenance pointing at
the proof is the natural trace/commit carrier. Full argument:
`../neurosymbolic-landscape/pivot-alignment.md` §X7, A1–A7. Also published: "Field Guide
to Sharable Clinical Pathways, Edition 3" (Informative, 2026-08-27) — title and date
only retrievable, unread.

## Source

See per-claim citations above; all also listed in `../reading-log.md` under Q1.

## Confidence

measured

## Last-verified

2026-09-02 (sweep #2 delta appended; founding claims unchanged)
