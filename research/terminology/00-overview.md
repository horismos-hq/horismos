# Q5: terminology licensing and subsumption at scale — overview

## Claim

Horismos's approach — a hand-picked SNOMED subset compiled offline into a `.ttl`
import, reasoned over by horismos's own Rete-lite engine — is architecturally
*correct*, arguably the only pattern compatible with horismos's own hard rules
(`horismos-core` deliberately I/O-free during inference; "trace equality is a valid
test assertion" is incompatible with a live terminology-server call during reasoning).
But the currently-shipped fixture already shows the failure mode the pattern is
supposed to avoid: single-parent structure with no version provenance. See
`snomed-ct.md` for the detailed finding and recommendation.

## Evidence

See `snomed-ct.md` for the full analysis. `loinc.md` and
`rxnorm.md` cover the other two terminology systems horismos may need.

## Source

See `../reading-log.md`, Q5 section.

## Confidence

mixed — see linked pages.

## Last-verified

2026-08-24
