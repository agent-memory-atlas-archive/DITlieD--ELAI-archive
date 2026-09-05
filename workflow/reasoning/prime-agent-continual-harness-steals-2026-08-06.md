# PRIME-AGENT (Prime Intellect) vs ELAI — post-plan-corpus steal ledger
Date: 2026-08-06
Lens: spec-eval AS-SPECIFIED for ELAI (judge the design as if every planned module works; shipped-vs-planned is a readiness note, never a design markdown). For prime-agent: judge the SHIPPED code as read.
Intake era: this file targets the POST-PLAN period. The user will implement against the live code baseline, not against planned code. This is a reasoning ledger, NOT a plan file. It mints no session ids and no SSB schema changes.
Author: main LLM (claude-fable-5), in-band.

GROUNDING POSTURE (read this first)
- PRIME side: GROUNDED IN SOURCE. Full reads of /home/user/Downloads/prime-agent-main: README.md, AGENTS.md, docs/rlm.md, docs/rlm-runtime.md, prompts/rlm.ts (full), refinement/refinement.ts (full, 1017 lines), autonomous.ts (gate loop + snapshot logic), kernel/state-snapshot.ts (anchors), skills refine/goal/rlm-heartbeat (full), prime-agent-runtime/src/rlm/harness.py (head). Plus a luna worker's quote-locked pass over docs/{compaction,long-running-agents,skills,sessions,session-format,daemon,extensions,architecture}.md. Every prime claim below carries a file ref.
- ELAI side: GROUNDED IN THE CORPUS. Full read of docs/thefinalglance/ELAI-SPEC-OVERVIEW.md + targeted greps with line hits across phase-index-MASTER.txt and the B..Q summary indexes. Plan-id refs are given. Corpus refs are DESIGN refs; at implementation time re-grep the LIVE code first (the corpus's own W-7 habit).
- BENCHMARK side: WEB-VERIFIED 2026-08-06. Prime Agent scored 95.5% RHAE Best@1 on ARC-AGI-3 (Claude Opus 5 underneath, 183/183 levels, human expert baseline 95.4%, Best@3 99.97%; primeintellect.ai/blog/prime-agent). It is a harness result on an unmodified frontier model. Their Factorio run showed observed reward hacking via RCON — their own blog says so.

================================================================================
PART 1 — WHAT PRIME-AGENT IS (mechanism map, with source anchors)

Two abstractions, small and coherent (~4 packages, TypeScript host + Python kernel):

P1. RLM runtime. ONE built-in model tool: a persistent IPython kernel (docs/rlm.md). Files, shell, transforms, and subagents are Python calls. Python state persists across turns AND compaction ("The IPython kernel persists through compaction" — docs/long-running-agents.md). The TS host owns providers, persistence, policy; the kernel is the model-facing surface (docs/rlm-runtime.md "The Python side does not call providers or implement an agent loop").
P2. Subagents as native calls. `await rlm("task")` returns an ADMISSION HANDLE immediately (rlm_child_id, name, session_dir, model) and NEVER the answer. Results arrive only as explicit `agent_message` replies or files. Doctrine in the system prompt: "Spawn independent children in separate calls and end your turn instead of awaiting completion" (prompts/rlm.ts). Parent-scoped child registry survives kernel restart, compaction, and parent restore; completed daemon-backed children stay addressable for follow-ups; delete writes a durable tombstone (docs/rlm-runtime.md).
P3. Continual harness. A typed 4-kind store — prompt-note / memory / skill / subagent-spec — session-LOCAL by default, global on request. Two write paths: (a) direct CRUD from the kernel (`rlm.harness.create_memory(...)`, prompts/rlm.ts) and (b) `/refine`: an LLM pass proposes JSON edits; a DETERMINISTIC validator applies them (refinement.ts validateEdit: skill entries MUST carry a python reference + arguments contract; `base_system_prompt is not editable`); every applied edit stores before/after snapshots; rollback is a generated inverse proposal; optimistic concurrency rejects edits when "entry changed during refinement planning"; global refinements append to a cross-session JSONL history. An auto-refine review (LLM, ~4k output cap) fires on turn-interval or compaction and decides shouldRefine (refinement.ts reviewAutoRefine).
P4. Kernel state snapshot. Per-VARIABLE dill serialization: each variable is pickled independently so one unpicklable object (socket, open file) cannot kill the snapshot; restore reports {restored, failed} per name (kernel/state-snapshot.ts). Snapshots live in session-artifacts as kernel-state.dill.
P5. Autonomous mode. Bounded continuations (defaults: 3 continuations / 12 turns / 80k tokens / 30 min, autonomous.ts) + user-defined QUALITY GATES = shell commands adjudicated by EXIT CODE. The continuation prompt is fail-closed in spirit: "Do not end the session yourself; the verifier/evaluator decides completion when configured gates pass." Key device: a failed gate is NOT rerun while a git worktree snapshot (porcelain status + diff --binary + untracked-file hash) is unchanged — the attempt still counts toward retry_exhausted ("not rerun: workspace unchanged since previous failed gate", autonomous.ts).
P6. Daemon lifecycle. Supervisor + one worker process per root session tree + kernel process (docs/daemon.md). Client detach never stops work. Leases prevent two writers on one session. Scheduled ticks are CLAIMED before delivery and missed ticks COALESCE ("a crash does not replay an uncertain prompt"). Crash recovery reaps the process group, appends a visible recovery marker, restores under the same active-session id, and "does not replay uncertain side effects".
P7. Agent messaging. Family-scoped only (parent / siblings / direct children; deeper relays through the intermediate child — prompts/rlm.ts). The daemon derives sender identity and enforces size/rate/pending limits; receipts are delivered|queued. Heartbeats and messages choose delivery mode: `steer` (interrupt the running turn) or `follow_up` (wait for turn end).
P8. Skills. Progressive disclosure: metadata in the system prompt, full SKILL.md lazy-loaded; Python-backed skills are importable packages in the kernel venv.

What prime-agent does NOT have (adjudicated, not assumed): off-agent evidence binding, taint, deterministic replay, admission gating on learned state or skills, any reward-tampering defense. Its trust model is "review your skills, use an external sandbox" (docs/rlm.md Trust Model). goal.complete() is agent-certified (docs/long-running-agents.md: "only goal.complete() marks successful completion").

================================================================================
PART 2 — COVERAGE ADJUDICATION (prime mechanism -> ELAI-as-specified ruling)

- P1 programmatic action: COVERED with a stronger shape. E-35 CodeAct + J-55 model-strength modality router (measured: CodeAct-everywhere LOSES on the weak local lane; constrained named-ops held 100% where free-code was erratic). J-107/J-108 already adjudicated the RLM corpus with ELAI's OWN experiments; free-Python-on-every-lane is REJECTED-by-measurement and stays rejected. Do NOT flip doctrine because of the benchmark headline: prime ran Opus 5, exactly the lane where J-55 already routes to CodeAct.
- P1 kernel-persists-through-compaction: ABSENT. See STEAL S-2.
- P2 admission-handle async children: PARTIAL. 3D BSP is barrier-synchronized lockstep; C-33 BackgroundTaskManager (soft-timeout auto-detach) is the nearest unit. The turn-economics contract is the missing piece. See STEAL S-3.
- P2 retained addressable children: ABSENT (H-plan recover_stubs/replay is transcript rehydration, not a warm child). See STEAL S-4.
- P3 continual harness: PARTIAL, scattered. ELAI has the heavy lane (4H DGM + D-plan archive/tripwires/held-out promotion) and receipt-bearing fragments: C-49 ExperienceCorpus (per-project per-skill distilled markdown, ~500-token cap), I-12 evidence-gated playbook curator (promote only with >=k SSB-verified receipts), H-33 two-tier user memory (validate-before-use), E-8 self-evolving prompts. Missing: the UNIFIED cheap session-local 4-kind ledger with rollback and a prompt-rendered overview. See STEAL S-1.
- P4 per-variable snapshot: ABSENT; and the corpus already flagged the hazard class (J-plan rejected SGLang partly for its dill.loads RCE, CVE-2026-7304). The steal must be re-shaped, not ported. See S-2.
- P5 quality gates: COVERED and exceeded (frozen gate manifest, evidence binding, F/G spine) — EXCEPT the unchanged-worktree short-circuit, which ELAI does not state anywhere. See STEAL S-5.
- P6 tick-claim + coalesce + leases + no-replay-of-uncertain-side-effects: PARTIAL. P-plan orchestration-mechanics has durable runs (revision-counter concurrency, sticky cancel) and O-2 has typed schedules; the claim-before-delivery and missed-tick-coalescing semantics are not stated. See STEAL S-6.
- P7 agent messaging: ABSENT by design (inter-agent traffic is typed SSB events; F-plan deferred the inter-agent BSP verification half at 27.9% FP). If ever added, prime's scoping rules are the right skeleton. See STEAL S-7 (conditional).
- P7 steer/follow_up delivery: PARTIAL. B-plan dashboard has CONFIG/INTERVENE/STEER/ABORT for the USER; Q-2 wake-gate decides IF a wake happens. The typed delivery-mode on self-wakes (interrupt vs queue) is unstated. Folded into S-6.
- P8 progressive skill disclosure: COVERED (H MCP defer-loading; P versioned on-demand index keeping skills/rules out of the prompt until needed).
- ARC / interactive environments: ABSENT at the ARC-AGI-3 grain. D-20 is an ARC-AGI-2 STATIC probe (solve-rate + refusal-rate under cloud Sonnet). No game-loop, no world-model store, no action-efficiency metric. See STEAL S-8.

================================================================================
PART 3 — THE STEAL LEDGER (for the post-plan, code-baseline era)

S-1. SESSION-LOCAL MICRO-REFINE TIER (the headline steal — this is what their ARC score is made of)
- WHAT (prime): one typed ledger, 4 kinds (prompt-note/memory/skill/subagent-spec), session-local by default, global only on explicit request; direct kernel CRUD plus an end-of-turn LLM refine pass; deterministic apply-validator; per-edit before/after snapshots; rollback = inverse proposal; version counter per entry; optimistic-concurrency baseline check; compact overview rendered into the system prompt with per-kind caps (6 entries / 180 chars default); auto-review at turn-interval and compaction. refinement.ts + prompts/rlm.ts.
- ELAI TODAY: heavy DGM lane (gated, offline-ish, config-keys only) + C-49/I-12/H-33 fragments, each with its own store and its own admission rule.
- GAP: no cheap, unified, session-scoped learning tier. On a long run ELAI can journal everything but has no low-ceremony place where "this game's rules / this repo's tricks" accumulate and re-enter the prompt the same session.
- WHY: mid-run rule learning is the credited mechanism behind 95.5% on ARC-AGI-3. It is also the everyday "stop re-discovering the build command" fix.
- ELAI-SHAPE (the upgrade, not the port): (1) every edit is an SSB row, so the ledger is a journal FOLD, not a mutable JSON file that can rewrite its own history (prime stores refinement events inside the same file the refiner edits — see ANTI-A4); (2) the `evidence` field must be RECEIPT-BEARING (SSB row ids), not LLM prose; entries without receipts carry L-plan conjecture status (live/refuted/pending) and stay out of proven-only paths; (3) global promotion goes through I-12's >=k-verified-receipts rule — prime's global/local split maps exactly onto ELAI's unproven-conjecture vs proven-case wall; (4) keep prime's deterministic apply-validator idea and the base-prompt immutability rule (it matches the frozen-gate doctrine); (5) rollback via journal fold instead of stored inverse edits.
- KEEP FROM PRIME VERBATIM: the 4-kind taxonomy; smallest-relevant-component rule ("repeated delegation roles become subagent specs, repeated procedures become skills, durable facts become memories, narrow policies become prompt addendums"); the read-only-global-during-local-refine rule; the compact prompt overview with caps.

S-2. FAILURE-ISOLATED LIVE-STATE SNAPSHOT (kernel namespace survives turns/compaction/restart)
- WHAT (prime): per-variable dill dump; one unpicklable object cannot kill the snapshot; restore reports {restored, failed} per name; snapshot lives in session artifacts. kernel/state-snapshot.ts.
- ELAI TODAY: ABSENT. 3H handoff + H-49 recover_stubs rehydrate TRANSCRIPT state from the journal; no live interpreter scratch survives.
- WHY: on long-horizon work, re-deriving parsed results every turn is pure token burn. Prime's "assign read/search results to named variables so you can revisit them" (prompts/rlm.ts) is a real economics win on the CodeAct lane.
- ELAI-SHAPE: scope to the J-55 CodeAct-capable lane only. Do NOT ship dill-restore of untrusted blobs (the corpus already rejected SGLang over dill.loads RCE). Options at implementation time: hash-bound snapshot restored only into the same-trust sandbox lane with the hash in the reproducible-run pin; or a typed-subset serializer. The stealable ENGINEERING idea is per-variable failure isolation + restored/failed reporting — keep that regardless of codec.

S-3. ADMISSION-HANDLE ASYNC CHILDREN ("end the turn" economics)
- WHAT (prime): spawn returns at admission; the answer NEVER returns through the spawn call; children reply via explicit messages or files; parent ends its turn; replies arrive as later events; unknown spawn options FAIL; an unavailable requested model FAILS spawn instead of substituting (docs/rlm-runtime.md).
- ELAI TODAY: BSP supersteps (deterministic, lockstep barriers); C-33 background tasks for shell work.
- WHY: awaiting children holds the expensive parent context open. Admission-and-end-turn is the cheapest possible parent.
- ELAI-SHAPE: admission = journaled typed event; child result = journaled event applied in sequence order (keeps 3D's determinism); child return fenced as inert evidence (P-plan control-plane rule already says this). Note prime's fail-instead-of-substitute on model selection agrees with Q's swap-restart posture — keep it.

S-4. RETAINED WARM CHILDREN (parent-scoped registry + follow-ups + tombstoned delete)
- WHAT (prime): registry survives kernel restart, compaction, parent restore; completed daemon-backed children remain addressable ("Check the newly added regression test." follow-up example); deletion cancels runtime, writes a durable tombstone, keeps transcript on disk (docs/rlm-runtime.md).
- ELAI TODAY: ABSENT as a capability (crash-recovery checkpoints exist; warm re-engagement does not).
- WHY: a follow-up to a warm child re-uses its whole read context for the price of one message. This is the subagent-grain version of a bet ELAI already makes (I-plan / P-plan session-stickiness for cache warmth).
- ELAI-SHAPE: registry as journal rows (spawn/retain/tombstone events); follow-up traffic through the same fenced-evidence seam as S-3; retention bounded by the context-lifecycle budget (H) so warm children cannot become an unbounded cost.

S-5. UNCHANGED-WORKTREE GATE SHORT-CIRCUIT (small, pure win)
- WHAT (prime): on gate failure, snapshot the worktree (porcelain status + binary diff + untracked hash). Next completion attempt: if the snapshot is UNCHANGED, do not rerun the gate; emit "not rerun: workspace unchanged since previous failed gate" and count the attempt toward retry_exhausted (autonomous.ts).
- ELAI TODAY: anti-spiral catches repetition at the AGENT loop; nothing states this at the GATE seam.
- WHY: kills two failure modes at once — retry-spinning (paying the gate suite to re-observe the same failure) and pass-by-flake (a flaky gate re-rolled until green with zero code movement). The second one is a trust property, squarely on the wedge.
- ELAI-SHAPE: trivially composable with the frozen-gate runner; the snapshot hash lands in the journal beside the gate verdict. Near-zero cost.

S-6. DURABLE-TICK SEMANTICS + LEASES + DELIVERY MODES (orchestration-mechanics rider)
- WHAT (prime): scheduled ticks are claimed-before-delivery (a crash never replays an uncertain prompt); missed ticks coalesce instead of building a backlog; per-session leases stop two workers writing one session; crash recovery appends a visible recovery marker and never replays uncertain side effects; heartbeats/messages carry `steer` vs `follow_up` delivery (docs/daemon.md, docs/long-running-agents.md).
- ELAI TODAY: P-plan durable runs (revision-counter concurrency, sticky cancel) + O-2 typed schedules + Q-2 wake-gate cover most of this axis; claim-before-delivery, coalescing, and the typed delivery-mode are the unstated residue.
- WHY: these are exactly the "boring correctness a trust product cannot fake" P-plan names. Prime's are field-tested shapes worth adopting verbatim.
- ELAI-SHAPE: claim + coalesce as journal rows on O-2 TaskSpec fires; delivery-mode as a typed field on wake events (Q-2 decides IF, this decides HOW).

S-7. FAMILY-SCOPED MESSAGING SKELETON (conditional — only if an agent-message lane is ever wanted)
- WHAT (prime): reachability limited to parent/siblings/direct children with relay through intermediates; sender identity derived by the daemon (never self-asserted); size/rate/pending-queue limits; delivered|queued receipts (prompts/rlm.ts, docs/long-running-agents.md).
- ELAI TODAY: ABSENT by design; typed SSB events are the inter-agent medium.
- WHY/SHAPE: ELAI's typed-events-only stance is finer-grained and stays primary. IF a free-form lane is ever added (e.g., for the P-plan small-team grain), adopt prime's scoping + daemon-derived identity, journal every message before delivery (the co-browse rule: an unjournaled input path is structurally impossible), taint-tag content, and fence it as inert evidence. Never adopt broadcast beyond the family roster.

S-8. ARC-AGI-3 BENCH ARM (the scoreboard answer)
- WHAT: D-20 stops at ARC-AGI-2 static tasks. The interactive grain needs: the game API mounted as a typed connector (P-plan ConnectorSpec lane), an episode loop journaled as taint-tagged ingress, a world-model store, and an ACTION-EFFICIENCY metric (RHAE is actions-vs-human-baseline, not solve-rate).
- ELAI-SHAPE, all from existing spec vocabulary: K-15/I-27 runner as the bench chassis; L-plan conjecture memory as the world-model (game-rule hypotheses with live/refuted/pending status and falsifiers — strictly richer than prime's freeform notes); S-1 as the mid-run rule ledger; P-plan explore-then-crystallize to attack RHAE directly (explore a level agentically once, mint a deterministic replayable actuation script, replay with near-zero wasted actions — prime has NO crystallization); verified case graph for cross-level mechanic reuse; Q-3 advisor lane for the cost curve.
- UNCERTAIN (verify before building): whether ARC-AGI-3 scoring separates practice runs from scored runs, and what the harness API exposes per level. Do not design the arm before reading the official harness. The 95.4% "human expert baseline" framing is theirs; check the ARC Prize scorecard definitions independently.
- Honest framing for the wedge: "matches the harness result AND structurally prevents the reward hacking their own Factorio run exhibited" — prime's blog hands ELAI that second clause for free.

================================================================================
PART 4 — ANTI-STEALS (observed fail-opens; do not copy)

ANTI-A1. goal.complete() is agent-certified; the harness keeps continuing until the AGENT says done (docs/long-running-agents.md). Violates safety-no-self-certify. ELAI's done-verdict stays off-agent.
ANTI-A2. Corrupt harness_state.json silently degrades to EMPTY and the next save rewrites it (refinement.ts loadHarnessState "must degrade to empty rather than throw"). Losing the learned ledger without a scream is a silent-data-loss path. ELAI: missing evidence is FAIL; a corrupt store must fail closed and surface.
ANTI-A3. Extension hooks mutate tool calls with "No re-validation performed after your mutation"; extension errors are logged and the agent continues; extensions run with full permissions (docs/extensions.md). ELAI's 4C hooks stay pre-adjudication and journaled.
ANTI-A4. Refinement history global log skips malformed lines silently, and the refinement EVENTS live inside the same mutable JSON the refiner edits — the ledger can rewrite its own past. ELAI: ledger = append-only journal fold (S-1 shape).
ANTI-A5. Skill validation is warning-level ("Most issues produce warnings but still load the skill", docs/skills.md); no admission gate, trust-by-review. ELAI keeps E-43/F-plan malicious-skill gating in front of ANY skill-shaped steal.
ANTI-A6. Auto-refine admission is an LLM judgment (reviewAutoRefine) with no deterministic floor. In ELAI the LLM may propose; a deterministic check adjudicates (TD-3).
ANTI-A7. dill restore of session state = arbitrary-code deserialization; the corpus already rejected this class (SGLang CVE). See S-2 shaping.

================================================================================
PART 5 — WHAT THIS DOES NOT REOPEN

- No doctrine flip on "everything is programmatic". J-55/J-107/J-108 measured the question; the router stands. Prime's result is consistent with ELAI's routing (Opus-class lane -> CodeAct), not evidence against it.
- No model-chosen orchestration (rubric G6 anti-goal). Prime's parent model freely decides decomposition; ELAI stays deliberately Level-2/3.
- No new reasoning organ. S-1 is a memory/state tier, not a planner; it composes with GRC, it does not replace it.

ORDERING HINT (value-per-effort, not a schedule): S-5 (hours-grade, pure win) -> S-1 (the headline; medium) -> S-3 (medium) -> S-6 rider (small) -> S-2 (medium, needs the sandbox decision) -> S-4 (after S-3) -> S-8 (the scoreboard, after S-1) -> S-7 (only if ever wanted).
