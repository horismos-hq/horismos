# Q1: guideline representation languages & standards — overview

## Claim

Horismos's step graph is, structurally, a **Task-Network Model (TNM)** — the same
family as GLIF/GLIF3, PROforma, Asbru, and EON. This is the field's converged-upon
answer to "a guideline is a graph of typed steps," reached independently by at least
five research groups from 1995–2010, and none of those TNM formalisms reached
sustained production, multi-site use. By contrast, the formalisms that reached real,
regulatory-mandated, national scale (CQL, CDS Hooks, FHIR PlanDefinition) won by being
thin, composable layers on top of FHIR and the already-dominant vendor CIS, not by
owning the whole domain+rules+workflow stack as one artifact.

## Evidence

**[measured]** Horismos's `PerceptionStep`/`ReasoningStep`/`ActionStep`/
`NarrationStep`/`ProposalStep`/`ChoiceStep`/`WaitStep`/`SubflowStep` graph matches the
TNM shape (Action/Conditional/Branch/Synchronization in GLIF3; Action/Decision/
Enquiry/Plan in PROforma). See
[PMC6172935](https://pmc.ncbi.nlm.nih.gov/articles/PMC6172935/) for the family
classification.

**[measured]** GLIF's own founding pilot study found that even within one
collaboration, two encoders of the same guideline produced non-interchangeable
representations, because of an "encoding-variability" and "vocabulary problem" its own
authors named explicitly
([PMC61313](https://pmc.ncbi.nlm.nih.gov/articles/PMC61313/)). See
`formalization/glif.md`.

**[measured]** EON's clinical instantiation (ATHENA-HTN, a real multi-site VA RCT)
found decision support alone produced no significant difference in blood-pressure
control versus a reminder-only control arm
([PMC2243943](https://pmc.ncbi.nlm.nih.gov/articles/PMC2243943/)). SAGE (5-year,
NIST-funded, six-institution) has no evidence of continuation after funding ended
around 2006. GLARE is self-described by its own authors as a "domain-independent
prototypical system." See `formalization/tnm-family.md`.

**[measured]** CQL (CMS-mandated quality reporting), CDS Hooks (broad EHR-vendor
support: Epic, Cerner, Allscripts, athenahealth), and FHIR PlanDefinition/
ActivityDefinition (the Da Vinci prior-authorization substrate) won specifically by
being thin, composable layers on top of FHIR and the already-dominant vendor CIS. See
`formalization/cql-cds-hooks-fhir.md`.

**[inference — the single most important comparative judgment in the founding
sweep]** Horismos's core bet — one ontology owning domain, rules, and workflow, with
its own World/gate/trace substrate rather than delegating to FHIR resources and vendor
CIS integration — structurally resembles the losing side of a 20-year natural
experiment the field has already run, even though its internal engineering (truth
maintenance, a uniform validation gate, formal pre-execution verification) is more
rigorous than any predecessor's. This is not a reason to abandon the design (three
horismos mechanisms have no precedent in the CIG literature — see below), but it is a
reason to expect "how does this integrate with an existing EHR/CIS" to be the first
hard question any credible reviewer asks.

### What's genuinely novel (no CIG precedent)

**[measured]** Three horismos mechanisms have no precedent in the surveyed CIG
formalism literature, each targeting a documented, named historical failure:

- **`PerceptionStep`** answers a gap GLIF's own authors explicitly flagged as unsolved
  in the 1990s — they left eligibility criteria as unstructured narrative text because
  "we do not have natural-language processors that can parse and make sense out of
  such clauses" ([PMC61313](https://pmc.ncbi.nlm.nih.gov/articles/PMC61313/)).
- **The uniform validation gate** (every mutation — perception, rule, tool, API —
  passes the same gate) has no analog anywhere surveyed. Arden Syntax's curly braces
  are the literal opposite: an unguarded, per-institution bypass around standard rule
  logic, and that bypass is exactly what defeated Arden's portability
  ([Samwald et al. 2012](https://pubmed.ncbi.nlm.nih.gov/22342733/)). See
  `formalization/arden-syntax.md`.
- **Truth maintenance with fixed-point recomputation on every transaction** has no CIG
  precedent at all — it directly answers the field's best-documented empirical CDS
  failure mode: Wright et al.'s taxonomy of 68 alert malfunctions across 14 US sites
  ([JAMIA](https://dx.doi.org/10.1093/jamia/ocx106)), and an OHSU/Epic study finding 4
  confirmed malfunctions none of which had ever been detected before the retrospective
  study ([PMC6220702](https://pmc.ncbi.nlm.nih.gov/articles/PMC6220702/)).

Formal (BDD-style) pre-execution verification of a rule set separately revives a
research line — model checking of Asbru guidelines with a purpose-built temporal logic
([spinroot.com](https://spinroot.com/spin/symposia/ws06/027.pdf)) — that stayed in
small academic case studies and never reached production tooling anywhere; it inherits
that line's open question of scaling past small case studies.

### Five named risks the field's history predicts

**[inference, synthesized from converging evidence]**, in order of how directly they
threaten horismos:

1. **Authoring/knowledge-acquisition burden at real breadth.** A 2024 multimorbidity
   project needed 296 hand-authored CDS rules across just 25 source guidelines
   ([PMC11163046](https://pmc.ncbi.nlm.nih.gov/articles/PMC11163046/)). Horismos has
   exactly one hand-authored workflow; the jump to N scenarios at institutional
   breadth is precisely where SAGE, GLIF, and Asbru stalled.
2. **Maintenance/versioning as source guidelines evolve.** The LASSIE case study found
   only 133 of 515 sentences in one guideline update were inheritable unchanged
   ([PMC2859225](https://pmc.ncbi.nlm.nih.gov/articles/PMC2859225/)). Horismos's
   "formally reverify before running" model has no described incremental/diffing
   reverification workflow yet.
3. **Escalation/fallback fatigue is a separate design problem formal correctness does
   not solve.** DDI alert override rates run 46–96%
   ([journals.sagepub.com](https://journals.sagepub.com/doi/10.1177/14604582241263242)),
   and Kawamoto et al.'s foundational 70-RCT review found only four features predict
   CDS success — none of them "logical correctness"
   ([Kawamoto 2005](https://pubmed.ncbi.nlm.nih.gov/15767266/)).
4. **No demonstrated EHR/vendor-CIS integration story** — the single biggest
   structural gap between horismos's current design and how the field's winners
   actually won.
5. **Trust/clinician-acceptance barriers are largely orthogonal to representation
   quality.** No amount of formal verification addresses this class of barrier.

## Source

See `research/reading-log.md`, Q1 section, for the full citation list.

## Confidence

Mixed — the TNM-family classification and the individual case outcomes are
[measured]; the comparative judgment about horismos's structural position is
[inference], flagged as such in the founding sweep.

## Last-verified

2026-08-24
