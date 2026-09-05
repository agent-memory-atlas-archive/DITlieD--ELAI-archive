harness redesign recommendations, 2026-06-10
companion to: workflow/reasoning/harness-redesign-reading-2026-06-10.md (the per-file reading log; every claim below is grounded there, entry numbers cited as [eN])
inputs: phase-index-MASTER + B/C/D/E/F/G/H/I summary indexes + vlm-vision-research-summary.md + Screenshot_42.png + the user's 7 ideas (U1-U7)

==================================================================
SECTION A: WHAT NEEDS NO NEW DESIGN, ONLY EXECUTION
==================================================================
four of the seven ideas are already fully designed in authored-but-unexecuted plan files. adding new plans for them would violate GD-1 (compose-not-restate). recommendation = run these sessions, in this order.

A-1. UNDO (U3) - execute H-35 then H-49, with I-52 as the agent-facing tool.
  files: docs/thefinalglance/H-plan/H-35_* (rewind over SSB journal), H-49_* (CAS undo/redo/branching time-travel), I-plan I-52 (checkpoint/rewind agent tool).
  why these and nothing new: the pre-image of every llm file edit is ALREADY captured at .elai_cc/crates/orchestrator/src/file_ops.rs:272/300 and then thrown away [e8]. edit_checkpoints.rs is shipped + test-proven but gated OFF [e8, e9]. SSB is append-only so the action history already exists [e1]. H-49 specifies exactly the user's ask: every llm change undoable, blake3+zstd content-addressed per-session store, undo-tree (undo-then-edit forks, timelines never lost), RV-1 guard so the USER's manual edits are never clobbered, 3 grains (edit/turn/babystep), lossless paged transcript. G-2 RRA gives replay --verify on top [e7].
  the one design note to carry in: journal gives the WHAT (replay), CAS gives the BYTES (restore). both are needed; H-49 already says so.

A-2. SESSION ORCHESTRATION + CROSS-SESSION CONTEXT/HANDOFF (U2) - execute H-50 then H-51; pick up G-2's resume-from-journal second half.
  files: H-plan H-50 (regime-aware in-session ctx mgmt), H-51 (sleep-time topic digest + auto-detect resume + tiered load 20-27K -> 1-3K), G-plan G-2 (recover_from_checkpoint is a STUB; resume-from-journal is the real gap [e7]).
  why: H-51 literally wires the existing .claude/rules/handoff-protocol.md INTO the elai runtime - digest authored from the journal (off-agent, not model recollection), fresh session auto-offers resume, details lazily re-expandable via recover_stubs. this is the user's "system that orchestrates sessions and context/handoff between sessions/plans/work".
  also fixes live bugs found in reading: local auto-compact never fires under 32K (flat 100K constant, conversation.rs:22), the 85% gate reads the wrong window, recover_stubs re-inflates trims to ~0 [e8].

A-3. TOKEN REDUCTION STACK (U4: "elai tunnel + active context reduction + tools/calls reduction = much more reduction?") - execute H-plan track 1 (H-1..H-9) + H-34 + B-28 + C-35/I-10.
  direct answer to the question: YES they stack, because each lever hits a DIFFERENT budget component, and NO you may not multiply the numbers - HD-1/ID-5 doctrine: every efficiency number is measured per config, never a product of literature figures [e8, e9].
    - input prefix: H-1/H-2 cache markers + byte-stable breakpoints + 1h TTL; I-10 hit-rate HARD GUARD (miss>10% halts and dumps the leaking byte range).
    - output tokens (the 5x-billed lever): H-3 = activate B-28 focus tunnel DIFF/FIM emission (currently always falls to whole-file STRUCTURED because editor_fs_override=None at task_runner.rs:7871) + streaming early-abort (validator hardcoded None at runtime_builder.rs:306) [e8]. B-28's own estimate: with ShortCoder, 45-65% effective budget increase [e2].
    - tool schemas: H-4 tool-search (~77K -> 8.7K tool defs) for the built-in catalog; H-34 defer-loading for external MCP servers (the live STDIO lane eagerly carries every inputSchema today); C-35 gating (cites 95% MCP-tax reduction, to be re-measured).
    - non-interactive lanes: H-5 batch api, flat 50%.
    - window content: H-6 observation masking + A-2's H-50/H-51.
  sequencing note: H-9 (cost cockpit - the spend ledger renders all-zeros today because .with_ssb_writer is never chained [e8]) must ship FIRST or none of the other deltas are falsifiable.

A-4. READ-REASON-WRITE-RESET LOOP (U6, the Screenshot_42 pattern) - execute I-5 + I-6 + I-11 + I-14, gated by E-19/G-7 confidence.
  the screenshot [e11] shows the working pattern: one artifact in -> short think -> append a 5-8 line structured verdict to a durable file -> next artifact. the conversation is disposable because the FILE is the state.
  the user's hard question - "how to keep the model always-enough context without monotonic growth" - is answered in I-plan track 13/14 [e9]:
    - I-11 BeliefState: a TYPED, versioned, bounded state object {tried, failed-because, current-state, constraints} replaces raw history at every handoff. growth bounded by schema, not transcript length.
    - I-5 pristine context injector: rebuild each step's context from the parent chain; failed attempts survive only as <=30-token contrastive constraints; the vividness tax is A/B measured.
    - I-6 anchor pane: task spec re-asserted LAST (after the cache breakpoint), so the loop never loses the goal (lost-in-the-middle defense).
    - I-14 reversible deterministic compression (LSP slicing + AST folding) with ContextTransformReceipts so replay reconstructs what the model saw.
    - recoverability: anything dropped stays pullable via recover_stubs / TraceQuery (H-6, I-13) - "always enough" = always RECOVERABLE, not always RESIDENT.
  cost control: do not reason after EVERY call - E-19 L2 confidence gate + G-7 claim_score fire reflection only when self-certainty is low [e5, e7]; heavy checks run async off the hot path (F-10 lesson [e6]).

A-5. BETTER ELAI HOOKS (the elai half of U5) - execute C-13/B-19 (hook unification; Stop hook is telemetry-only and PreCompact is a NO-OP stub at task_runner.rs:3898 today [e2, e3]) + E-30 (deterministic argv destructive-verb deny) + I-35 (temporal tool-sequence rules) + I-46 (bash refuse-with-redirect).
  doctrine from F-1 [e6]: unify axes into ONE compound fail-closed evaluator with a measured FP bound, not N stacked hooks (CodeGate died of FP friction).
  "no comments on code": this is a PROMPT/RULE concern, not a hook - put it in I-47's frozen per-role rule block / I-48 prompt file artifacts (mechanically injected, byte-stable), with a cheap post-write grep hook as the deterministic backstop. for the harness side, add one line to codeRules.md and let the existing PostToolUse check enforce.

==================================================================
SECTION B: GENUINELY NEW - PROPOSE TO ADD (no existing plan covers these)
==================================================================

N-1. PERCEPTION GATEWAY (U7: "the LLM doesnt SEE properly") - the only user idea with NO lane in any of B..I [e5, e8, e9].
  what: an optional, deterministic image-ingestion pipeline that every screenshot/diagram/mockup passes through before ANY model sees it, plus a two-call perception-scaffold service. grounded entirely by vlm-vision-research-summary.md [e10]:
    leg 1 PREPROCESS (pure code, no model): resize to the TARGET model's sweet spot (Claude 1568px / GPT 2048 / Qwen-VL 1280 longest side), PNG -> RGB JPEG q85 (~60% token savings), overlapping tiles + global thumbnail for dense UIs. this alone recovers 30-60% accuracy - most production VLM failures are silent-downscale input failures, not model failures.
    leg 2 HYBRID COMPOSER: when the image is a UI, attach the accessibility tree / DOM text alongside (hybrid beats pure visual; independently confirms E-14's AXTree-first doctrine). for documents, route to the E-36 ColQwen/PaddleOCR leg; for OCR-heavy crops, dots.ocr-class specialist.
    leg 3 PERCEPTION SCAFFOLD: call 1 = "describe every literally-visible element" to structured JSON (temp 0, normalized 0-1000 coords, instruction BEFORE image); call 2 = the actual coding model reasons over description+image. measured near-zero -> 40% on hard visual tasks.
    leg 4 ROUTER: probe small VLM first, escalate on low confidence (-78% cost within 2pts); image prompt caching on re-queried screenshots (~90%).
    hard-limit routing: visual diffs go to pixel-diff/Grounded-SAM-2, never a VLM (B-24 tier-1 veto already encodes this); counting >10 elements and micro-text get tiling or refusal.
  where: .elai_cc side as a new plan session (suggest J-1 or H-55+, nature BUILD, composes B-24 MediaRenderAdapter + media_validator Tier2 + E-36 + E-16 + H-13 canvas_screenshot; integration point = a PerceptionGateway in crates/orchestrator/src/perception/ with the heavy CV/VLM legs on the python sidecar). harness side: a thin .claude/skills/see skill that runs the same preprocess locally (python, stdlib+pillow) before attaching any image to a claude-code message, so my own image reads benefit too.
  observable_effect (W-1 shape): a 4K screenshot fixture through the gateway lands a perception_scaffold SSB row + the model answers a fine-text question the raw-image control arm fails; coordinate error <2% on a seeded click-target fixture.

N-2. ELAI-AS-MCP-SERVER: DIRECT CONTROL PLANE (U1: "direct mcp control, not per session or gate like claude code").
  what: expose elai's task/session/fsm primitives as a long-lived MCP server so an external client (claude code, dashboard, scripts) can drive a RUNNING elai directly: task_submit, task_status, fsm_get_state, session_fork/rewind (rides A-1), plan_status (I-39), rra_show, intervention_respond. continuous control surface instead of one chat loop per session.
  why new: only B-25 ElaiNeuron sketched this (4 capabilities) and it is DEFERRED on the Empire dependency [e2]; C-39 built the opposite direction (CLI -> MCP client, partially landed per git status) [e3]; I-40 wraps OTHER agents [e9]. nobody authored "elai exposes its own control plane".
  preconditions (all exist): C-42 authz fuzzing (#[authz(states,actors)], DENY-ALL default), E-24 blake3 schema pinning, G-4 rug-pull re-pin, E-30/G-3 destructive gates behind every exposed verb [e3, e5, e7]. without these this feature is a remote-code-execution hole; with them it is defensible.
  where: new plan session; crates/cli/src/mcp_server.rs (sibling of mcp_client.rs) or a thin crates/orchestrator/src/control_plane/; reuse the B-25 capability list minus Empire transport. every exposed verb emits the existing SSB rows so the journal stays the single source of truth (GD-2).

N-3. HARNESS-SIDE UNDO + PER-SESSION FILE BACKUPS (the claude-code workflow half of U3).
  what: H-49 covers ELAI. nothing covers THIS harness (the .claude/ tooling that runs me). add: a PreToolUse hook on Edit/Write/NotebookEdit that copies the pre-image of the target file into workflow/sessions/{id}/backups/{blake3-prefix}/{relpath}@{seq} before the write lands (content-addressed, deduped, zstd above a size threshold), plus .claude/tools/undo.py (list/diff/restore by file or by seq, divergence-refusing like RV-1: if the on-disk file no longer matches the post-image, refuse and show the diff instead of clobbering).
  why: the user's ask is "every change the llm makes to a file, that file is then made backup of in the session... revert at any moment" - for BOTH systems. the harness already has the hook seam (C:/Users/user/.claude/rag/hooks, per-session workflow/sessions/{id}/), so this is ~1 hook + 1 stdlib tool. git is not a substitute: it only sees committed state and this root gitignores almost everything.
  delete-bad-keep-last-good (U5): the restore tool marks the last version that preceded an audit PASS (read from the session state the PostToolUse hook already records) as "last good"; `undo.py prune` keeps last-good + newest and drops the rest. simple last-PASS-wins, deliberately NOT the D-5 bandit machinery [e4].

N-4. HARNESS HANDOFF UPGRADE: FILE LEDGER SECTION (the rest of U5).
  what: extend .claude/rules/handoff-protocol.md's entry template with a machine-derived FILES TOUCHED ledger: every file edited this session (from the PostToolUse change-recording the hooks already do), one line each: path, edit count, last-good backup seq (from N-3), verdict if audited. generated by a small .claude/tools/handoff_ledger.py the main LLM runs when authoring the handoff - derived from logs, not from recollection (the G-5 DSR principle: numbers from the journal, not the narrator [e7]).
  why: the user explicitly asked for "better handoff that shows every file edited". H-51 does this for elai (DECISIONS/FILES must match journal FileWrite rows); the harness should mirror it cheaply.

N-5. LITEGAME MCP + HOOK PORT (U5 first clause).
  what: port the matured pieces of this root's harness to D:\.liteGAME_workflow: the local-rag MCP (FSM/think/memory - the resolver was already unified 2026-06-05 via paths.py, so multi-root is safe per memory), the N-3 backup hook + undo tool, the N-4 handoff ledger, and the F-1-style compound PreToolUse shape instead of stacked hooks. propagation manifest precedent already exists: docs/strategy/WIRING-FIX-PROPAGATION.md lists apps/liteGAME/REGRET as pending targets (memory: project_wiring_gate_failclosed_shipped).
  caveat: 7 other liteGAME compat files were never verified after the F:-move (memory: project_litegame_orchestrate_compat) - verify those first; this is a half-day port, not a plan.

N-6. U6 AS A HARNESS SKILL: incremental-eval protocol.
  what: formalize the Screenshot_42 pattern as .claude/skills/incremental-eval (or a rule block): when given N artifacts to evaluate/read, MUST process one at a time, appending a bounded structured verdict block (anchor + 3-8 lines) to a single topic-keyed file in workflow/reasoning/, never batch-reading. include the BeliefState trick: each cycle's required context = {task spec, verdict-file index, next artifact} only.
  why: the user re-states this MAJOR RULE every time; encoding it as a skill makes it durable and self-triggering. zero code, one SKILL.md. (this very session is the validation run.)

==================================================================
SECTION C: SEQUENCING (recommended order, cheapest-falsifiable first)
==================================================================
  1. N-6 (one SKILL.md, today) + N-4 (one python tool + rule edit).
  2. N-3 harness backup hook + undo tool (small, immediately protective).
  3. H-9 cost cockpit, then H-1/H-2/H-3/H-4 (A-3) - the instrument before the levers.
  4. H-35 -> H-49 -> I-52 (A-1 undo, elai side).
  5. H-50 -> H-51 (A-2 context lifecycle + handoff), then G-2 journal-resume leg.
  6. I-5/I-6/I-11/I-14 (A-4 loop substrate; I-5 and I-6 are S/M efforts and dont need the I-4 search ladder).
  7. N-1 perception gateway (new authoring; leg 1+3 first - pure preprocess + scaffold are model-free/cheap; legs 2/4 follow).
  8. N-2 elai MCP control plane (after C-42/G-4 are confirmed live - security prerequisites).
  9. N-5 litegame port (any idle half-day, after N-3/N-4 stabilize here).
  10. A-5 hook unification (C-13 is already the planned vehicle; fold I-35/I-46 in when executed).

==================================================================
SECTION D: HONESTY NOTES
==================================================================
- plan statuses in B..E summaries are STALE (many since shipped/partial); before executing any A-item, check workflow/sessions/s-plan-audit-2026-06-04/FINISH/ + `git log` rather than trusting the index labels [e3, e4].
- the combined token-reduction number is UNGROUNDED until measured per HD-1; per-lever numbers above are the plans' own cited measurements or projections, flagged as such in their files.
- U7's 30-60%/40%/-78% figures come from the user's research summary's cited sources, not from elai measurements; N-1's done-gate must produce ELAI-measured deltas (ID-5).
- everything in section B follows the house gates: W-1 wiring done-gates, W-2 no default-off graveyard, GD-2 journal-recomputable numbers, GD-4 FP-bounded before default-ON.

end of recommendations.
