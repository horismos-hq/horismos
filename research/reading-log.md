# Reading log

The single canonical cross-topic log of every source cited anywhere in this wiki.
Append-only — new rows go at the bottom of the relevant section, or a new section is
added for a new topic. Grouped by research question/topic for scan-ability.
`[unverified]` flags a source relayed only via a search snippet where direct
primary-text fetch failed — noted honestly rather than silently upgraded to a
confirmed citation.

Populated from the founding `horismos-research-s1` sweep (2026-08-24); filed-under
paths point to where each source's finding lives in this wiki.

### Captain intake

| Source | Date | Takeaway | URL | Filed under |
|---|---|---|---|---|
| Vasilije Markovic, "How do you go from learnable ontologies to world models?" | 2026-08-18 | Cognee founder argues agent memory can approach a world model via a continuous trace↔ontology feedback loop; explicitly hedges this isn't yet real predictive dynamics | https://www.linkedin.com/pulse/how-do-you-go-from-learnable-ontologies-world-models-markovic-flqtf/ | `intake/filed/2026-08-24-markovic-learnable-ontologies.md`, `ontology-authoring/autonomous-agents-failure-modes.md` |
| Markovic, Obradovic, Hajdu, Pavlovic, "Optimizing the Interface Between Knowledge Graphs and LLMs for Complex Reasoning" | 2025 | Cognee's own KG-LLM hyperparameter-tuning study across HotPotQA/2WikiMultiHop/MuSiQue; gains inconsistent across datasets | https://arxiv.org/abs/2505.24478 | `intake/filed/2026-08-24-markovic-learnable-ontologies.md` |
| Ken Huang, "Why Ontology Matters for Agentic AI in 2026: From World Models to Governable Decisions" | 2026 | Frames ontology as a "context compiler"/operational interface; cites Palantir Ontology, W3C PROV, NIST AI RMF, EU AI Act | https://kenhuangus.substack.com/p/why-ontology-matters-for-agentic | `intake/filed/2026-08-24-markovic-learnable-ontologies.md` |
| Cognee (product/GitHub) | ongoing | Open-source AI agent memory platform, MIT-licensed, $7.5M seed | https://github.com/topoteretes/cognee | `intake/filed/2026-08-24-markovic-learnable-ontologies.md` |

### Q1 — Clinical guideline/workflow formalization

| Source | Date | Takeaway | URL | Filed under |
|---|---|---|---|---|
| Wikipedia, Arden Syntax | accessed 2026 | History, real deployments (Regenstrief CARE, LDS HELP, CHICA), curly-braces criticism | https://en.wikipedia.org/wiki/Arden_Syntax | `formalization/arden-syntax.md` |
| Samwald et al., "The Arden Syntax standard for clinical decision support: Experiences and directions" | 2012 | Works for standalone rules; major technical-integration barriers in heterogeneous HIS | https://pubmed.ncbi.nlm.nih.gov/22342733/ | `formalization/arden-syntax.md` |
| "Sharing MLM's: An Experiment between Columbia-Presbyterian and LDS Hospital" | archival | Direct documentation of the curly-braces portability problem | https://pdf.core.ac.uk/corefilesystem/pdf/11926/276277185.pdf | `formalization/arden-syntax.md` |
| Ohno-Machado et al., "The GuideLine Interchange Format" | ~1998 | GLIF's own pilot: encoding variability + vocabulary problem defeated interchange; no NLP existed for eligibility criteria | https://pmc.ncbi.nlm.nih.gov/articles/PMC61313/ | `formalization/glif.md` |
| GLIF3 evolution paper | 2000 | 3-level architecture: flowchart / verifiable computable spec / implementable spec | https://pubmed.ncbi.nlm.nih.gov/11079963/ | `formalization/glif.md` |
| "Evaluation of PROforma as a language for implementing medical guidelines" | 2006 | 6-pharmacy hypertension trial; documents authoring-burden limitations | https://pmc.ncbi.nlm.nih.gov/articles/PMC1459125/ | `formalization/proforma-deontics.md` |
| Deontics company site | accessed 2026 | PROforma's commercial descendant | https://deontics.com/about/ | `formalization/proforma-deontics.md` |
| "Pioneering NHS trust using AI to help doctors make right treatment decisions" | 2023 | Deontics/Guy's & St Thomas' prostate pathway, 33% triage figure (vendor-reported) | https://hospitalhealthcare.com/news/pioneering-nhs-trust-using-ai-to-help-doctors-make-right-treatment-decisions/ | `formalization/proforma-deontics.md` |
| Peleg, "Computer-interpretable clinical guidelines: a methodological review" | 2013 | Full lifecycle review across CIG formalisms | https://www.sciencedirect.com/science/article/pii/S1532046413000841 | `formalization/00-overview.md` |
| Review of CIGs for multimorbidity management | 2018 | TNM family confirmed; most cited works proof-of-concept, not operational | https://pmc.ncbi.nlm.nih.gov/articles/PMC6172935/ | `formalization/tnm-family.md` |
| "Versioning computer-interpretable guidelines... LASSIE" | 2010 | 133/515 sentences inheritable on one guideline update | https://pmc.ncbi.nlm.nih.gov/articles/PMC2859225/ | `formalization/tnm-family.md` |
| "Identifying Barriers to Hypertension Guideline Adherence... ATHENA" | ~2007 | ATHENA-HTN VA trial clinician-feedback analysis | https://www.ncbi.nlm.nih.gov/pmc/articles/PMC1839712/ | `formalization/tnm-family.md` |
| "Implementing clinical practice guidelines... ATHENA DSS" | ~2008 | No significant BP-control difference from DSS alone vs. control | https://pmc.ncbi.nlm.nih.gov/articles/PMC2243943/ | `formalization/tnm-family.md` |
| ATHENA-HTN trial registry | — | Multi-site VA RCT record | https://classic.clinicaltrials.gov/ct2/show/results/NCT00374452 | `formalization/tnm-family.md` |
| SAGE Guideline Model overview | 2007 | 5-year NIST-funded consortium; no post-2006 continuation found | https://www.clinfowiki.org/wiki/index.php/The_SAGE_guideline_model | `formalization/tnm-family.md` |
| Terenziani et al., "The GLARE Approach to Clinical Guidelines" | ~2004 | 7-year Turin collaboration; self-framed as prototypical | https://people.unipmn.it/alessio/doc/terenziani_CPG.pdf | `formalization/tnm-family.md` |
| "GEM at 10" | ~2011 | Decade retrospective still asking what limits adoption | https://www.researchgate.net/publication/51921221 | `formalization/00-overview.md` |
| CAREPATH study, Frontiers in Medicine | 2024 | 296 CDS rules/326 CDS Hooks cards from 25 guidelines; bespoke co-creation methodology needed | https://pmc.ncbi.nlm.nih.gov/articles/PMC11163046/ | `formalization/cql-cds-hooks-fhir.md` |
| "Assessment of 3 standards-based CDS tools... CQL, CDS Hooks, FHIR" | 2025 | 96–100% accuracy; real barrier was EHR data-documentation gaps, not the language | https://pmc.ncbi.nlm.nih.gov/articles/PMC12309839/ | `formalization/cql-cds-hooks-fhir.md` |
| CDS Hooks specification home | 2025 | v2.0.1 STU2; normative ballot planned 2025 | https://cds-hooks.hl7.org/ | `formalization/cql-cds-hooks-fhir.md` |
| FHIR Clinical Reasoning Module spec | current | PlanDefinition/ActivityDefinition; targeting FHIR R6 normative | https://hl7.org/fhir/clinicalreasoning-module.html | `formalization/cql-cds-hooks-fhir.md` |
| BPM+ Health / HL7 BPM Community of Practice | 2024–2025 | OMG-to-HL7 transition; generic BPMN/DMN/CMMN reuse strategy | https://www.bpm-plus.org/ | `formalization/bpm-plus-health.md` |
| Da Vinci Prior Authorization Support IG | current | CRD/DTR/PAS architecture; X12-membership friction noted | https://hl7.org/fhir/us/davinci-pas/ | `formalization/prior-authorization-standards.md` |
| CAQH 2023 Index Report | 2024 | Only 35% of PA transactions fully electronic (up from 28%) | https://caqh.org/hubfs/43908627/drupal/2024-01/2023_CAQH_Index_Report.pdf | `formalization/prior-authorization-standards.md` |
| Override rate of DDI alerts, systematic review/meta-analysis | 2024 | ~90% DDI override rate; 46.2–96.2% range across 23 studies | https://journals.sagepub.com/doi/10.1177/14604582241263242 | `formalization/00-overview.md` |
| Kawamoto, Houlihan, Balas, Lobach, BMJ | 2005 | 70-RCT review; 4 features predict CDS success | https://pubmed.ncbi.nlm.nih.gov/15767266/ | `formalization/00-overview.md` |
| Wright et al., CDS alert malfunction taxonomy, JAMIA | 2018 | 68 malfunctions, 14 sites, persist for long periods | https://dx.doi.org/10.1093/jamia/ocx106 | `formalization/00-overview.md` |
| "CDS malfunctions in a commercial EHR" (OHSU/Epic) | 2018 | 4 malfunctions, all silent/undetected, all from unrelated system changes | https://pmc.ncbi.nlm.nih.gov/articles/PMC6220702/ | `formalization/00-overview.md` |
| "Verification of Medical Guidelines by Model Checking" (Asbru) | ~2005 | Model-checking verification research line; stayed small-scale/academic | https://spinroot.com/spin/symposia/ws06/027.pdf | `formalization/tnm-family.md` |
| MobiGuide evaluation | 2017 | Small-scale patient trial (10 AF + 19 GDM patients) | https://link.springer.com/article/10.1007/s11257-017-9190-5 | `formalization/tnm-family.md` |
| CDS Hooks implementer list | — | Vendor support breadth | https://www.trisotech.com/cds-hooks/ | `formalization/cql-cds-hooks-fhir.md` |

### Q2 — Scenario landscape beyond triage

| Source | Date | Takeaway | URL | Filed under |
|---|---|---|---|---|
| Wong et al., JAMA Internal Medicine | 2021 | Epic Sepsis Model: AUC 0.63, sensitivity 33%, 109 alerts/true case | https://jamanetwork.com/journals/jamainternalmedicine/fullarticle/2781307 | `scenarios/sepsis-bundle-compliance.md`, `validation/case-studies.md` |
| STAT News, Epic sepsis coverage | 2021-06-21 | "Little better than a coin flip"; no prior external validation | https://www.statnews.com/2021/06/21/epic-sepsis-prediction-tool/ | `validation/case-studies.md` |
| Habib et al. editorial, JAMA IM | 2021 | "The Epic Sepsis Model Falls Short — The Importance of External Validation" | https://pubmed.ncbi.nlm.nih.gov/34152360/ | `validation/case-studies.md` |
| Sepsis alert systematic review/meta-analysis | 2024 | Alert systems associated with lower mortality/better SSC-bundle adherence, alert-fatigue caveat | https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11265133/ | `scenarios/sepsis-bundle-compliance.md` |
| Adams et al., Nature Medicine (TREWS) | 2022 | 590,736 patients, 5 hospitals; confirmed alerts → ~18% relative mortality reduction | https://www.nature.com/articles/s41591-022-01894-0 | `scenarios/sepsis-bundle-compliance.md`, `validation/case-studies.md` |
| NEWS2 prognostic accuracy study | — | AUROC ~0.67–0.69 for mortality, poor for cardiac arrest | https://www.ncbi.nlm.nih.gov/pmc/articles/PMC8718668/ | `scenarios/news2-deterioration.md` |
| NEWS/NEWS2 systematic review | 2026 | Outcome improvement inconsistent; attributed to escalation non-adherence | https://link.springer.com/article/10.1186/s13643-026-03088-y | `scenarios/news2-deterioration.md` |
| Kucher et al., NEJM | 2005 | RCT (1,255 vs. 1,251): computer alert reduced DVT/PE rates | https://www.nejm.org/doi/full/10.1056/NEJMoa041533 | `scenarios/vte-prophylaxis.md` |
| AHRQ MATCH Toolkit | — | Medication-reconciliation-at-transitions toolkit | https://www.ahrq.gov/patient-safety/settings/hospital/match/index.html | `scenarios/ruled-out.md` |
| CMS Interoperability and Prior Authorization Final Rule | 2024-01-17 | CMS-0057-F requires FHIR PA APIs (Da Vinci CRD/DTR/PAS) | https://www.cms.gov/newsroom/fact-sheets/cms-interoperability-and-prior-authorization-final-rule-cms-0057-f | `scenarios/ruled-out.md` |
| CBS News, UnitedHealth/naviHealth lawsuit coverage | 2023–2024 | Lawsuit alleges nH Predict "known 90% error rate" | https://www.cbsnews.com/news/unitedhealth-lawsuit-ai-deny-claims-medicare-advantage-health-insurance-denials/ | `scenarios/ruled-out.md` |
| ProPublica, Cigna PXDX investigation | 2023-03-25 | 300,000 claims denied in 2 months, 1.2 sec/claim average | https://www.propublica.org/article/cigna-pxdx-medical-health-insurance-rejection-claims | `scenarios/ruled-out.md` |
| STAT News, Medicare Advantage AI probe | 2026-07-15 | Senate inquiry into UnitedHealth/Humana/CVS AI-driven denials | https://www.statnews.com/2026/07/15/medicare-advantage-ai-care-denials-probe-blumenthal-hawley/ | `scenarios/ruled-out.md` |
| STAT News, OIG report coverage | 2026-06-11 | HHS OIG: post-hospital care denial rates 51–80% at 3 major insurers | https://www.statnews.com/2026/06/11/medicare-advantage-oig-report-rehab-care-deny-appeal-reverse/ | `scenarios/ruled-out.md` |
| Clinical Infectious Diseases (WISCA antimicrobial CDS trial) | 2021 | RCT, 6,849 patients; no overall benefit except cellulitis LOS | https://academic.oup.com/cid/article/72/9/e265/5876588 | `scenarios/00-catalog.md` |
| Antibiotic CDSS meta-analysis (30 trials) | — | Median 3% improvement; questions clinical significance | https://www.researchgate.net/publication/331401165 | `scenarios/00-catalog.md` |
| JAMIA chronic-disease CDS meta-analysis | 2022 | 76 studies; small/heterogeneous effects, no reliable design predictor | https://academic.oup.com/jamia/article/29/10/1757/6639847 | `scenarios/ruled-out.md` |
| STRATIFY ED disposition tool | 2025 | 13-factor low-risk HF-discharge score | https://link.springer.com/article/10.1186/s43058-025-00779-w | `scenarios/ruled-out.md` |
| CMS AUC Program pause, ACC | 2023-12-14 | Program paused for claims-infrastructure reasons, not clinical ones | https://www.acc.org/Latest-in-Cardiology/Articles/2023/12/14/15/16/CMS-Pauses-AUC-Program-For-Advanced-diagnostic-Imaging-Rescinds-Current-Regulations | `scenarios/ruled-out.md` |
| iCareWean, AJRCCM | 2006 | Closed-loop weaning cut wean time 4 days → 2 days | https://www.atsjournals.org/doi/10.1164/rccm.200511-1780OC | `scenarios/ruled-out.md` |
| Alert-override systematic review | 2020 | DDI override prevalence ~90% (95% CI 85–95%) | https://www.ncbi.nlm.nih.gov/pmc/articles/PMC7400042/ | `scenarios/00-catalog.md` |

### Q3 — Neuro-symbolic / ontology-grounded clinical AI

| Source | Date | Takeaway | URL | Filed under |
|---|---|---|---|---|
| Li, Huang, Naik, "Scallop: A Language for Neurosymbolic Programming" | 2023 | Datalog + differentiable provenance semirings; formal but built for gradient training | https://arxiv.org/abs/2304.04812 | `neurosymbolic-landscape/00-overview.md`, `neurosymbolic-landscape/lineage-assessment.md` |
| Riegel et al., "Logical Neural Networks" | 2020 | Weighted real-valued logic, differentiable, bound-propagation | https://arxiv.org/abs/2006.13155 | `neurosymbolic-landscape/lineage-assessment.md` |
| IBM/LNN GitHub | ongoing | Reference implementation, mature/stable | https://github.com/IBM/LNN | `neurosymbolic-landscape/lineage-assessment.md` |
| Manhaeve et al., "DeepProbLog" | 2018 | Neural predicates + ProbLog, exact/approx weighted model counting | https://arxiv.org/abs/1805.10872 | `neurosymbolic-landscape/lineage-assessment.md` |
| Marra, Dumančić, Manhaeve, De Raedt, survey, Artificial Intelligence 328 | 2024 | Unifying statistical-relational + neurosymbolic AI survey | https://www.sciencedirect.com/science/article/pii/S0004370223002084 | `neurosymbolic-landscape/00-overview.md` |
| Wikipedia, Neuro-symbolic AI | current | Kautz Type 1–6 taxonomy; no mention of MYCIN/Arden/expert systems | https://en.wikipedia.org/wiki/Neuro-symbolic_AI | `neurosymbolic-landscape/lineage-assessment.md` |
| Wikipedia, Arden Syntax | current | Still-active HL7 standard, v3.0 Oct 2023 | https://en.wikipedia.org/wiki/Arden_syntax | `neurosymbolic-landscape/lineage-assessment.md` |
| Wikipedia, MYCIN | current | Backward chaining, certainty factors, trace-as-explanation | https://en.wikipedia.org/wiki/Mycin | `neurosymbolic-landscape/lineage-assessment.md` |
| Trinh, Wu, Le, He, Luong, "AlphaGeometry," Nature | 2024 | LLM proposes, symbolic deduction engine verifies — genuine hard gate | https://www.nature.com/articles/s41586-023-06747-5 | `neurosymbolic-landscape/00-overview.md` |
| Pan et al., "Logic-LM" | 2023 | LLM→FOL, Z3/Prover9 decides — hard gate | https://arxiv.org/abs/2305.12295 | `neurosymbolic-landscape/00-overview.md` |
| Ye et al., "SATLM" | 2023 | NL→SAT, Z3 solves — hard gate | https://proceedings.neurips.cc/paper_files/paper/2023/file/8e9c7d4a48bdac81a58f983a64aaf42b-Paper-Conference.pdf | `neurosymbolic-landscape/00-overview.md` |
| Loaiza-Bonilla et al., oncology trial-matching neurosymbolic system | 2026 | Graph-enforced consistency checks; F1 0.82 vs. 0.47/0.67 ungated GPT-4 | https://pmc.ncbi.nlm.nih.gov/articles/PMC13091143/ | `neurosymbolic-landscape/00-overview.md` |
| Gao et al., DR.KNOWS, JMIR AI | 2025 | UMLS graph-path RAG for diagnosis — retrieval, not gating | https://ai.jmir.org/2025/1/e58670 | `neurosymbolic-landscape/00-overview.md` |
| Chang & Sung, SNOMED CT in LLMs scoping review | 2024 | Field lacks standardized KG-integration methods | https://medinform.jmir.org/2024/1/e62924 | `terminology/snomed-ct.md` |
| JAMIA RAG-in-biomedicine meta-analysis | 2025 | Pooled OR 1.35 favoring RAG; hallucination reduction not consistently quantified | https://academic.oup.com/jamia/article/32/4/605/7954485 | `neurosymbolic-landscape/00-overview.md` |
| Singhal et al., Med-PaLM 2, Nature Medicine | 2025 | 86.5% MedQA; pure scaling, no symbolic component | https://www.nature.com/articles/s41591-024-03423-7 | `neurosymbolic-landscape/00-overview.md` |
| Adversarial hallucination study, Communications Medicine | 2025 | 6 LLMs repeated planted clinical errors in 50–83% of cases | https://www.nature.com/articles/s43856-025-01021-3 | `neurosymbolic-landscape/00-overview.md` |
| "Representation Before Retrieval," medRxiv | 2026 | RAG raised unsupported claims 5.0%→43.6% in one setting | https://www.medrxiv.org/content/10.64898/2026.02.13.26346256v1.full | `neurosymbolic-landscape/00-overview.md` |
| Abridge product page | 2026 | Transcript-linked notes; human-verified, no symbolic layer claimed | https://www.abridge.com/ai | `neurosymbolic-landscape/00-overview.md` |
| Epic AI agents launch, Healthcare IT News | 2025 | Physician sign-off is the safety gate for Epic's Art/Emmie/Penny | https://www.healthcareitnews.com/news/epic-unveils-ai-agents-showcases-new-foundational-models | `neurosymbolic-landscape/00-overview.md` |
| OpenAI Agents SDK, Guardrails docs | current | Guardrails confirmed soft/post-hoc; tools may fire before a tripwire cancels | https://openai.github.io/openai-agents-python/guardrails/ | `neurosymbolic-landscape/00-overview.md` |
| Google ADK, Workflow Agents docs | current | Genuine hard gate, but only over fixed orchestration, not LLM node content | https://adk.dev/agents/workflow-agents/ | `neurosymbolic-landscape/00-overview.md` |
| Apache KIE (Drools) homepage | current | Plain rules engine, no LLM/agent/formal-verification content | https://kie.apache.org/ | `neurosymbolic-landscape/00-overview.md` |
| AUI / Apollo-1 | GA claimed Q2 2026 | Markets "first neuro-symbolic model"; no published technical proof found | https://www.aui.io | `neurosymbolic-landscape/00-overview.md` |
| Lean4Agent | 2026 | Lean4 verifies agent workflow structure pre-execution; ~12% SWE-Bench gain | https://arxiv.org/abs/2606.06523 | `neurosymbolic-landscape/lineage-assessment.md` |
| VeriPlan, CHI 2025 | 2025 | LLM planner → formal property translation → model checker | https://doi.org/10.1145/3706598.3714113 | `neurosymbolic-landscape/00-overview.md` |
| Hekmatnejad, Simms, Fainekos, "Model Checking CDS Using SMT" | 2019 | Classical CDS formal-verification antecedent, no LLM connection | https://arxiv.org/abs/1901.04545 | `neurosymbolic-landscape/00-overview.md` |
| Sigloch & Benzmüller, "Neuro-Symbolic Verification of LLM Outputs" | 2026 | Formal+embedding hybrid gate; 83%/72% hallucination detection (medical-device domain) | https://arxiv.org/abs/2605.26942 | `neurosymbolic-landscape/00-overview.md` |

### Q4 — Clinical AI validation practice

| Source | Date | Takeaway | URL | Filed under |
|---|---|---|---|---|
| Wong et al., JAMA Internal Medicine | 2021 | Epic Sepsis Model independent validation: AUC 0.63, PPV 12% | https://jamanetwork.com/journals/jamainternalmedicine/fullarticle/2781307 | `validation/case-studies.md` |
| "Evaluation of Sepsis Prediction Models before Onset of Treatment," NEJM AI | 2024 | Label leakage: AUROC fell to 0.47 excluding post-suspicion data | https://ai.nejm.org/doi/full/10.1056/AIoa2300032 | `validation/case-studies.md` |
| "Multicenter Prospective Validation of an Updated Proprietary Sepsis Prediction Model," JAMA Network Open | 2026 | Epic Sepsis Model v2 (silent Dec 2023): AUROC 0.82–0.92, PPV still 0.13–0.26 | https://jamanetwork.com/journals/jamanetworkopen/fullarticle/2845595 | `validation/case-studies.md` |
| Adams et al., Nature Medicine (TREWS) | 2022 | Prospective outcome-level sepsis-alert evaluation | https://www.nature.com/articles/s41591-022-01894-0 | `validation/case-studies.md` |
| Sendak et al., "Sepsis Watch integration," JMIR Med Inform | 2020 | Duke's implementation study; live since Nov 2018 | https://medinform.jmir.org/2020/7/e15182/ | `validation/case-studies.md` |
| "Sepsis Watch multisite external validation," npj Digital Medicine | 2025 | Transferred to 4 new EDs, no retraining, per-site threshold tuning needed | https://pmc.ncbi.nlm.nih.gov/articles/PMC12159134/ | `validation/case-studies.md` |
| STAT News, "IBM's Watson supercomputer recommended 'unsafe and incorrect' cancer treatments" | 2018-07-25 | Trained on synthetic cases + a few specialists' preferences, not guidelines | https://www.statnews.com/2018/07/25/ibm-watson-recommended-unsafe-incorrect-treatments/ | `validation/case-studies.md` |
| STAT News, "IBM pitched Watson as a revolution... nowhere close" | 2017-09-05 | Earlier investigation into Watson for Oncology underperformance | https://www.statnews.com/2017/09/05/watson-ibm-cancer/ | `validation/case-studies.md` |
| Royal Free London, Streams audit | n.d. | NHS trust's own DeepMind Streams evaluation disclosure | https://www.royalfree.nhs.uk/news/royal-free-london-publishes-audit-streams-app | `validation/case-studies.md` |
| Digital Health News, Streams cost-savings peer review | 2019-07 | ~£2,000/patient cost savings, reduced AKI admission costs | https://www.digitalhealth.net/2019/07/deepminds-streams-saves-2000-peer-review/ | `validation/case-studies.md` |
| "The silent trial" methodology paper | 2022 | Hydronephrosis case: retrospective AUROC 0.90 → live shadow AUROC 0.50 (chance) | https://pmc.ncbi.nlm.nih.gov/articles/PMC9424628/ | `validation/00-ladder.md` |
| DDX-BRO trial, Lancet Digital Health | 2024/2025 | Well-powered cluster RCT, null result on diagnostic-quality-risk outcome | https://www.thelancet.com/journals/landig/article/PIIS2589-7500(24)00250-4/fulltext | `validation/00-ladder.md` |
| eRegQual trial, Lancet Digital Health | 2021 | Positive CDS RCT counter-example (structured antenatal care) | https://www.thelancet.com/journals/landig/article/PIIS2589-7500(21)00269-7/fulltext | `validation/00-ladder.md` |
| "Effect of Clinical Decision-Support Systems," Annals of Internal Medicine | 2012 | Process-of-care gains, little/no mortality effect | https://www.acpjournals.org/doi/10.7326/0003-4819-157-1-201207030-00450 | `validation/00-ladder.md` |
| "1,357 AI medical devices cleared, 3 tested on patient outcomes," PLOS Digital Health | 2025 | Only 0.2% of FDA-cleared AI devices ever evaluated against a patient-centered outcome | https://journals.plos.org/digitalhealth/article?id=10.1371%2Fjournal.pdig.0001597 | `validation/00-ladder.md` |
| FDA, AI/ML-Enabled Medical Devices list | ongoing | Public registry of cleared AI/ML devices | https://www.fda.gov/medical-devices/software-medical-device-samd/artificial-intelligence-and-machine-learning-aiml-enabled-medical-devices | `validation/regulatory.md` |
| FDA, PCCP final guidance | 2024-12-03 | Pre-authorized bounded future AI-device modification envelopes | https://www.fda.gov/regulatory-information/search-fda-guidance-documents/marketing-submission-recommendations-predetermined-change-control-plan-artificial-intelligence | `validation/regulatory.md` |
| FDA/Health Canada/MHRA, GMLP guiding principles | 2021-10 | 10 joint guiding principles for AI/ML medical device development | https://www.fda.gov/media/153486/download | `validation/regulatory.md` |
| Federal Register, CDS software guidance | 2022-09-28 | Final interpretation of the 21st Century Cures Act's non-device CDS exemption | https://www.federalregister.gov/documents/2022/09/28/2022-20993/clinical-decision-support-software-guidance-for-industry-and-food-and-drug-administration-staff | `validation/regulatory.md` |
| "27 Years of AI/ML Medical Device Recalls" | n.d. | ~5–6% of AI/ML devices ever recalled, mostly software bugs | https://www.ncbi.nlm.nih.gov/pmc/articles/PMC12274014/ | `validation/regulatory.md` |
| [unverified — preprint] "Calibration Drift Under Cross-Institutional Deployment," medRxiv | 2026 | Internal calibration fine, external calibration can silently fail; AUROC blind to it | https://www.medrxiv.org/content/10.64898/2026.05.03.26352335v1.full | `validation/00-ladder.md` |

### Q5 — Terminology licensing and subsumption at scale

| Source | Date | Takeaway | URL | Filed under |
|---|---|---|---|---|
| SNOMED International, "Get SNOMED CT" | accessed 2026 | Free for Member countries; fees by territory outside membership | https://www.snomed.org/get-snomed | `terminology/snomed-ct.md` |
| SNOMED International, vendor licensing guide | accessed 2026 | Distributing SNOMED-enabled products requires Affiliate status + sub-licensing | https://docs.snomed.org/snomed-ct-practical-guides/vendor-introduction-to-snomed-ct/7-licensing | `terminology/snomed-ct.md` |
| NLM, SNOMED CT FAQs | accessed 2026 | NLM distributes SNOMED CT free to US/international users via UMLS license | https://www.nlm.nih.gov/healthit/snomedct/faq.html | `terminology/snomed-ct.md` |
| SNOMED International, "Do I need an external interface terminology?" | accessed 2026 | ~368,000 active concepts; explicitly warns against hand-rolled custom subsets | https://docs.snomed.org/implementation-guides/implementation-fact-sheets/implementation-dilemmas/do-i-need-an-external-interface-terminology-to-implement-snomed-ct | `terminology/snomed-ct.md` |
| SNOMED International, Concept Inactivation | accessed 2026 | ~25% of concept space inactive at any time | https://confluence.ihtsdotools.org/display/DOCEG/Concept+Inactivation | `terminology/snomed-ct.md` |
| SNOMED International, ECL introduction | accessed 2026 | Formal language for intensional subset definitions | https://docs.snomed.org/snomed-ct-specifications/snomed-ct-expression-constraint-language/introduction/1-introduction | `terminology/snomed-ct.md` |
| SNOMED International, ICD-10 Map Overview | accessed 2026 | SNOMED→ICD-10 maps documented as one-directional and lossy | https://docs.snomed.org/snomed-ct-specifications/snomed-ct-to-icd-10-map-specification/introduction/2-icd-10-map-overview | `terminology/snomed-ct.md` |
| GitHub, IHTSDO/snowstorm | accessed 2026 | SNOMED International's own open-source terminology server | https://github.com/IHTSDO/snowstorm | `terminology/snomed-ct.md` |
| CSIRO, Ontoserver FAQs | accessed 2026 | Decouples implementers from raw RF2; enables frequent version updates | https://ontoserver-wp.csiro.au/faqs/ | `terminology/snomed-ct.md` |
| Metke-Jimenez et al., "Ontoserver: a syndicated terminology server" | — | Peer-reviewed description of CSIRO's FHIR terminology server | https://pubmed.ncbi.nlm.nih.gov/30223897/ | `terminology/snomed-ct.md` |
| [unverified — secondary summary only] Dentler et al., "Comparison of reasoners for large ontologies in OWL 2 EL" | — | ELK classifies full SNOMED CT in ~5s; CEL ~20 min | https://www.semantic-web-journal.net/sites/default/files/swj120_2.pdf | `terminology/snomed-ct.md` |
| Chu et al., JMIR Medical Informatics | 2019 | Extensional value sets capture median 65% (as low as 2%) of terms an intensional definition would | https://pmc.ncbi.nlm.nih.gov/articles/PMC6351992/ | `terminology/snomed-ct.md` |
| Daniel Vreeman, "Regenstrief's approach to licensing LOINC" | — | Free for commercial/non-commercial use; cannot fork/modify core content | https://danielvreeman.com/blog/2016/11/24/regenstriefs-approach-to-licensing-loinc/ | `terminology/loinc.md` |
| NLM, RxNorm Terms of Service | accessed 2026 | Core RXCUI/names are US public domain; full releases need free UMLS license | https://www.nlm.nih.gov/research/umls/rxnorm/docs/termsofservice.html | `terminology/rxnorm.md` |
| NLM, RxNav Data Sources | accessed 2026 | RxTerms/MED-RT ecosystem relationships | https://lhncbc.nlm.nih.gov/RxNav/information/DataSources.html | `terminology/rxnorm.md` |
| Repo file: `examples/triage/snomed-ct-subset.ttl` | current HEAD | Shipped fixture: ~50 concepts, single-parent only, no polyhierarchy | local repo | `terminology/snomed-ct.md` |
| Repo file: `docs/framework/02-ontology.md` (~line 132) | current HEAD | Resolved decision D5/PRP-001: vocabularies consumed as pre-extracted subset imports | local repo | `terminology/snomed-ct.md` |
| Repo file: `docs/framework/13-authoring.md` | current HEAD | Planned (unbuilt) `search_vocab` SNOMED-anchoring tool, PRP-007 scope | local repo | `terminology/snomed-ct.md` |

### Q6 — Agent-authored and learnable ontologies

| Source | Date | Takeaway | URL | Filed under |
|---|---|---|---|---|
| OntoGPT, Monarch Initiative | ongoing | Most mature tool found; structured extraction against an existing vocabulary, not from-scratch axiom authoring | https://github.com/monarch-initiative/ontogpt | `ontology-authoring/ontology-learning-benchmarks.md` |
| open-ontologies (Tesseract Academy) | 2026 | Closest external architectural analog: deterministic Rust OWL2-DL+SHACL engine gates LLM tool calls; self-reports 9th/13 on an OAEI benchmark | https://github.com/fabio-rovai/open-ontologies | `ontology-authoring/00-overview.md` |
| LLMs4OL 2024 Overview | 2024 | Term typing F1 0.9938 vs. relation extraction F1 0.0783 | https://arxiv.org/html/2409.10146v1 | `ontology-authoring/ontology-learning-benchmarks.md` |
| "Ontology Learning with LLMs: Axiom Identification Benchmark" (OntoAxiom) | 2025/26 | F1 0.218–0.642 by domain; "not high enough to fully automate axiom identification" | https://arxiv.org/abs/2512.05594 | `ontology-authoring/ontology-learning-benchmarks.md` |
| Li, Garijo & Poveda-Villalón, SLR, Semantic Web Journal | 2026 | 15,688 papers screened to 36; no standardized benchmarks, poor reproducibility field-wide | https://www.semantic-web-journal.net/content/large-language-models-ontology-engineering-systematic-literature-review | `ontology-authoring/00-overview.md` |
| Manda, "LLMs in Bio-Ontology Research: A Review" | 2025 | Maturity tiers; documents "workload shift rather than reduction" as a real failure mode | https://pmc.ncbi.nlm.nih.gov/articles/PMC12649945/ | `ontology-authoring/ontology-learning-benchmarks.md` |
| Rice trait ontology curation study | 2025 | LLM-as-fallback + mandatory expert sign-off; kappa 0.719 — closest structural sibling to PRP-007 found | https://pmc.ncbi.nlm.nih.gov/articles/PMC12670841/ | `ontology-authoring/prp-007-open-items.md` |
| AutoSchemaKG | 2025 (ACL 2026) | Fully autonomous web-scale schema induction; no dedup mechanism, admits residual inconsistency risk | https://arxiv.org/abs/2505.23628 | `ontology-authoring/autonomous-agents-failure-modes.md` |
| KARMA | 2025 | 9-agent autonomous KG enrichment; confidence-gated, not systematic human review | https://arxiv.org/abs/2502.06472 | `ontology-authoring/autonomous-agents-failure-modes.md` |
| Wikipedia, Never-Ending Language Learning (NELL) | accessed 2026 | Cookies→baked-goods→computer-files drift chain; 3% belief confidence | https://en.wikipedia.org/wiki/Never-Ending_Language_Learning | `ontology-authoring/autonomous-agents-failure-modes.md` |
| Wikipedia, Wikidata (governance) | accessed 2026 | Schema changes require mandatory human community discussion | https://en.wikipedia.org/wiki/Wikidata | `ontology-authoring/autonomous-agents-failure-modes.md` |
| "Neuro-Symbolic Integration + Ontological Reasoning" (HermiT-in-loop paper) | 2025 | Genuine reasoner-in-loop precedent — withdrawn by its own authors for inaccuracies | https://arxiv.org/abs/2504.07640 | `ontology-authoring/autonomous-agents-failure-modes.md` |
| GLaMoR | 2025 | Replaces symbolic OWL reasoner with a learned approximator — counter to horismos's anti-wrapper stance | https://arxiv.org/abs/2504.19023 | `ontology-authoring/autonomous-agents-failure-modes.md` |
| "Is Three the Magic Number?" repair-loop study | 2026 | Diminishing returns after 3–4 repair iterations | https://arxiv.org/abs/2607.05197 | `ontology-authoring/verification-of-llm-drafted-artifacts.md` |
| LASA | 2025 | Names "vacuous assertions" as a known LLM-verification failure mode | https://arxiv.org/abs/2506.17865 | `ontology-authoring/verification-of-llm-drafted-artifacts.md`, `ontology-authoring/prp-007-open-items.md` |
| CHARGE | 2026 | Reports non-vacuity rate as a headline verification metric | https://arxiv.org/abs/2607.27776 | `ontology-authoring/verification-of-llm-drafted-artifacts.md` |
| "From Natural Language to Verified Code" | 2026 | Names and designs against "vacuous verification" explicitly | https://arxiv.org/abs/2604.22601 | `ontology-authoring/verification-of-llm-drafted-artifacts.md` |
| Lahiri, "Evaluating LLM-driven User-Intent Formalization" | 2024 | No algorithmic guarantee a formal spec captures user intent | https://arxiv.org/abs/2406.09757 | `ontology-authoring/verification-of-llm-drafted-artifacts.md` |
| DafnyCOMP | 2025 | Per-function verification success drops sharply under composition | https://arxiv.org/abs/2509.23061 | `ontology-authoring/verification-of-llm-drafted-artifacts.md` |
| VeriContest | 2026 | 92% plain-coding success vs. 5.29% end-to-end on formally-verified variants | https://arxiv.org/abs/2605.08553 | `ontology-authoring/verification-of-llm-drafted-artifacts.md` |
| ReForm | 2025/26 | Names the "formalization gap" between provable and faithful | https://arxiv.org/abs/2510.24592 | `ontology-authoring/verification-of-llm-drafted-artifacts.md` |
| Turpin et al., "LMs Don't Always Say What They Think" | 2023 | CoT explanations can systematically misrepresent the true reason for a prediction | https://arxiv.org/abs/2305.04388 | `ontology-authoring/prp-007-open-items.md` |
| Lanham et al., "Measuring Faithfulness in Chain-of-Thought Reasoning" | 2023 | Larger, more capable models produce less faithful reasoning | https://arxiv.org/abs/2307.13702 | `ontology-authoring/prp-007-open-items.md` |
| xpSHACL | 2025 | Renders SHACL violations in NL — precedent for horismos's witness-rendering approach | https://www.vldb.org/2025/Workshops/VLDB-Workshops-2025/LLM+Graph/LLMGraph-6.pdf | `ontology-authoring/verification-of-llm-drafted-artifacts.md` |
| ARPaCCino | 2025 | LLM drafts Rego/OPA policy, iteratively refined against IaC compliance checks | https://arxiv.org/abs/2507.10584 | `ontology-authoring/verification-of-llm-drafted-artifacts.md` |
