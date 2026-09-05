# J-19: vLLM-V1 as an ADDITIVE alternative serving backend — the swap-out decision stays with the user

- Track J (pillar-SOTA recheck extensions) | lever EFFICIENT | grounding STRONG (vLLM docs VERIFIED: /v1/messages, batch-invariant SM8.6, AWQ-Marlin, prefix cache) | nature BUILD (additive, NOT a removal) | effort M | WAVE 3 (fixture legs now; live GPU legs remote-sidecar/RunPod-gated, W-10) | depends none (rides B-37 RuntimeAdapter)

- Premise: two deep-research reports recommend REPLACING the HF Transformers serving layer (the lone REPLACE verdict of the whole recheck). One says SGLang, the other vLLM V1. Per CLAUDE.md NO removal/simplification of a shipped component without explicit user confirmation — so J-19 does NOT remove HF. It ADDS vLLM V1 as one more RuntimeBackend behind the EXISTING RuntimeAdapter trait (we ship 11 backends, B-37), measures it against the live HF sidecar, and surfaces the comparison so the USER makes the swap-out call. The win is real and confirmed; the decision is the user's.

- HONEST CONSTRAINT (the DoLa blocker, verify-p5): our custom LogitsProcessor chain (DoLa layer-contrast, ETC entropy, prompt-lookup ngram, XGrammar, planned seeded-Gumbel) does NOT all port. DoLa registers PyTorch forward hooks on model.model.layers — vLLM/SGLang logits processors see FINAL logits only, so DoLa is STRUCTURALLY non-portable to either engine. This is acceptable: DoLa is ALREADY disabled on the live remote (--no-dola boot flag) and our own baseline (metrics/baseline_findings) found it marginal on small models. The vLLM lane ships WITHOUT DoLa; the HF sidecar retains it for any DoLa-dependent run.

## Steal/build (logits-chain PORTABILITY MAP — the load-bearing engineering)
| component        | vLLM V1            | SGLang             | keep on HF |
| DoLa layer-contrast | NOT-PORTABLE (forward hooks, no surface) | NOT-PORTABLE | yes (already --no-dola live) |
| XGrammar masking | NATIVE (default backend) | NATIVE (default) | n/a |
| ETC entropy      | portable-as-custom (final-logits processor) | portable-as-custom | n/a |
| prompt-lookup ngram | NOT-PORTABLE to V1 (V0 fallback, closed not-planned) | check at edit | yes |
| seeded-Gumbel (J-13) | portable-as-custom | portable-as-custom (multinomial_with_seed native) | n/a |

1. add RuntimeBackend::VllmV1 to the RuntimeAdapter (B-37) — vLLM serves /v1/messages NATIVELY (vLLM docs claude_code integration page, VERIFIED), so ZERO Rust client change (the existing runtime_client.rs request shape works as-is). Behind a backend-selection flag, never default.
2. AWQ-Marlin quant path: offline AutoAWQ checkpoint of the abliterated Qwen2.5-Coder-7B (calibration data needed — document the calibration corpus), replacing runtime NF4 dequant for the vLLM lane. AWQ+Marlin VERIFIED on SM 8.6 (RTX 3090). This is E-27's planned-but-never-built AWQ checkpoint, carried on the vLLM leg.
3. enable vLLM automatic prefix caching (cross-REQUEST KV reuse, VERIFIED) + VLLM_BATCH_INVARIANT=1 (beta, SM>=8.0, satisfies J-13's token-determinism requirement) — the two headline wins the HF sidecar STRUCTURALLY lacks (HF has no APC, GIL-serialized, no batch invariance).
4. port ETC + seeded-Gumbel as vLLM custom logits processors (the portable subset); leave DoLa + prompt-lookup HF-only.
5. MEASURE: a side-by-side fixture (same prompts, same seed) HF-sidecar vs vLLM-lane on throughput, TTFT, prefix-cache hit-rate, token determinism — SURFACE the comparison (the believe-the-number instrument, GD-2). The swap-out is a SEPARATE user-confirmed decision, never auto.
EXCLUDED: removing the HF sidecar (user-confirmation-gated, not this unit); SGLang as the target (the report divergence + SGLang's CVE-2026-7304 RCE on the dill.loads custom-logits path + no native /v1/messages make vLLM the safer lane — recorded, SGLang stays the E-26 adapter); "Qwen3-Coder 14B" as a model upgrade (REFUTED — does not exist, ledger Tier-3); TensorRT-LLM / candle / mistral.rs (rejected for this single-user config, verify-p5).

## Existing ground truth (grep-confirmed; re-grep at edit, W-7/W-9)
- MULTI-RUNTIME BACKEND [WIRED] RuntimeAdapter trait, 11 backends, PerRoleRuntime, resolve_backend (feature map 1.9; B-37, phase-4M) — the trait vLLM plugs into; live cloud Sonnet 4.6 PROVEN.
- PYTHON SIDECAR [WIRED] server.py model.generate() + LogitsProcessor chain (DoLaSteeringProcessor:786, ETCLogitsProcessor:990) — the chain whose portability is mapped; PrefixKVCache:1819 BYPASSED on the streaming path (the NO-OP local caching vLLM's APC fixes).
- runtime_client.rs:596-597 stream:true seed:None — the request shape that works unchanged against vLLM's /v1/messages; the seed slot J-13 threads.
- E-27 QUANTIZATION [PARTIAL]: AWQ+Marlin 4.4x planned, checkpoint never built — J-19 builds it on the vLLM leg.
- E-26 SGLANG [PARTIAL]: the SGLang adapter stays; J-19 does not touch it (vLLM is the additive lane).
- J-13 LocalDeterminismEvent: vLLM's VLLM_BATCH_INVARIANT is the engine-native path to the same token-determinism J-13 vendors for the HF lane — compose, the two lanes share the RRA seed contract.

## Integration
- crates/providers/src/runtime/ new RuntimeBackend::VllmV1 adapter (~150 LOC: backend registration + the AWQ/APC/batch-invariant launch config + the portable logits-processor subset wiring; reuses the /v1/messages client unchanged).
- offline tooling: an AutoAWQ checkpoint build script + the calibration corpus (documented, anti-slopsquat the autoawq dep at edit).
- PREFER-REUSE SSB: vLLM is another backend -> reuse B-37 RuntimeBackendSelected / RuntimePanel rows (net 0). A vLLM-specific cache/determinism metric rides J-13's LocalDeterminismEvent + H-9's CloudSpend-style cost rows. Decide at edit; +0..+1 only if a genuinely new metric has no home.
- frontend_pair: the HF-vs-vLLM comparison on the existing RuntimePanel / CloudPanel surface (B-37/H-9) — the believe-the-number table; cross_lang_diff if the sidecar config changes ship with a Rust client change (W-4).

## WIRING_CONTRACT
produces_symbols:
- crates/providers/src/runtime RuntimeBackend::VllmV1 (additive backend on the B-37 trait)
- the AWQ-Marlin checkpoint build tooling (offline; E-27's never-built checkpoint)
- the portable logits-processor subset on the vLLM lane (ETC + seeded-Gumbel; DoLa/prompt-lookup excluded by construction)
consumes_symbols:
- B-37 RuntimeAdapter / resolve_backend (the trait; NOT reauthored)
- runtime_client.rs /v1/messages request shape (UNCHANGED — vLLM native)
- J-13 RRA seed contract (the determinism lane the vLLM batch-invariant path satisfies natively)
reachable_from:
- a real `elai run --local` with the backend flag set to vllm-v1 (inference dispatcher -> RuntimeAdapter::VllmV1 -> vLLM /v1/messages)  [PRODUCTION local lane, remote-GPU-gated W-10]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO removal of the HF sidecar backend (additive only; the swap-out is a user-confirmed separate decision)
- NO DoLa-on-vLLM claim (structurally impossible; the capability echo must say DoLa is HF-only)
- NO registry-resolved autoawq / vllm install (pin versions; anti-slopsquat at edit)
- NO determinism/cache claim the lane cannot deliver (vLLM APC + batch-invariant are MEASURED per config, never a literature multiple; HD-1)
- NO new default-ON backend (vLLM behind a flag until the side-by-side bound is measured; W-2/R9)
frontend_pair: HF-vs-vLLM comparison on RuntimePanel/CloudPanel (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "two `elai run --local` fixture runs against the remote 3090, one on the HF sidecar backend, one on the vLLM-V1 backend, same prompts + same ELAI_SEED (live legs remote-sidecar/RunPod-gated W-10; the deterministic fixture harness runs offline against recorded protocol shapes)"
- expect_ssb: "RuntimeBackendSelected=VllmV1 on the vLLM run; the side-by-side metric rows (throughput, prefix-cache hit-rate, token-stream blake3) for both lanes"
- expect_artifact: "the vLLM lane shows cross-request prefix-cache reuse (HF shows none — the structural win); VLLM_BATCH_INVARIANT yields an identical token-stream hash across the two vLLM runs; the comparison TABLE is surfaced for the user's swap decision. NEGATIVE (R2): the vLLM lane with DoLa requested FAILS the capability check (DoLa unavailable, stated not hidden); an AWQ checkpoint built without calibration data FAILS its build assert; the lane claiming a cache hit it cannot deliver FAILS the measured-cache contract."

## Grounding
- vLLM V1 (VERIFIED via verify-p5, vLLM docs): native /v1/messages (claude_code integration page), VLLM_BATCH_INVARIANT beta SM>=8.0, AWQ+Marlin on SM 8.6, cross-request automatic prefix caching, XGrammar built-in. DoLa NOT-portable (custom_logitsprocs page: global-init-only, final-logits-only; hidden-states RFC #12249 single-layer no multi-layer contrast). prompt-lookup ngram NOT-portable to V1 (#16883 closed not-planned). SGLang CVE-2026-7304 (dill.loads RCE on custom-logits path) — the safety reason vLLM is the chosen lane.
- "Qwen3-Coder 14B" REFUTED (verify-p5: QwenLM/Qwen3-Coder has no 14B dense variant; ledger Tier-3).
- E-27 AWQ plan (feature map 1.9 PARTIAL, checkpoint never built); B-37 RuntimeAdapter [WIRED]. live SSB 231/542 (grep 2026-06-12).

## COMPOSES-WITH
B-37 multi-runtime backend (vLLM is backend #12 on the trait), J-13 local token determinism (vLLM batch-invariant = the engine-native path to the same RRA seed contract J-13 vendors for HF), E-27 quantization (builds the AWQ checkpoint), H-1/H-2 prompt caching (the local-lane APC is the analog of the cloud cache discipline), E-26 SGLang (untouched; vLLM is the additive lane). Restates none.

## Open questions / risks
- DoLa value: if any production run still depends on DoLa, that run stays on HF — document which roles/lanes are DoLa-dependent (likely none, given --no-dola is the live default) so the vLLM lane's coverage is honest.
- AWQ calibration: an AWQ checkpoint of an ABLITERATED finetune needs representative calibration data (code prompts) — a bad corpus degrades quality silently; the build assert + a quality probe vs NF4 is the first checkbox.
- the swap decision: J-19 deliberately stops at the measured comparison. The user decides whether vLLM REPLACES HF. Do not author the removal in this unit.

## Next
The measured HF-vs-vLLM table is the artifact the user uses to decide the serving-engine swap. J-13's determinism work and this lane's batch-invariant path converge on the same RRA seed contract — local reproducibility is the shared payoff.

End of J-19.
