# E-16: VLM image validation — AXTree adapter + anti-sycophancy + RCE convergence
Objective: harden the SHIPPED B-24 Media Validator (Phase 4I, s-59964) with the four highest-leverage 2025-2026 findings the original spec did not encode — AXTree-YAML as a deterministic Tier 1 web adapter, anti-sycophancy prompt protocol for Tier 2, swap-order pairwise eval, and a true RCE loop with LPIPS-delta convergence + iteration cap. This is a DELTA on top of an already-running module; it does not re-author tier1/tier2/feedback_loop, it adds adapters and replaces prompt/eval surfaces.
Estimated effort: 2 sessions, ~10-12 h
Pre-requisites: B-24 SHIPPED (s-59964), C-7 ReviewWindow PLANNED (frontend reuse for HumanReview reduction surface), B-37 SHIPPED (RuntimeAdapter for SGLang/InternVL Mode B).
Parallel-safe with: E-17 (knowledge), E-18 (tool registry), C-7, C-8.
Source report: workflow/improvement-sweep-2026-05-19/reports/16-vlm-image-validation.md

## Context

B-24 shipped Phase 4I as a two-tier Media Validator: Tier 1 deterministic checks (pHash/SSIM/LPIPS ONNX CPU EP, axe-core, Tesseract, ffmpeg/ImageMagick) and Tier 2 VLM advisory (Qwen3.5-VL-30B-A3B via ollama Mode A, InternVL3.5-38B via SGLang Mode B) with an RCE feedback loop, 71 lib + 21 RCE + 19 dashboard + 2 e2e tests passing. The shape is right (literature converges on symbolic-first + VLM-as-tiebreaker per arxiv:2408.11261 and arxiv:2406.07791), but four documented failure modes are not yet wired:

1. VLM-as-judge is provably sycophantic — Qwen-VL, CogVLM, InternVL, LLaVA, mPLUG agree with leading user prompts and override correct visual evidence (arxiv:2408.11261 MM-SY benchmark; arxiv:2509.21979 EchoBench). If the executor tells the validator "this is the TODO app you asked for", the VLM will rationalise away genuine defects.
2. Position bias shifts pairwise accuracy >10pp from swapped option order alone, across 15 judges / ~150K instances (arxiv:2406.07791, AACL-IJCNLP 2025). The current Veto logic does not enforce swap-order agreement.
3. AXTree YAML structural snapshot (`page.ariaSnapshot()`, Playwright v1.59+) catches ~40% of WCAG issues axe-core misses and is deterministic + sub-100ms. ELAI's current Tier 1 web adapter does not produce one.
4. The RCE loop in `feedback_loop.rs` has no LPIPS-delta convergence detector or hard iteration cap; arxiv:2604.05839 RRVF and arxiv:2507.20766 both motivate convergence-gated iteration with cap 3-5.

Cited grounding: arxiv:2408.11261 (VLM sycophancy), arxiv:2509.21979 (MM-SY medical sycophancy), arxiv:2406.07791 (position bias AACL-IJCNLP 2025), arxiv:2511.21631 (Qwen3-VL tech report; Nov 2025 release confirms 30B-A3B sweet spot for single 24GB GPU), arxiv:2508.18265 (InternVL3.5 Visual Resolution Router), arxiv:2604.05839 + arxiv:2507.20766 (RCE / RRVF convergence loop), playwright.dev v1.59 aria-snapshots, arxiv:2411.15100 (XGrammar for constrained Tier 2 JSON).

## Existing state (B/C/D coverage)

- B-24 (DONE s-59964): Tier1+Tier2+RCE shipped at `crates/orchestrator/src/media_validator/{mod.rs, tier1.rs, feedback_loop.rs, render_adapters/{web.rs, blender.rs, comfyui.rs, ui_screenshot.rs}}`. DELTA only — do NOT rewrite tier1/feedback_loop, ADD adapters and SWAP prompts.
- B-24 dashboard: `crates/dashboard-v2/src-tauri/src/media_validation.rs` exposes `tier1`/`tier2` fields per row. EXTENSION needed: AXTree YAML field + sycophancy_flag column.
- B-24 cloud_mode_live test currently `#[ignore]`'d (claude-sonnet-4-5-20250929). E-16 un-ignores it under a feature flag once anti-sycophancy prompt protocol passes.
- B-37 (SHIPPED fd5c2e5): `RuntimeAdapter` trait used by Tier 2 Mode B. EXTENSION — InternVL3.5 second-opinion judge slots into existing adapter table; no new runtime.
- C-7 ReviewWindow (PLANNED): HumanReview surface; E-16 disagreement-gated escalation feeds C-7's queue.
- D-13 Adversary role (PLANNED): cross-family adversarial verifier is a sibling design to E-16's second-opinion VLM gate; they share the "cross-family" axis. E-16 does NOT pre-empt D-13; the second-opinion is intra-VLM (Qwen3-VL vs InternVL3.5), not cross-mode.

## Reads (mandatory before coding)

- workflow/improvement-sweep-2026-05-19/reports/16-vlm-image-validation.md (full)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\media_validator\mod.rs (lines 569-1100 cover request handling, human_review_required logging at :921, started_at metadata at :1034/:1094)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\media_validator\tier1.rs (existing deterministic adapters; AXTree adapter goes alongside ImageMagick at :208 area)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\media_validator\feedback_loop.rs (lines 461-554; existing fail/pass pathways for tier1 — convergence detector inserts at the loop boundary)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\media_validator\render_adapters\web.rs (Playwright integration — `page.ariaSnapshot()` call goes here)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\media_validation.rs (row schema for tier1/tier2 surfacing)
- arxiv:2408.11261 PDF + arxiv:2406.07791 PDF (anti-sycophancy + swap-order rationale; cite in tests)
- playwright.dev/docs/aria-snapshots (page.ariaSnapshot() v1.59+ API contract)

## Tasks

1. Add `Tier1::AXTreeAdapter` to `tier1.rs`. The adapter runs after page render in the web adapter path (`render_adapters/web.rs`); it calls `page.ariaSnapshot()` and stores the returned YAML in the existing tier1 result struct. Compare against a stored AXTree snapshot per render task ID; emit `axtree_mismatch` if structural drift detected. Determinism budget: <100ms; CPU only. The adapter is web-only — Blender/FFmpeg/ComfyUI adapters skip it.

2. Add an anti-sycophancy prompt protocol module at `crates/orchestrator/src/media_validator/sycophancy.rs`. Two functions: `neutralize_query(executor_intent: &str) -> NeutralPrompt` (strips "this is the X you asked for", "does this match the spec", "verify the title says"); `contrastive_decode(neutral: &NeutralPrompt, leading: &NeutralPrompt, vlm_client: &dyn VlmJudge) -> SycophancyVerdict { agree, divergent, divergence_score }`. Tier 2 inference paths in `mod.rs` use the neutral prompt for the primary judge call; the leading variant is only used in the contrastive comparison test.

3. Add swap-order pairwise eval to the Veto pathway. When Tier 2 compares two candidate renders (before/after fix), the adapter calls the VLM twice — option ordering [A, B] then [B, A] — and only accepts agreement. Disagreement routes to HumanReview (existing surface at `mod.rs:921`). Add `swap_order_disagreement` SSB row.

4. Replace `feedback_loop.rs` iteration logic with an LPIPS-delta convergence detector. Inputs: previous-iteration screenshot + current-iteration screenshot. Convergence = LPIPS(prev, curr) < 0.02 OR VLM critic returns "no actionable diffs" OR iteration count == 5 (hard cap). Use the existing CPU-EP LPIPS ONNX runner; no new model file. The cap is per arxiv:2604.05839 anti-pattern "no iteration cap".

5. Wire InternVL3.5-38B as a second-opinion judge via existing `RuntimeAdapter`. Trigger condition: primary Tier 2 (Qwen3-VL) returns a JSON verdict with low logprob OR sycophancy_score above threshold (from task 2). Both judges must agree before passing; disagreement → HumanReview. Reuses B-37's `[agents.media_validator.runtime]` schema; add a `second_opinion_backend` key.

6. Enforce XGrammar JSON-schema constrained decoding for the `VlmValidationOutput` struct. Verify the SGLang request path already passes the schema (B-24 noted "constrained decoding"); add an integration test that fuzz-feeds the schema and asserts mask-emitted compliance. If SGLang fallback path doesn't honour the schema, surface as `vlm_schema_unconstrained` SSB row.

7. Add a `sycophancy_probe` mode that, for every Tier 2 verdict on a user-aligned claim, re-runs the prompt with inverted framing ("this is NOT the X you asked for; identify defects"). Converging outputs = robust; divergent = `sycophancy_flag` row + quarantine to HumanReview. This is the structural defense; D-14 will extend it to non-VLM modalities.

8. Surface AXTree, sycophancy_flag, swap_order_disagreement, and convergence_history in the dashboard's `media_validation.rs` row schema. Extend `MediaValidationDrawer.svelte` (V2 path) with three new fields. Do not collapse them into existing tier1/tier2 fields — auditor needs separability.

9. Un-ignore `cloud_mode_live` after tasks 2 + 3 + 7 land. Add a separate `#[ignore = "needs-real-vlm"]` test guard so CI doesn't accidentally fire the cloud path.

10. Document the AXTree-stability open question (report §Open questions Q4): log YAML diff rate per run across N=20 identical renders; if drift exceeds 1%, fall back to axe-core-only gate.

## WIRING_CONTRACT

produces_symbols:
  - orchestrator::media_validator::tier1::AXTreeAdapter (new)
  - orchestrator::media_validator::sycophancy::neutralize_query (new)
  - orchestrator::media_validator::sycophancy::contrastive_decode (new)
  - orchestrator::media_validator::feedback_loop::lpips_delta_converged (new fn)
  - orchestrator::media_validator::SycophancyVerdict (new struct)
  - orchestrator::media_validator::AxTreeResult (new struct)
  - SsbActionType::SycophancyFlag (new variant)
  - SsbActionType::SwapOrderDisagreement (new variant)
  - SsbActionType::AxtreeMismatch (new variant)
  - SsbActionType::VlmSchemaUnconstrained (new variant)
consumes_symbols:
  - orchestrator::media_validator::mod::validate (existing entry point)
  - orchestrator::media_validator::render_adapters::web::Web::render (extended to call ariaSnapshot)
  - providers::runtime::RuntimeAdapter (B-37) for second-opinion InternVL invocation
  - dashboard-v2::media_validation::row_schema (extended fields)
reachable_from:
  - crates/orchestrator/src/task_runner.rs::media_validation_phase
  - crates/dashboard-v2/src-tauri/src/media_validation.rs::get_media_validation_recent
stub_ban:
  - no "TODO" / "unimplemented" in sycophancy.rs
  - no `#[ignore]` on cloud_mode_live after tasks 2/3/7
  - no hard-coded iteration count > 5 in feedback_loop
  - no pixel-diff fallback as primary gate (Percy 2025 abandoned this; AXTree+VLM is canonical)
frontend_pair: crates/dashboard-v2/src/lib/components/MediaValidationDrawer.svelte
observable_effect:
  trigger: "elai /task --mode cloud 'build a TODO app webpage with add/delete/persist'"
  expect_ssb: "axtree_mismatch absent on clean run; sycophancy_probe rows present; convergence_history shows ≤5 iterations"
  expect_artifact: ".elai_cc/sessions/{id}/media-validator/{task}/axtree.yaml exists; LPIPS-delta log shows monotonic decrease"

## DETERMINISTIC_CHECK

```bash
# 1. AXTree adapter wired
rg "AXTreeAdapter|ariaSnapshot" .elai_cc/crates/orchestrator --type rust
# Expected: ≥4 hits (adapter struct, web::render call, tier1 dispatch, test).

# 2. Sycophancy module exists with neutralize_query + contrastive_decode
rg "fn neutralize_query|fn contrastive_decode" .elai_cc/crates/orchestrator/src/media_validator/sycophancy.rs
# Expected: 2 hits.

# 3. Iteration cap + LPIPS-delta convergence
rg "MAX_RCE_ITERATIONS|lpips_delta_converged" .elai_cc/crates/orchestrator/src/media_validator/feedback_loop.rs
# Expected: const MAX_RCE_ITERATIONS = 5 + at least 1 use in loop guard.

# 4. New SSB action types registered
rg "SycophancyFlag|SwapOrderDisagreement|AxtreeMismatch|VlmSchemaUnconstrained" .elai_cc/crates/orchestrator/src/ssb.rs
# Expected: enum variants + as_sql_str arms.

# 5. cloud_mode_live un-ignored
rg "ignore.*cloud_mode_live" .elai_cc/crates/orchestrator/src/media_validator/
# Expected: 0 hits in test attributes (only as commented-out historical reference if any).

# 6. e2e smoke under cloud:
elai /task --mode cloud "build a TODO app webpage with add/delete/persist"
# Inspect SSB after run:
sqlite3 .elai_cc/.elai_cc/ssb.db "SELECT action_type, COUNT(*) FROM ssb_events WHERE action_type IN ('sycophancy_probe','swap_order_disagreement','axtree_mismatch','rce_iteration') GROUP BY action_type;"
# Expected: sycophancy_probe>=1, rce_iteration<=5, axtree_mismatch=0 on a clean spec.
```

## ACCEPTANCE

- [ ] AXTree-YAML produced for every web render adapter call; stored at `.elai_cc/sessions/{id}/media-validator/{task}/axtree.yaml`
- [ ] Neutral-framing prompt used for primary Tier 2 calls (no executor-intent leakage)
- [ ] Swap-order pairwise eval enforced; disagreement → HumanReview
- [ ] RCE loop hard-capped at 5 iterations + LPIPS-delta convergence detector wired
- [ ] InternVL3.5 second-opinion path activated on low-logprob or sycophancy>=threshold
- [ ] XGrammar schema-mask enforced for Tier 2 JSON; violation → `vlm_schema_unconstrained` row
- [ ] Sycophancy probe inverted-framing re-run; divergent verdicts quarantined
- [ ] cloud_mode_live test un-ignored and passes on Sonnet 4.5 (or current default cloud model)
- [ ] AXTree-stability open Q4 measured: ≤1% YAML diff across 20 identical renders
- [ ] Zero new `.unwrap()` or `.expect()` in production paths per CLAUDE.md anti-unwrap rule

## Tests to add

- `crates/orchestrator/tests/axtree_adapter_contract.rs`: round-trip YAML, deterministic re-render diff < 1%, ariaSnapshot timing < 100ms.
- `crates/orchestrator/tests/sycophancy_neutralization.rs`: fuzz-feed 50 leading prompts; verify neutralization preserves semantic ask while stripping intent leakage.
- `crates/orchestrator/tests/swap_order_pairwise.rs`: synthetic A/B render pair; assert agreement-required pathway; inject disagreement → assert HumanReview row.
- `crates/orchestrator/tests/rce_convergence.rs`: synthetic LPIPS sequence [0.30, 0.18, 0.05, 0.018]; assert convergence at iter 4; cap test injects 7-iter sequence and asserts hard-stop at 5.
- `crates/contracts/tests/media_validator_anti_sycophancy_contract.rs`: contract guard for the four new SSB variants.

## Files touched

- CREATE: `.elai_cc/crates/orchestrator/src/media_validator/sycophancy.rs` (~180 LOC)
- CREATE: `.elai_cc/crates/orchestrator/src/media_validator/axtree.rs` (~140 LOC)
- MODIFY: `.elai_cc/crates/orchestrator/src/media_validator/tier1.rs` (+~50 LOC dispatch)
- MODIFY: `.elai_cc/crates/orchestrator/src/media_validator/feedback_loop.rs` (+~70 LOC convergence)
- MODIFY: `.elai_cc/crates/orchestrator/src/media_validator/mod.rs` (+~40 LOC second-opinion gate)
- MODIFY: `.elai_cc/crates/orchestrator/src/media_validator/render_adapters/web.rs` (+~30 LOC ariaSnapshot)
- MODIFY: `.elai_cc/crates/orchestrator/src/ssb.rs` (+4 SsbActionType variants)
- MODIFY: `.elai_cc/crates/dashboard-v2/src-tauri/src/media_validation.rs` (+~30 LOC schema)
- MODIFY: `.elai_cc/crates/dashboard-v2/src/lib/components/MediaValidationDrawer.svelte` (+~50 LOC display)
- CREATE: `workflow/reasoning/E-16-decisions.md`

## Citations / Grounding

- arXiv:2408.11261 (cited in source) — VLM sycophancy systematic analysis + contrastive-decoding mitigation
- arXiv:2509.21979 (cited in source) — EchoBench / MM-SY medical-VLM sycophancy
- arXiv:2406.07791 (cited in source) — Position bias AACL-IJCNLP 2025, >10pp pairwise-shift evidence
- arXiv:2511.21631 (cited in source) — Qwen3-VL technical report (Nov 2025 release; 30B-A3B sweet-spot)
- arXiv:2508.18265 (cited in source) — InternVL3.5 second-opinion judge candidate
- arXiv:2604.05839 + arXiv:2507.20766 (cited in source) — RCE / RRVF convergence-gated iteration
- arXiv:2411.15100 (cited in source) — XGrammar JSON-schema constrained decoding
- playwright.dev/docs/aria-snapshots — `page.ariaSnapshot()` v1.59+ API contract (verified via vendor docs page)

## Open questions / risks

- AXTree YAML stability on the same React render: report Q4 — need ≤1% diff across 20 runs to make it a Tier 1 gate. Risk: hydration-order drift may force fallback to axe-core only on deterministic-mode failures.
- Sycophancy under neutral framing on Qwen3-VL-30B-A3B: arxiv:2408.11261 measures MM-SY rate >20% on Qwen-VL family; ELAI's neutralization may not get below 5% without contrastive-decoding at the model level — possibly defer that to SGLang upstream.
- Second-opinion InternVL3.5-38B at Q4 (~22GB VRAM) is borderline for a single 24GB GPU; co-running with primary Qwen3-VL needs runtime sequencing (see B-37 PerRoleRuntime).
- XGrammar CPU-mask-gen erratic behaviour note (arxiv:2501.10868 JSONSchemaBench v3): cold-schema warmup at sidecar boot is mandatory; if Tier 2 JSON schema isn't pre-compiled, first call may stall multi-seconds.
- Anti-pattern (cited): pixel-diff as primary gate is being abandoned (Percy 2025). E-16 does NOT introduce one; the existing pHash/SSIM stay as flake-suppressors only.

## Next

- E-17 (RAG retrieval quality) is independent and parallel-safe.
- D-13 Adversary role consumes the cross-family agreement primitive E-16 establishes; sequence D-13 after E-16 for the cross-modal extension.
- D-14 Sycophancy probe at the orchestrator level reuses `neutralize_query`/`contrastive_decode` from this E-16 module — promote the module to a shared crate when D-14 ships.
- Open ROC-curve measurement for AXTree+axe → Qwen3-VL escalation (report Q5) becomes a C-20 metrics task post-E-16 ship.
