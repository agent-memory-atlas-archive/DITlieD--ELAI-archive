# E-4: Chat UX critique (sycophancy, doom loops, false-completion)
Objective: ship the trust-impact-per-effort fixes that the report ranks as #1-#8 — per-claim confidence chips, sycophancy guardrails, pinned-rules visible panel, session re-anchor every N turns, doom-loop detector wired to anti-spiral skill, evidence-required "done" claims, one-shot clarifier, and mid-stream interrupt. Plus the ROLE handoff + intervention controls that the report's "When chat is the wrong UI" section calls out.
Estimated effort: 3 sessions (~10-12 h)
Pre-requisites: B-14 (live token streaming chat, DONE) + B-22 (autonomous mode + AutoAdvanceEvaluator, DONE) + B-39 (Chat REFLECT pipeline, DONE) + B-41 (Per-Level Auditor Modes, SHIPPED — supplies the AuditTimelineWindow + verdict surface) + E-2 (frame-pacing + skeleton bubble — provides the bubble-header surface this E-N extends) + E-3 (ROLE badge in bubble header).
Parallel-safe with: E-1 (memory_notes substrate — confidence chips can write provenance to memory_notes), E-5 (user modeling — expertise-level field feeds Task 8 expertise-aware terseness).
Source report: workflow/improvement-sweep-2026-05-19/reports/04-chat-ux-critique.md

## Context

ELAI's CommandWindow is the user-facing surface for trust formation. The report ranks 11 measurable failure modes (Table "Top failure modes, grounded, ranked by harm"): sycophancy is #1 trust-killer (Sharma arXiv:2310.13548; OpenAI's April 2025 GPT-4o rollback at openai.com/index/sycophancy-in-gpt-4o/); persona drift hits 20-40% in 8-15 turns (arXiv:2412.00804); "lost in the middle" drops mid-context recall ~30% (aclanthology 2024.tacl-1.9); doom loops are the dev-tool-specific failure mode (warpedvisions.org); over-refusal rates #2 frustration for power users (arXiv:2509.08833 + HN 39988419); false-completion claims are the HN-canonical complaint about Claude Code (HN 47936579).

What flagship chat does well: Anthropic's Constitution operationalises honesty as near-hard-constraint ("diplomatic honesty rather than dishonest diplomacy"); Claude 4+ trained toward conciseness; mid-stream interrupt; one sharp clarifier (arXiv:2507.21285). What destroys trust permanently: one sycophancy event ("I agreed with your wrong claim about library X"), false-completion claim, gaslighting / repair-resistance, persona drift mid-session, confident hallucination cited as fact.

ELAI is structurally well-placed: it has the FSM, SSB evidence ledger, anti-spiral skill in `.claude/skills/anti-spiral/`, the GRC tools with mcp_think + grounding, AUDITING state with auditor mode, and B-41's per-level auditor modes (concept/phase/step/babystep/babystep-detail/plan). The fixes are mostly WIRING into the existing CommandWindow rather than net-new components.

The report's 14 ranked recommendations cluster into three groups: (A) bubble-level affordances (confidence chips, ROLE badge, evidence link on "done", honest-correction surface) — extend the CommandWindow bubble schema; (B) session-level rails (pinned-rules panel, session re-anchor, doom-loop detector, intervention controls) — surface FSM + SSB state to the UI; (C) hard rules in the model output path (sycophancy guardrails, one-shot clarifier, terse-by-default, refusal-with-override) — go in the role-prompt voice anchor (E-3) and the ChatReflectPipeline AUDITOR stage.

## Existing state (B/C/D coverage)

- B-14 (SHIPPED): live token streaming chat in CommandWindow.svelte + intent classifier (TASK/FIX/EXPLORE/CONFIG/INTERVENE/STEER/ABORT) + card variants (status / intervention / error / disambiguation / steering / user). The intervention + steering card variants already exist — this E-N extends them with the doom-loop + clarifier surfaces.
- B-22 (SHIPPED): AutoAdvanceEvaluator + autonomous-mode FSM substates (per task_runner.rs:3996 derive_autonomous_evidence). The doom-loop detector reuses this evidence-derivation pattern.
- B-39 (SHIPPED): ChatReflectPipeline 4-stage TRIAGE -> REASONER -> AUDITOR -> SYNTHESIZE. Sycophancy guardrails ship in the AUDITOR stage. One-shot clarifier ships in the TRIAGE stage (gate: if task ambiguity > threshold, emit clarifier card, ELSE proceed).
- B-41 (SHIPPED): Per-Level Auditor Modes + AuditTimelineWindow.svelte. The "false-completion claim" anti-pattern is structurally caught by per-level audits; this E-N adds a UI affordance for "every 'done' must reference at least one audit verdict + one deterministic check".
- B-38 (PLANNED): Ambiguity-Score Continuous Metric (4 dimensions, threshold <=0.2). The one-shot clarifier in this E-N (Task 7) gates on B-38's ambiguity score. Soft dependency — ship Task 7 with a heuristic floor first, swap to B-38 when it lands.
- C-21 (PLANNED): MistakeBook + rules.db consolidation. The pinned-rules-visible panel (Task 3) reads from rules.db via the existing `crates/dashboard-v2/src-tauri/src/lib.rs::get_rules_*` Tauri command path (verify exact name during reads).
- D-14 / D-23 (PLANNED): sycophancy probe + benchmark. Task 2 (sycophancy guardrails) is the structural defense; D-14/D-23 are the measurement harness. Direct alignment.
- E-3 (THIS PLAN): unified system-prompt preamble + sycophancy_guard clause in voice.toml. Task 2 (sycophancy guardrails) lives in that voice anchor.
- E-1 (THIS PLAN): memory_notes substrate. Task 1 (confidence chips) writes to memory_notes as `fact_type='claim'` rows with `confidence_tier` field.
- This E-N is a DELTA over B-14 (extends bubble schema) + B-22 (reuses doom-loop detection) + B-39 (extends AUDITOR stage) + B-41 (consumes audit verdict surface) and a NET-NEW for the pinned-rules panel, session re-anchor toast, and intervention control bar.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\04-chat-ux-critique.md` (source report, full)
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\windows\CommandWindow.svelte` (full file ~600 LOC; bubble + card variants)
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\windows\SettingsWindow.svelte:23-58` (settings rows surface — `currently honoring` panel hooks here)
- `F:\.ELAI_workflow\.elai_cc\crates\reasoning\src\chat_reflect.rs:635` (AUDITOR stage — sycophancy guardrails inject here)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs:3996,4645,7776` (AutoAdvanceEvaluator + MistakeBook query call sites — doom-loop hook lands here)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs` (SsbActionType registry)
- `F:\.ELAI_workflow\.claude\skills\anti-spiral\SKILL.md` (4-pattern detector + 5-persona switcher already documented; this E-N wires it to UI)
- `F:\.ELAI_workflow\.claude\rules\output-formatting.md` (maintainer voice + terse defaults)
- `F:\.ELAI_workflow\CLAUDE.md` (Confidence calibration rule: Verified / High confidence / Uncertain prefixes)
- arXiv:2310.13548 — Sharma sycophancy
- arXiv:2503.02623 — Rewarding Doubt
- arXiv:2505.24858 — MetaFaith (linguistic uncertainty markers)
- arXiv:2412.00804 — Examining Identity Drift in LLM Agents
- aclanthology.org/2024.tacl-1.9 — Lost in the Middle
- arXiv:2507.21285 — Curiosity by Design (LLM clarification questions)
- arXiv:2604.19245 — Talking to a Know-It-All GPT (repair-driven multi-turn failure)
- arXiv:2509.08833 — Pitfalls of Over-Alignment (over-refusal harm)
- HN 47936579 — Claude Code false-completion thread
- warpedvisions.org/blog/2025/getting-unstuck-when-using-llms-for-coding (doom-loop UX)

## Tasks

1. **Per-claim confidence chips (~280 LOC across reasoning + dashboard).** New struct `ClaimWithConfidence { text, tier: enum { Verified | ReadFromSource | FromTraining | Guessed }, evidence_ref: Option<EvidenceRef> }`. ChatReflectPipeline AUDITOR stage emits these inline. CommandWindow renders as small pills before the claim text. Tier sourced from: existing GRC grounding methods (FILE_CHECK / SOURCE_CHECK / TEST_CHECK / WEB_CHECK at `crates/reasoning/`) -> Verified or ReadFromSource; no grounding -> FromTraining; explicit `mcp_think(step_type="status")` echo with low confidence -> Guessed. Maps to CLAUDE.md "Confidence calibration" rule. Source: arXiv:2503.02623, arXiv:2505.24858, arXiv:2412.02904.

2. **Sycophancy guardrails in ChatReflectPipeline AUDITOR (~180 LOC delta in `crates/reasoning/src/chat_reflect.rs`).** AUDITOR-stage prompt amended with a calibrated-dissent block: "If user has asserted X and you have evidence Y contradicting X, hold position with citation. If you have no evidence, say so explicitly; do NOT agree to be agreeable." Emit `sycophancy_pushback_detected` SSB row when AUDITOR overrides the REASONER's stance based on user-disagreement evidence. Reward signal must NOT be "user said thanks" (Source: openai.com/index/sycophancy-in-gpt-4o/; arXiv:2502.10844). Lives alongside the existing AUDITOR REFINE cap=2 logic.

3. **Pinned-rules visible panel (~250 LOC, new component `crates/dashboard-v2/src/lib/components/PinnedRulesPanel.svelte`).** Renders all user-stated constraints from the session, sourced from rules.db ACTIVE rows + per-session ad-hoc constraints (new SsbActionType `user_rule_pinned`). User clicks to remove a rule; emits `user_rule_unpinned`. Defeats lost-in-the-middle for instructions (aclanthology 2024.tacl-1.9). Lives on the CommandWindow sidebar (collapsed by default, expandable).

4. **Session re-anchor every N turns (~150 LOC across reasoning + cli).** After N turns (default 10, configurable) OR before any destructive action (commit, force-push, file-delete), re-inject system prompt + first user message + active rule list. Surface a `session_re_anchored` SSB row + a "session re-anchored" toast in CommandWindow. Configurable via SettingsWindow. Source: arXiv:2402.10962, arXiv:2412.00804.

5. **Doom-loop detector wired to anti-spiral skill UI (~200 LOC across task_runner + dashboard).** Trigger: 3 same-file edits for the same issue OR 3 failed hypotheses in mcp_think OR auditor returns BLOCK twice in a row OR test-pass-rate plateau across attempts (per `.claude/skills/anti-spiral/SKILL.md`). On trigger: freeze the loop, emit `doom_loop_detected` SSB row, surface a CommandWindow intervention card showing diff of attempts + 3 buttons: "Switch tactic" / "Hand back to user" / "Continue anyway". Source: warpedvisions.org, internal anti-spiral skill.

6. **Evidence-required "done" claim (~120 LOC).** Every completion claim ("I've finished", "done", "task complete", "successfully implemented") in the chat output must reference at least one: (a) file diff link, (b) test result row (auditor verdict SSB), (c) deterministic check pass. AUDITOR stage in ChatReflectPipeline rejects completion claims with no evidence_ref and emits `completion_claim_rejected_no_evidence` SSB row. Renders as a small "evidence:" link cluster under the claim in CommandWindow. Source: HN 47936579.

7. **One-shot clarifier on ambiguous tasks (~150 LOC in `crates/reasoning/src/chat_reflect.rs` TRIAGE stage).** If task description ambiguity > threshold (heuristic floor: <50 chars + missing action verb + no target file/scope; swap to B-38 Ambiguity-Score when shipped), emit exactly ONE clarifier card with up to 3 yes/no questions. Never more than one round. After user response, proceed without further clarification. Emit `one_shot_clarifier_emitted` / `one_shot_clarifier_resolved` SSB rows. Source: arXiv:2507.21285, arXiv:2407.12017.

8. **Terse-by-default + /verbose toggle (~100 LOC across reasoning + cli).** Default output mode: action + 1-line why + 1-line risk. `/verbose` slash command (count 146 -> 147 per B-39 history) enables long-form for the next N turns or until `/terse`. Voice anchor (`agents/voice.toml` from E-3) gets a `[verbosity]` section. Source: Claude 4 system-prompt direction, Anthropic engineering post-mortem 2025-04-23.

9. **Expertise persistence (~100 LOC delta in E-1 memory_notes + ChatReflectPipeline).** Read `fact_type='expertise'` from memory_notes (per E-5 schema) before TRIAGE. Per-language expertise level (Dreyfus 5-stage) calibrates terseness + jargon level. Defaults to `competent` if no row found. Source: arXiv:2504.06808 (expertise-mismatched suggestions actively harm experts).

10. **Refusal with override (~120 LOC).** When ELAI refuses a local-codebase action (e.g. on a security or anti-pattern rule), it MUST: (a) log refusal reason via `refusal_emitted` SSB row, (b) render a "Override" button in CommandWindow that, on click, re-runs the action with the rule bypassed for that single dispatch (and logs `refusal_overridden_by_user` SSB row). Hard-refuse only on out-of-scope-of-project (e.g. modifying files outside `.elai_cc/`). Source: arXiv:2509.08833.

11. **Mid-stream interrupt (~80 LOC delta in CommandWindow.svelte + cli).** During an in-flight chat stream, the user can type a correction in the input box. On Enter mid-stream: cancel the in-flight stream via the existing B-14 `cancel_stream` path, prepend the user correction to the conversation, restart with the corrected context. Emit `mid_stream_interrupt_committed` SSB row. Source: TechRadar / OpenAI rollout, github.com/jupyterlab/jupyter-ai#965.

12. **Honest-correction surface (~80 LOC).** When ELAI realises it was wrong (test failed, doc disagreed), open the correction with "I was wrong. Correct: X because Y." This is in CLAUDE.md already; this task lifts it to the UI as a styled correction block. Emit `honest_correction_emitted` SSB row. Source: anthropic.com/constitution, internal CLAUDE.md.

13. **Intervention pipeline visible control bar (~150 LOC in CommandWindow + ActivityWindow).** `/pause`, `/redirect`, `/cancel` slash commands wired to a visible control bar on the ActivityWindow. The FSM + SSB already supports this (per B-14 INTERVENE card variant); this task makes the bar permanently visible during EXECUTING / AUDITING / RESEARCHING states. Source: developers.googleblog.com on ADK pause/resume, docs.ag-ui.com.

14. **Write decision doc at `workflow/reasoning/E-4-decisions.md`** covering: (a) confidence-chip tier mapping authority (recommend: GRC grounding methods are the source of truth — Verified iff at least one grounding method passed); (b) re-anchor cadence default (10 turns, configurable); (c) doom-loop threshold (3 same-file edits OR 3 failed hypotheses; configurable); (d) clarifier ambiguity threshold (start with heuristic floor, swap to B-38 when shipped); (e) refusal scope policy (local-codebase = override allowed; outside .elai_cc/ = hard refuse, no override).

## WIRING_CONTRACT

produces_symbols:
  - crates::reasoning::chat_reflect::ClaimWithConfidence + ConfidenceTier enum
  - crates::reasoning::chat_reflect::SycophancyGuard (AUDITOR stage helper)
  - crates::reasoning::chat_reflect::OneShotClarifier (TRIAGE stage helper)
  - crates::orchestrator::doom_loop::DoomLoopDetector
  - crates::dashboard-v2::src::lib::components::PinnedRulesPanel.svelte
  - SsbActionType::{ClaimConfidenceTierEmitted, SycophancyPushbackDetected, UserRulePinned, UserRuleUnpinned, SessionReAnchored, DoomLoopDetected, CompletionClaimRejectedNoEvidence, OneShotClarifierEmitted, OneShotClarifierResolved, RefusalEmitted, RefusalOverriddenByUser, MidStreamInterruptCommitted, HonestCorrectionEmitted}
  - /verbose + /terse + /pause + /redirect + /cancel slash commands (count 146 -> 151)
consumes_symbols:
  - crates/reasoning/src/chat_reflect.rs:635 (PRE-stage clarifier + AUDITOR-stage sycophancy guard + completion-claim check)
  - crates/orchestrator/src/task_runner.rs:3996,4645 (doom-loop hook + MistakeBook query for expertise)
  - crates/dashboard-v2/src/lib/windows/CommandWindow.svelte (bubble-header chips + correction block + override button)
  - crates/dashboard-v2/src/lib/windows/ActivityWindow.svelte (control bar)
  - rules.db ACTIVE rules table (PinnedRulesPanel data source)
reachable_from:
  - User submits chat -> ChatReflectPipeline TRIAGE -> OneShotClarifier (if ambiguous)
  - ChatReflectPipeline AUDITOR -> SycophancyGuard + CompletionClaimCheck
  - task_runner per-baby-step boundary -> DoomLoopDetector
  - CommandWindow renders ClaimWithConfidence + override button + correction block
stub_ban:
  - "TODO: add evidence link" literal absent in non-test source
  - "// always-agree" literal absent
  - "fatalError(\"unimplemented\")" or "todo!()" absent
  - any "done" / "complete" / "finished" output path WITHOUT a CompletionClaim-with-evidence wrapper
  - any sycophancy_pushback opportunity SKIPPED without an explicit `no_evidence_for_pushback` SSB row
frontend_pair: crates/dashboard-v2/src/lib/windows/CommandWindow.svelte + new components/PinnedRulesPanel.svelte + extended ActivityWindow.svelte control bar
observable_effect:
  trigger: "elai.bat repl, ask 'use the foobar2000 crate to do X' (a known non-existent crate); then mid-stream interrupt to redirect; then claim 'done' without committing a file"
  expect_ssb: "sycophancy_pushback_detected (model holds position that crate does not exist, doesn't just agree); one_shot_clarifier_emitted (if ambiguity hit threshold); mid_stream_interrupt_committed; completion_claim_rejected_no_evidence on the bogus 'done'"
  expect_artifact: "stdout / CommandWindow shows: confidence chip 'Verified: crate does not exist' on the pushback, override button on any refusal, correction block on any retraction; no bare 'done' without evidence link"

## DETERMINISTIC_CHECK

```bash
# 1. New SsbActionType variants present
rg "claim_confidence_tier_emitted|sycophancy_pushback_detected|user_rule_pinned|user_rule_unpinned|session_re_anchored|doom_loop_detected|completion_claim_rejected_no_evidence|one_shot_clarifier_emitted|one_shot_clarifier_resolved|refusal_emitted|refusal_overridden_by_user|mid_stream_interrupt_committed|honest_correction_emitted" .elai_cc/crates/orchestrator/src/ssb.rs

# 2. PinnedRulesPanel exists and is mounted
test -f .elai_cc/crates/dashboard-v2/src/lib/components/PinnedRulesPanel.svelte
rg "PinnedRulesPanel" .elai_cc/crates/dashboard-v2/src/lib/windows/CommandWindow.svelte

# 3. Slash command count grew from 146 -> 151 (5 new: /verbose, /terse, /pause, /redirect, /cancel)
rg "SLASH_COMMAND_COUNT" .elai_cc/crates/cli/src/live_cli.rs

# 4. ChatReflectPipeline emits ClaimWithConfidence
rg "ConfidenceTier::|ClaimWithConfidence" .elai_cc/crates/reasoning/src/chat_reflect.rs

# 5. DoomLoopDetector triggers and emits SSB
sqlite3 .elai_cc/.elai_cc/ssb.db "SELECT COUNT(*) FROM ssb_events WHERE action_type='doom_loop_detected';"
# After a simulated 3-same-file-edit cycle: >=1

# 6. Completion-claim guard fires
sqlite3 .elai_cc/.elai_cc/ssb.db "SELECT COUNT(*) FROM ssb_events WHERE action_type='completion_claim_rejected_no_evidence';"

# 7. Verify no bare-"done" output paths
rg "println!\\(\"done\"|writeln!.*\"done\"" .elai_cc/crates/cli/src/live_cli.rs
# Expected: any hits are wrapped in evidence-link assertions
```

## ACCEPTANCE

- [ ] Every ChatReflectPipeline AUDITOR output that contains a factual claim renders with a ClaimWithConfidence pill (Verified / ReadFromSource / FromTraining / Guessed)
- [ ] SycophancyGuard fires (emits sycophancy_pushback_detected) on a planted false-claim test fixture (user asserts known-false fact, model holds position)
- [ ] PinnedRulesPanel renders all ACTIVE rules + user-pinned ad-hoc constraints; click-to-remove emits user_rule_unpinned
- [ ] Session re-anchor fires at N=10 turns (default); session_re_anchored SSB row + toast appears
- [ ] DoomLoopDetector fires within 3 same-file edits for same issue; intervention card with 3 buttons renders
- [ ] Every "done" / "complete" output references at least one of: file diff, audit verdict SSB row, deterministic check pass
- [ ] OneShotClarifier fires at most once per task on ambiguous input; never more
- [ ] /verbose, /terse, /pause, /redirect, /cancel slash commands wired and visible in CommandPalette
- [ ] Mid-stream interrupt cancels in-flight stream + commits user correction + restarts with corrected context
- [ ] Refusal with override: every refusal_emitted SSB row has a paired UI affordance for refusal_overridden_by_user
- [ ] Honest-correction block renders when ELAI explicitly retracts a prior claim
- [ ] No new dependency added; all components use Svelte 5 runes only (no `$:` labels, no createEventDispatcher)

## Tests to add

- `crates/reasoning/tests/sycophancy_guard_holds_position.rs`: planted-false-claim fixture, assert sycophancy_pushback_detected SSB row.
- `crates/reasoning/tests/one_shot_clarifier_exactly_once.rs`: ambiguous task, assert one clarifier emitted then proceed.
- `crates/reasoning/tests/completion_claim_evidence_required.rs`: "done" output without evidence_ref, assert rejected.
- `crates/orchestrator/tests/doom_loop_detector_threshold.rs`: simulate 3 same-file edits, assert detector fires.
- `crates/contracts/tests/claim_with_confidence_round_trip_contract.rs`: serde round-trip + SSB shape.
- `crates/dashboard-v2/src/lib/components/PinnedRulesPanel.test.ts`: render with N rules, click-to-remove emits unpin event.
- `crates/dashboard-v2/src/lib/windows/CommandWindow.test.ts`: mid-stream interrupt e2e — submit, mid-stream type correction, assert cancel_stream fired + restart with correction prepended.
- `crates/contracts/tests/refusal_with_override_contract.rs`: refusal_emitted -> click override -> refusal_overridden_by_user.

## Files touched

- CREATE: `crates/dashboard-v2/src/lib/components/PinnedRulesPanel.svelte` (~250 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/InterventionControlBar.svelte` (~140 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/HonestCorrectionBlock.svelte` (~70 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/ClaimConfidenceChip.svelte` (~80 LOC)
- CREATE: `crates/orchestrator/src/doom_loop.rs` (~200 LOC, DoomLoopDetector + thresholds)
- CREATE: `workflow/reasoning/E-4-decisions.md` (~120 LOC)
- MODIFY: `crates/reasoning/src/chat_reflect.rs:635` (~350 LOC: ClaimWithConfidence + SycophancyGuard + OneShotClarifier + CompletionClaimCheck)
- MODIFY: `crates/orchestrator/src/task_runner.rs:3996,4645,7776` (~150 LOC: doom-loop hook + expertise read + completion-claim audit)
- MODIFY: `crates/orchestrator/src/ssb.rs` (~80 LOC: 13 new SsbActionType variants + as_sql_str entries)
- MODIFY: `crates/orchestrator/src/fsm.rs` (~40 LOC: session_re_anchor trigger hook)
- MODIFY: `crates/dashboard-v2/src/lib/windows/CommandWindow.svelte` (~200 LOC: confidence chips + correction block + override button + mid-stream interrupt handler)
- MODIFY: `crates/dashboard-v2/src/lib/windows/ActivityWindow.svelte` (~80 LOC: InterventionControlBar mount)
- MODIFY: `crates/dashboard-v2/src/lib/layout/CommandPalette.svelte` (~50 LOC: /verbose /terse /pause /redirect /cancel slash entries)
- MODIFY: `crates/cli/src/live_cli.rs` (~180 LOC: slash command handlers + terse-by-default rendering toggle)
- MODIFY: `.elai_cc/agents/voice.toml` (from E-3) (~15 LOC delta: [verbosity] section)

## Citations / Grounding

- arXiv:2310.13548 — Sharma et al. Towards Understanding Sycophancy (PRIMARY for Task 2)
- arXiv:2412.02802 — Flattering to Deceive: Sycophancy and User Trust
- arXiv:2502.10844 — Be Friendly Not Friends (CHI 2026) — neutral LLMs that adapt enhance trust; complimentary LLMs reduce it
- openai.com/index/sycophancy-in-gpt-4o — PRIMARY for sycophancy anti-pattern (April 2025 rollback)
- arXiv:2402.10962 — Measuring and Controlling Persona Drift (PRIMARY for Task 4)
- arXiv:2412.00804 — Examining Identity Drift in LLM Agents (8-15 turn degradation)
- aclanthology.org/2024.tacl-1.9 — Lost in the Middle (PRIMARY for Task 3)
- arXiv:2604.19245 — Talking to a Know-It-All GPT (repair-resistance)
- arXiv:2503.15050 — Studying ChatRepair failures
- arXiv:2509.08833 — Pitfalls of Over-Alignment (PRIMARY for Task 10)
- arXiv:2507.21285 — Curiosity by Design (PRIMARY for Task 7)
- arXiv:2503.02623 — Rewarding Doubt (PRIMARY for Task 1)
- arXiv:2505.24858 — MetaFaith linguistic uncertainty markers
- arXiv:2412.02904 — uncertainty signaling
- arXiv:2604.22417 — Trust as Situated User State (Snapchat My AI 4-week study)
- arXiv:2504.14112 — Longitudinal Study Social/Emotional AI (5-week +33pp attachment)
- arXiv:2504.10050 — Emotional Strain and Frustration in LLM (EASE 2025)
- arXiv:2509.08912 — User-Reported Risks "In the Wild"
- arXiv:2402.02136 — User Intent Recognition and Satisfaction with ChatGPT
- arXiv:2504.06808 — Copilot Suggestion Impact on Developer Frustration (Task 9 expertise mismatch)
- HN 47936579 — Claude Code false-completion thread (PRIMARY for Task 6)
- HN 39988419 — Claude over-refusal complaints (Task 10)
- anthropic.com/constitution — "diplomatic honesty rather than dishonest diplomacy" (Task 12)
- wattenberger.com/thoughts/boo-chatbots — affordances > prompts
- maggieappleton.com/lm-sketchbook — chatbot is the lazy solution
- allenpike.com/2025/post-chat-llm-ui — Post-Chat UI (Task 13 control bar)
- warpedvisions.org/blog/2025/getting-unstuck-when-using-llms-for-coding (Task 5)

## Open questions / risks

- Sycophancy guardrail false-positive rate is unknown — overcorrection could cause the model to push back on correct user statements (per arXiv:2502.10844, "complimentary LLMs reduce authenticity; neutral LLMs that adapt enhance both"). Mitigation: D-23 sycophancy detection benchmark when shipped will give detection-rate >=70%, false-positive-rate <=15% targets to measure against.
- Doom-loop threshold (3 same-file edits) may fire too eagerly on legitimate refactor iteration. Mitigation: env-configurable threshold; emit `doom_loop_threshold_adjusted` SSB row when user dismisses.
- One-shot clarifier with heuristic floor (length + verb-presence) is brittle pending B-38 Ambiguity-Score. Mitigation: dual-gate (heuristic floor OR B-38 score when available).
- Refusal-with-override creates a security surface: user can override anti-pattern rules. Mitigation: out-of-scope-of-project (outside .elai_cc/) is hard-refuse, no override; document in decision doc.
- Mid-stream interrupt requires the in-flight runtime to handle cancel cleanly across all 11 RuntimeAdapter backends. Verify each adapter's cancel path during Task 11; AnthropicApi + Sidecar are known good (B-14); CLI-subprocess backends may need explicit kill().
- Expertise persistence (Task 9) depends on E-5 schema landing first. Soft dependency; ship Task 9 reading a default `competent` until E-5 lands the expertise rows.
- The report itself flagged this as risky: "Never agree with the user when you have evidence they're wrong. Disagree with citation." Task 2 implements this — but mis-calibrated, it can cause user frustration ("model contradicts me on something I know to be true"). The cure is calibration on the confidence-tier surface (Task 1) — if model has no Verified-tier evidence, it must NOT pushback.

## Next

E-3 (THIS PLAN) supplies the unified voice anchor (voice.toml) that holds the sycophancy_guard + verbosity clauses Task 2 + 8 consume. E-1 supplies the memory_notes substrate Task 1 + 9 read/write through. E-5 supplies the expertise-level row Task 9 reads. D-14 / D-23 (PLANNED) measure Task 2's sycophancy push-back rate. B-38 (PLANNED) replaces Task 7's heuristic floor with a continuous ambiguity score. A follow-up E-N can take Task 5's doom-loop detector and wire it to the persona-switcher in `.claude/skills/anti-spiral/` (Hacker / Researcher / Simplifier / Architect / Contrarian) once we have telemetry on which persona breaks which loop pattern.
