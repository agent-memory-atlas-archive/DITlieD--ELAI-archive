E-14 browser_e2e_agents — scope-lean decisions
==============================================

Status: SHIPPED (scope-lean cut)
Plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-14_browser_e2e_agents.md
Session: s-25120 (session 2/4 of the cluster — sessions 1 + 3 concurrent)
Re-check report: F:\.ELAI_workflow\workflow\sessions\s-25120\cross-check-E\E-14_browser_e2e_agents-report.md

What shipped
------------

1. crates/orchestrator/src/browser_e2e/ — new module with 5 submodules:
   * mod.rs       — re-exports + E2eError thiserror enum.
   * playwright.rs — PlaywrightRunner via tokio::process::Command.
                    Spawns npx with --no-install + embedded Node helper.
                    Wire-dark on ENOENT.
   * snapshot.rs  — PageSnapshot { url, html, ax_tree, screenshot_path, ts_ms }
                    + SnapshotKind enum (HtmlOnly / HtmlAndScreenshot /
                    HtmlAndAxTree / Full).
   * assertion.rs — pure run_assertions(snapshot, asserts) -> AssertionReport.
                    3 assertion kinds: ContainsText / HasSelector / StatusOk.
   * journey.rs   — load_journey(path: &Path) -> Result<Journey, E2eError>
                    parsing .elai_cc/journeys/*.toml. 4 step kinds: Navigate
                    / Click / Fill / WaitFor. Steps are shape-validated only;
                    semantic execution (Click / Fill) is deferred.
   * axe_core.rs  — run_axe_core via npx + @axe-core/playwright subprocess.
                    parse_axe_results public for in-process JSON inspection.
                    AxeImpact mapping: critical+serious blocking,
                    moderate+minor advisory.

2. crates/orchestrator/src/media_validator/render_adapters/web.rs —
   EXTENDED (not replaced) WebRenderAdapter with optional
   `e2e_assertions: Option<Vec<Assertion>>` field + `with_e2e_assertions()`
   builder + `run_e2e_assertions()` delegating to
   `crate::browser_e2e::run_assertions`. Per rebase rule #2 — no
   parallel `playwright_web.rs` file.

3. crates/orchestrator/src/media_validator/types.rs — added
   `SkipReason::ExternalProtection` enum + `Verdict::Skipped(SkipReason)`
   5th variant. The dashboard renders this as a non-fail badge for pages
   gated by Cloudflare Turnstile / hCaptcha / Akamai BMP. Audit for
   exhaustive matches confirmed no `match` on Verdict exists anywhere
   in the codebase — all sites use `matches!()` (which is fine with new
   variants) and the dashboard stores Verdict as String, so no
   exhaustiveness sweep was required.

4. crates/orchestrator/src/ssb.rs — atomic SCHEMA_VERSION 130 -> 131
   bump for 4 new SsbActionType variants:
   * BrowserE2eRunStarted     ({ journey_name_hash, url_hash, ts_ms })
   * BrowserE2eAssertionFailed({ journey_name_hash, assertion_hash, error_hash, ts_ms })
   * BrowserE2eAssertionPassed({ journey_name_hash, assertion_hash, ts_ms })
   * BrowserE2eSnapshotCaptured({ journey_name_hash, snapshot_kind_hash, ts_ms })

   9-tuple cascade per variant:
   (a) enum decl, (b) as_sql_str arm, (c) from_str_public arm,
   (d) SsbPayload enum variant, (e) Self::*(_)=>SsbActionType::* arm,
   (f) payload struct decl, (g) test_payload constructor arm,
   (h) all_test_payloads vec entry + count 300 -> 304,
   (i) SQL CHECK constraint string.

   Lock file: tests/action_type_lock.rs TOTAL_VARIANTS 361 -> 365 +
   ALL_VARIANTS push + exhaustive_match_guard arms. Golden file
   regenerated alphabetically: 4 new entries at ordinals 0036-0039
   (between bsp_superstep and bubble_up_typed alphabetically — note
   that `browser_e2e_*` sort before `bsp_*`).

5. .elai_cc/journeys/ (NEW directory) — example_journey.toml +
   README.md documenting the TOML schema.

6. crates/cli/src/browser_e2e_cmd.rs (NEW) — `elai browser
   <run|list-journeys|status>` handler. Scope-lean: the CLI handler
   does NOT spawn Playwright (kept deterministic + GPU-free); it
   builds a synthetic snapshot from the journey URL and runs the pure
   assertion checker. Live Playwright runs land via the orchestrator
   path when wired by the task runner. The CLI's `probe_toolchain`
   merely checks PATH for `npx` (no actual spawn).

7. crates/cli/src/args.rs — added BrowserAction sub-enum +
   CliAction::Browser variant + parse_browser_subcommand_args. Mirrors
   the C-46 Routing / C-47 Calibration shape.

8. crates/dashboard-v2/src-tauri/src/commands_e14.rs (NEW) — single
   read-only Tauri command `browser_e2e_status` aggregating the 4 SSB
   variants into a typed summary (runs_started + assertions_passed +
   assertions_failed + snapshots_captured + last_journey_name_hash +
   last_ts_ms). Mirrors the C-47 viewer.recent_rows_of_type pattern.

9. crates/dashboard-v2/src/lib/components/MediaValidationPanel.svelte
   — extended with a new "browser e2e" section above the entries
   list. Path-drift fix per rebase rule #1: edited under
   `lib/components/` (not `lib/windows/` as the plan claimed).

10. crates/contracts/tests/e14_browser_e2e_agents_contract.rs (NEW) —
    9 contract tests (c1..c9) covering:
    c1: Verdict::Skipped(SkipReason::ExternalProtection)
    c2: WebRenderAdapter extension (not parallel adapter)
    c3: 4 SSB variants + wire-tag stability
    c4: Journey TOML round-trip + 4 step kinds
    c5: Playwright runner wire-dark for absent npx
    c6: Citation discipline (3 deferred arXiv IDs not inlined)
    c7: axe-core impact mapping (critical+serious blocking)
    c8: run_assertions single source of truth (adapter delegates)
    c9: JourneyStep value optional for non-fill kinds

Scope cuts (vs original plan)
-----------------------------

Plan task 4 (VLM Mode A constrained-decoding) — DEFERRED.
Plan task 5 (VlmConstrainedClient + outlines/xgrammar) — DEFERRED.
Plan task 6 (HybridDiff SSIM + image crop) — DEFERRED. The image
crate is already a workspace dep but the hybrid-diff cropping
algorithm requires careful calibration (open Q1 in the plan calls
for a 500-app sweep). Deferred to follow-up patch.
Plan task 8 (DeployPreviewWorker) — DEFERRED. Requires Vercel/Netlify
API tokens + air-gapped guard (B-27); cosmos_worker mirror is a
~420 LOC standalone task that fits cleanly as a follow-up patch-N.
Plan task 9 (SaasJourneyRunner multi-context) — DEFERRED. The
scope-lean Journey + assertion runner is the shape primitive; the
multi-context user-isolation work lands after the single-context
path is observed in production.
Plan task 10 (CfDetector) — PARTIALLY SHIPPED. The Verdict variant
ships (SkipReason::ExternalProtection) so consumers can render the
badge; the actual cf-detect heuristic module is deferred (heuristics
drift quarterly — paper-ingestor refresh required).

SSB variants: 9 plan-proposed -> 4 essential
--------------------------------------------

Plan proposed 9 SsbActionType variants:
  ax_snapshot_emitted, axe_findings_count, hybrid_diff_short_circuit,
  hybrid_diff_vlm_dispatched, deploy_preview_polling_started,
  deploy_preview_ready, deploy_preview_failed,
  deploy_preview_app_validation_complete, cf_protection_detected.

Scope-lean shipped 4:
  BrowserE2eRunStarted, BrowserE2eAssertionFailed,
  BrowserE2eAssertionPassed, BrowserE2eSnapshotCaptured.

Rationale: the 5 deferred variants tie to deferred plan tasks
(hybrid_diff_*, deploy_preview_*, cf_protection_detected — the cf
variant ships at the Verdict level via Skipped(ExternalProtection)
instead). The ax_snapshot_emitted variant collapses into
BrowserE2eSnapshotCaptured with a snapshot_kind_hash discriminator
covering html_only / html_and_screenshot / html_and_ax_tree / full
in a single variant.

RUNPOD-PENDING
--------------

Live Playwright execution requires `npx` + `playwright` + (for axe)
`@axe-core/playwright`. The runner wires the toolchain probe as
wire-dark; the user's primary box (12 GB VRAM, 1.8 GB free) cannot
host the VLM tier (Mode A Qwen3.5-VL-30B-A3B Q4 ~17 GB; Mode B
InternVL3.5-38B ~38 GB) so VLM-tier validation lands at the remote
3090 (or runpod) when the journey runner is wired into the task
runner.

Citations
---------

Design grounding (referenced in this doc, NOT inlined in source):
- Microsoft Playwright 2026 official blog — CLI snapshot-to-disk
  ~4x fewer tokens than MCP (claim deferred to ELAI-scale A/B per
  plan open Q3).
- Inclly 2026 + TestParty 2026 — axe-core 96 rules vs Lighthouse 50;
  serious+critical = WCAG blocking.
- BrowserStack 2026 — Playwright flaky tests; retries:2,
  trace:on-first-retry, fixed seeds.
- Browserless 2026 / Nerdbot Apr 2026 — Cloudflare Turnstile
  detection patterns + ToS forbids bypass.

Deferred per cluster cite-incident discipline (workflow/sessions/
s-25120/mistakes.md E-14 entry):
- arXiv:2604.25235 — VLM Judges Can Rank but Cannot Score
- arXiv:2410.11302 — VLM sycophancy
- arXiv:2509.25142 — VLM serial-processing deficit

These are referenced by author/title in this doc only; contract test
C6 enforces non-inlining in orchestrator/src/browser_e2e/.

Schema state
------------

SCHEMA_VERSION   130 -> 131
TOTAL_VARIANTS   361 -> 365
all_test_payloads 300 -> 304
Existing v130 session DBs are DROPped on first read (DESTRUCTIVE per
the standard schema bump posture).

Test results
------------

cargo check --workspace --lib                       PASS (9 warnings, pre-existing)
cargo test -p elai-orchestrator --test action_type_lock   2 PASS, 1 ignored (regen helper)
cargo test -p elai-orchestrator --lib browser_e2e         25 PASS
cargo test -p elai-orchestrator --lib ssb::tests::test_typed_payloads 1 PASS
cargo test -p elai_cc_contracts --test e14_browser_e2e_agents_contract 9 PASS

Path drift fixes
----------------

Plan WIRING_CONTRACT cited frontend_pair at
`lib/windows/MediaValidationPanel.svelte` — the actual location is
`lib/components/MediaValidationPanel.svelte` (confirmed via ls +
existing MonitorWindow.svelte:29 import). The component was edited
in place at the real path; the plan text remains cosmetically wrong
but the implementation matches reality.
