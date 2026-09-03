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

Run the standalone CDS Hooks-shaped invocation with:

```console
cargo run -p horismos-fhir --example vte_standalone
```

The clinical values are synthetic and this is not a treatment recommendation or a
conformance/certification claim.
