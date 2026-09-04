# G2MH Ensembl VEP parity work

This fork contains scientific compatibility changes used by the rare-variant
pipeline. It is not an unmodified checkout of `Huang-lab/fastVEP`.

## Production revision

The rare-variant annotation container pins commit
`cb8113d7bab2db42cb06bb2b2a40c57b60ea2561` from branch
`fix/vep-lof-parity`.

The main changes are:

- commit `3bdb862`: improve consequence and indel `HGVS_OFFSET` parity with
  Ensembl VEP;
- commits `be54445` through `cb8113d`: correct terminal shifted-insertion
  consequences using peptide-derived and ordered CDS boundaries.

Transcript selection is not part of these changes. The rare-variant pipeline
runs FastVEP for all transcript consequences and applies its own versioned
implementation of Ensembl VEP's `--pick_allele` hierarchy downstream.

## Regression evidence

In the G2MH chr22 10-gene pilot, the pre-patch upstream binary omitted three
indel `HGVS_OFFSET` values. This lost one qualifying LoF allele and two carrier
rows. The parity fork restored them.

Full-VEP comparison subsequently identified terminal shifted insertions on
chrX in `TMSB4X` and `PDK3`. The old FastVEP behavior classified them as
frameshifts; Ensembl VEP classified them as
`inframe_insertion&stop_retained_variant`. Commit `cb8113d` corrected them and
restored exact chrX burden parity with the older VEP-based G2MH output. The same
change corrected a `CBY1` consequence on chr22, without changing tiered chr22
burdens because that allele was outside the tiered genes.

Eight remaining normalized insertion consequence differences from the
comprehensive G2MH comparison were independently adjudicated by reconstructing
and translating the Ensembl 115 CDS. The results supported the FastVEP-selected
consequences; no additional correction was required.

The downstream G2MH chr22 comparison matched all 1,149 final
allele-sample-tier rows. One retained allele gained the newer Ensembl 115
`splice_polypyrimidine_tract_variant` subcategory, but its LoF classification,
tier, carrier, and burden were unchanged.

## Performance note

On 2026-09-04, the pinned container annotated the G2MH all-observed chr22 input
(1,148,541 allele records) on the warm 8-vCPU AWS orchestration host in 52.45
seconds. The raw FastVEP VCF was 3.1 GB. The downstream Python picker took
2:47.64 when reading that materialized file from FSx and 2:46.63 from a
memory-backed file, showing that Python parsing/picking—not FSx reads—dominates
the separated picker run. Both picker runs produced 1,148,541 data rows and the
exact established SHA-256:
`37ddb67c9c4908aebb7785b5e750c9acbecfa8e685991916d5cfc2f430d2c5d4`.

Production should continue streaming FastVEP directly into the picker, avoiding
the 3.1-GB intermediate. The clearest future optimization is to implement the
validated picker hierarchy in Rust or directly inside FastVEP, guarded by the
existing byte-level regression fixture.
