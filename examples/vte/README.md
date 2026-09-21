# VTE prophylaxis vertical slice

This fixture is the PRP-008 bounded-projection prototype. `recorded-bundle.json` is a
zero-network FHIR R4 input whose Patient and four Observations are explicitly bound to the
same patient. `manifest.json` pins real SHA-256 digests for the ontology, local
`StructureDefinition`, owned mapping, bounded PlanDefinition, offline terminology expansion,
and their composite package. The six-member closure binds its Patient member from the request's
`context.patientId`, so the recorded states and ETags must describe that patient; the platelet member
is deliberately `verified-absent`, exposing one benign closure-completion cost rather than
confusing it with `not-fetched` or `not-authorized`.

The prototype validates the local R4 profile's Bundle constraint and its narrower VTE resource
requirements (resource identities, versions, patient references, final boolean Observations)
before hydration. It does not claim external FHIR certification.

The rule set assigns `:recommendation` for every input region, so the ontology passes
verification without the `allow_incomplete` escape. The third value,
`:NoProphylaxisIndicated`, covers the most common case — no recorded risk factor and no
bleeding risk — and it still writes to the record: a `RiskAssessment` proposal stating
that VTE risk was assessed and pharmacologic prophylaxis is not indicated. Recording that
decision is what quality reporting asks for; a silent absence reads as an oversight rather
than a decision. The low-risk path deliberately proposes no medication order, and early
ambulation or mechanical measures remain a host decision this fixture does not make.

Run the standalone CDS Hooks-shaped invocation with:

```console
cargo run -p horismos-fhir --example vte_standalone
```

The same `vte.ttl` also backs the `text → decisions` demo (`make demo-jev`,
`make demo-jev-live`), which perceives the four booleans from a free-text clinical
note instead of the recorded bundle and lets the same rules decide — see
`docs/framework/08-bridge.md` § "A decision model: `jev:`".

The clinical values are synthetic and this is not a treatment recommendation or a
conformance/certification claim.
