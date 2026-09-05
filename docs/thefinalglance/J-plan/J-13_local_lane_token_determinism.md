# J-13: local-lane token determinism — batch-invariant ops + seeded sampling on the sidecar, seed in the RRA

- Track J (determinism-steal extensions) | lever PROVABLE | grounding STRONG (source ingest + live grep) | nature BUILD | effort M | WAVE 3 (fixture legs now; live GPU legs remote-sidecar/RunPod-gated per W-10) | depends none

- Premise: ELAI's reproducibility story is the ACTION-stream (G-2 RRA replays actions, not tokens). The local lane can do better: thinking-machines batch_invariant_ops (MIT, docs/.source/batch_invariant_ops) makes mm/addmm/log_softmax/mean batch-invariant via torch.Library swaps wrapped around model.generate — framework-agnostic, bf16 NF4 covered (addmm on dequantized weights), Ampere ok; SGLang's det-mode contributes the seeded-sampling pattern (multinomial_with_seed: Gumbel noise from a seeded hash makes same (prompt,seed) -> same token even non-greedy). The seed slot ALREADY EXISTS and is hardcoded dark: crates/cli/src/runtime_client.rs:597 `seed: None` on the live streaming request (grep 2026-06-11). HONEST RESIDUE (GD-5): attention (SDPA/FA2) is NOT covered by the ops library — full cross-batch token identity needs greedy or batch_size=1 at attention; this ships as a measured opt-in mode, never a blanket guarantee.

## Steal/build
1. sidecar: vendor/pin batch_invariant_ops (alpha v0.1.0 — pin exact commit; anti-slopsquat: install from the local clone, never a name-resolved registry guess) + wrap the generate path in set_batch_invariant_mode ONCE, never nested (the context manager has a destroy-on-inner-exit bug, confirmed from source); Triton warm-up forward pass at startup; assert the bf16 log_softmax path (_half_to_float=False) with one probe.
2. sidecar: SeededGumbelProcessor as a LogitsProcessor in the EXISTING chain (server.py DoLaSteeringProcessor:786 / ETCLogitsProcessor:990 precedent) implementing multinomial_with_seed; active only when the request carries a seed.
3. rust: thread sampling_seed from ELAI_SEED through the request struct (flip runtime_client.rs:597 from None) and RECORD it in the RRA bundle (rra.rs) so replay re-issues the same seed.
4. E-26 contract note: the SGLang adapter's det-mode requirements (--enable-deterministic-inference + per-request sampling_seed + FlashInfer/FA3/Triton backend constraint) written into the adapter as a capability check, not prose.
5. determinism mode is a kill-switch-shaped config (R-f): ON for elai run --emit-rra local-lane runs by default ONLY after the GD-4 overhead bound is measured (~20% matmul overhead reported upstream; 3090 Triton fallback unmeasured) — until then opt-in via the run config with a wire-dark-allowlist row + promotion plan.
EXCLUDED: any attention-kernel patch (upstream fix is vLLM/FlexAttention-specific); LayerCast/TBIK (TP=1 single 3090 — rejected at triage); a determinism CLAIM on the cloud lane (provider-side, impossible).

## Existing ground truth (grep-confirmed 2026-06-11)
- crates/cli/src/runtime_client.rs:596-597 `stream: true, seed: None` — the live streaming request; the seed field exists, dark.
- python/elai_inference/server.py LogitsProcessor chain: DoLaSteeringProcessor:786, ETCLogitsProcessor:990 — the insertion seam for SeededGumbelProcessor.
- crates/orchestrator/src/rra.rs verify_rra:474 / verify_artifact:481 — the bundle the seed lands in.
- sidecar deploy: REMOTE 3090 only (CLAUDE.md; HARDWARE-HEAVY — flag before boot); python 3.10+ required by the ops lib (match/case), verify the remote venv at edit.
- interface map (W-3 ground truth): workflow/papers/batch-invariant-ops-source-interface.md (API, dtype coverage, NF4 verdict, nesting bug, contract stubs).

## Integration
- python/elai_inference/server.py (~120 LOC: mode wrap + warm-up + SeededGumbelProcessor + capability echo in /health) + vendored batch_invariant_ops; crates/cli/src/runtime_client.rs (~20 LOC seed threading); crates/orchestrator/src/rra.rs (~30 LOC seed field + replay re-issue).
- +LocalDeterminismEvent SSB (Pattern-2 single-variant; Kind: BatchInvariantEnabled / SeededSamplingApplied / DeterminismProbeRun / DeterminismDivergence). baseline+1 from LIVE 229/540 (ssb.rs:3066 / action_type_lock.rs:662, grep 2026-06-11); re-read at edit (W-9). Full 6-part variant discipline.
- frontend_pair: determinism status line on the existing runtime/cloud panel surface (decide at edit; W-4).
- cross_lang pair: sidecar route change + rust client change ship together (cross_lang_diff.py).

## WIRING_CONTRACT
produces_symbols:
- python/elai_inference/server.py::SeededGumbelProcessor (+ batch-invariant mode wrap + startup warm-up)
- crates/cli/src/runtime_client.rs (seed threading from ELAI_SEED config, replacing the :597 None)
- crates/orchestrator/src/rra.rs (sampling_seed recorded in the bundle; replay re-issues it)
consumes_symbols:
- docs/.source/batch_invariant_ops (vendored, pinned commit; the torch.Library swap is upstream code, not reauthored)
- crates/orchestrator/src/rra.rs::verify_rra (the existing verify leg; J-14 upgrades its verdict taxonomy)
- E-26 SGLang adapter (gains the det-mode capability check)
reachable_from:
- elai run --local (inference dispatcher -> runtime_client.rs:596 request build -> sidecar /v1/messages)  [PRODUCTION local lane]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO nested set_batch_invariant_mode (source-confirmed destroy bug; single wrap, contract-tested)
- NO determinism claim while the mode is off or the attention residue applies — capability echo must state exactly what is covered
- NO registry-resolved install of the alpha lib (pin the vendored commit; anti-slopsquat)
- NO new default-OFF ELAI_*_ENABLE without a wire-dark-allowlist row + promotion plan (R9/W-2)
- NO cloud-lane determinism claims
frontend_pair: determinism status on the existing runtime panel (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "two consecutive `elai run --local --emit-rra` fixture runs against the remote sidecar with determinism mode on and the same ELAI_SEED" (live leg remote-sidecar-gated per W-10; the deterministic fixture harness runs offline against the recorded protocol shapes)
- expect_ssb: "BatchInvariantEnabled + SeededSamplingApplied rows on boot/first-request; DeterminismProbeRun with identical blake3 over the two runs' token streams"
- expect_artifact: "both RRA bundles carry the same sampling_seed and token-stream hash; `elai replay --verify` matches. NEGATIVE (R2): mode off -> the capability echo says so and DeterminismProbeRun records non-identical streams as DeterminismDivergence (measured, not hidden); a seed-less request with mode on FAILS the contract test."

## Grounding
- batch_invariant_ops source ingest: workflow/papers/batch-invariant-ops-source-interface.md (agent p2p-ingest-4; dtype/NF4/nesting verdicts from source).
- SGLang det-mode: agent det-repos-S1 (docs.sglang.io advanced_features + lmsys blog, fetched 2026-06-11): kernel swaps + multinomial_with_seed + 24-46% overhead figure [DESIGN, never claimed].
- steal ledger: workflow/reasoning/steal-determinism-repos-2026-06-11.md Tier-1 item 4.
- live anchors: runtime_client.rs:596-597, server.py:786/990, rra.rs:474 (grep 2026-06-11).

## COMPOSES-WITH
G-2 RRA (the seed makes local replay token-faithful), G-10 model-drift canary (a deterministic lane makes drift detection exact), E-26 SGLang adapter, E-2/E-28 sidecar perf work (overhead interplay measured per HD-1), H-50 context policy (unchanged). Restates none.

## Open questions / risks
- 3090 Triton-fallback overhead is UNMEASURED upstream; the GD-4 bound (>=3 real runs) decides default-ON vs opt-in.
- NF4 interplay verdict is source-reasoned, not yet probe-confirmed on the live box — the startup probe (one bf16 mm + log_softmax assert) is the first live checkbox.
- greedy-decode already dominates ELAI local runs; the seeded-Gumbel leg mostly buys reproducible NON-greedy sampling — worth shipping for RRA completeness, but the batch-invariance leg is the bigger win.

## Next
Makes the local lane's RRA token-faithful (T1/T2 in J-14's tier language); J-14's verify upgrade then has something to grade PASS.

End of J-13.
