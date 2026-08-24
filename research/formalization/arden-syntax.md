# Arden Syntax

## Claim

Arden Syntax is the earliest widely-adopted clinical rule-logic standard (Medical
Logic Modules, still an active HL7 standard, v3.0 published October 2023). Its
"curly braces" escape hatch — an unguarded, per-institution bypass around standard
rule logic — is the documented cause of its portability failure, and is the literal
opposite of horismos's uniform validation gate.

## Evidence

**[measured]** Arden Syntax saw real deployments (Regenstrief CARE, LDS HELP, CHICA)
per [Wikipedia's overview](https://en.wikipedia.org/wiki/Arden_Syntax|).

**[measured]** Samwald et al. (2012) find Arden works for standalone rules but hits
major technical-integration barriers in heterogeneous hospital information systems
([pubmed.ncbi.nlm.nih.gov/22342733](https://pubmed.ncbi.nlm.nih.gov/22342733/)).

**[measured]** "Sharing MLM's: An Experiment between Columbia-Presbyterian and LDS
Hospital" directly documents the curly-braces portability problem: institution-specific
free-text blocks inside otherwise-standard modules broke interchange
([archival PDF](https://pdf.core.ac.uk/corefilesystem/pdf/11926/276277185.pdf)).

**[inference]** Arden's curly braces are the structural opposite of horismos's rule:
"every World mutation passes the [same validation] gate, regardless of provenance."
Arden's failure mode is direct evidence for why that hard rule matters, not just an
abstract design preference.

## Source

- Wikipedia, Arden Syntax — https://en.wikipedia.org/wiki/Arden_Syntax
- Samwald et al., "The Arden Syntax standard for clinical decision support:
  Experiences and directions" (2012) — https://pubmed.ncbi.nlm.nih.gov/22342733/
- "Sharing MLM's: An Experiment between Columbia-Presbyterian and LDS Hospital"
  (archival) — https://pdf.core.ac.uk/corefilesystem/pdf/11926/276277185.pdf

## Confidence

measured (deployment history and the curly-braces failure mode), with one inference
connecting it to horismos's own gate design.

## Last-verified

2026-08-24
