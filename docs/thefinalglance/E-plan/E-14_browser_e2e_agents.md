# E-14: Browser E2E agents — Playwright AXTree-first Tier 1, hybrid VLM Tier 2, deploy-preview Cosmos worker
Objective: ship Tier 1 web validation as AXTree-first Playwright (CLI snapshot-to-disk, 4x token saving) with axe-core gates, restrict Tier 2 VLM-as-judge to constrained-decoding downgrade-only paths, add a DeployPreviewWorker in the Cosmos tier that polls Vercel/Netlify Deployment Checks API + cURL retry-with-backoff, and lock Tier 2 asymmetric-veto semantics (Tier 2 may only DOWNGRADE Tier 1 pass to HumanReview, never UPGRADE a Tier 1 fail). Hybrid SSIM + axe-core + AXTree shape catches what pure pixel diff floods false positives on.
Estimated effort: 2-3 sessions (~10-12 h).
Pre-requisites: B-24 SHIPPED (Phase 4I Media Validator — Tier1+Tier2 + 5 MediaRenderAdapter at .elai_cc/crates/orchestrator/src/media_validator/), B-42 SHIPPED (Cosmos Artifact Tier — cosmos_worker.rs + VerificationMethod::CosmosCheck), C-7 PLANNED (Playwright visual.spec.ts harness in dashboard-v2), C-16 PLANNED (Cosmos pipeline integration).
Parallel-safe with: E-11, E-12, E-13, E-15. AUGMENTS B-24 + B-42 + C-16.
Source report: workflow/improvement-sweep-2026-05-19/reports/14-browser-e2e-agents.md

## Context

The May-2026 browser-agent landscape converges on a clear architectural split: AXTree-first deterministic structural checks + axe-core a11y gates catch the bulk of regressions cheaply (TestDino 2026, Microsoft Playwright 2026 official); VLM-as-judge (Applitools / Qwen3.5-VL) catches semantic regressions but is sycophantic (arXiv:2410.11302) and weak at absolute scoring (arXiv:2604.25235 "VLM Judges Can Rank but Cannot Score"). The 2026 Playwright 1.56+ CLI snapshot-to-disk mode uses ~4x fewer tokens than MCP for equivalent work (Microsoft official blog). Cloudflare Turnstile in 2026 fingerprints JA4+TLS and headless Chromium is detected within a session (Browserless 2026, Nerdbot Apr 2026); the report is unambiguous: do NOT attempt bypass. Vercel's Deployment Checks API + the canonical url-health-check-action GitHub Action provide a portable wait-on-deploy primitive — local PASS is not deploy PASS (Autonoma 2026 vercel-preview-deployments). ELAI's B-24 Media Validator already ships Tier 1 (in-process parsers <100ms) + Tier 2 (VLM advisory, Mode A Qwen3.5-VL-30B-A3B via ollama / Mode B InternVL3.5-38B) with the correct veto-logic skeleton (Tier1 fail → Fail; Tier1 pass + Tier2 disagree → HumanReview); the missing pieces are: AXTree as the Tier 1 structural primitive (currently SSIM/LPIPS pixel-only), axe-core injection at every Tier 1 web check, constrained-decoding VLM (currently free-text), hybrid SSIM-first → VLM-only-on-changed-regions, the DeployPreviewWorker in Cosmos, multi-context Playwright for SaaS journey validation, Stagehand fallback adapter for natural-language extraction. The asymmetric Tier-2-veto rule the report calls "under-specified" in the current ELAI spec is closed here: Tier 2 may only DOWNGRADE Tier 1 PASS to HumanReview, never upgrade a Tier 1 FAIL — grounded in the sycophancy + score-reliability findings.

## Existing state (B/C/D coverage)

- EXTENSION of B-24 (SHIPPED) Phase 4I Media Validator — extends 5 MediaRenderAdapter (.elai_cc/crates/orchestrator/src/media_validator/render_adapters/) with axe-core + AXTree primitive; current adapters are Playwright web, Blender 3D, FFmpeg animation, direct read screenshots, ComfyUI AI-gen.
- EXTENSION of B-42 (SHIPPED) Cosmos Artifact Tier — adds a DeployPreviewWorker variant alongside the existing cosmos_worker.rs; uses cosmos_worker's manifest pattern + tick() loop.
- AUGMENTS C-16 (PLANNED) Cosmos pipeline integration — adds DeployPreviewWorker into the planned CosmosStage in QueryPipeline.
- AUGMENTS C-7 (PLANNED) Dashboard UX completeness — extends visual.spec.ts harness with AXTree snapshot assertions.
- REUSES B-24's Tier 2 VLM surface (Mode A Qwen3.5-VL-30B-A3B / Mode B InternVL3.5-38B) — does NOT change the model; only enforces constrained decoding via outlines/xgrammar and the asymmetric veto-logic.
- NET-NEW: `PlaywrightWebAdapter` (Tier 1 deterministic, replaces current Playwright surface with AXTree-first), `StagehandWebAdapter` (Tier 2 natural-language fallback when AXTree role missing), `DeployPreviewWorker` (Cosmos tier).
- NET-NEW: `axe-core injection` at every Tier 1 web check; `serious`/`critical` impact = hard fail, `moderate` = Tier 2 advisory.
- NET-NEW: hybrid visual diff (SSIM first; if SSIM_delta < threshold, no VLM call — cost saved + sycophancy dodged; if SSIM_delta > threshold, crop changed region and route to Tier 2).

## Reads (mandatory before coding)

- workflow/improvement-sweep-2026-05-19/reports/14-browser-e2e-agents.md
- .elai_cc/crates/orchestrator/src/media_validator/mod.rs lines 569-1130 (Tier1/Tier2 dispatch + Verdict + veto logic)
- .elai_cc/crates/orchestrator/src/media_validator/render_adapters/blender.rs (existing adapter shape pattern)
- .elai_cc/crates/orchestrator/src/media_validator/render_adapters/comfyui.rs (existing adapter shape pattern)
- .elai_cc/crates/orchestrator/src/media_validator/feedback_loop.rs lines 461-554 (RCE feedback loop integration)
- .elai_cc/crates/knowledge/src/cosmos_worker.rs lines 247-1190 (CosmosError + manifest + tick() — DeployPreviewWorker mirrors this)
- .elai_cc/crates/orchestrator/src/ssb.rs (Cosmos SSB action_types pattern at line 5993; new deploy_preview_* mirrors)
- .elai_cc/crates/dashboard-v2/src/lib/windows/MediaValidationPanel.svelte (existing surface; new fields added)
- arXiv:2604.25235 VLM Judges Can Rank but Cannot Score (via paper-ingestor)
- arXiv:2410.11302 VLM Sycophancy (via paper-ingestor)
- arXiv:2509.25142 VLM Serial-processing deficit (via paper-ingestor)
- TestDino 2026 + Microsoft Playwright official 2026 blog (via paper-ingestor mode=source, target Playwright 1.56 release notes)

## Tasks

1. Add AXTree-first Tier 1 primitive. New module `.elai_cc/crates/orchestrator/src/media_validator/ax_snapshot.rs` with `AxNode { role, name, state }`, `AxSnapshot { root: AxNode, ... }`, `AxSnapshotReader::from_disk(path: &Path) -> AxSnapshot`. Surface returns `{ax_snapshot_path, screenshot_path, console_log_path}` per Tier 1 invocation. Downstream rules grep AXTree by role/name not screenshot pixels. The reader is plain TS parse of Playwright CLI's JSON dump — no Playwright runtime dependency in Rust.

2. Refactor existing PlaywrightWebAdapter under `.elai_cc/crates/orchestrator/src/media_validator/render_adapters/playwright_web.rs` to use Playwright 1.56+ CLI snapshot-to-disk mode (4x token saving grounded in Microsoft 2026 blog). The adapter spawns `npx playwright snapshot --url <url> --out <dir>` per render call (Tier-C sandboxed); output dir contains `ax-snapshot.json`, `screenshot.png`, `console.log`, `trace.zip`. Default `RenderOptions { retries: 2, trace_on_retry: true, disable_animations: true, seed: deterministic }` per BrowserStack 2026 flake-mitigation defaults.

3. Inject axe-core at every Tier 1 web check. New file `.elai_cc/crates/orchestrator/src/media_validator/axe_gate.rs`. The Playwright snapshot adapter additionally runs `@axe-core/playwright` against the rendered page; output `axe-results.json` parsed for impact ∈ {minor, moderate, serious, critical}. `serious` and `critical` impact = hard Tier 1 FAIL (immediate Verdict::Fail). `moderate` impact = Tier 2 advisory (passes Tier 1, surfaces in dashboard). `minor` logged only. Aligns with Inclly + TestParty grounding (axe-core 96 rules vs Lighthouse 50).

4. Add `StagehandWebAdapter` at `.elai_cc/crates/orchestrator/src/media_validator/render_adapters/stagehand_web.rs`. Used as fallback when AXTree role/name cannot be located (AI-generated app used non-semantic markup). Calls Browserbase/Stagehand v3 act/extract/observe primitives via local Node process (Tier-C sandboxed). The adapter caches action sequences keyed by (url_hash, goal_hash); cache replay survives DOM drift between regenerations of the same generated app. Cache at `~/.cache/elai/stagehand/`. Pure-fallback semantics: never used when AXTree-first succeeds.

5. Implement constrained-decoding VLM Tier 2 with strict Pydantic schema. New file `.elai_cc/crates/orchestrator/src/media_validator/vlm_constrained.rs`. The existing Mode A Qwen3.5-VL-30B-A3B / Mode B InternVL3.5-38B path stays, but every VLM call enforces schema `{"pass": bool, "failures": [{"region": "...", "reason": "...", "confidence": 0.0..=1.0}]}` via outlines / xgrammar / Anthropic Tool API. Reject any free-text response with `VlmInvalidOutput` error → degrades to HumanReview. Grounding: Daft 2026 multimodal-structured-outputs; report best-technique #3 ("kills sycophancy-by-elaboration").

6. Implement hybrid visual diff with SSIM-first short-circuit. New file `.elai_cc/crates/orchestrator/src/media_validator/hybrid_diff.rs`. Compute SSIM between baseline and candidate screenshots; if `SSIM_delta < threshold` (default 0.02, configurable per elai_cc.toml [media_validator.hybrid_diff].ssim_threshold), no VLM call (cost saved). If `SSIM_delta >= threshold`, crop changed regions (bounding boxes from SSIM delta map) and route ONLY the cropped regions to Tier 2 VLM. Emit `hybrid_diff_short_circuit` SSB when no VLM call needed; emit `hybrid_diff_vlm_dispatched` when VLM called.

7. Lock asymmetric Tier-2 veto semantics. Update `.elai_cc/crates/orchestrator/src/media_validator/mod.rs::Verdict` aggregation function: `Tier1 FAIL → Verdict::Fail` (Tier 2 cannot override); `Tier1 PASS + Tier2 PASS → Verdict::Pass`; `Tier1 PASS + Tier2 FAIL → Verdict::HumanReview` (downgrade only); `Tier1 PASS + Tier2 unavailable → Verdict::Degraded` (existing behavior); `Tier1 absent + Tier2 PASS → Verdict::HumanReview` (new — Tier 2 cannot rubber-stamp without Tier 1); `Tier1 absent + Tier2 FAIL → Verdict::HumanReview`. Grounding: arXiv:2410.11302 sycophancy + arXiv:2604.25235 score-reliability.

8. Implement `DeployPreviewWorker` in Cosmos tier. New file `.elai_cc/crates/knowledge/src/deploy_preview_worker.rs` mirroring `cosmos_worker.rs` surface. Poll Vercel/Netlify Deployment Checks API (config in elai_cc.toml [deploy_preview] provider + api_token + project_ids) every 30s; alternative cURL retry-with-backoff against the preview URL (semantics of GitHub url-health-check-action). On first 200 OK, fan out smoke RCE via existing media_validator on the live URL. Emits new SSB action_types: `deploy_preview_polling_started`, `deploy_preview_ready`, `deploy_preview_failed`, `deploy_preview_app_validation_complete`. The worker is gated by `[deploy_preview].enabled` (default false; opt-in per project).

9. Add multi-context Playwright SaaS journey adapter. New file `.elai_cc/crates/orchestrator/src/media_validator/saas_journey.rs`. Spawns N browser contexts (one per simulated user) via Playwright's `browser.newContext()` API; supports network-mock to capture outgoing emails/webhooks rather than relying on real SMTP; asserts notif-arrival via captured request. Each context uses isolated cookies/localStorage to prevent state-leakage flake (BrowserStack 2026 #1 flake cause). Critical journey = signup → first-value loop (Tenjin 2026); journeys defined per project at `.elai_cc/journeys/{project}.toml` with steps and assertions.

10. Add Cloudflare/Turnstile detection + skip semantics. New module `.elai_cc/crates/orchestrator/src/media_validator/cf_detect.rs`. On any rendered page, detect cf-mitigated response (Cloudflare challenge HTML signature, Turnstile widget, JA4 mismatch banner); mark check as `Verdict::Skipped(ReasonExternalProtection)`. NEVER attempt bypass (legal + ToS risk per report anti-pattern #4). Emit `cf_protection_detected` SSB.

11. Wire DeployPreviewWorker into the existing Cosmos pipeline (C-16 PLANNED). Add `DeployPreviewArtifact` variant alongside the existing Cosmos artifact types; the worker emits artifacts the QueryPipeline's CosmosStage can use. If C-16 has not shipped, ship the DeployPreviewWorker as standalone with its own tick loop in task_runner and migrate to CosmosStage when C-16 lands.

12. Update `MediaValidationPanel.svelte` (existing, B-24 surface): add AXTree snapshot viewer (collapsible role/name tree), axe-core findings list with impact filter, hybrid-diff SSIM gauge with VLM-dispatched events, DeployPreviewWorker status badge, multi-context journey timeline, CF-skip notice banner. Three new Tauri commands: `get_ax_snapshot_recent`, `get_axe_findings_recent`, `get_deploy_preview_status`.

13. Document calibration measurement plan: open Q1 in the report (Brier score + ECE for Qwen3.5-VL-30B-A3B) becomes a follow-up measurement patch run after E-14 ships; the result feeds the SSIM threshold default in [media_validator.hybrid_diff].

## WIRING_CONTRACT
produces_symbols:
  - orchestrator::media_validator::ax_snapshot::AxSnapshot
  - orchestrator::media_validator::ax_snapshot::AxSnapshotReader
  - orchestrator::media_validator::axe_gate::AxeGate
  - orchestrator::media_validator::render_adapters::playwright_web::PlaywrightWebAdapter (refactored)
  - orchestrator::media_validator::render_adapters::stagehand_web::StagehandWebAdapter
  - orchestrator::media_validator::vlm_constrained::VlmConstrainedClient
  - orchestrator::media_validator::hybrid_diff::HybridDiff
  - orchestrator::media_validator::saas_journey::SaasJourneyRunner
  - orchestrator::media_validator::cf_detect::CfDetector
  - knowledge::deploy_preview_worker::DeployPreviewWorker
consumes_symbols:
  - orchestrator::media_validator::mod::Verdict (updated aggregation function)
  - orchestrator::media_validator::feedback_loop::RceFeedbackLoop (existing)
  - knowledge::cosmos_worker::tick (pattern reused by DeployPreviewWorker)
  - orchestrator::ssb::SsbActionType (new variants)
  - dashboard-v2::commands::get_ax_snapshot_recent / get_axe_findings_recent / get_deploy_preview_status
reachable_from:
  - crates/orchestrator/src/task_runner.rs::drive_task → wire_media_validator_for_spec → PlaywrightWebAdapter
  - crates/cli/src/main.rs → /research → CosmosStage → DeployPreviewWorker (when C-16 ships)
  - crates/dashboard-v2/src-tauri/src/lib.rs → MediaValidationPanel renders
stub_ban:
  - todo!() / unimplemented!() in any new adapter or worker
  - bypass attempt patterns in cf_detect.rs (NEVER); only detection + skip
  - free-text VLM responses accepted (must be schema-constrained)
  - Box<dyn Error> in new modules; use thiserror per-module enum
  - .unwrap() outside #[cfg(test)]
frontend_pair: crates/dashboard-v2/src/lib/windows/MediaValidationPanel.svelte
observable_effect:
  trigger: "elai /task --mode cloud 'build a Next.js todo app and validate it end-to-end on the deploy preview'"
  expect_ssb: "media_render_started → ax_snapshot_emitted → axe_findings_count > 0 (probably some minor) → if SSIM_delta < 0.02 hybrid_diff_short_circuit emits, else hybrid_diff_vlm_dispatched + vlm_pass/vlm_fail; deploy_preview_polling_started → deploy_preview_ready → deploy_preview_app_validation_complete"
  expect_artifact: ".elai_cc/sessions/{id}/media/{request_id}/ax-snapshot.json exists; .elai_cc/sessions/{id}/media/{request_id}/axe-results.json present; Verdict aggregation respects asymmetric veto rule (Tier 1 fail = Verdict::Fail regardless of Tier 2)"

## DETERMINISTIC_CHECK
```bash
# 1. AXTree module + Stagehand adapter exist
rg "pub struct AxSnapshot|pub struct PlaywrightWebAdapter|pub struct StagehandWebAdapter|pub struct HybridDiff" .elai_cc/crates/orchestrator/src/media_validator/
# expect: 4+ hits

# 2. axe-core integration
rg "@axe-core/playwright|axe_gate::AxeGate" .elai_cc/crates/orchestrator/src/media_validator/
# expect: >=2 hits

# 3. DeployPreviewWorker mirrors cosmos_worker shape
rg "pub struct DeployPreviewWorker" .elai_cc/crates/knowledge/src/deploy_preview_worker.rs
# expect: 1 hit

# 4. SSB action_types registered
rg "ax_snapshot_emitted|axe_findings_count|hybrid_diff_short_circuit|hybrid_diff_vlm_dispatched|deploy_preview_polling_started|deploy_preview_ready|deploy_preview_failed|deploy_preview_app_validation_complete|cf_protection_detected" .elai_cc/crates/orchestrator/src/ssb.rs
# expect: >=9 hits

# 5. VLM constrained-decoding enforced
rg "VlmInvalidOutput|VlmConstrainedClient" .elai_cc/crates/orchestrator/src/media_validator/vlm_constrained.rs
# expect: >=2 hits

# 6. Asymmetric veto codified
rg "Tier1 absent.*HumanReview|Tier1 absent.*Verdict::HumanReview" .elai_cc/crates/orchestrator/src/media_validator/mod.rs
# expect: >=1 hit (or comment block citing the rule)

# 7. CF detect never attempts bypass
rg "ReasonExternalProtection|Verdict::Skipped" .elai_cc/crates/orchestrator/src/media_validator/cf_detect.rs
# expect: >=2 hits

# 8. Smoke: hybrid diff short-circuits below threshold
cargo test -p elai-orchestrator --test hybrid_diff_short_circuit
# expect: pass

# 9. Smoke: Tier 1 fail cannot be overridden by Tier 2 pass
cargo test -p elai-orchestrator --test asymmetric_veto_tier1_fail_dominates
# expect: pass

# 10. Smoke: cf-protected page is Skipped not Fail and never attempts bypass
cargo test -p elai-orchestrator --test cf_detect_skip_not_bypass
# expect: pass
```

## ACCEPTANCE
- [ ] AxSnapshot + AxSnapshotReader modules; Playwright CLI snapshot-to-disk wired in PlaywrightWebAdapter
- [ ] axe-core integrated; serious/critical = Tier 1 fail; moderate = Tier 2 advisory; minor = log only
- [ ] StagehandWebAdapter as fallback only when AXTree role missing; action cache at ~/.cache/elai/stagehand/
- [ ] VlmConstrainedClient enforces Pydantic-style schema; free-text response → VlmInvalidOutput → HumanReview
- [ ] HybridDiff: SSIM-first, VLM dispatched only when SSIM_delta >= threshold (default 0.02)
- [ ] Asymmetric veto semantics: Tier 1 fail dominates; Tier 2 only downgrades Pass to HumanReview; Tier 1 absent → HumanReview
- [ ] DeployPreviewWorker polls Vercel/Netlify Deployment Checks API + cURL backoff; smoke RCE on first 200 OK
- [ ] SaasJourneyRunner: multi-context Playwright; network-mock for SMTP/webhook capture; per-project journeys at .elai_cc/journeys/{project}.toml
- [ ] CfDetector marks cf-mitigated as Verdict::Skipped(ReasonExternalProtection); NEVER attempts bypass
- [ ] 9+ new SSB action_types registered in ssb.rs
- [ ] MediaValidationPanel.svelte extended; 3 new Tauri commands wired
- [ ] No NoopSsbNotifier; no Box<dyn Error>; no .unwrap() outside #[cfg(test)]
- [ ] elai_cc.toml [media_validator.hybrid_diff] + [deploy_preview] + [media_validator.cf_detect] tables present

## Tests to add
- `.elai_cc/crates/orchestrator/tests/ax_snapshot_parse.rs`: 4 cases (well-formed, missing role, empty tree, malformed JSON)
- `.elai_cc/crates/orchestrator/tests/axe_gate_severity.rs`: 5 cases (critical / serious / moderate / minor / clean)
- `.elai_cc/crates/orchestrator/tests/hybrid_diff_short_circuit.rs`: 3 cases (below threshold short-circuits, above threshold dispatches, threshold-edge)
- `.elai_cc/crates/orchestrator/tests/asymmetric_veto_tier1_fail_dominates.rs`: 5 cases (T1 fail + T2 pass / T1 fail + T2 fail / T1 pass + T2 pass / T1 pass + T2 fail / T1 absent + T2 pass)
- `.elai_cc/crates/orchestrator/tests/vlm_constrained_rejects_free_text.rs`: 3 cases (schema-conformant / free-text / partial-schema)
- `.elai_cc/crates/orchestrator/tests/stagehand_action_cache.rs`: 3 cases (cache miss / cache hit / DOM-drift cache stale)
- `.elai_cc/crates/orchestrator/tests/cf_detect_skip_not_bypass.rs`: 4 cases (cf challenge HTML / turnstile widget / clean page / JA4 mismatch)
- `.elai_cc/crates/knowledge/tests/deploy_preview_worker_polls.rs`: 4 cases (200 OK / 404 retry / timeout / API error)
- `.elai_cc/crates/orchestrator/tests/saas_journey_signup_flow.rs`: 3 cases (happy path / webhook captured / context-isolation flake regression test)

## Files touched
- CREATE: `.elai_cc/crates/orchestrator/src/media_validator/ax_snapshot.rs` (~220 LOC)
- CREATE: `.elai_cc/crates/orchestrator/src/media_validator/axe_gate.rs` (~180 LOC)
- CREATE: `.elai_cc/crates/orchestrator/src/media_validator/render_adapters/stagehand_web.rs` (~280 LOC)
- CREATE: `.elai_cc/crates/orchestrator/src/media_validator/vlm_constrained.rs` (~220 LOC; outlines/xgrammar binding)
- CREATE: `.elai_cc/crates/orchestrator/src/media_validator/hybrid_diff.rs` (~260 LOC; SSIM + crop)
- CREATE: `.elai_cc/crates/orchestrator/src/media_validator/saas_journey.rs` (~340 LOC; multi-context Playwright)
- CREATE: `.elai_cc/crates/orchestrator/src/media_validator/cf_detect.rs` (~140 LOC; detection only, no bypass)
- CREATE: `.elai_cc/crates/knowledge/src/deploy_preview_worker.rs` (~420 LOC; mirrors cosmos_worker shape)
- CREATE: `.elai_cc/journeys/example.toml` (~30 LOC; reference journey schema)
- MODIFY: `.elai_cc/crates/orchestrator/src/media_validator/mod.rs` (+~100 LOC; Verdict aggregation update; asymmetric veto)
- MODIFY: `.elai_cc/crates/orchestrator/src/media_validator/render_adapters/playwright_web.rs` (existing — verify path; +~80 LOC for AXTree + CLI snapshot-to-disk)
- MODIFY: `.elai_cc/crates/orchestrator/src/ssb.rs` (+~70 LOC; 9 new SsbActionType variants)
- MODIFY: `.elai_cc/elai_cc.toml` (+~50 LOC; 3 new tables)
- MODIFY: `.elai_cc/crates/dashboard-v2/src/lib/windows/MediaValidationPanel.svelte` (+~140 LOC; new sections)
- MODIFY: `.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs` (+~70 LOC; 3 new Tauri commands)
- MODIFY: `.elai_cc/crates/orchestrator/src/task_runner.rs` (+~30 LOC; DeployPreviewWorker spawn loop until C-16 lands)

## Citations / Grounding
- Microsoft Playwright 2026 official blog — CLI snapshot-to-disk ~4x fewer tokens than MCP for same task
- TestDino 2026 Playwright AI ecosystem — Playwright 1.56 AXTree-first + planner/generator/healer
- arXiv:2604.25235 — VLM Judges Can Rank but Cannot Score; asymmetric-veto grounding
- arXiv:2410.11302 — VLM sycophancy (flip correct answers under push-back); constrained decoding rationale
- arXiv:2509.25142 — VLM serial-processing deficit (multi-step UI forms); structured-output rationale
- Daft 2026 multimodal-structured-outputs — Pydantic-enforced VLM output is required for determinism
- Inclly 2026 axe vs Lighthouse — Lighthouse runs ~50 axe rules; axe-core 96; axe is industry standard
- TestParty 2026 — Automated catches 30-40% WCAG; rest needs manual/VLM
- Vercel kb / deployment-checks API — native external test results blocking deployment
- GitHub url-health-check-action — cURL retry-with-backoff canonical wait-on-deploy gate
- Browserless 2026 / Nerdbot Apr 2026 — Turnstile JA4 + behavioral ML detection; do NOT bypass
- BrowserStack 2026 Playwright flaky tests — retries:2, trace:on-first-retry, fixed seeds, disable animations
- Tenjin 2026 SaaS E2E guide — critical journey = signup → first-value loop, not feature matrix
- Browserbase 2026 Stagehand v3 — act/extract/observe primitives + action cache replay
- Autonoma 2026 Vercel preview deployments — local PASS != deploy PASS

## Open questions / risks
- Ungrounded: SSIM threshold default 0.02 is a starting heuristic; the report's open Q2 calls for a 500-app sweep to plot cost curve. Plan to measure on 50 generated apps as a follow-up.
- Microsoft's "4x token saving for CLI snapshot-to-disk vs MCP" headline is a Microsoft-task-specific number; report open Q3 calls for an A/B on ELAI-scale tasks before broadcasting the win.
- Stagehand action-cache survival across regenerations (report open Q4) is uncalibrated; the cache may miss on every regeneration if DOM drift is large. Measure on 30 regen runs.
- DeployPreviewWorker depends on Vercel/Netlify Deployment Checks API tokens and project_ids; air-gapped mode (B-27 [execution] air_gapped=true) must disable the worker with an SSB row `deploy_preview_disabled_air_gapped`.
- Tier 2 VLM at Mode B (InternVL3.5-38B, ~38GB 8-bit) does not fit the user's 12GB primary VRAM box (per project_gpu_context.md ~1.8GB free); Mode A (Qwen3.5-VL-30B-A3B, ~17GB Q4) is the sole viable local path. The constrained-decoding wrapper must be tested against Mode A first; Mode B is cloud-only deferred.
- Playwright 1.56+ snapshot-to-disk mode requires `npx playwright` in the Tier-C sandbox; the sandbox profile must allow node + npx. Coordinate with E-15 sandbox profile.
- Cloudflare/Turnstile arms race: detection patterns drift quarterly; cf_detect.rs heuristics must be refreshed via paper-ingestor mode=source quarterly.

## Next
- Calibrate Qwen3.5-VL-30B-A3B pass-probability vs ground truth (200-app fixture; report Brier + ECE). Follow-up patch-N.
- E-15 (sandbox) provides the Tier-C profile for `npx playwright` + headless Chromium; coordinates with PlaywrightWebAdapter.
- C-16 absorption: when C-16 ships, migrate DeployPreviewWorker from task_runner spawn loop into CosmosStage pipeline. Schedule as a follow-up patch.
- Open Q5 from the report (DNS/edge propagation gap between Deployment Checks API ready and first-200-on-HEAD) becomes a measurement-only patch.
