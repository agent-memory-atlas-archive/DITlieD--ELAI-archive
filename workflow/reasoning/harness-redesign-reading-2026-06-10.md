harness redesign reading log, 2026-06-10
protocol: read ONE file, reason, append entry, next file. final recommendations file written after all 11 reads + the 7 user ideas.
user ideas under evaluation:
  U1 direct mcp control (not per-session/per-gate like claude code)
  U2 a system that orchestrates sessions + context/handoff between sessions/plans/work
  U3 undo feature: every llm change undoable, per-session file backups, every session saved, revert anything anytime
  U4 elai tunnel + active context reduction + tools/calls token reduction stacking
  U5 litegame mcp + better hooks, no code comments, better handoff listing every file edited, delete bad versions keep last good
  U6 reason after each tool call (read one thing, reason, write, reset context with important stuff only, repeat) - core problem: keeping "always enough" context without monotonic growth
  U7 optional system to make the LLM SEE things properly (vision aid)

=== entry 1: docs/specs/guide/steps/phase-index-MASTER.txt (1057 lines, read fully) ===

what it is: single-file master index of all 38 phase spec files, pre-phase-0 through 4M. the canonical architecture of elai_cc.

relevance map to user ideas:
- U3 (undo): ALREADY PARTIALLY SPECCED in two places. phase 4B git2 native ops explicitly lists "auto-commit deterministic msg, branch plan-{id}, WIP commits, safe undo". phase 3H session handoff has DAG-based snapshots (immutable, branch/merge, abandoned branches tagged not deleted) + stub recovery from SSB by seq. and per memory, H-49 (track 8) already designs the full time-travel: CAS object store WRAPPING the dark edit_checkpoints, pre-image already captured at file_ops.rs:272/300 then discarded. so U3 is an ACTIVATION problem (wire-dark capability), not an invention problem. key evidence: SSB is append-only (3B), every file_write is an SSB row, so the event log for replay/revert already exists structurally.
- U2 (session orchestration/handoff): phase 3H is exactly this (CMV-derived session handoff, 85% token trigger, branch tips, commit_hashes persist across handoff) + H-51 (cross-session sleep-time handoff, topic-digest from journal, tiered load 20-27K -> 1-3K). again activation/extension not invention.
- U6 (reason-per-tool-call + context reset): the architecture already has the substrate: 3E fixed-slot context budget (8 slots, floors, 3-level adaptive reduction), A-RAG ContextTracker with "(previously read)" placeholders (1E.5), 3H 3-pass trimming with stubs recoverable from SSB. the "always enough context" problem the user names is exactly what slots + stub-recovery solve: keep summaries inline, keep full content recoverable by seq. GRC (2A-2D) is the reason-step machinery, reasoning_trace is an SSB action_type.
- U1 (direct mcp control): 3J.10 "MCP bridge port from claw-code (1751 LOC)", 4C defers MCP bridge as separate sub-phase. so elai has an MCP bridge in its lineage; the question is whether the WORKFLOW HARNESS (claude code side) gets direct mcp control. nothing in the master index covers harness-side mcp; that lives in .claude/ tooling.
- U7 (vision): only vision presence is phase 4I media validator (Qwen3.5-VL / InternVL3.5 via VLM structured output, tier-1 deterministic veto, render-capture-evaluate loop). it is aimed at validating produced media, NOT at helping the llm see screenshots/UIs during dev. a "help the llm see" system would be a new lane but can reuse 4I adapters (playwright render, VLM structured output).
- U4 (token reduction stacking): 3E (budget allocator), 3K (rag hardening, 55-87% token savings target), 1E A-RAG snippet disclosure, H-50 (regime-aware ctx mgmt). per memory, efficiency deepdive found absent levers: batch api 50%, tool search 85%, DIFF mode. tunnel = the remote sidecar ssh tunnel; stacking reductions is plausible since they hit different budget components (input prompt vs tool schemas vs tool results), but multiplicative claims need measurement. UNGROUNDED until measured.

key takeaway for the final doc: elai_cc already contains the skeleton for U2, U3, U6 (SSB append-only event log + snapshots + slots + stubs). user ideas should be framed as wiring/activation of existing dark capability + a thin new lane for U7. avoids reinventing.

=== entry 2: docs/thefinalglance/B-plan/B-PLAN-SUMMARY.md (1112 lines, read fully) ===

what it is: index of B-1..B-42 (MVP wiring, dashboard V2, harness hardening, 4F-4M, knowledge extensions, REFLECT/framing/audits/cosmos). status snapshot 2026-05-17.

relevance map:
- U4 (elai tunnel + reductions): "elai tunnel" = B-28 FOCUS TUNNEL, still PLANNED. 3 generation modes (FIM ~350 tok/step saved, DIFF 30-50% smaller, STRUCTURED fallback) + streaming validation (kill bad gens at token 50 not 500, ~90% gen-time saved on bad gens) + pipeline parallelism. file's own estimate: ~6800 tokens saved on 30-step task = 27% of 25K; COMBINED with ShortCoder = 45-65% effective budget increase. so yes, the reductions COMPOSE because they hit different components: B-28 hits output/generation, 3E slots hit input context, tool-schema reduction hits the fixed prompt overhead. answer to user's "much more reduction?": grounded YES directionally (different budget components multiply), exact number ungrounded until measured per efficiency-deepdive memory (batch api 50%, tool search 85%, DIFF mode all absent levers).
- U6 (reason-per-call, fresh context): B-39 REFLECT (SHIPPED) already implements the core mechanism: per-stage KV reset = separate /v1/messages calls (Agentless pattern arXiv:2407.01489), TRIAGE > REASONER best-of-4 > AUDITOR refine cap=2 > SYNTHESIZE, budget guard at 85%. this is the user's "read, reason, write, reset context" loop in production form for chat turns. extending it per-TOOL-CALL is the new ask; substrate exists (reasoning_trace SSB, ChatInferenceClient trait).
- U3 (undo): B-20 (PLANNED) has WIP commits: 5+ edits > per-WIP commit, squash on AUDITING>MEMORIZING. B-9 (DONE) SessionDag checkpoints per GRC step / tool call / phase transition. so undo-grain exists at commit level (planned) and snapshot level (shipped) but NOT per-file-backup level; that is H-49's CAS store.
- U2 (handoff): CRITICAL FINDING: B-19 (PLANNED, not shipped) notes PreCompact hook is currently a NO-OP STUB at task_runner.rs:3898 and Stop hook is telemetry-only. so elai runtime handoff-on-compaction is wire-dark today. B-9 wired create_handoff at 85% threshold (DONE) but the hook path is dead.
- U1 (mcp): B-25 ElaiNeuron MCP server (4 capabilities task_submit/task_status/pipeline_result/health_check) is DEFERRED-conditional on Empire. B-36 shipped get_mcp_status. an "elai as MCP server" lane exists on paper but parked. direct mcp control of a RUNNING elai = resurrect the ElaiNeuron shape without the Empire dependency.
- U5 (hooks): better hooks = ship B-19 (hook unification, Stop enforcement, PreCompact real, DAIRA 2>15 markers, latency budgets). already specced.
- U7 (vision): B-24 SHIPPED the RCE loop: execute > render (5 adapters: playwright web, blender 3D, ffmpeg anim, direct screenshot read, comfyui) > VLM evaluate (structured JSON, advisory, tier-1 veto) > feedback > iterate. so a "help the llm SEE" system can reuse: MediaRenderAdapter + VlmValidationOutput constrained decoding + Mode A Qwen3.5-VL via ollama. what's missing for the user's pain: this runs only for MEDIA pipeline outputs, not as a general "look at this screenshot/UI and describe it precisely to the coding model" service.

key takeaway: U4 has a named planned home (B-28), U6 has a shipped mechanism to generalize (B-39), U5/U2 hinge on B-19 (planned, hooks currently dead), U7 has shipped infra (B-24 RCE) lacking only a dev-time entry point, U1 has a parked spec (B-25 ElaiNeuron).

=== entry 3: docs/thefinalglance/C-plan/C-SUMMARY_INDEX.md (815 lines, read fully) ===

what it is: 59 C-sessions closing B-plan gaps + paper-grounded improvements (waves 7-9 benchmarks RUNPOD-dependent). all PLANNED at authoring (per later memory, plan-audit 2026-06-04 shows many since shipped/partial).

relevance map:
- U1 (direct mcp control): C-39 is the closest existing plan: MCPClient at cli/src/mcp_client.rs, Dispatch trait with McpClientDispatch vs InProcessDispatch, --via-mcp flag, CI parity job running every CLI test through MCP. note git status shows crates/cli/src/mcp_client.rs + mcp_glue.rs MODIFIED, so some of this likely landed. C-42 adds authz fuzzing (#[authz(states,actors)], DENY-ALL default) which is the safety story for exposing direct control. so "direct mcp control of elai" = C-39 surface + C-42 authz + the parked B-25 ElaiNeuron capability set (task_submit/status/result/health). NOT per-gate like claude code: elai's surface would expose task/session/fsm primitives, not one chat loop.
- U4: C-35 tool attention gating + lazy mcp schema loading, cites arXiv:2604.21816 95% MCP-tax reduction, 5-10K prompt tokens saved per feature. this is the "tools/calls token reduction" leg of U4, already specced. stacking story now concrete: B-28 (output/generation) + C-35 (tool schema overhead) + 3E/H-50 (context slots) + C-25 CumulativeTokenTracker (attention ceiling recalibration) + prompt caching. all different components, so they compose; each has its own measured/projected number.
- U6: C-25 compiled pipeline (template crystallization after 3 successes; skips LLM entirely on cache hit = ultimate context reduction) + C-46 parsimonious routing (difficulty-gated delegation chains). both reduce per-step context need, complementing the read-reason-write-reset loop.
- U3: C-23 SessionDag branch UI (/session fork|merge|close + load_handoff for /continue) + C-13 WIP squash (>5 WIP commits > 1 squashed on AUDITING>MEMORIZING). undo at session-branch + commit grain specced; per-file CAS grain still only in H-49.
- U5: C-13 is the hook-unification execution session (IpcHookDispatcher production swap, RuffPostToolUseRunner, 15 DAIRA markers). "better hooks" has a concrete vehicle.
- U7: nothing new in C-plan; C-56 codebase-memory benchmark is text-only.

key takeaway: C-plan supplies the EXECUTION vehicles for U1 (C-39+C-42), U4 (C-35), U5 (C-13). when writing recommendations, check shipped-status against the 2026-06-04 plan audit instead of assuming PLANNED.

=== entry 4: docs/thefinalglance/D-plan/D-SUMMARY_INDEX.md (312 lines, read fully) ===

what it is: 30 sessions, two waves of substrate: DGM-class self-improvement scaffolding (D-1..D-9: typed templates, PPM verifier sidecar, tripwires, BAI promotion, crypto archive) + inference-time grounding modes (D-10..D-18: mode routing, checker registry, adversary role, sycophancy probe, conformal refusal) + benchmarks (D-19..D-25) + dashboards (D-26/27). note from memory: H-plan track 6 found D-10/D-11/D-16/D-17 are SHIPPED (the summary's PLANNED labels are stale).

relevance map (thinner than B/C; D is about reasoning quality not harness ergonomics):
- U3 (undo integrity): D-8 crypto archive pattern (blake3 hash chain, append-only, `elai archive verify`, 10 tamper scenarios in D-25) is the right integrity model for an undo/backup store: if every file pre-image goes into a content-addressed chain, "delete bad versions, keep last good" (U5) becomes provable rather than hopeful. reuse pattern, not code.
- U6 (reason-per-call): D-12 operationalization predicate (claims without a measurable procedure BLOCK) + D-10 mode routing are the QUALITY guards for a per-tool-call reasoning loop: if the loop emits a reasoning step after each read, mode routing decides whether that step needs checking, refusal, or nothing. without this the per-call loop just generates more ungrounded prose. important design coupling for the U6 recommendation.
- U5 ("delete bad versions keep the last good one"): D-5 BAI promotion + D-4 tripwires are literally select-best-and-retire machinery, but for reasoning templates. the handoff version is simpler (last-audit-PASS version wins) and should not borrow this complexity.
- U1/U2/U4/U7: no direct contribution.

key takeaway: D-plan constrains HOW the U6 loop should reason (grounded modes, operationalized claims), and D-8 gives the integrity pattern for U3's backup store. otherwise orthogonal.

=== entry 5: docs/thefinalglance/E-plan/E-SUMMARY_INDEX.md (616 lines, read fully) ===

what it is: 55 sessions from the 2026-05-19 grounded sweep (55 reports, ~770 fetches). per the 2026-06-04 plan audit many are now shipped or partial; treat statuses as stale.

relevance map (dense; E-plan touches every user idea):
- U7 (SEE properly): TWO direct vehicles. E-36 document vision RAG: GroundingMethod::VisualArtifactCheck (8th method), ColQwen2.5 (ViDoRe v2 0.597) + PaddleOCR-VL 0.9B, routes pdf/design claims to a vision verifier. E-16 VLM validation delta: AXTree-first + anti-sycophancy + intra-modal disagreement. so elai's planned vision story = verify documents + validate produced UIs. the user's pain ("LLMs dont SEE things properly") is the THIRD lane: perception assist DURING coding (screenshots, UI state, diagrams as model input). E-14 AXTree-first is the key grounded insight: structured accessibility-tree text beats raw pixels for UI understanding; any new "see properly" system should be AXTree/DOM/OCR-first with VLM as fallback, not VLM-first.
- U3 (undo): E-25 ships pre-edit checkpoints + session forking (claude-code-architecture steal); E-44 ships idempotency keys + SAGA compensation log (semantic rollback, closes ACRFence attack) + Continue-As-New rollover. H-49 later wraps these. undo substrate now appears in FOUR plans (4B git2, B-9/3H dag, E-25/E-44, H-49) - the recommendation must UNIFY, not add a fifth.
- U2 (session orchestration): E-25 6-tier compaction ladder above SESSION_HANDOFF ("full session reconstructs from any tier") is the in-session leg; H-51 the cross-session leg. E-29 ssb_resume_from replay = session replay from the event log.
- U6: E-19 L2 confidence gate = the cost-control for reason-per-call: only fire reflection when L1 self-certainty is low (arXiv:2502.18581). this answers part of the user's "always enough context without growing": don't reason on every call, reason on uncertain calls; and reset via per-stage KV (B-39 pattern).
- U1 (mcp): E-24 MCP hardening (blake3 schema pinning, line-jumping sanitization, streamable HTTP, per-tool token budget) is the security prerequisite for exposing direct elai control via MCP.
- U4: E-22 deferred MCP schema loading flag + C-35 gating = tool-token leg. E-32 32K lift + FITM. E-2 prompt-lookup decoding. E-27 AWQ-Marlin 4.4x. stacking continues to be component-wise.
- U5 (hooks): E-30 non-LLM PreToolUse argv-regex destructive-verb allowlist = exactly "better hooks" (deterministic, not model-judgment). E-52 LSP-as-verifier PostToolUse diagnostics probe.

key takeaway: every user idea has at least one E-plan vehicle. the recommendations file should mostly SEQUENCE existing vehicles and only propose genuinely-new work for: (a) harness-side (claude-code .claude/ tooling) undo/handoff improvements, (b) the U7 dev-time perception assist lane, (c) litegame mcp port.

=== entry 6: docs/thefinalglance/F-plan/F-SUMMARY_INDEX.md (287 lines, read fully) ===

what it is: 13 sessions (F-1..F-10 + F-2c/F-11/F-12/F-13) from the VibeSec/SHARD steal-and-build. per memory project_fplan_tier1_run: ALL 13 units SHIPPED 2026-06-03 (commit 551d1aa), with RunPod/live checkboxes deferred. F-plan's differentiator: enforced wiring done-gates (W-1..W-10), no default-off graveyard.

relevance map (mostly security; three touches):
- U3 (undo/backup): F-13 session archive on schema-bump is shipped precedent for "never silently destroy session history": file-copy + blake3 verify before destructive DROP. confirms the codebase norm: session state is sacred, archives are blake3-verified. an undo/CAS store should follow the same shape (and note F-13 is gated by ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE - a W-2 deviation; H-49 must not inherit default-off-forever).
- U5 (better hooks): F-1 VibeGate (shipped) is the model for "better hooks" done right: ONE compound PreToolUse evaluator, fail-closed, deterministic axes, measured FP bound (CodeGate died of FP friction). lesson for litegame mcp + harness hooks: unify hook axes into one evaluator with a bounded-FP requirement rather than stacking N independent hooks.
- U6: F-10 security-monotonicity gate runs POST-write ASYNC feeding the auditor "to avoid stalling the live REFLECT pipeline" - same principle applies to per-tool-call reasoning: heavy checks must be async/off-hot-path or the loop's latency kills it.
- U1/U2/U4/U7: no direct contribution.

key takeaway: F-plan contributes DOCTRINE more than features for this task: W-1 wiring done-gates, W-2 no default-off terminal states, blake3-verified archives, fail-closed compound hooks with FP bounds, heavy checks off the hot path. all recommendations should be written against these gates.

=== entry 7: docs/thefinalglance/G-plan/G-SUMMARY_INDEX.md (226 lines, read fully) ===

what it is: 10 trust-maximization sessions (taint, RRA keystone, blast-radius gates, MCP rug-pull, reviewer cockpit, SAFEDELTA, confidence producer, attestation, redaction, model-drift canary). per memory: all 9 detail files authored; rra.rs/attestation.rs confirmed SHIPPED per I-plan track-20 memory.

relevance map:
- U3 (undo): G-2 RRA is the strongest existing answer to "every session is saved, every llm action can be reverted": the SSB action stream + seed + pins bundled into one portable replayable file with `elai replay --verify`. an undo feature should be specified as a VIEW over this same journal (replay-to-N / invert-from-N), not a parallel store. honest-scope caveat carries over: SSB stores HASHED tool i/o for some steps, so byte-level file restore needs the CAS pre-image store (H-49) as the complement - journal gives the WHAT, CAS gives the BYTES.
- U2: G-2's resume caveat is load-bearing: recover_from_checkpoint is a STUB; resume-from-journal is the real gap. so the "system that orchestrates sessions + handoff" must include journal-resume, already identified as G-2's optional second half.
- U1 (mcp): G-4 rug-pull re-pin gate (blake3 re-pin on tools/list_changed, fail-closed) completes the MCP security stack (E-24 pin + C-42 authz + G-4 re-pin). exposing elai control via MCP becomes defensible only with these three.
- U5 (handoff showing every file edited): G-5 reviewer cockpit + DSR metric is the grown-up version of "handoff that shows every file edited": per-hunk view with proof tokens (test hash, gate verdict, citation) + deterministic DSR recomputable from SSB. the harness-side handoff should adopt the same shape cheaply: file list + verdict per file from the session journal.
- U6: G-7 confidence producer (claim_score from log-prob + self-consistency + execution agreement) is the principled trigger for when the per-call reasoning loop should reflect/abstain - pairs with E-19's L2 confidence gate.
- U4/U7: no direct contribution.

key takeaway: G-plan doctrine binds the recommendations: GD-2 (every number recomputable from journal), GD-4 (no default-ON until FP bounded on >=3 runs), GD-5 (no overclaimed guarantees). undo = journal replay (G-2) + CAS bytes (H-49); handoff file-list = SSB-derived like DSR.

=== entry 8: docs/thefinalglance/H-plan/H-SUMMARY_INDEX.md (1039 lines, read fully in 3 passes) ===

what it is: 54 sessions across 11 tracks (efficiency, canvas, trading-trust, UX+user-memory, grounded-reasoning, DGM verification, REVERSIBILITY, CONTEXT-LIFECYCLE, review-anchoring, external steals). this is THE plan family that already contains most of the user's asks, authored but unexecuted (PLANNED, uncommitted).

direct hits on the user ideas:
- U3 (undo) = H-35 + H-49, VERBATIM. H-35: `elai rewind <seq> --scope conversation|code|both` over the SSB journal (all restore primitives PROD-WIRED, only the affordance missing). H-49: every llm edit reversible+re-appliable, CAS blake3+zstd object store wrapping the dark edit_checkpoints, pre-image ALREADY captured at file_ops.rs:272/300 then discarded, undo-tree HEAD over SessionDag (undo=back, redo=forward, undo-then-edit=fork, timelines never lost), RV-1 divergence guard so user manual edits never clobbered, 3 grains (edit/turn/babystep), lossless paged transcript. the user's "every change backed up per session, revert anything anytime" is H-49's exact spec. NOTHING NEW TO DESIGN for elai-side undo; the gap is (a) EXECUTING H-35/H-49 and (b) a HARNESS-side (claude-code workflow) equivalent, which does not exist in any plan.
- U2 (session orchestration/handoff) = H-50 + H-51, VERBATIM. H-50: regime-aware context policy (local flat-100K auto-compact never fires <32K; 85% gate reads wrong window; recover_stubs re-inflates trims; cloud caching live). H-51: wires the user's own .claude/rules/handoff-protocol.md INTO the elai runtime - sleep-time topic digest authored from the journal, auto-detect + resume offer, tiered load ~20-27K -> ~1-3K. also answers U5's "better handoff that shows every file edited": H-51's digest DECISIONS/FILES sections must match the journal's FileWrite rows (off-agent, not recollection).
- U4 (reductions stack) = track 1 entire. H-1/H-2 caching, H-3 diff/FIM output (5x lever), H-4 tool-search (~77K->8.7K tool defs), H-5 batch 50%, H-6 observation masking, H-34 MCP defer-load for external servers. HD-1 doctrine answers the user's "= much more reduction?" question directly: every number MEASURED per config, NEVER multiplied. so the honest answer: yes they stack because they hit different components (input cache / output tokens / tool schemas / window), but the combined figure must be measured, not multiplied.
- U1 (direct mcp control): H-34 confirms the live elai MCP lane is STDIO McpServerManager; the HTTP bridge (mcp_client.rs) is wire-dark. exposing elai AS a controllable MCP server remains net-new (only B-25 ElaiNeuron sketch + C-39 --via-mcp).
- U6: H-42 serial-depth-scaled reasoning budget formalizes "how much reasoning per step"; H-50 CL-rules formalize the "reset context with important stuff only" loop (unload/keep-stubs-unloaded/selective-reload). the user's screenshot idea maps to H-50 + per-call confidence gating (E-19/G-7).
- U7: no vision-assist lane in H-plan either (H-13 canvas_screenshot -> media_validator Tier2 is the closest: model-driven screenshot capture wired to VLM). confirms U7 is the one genuinely unplanned lane.

key takeaway: U2/U3/U4 are AUTHORED-BUT-UNEXECUTED H-plan tracks. recommendation = sequencing/execution priority, not new design. genuinely new work remaining: harness-side undo+handoff (claude-code .claude/ tooling), U7 perception assist, litegame mcp port, elai-as-MCP-server.

=== entry 9: docs/thefinalglance/I-plan/I-SUMMARY_INDEX.md (1049 lines, read fully in 2 passes) ===

what it is: 54 sessions, tracks 12-20 (fault-isolation, evidence-ranked search, cache economics, anti-gaming gates, routing/budgets, open-weight flywheel, deterministic-over-prompt, design critique, competitor steals). authored 2026-06-10 (TODAY) after the wiring bedrock.

relevance map:
- U6 (read-reason-write-reset loop): track 13/14 IS the formalization of the user's screenshot idea. I-5 pristine context injector: rebuild context from the PARENT CHAIN per branch, hard-exclude failed siblings + raw tracebacks, keep only a <=30-token contrastive constraint - this is "reset context with important stuff only" made mechanical, with the vividness tax MEASURED. I-11 typed BeliefState (tried/failed-because/current-state/constraints) replacing raw history on every handoff = the answer to "how to keep giving the model always-enough context without growing": a TYPED, bounded, versioned state object + lazy detail re-expansion (recover_stubs/TraceQuery) instead of an accumulating transcript. I-14 deterministic compression (LSP slicing + AST folding, reversible receipts) is the per-payload leg. I-6 anchor pane (task spec re-asserted LAST, after cache breakpoint) solves lost-in-the-middle. TOGETHER I-5+I-6+I-11+I-14 = the user's U6 system, already designed.
- U3 (undo): I-52 checkpoint/rewind AGENT TOOL (save/rewind/diff over shipped edit_checkpoints, divergence-refusing) - the model itself can checkpoint before risky edits. coordination clause with H-49 already written. I-45 objective-hash receipts protect against goal-mutation-after-pass.
- U1 (direct mcp control): I-40 trust-layer wrapper is the inverse direction (elai wraps OTHER agents via proxy/MCP) - useful shape: the same proxy seam can expose elai's own control surface. still no dedicated "elai-as-MCP-server for session/task control" session anywhere; remains the genuine net-new for U1.
- U5 (better hooks): I-46 bash interceptor (refuse-with-redirect to registered tools), I-35 temporal tool-sequence rules ("tests ran since last src write"), I-19 tdd preflight, I-47 mechanical rule injection. "no comments on code" type rules belong in I-47's frozen per-role rule block / I-48 prompt artifacts, NOT in hook logic.
- U4: I-10 prefix invariance + ttl tiering + cache-hit HARD GUARD (unexplained miss >10% halts + dumps blake3 prefix diff) + I-15 semantic cache + I-7 cascading escalation. the measurement doctrine (ID-5: no paper constant is a threshold) again forbids multiplying claimed savings.
- U7: nothing. confirmed across ALL NINE plan files: no dev-time perception-assist lane exists.

key takeaway: I-plan track 13/14 already specifies the U6 loop end-to-end. U7 and harness-side (claude-code) U2/U3/U5 ports and elai-as-MCP-server are the only genuinely unplanned items.

=== entry 10: C:\Users\user\Downloads\vlm-vision-research-summary.md (74 lines, read fully) ===

what it is: condensed 2025-2026 VLM research (model ranking, limits, pipeline rules, domain playbooks, specialist CV modules). this is the grounding corpus for U7.

the load-bearing findings for the "help the LLM SEE" system:
1. MOST FAILURES ARE INPUT-PIPELINE FAILURES, NOT MODEL FAILURES. APIs silently downscale (4K -> 800x800); fine text/buttons vanish before the encoder. fixing input recovers 30-60% accuracy WITHOUT changing model. so U7's core is a DETERMINISTIC PREPROCESSOR, not a smarter model: resize to per-model sweet spot (Claude 1568px / GPT-4o 2048 / Qwen3-VL 1280), PNG->JPEG q85 (~60% token savings), overlapping tiles + global thumbnail for dense UIs.
2. HYBRID BEATS PURE VISUAL: pass accessibility tree / DOM alongside the screenshot. confirms the E-14 AXTree-first doctrine independently.
3. PERCEPTION-SCAFFOLD: call 1 describes every element to structured text, call 2 reasons over description+image (near-zero -> 40% on game tasks, lmgame-Bench ICLR 2026). + anti-hallucination "describe whats literally visible first". + instruction BEFORE image. + normalized 0-1000 coords (15% -> <2% coordinate error). + structured JSON at temp 0.
4. KNOWN HARD LIMITS (route around, dont prompt around): visual regression diffs FAIL on VLMs - use pixel-diff/Grounded-SAM-2 diff (matches B-24's tier-1 deterministic veto); counting >10 small elements degrades; nested micro-text brittle even tiled.
5. ROUTING/COST: probe small VLM (7B) first, escalate on low confidence (-78% cost within 2pts); image prompt caching ~90% on re-queried screenshots; fixed visual domain -> fine-tune 7-32B.
6. SPECIALIST COMPOSITION (MoAI pattern): verbalize specialist CV outputs (Depth Anything V2, SAM 2.1, Grounding DINO 1.5, dots.ocr 1.7B which beats GPT-4o on OCR) into the VLM prompt; beats any VLM alone.

mapping to elai substrate: B-24 MediaRenderAdapter (5 adapters incl direct screenshot read) + Tier1 deterministic checks + Tier2 VLM constrained JSON = the skeleton already shipped. E-36 ColQwen/PaddleOCR = the document leg. H-13 canvas_screenshot -> media_validator Tier2 = the capture wire. what does NOT exist anywhere: the PREPROCESSING pipeline (resize/JPEG/tile per target model), the perception-scaffold two-call chain, the hybrid AXTree+screenshot composer for DEV-TIME inputs (user pastes a screenshot / model needs to see the app), and the small-first VLM router. that is the new U7 session: a "perception gateway" every image passes through before any model sees it.

=== entry 11: C:\Users\user\Desktop\Screenshot_42.png (image, read) ===

what it shows: a claude-code session running the exact U6 loop the user wants formalized: Read one repo summary (impeccable.md) -> "Thought for 11s" -> Edit ext-repo-eval-2026-06-10.md (added 7 lines, a self-contained REPO 5 verdict block) -> Read the next summary (Agent-Reach.md) -> "Thought for 6s" -> Edit again (added 8 lines, REPO 6 block). each cycle: ingest ONE artifact, reason briefly, externalize the conclusion into an append-only verdict file, then move on. the verdict file (not the conversation) is the accumulating state.

why this matters for U6's design: the pattern works because (a) each reasoning step has exactly one input artifact (no cross-contamination), (b) the OUTPUT is a compressed, structured, durable verdict (5-8 lines per item, with anchors), (c) the conversation context can be discarded after each cycle since the file holds everything needed. the user's stated problem - "how to keep giving the model always-enough context without growing it" - is answered by the file-as-state pattern PLUS I-11's BeliefState: the next cycle needs only {the task spec, the verdict-file-so-far (or its index), the next artifact}. growth is bounded by the verdict file's compression discipline, not the transcript. this session itself ran the same protocol as a live proof.

=== all 11 reads complete; final recommendations file: harness-redesign-recommendations-2026-06-10.md ===
