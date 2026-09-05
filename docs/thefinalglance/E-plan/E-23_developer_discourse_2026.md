# E-23: Positioning + UX-guard spec (developer-discourse 2026)
Objective: Translate the 2026 developer-discourse survey into concrete UX-guard wiring in CommandWindow + dashboard status surface, an honest positioning rewrite of README + AGENTS.md + the elai.bat first-run copy, and a measurable productivity / trust telemetry plane that lets the user defend against the METR "-19% slowdown" critique with data.
Estimated effort: 2 sessions, ~6-8 h
Pre-requisites: B-22 SHIPPED (autonomous mode + auto-advance + StagnationDetector port), B-26 DONE-PARTIAL (V2 dashboard polish), B-37 SHIPPED (PerRoleRuntime), B-40 SHIPPED (Idea-Framer + tradeoff cards)
Parallel-safe with: E-21, E-22, E-24, E-25 (no shared source surface — copy + dashboard panels only)
Source report: workflow/improvement-sweep-2026-05-19/reports/23-developer-discourse-2026.md

## Context

The 2026 sentiment data is bimodal and load-bearing for positioning. METR's RCT (arXiv:2507.09089, 16 senior OSS devs, 246 tasks, Feb-Jun 2025) found Cursor Pro + Sonnet 3.5/3.7 made devs 19% SLOWER while they FELT 20% faster. Stack Overflow 2025 reports 84% AI use but only 29% trust the output (down from 70%+ in 2023). JetBrains April 2026 AI Pulse shows 90% regular use and Claude Code 91% CSAT / NPS 54. The discourse has split into "ship-with-it" (Cloudflare, Ronacher, Birchler, Yegge, Willison) and "this-is-broken-and-getting-worse" (METR-cohort, HN long-tail). ELAI's user sits in cohort 1 with cohort-2 hygiene; the harness must make that defensible.

Top complaints (ranked by report frequency): (1) context loss / compaction without warning (Anthropic issue #13171, #19567 — Ctrl+C fails; orphan process); (2) review burden inverts the productivity claim (HN 45242788, "AI babysitter" thread); (3) surprise pricing / silent caps (Cursor June 2025 backlash); (4) quality regression on upgrades (Dolthub Jun 2025, Claude Code Feb 2025 issue #42796); (5) high-confidence hallucination; (6) AI-generated tech debt the AI itself can't decode (HN marginalia_nu); (7) MCP overhead (Ronacher Dec 2025 "I quite vocally shared my lack of success with MCP" + Cloudflare "MCP tool defs consume context tokens at scale"); (8) vendor lock-in; (9) prompt-injection as production attack (Cursor IDE CVSS 9.8, Copilot CVSS 9.3, Willison's "Challenger disaster" Jan 2026 prediction); (10) "AI ready" caveat invalidates most production repos.

What devs say they WANT next maps directly onto ELAI's existing investments: orchestration of bounded workflows over full autonomy, deterministic checks around the model, CLI tools over MCP (Ronacher Jun 2025 + Cloudflare "Code Mode"), transparent context state, local fallback for cost / privacy / vendor risk, eval / test loops over code-gen, multi-reviewer pattern (Cloudflare's 7 specialised reviewers — matches B-41 per-level auditor), no surprise pricing (ELAI is self-hosted; moat by construction). E-23 makes those moats visible to anyone reading the README, anyone watching CommandWindow during a run, and anyone trying to compare ELAI to Cursor or Claude Code on data — not vibes.

Per HARD RULE special case for E-23, this is a positioning + UX-guard spec, not a code-heavy spec. Code is limited to: telemetry plumbing for productivity / trust metrics, dashboard surface for context-state visibility, and CommandWindow / README / AGENTS.md copy rewrites. No new agent loops, no new runtime, no new MCP surface.

## Existing state (B/C/D coverage)

- B-22 SHIPPED: autonomous mode emits `auto_advance_decision`, `model_swap`, `autonomy_escalation`, `split_task_initiated` SSB rows per B-PLAN-SUMMARY line 976. EXTENSION: E-23 adds productivity / trust telemetry consuming these existing rows; no new SSB row schema needed for the metrics layer (only the visualization).
- B-26 DONE-PARTIAL (dashboard V2 polish): per B-PLAN-SUMMARY line 499. EXTENSION: E-23 adds two panels — `ContextStatePanel.svelte` (warning before compaction; addresses Anthropic issue #13171) and `TrustTelemetryPanel.svelte` (productivity / trust metrics).
- B-40 SHIPPED (Idea-Framer + Research Presets + Critic Loop): tradeoff_card_emitted SSB row per B-PLAN-SUMMARY line 992. EXTENSION: E-23 wires the tradeoff cards into the new CommandWindow first-run flow ("here's what the system will and won't do for you").
- B-41 SHIPPED (per-level auditor modes): matches Cloudflare's 7-reviewer pattern. EXTENSION: E-23 positioning copy explicitly cites this match in README.
- Existing skills: anti-spiral (CLAUDE.md anti-spiral rule), counterfactual gate (CSQ in CLAUDE.md), grounded-by-default (HARD RULE). E-23 surfaces these as readable status badges, not new mechanisms.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\23-developer-discourse-2026.md` (full)
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\windows\CommandWindow.svelte` (existing first-run + slash-command surface — modify, do not rewrite)
- `F:\.ELAI_workflow\.elai_cc\README.md` (if present — verify path via Glob; otherwise create at .elai_cc root)
- `F:\.ELAI_workflow\.elai_cc\AGENTS.md` (if present — verify path via Glob)
- `F:\.ELAI_workflow\.elai_cc\elai.bat` (first-run boot copy)
- `F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md` lines 970-1010 (SSB row registry for telemetry consumption)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\session_dag.rs` (PreCompact site for context-state warning hook)
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\commands.rs` (telemetry Tauri commands attach point)
- arXiv:2507.09089 METR RCT (workflow/papers/ if ingested; otherwise WebFetch the abstract — needed for honest positioning copy with the right citation)
- JetBrains AI Pulse blog post April 2026 (blog.jetbrains.com/research/2026/04/which-ai-coding-tools-do-developers-actually-use-at-work/)
- Anthropic issues #13171 (context loss UX) and #19567 (compaction hang)

## Tasks

1. Context-state visibility panel. New `crates/dashboard-v2/src/lib/components/ContextStatePanel.svelte` (~120 LOC). Subscribe via existing SSB stream to: current FSM state, current substate, token-budget % full, mcp_think topic + budget remaining per topic, last 3 PreCompact estimated thresholds. Above 70% context, badge turns yellow; above 85%, badge turns red. On red, show a "compact now" button that calls existing `create_handoff` Tauri command via dashboard-v2 commands. Reference Anthropic issue #13171 ("Context loss without warning breaks trust") as the design driver in a panel-header tooltip. Wire to `get_context_state` Tauri command (new — pulls from FSM + session_dag in-memory state).

2. Trust telemetry plane. New `crates/orchestrator/src/trust_telemetry.rs` (~180 LOC). Aggregate the following from existing SSB rows: (a) tasks_per_hour (productivity proxy), (b) auditor_findings_per_task (verification density), (c) revert_rate (rollback signal), (d) ungrounded_claims_per_session (counter from mcp_think calls with `grounding=""`), (e) stagnation_pattern_distribution (consumes B-22 StagnationDetector). Persist to `.elai_cc/.elai/metrics/trust-{date}.json`. Expose via new `get_trust_telemetry` Tauri command. Panel `TrustTelemetryPanel.svelte` (~150 LOC) renders a small 5-card grid.

3. Productivity-vs-METR comparison artifact. Run a one-off measurement on the .elai_cc tree itself: tasks shipped in last 30 days, average wall time per task, audit pass-rate. Output `workflow/audits/productivity-baseline-2026-05-19.md` with the numbers, METR's 19% slowdown as the reference point, and a note documenting that this is a baseline — not a counter-measurement — so the user can collect future-task data on the same metrics. Reference: arXiv:2507.09089. CRITICAL: do NOT claim ELAI beats METR without paired-task data — per memory feedback_verify_semantics and HARD RULE grounded-by-default, this is a "baseline established" artifact, not a "speedup proven" claim.

4. CommandWindow first-run rewrite. Modify `crates/dashboard-v2/src/lib/windows/CommandWindow.svelte` lines 17-200 area (verify by Read). The first-run shown on empty session must surface three short paragraphs (≤4 lines each): (a) what ELAI does — "FSM-gated, grounded, audited code work — not a chat assistant"; (b) what ELAI does NOT do — "no full autonomous YOLO mode; no surprise pricing; no hidden third-party calls; no SDK abstractions over raw API"; (c) where the moats are — "rules-as-data (not prompt), per-level auditor, local-first hybrid, cross-session FSM, grounded reasoning chain." All paragraphs cite the discourse complaint they address via a `<small>` link to the report sources. Per memory feedback_user_voice, copy follows maintainer voice — lowercase i, no em-dashes, no robotic prefixes, contractions drop apostrophes.

5. README.md / AGENTS.md alignment. Verify `.elai_cc/README.md` and `.elai_cc/AGENTS.md` exist via Glob; if missing, create. README must NOT make uncited capability claims — every claim referenced by the developer-discourse report's complaint list must have a source link or be rephrased as a design intent. AGENTS.md documents the canonical 15-agent list per CLAUDE.md (auditor, context-compiler, memory-writer, paper-ingestor, prompt-engineer, scope-planner-elai, integration-explorer-elai, synthesizer-elai, impact-mapper, spec-grounder, fixer, test-attributer, wiring-contract-validator, sidecar-health, dashboard-smoker). Cross-vendor adoption pattern: AGENTS.md is a Codex / Cursor / Claude Code shared file — verify the format via WebFetch on at least developers.openai.com/codex/skills and code.claude.com/docs/en/skills before committing structure.

6. Pricing-transparency badge. Surface `runtime_choice_badge` in CommandWindow status row, derived from existing B-37 PerRoleRuntime selection. When the active runtime is `anthropic_local_sidecar` or `ollama`, badge reads "local". When `anthropic_cloud` or any cloud backend, badge reads "cloud — billed". Token cost estimate streams in for cloud runs. This directly addresses report complaint #3 (surprise pricing / silent caps) — when the user can see "cloud / billed" they can interrupt before a runaway. Reuses B-37 SSB row `runtime_fallback`.

7. Anti-injection guard surface. The report flags prompt-injection as #9 complaint (Cursor IDE CVE-2026-26268, Copilot CVSS 9.3, Willison's "Challenger disaster" Jan 2026). E-23 does NOT implement injection defense (that is C-42 / E-24 territory). E-23 DOES surface a `prompt_injection_check_status` row in CommandWindow status bar showing whether the C-42 fuzzing job has run in the current session and what its last verdict was. The status reads `passed / blocked / not_run`. When `not_run` for >24h, show a yellow advisory. Reference: C-42 (PLANNED in C-SUMMARY_INDEX line 647).

## WIRING_CONTRACT

produces_symbols:
  - dashboard_v2::components::ContextStatePanel (Svelte)
  - dashboard_v2::components::TrustTelemetryPanel (Svelte)
  - dashboard_v2::commands::get_context_state -> ContextState (Tauri)
  - dashboard_v2::commands::get_trust_telemetry -> TrustReport (Tauri)
  - orchestrator::trust_telemetry::aggregate_session(ssb_rows) -> TrustReport
  - orchestrator::trust_telemetry::TrustReport { tasks_per_hour, findings_per_task, revert_rate, ungrounded_count, stagnation_dist }
consumes_symbols:
  - crates/orchestrator/src/session_dag.rs::ssb_rows
  - crates/orchestrator/src/runtime_orchestrator.rs (B-37 runtime badge)
  - crates/agent/src/stagnation.rs::StagnationPattern (B-22b)
  - existing PreCompact SSB rows for context-state %
  - existing auditor SSB rows for findings_per_task
reachable_from:
  - crates/dashboard-v2/src/lib/windows/CommandWindow.svelte (status row)
  - crates/dashboard-v2/src/lib/windows/CommandWindow.svelte (first-run copy block)
  - crates/dashboard-v2/src-tauri/src/commands.rs (Tauri command registration)
  - .elai_cc/README.md, .elai_cc/AGENTS.md, .elai_cc/elai.bat (copy surface)
stub_ban:
  - no "TODO: implement injection defense" — link to C-42 instead
  - no uncited capability claim in README ("ELAI is faster than X" without data citation)
  - no marketing prose like "industry-leading", "revolutionary", "state-of-the-art"
  - no token-cost claim without a `runtime_choice` row to back it
frontend_pair: crates/dashboard-v2/src/lib/windows/CommandWindow.svelte (modifies existing; not paired with new backend file because telemetry already lives in orchestrator)
observable_effect:
  trigger: open dashboard-v2 on a clean session
  expect_ssb: "trust_telemetry_aggregated" within 1000ms with non-null fields
  expect_artifact: ContextStatePanel renders with a context-% badge; TrustTelemetryPanel renders 5 cards; first-run copy shows 3-paragraph block with citations

## DETERMINISTIC_CHECK

```bash
# 1. Trust telemetry module exists
rg "pub fn aggregate_session" .elai_cc/crates/orchestrator/src/trust_telemetry.rs
# Expected: 1 hit

# 2. Two new Svelte panels exist
test -f .elai_cc/crates/dashboard-v2/src/lib/components/ContextStatePanel.svelte
test -f .elai_cc/crates/dashboard-v2/src/lib/components/TrustTelemetryPanel.svelte
# Expected: both exist

# 3. Two new Tauri commands registered
rg "get_context_state|get_trust_telemetry" .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
# Expected: ≥2 hits

# 4. CommandWindow first-run copy has the 3-paragraph block
rg "ELAI does NOT do|moats are|FSM-gated" .elai_cc/crates/dashboard-v2/src/lib/windows/CommandWindow.svelte
# Expected: ≥3 hits

# 5. README citations present
rg "arxiv|arXiv|METR" .elai_cc/README.md
# Expected: ≥1 hit (METR citation in the trust paragraph)

# 6. Productivity baseline artifact
test -f workflow/audits/productivity-baseline-2026-05-19.md
# Expected: file exists

# 7. Pricing-transparency badge wired
rg "runtime_choice_badge|local|cloud — billed" .elai_cc/crates/dashboard-v2/src/lib/windows/CommandWindow.svelte
# Expected: ≥2 hits (one for local-mode label, one for cloud-billed label)
```

## ACCEPTANCE

- [ ] ContextStatePanel renders with context % full and turns yellow/red at 70/85
- [ ] TrustTelemetryPanel renders 5 cards (tasks_per_hour, findings_per_task, revert_rate, ungrounded_count, stagnation_dist)
- [ ] CommandWindow first-run shows 3-paragraph "does / does NOT / moats" block in maintainer voice
- [ ] README and AGENTS.md updated with cited claims only; uncited claims removed or rephrased as design intent
- [ ] Productivity-baseline artifact exists with concrete numbers (tasks shipped, wall time, audit pass-rate)
- [ ] Pricing-transparency badge changes between "local" and "cloud — billed" when runtime switches
- [ ] Prompt-injection check status row shows "passed / blocked / not_run" derived from C-42 last-run-time
- [ ] No marketing prose without citation in any user-facing copy
- [ ] No new agent loops, no new runtime, no new MCP surface added (positioning + UX-guard only)

## Tests to add

- `crates/orchestrator/tests/trust_telemetry_aggregation.rs`: synthetic SSB row set with known counts; aggregate matches hand-computed
- `crates/dashboard-v2/tests/context_state_panel_thresholds.rs` (or Svelte component test if framework allows): yellow at 70%, red at 85%
- `tests/contracts/readme_citations.rs`: walk every claim in README.md and assert each has a backing link
- `tests/contracts/command_window_first_run_voice.rs`: assert first-run copy lacks "—" (em-dash) and "->" arrow per maintainer voice rules
- `crates/orchestrator/tests/runtime_choice_badge_transitions.rs`: switch runtime mid-session, assert badge value changes

## Files touched

- CREATE: `crates/dashboard-v2/src/lib/components/ContextStatePanel.svelte` (~120 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/TrustTelemetryPanel.svelte` (~150 LOC)
- CREATE: `crates/orchestrator/src/trust_telemetry.rs` (~180 LOC)
- CREATE: `workflow/audits/productivity-baseline-2026-05-19.md`
- MODIFY: `crates/dashboard-v2/src-tauri/src/commands.rs` (+~40 LOC for 2 new commands)
- MODIFY: `crates/dashboard-v2/src/lib/windows/CommandWindow.svelte` (~+80 LOC for status row badges + 3-paragraph first-run block; ~-10 LOC for any marketing prose removal)
- MODIFY: `.elai_cc/README.md` (rewrite — strip uncited claims, add 3 citations, anchor to discourse complaints)
- MODIFY: `.elai_cc/AGENTS.md` (create or rewrite to canonical 15-agent list per CLAUDE.md)
- MODIFY: `.elai_cc/elai.bat` (~+3 lines first-run echo block; verify file exists per CLAUDE.md PATH section)
- CREATE: `workflow/reasoning/E-23-decisions.md`

## Citations / Grounding

- arXiv:2507.09089 (METR RCT, Joel Becker et al.) — 19% slower with AI, 20% felt-faster gap — grounds Task 3 baseline framing
- survey.stackoverflow.co/2025 — 84% AI use, 29% trust — grounds Task 1 design driver
- blog.jetbrains.com/research/2026/04/which-ai-coding-tools-do-developers-actually-use-at-work — 90% regular use, CC 91% CSAT / NPS 54 — grounds positioning balance (capability up, trust down)
- github.com/anthropics/claude-code/issues/13171 (context loss UX) — grounds Task 1
- github.com/anthropics/claude-code/issues/19567 (compaction hang, Ctrl+C fails) — grounds Task 1 red-state behavior
- fly.io/blog/youre-all-nuts (Ptacek Jun 2025) — load-bearing pro-AI polemic for balance
- lucumr.pocoo.org/2025/12/22/a-year-of-vibes (Ronacher) — Claude Code "state of the art" — grounds the moat-existence claim
- birchtree.me/blog/the-data-has-changed (Birchler Feb 2026) — Sonnet 3.5 40% pass → Opus 4.6 90% on entire features — grounds the capability-curve claim
- blog.cloudflare.com/ai-code-review (Cloudflare specialised reviewers) — 7-reviewer pattern matches B-41 — grounds positioning
- CVE-2026-26268 (Cursor RCE), CVSS 9.8 Cursor IDE, CVSS 9.3 Copilot — grounds Task 7 status surface
- Anthropic SKILL.md spec + developers.openai.com/codex/skills — grounds AGENTS.md format (Task 5)
- memory feedback_user_voice — maintainer voice rules — grounds Task 4 + Task 5 copy style

## Open questions / risks

- README rewrite touches surface the user reads first; per HARD RULE no marketing prose, per memory feedback_user_voice copy must be maintainer voice — CONFIRM with user before pushing the README change. Risk: tone mismatch.
- Productivity-baseline-2026-05-19.md is a baseline NOT a comparison. Per CLAUDE.md grounded-by-default, claiming ELAI beats METR without paired-task data is Ungrounded. The artifact stays a baseline until paired data exists.
- Pricing-transparency badge: when local sidecar OOMs and falls back to cloud (B-37 runtime_fallback path), the badge transitions mid-task — does the user see this in time to abort? Open question: should fallback to cloud REQUIRE user-approval-on-first-fallback (default) or only warn (current B-37)?
- AGENTS.md cross-vendor format: WebFetch confirmation before committing structure. If Codex / Cursor format diverges from Claude Code's expectations, the file may need vendor-specific blocks.
- Trust telemetry plane: counting "ungrounded_claims_per_session" depends on mcp_think hook discipline. Per memory feedback_never_skip_audit_memorize + CLAUDE.md GRC enforcement, this is observable. If the user has been mcp_think-bypassing on simple tasks, the count is artificially low — the panel should disclose its source ("based on mcp_think calls with grounding=''").
- Anthropic Apr 2026 third-party tool blocking (per E-24 report): if ANTHROPIC_BASE_URL behind local sidecar gets caught by the policy, the local-first claim degrades. Out of scope for E-23 but worth a one-line caveat in the README's "no surprise pricing" paragraph.

## Next

E-21 produces the SWE-bench-Verified number that gives Task 3's productivity-baseline a comparable peer benchmark — once both ship, the user can claim "ELAI baseline = X% SWE-Verified at local cost vs Agentless 32% / OpenHands 72.8%" with data. E-24 hardens the MCP bridge which Task 7's status surface visualizes; coordinate so the C-42 fuzzer's last-run-time is queryable via a Tauri command. E-25 (compaction ladder) addresses report complaint #1 (context loss) at the mechanism layer; E-23 only surfaces the warning at the UX layer — both ship together for a complete fix.
