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

## Source

See per-claim citations above; all also listed in `research/reading-log.md` under Q1.

## Confidence

measured

## Last-verified

2026-08-24
