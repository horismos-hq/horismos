# Insurance

## The scene

A claim arrives. First-pass adjudication is exactly the decision regulators and courts have been looking at: when it is automated opaquely, the record is not good. One Medicare Advantage tool faced litigation over an alleged "known 90% error rate"; ProPublica found another payer batch-denied 300,000 claims in two months at roughly 1.2 seconds per claim; a 2026 Senate and OIG inquiry followed (`research/scenarios/ruled-out.md`). CMS-0057-F now requires FHIR prior-authorization APIs, and the Da Vinci CRD/DTR/PAS guides define the wire.

None of that argues against automation. It argues against automation that cannot say *why*. The Horismos version of this decision produces, for every determination, a proof tree that names the exact clause it rests on — which is what makes a denial appealable and an approval auditable.

## The overlay

```turtle
--8<-- "examples/gallery/claims-adjudication.ttl"
```

Two design choices are the whole point. A missing prior authorization **pends** the claim for documentation rather than denying it (`pendMissingPriorAuth` outranks `approveClean` and never concludes `:Denied`), and an amount over the auto-adjudication ceiling goes to a human. The rules encode the policy you would defend in front of a regulator, and nothing else can set `:adjudication`.

## The decision, with proof

```python
from horismos import HorismosValidationError, Ontology, World

onto = Ontology.load("examples/gallery/claims-adjudication.ttl")
world = World(onto)

def adjudicate(subject: str, **facts: object) -> None:
    ref = world.assert_(":Claim", subject=subject, **facts)
    proof = world.explain(ref, ":adjudication")
    print(proof.render() if proof else f"{subject}: no determination")

adjudicate(":clm-1001", policy_active_on_service_date=True, service_covered=True,
           prior_auth_required=False, billed_amount=1840.00, network=":InNetwork")
adjudicate(":clm-1002", policy_active_on_service_date=True, service_covered=True,
           prior_auth_required=True, prior_auth_on_file=False, billed_amount=3200)
adjudicate(":clm-1003", policy_active_on_service_date=False, service_covered=True,
           prior_auth_required=False, billed_amount=120)
```

```text
:adjudication = :Approved   via approveClean — "Active policy, covered service, within ceiling, authorization satisfied"
  ├─ policyActiveOnServiceDate true is true   (asserted by the api)
  ├─ serviceCovered true is true   (asserted by the api)
  ├─ billedAmount 1840 atMost 10000   (asserted by the api)
  └─ any of:
     ├─ priorAuthRequired false is false   (asserted by the api)
     └─ priorAuthOnFile is true   (not held)

:adjudication = :Pended   via pendMissingPriorAuth — "Prior authorization is required and none is on file — pend for documentation, do not deny"
  ├─ priorAuthRequired true is true   (asserted by the api)
  └─ priorAuthOnFile false is false   (asserted by the api)

:adjudication = :Denied   via denyInactivePolicy — "Policy was not active on the date of service"
  └─ policyActiveOnServiceDate false is false   (asserted by the api)
```

The denial is one line long and names the one fact it rests on. An appeal disputes that fact, not a black box.

The authorization arrives for the pended claim. The determination changes in the transaction that recorded the document — no batch job, no stale "pended" status left behind — and the proof now shows the *other* branch of the disjunction held:

```python
adjudicate(":clm-1002", prior_auth_on_file=True)
```

```text
:adjudication = :Approved   via approveClean — "Active policy, covered service, within ceiling, authorization satisfied"
  ├─ policyActiveOnServiceDate true is true   (asserted by the api)
  ├─ serviceCovered true is true   (asserted by the api)
  ├─ billedAmount 3200 atMost 10000   (asserted by the api)
  └─ any of:
     ├─ priorAuthRequired is false   (not held)
     └─ priorAuthOnFile true is true   (asserted by the api)
```

High-dollar claims go to a person, and no adjuster script, model, or API caller can write the determination directly:

```python
adjudicate(":clm-1004", policy_active_on_service_date=True, service_covered=True,
           prior_auth_required=True, prior_auth_on_file=True, billed_amount=48000)
try:
    world.assert_(":Claim", subject=":clm-1005", policy_active_on_service_date=True,
                  service_covered=True, adjudication=":Approved")
except HorismosValidationError as e:
    print(f"[{e.constraint}] {e}")
```

```text
:adjudication = :Pended   via pendHighDollar — "Billed amount exceeds the auto-adjudication ceiling — human review"
  └─ billedAmount 48000 exceeds 10000   (asserted by the api)

[provenance_refused] adjudication may only be assigned by rules (horis:assignedBy horis:RulesOnly); assertion came from the api
```

## Why verification matters here

This is the rule set where completeness is a compliance property: *is there any claim this policy neither approves, pends, nor denies?* There was. The check answered over the declared input space and rendered each gap as a sentence — a claim with no `billedAmount` recorded, which could be tested against neither `approveClean`'s ceiling nor `pendHighDollar`'s floor; and a claim where `priorAuthRequired` was never populated, which satisfied neither branch of `approveClean`'s authorization test nor `pendMissingPriorAuth`'s. Both now resolve to `:Pended`, which is the outcome this rule set already reached for missing documentation: `pendMissingPriorAuth` says *pend for documentation, do not deny*. Auto-denying a claim because a field was never populated is precisely the failure this overlay exists to argue against, and completeness is what turns that principle into something a check can enforce. Conflict-freedom is equally load-bearing — `pendMissingPriorAuth` and `pendHighDollar` can both match one claim and agree, so neither is a conflict; `denyNonCoveredService` and `approveClean` must never both match, and the check proves it rather than trusting the author. It did name four real overlaps, each a deny rule against a pend rule, all resolved by priority and now guarded explicitly. Dead-rule detection catches a ceiling that `horis:min` makes unreachable. The assumption manifest records which benefit-plan version and which code-system edition the closure assumed, so an auditor never has to infer scope from silence.

What verification does not claim: that the plan document was transcribed correctly into these rules, that the eligibility feed was complete, or that the clinical criteria behind `serviceCovered` are sound. Those are outside the closure and are stated as such.

## Status and honest edges

- Runs today with structured input. Hydrating claims from a FHIR Claim/Coverage bundle with source versions in the proof, and returning a ClaimResponse or a CDS Hooks card conditionally, is PRP-008 territory.
- The research wiki deliberately deprioritizes prior authorization as a *first* Horismos deployment because of the harm record above — a maturity and liability judgment, not a technical one. The mechanism shown here is what a trustworthy deployment would need; when to deploy it is a human decision.
- Thresholds and rules are illustrative. A real plan has hundreds of clauses; the shape does not change.
