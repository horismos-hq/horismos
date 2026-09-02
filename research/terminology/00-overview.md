# Q5: terminology licensing and subsumption at scale — overview

## Claim

**[inference]** Horismos's approach — a hand-picked SNOMED subset compiled offline into a
`.ttl` import, reasoned over by horismos's own Rete-lite engine — is architecturally
*correct*, arguably the only pattern compatible with horismos's own hard rules
(`horismos-core` deliberately I/O-free during inference; "trace equality is a valid
test assertion" is incompatible with a live terminology-server call during reasoning).
**[measured — verified directly against the repo]** But the currently-shipped fixture
already shows the failure mode the pattern is supposed to avoid: single-parent structure
with no version provenance. See `snomed-ct.md` for the detailed finding and
recommendation.

## Evidence

**[measured — local repo, `examples/triage/snomed-ct-subset.ttl`, re-verified
2026-09-02]** The fixture declares 50 `rdfs:subClassOf` links, no concept carries more
than one parent, and the file records no SNOMED release, edition, or version anywhere;
its own header says "the fixture's point is the structure, not registry fidelity." The
hard rules the pattern must stay compatible with are stated in `CLAUDE.md` § Hard rules
(anti-wrapper boundary, determinism) and `docs/framework/14-architecture.md`.

**[measured]** SNOMED International's own vendor guidance calls infrequently-updated
custom subsets "a consistent weakness of existing systems not using a terminology
server" and recommends a terminology server plus ECL/ValueSets over hand-rolled subsets
([docs.snomed.org](https://docs.snomed.org/implementation-guides/implementation-fact-sheets/implementation-dilemmas/do-i-need-an-external-interface-terminology-to-implement-snomed-ct)).

**[inference]** Those two facts together are why the recommendation in `snomed-ct.md` is
to make extraction a build-time, versioned, reproducible pipeline — not to change the
architecture. Under the bounded-projection pivot this is now owned work: PRP-008's
"Terminology provenance" scope item and its D8 (terminology service / build-time
expansion policy) — see `PRPs/PRP-008-standards-host-integration.md`.

See `snomed-ct.md` for the full analysis. `loinc.md` and `rxnorm.md` cover the other two
terminology systems horismos may need.

## Sweep #2 delta (2026-09-02): how the field pins terminology versions

**[measured]** The 2027 eCQM Annual Update pins a full code-system vector — "SNOMED CT
US Edition 2025-09", "LOINC 2.81", "RxNorm 2026-01", "ICD-10-CM 2026", "CPT 2026" — with
logic in "CQL Specification v1.5.3" and "QDM v5.6" (not CQL R2), and VSAC posting the
final code-system/value-set list "in Spring 2026"
([2027 AU pre-publication PDF](https://ecqi.healthit.gov/sites/default/files/2027-AU-PrePublication-Document.pdf)).
VSAC's FHIR API serves a frozen expansion by release label, e.g.
`$expand?expansion=eCQM Update 2025-05-08`
([nlm.nih.gov/vsac](https://www.nlm.nih.gov/vsac/support/usingvsac/vsacfhirapi.html)).
FHIR R6's `$expand` defines `system-version`, `check-system-version` ("If a value set
specifies a different version, an error is returned") and `force-system-version`, says
servers "SHOULD ensure that all the parameters that affect the contents of the
expansion are recorded in the ValueSet.expansion.parameter list," and calls expansions
"a transient result that will change over time"
([valueset-operation-expand](https://build.fhir.org/valueset-operation-expand.html)).
LOINC 2.82 (2026-02-24, snippet → [unverified]) is out while the 2027 program pins 2.81.
Firely CQL SDK 2.14.0 resolves local value sets with closed-world semantics and frozen
expansions ([releases](https://github.com/FirelyTeam/firely-cql-sdk/releases)).

**[inference]** National practice is a per-release version vector plus labeled,
frozen expansions that deliberately lag "latest" — the shape PRP-008's terminology-
provenance scope item and D8 describe. `check-system-version` is the fail-closed knob
for hydration and `expansion.parameter` the natural carrier for pinned versions in the
trace (`../neurosymbolic-landscape/pivot-alignment.md`, A7). SNOMED-specific release
changes: `snomed-ct.md` (sweep #2 delta).

## Source

See `../reading-log.md`, Q5 section; repo facts cite `local repo` paths inline above.

## Confidence

mixed — the fixture facts and SNOMED guidance are [measured]; the "architecturally
correct" judgment and the pipeline recommendation are [inference].

## Last-verified

2026-09-02 (repo facts re-checked against current HEAD; sweep #2 tag fix — the page
previously carried no evidence tags, deferred from PR #9 per `../CHANGELOG.md`; sweep #2
delta appended)
previously carried no evidence tags, deferred from PR #9 per `../CHANGELOG.md`)
