E-4 chat UX critique — decisions log

Plan: docs/thefinalglance/E-plan/E-4_chat_ux_critique.md
Session: s-25120 (completion-drive cluster, resumed after 529 mid-stream)
Author: executor (main LLM)
Date: 2026-05-28

(a) Confidence-chip tier mapping authority
Decision: the GRC grounding-method ledger is the single source of truth.
  - Verified: at least one of FILE_CHECK / SOURCE_CHECK / TEST_CHECK /
    WEB_CHECK passed for the claim, recorded via mcp_think grounding=
    "file:..." / "test:..." / "search:...".
  - ReadFromSource: the AUDITOR cites a file:line that exists in this
    repo but no det-check was run on the cited assertion.
  - FromTraining: no grounding method called; claim originates in the
    model's training data.
  - Guessed: explicit mcp_think(step_type="status") echo with low
    confidence, OR the AUDITOR self-labels via the inline pill.
The 4-tier shape aligns with CLAUDE.md "Confidence calibration"
(Verified / High confidence / Uncertain) with one extra "guessed"
bucket the canonical rule does not name. The Rust substrate enforces
this via ConfidenceTier in crates/orchestrator/src/chat_ux/mod.rs.

(b) Re-anchor cadence default
Decision: SessionReAnchorPolicy { turn_threshold: 10,
  fire_before_destructive: true }.
Rationale: arXiv:2412.00804 measured 20-40% persona drift in 8-15
turns; 10 turns sits inside that window with room to drift but
catches it before the lost-in-the-middle effect (aclanthology
2024.tacl-1.9) hardens. fire_before_destructive=true mirrors the
git-commit pre-flight rule in CLAUDE.md commit-cadence.

(c) Doom-loop threshold
Decision: DoomLoopDetector defaults:
  same_file_edit_threshold = 3
  failed_hypothesis_threshold = 3
  auditor_block_streak_threshold = 2
The thresholds match .claude/skills/anti-spiral/SKILL.md ("Same file
edited >3 times for same issue" + "If 3 hypotheses fail, add debug
logging" + "auditor returns BLOCK twice in a row"). The detector is
a thin facade over crate::anti_spiral so the 4-pattern detector
(Spinning / Oscillation / NoDrift / DiminishingReturns) is not
duplicated. DoomLoopPattern variants (SameFileEditStorm /
HypothesisStreak / AuditorBlockStreak / PassRatePlateau) carry the
chat-UX-specific labels for the dashboard intervention card.

(d) Clarifier ambiguity threshold
Decision: heuristic floor (length<50 chars AND no leading action verb
AND no scope hint) for the OneShotClarifier. Fires at most once per
task — the emitted flag is sticky until a new task instance is built.
Will swap to B-38 Ambiguity-Score continuous metric (threshold <=0.2)
when B-38 ships; the heuristic floor stays as a fallback if the
score is unavailable.
Action-verb whitelist is the small set of 28 verbs in
leading_clause_has_action_verb; scope hints look for '/', '.rs',
'.ts', '.svelte', '.py', 'crate::', 'crates/' substrings. Both are
explicitly noted as B-38 swap-out targets.

(e) Refusal scope policy
Decision: RefusalScope::LocalCodebase is user-overrideable
(`Override` button renders; refusal_overridden_by_user SSB row on
click). RefusalScope::OutsideProject is hard-refuse — no button, no
override path. Out-of-scope-of-project is defined per CLAUDE.md
Project-boundary HARD RULE: any path outside .elai_cc/ except
workflow/reasoning/. RefusalScope::is_overrideable() encodes this.

Cancel-surface decision (mid-stream interrupt, Task 11)
RuntimeAdapter trait has NO cancel() method (verified at
crates/providers/src/runtime/types.rs:554-581). Mid-stream
cancellation rides BoxStream drop semantics:
  - AnthropicApi: hyper drops connection on stream drop (B-14 verified).
  - Sidecar: mpsc receiver closes on stream drop (B-14 verified).
  - 9 CLI-subprocess adapters per E-37: drop-semantics UNVERIFIED.
Decision: ship the chat_ux substrate today with the InterruptCommit
record + dashboard surface; gate live mid-stream interrupt on the
existing AnthropicApi + Sidecar backends; add a follow-up regression
test for the 9 CLI-subprocess adapters' drop-on-stream behavior
before promoting Task 11 out of wire-dark.

Sycophancy-guard vs existing sycophancy panels — integration policy
The pre-existing 3 sycophancy surfaces stay; E-4 ADDS, never replaces:
  - SycophancyFlagStrip (D-27 GAP-1): per-row D-14 ProbeVerdict flag
    history. Lives wherever it is mounted today; E-4 does not move it.
  - SycophancyMatchedPairPanel (E-54): matched-pair benchmark visualizer.
    Stays in MonitorWindow at the existing mount.
  - SycophancyBenchPanel (D-23): D-14 benchmark run results. Stays.
  - output_postprocessor::strip_sycophancy (regex stripper, removes
    flattery tokens from output). DISTINCT from SycophancyGuard.
SycophancyGuard (NEW, crates/orchestrator/src/chat_ux/mod.rs) adds the
GUARD behavior at AUDITOR refine loop: holds position when evidence
supports it, emits no_evidence_for_pushback SSB row when no evidence.
SycophancyGuardBadge (NEW, dashboard) renders the verdict on the
bubble itself. The matched-pair panel remains the bench-side
observability surface; the badge is the live-session surface.

SCHEMA bump path
SCHEMA 175 -> 178 (E-38 baseline at 175, prior cluster sweep took it to
177; this E-4 atom bumped to 178 via ChatUxCritiqueEvent Pattern-2
single-variant with 13-arm ChatUxCritiqueKind discriminator). 13-arm
exceeds the prior 9-arm joint-max (E-25 + E-55), accepted as the new
joint-max because the alternative (13 separate SsbActionType variants)
would cascade 130+ golden ordinal shifts. 30th application of the
Pattern-2 envelope discipline locked 2026-05-28.

Plan-line-ref drift acknowledgement
Plan body lines 36/37 cite chat_reflect.rs:635 and
task_runner.rs:3996/4645/7776. Both are STALE per the cross-check
report. Real anchor sites at execute time:
  - chat_reflect.rs AUDITOR refine loop: roughly lines 379-393.
  - task_runner.rs derive_autonomous_evidence: defn line 11379,
    call line 6438.
Decision: future producer-side wiring (the actual SsbWriter calls
from chat_reflect + task_runner) re-anchors at edit time. The
substrate landed in chat_ux/mod.rs is anchor-agnostic; only the
producer hooks need re-grep.

Paper-citation verification (per feedback_paper_citation_verify)
Verified via WebSearch 2026-05-28:
  - arXiv:2604.19245 "Talking to a Know-It-All GPT or a Second-Guesser
    Claude" by Lachenmaier / Bultmann / Zarrieß (Apr 2026). EXISTS.
  - arXiv:2604.22417 "Trust as a Situated User State in Social
    LLM-Based Chatbots: A Longitudinal Study of Snapchat's My AI" by
    Landerberg / Flatmo / Said (Apr 2026). EXISTS.
The 2604.* prefix is genuine April 2026 publication, not hallucination.
Other arXiv IDs in the plan body (2310.13548 Sharma sycophancy +
2503.02623 Rewarding Doubt + 2507.21285 Curiosity by Design etc.) are
cited inline in the chat_ux/mod.rs doc-comments; further verification
deferred to the auditor pass.

Slash-command count deterministic-check amendment
Plan's deterministic_check #3 references SLASH_COMMAND_COUNT 146->151.
The const does NOT exist in crates/cli/src/live_cli.rs (verified).
Decision: replace check #3 with rg literal-match on the 5 new slash
strings (/verbose /terse /pause /redirect /cancel) once the cli side
of E-4 lands. The dashboard side shipped today does not depend on the
cli slash count.

Dashboard component checklist (Svelte 5 runes only)
  - PinnedRulesPanel.svelte: sidebar strip above stream in
    CommandWindow.svelte. let { ... } = $props(); $state; $derived;
    onclick callback prop. NO createEventDispatcher, NO $: labels.
  - InterventionControlBar.svelte: bottom-centered overlay in
    ActivityWindow.svelte; auto-hidden outside EXECUTING / AUDITING /
    RESEARCHING via own visibility derived. Reads `fsm.state` from
    $lib/state/fsm.svelte for the FSM gate.
  - ClaimConfidenceChip.svelte: bubble-inline; DISTINCT from
    ConfidenceTag (orbit verdict chip). Different prop shape.
  - HonestCorrectionBlock.svelte: bubble-inline; styled retraction
    block with strike-through prior + corrected text.
  - SycophancyGuardBadge.svelte: bubble-inline; renders the guard
    verdict (hold_position with evidence OR no_evidence).
All 5 components avoid $: labels, createEventDispatcher, export let.
Rune-shadow risk mitigated by never naming a $state var with the same
identifier as a prop (lesson learned from ContextStatePanel
batch6-fixer panelState drift).

Scope kept narrow (substrate-only delivery)
The dashboard side ships the 5 components + 2 mounts + types in
crates/dashboard-v2. The producer side (chat_reflect.rs AUDITOR-stage
emission, task_runner.rs hook, commands_e4.rs Tauri commands, slash
command handlers in live_cli.rs, voice.toml [verbosity] section) is
DEFERRED to a follow-up patch. Today's deliverable is the substrate
+ visible-but-dark wiring so subsequent producer-side patches have
the surface to plug into. ELAI_CHAT_UX_CRITIQUE_ENABLE gate is the
release lever.
