# N-1: measurement spine as ONE subsystem — unify the ten benches + corpus feeders + one calibration sink + one label fold + a bootstrap pack

- Track: TIER 1 (CRITICAL) | lever: the-selling-number | finding: W-1+W-11+W-23 | grounding: STRONG | nature: ACTIVATION | effort: XL | WAVE 1 | [KEYSTONE][BLOCKING] | depends: none

- Premise: ~30 features across 6 letters terminate their promotion path at "ELAI-measured number" — the single highest fan-in dependency in the corpus — yet measurement is catalogued as scattered entries, not a first-class subsystem with its own index/owner/sequencing. If the corpus+runner substrate slips, the entire enforcement layer stays advisory forever and the GTM (believe-the-number) has no number. N-1 declares the spine. Everything else in the corpus that claims a number is measured THROUGH this (ND-4).

## Steal/build
Declare ONE measurement spine = (1) the corpus: J-56 problem corpus with L-20 commit-mined supply + J-110 mutators as FEEDERS (not sibling benches); (2) ONE runner family (the K-15/I-27 shape, already the pairwise convention across the ten benches); (3) ONE calibration SINK: the W-5 ModelCapabilityProfile (I-51 models.toml) — no router reads a knee from anywhere else; (4) ONE outcome-label fold: generalize J-93-F (deterministic non-LLM rule labels from the SSB outcome stream) from rules-only to the six promotion loops (W-11); (5) a shipped BOOTSTRAP calibration pack (B-42-style default artifacts, provenance=BOOTSTRAP, hard-superseded by local measurement via F-5/G-10 invalidation, W-23 — the cold-start fix so a fresh install is not advisory-only for weeks).
The ten benches it UNIFIES (bind to the one runner + one sink, do NOT replace): G-6 SAFEDELTA, J-34 context-knee, K-15 reasoning-knee, J-107 longcot, L-10 hallucination-delta, K-18 case-reuse, I-38 QDELTA, I-27 convergence, L-20 supply, J-110 timeliness (+ D-19..D-25 bench/validation suites).
EXCLUDED (not this unit): inventing any new bench (N-1 unifies, it does not add an 11th); any new corpus family (J-56/E-54/L-20 are the corpus); a second calibration store (that is the ND-4 violation this unit exists to prevent).

## Existing ground truth (PLAN-asserted from ELAI-FEATURE-MAP + interlace doc; executor re-greps per W-7)
- the ten bench units exist as PLANNED entries across G/J/K/L/I; each "reuses the runner" pairwise (feature-map bench group).
- J-93-F supplies deterministic outcome labels to RULES only today (I-plan / feature-map).
- ModelCapabilityProfile (I-51 models.toml) already gains fill_degradation_curve (J-34), tool_call_adherence (J-51), reasoning-knee (K-15), longcot arm (J-107); I-23 flags "unified single-impl calibration, three consumers, semantics cannot drift" — N-1 is that binding at bench grain.
- B-42 ships ~12 default cosmos artifacts as the in-corpus precedent for a shipped default pack (memory project_b42_shipped).
- D-30 names cold-start for ONE gate (UniCR N=500) and DEFERS it; the same hole sits under D-16 tau / J-34/K-15 knees / J-51 floors / L-11 OQ / 3F FP bounds / H-43 baselines (W-23).

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1, in-process, fail-closed): the spine INDEX (the one runner + one sink + the label fold binding) + the bootstrap pack + the ND-4 enforcement (a bench writing a knee to any store other than the profile FAILS). The label fold generalized to the six loops.
- DEFER (Tier-2, network/RunPod-gated): the actual SAFEDELTA live number (G-6, already gated behind F-2b keystone + RunPod); N-1 ships the SUBSTRATE + a deterministic fixture, the live number stays G-6's deferred checkbox.

## Integration
- crates/knowledge: the corpus + runner surface. crates/orchestrator: ModelCapabilityProfile sink + the J-93-F label fold generalization. CLI: the `elai bench *` seams as the real trigger.
- frontend_pair: a MeasurementPanel tile fed by the spine index, OR N/A justified if G-5 cockpit already surfaces the profile (confirm at authoring).

## WIRING_CONTRACT
produces_symbols:
  - crates/knowledge/src/measurement/spine.rs — the spine index (corpus + runner + sink binding) [PLAN-asserted path]
  - crates/orchestrator/src/measurement/bootstrap_pack.rs — BOOTSTRAP default artifacts + the supersession rule
  - the generalized label fold (J-93-F extended to six loops)
consumes_symbols:
  - crates/knowledge J-56/E-54/L-20 corpus (the one corpus family)
  - crates/orchestrator ModelCapabilityProfile / I-51 models.toml (the ONE sink)
  - J-93-F outcome-label fold (extend, do not re-author)
  - the K-15/I-27 runner shape (the ONE runner)
reachable_from:
  - CLI `elai bench <family>` -> runner -> corpus row + profile curve write  [PRODUCTION bench seam]
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO second calibration store (any knee written outside the profile)
  - NO new 11th bench (unify only)
  - NO paper-constant knee claimed as ELAI-measured (bootstrap defaults carry provenance=BOOTSTRAP, ID-5)
  - NO .unwrap()/.expect() on prod paths; blake3 only
frontend_pair: MeasurementPanel tile OR "N/A — G-5 cockpit surfaces the profile; justified" (decide at authoring)
observable_effect:
  trigger: "elai bench <any-family>"
  expect_ssb: "a corpus row + (on bootstrap replacement) +calibration_bootstrap_superseded(Prefilled|LocallySuperseded|DriftInvalidated)"
  expect_artifact: "a calibration curve written into the ONE ModelCapabilityProfile; NEGATIVE: a bench writing a knee to any other store must FAIL the ND-4 check"

## Grounding
- workflow/reasoning/elai-weakpoints-interlace-2026-07-04.md W-1 (evidence: the ten benches + the fan-in), W-11 (outcome-label supply under four promotion systems), W-23 (cold-start), TIER-1 synthesis.
- memory project_b42_shipped (the default-pack precedent), reference_b40_papers/reference_b42_papers (arXiv IDs — re-verify at authoring per house rule; 2 bad cites shipped before this rule).
- ID-5 (ELAI-measured, never paper constants) satisfied by the BOOTSTRAP provenance label.

## COMPOSES-WITH
J-56/E-54/L-20 corpus; K-15/I-27 runner; ModelCapabilityProfile (I-51); J-93-F label fold; B-42 default-pack shape; F-5/G-10 invalidation. Re-implements NONE (GD-1). ND-4 makes this the single sink every other N-N and every J..M number routes through.

End of N-1.
