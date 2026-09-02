# SNOMED CT: licensing, subsumption at scale, and the current fixture

## Claim

The offline-compiled-subset pattern is correct and matches horismos's I/O-free,
deterministic inference constraints — but `examples/triage/snomed-ct-subset.ttl`
already shows the exact risk this research question asked about: single-parent
structure and no version provenance, on real SNOMED CT, which is a true polyhierarchy
with ~368,000 active concepts. The recommendation is to make extraction a build-time,
versioned, reproducible pipeline, not to change the architecture.

## Evidence

**[measured]** Horismos's pattern — compiling a closure once, offline, per release,
and serving it as static versioned data — is exactly what production terminology
servers do internally. SNOMED International's own vendor guidance calls infrequent-
update custom subsets "a consistent weakness of existing systems not using a
terminology server" while explicitly recommending a terminology server + ECL/ValueSets
over hand-rolled subsets
([docs.snomed.org](https://docs.snomed.org/implementation-guides/implementation-fact-sheets/implementation-dilemmas/do-i-need-an-external-interface-terminology-to-implement-snomed-ct)).
Horismos's compile-before-runtime pattern is structurally analogous to what that
guidance recommends — it just needs the compilation step built.

**[measured — verified directly against the repo]** `examples/triage/
snomed-ct-subset.ttl` today ships ~50 hand-picked concepts using single-parent
`rdfs:subClassOf` only. Real SNOMED CT is a true polyhierarchy (most concepts have
multiple `is-a` parents by design; ~368,000 active concepts as of the June 2024 figure
in SNOMED's own guidance) and the shipped fixture represents none of that structure.
Its own header comment admits this candidly: "the fixture's point is the structure,
not registry fidelity." That's a defensible statement about a demo fixture, but it
means the exact risk this research question asked about — "polyhierarchy edge cases
the ontology author's hand-picked subset might not represent correctly" — is not
hypothetical; it is visible in the shipped example today. There is also no
version/release provenance recorded anywhere in the fixture, and SNOMED CT
retires/replaces roughly a quarter of its concept space on an ongoing basis
([confluence.ihtsdotools.org](https://confluence.ihtsdotools.org/display/DOCEG/Concept+Inactivation)),
so a hand-copied subset silently drifts stale with no mechanism to detect it. (See
`docs/framework/02-ontology.md` ~line 132 for the resolved D5/PRP-001 decision to
consume vocabularies as pre-extracted subset imports, and
`docs/framework/13-authoring.md` for the planned, unbuilt `search_vocab`
SNOMED-anchoring tool that is PRP-007 scope.)

**[measured]** Chu et al. (JMIR Medical Informatics, 2019) directly quantify how much
a hand-curated ("extensional") subset loses relative to a proper subsumption-based
("intensional") one: extensional value sets captured a median of only 65% of
clinically relevant SNOMED terms that an equivalent ECL-based hierarchy definition
would capture — as low as 2% for hypertension specifically — while being far less
concise (78 vs. 3 concepts, median) and slower to author (37 vs. 5 minutes, median)
([PMC6351992](https://pmc.ncbi.nlm.nih.gov/articles/PMC6351992/)). This is, concretely,
the risk horismos's own reasoning engine is exposed to today through its shipped
fixture, on a scenario horismos itself uses as the flagship example.

**[measured]** ELK classifies the full SNOMED CT ontology in ~5 seconds, versus ~20
minutes for CEL, per a comparison of OWL 2 EL reasoners `[unverified — secondary
summary only]`
([semantic-web-journal.net](https://www.semantic-web-journal.net/sites/default/files/swj120_2.pdf)) —
relevant to whether a build-time full-classification pipeline is computationally
feasible.

**[inference — recommendation]** Don't change the offline-compiled-subset
architecture — it's correct. Do treat extraction as a build-time, versioned,
reproducible pipeline (an ECL expression plus a pinned SNOMED edition/version as the
actual source of truth, ideally backed by a self-hostable terminology server such as
Snowstorm), preserve full multi-parent polyhierarchy in the generated output rather
than collapsing to one parent, and stamp provenance (source edition/version +
generating ECL expression) into the subset file so staleness is mechanically
detectable. This is exactly `search_vocab`/PRP-007's stated scope — the finding is
that its output format needs to support polyhierarchy and provenance metadata, not
just an `owl:sameAs` anchor. This recommendation is a captain-held decision, not
applied here — see `../ontology-authoring/prp-007-open-items.md`.

**[measured]** Licensing: distributing SNOMED CT content is a real deploying-org
obligation. Whoever ships a horismos ontology bundle containing real SNOMED CT content
to third parties is themselves distributing SNOMED CT and needs their own SNOMED
International Affiliate status with sub-licensing obligations to *their* downstream
users
([docs.snomed.org licensing](https://docs.snomed.org/snomed-ct-practical-guides/vendor-introduction-to-snomed-ct/7-licensing)).
SNOMED CT is free for Member countries, with fees by territory outside membership
([snomed.org/get-snomed](https://www.snomed.org/get-snomed)); the NLM distributes it
free to US/international users via UMLS license
([nlm.nih.gov](https://www.nlm.nih.gov/healthit/snomedct/faq.html)). **[inference]**
This is a deploying-org obligation horismos-the-framework cannot discharge on anyone's
behalf — worth a line in `docs/framework/02-ontology.md` or an ops guide before this
becomes someone's surprise.

**[measured]** SNOMED's own maps to ICD-10 are documented as one-directional and lossy
([docs.snomed.org](https://docs.snomed.org/snomed-ct-specifications/snomed-ct-to-icd-10-map-specification/introduction/2-icd-10-map-overview)).
SNOMED International's own open-source terminology server is Snowstorm
([github.com/IHTSDO/snowstorm](https://github.com/IHTSDO/snowstorm)); CSIRO's
Ontoserver decouples implementers from raw RF2 and enables frequent version updates
([ontoserver-wp.csiro.au](https://ontoserver-wp.csiro.au/faqs/); peer-reviewed
description: [pubmed.ncbi.nlm.nih.gov/30223897](https://pubmed.ncbi.nlm.nih.gov/30223897/)).
ECL (Expression Constraint Language) is the formal language for intensional subset
definitions
([docs.snomed.org](https://docs.snomed.org/snomed-ct-specifications/snomed-ct-expression-constraint-language/introduction/1-introduction)).

## Sweep #2 delta (2026-09-02): release-cadence and format changes

**[measured]** SNOMED International's July 2026 International Edition notes (release
date 20260701): "The only change, then, for this July 2026 International release, was
to transition the International Edition to the new Description Length Limit" — 4096
characters, up from 255 — with "NO DESCRIPTIONS … EXPANDED BEYOND 255 CHARACTERS FOR THE
JULY 2026" release and the first expansions "currently planned for … the August 2026
International release"; the August notes add that future description changes "will
align with the updated specification of description length up to 4096 characters
without early visibility of these changes." Concept Non Current indicators "were
deprecated in the January 2026 International Edition" and will be "removed from both
Full and Snapshot files … from the January 2027 release onwards." The notes reference
"new monthly editing cycles"
([July 2026 notes](https://conf.spaces.snomed.org/wiki/spaces/RMT/pages/972029981/);
[August 2026 notes](https://conf.spaces.snomed.org/wiki/spaces/RMT/pages/1065124453/)).
The US Edition remains on a March/September cycle (NLM page, snippet → [unverified]).
No SNOMED licensing change was found in 2026.

**[inference]** Two consequences for the build-time extraction pipeline this page
recommends: provenance must carry edition *and* release month — the "same" SNOMED now
differs monthly (International) versus semiannually (US) — and any fixed 255-character
description field in an RF2 or FHIR ingest would now fail closed on legitimate data.

## Source

See `../reading-log.md`, Q5 section.

## Confidence

mixed — the fixture's current state and licensing rules are [measured]; the
extraction-pipeline recommendation is [inference] and captain-held (PRP-007 scope).

## Last-verified

2026-09-02 (sweep #2 delta appended; founding claims unchanged)
