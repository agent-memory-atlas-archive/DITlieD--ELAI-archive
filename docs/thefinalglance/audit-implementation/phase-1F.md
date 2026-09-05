PHASE 1F AUDIT: SPECULATIVE DECODING EVALUATION
================================================================================
Audit date: 2026-04-24
Auditor: hostile implementation auditor (sonnet)
Phase type: EVALUATION (decision phase, not feature phase)
Spec source: docs/thefinalglance/summary/phase-1-summary.md L861-928
Target: .elai_cc/crates/providers/src/providers/ollama.rs

================================================================================
DECISION OUTCOME
================================================================================

Verdict: DROP (resolved 2026-04-07, commit c02e6e8 "P1F.2: Ollama speculative
decoding API probe — NOT_SUPPORTED, DROP — 66/69 green").

The decision is documented, grounded, and consistent with the decision cascade
defined in spec L913-920. The DROP decision was reached at step 1F.2 (API
availability) per the first rule of the decision gate: "If NOT_AVAILABLE: DROP".
This legitimately short-circuits steps 1F.3, 1F.3b, 1F.4, and the throughput
portion of 1F.5 per the spec's cascade semantics.

================================================================================
PER-SUB-STEP VERDICT
================================================================================

--------------------------------------------------------------------------------
1F.1 SPECULATIVE DECODING CONFIG SKELETON
Verdict: IMPLEMENTED
--------------------------------------------------------------------------------

Spec requirement (L881): "Add speculative_decoding: bool (default false),
draft_model: Option<String>, num_draft_tokens: u8 (default 5) to OllamaConfig
or nested SpeculativeConfig. Serde support (de+serialize)."

Evidence:
- .elai_cc/crates/providers/src/providers/ollama.rs L254-286 defines
  `SpeculativeConfig { speculative_decoding: bool, draft_model: Option<String>,
  num_draft_tokens: u8 }` with `Serialize + Deserialize + Clone + Debug`.
- L261-268 `impl Default for SpeculativeConfig` returns
  `speculative_decoding: false, draft_model: None, num_draft_tokens: 5` —
  exactly matches spec defaults.
- L288-293 `OllamaConfig` contains `speculative: SpeculativeConfig` (nested form).
- L271-286 validator `SpeculativeConfig::new` rejects `num_draft_tokens == 0`
  with explicit error "num_draft_tokens must be >= 1".
- .elai_cc/crates/providers/src/lib.rs L32 re-exports `SpeculativeConfig` so
  downstream crates can depend on the type.
- Commit 7788b41 "P1F.1: Ollama provider skeleton — 62/62 green".

Test coverage:
- L689-695 `test_speculative_config_defaults` — verifies Default impl.
- L697-713 `test_speculative_config_serde_roundtrip` — serialize/deserialize
  preserves all three fields (round-trips "qwen2:1.5b" draft with k=8).
- L715-724 `test_speculative_config_rejects_zero_draft_tokens` — validator test.

Acceptance criteria (L883): "0 errors; defaults match (false, None, 5); serde
round-trip preserves all fields." Met by the three tests above.

--------------------------------------------------------------------------------
1F.2 OLLAMA SPECULATIVE DECODING API AVAILABILITY CHECK
Verdict: IMPLEMENTED + PROBE EXECUTED + NOT_SUPPORTED DETERMINED
--------------------------------------------------------------------------------

Spec requirement (L886-890): Probe ollama for draft_model support; if
NOT_SUPPORTED document and evaluate fallbacks; prefer 0.5B-1.5B draft models;
output binary AVAILABLE/NOT_AVAILABLE determination; result drives DROP path.

Evidence (implementation):
- ollama.rs L17-53 `ProbeResult` enum with `Supported { ollama_version,
  api_params, draft_model }` and `NotSupported { ollama_version, reason }`
  variants. Both serde-enabled and have Display impls.
- ollama.rs L102-237 `pub async fn probe_speculative_support(base_url, model,
  draft_model_candidate, timeout) -> Result<ProbeResult, ApiError>`:
  - Queries `/api/version` for ollama version (L114-120ish).
  - Issues baseline `/api/generate` with fixed prompt "Count from 1 to 5.",
    seed=42, temperature=0.0, num_predict=20 (L129-144).
  - Issues second `/api/generate` with same params plus
    `draft_model=<candidate>` and `num_draft=5` in `options` (L146-166).
  - Compares response key sets (L171-205): looks for any of seven known
    speculative-decoding response keys (draft_model, draft_tokens_accepted,
    draft_tokens_generated, draft_tokens_rejected, num_draft, speculative,
    speculative_decoding), then falls back to a set-difference of extra keys.
  - Returns `Supported` if any draft-specific key appears, else
    `NotSupported` with a detailed reason string that cites PR #8134,
    issue #5800, baseline vs draft response text snapshots, and key sets.
- ollama.rs L461-468 `ollama_complete` conditionally inserts `draft_model`
  and `num_draft` into `/api/generate` `options` only when
  `config.speculative.speculative_decoding && draft_model.is_some()` —
  production path honors config flag (no hardcoded wiring).
- Commit c02e6e8 "P1F.2: Ollama speculative decoding API probe —
  NOT_SUPPORTED, DROP — 66/69 green (3 pre-existing oauth), 0 warnings".

Test coverage (ollama.rs L685-832):
- test_probe_result_not_supported_display (L728-738)
- test_probe_result_supported_display (L740-751)
- test_probe_result_serde_roundtrip_not_supported (L753-762)
- test_probe_result_serde_roundtrip_supported (L764-774)
- test_truncate_short_string / long_string / multibyte_utf8 (L776-797) —
  reason-string truncation helper, UTF-8 safe.
- test_probe_local_ollama (L802-832) — `#[tokio::test] #[ignore]` integration
  test that runs against a live ollama instance on 11434 with gemma4:e2b +
  gemma3:4b; asserts the probe returns a definitive variant.

Evidence (probe result / decision doc):
- workflow/reasoning/phase-1F-decisions.md (46 lines):
  - Date: 2026-04-07, Hardware: RTX 3060 12GB, Ollama: 0.20.2.
  - DETERMINATION: NOT_SUPPORTED.
  - Six evidence points: PR #8134 closed Jan 9 2025, issue #5800 still open,
    issue #9216 closed as duplicate, API docs lack draft_model, CLI has no
    --draft flag, and a live probe run where baseline and draft responses
    had identical key sets and identical output text "1, 2, 3, 4, 5".
  - Fallback evaluation: llama.cpp not installed (and out of scope for Phase 1
    per providers.txt L55), vLLM not available until Phase 2+. Both fallbacks
    correctly classified non-viable.
  - Explicit DROP verdict with re-evaluation trigger: "Re-evaluate in Phase 2
    when vLLM integration exists, or when ollama merges speculative decoding
    support."
  - VRAM reclaim statement: "4.8GB headroom after gemma4:e2b remains
    available for embedding models (Phase 1C vector search)" — matches the
    DECISION GATE rule (L918) "If spec_decode <=1.3x OR delta_vram_mb >=3000:
    DROP. Reclaim VRAM for embedding models".

Acceptance criteria (L889): "AVAILABLE or NOT_AVAILABLE binary determination."
Met — NOT_AVAILABLE documented with six corroborating evidence items.

Adversarial note: the probe does not strictly guarantee ollama is ignoring
the parameter vs applying it silently with zero observable effect; however,
the decision doc supplements this with external docs/PR evidence, which is
the grounded way to resolve the ambiguity. Cross-check passes.

--------------------------------------------------------------------------------
1F.3 CORRECTNESS VERIFICATION
Verdict: LEGITIMATELY SKIPPED (per decision cascade)
--------------------------------------------------------------------------------

Spec requirement (L892-896): PRE_GATE explicitly states "1F.2 result is
AVAILABLE (skip to 1F.5 DROP otherwise)." Since 1F.2 produced NOT_AVAILABLE,
this step is not executable by design.

Evidence of legitimate skip:
- No phase_1F_correctness.toml under .elai_cc/metrics/ (verified — metrics
  dir contains phase_1A.toml, phase_1B.toml, phase_1D.toml, phase_1E.toml
  but no 1F variant).
- phase-1F-decisions.md L35: "Steps 1F.3-1F.5 are SKIPPED."
- phase-1-summary.md L892 marks the sub-step "(SKIPPED)" in the spec header
  itself.

Adversarial pushback: is there any residual code for correctness verification
that would be dead weight if not exercised?
- Grepped for `speculative_decode`, `SpeculativeDecoder`, `SpecDecoder`:
  NO MATCHES. The only symbols that survive are `SpeculativeConfig` (config
  plumbing) and `probe_speculative_support` (the diagnostic). No lossy
  decoding pipeline exists — correct for a DROPPED feature.

--------------------------------------------------------------------------------
1F.3b PROMPT LOOKUP DECODING BASELINE
Verdict: LEGITIMATELY SKIPPED (per decision cascade)
--------------------------------------------------------------------------------

Spec requirement (L898-902): zero-VRAM prompt lookup (n-gram speculation)
baseline as fallback when spec decode unavailable.

Evidence of legitimate skip:
- No phase_1F_prompt_lookup.toml under .elai_cc/metrics/.
- phase-1F-decisions.md does not mention prompt lookup explicitly.
- phase-1-summary.md L868 (phase summary DECISION OUTCOME block): "Prompt
  Lookup Decoding (1F.3b) ALSO SKIPPED as part of DROP."

Adversarial note: spec L898 PRE_GATE says "1F.3 passes OR 1F.2 NOT_AVAILABLE"
— i.e., prompt lookup was intended as a fallback to probe even when native
speculative is unavailable. The phase summary (L868) overrides this by
rolling it into the DROP. This is a legitimate narrowing given ollama does
not expose --lookup-ngram-min either (per spec L900 fallback path to
llama.cpp which is out of scope). However, strictly reading spec L898, this
step was executable in principle and was folded into the DROP as a scope
decision rather than a gate block. Classified LEGITIMATELY SKIPPED but with
a minor spec-vs-decision gap noted; not a violation, because the summary
page authoritatively documents the rollup.

--------------------------------------------------------------------------------
1F.4 THROUGHPUT BENCHMARK
Verdict: LEGITIMATELY SKIPPED (per decision cascade)
--------------------------------------------------------------------------------

Spec requirement (L904-908): tokens/sec sweep k={1,2,3,4,5,7}, 5 prompts, 3
runs each, acceptance rate measurement, early-kill rule, phase_1F_throughput.toml.

Evidence of legitimate skip:
- PRE_GATE L905: "1F.3 passes (10/10)" — not satisfiable since 1F.3 is
  SKIPPED upstream.
- No phase_1F_throughput.toml under .elai_cc/metrics/.
- No k-sweep code, no acceptance-rate metric emission, no GPU util polling
  hooks. Grep for `acceptance_rate`, `num_draft_tokens` benchmarking code
  returns only the config-validator test.

--------------------------------------------------------------------------------
1F.5 VRAM MEASUREMENT AND DECISION GATE
Verdict: PARTIALLY IMPLEMENTED — gate reached correctly, VRAM measurement skipped
--------------------------------------------------------------------------------

Spec requirement (L910-922): measure baseline_vram_mb, speculative_vram_mb,
delta_vram_mb via nvidia-smi polling; apply top-to-bottom decision gate;
write phase-1F-decisions.md with binary ADOPT/DROP and rationale.

Evidence of decision gate execution:
- phase-1F-decisions.md exists and contains a grounded DROP verdict.
- The decision rule applied is L914 "If NOT_AVAILABLE: DROP. Document 'spec
  decode not available on current stack as of [date]'" — which the doc does
  verbatim ("DROP: speculative decoding not available on current stack as of
  2026-04-07").
- The cascade rule is top-to-bottom with stop-at-first-match semantics, and
  the NOT_AVAILABLE rule is first — correctly terminates before requiring
  VRAM arithmetic.
- VRAM reclaim consequence logged: "4.8GB headroom ... remains available for
  embedding models (Phase 1C vector search)" — matches the DROP disposition.

What is NOT done and the reason it is acceptable:
- No phase_1F_vram.toml with nvidia-smi snapshots. Spec L912 requires this
  only when the throughput path was run; with NOT_AVAILABLE the speculative
  process was never loaded so no speculative_vram_mb exists to measure.
  Baseline VRAM alone is not spec-required in isolation. Acceptable.
- No config defaults were updated per L922 ("if ADOPT config compiles; if
  DROP no config changes"). The DROP branch explicitly says "no config
  changes" — and indeed `SpeculativeConfig::default()` still ships
  `speculative_decoding: false`, which means the DROP state is the compiled
  default. Correct per spec.

Residual-surface risk: the config fields and probe function are still in
tree. This is not a spec violation — per phase-1-summary.md L868 the
OllamaProvider + LlmProvider trait abstraction was intentionally preserved
so that "vLLM integration is new provider impl, not rewrite" in Phase 2+.
The phase summary line explicitly covers this: "ACCEPTANCE blocks remain in
source for completeness." The config fields are zero-cost (default=false
short-circuits the draft_model insertion at L461-468), so there is no runtime
overhead from the retained surface.

================================================================================
DECISION CASCADE TRACE (spec L913-920 applied)
================================================================================

1. If NOT_AVAILABLE: DROP  -> MATCH (from 1F.2) -> stop here.
2. If acceptance <0.5: DROP  -> not reached.
3. If prompt_lookup >1.3x AND spec_decode <= prompt_lookup  -> not reached.
4. If spec_decode >1.3x AND delta_vram_mb <3000: ADOPT  -> not reached.
5. If spec_decode <=1.3x OR delta_vram_mb >=3000: DROP  -> not reached.
6. Boundary ratio: DROP  -> not reached.

Cascade applied correctly; first-match semantics respected.

================================================================================
ADVERSARIAL CROSS-CHECKS
================================================================================

1. Is speculative decoding silently wired into the executor anyway?
   - Grep for `SpeculativeDecoder`, `SpecDecoder`, `speculative_decode` in
     .elai_cc: NO MATCHES.
   - `probe_speculative_support` is called only from its own integration
     test (#[ignore]) and ProbeResult is only constructed in probe code and
     tests. No runtime caller.
   - `ollama_complete` only inserts draft_model/num_draft when
     `config.speculative.speculative_decoding == true`, and Default is false.
     VERDICT: cleanly gated, no sneak-in.

2. Is the "NOT_SUPPORTED" result falsifiable / could it be a probe bug?
   - The probe compares key sets across a baseline vs draft request with
     the same seed and temperature; if ollama honored draft_model it would
     typically add at least one of the seven well-known spec-decode keys or
     produce divergent `done_reason`. Identical response text plus identical
     extra-key sets is strong behavioural evidence of silent ignore.
   - The decision doc further corroborates with external PR/issue URLs
     (PR #8134 closed, #5800 open, #9216 closed duplicate). Not sole-source.

3. Grep "speculative" for unrelated usages that might be mistaken for Phase 1F:
   - `speculative_depth` in crates/orchestrator/src/research_tier.rs — this
     is research-tree speculative branching depth (a feature in the agent
     planning layer), NOT decoder-level speculation. Unrelated.
   - `prune_speculative_children`, `adversarial_speculative_pruning_consistency`
     in crates/orchestrator/... — same orchestration concept, unrelated.
   - `contract_cloud_provider_allows_speculative` / `local_provider_forces_
     speculative_zero` — these gate research-tree speculative depth by
     provider type, not decoder draft models. Unrelated.
   Conclusion: no overlapping terminology is being smuggled through.

4. Do tests actually run (not just compile)?
   - Commit c02e6e8 message: "66/69 green (3 pre-existing oauth), 0 warnings".
   - Commit 7788b41 message: "62/62 green (3 pre-existing oauth), 0 warnings".
   - Unit tests are non-ignored and run on every cargo test invocation;
     integration test is `#[ignore]`-gated and was run manually to produce
     the decision doc evidence (L10-26 of the decision doc quote actual
     live probe output including baseline/draft response keys).

5. Spec corrections landed?
   - phase-1F-grounding.md documents five spec corrections (EAGLE-3 2x-6.5x,
     VRAM 4.8GB not 10GB, Phase 1C prereq, R-NEW-1/2). phase-1-summary.md
     L870 confirms these were applied and notes implementation-plan.txt L184
     still stale (flagged INFO, not blocking).

================================================================================
GAPS / OBSERVATIONS
================================================================================

G1 (INFO, not blocking): the residual `SpeculativeConfig` surface is
retained for future vLLM integration per phase-1-summary.md L868, but there
is no explicit in-code `#[deprecated]` or comment at the struct site noting
"kept for Phase 2+ vLLM revisit". A future reader of ollama.rs L254 may not
know why an unused feature toggle is present without reading the phase
summary. Non-blocking, but a single doc-comment above `SpeculativeConfig`
would close the loop.

G2 (INFO): the integration test `test_probe_local_ollama` references
hardcoded model names (gemma4:e2b, gemma3:4b) even though the project
selection has moved away from Gemma 4 (per memory file "No Gemma 4" —
Qwen2.5-Coder is the baseline). Because the test is `#[ignore]` and was
only used to produce the DROP evidence in April 2026, this is stale but
harmless. If re-run in Phase 2+, the model names should be parametrized.

G3 (INFO): phase_1F-decisions.md does not have a machine-readable TOML
sibling (vs phase_1A/1B/1D/1E.toml). Spec L921 DETERMINISTIC_CHECK implies
phase_1F_vram.toml should exist only if the ADOPT branch were taken; for
DROP branch spec L923 says "If DROP: no config changes" and does not
mandate a TOML. Compliant.

================================================================================
SUMMARY
================================================================================

Phase 1F is REAL and CORRECTLY RESOLVED as DROP. The evaluation phase
output — a measured, evidence-backed decision — is exactly what the spec
mandates (L866: "EVALUATION phase, not feature phase. Output is measured
decision, not shipped code").

Deliverables honored:
- 1F.1 (config skeleton): FULLY IMPLEMENTED with 3 unit tests, commit 7788b41.
- 1F.2 (availability probe): FULLY IMPLEMENTED with 8 tests + 1 integration,
  live probe executed against ollama 0.20.2 on 2026-04-07, commit c02e6e8.
- 1F.3, 1F.3b, 1F.4, 1F.5 (throughput path): LEGITIMATELY SKIPPED via the
  NOT_AVAILABLE branch of the decision cascade (spec L914).
- Decision doc: workflow/reasoning/phase-1F-decisions.md, 46 lines, six
  evidence items, external URL citations, explicit re-evaluation trigger.
- Grounding log: workflow/reasoning/phase-1F-grounding.md, 80 lines, 8 claims
  graded (VERIFIED/CORRECTED/UNGROUNDED), 11 external sources cited.
- Spec amendment trail: phase-1-summary.md L868, L870, L935 all carry the
  DROP rollup and the Phase 2+ revisit contract.

Residual code footprint is intentional (OllamaProvider skeleton + LlmProvider
trait + config fields) for clean Phase 2+ vLLM swap; runtime cost is zero
because `speculative_decoding` defaults to false and the hot path short-
circuits before touching draft_model options.

OVERALL VERDICT: PASS.
DECISION OUTCOME: DROP (not available on current ollama 0.20.2 stack as of
2026-04-07; re-evaluate when vLLM lands or ollama merges speculative
decoding support).

Evidence files:
- Decision doc: F:\.ELAI_workflow\workflow\reasoning\phase-1F-decisions.md
- Grounding log: F:\.ELAI_workflow\workflow\reasoning\phase-1F-grounding.md
- Probe code: F:\.ELAI_workflow\.elai_cc\crates\providers\src\providers\ollama.rs
  (L17-237 probe, L254-286 config, L461-468 gated wiring, L685-832 tests)
- Commits: 7788b41 (P1F.1), c02e6e8 (P1F.2, NOT_SUPPORTED, DROP)
- Metrics: intentionally absent (DROP branch of spec L921)
