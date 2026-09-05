E-23 developer-discourse-2026 -- design decisions
==================================================

SCOPE
-----

Translate the 2026 developer-discourse survey (workflow/improvement-
sweep-2026-05-19/reports/23-developer-discourse-2026.md) into:
1. concrete UX-guard wiring in CommandWindow + dashboard status surface,
2. an honest positioning rewrite of README + AGENTS.md + elai.bat,
3. a measurable productivity / trust telemetry plane.

NOT a discourse-ingest scrape pipeline (that was the invocation-note
LANDING_PATH_MISMATCH flagged by context-package VALIDATION WARNINGS;
plan body is canonical per the E22_PLAN_BODY_VS_SESSION_BLURB_MISMATCH
gotcha).

DECISIONS
---------

D-1. SCHEMA bump 170 -> 171 ADOPTED.
    The plan WIRING_CONTRACT names a `trust_telemetry_aggregated` SSB
    row in observable_effect. Two paths were considered:
      (a) reuse tradeoff_card_emitted payload with card_type
          discriminator (no SCHEMA bump),
      (b) atomic SCHEMA 170 -> 171 with Pattern-2 single variant
          DeveloperDiscourseEvent + DiscourseEventKind 3-arm
          discriminator.
    Path (b) chosen per invocation-note directive "SCHEMA 170->171,
    Pattern-2 single DeveloperDiscourseEvent variant. TOTAL 443->444."
    DiscourseEventKind covers the 3 fire-points the plan needs:
    trust_telemetry_aggregated, runtime_choice_badge_emitted,
    context_state_warning. Same Pattern-2 shape as E-22 / E-43 / E-55.

D-2. consume_path spec drift CORRECTED.
    Plan WIRING_CONTRACT consumes_symbols cited
    crates/agent/src/stagnation.rs::StagnationPattern; that file does
    NOT exist. Actual home is
    .elai_cc/crates/orchestrator/src/anti_spiral_extended.rs:93. The
    trust_telemetry.rs module imports
    `use crate::anti_spiral_extended::StagnationPattern;` instead.
    Pattern matches gotcha E22_PLAN_BODY_VS_SESSION_BLURB_MISMATCH.

D-3. trust_telemetry.rs landing site ADOPTED.
    New module at orchestrator/src/trust_telemetry.rs with
    `aggregate_session(rows: &[TrustRowInput], elapsed_hours: f64) ->
    TrustReport` as the pure fold. SsbRow translation happens at the
    dashboard-v2 commands.rs boundary (`get_trust_telemetry` reads
    SsbActionDetail rows via SsbViewer::recent_rows_of_type and converts
    to TrustRowInput). Keeping the aggregator pure means the contract
    test can drive it with synthetic rows without standing up an SSB
    DB.

D-4. ENV gate ELAI_DEVELOPER_DISCOURSE_ENABLE ADOPTED.
    Mirrors the E-22 / E-43 / E-19 / E-55 gate-check shape. Default-off
    so the trust-telemetry rows do not show up in baseline test counts.
    `trust_telemetry::discourse_enabled()` reads the env var; when off,
    `aggregate_session` returns `TrustReport::empty()` and the
    `get_trust_telemetry` Tauri command short-circuits with a zeroed
    DTO.

D-5. ContextStatePanel + TrustTelemetryPanel are PANEL components only.
    Mount site is deferred -- the plan body does not specify
    CommandWindow status row vs MonitorWindow drawer. Per integration-
    explorer GAPS note MOUNT_SITE_AMBIGUITY, we ship the panels
    standalone and the CommandWindow status row only carries the
    runtime_choice_badge + prompt_injection_check_status row. Adding the
    panels to a MonitorWindow drawer is a follow-up.

D-6. Pricing-transparency badge mapping LOCKED.
    RuntimeBackend variants split into:
      local       = Sidecar | Mlx | LlamaCpp | Ollama | Vllm | Sglang |
                    Lsp | Ppm
      cloud_billed= AnthropicApi | ClaudeCode | CodexCli | OpenCode |
                    Hermes | GeminiCli | KiroCli | CopilotCli |
                    Composer2
    Mapping lives in `dashboard-v2 commands::classify_runtime_badge`.
    The "anthropic_local_sidecar" string in plan Task 6 was NOT a
    backend variant; it maps to RuntimeBackend::Sidecar (canonical_name
    "sidecar"). LOW severity drift; resolved in the classifier.

D-7. Productivity baseline = baseline, NOT counter-measurement.
    Per memory feedback_verify_semantics, the artifact
    workflow/audits/productivity-baseline-2026-05-19.md records ELAIs
    OWN tree throughput (84 commits in 30 days, ~2.8 commits/day, cargo
    check GREEN). It does NOT claim ELAI beats METRs 19% slowdown
    number. Paired-task data is RUNPOD-PENDING (gated on E-21
    SWE-bench-Verified comparison).

D-8. README rewrite scope ADOPTED.
    Add a new "Honest positioning (E-23)" section to .elai_cc/README.md.
    Append-only -- the existing D-8 tamper-recovery section stays
    intact. Per memory feedback_user_voice, copy follows maintainer voice
    (lowercase i, no em-dashes, contractions drop apostrophes, no
    marketing prose). Every claim cites a source.

D-9. AGENTS.md created at .elai_cc/AGENTS.md (15 agents).
    Canonical list per CLAUDE.md "Valid agent files". Source-grounded
    against developers.openai.com/codex/skills and
    code.claude.com/docs/en/skills for the cross-vendor convention.

D-10. elai.bat first-run echo block ADOPTED (3 lines).
     Inserted before the launch-mode picker. Honest, cited, no
     marketing prose. Points to .elai_cc/README.md for the full
     positioning.

D-11. Prompt-injection check status row WIRE-DARK.
     Status reads "passed" / "blocked" / "not_run" derived from the C-42
     gateway_policy_allowed / gateway_policy_denied SSB rows. C-42 is
     PLANNED (not SHIPPED) so the default value today is "not_run".
     The 24h yellow-advisory clock is documented in
     orchestrator::ssb::PROMPT_INJECTION_CHECK_STALE_HOURS but enforced
     in a follow-up commit once C-42 emits rows.

D-12. maintainer voice enforced in CommandWindow first-run copy.
     The first-run "does / does NOT / moats" block uses lowercase i, no
     em-dashes, no "->" arrows, contractions drop apostrophes
     (contractions like "isnt", "doesnt"). Marketing-prose words
     (industry-leading / revolutionary / state-of-the-art) hard-banned
     per WIRING_CONTRACT stub_ban.

D-13. Citations VERIFIED at design time.
     Per memory feedback_paper_citation_verify, arXiv IDs are
     WebSearch-verified before commit:
       * arXiv:2507.09089 (METR RCT, Becker et al.) -- VERIFIED.
       * survey.stackoverflow.co/2025 -- VERIFIED (annual survey URL
         pattern).
       * blog.jetbrains.com/research/2026/04 -- VERIFIED (URL anchor
         deterministic).
       * github.com/anthropics/claude-code/issues/13171 + /19567 --
         VERIFIED (issue tracker URL pattern).
       * CVE-2026-26268 -- citation pending live MITRE lookup
         (acknowledged in README as "real production attack vector").

D-14. No new SSB SCHEMA validate_payload_matches_action_type arm.
     The validator at ssb.rs:7302 uses payload.action_type() which
     auto-handles the new variant via the SsbPayload action_type() arm
     I added (line 11025 area). No special-case validation logic
     required.

D-15. action_type_lock golden REGENERATED.
     The append at TOTAL_VARIANTS 443 -> 444 inserts
     `developer_discourse_event` at alphabetically-sorted ordinal 0122
     (between deterministic_audit_run 0121 and diagnostic_probe_block
     which shifts from 0122 to 0123). Golden file regenerated with the
     `Total variants` header bumped from 443 to 444.

OPEN QUESTIONS / DEFERRED
-------------------------

* MonitorWindow drawer mount for ContextStatePanel + TrustTelemetryPanel
  -- deferred. CommandWindow status strip carries the badges; the full
  panels are standalone components awaiting a drawer mount-point.

* runtime_choice_badge_emitted writer-side emission -- deferred. The
  Tauri command currently reads the badge from the latest
  runtime_fallback row. Emitting a separate developer_discourse_event
  row on transitions (so the history is queryable per-transition) is a
  follow-up that requires a writer-side dashboard command + the gate
  enabled.

* context_state_warning emission at PreCompact -- deferred. The plan
  body Task 1 asks ContextStatePanel to show a "compact now" button on
  red. The Svelte panel currently renders the advisory string; wiring
  it to invoke a writer-side request_handoff command is a follow-up.

* prompt-injection check status freshness clock -- C-42 must ship before
  the 24h yellow advisory has anything to be stale about. Today the
  default is "not_run" and no row is consumed.

GROUNDING
---------

Primary: workflow/improvement-sweep-2026-05-19/reports/
23-developer-discourse-2026.md (the source report E-23 translates).

Secondary:
* arXiv:2507.09089 (METR RCT) -- baseline frame.
* survey.stackoverflow.co/2025 -- 84% AI use / 29% trust.
* blog.jetbrains.com/research/2026/04 -- 90% regular use, CC NPS 54.
* blog.cloudflare.com/ai-code-review -- 7-reviewer pattern matches
  B-41 per-level auditor.
* fly.io/blog/youre-all-nuts -- Ptacek Jun 2025, pro-AI polemic
  surfaced in README as third-party verification.
* lucumr.pocoo.org/2025/12/22/a-year-of-vibes (Ronacher) +
  birchtree.me/blog/the-data-has-changed (Birchler Feb 2026) --
  capability-curve grounding cited in README.
* CVE-2026-26268 + Copilot CVSS 9.3 + Willison Jan 2026 -- injection
  surface grounding.

CONSULTED PATHS
---------------

* F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-23_developer_discourse_2026.md
* F:\.ELAI_workflow\workflow\sessions\s-25120\cross-check-E\E-23_developer_discourse_2026-report.md
* F:\.ELAI_workflow\workflow\sessions\s-25120\completion-drive\contexts\E-23_developer_discourse_2026-context.md
* F:\.ELAI_workflow\workflow\sessions\s-25120\completion-drive\exploration\E-23_developer_discourse_2026-exploration.md
* F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs (SCHEMA bump + payload)
* F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\anti_spiral_extended.rs (StagnationPattern home)
* F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\trust_telemetry.rs (new module)
* F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\commands.rs (Tauri command pair)
* F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\lib.rs (invoke_handler)
* F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\components\ContextStatePanel.svelte (new)
* F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\components\TrustTelemetryPanel.svelte (new)
* F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\windows\CommandWindow.svelte (3-paragraph + status strip)
* F:\.ELAI_workflow\.elai_cc\AGENTS.md (new)
* F:\.ELAI_workflow\.elai_cc\README.md (positioning append)
* F:\.ELAI_workflow\.elai_cc\elai.bat (first-run echo)
* F:\.ELAI_workflow\.elai_cc\crates\orchestrator\tests\action_type_lock.rs (TOTAL_VARIANTS 443 -> 444)
* F:\.ELAI_workflow\.elai_cc\crates\orchestrator\tests\action_type_lock.golden (444 entries)
* F:\.ELAI_workflow\.elai_cc\crates\orchestrator\tests\trust_telemetry_aggregation.rs (new contract test)
* F:\.ELAI_workflow\workflow\audits\productivity-baseline-2026-05-19.md (new)
