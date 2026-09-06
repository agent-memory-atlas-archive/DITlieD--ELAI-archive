H-46 decisions — HGM clade-metaproductivity parent selection
Source plan: docs/thefinalglance/H-plan/H-46_clade_metaproductivity_selection.md
Session: s-10612  Date: 2026-06-11

DECISION 1: Metaproductivity-Performance Mismatch (HGM 2510.21614 grounded)
The DGM config archive selected parents by immediate sigmoid score. HGM (ICLR 2026 oral,
metauto-ai/HGM, verified WebSearch 2026-06-08) shows that a node's immediate score has a
surprisingly weak correlation with its long-term potential to generate better descendants.
The Clade-Metaproductivity (CMP) key aggregates the measured fitness improvements of all
direct descendants, biasing selection toward productive lineages rather than stagnant peaks.
Paper result: HGM achieves SWE-Verified-60 56.7 vs DGM 53.3, Polyglot 30.5 vs 27.1, at
~2.38x less compute (517 vs 1,231 CPU-hrs). These are PAPER NUMBERS for cloud flagships; not
yet measured for ELAI's abliterated 7B local lane (HD-1, reporting the paper's numbers as the
paper's). No ELAI-measured lift should be claimed until Tier-2 multi-generation measurement
completes (W-10 separately gated checkbox).

DECISION 2: CMP aggregate = mean delta over direct descendants (Tier-1)
HGM aggregates over the full descendant clade. For Tier-1 we use direct children only (first-
generation descendants). Mean delta (not sum) normalizes for clade size so large clades don't
dominate simply by having more children. Sum is kept as a named CmpAggregate variant for
future ablation. Tier-2 multi-generation lineage measurement is a separately gated checkbox
per W-10 -- the fail-closed done-gate is Tier-1.

DECISION 3: ImmediateScoreFallback for clade_size == 0 (cold-start guard)
A new node with no measured descendants has no CMP signal. Without a fallback, new variants
would get weight = immediate_sigmoid + 0_delta = sigmoid (same as before), which is fine, but
the key would still say CladeMetaproductivity on a signal that is identical to the fallback.
To be honest about the measurement, clade_size == 0 explicitly uses ImmediateScoreFallback.
The HGM benefit accumulates across generations; early in a run, most nodes are leaves and the
fallback dominates. This is expected (CMP only overrides once a lineage exists).

DECISION 4: CMP formula = max(1e-9, immediate_score + mean_delta)
The floor at 1e-9 preserves the DGM invariant that all weights are strictly positive (required
by the CDF sampler). A strongly regressing clade (large negative delta) cannot zero out a
candidate's selection probability -- it can only reduce it significantly. immediate_score is the
floor anchor: even a fully regressing clade still gets the node's own sigmoid weight as the
minimum CMP contribution.

DECISION 5: select_parent_with_cmp is a new method; select_parent unchanged
GD-1: do not rewrite the archive. The original select_parent and select_parent_with paths are
preserved for callers that don't yet pass fitness deltas. select_parent_with_cmp is the new
production path wired in darwin_admit_loop. sample_variant_weights is a new helper that accepts
a pre-computed weight slice, reusing the same CDF logic.

DECISION 6: R2 broken arm = force_immediate_score=true
The falsifiability contract (R2) requires a broken arm that selects A (dead-end high-immediate)
over B (productive low-immediate). force_immediate_score=true is the parameter -- production
code NEVER passes true. The contract test (c) exercises this arm and confirms A wins, proving
CMP is load-bearing, not a decorative wrapper over sigmoid.

DECISION 7: VD-1 compliance confirmed
CMP aggregates fitness_deltas (H-43 execution-grounded pass-rate signal). The contract test (i)
performs a static grep on the production source to verify no reference to blake3::hash(config_blob)
or derived_alpha in non-comment lines. The old GAP-1 byte-hash path is explicitly banned.

HONEST RESIDUE
- CMP is a measured proxy for potential, bounded by the fitness signal it aggregates (VD-1).
  If H-43's pass-rate oracle is miscalibrated, CMP faithfully amplifies the miscalibration.
- The Solver-Verifier gap (2507.00075) applies: a verifier that can be gamed defeats the CMP
  signal. H-47 eval-firewall is the backstop for this.
- ELAI-lane CMP benefit is unproven until Tier-2 multi-generation measurement on the abliterated
  7B local lane (3 runs minimum per GD-4). The 2.38x compute reduction from HGM is for cloud
  flagships only; do not claim it for ELAI (HD-1).
- Direct children only (Tier-1) is a structural undercount of the clade. Multi-generation
  aggregation (Tier-2) is expected to surface a stronger signal once lineages accumulate.
