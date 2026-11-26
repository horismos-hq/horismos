# Horismos

**Ontology-first neuro-symbolic decision and capability kernel. Rust core, Python bindings.**

A subject-matter expert authors one artifact — a small RDFS-plus overlay declaring the
rules, constraints, and decision/workflow fragment a deployment needs — and the runtime
evaluates it as a verified, versioned projection over an existing system of record. LLMs
exist only at two declared transducer boundaries (perception: NL → typed instances;
narration: proof → NL). Every conclusion carries a proof tree; every rule set is formally
verified before it runs. Delete the LLM and the kernel still runs.

**Neural proposes. Symbolic decides.**

## Status

Pre-alpha. The symbolic core, the Python bindings, the agent loop (from Rust and from
Python, with the Jev and OpenRouter bridges), formal verification and the standards/host
integration layer are built.

## Install

```sh
pip install horismos          # or: uv pip install horismos
```

Every version so far is a pre-release. While no stable version exists, pip and uv install
the latest pre-release without being asked; once a stable version ships, a plain install
picks that instead, and `pip install --pre horismos` or
`uv pip install --prerelease allow horismos` asks for a newer pre-release explicitly.

A release ships prebuilt wheels for Linux x86_64 (glibc 2.17 or newer), Linux aarch64
(glibc 2.28 or newer), macOS universal2 and Windows x86_64 — one abi3 wheel per platform,
serving every CPython from 3.11 onward. The release pipeline installs each of the four and
runs the symbolic quickstart and `python -m horismos demo` against it on Python 3.11 through
3.14, the aarch64 wheel under emulation. Every wheel includes the Jev and OpenRouter model
bridges and the example ontologies the docs load. Anything else installs from the source
archive, which compiles the Rust core locally, so it needs a Rust toolchain and a few minutes.

The first run needs nothing else — no clone, no key, no network:

```sh
python -m horismos demo       # the VTE text → decisions demo on the scripted mock bridge
```

## A first rule set

The ontology is the program. This one declares a property, a decision, and the rules that
settle it — then the engine derives the answer and explains it:

```python
import pathlib
import tempfile

from horismos import Ontology, World

TRIAGE = """
@prefix :      <https://example.org/triage#> .
@prefix horis: <https://w3id.org/horismos#> .
@prefix rdf:   <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs:  <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .

:Patient a rdfs:Class .
:heartRate a rdf:Property ;
    rdfs:domain :Patient ; rdfs:range xsd:decimal ;
    horis:unit "bpm" ; horis:min 0 ; horis:max 300 ;
    horis:cardinality "0..1" .

:AcuityLevel a rdfs:Class ;
    horis:oneOf ( :Immediate :Standard ) .
:acuity a rdf:Property ;
    rdfs:domain :Patient ; rdfs:range :AcuityLevel ;
    horis:cardinality "0..1" ;
    horis:assignedBy horis:RulesOnly .          # a model may never set this

:r1 a horis:Rule ;
    horis:priority 10 ;
    horis:when [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 120 ] ;
    horis:then [ horis:property :acuity ; horis:value :Immediate ] ;
    horis:explain "Tachycardia - immediate track" .

:r2 a horis:Rule ;
    horis:priority 9 ;
    horis:when [ horis:subject :Patient ; horis:property :heartRate ; horis:atMost 120 ] ;
    horis:then [ horis:property :acuity ; horis:value :Standard ] ;
    horis:explain "Heart rate at most 120 - standard track" .

:r3 a horis:Rule ;
    horis:priority 8 ;
    horis:when [ horis:subject :Patient ; horis:property :heartRate ; horis:isEmpty true ] ;
    horis:then [ horis:property :acuity ; horis:value :Standard ] ;
    horis:explain "No heart-rate reading - standard track" .
"""

path = pathlib.Path(tempfile.mkdtemp()) / "triage.ttl"
path.write_text(TRIAGE)

onto = Ontology.load(path)          # verifies the rule set, or refuses to load
world = World(onto)

patient = world.assert_(":Patient", heart_rate=128)
assert world.value(patient, ":acuity") == onto.ref(":Immediate")    # derived by :r1
print(world.explain(patient, ":acuity").render())
```

```
:acuity = :Immediate   via r1 — "Tachycardia - immediate track"
  └─ heartRate 128 exceeds 120   (asserted by the api)
```

Loads fail closed: drop `:r3` and the load refuses, naming the case it cannot settle
("no acuity assigned when no heartRate assigned") rather than leaving a decision
undefined at runtime.

## The two boundaries

A model touches the system in exactly two declared places, and neither of them decides
anything:

- **Perception** turns natural language into typed instances, which then pass the same
  validation gate as any other mutation.
- **Narration** turns a finished proof into prose.

No rule fires, no workflow transitions, and no tool is called by a model. That line is
what makes the proof tree worth reading — and it is why removing the model leaves a
working kernel rather than a broken one.

## Documentation

- [Quickstart](https://docs.horismos.ai/quickstart/) — the zero-key path
- [Guarantees](https://docs.horismos.ai/guarantees/) — what is proven, and what is not
- [Architecture](https://docs.horismos.ai/architecture/)
- [Coming from LangGraph](https://docs.horismos.ai/coming-from-langgraph/)
- [Governing a coding agent](https://docs.horismos.ai/governing-a-coding-agent/)
- [Use cases](https://docs.horismos.ai/use-cases/)
- [Framework reference](https://docs.horismos.ai/framework/00-index/) — the full
  definition, including the [Python API](https://docs.horismos.ai/framework/11-python-api/)

Everything is at <https://docs.horismos.ai/>.

## Contributing

Repository: <https://github.com/horismos-hq/horismos>. Build, test and docs-site
instructions are in
[CONTRIBUTING.md](https://github.com/horismos-hq/horismos/blob/main/CONTRIBUTING.md).

## License

MIT OR Apache-2.0.
