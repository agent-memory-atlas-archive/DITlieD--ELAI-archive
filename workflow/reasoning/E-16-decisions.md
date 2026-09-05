E-16 -- VLM image validation hardening (decisions log)

Session: s-25120 completion-drive E-16 executor
Plan: docs/thefinalglance/E-plan/E-16_vlm_image_validation.md
Cross-check report: workflow/sessions/s-25120/cross-check-E/E-16_vlm_image_validation-report.md
Context package: workflow/sessions/s-25120/completion-drive/contexts/E-16_vlm_image_validation-context.md
Exploration brief: workflow/sessions/s-25120/completion-drive/exploration/E-16_vlm_image_validation-exploration.md
SCHEMA bump: 178 -> 179
TOTAL_VARIANTS bump: 451 -> 452
golden insertion: 0439 vlm_image_validation_event (shifts vram_tier_detected 0439 -> 0440)
Wire-dark env gate: ELAI_VLM_IMAGE_VALIDATION_ENABLE
Live VLM path: RUNPOD-PENDING per plan; host-code only this session.

Decision 01 -- SSB SHAPE COLLAPSE TO PATTERN-2 (single variant + 3-arm kind)
The plan body proposed 4 separate SsbActionType variants
(SycophancyFlag + SwapOrderDisagreement + AxtreeMismatch +
VlmSchemaUnconstrained). SycophancyFlag already shipped via E-4 / D-14
at SCHEMA 124 (ssb.rs:5450) so it is REUSED, leaving 3 new fire-points.
Pattern-2 substrate has shipped 30 prior single-variant collapses;
E-16 follows the precedent + collapses the 3 remaining additions into
ONE VlmImageValidationEvent + VlmImageValidationKind 3-arm Kind. Net
effect: ONE ordinal golden insertion + ONE SCHEMA bump 178->179
(vs the 3-bump path the plan body implied). 31st application of
Pattern-2.

Decision 02 -- RECONCILE FEEDBACK_LOOP CAP CONFLICT (path-c: distinct semantics)
The plan body called for MAX_RCE_ITERATIONS=5 but the SHIPPED
DEFAULT_MAX_ITERATIONS=3 is spec-anchored at phase-4I.txt L451. Path-c
reconciliation per the context package's recommended approach: keep
DEFAULT_MAX_ITERATIONS=3 as the per-call default budget; introduce
MAX_RCE_ITERATIONS=5 as the upper-bound HARD CAP that
lpips_delta_converged + should_stop_for_rce_cap enforce independently
of the per-call budget. No spec amendment required. The const lives at
crates/orchestrator/src/media_validator/feedback_loop.rs:
MAX_RCE_ITERATIONS + LPIPS_DELTA_CONVERGED_FLOOR + ConvergenceState +
ConvergenceReason. The DETERMINISTIC_CHECK grep
("MAX_RCE_ITERATIONS|lpips_delta_converged") returns >=2 hits.

Decision 03 -- LPIPS SURFACE IS A DETERMINISTIC SURROGATE (SSIM-derived)
The plan body assumed an "existing CPU-EP LPIPS ONNX runner" but grep
on tier1.rs / external_tools.rs / mock_vlm.rs returned 0 hits for
LPIPS / lpips. Authoring a real LPIPS ONNX runner is non-trivial
(ort=2.0.0-rc.12 pin per CLAUDE.md Anti-Stale-Training Rust rule) +
out-of-scope for this session. The shipped surface accepts
delta_prev_curr: Option<f64> from the caller; SSIM-derived surrogate
(1.0 - ssim) is the canonical default until a real LPIPS runner lands
as a follow-up. The convergence threshold (0.02) is the plan-body
value preserved verbatim. Note: arXiv:2604.05839 cited in plan body as
the convergence-threshold grounding was NOT VERIFIED on WebSearch
(2604.05839 returned no hits) -- tagged supporting-only.

Decision 04 -- MediaValidationPanel.svelte EXTENDED (NOT a new Drawer)
The plan body's WIRING_CONTRACT cited MediaValidationDrawer.svelte
but the baseline ships MediaValidationPanel.svelte
(components/MediaValidationPanel.svelte) mounted in MonitorWindow.
E29_PATH_DRIFT precedent applies. Decision: extend
MediaValidationPanel.svelte in-place (~+50 LOC) with an mvp-e16-section
that renders axtree drift chip + sycophancy verdict chip +
swap-order chip + rce sparkline chip. No new Drawer file. Existing
4 sycophancy-themed Svelte surfaces (SycophancyGuardBadge,
SycophancyMatchedPairPanel, SycophancyFlagStrip, SycophancyBenchPanel)
remain on the E-4 chat-UX path; the media panel renders the
sycophancy verdict inline rather than mounting one of the strips, to
avoid coupling the two domains' fold paths.

Decision 05 -- ARCH WIRING: SECOND-OPINION via RuntimeAdapter (no new variant)
Per E39_RUNTIME_BACKEND_REUSE gotcha, the canonical RuntimeBackend
enum at providers::runtime::types.rs is NOT extended with a per-model
variant for InternVL3.5. Instead, vlm_validator.rs::second_opinion_required
is the gate function; the actual RuntimeBackend selection is the
producer's responsibility (config-level second_opinion_backend field
addition is deferred to the live VLM wire-up follow-up).
Heuristics shipped: primary_logprob < -1.5 OR sycophancy verdict ==
"divergent_sycophancy_flag" -> required.

Decision 06 -- XGRAMMAR SCHEMA-MASK GATE IS HOST-SIDE ONLY
The plan body called for "XGrammar schema-mask enforced for Tier 2
JSON; violation -> vlm_schema_unconstrained row". XGrammar enforcement
itself lives in the SGLang/Ollama backends (RUNPOD-PENDING). Host code
ships vlm_validator.rs::vlm_output_schema_compliant which validates
the canonical VlmValidationOutput shape (issues array +
advisory_pass bool) on the raw VLM output. Producers emit a
VlmSchemaUnconstrained row when this returns false.

Decision 07 -- PLAYWRIGHT helper PATCHED (NOT a crate add)
Per workflow/papers/playwright-cdp-interface.md HIGH rank, the
subprocess Playwright route is canonical. The helper template at
web.rs:74-115 was extended with a per-viewport ariaSnapshot() call
that writes the YAML to a sibling file + emits an AXTREE:w:h:path
stdout line. The Rust side parses these lines (mirroring the
existing FRAME:w:h:path parsing) + stores the per-call set in a
new last_axtree_paths Mutex<Vec<(u32, u32, PathBuf)>> field on
WebRenderAdapter. No new crate dependency.

Decision 08 -- NO BLENDER / FFMPEG / COMFYUI dispatch for AXTree
Per the plan body's "web-only" constraint + axtree.rs module
docstring. The AxTreeAdapter is called only from the
render_adapters::web path; other adapters are unaffected.

Decision 09 -- PAPER GROUNDING (3 verified + 1 unverified)
WebSearch verification performed inline before authoring:
  * arXiv:2408.11261 "Sycophancy in Vision-Language Models" --
    VERIFIED (arxiv.org/abs/2408.11261; Aug 2024; LQCD leading-query
    contrastive decoding).
  * arXiv:2406.07791 "Judging the Judges: A Systematic Study of
    Position Bias in LLM-as-a-Judge" -- VERIFIED (arxiv.org/abs/
    2406.07791; Jun 2024; 15 judges x 22 tasks x ~40 models >150K
    instances; >10pp pairwise position-bias swing).
  * playwright.dev/docs/aria-snapshots -- VERIFIED (vendor doc;
    page.ariaSnapshot() v1.59+).
  * arXiv:2411.15100 XGrammar -- NOT INDEPENDENTLY VERIFIED this
    session; cited via plan body + treated as supporting-only.
  * arXiv:2604.05839 "RCE / RRVF convergence" -- NOT VERIFIED;
    WebSearch returned no matching paper for the prefix
    (2604.05839). Per memory:feedback_paper_citation_verify, tagged
    supporting-only -- the LPIPS_DELTA_CONVERGED_FLOOR = 0.02 value
    is preserved from the plan body but the academic anchor is NOT
    a verified citation.
  * arXiv:2511.21631 (Qwen3-VL), 2508.18265 (InternVL3.5),
    2509.21979 (EchoBench), 2507.20766 (RRVF) -- NOT VERIFIED this
    session; cited via plan body + treated as supporting-only.

Decision 10 -- TEST PARTITION (2 files, no full RCE flow)
The plan body's "Tests to add" block lists 5 files. Substrate
discipline: combine the 4 orchestrator-side tests into ONE
integration file (e16_vlm_image_validation_integration.rs) +
ship the contract crate file separately
(e16_media_validator_anti_sycophancy_contract.rs). Per the
"scoped test runs" rule (no cargo test unless permitted), only
cargo check was run this session; full test runs are auditor's job.

Decision 11 -- LANDING MODULE NAMING (vlm_validator.rs sibling)
Per the executor brief KEY CORRECTIONS, the landing surface is
orchestrator::media_validator::vlm_validator (NOT a sibling
top-level module). Three new files under
crates/orchestrator/src/media_validator/:
  * sycophancy.rs (~440 LOC)
  * axtree.rs (~330 LOC)
  * vlm_validator.rs (~260 LOC)
+ re-export of AxTreeAdapter from tier1.rs + new MAX_RCE_ITERATIONS
+ lpips_delta_converged surface in feedback_loop.rs.

DETERMINISTIC_CHECK grep coverage (per plan E-16 lines 95-121):
  1. rg "AXTreeAdapter|ariaSnapshot" -> >=4 hits across axtree.rs,
     tier1.rs (re-export), web.rs (helper + parser), tests.
  2. rg "fn neutralize_query|fn contrastive_decode" sycophancy.rs ->
     2 hits.
  3. rg "MAX_RCE_ITERATIONS|lpips_delta_converged"
     feedback_loop.rs -> >=2 hits (const + fn).
  4. rg "SwapOrderDisagreement|AxtreeMismatch|VlmSchemaUnconstrained"
     ssb.rs -> 3 enum-arm hits + as_str impls.
  5. SycophancyFlag remains untouched at ssb.rs:5450 (E-4 prior).
  6. e2e smoke + SQLite SELECT remain RUNPOD-PENDING.

Remaining follow-ups (out-of-scope for this session):
  * Real LPIPS ONNX runner (ort=2.0.0-rc.12 wire-in)
  * Live SGLang/Ollama second-opinion judge wire-up
  * cloud_mode_live test un-ignore (depends on the above two)
  * second_opinion_backend MediaValidationConfig field +
    PerRoleRuntime sequencing
  * AXTree-stability Q4 production measurement
  * paper-ingestor mode=paper for the 5 unverified arXiv IDs
