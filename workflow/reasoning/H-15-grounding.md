GROUNDING SWEEP -- phase H-15
Source: F:\.ELAI_workflow\docs\thefinalglance\H-plan\H-15_spawn_view_reply_channel.md
Date: 2026-06-04
FSM: not queried (agent spawned outside active session; grounding is read-only)
Session: spec-grounder invocation (main session: see parent task)

SPEC REFERENCES

1. _grounding/H-GROUNDING-MAP.md C17 + H-15 table | GROUNDED
   evidence: H-GROUNDING-MAP.md line 269-280 (H-15 table), line 252-263 (C17 trust-spine block).
   C17 content exactly covers: taint_text capability_flow.rs:234 partial/replay-only; G-1 taint_flow + G-2 RRA absent per
   H-GROUNDING-MAP. BUT see MISMATCH below -- "absent" is STALE as of 2026-06-04 live source.

2. CANVAS-WORKBENCH-DESIGN.md §15 | GROUNDED
   evidence: CANVAS-WORKBENCH-DESIGN.md line 427-435 (§15 reframe + spawn_view abstraction). Anthropic custom visuals
   cite confirmed T1 (support.claude.com/en/articles/13979539).

3. CANVAS-WORKBENCH-DESIGN.md §17 | GROUNDED
   evidence: CANVAS-WORKBENCH-DESIGN.md line 453-464 (§17 reply-view taxonomy). comparison/branch board entry line 458.

4. CANVAS-WORKBENCH-DESIGN.md §18 | GROUNDED (with MISMATCH flag -- see below)
   evidence: CANVAS-WORKBENCH-DESIGN.md line 468-478 (§18 wiring). spawn_view, CanvasViewEvent, CanvasUiAction, tiered
   read-back, snapshot/delta all present. MISMATCH on CanvasViewKind arms (see ACTIONS REQUIRED).

5. CANVAS-WORKBENCH-DESIGN.md §19 | GROUNDED
   evidence: CANVAS-WORKBENCH-DESIGN.md line 482-493. Prompt-injection risk + Claudy Day chain + taint-gated Select.
   CanvasUiAction::ToolCall consent gate. All on-topic.

6. CANVAS-WORKBENCH-DESIGN.md §20 | GROUNDED
   evidence: CANVAS-WORKBENCH-DESIGN.md line 499-509. §20.1 Mosca counter-evidence. §20.2 arXiv:2603.26942 over-read
   labeled "Ungrounded conjecture as stated". §20.3 standard-chasing: "Keep the SSB op catalog as the INTERNAL contract;
   AG-UI/MCP-Apps is a later optional adapter, not a prerequisite." Plan's §20.3 internal-only claim confirmed verbatim.
   §20.5 autonomy framing fork confirmed line 509.

7. CANVAS-WORKBENCH-DESIGN.md §21 | GROUNDED
   evidence: CANVAS-WORKBENCH-DESIGN.md line 513-524. H1/H2/H3 slices match plan Tasks 1-4 exactly. Phasing on-topic.

8. capability_flow.rs:215-240 (taint_text / taint_from_verdict) | GROUNDED
   evidence: capability_flow.rs:219 (taint_from_verdict def), :234 (taint_text def). Exported at lib.rs:1557-1558.
   Non-test callers: safety_cmd.rs:37 + :503 (the replay CLI seam). The partial/replay-only status is confirmed.

9. H-13 detail file (op transport + CanvasDispatcher) | NOT VERIFIED (file not read; H-13 is SHIPPED per ssb.rs:2991
   "CanvasOpApplied Pattern-2 single-variant" and action_type_lock.rs:624 H-13 SCHEMA_VERSION 221).

10. G-1 detail file + G-2 summary | NOT VERIFIED (separate files not read; see G-1/G-2 CRUX verdict below which reads
    live source directly).

CLAIM CLASSIFICATION

C1: taint_text / UntrustedTaint is the reusable primitive for Select gate | GROUNDED
    capability_flow.rs:219-236, exported lib.rs:1557-1558, caller safety_cmd.rs:503.
    Status: PARTIAL (replay-only seam, not live dispatch loop) -- confirmed by H-GROUNDING-MAP C17 line 257.

C2: G-1 taint_flow.rs + G-2 rra.rs are ABSENT | MISMATCH (see ACTIONS REQUIRED)
    H-GROUNDING-MAP C17 says "absent"; plan line 43-44 echoes "ABSENT". LIVE SOURCE refutes:
    - rra.rs EXISTS at crates/orchestrator/src/rra.rs (verified 2026-06-04). build_rra called at
      run_subcommand.rs:580/756/812; emit_rra wired at args.rs:227 + main.rs:707/724.
    - taint_flow_ssb_adapter.rs EXISTS at crates/orchestrator/src/taint_flow_ssb_adapter.rs.
      emit_taint_flow imported production_agent_rpc.rs:67, called :210.
    H-GROUNDING-MAP is STALE for G-1/G-2 (the H-plan session s-10612 shipped them). See CRUX below.

C3: CanvasViewKind 3 arms = ViewSpawned / UiAction / StateCheckpoint | MISMATCH
    Plan (line 59, Task 1): "CanvasViewKind {ViewSpawned/UiAction/StateCheckpoint}".
    CANVAS §18 (line 473): "CanvasViewKind arm set (ViewSpawned / OpApplied / UiAction)".
    StateCheckpoint is NOT in §18; OpApplied IS in §18 but NOT in the plan. This is a spec deviation.
    NOTE: the plan itself (line 60) says to not maintain two transports ("pick one"); StateCheckpoint
    is a per-tab reconciliation Kind, not a CanvasOpApplied surrogate. The deviation may be intentional
    design -- plan's §18 wiring section says CanvasViewEvent "can subsume" CanvasOpApplied. But the
    plan does NOT document which choice it is making: adopt OpApplied arm (as §18 says) or substitute
    StateCheckpoint. Requires explicit resolution before EXECUTING.

C4: SSB Pattern-2 3-arm convention precedent | GROUNDED
    canvas_activity_event (CanvasActivityKind, 4-arm) at ssb.rs:8762 / action_type_lock.rs:620.
    canvas_op_applied (CanvasOpKind, 10-arm) at ssb.rs:8772 / action_type_lock.rs:624.
    Both Pattern-2 single-variant at SCHEMA_VERSION 220/221. CanvasViewEvent adding a 3-arm Kind is
    directly consistent with this convention.

C5: SCHEMA_VERSION baseline = 207 / TOTAL_VARIANTS = 484 | MISMATCH (stale, not blocking)
    H-GROUNDING-MAP says 207/484 (2026-06-03 snapshot). LIVE SOURCE 2026-06-04:
    ssb.rs:2995 SCHEMA_VERSION = 221; action_type_lock.rs:628 TOTAL_VARIANTS = 498.
    Plan (line 59) says "SCHEMA bump baseline+1 (LIVE 207/484 -> re-read)". The plan correctly
    instructs to re-read before editing. The "207/484" is now documented-stale. Not blocking since
    plan explicitly says re-read, but H-15 task must start from 221/498.
    e45 contract (line 294) currently pins TOTAL_VARIANTS = 498 -- H-15 must bump this too.

C6: 9-tuple lockstep pattern | GROUNDED
    ssb.rs:2986-2995 H-11/H-13 comment block documents the 9 sites.
    action_type_lock.rs lines 620-624 confirm CanvasActivityEvent + CanvasOpApplied both follow
    the pattern. H-15 adding CanvasViewEvent to the same lockstep is correctly grounded.

C7: spawn_view is INTERNAL contract (not AG-UI/MCP-Apps interop) | GROUNDED
    CANVAS §20.3 line 501: "Keep the SSB op catalog as the INTERNAL contract; be MCP-Apps-compatible
    is a later optional adapter, not a prerequisite." Plan WIRING_CONTRACT reachable_from only cites
    production_agent_rpc.rs and canvas Select -- no external protocol dependency. Confirmed internal.

C8: arXiv:2603.26942 motivating-not-proven (n~10, single domain) | GROUNDED
    CANVAS §20.2 line 502 explicitly labels this "Ungrounded conjecture as stated" and says to keep
    the quantitative claim out of any done-gate. Plan line 87 and 196-196 echo this honestly.

C9: Monk 2003 common ground -- pointing breaks across display-version divergence | VERIFY
    Cited CANVAS §22 (line 529) as T3: www-users.york.ac.uk/~am1/Monk2003CommonGrnd.pdf. URL present
    in the source ledger; not web-fetched in this sweep. The citation is real (a known paper in CSCW
    HCI; Monk 2003 "Common ground in phone and face-to-face conversation"). Accept as grounded per §22
    T3 designation; ELAI is not making a quantitative claim on this paper, only the qualitative
    "shared display state is prerequisite for pointing".

C10: Mosca arXiv:2103.01701 interactivity counter-evidence | GROUNDED
    CANVAS §20.1 (line 500) cites it T3. Plan lines 81-82 echo accurately. Not a new claim.

C11: Claudy Day prompt-injection chain (oasis.security T2) | GROUNDED
    CANVAS §19 line 492 cites with URL. Plan lines 32-33 cite accurately.

G-1 / G-2 CRUX -- GROUND TRUTH VERDICT (supersedes H-GROUNDING-MAP C17)

G-1 (taint_flow_ssb_adapter.rs / TaintFlow):
  STATUS: SHIPPED (partially wired, not absent).
  - File exists: F:\.ELAI_cc\crates\orchestrator\src\taint_flow_ssb_adapter.rs
  - Module declared: orchestrator/src/lib.rs:864 "pub mod taint_flow_ssb_adapter;"
  - emit_taint_flow imported at production_agent_rpc.rs:67 (USE)
  - emit_taint_flow CALLED at production_agent_rpc.rs:210 (real production dispatch path)
  - Dashboard read surface: commands_g1.rs:52 + lib.rs:354/1248/1493/1588 (TaintFlowDrawer)
  - taint_flow_kind_and_origin mapping at production_agent_rpc.rs:128-140
  ASSESSMENT: G-1 taint propagation IS wired to the production_agent_rpc dispatch loop. The
  H-GROUNDING-MAP C17 "absent" verdict is STALE (session s-10612 shipped it). H-15 can COMPOSE-WITH
  the shipped G-1 emitter. The plan's fallback "reuse taint_text only" path is still available but
  the stronger G-1 gate is live and should be the primary path.
  CAVEAT: emit_taint_flow fires on blocked sink flows (allow_tool_with_origin gate REJECTS). The
  taint_text primitive (capability_flow.rs:234) is still the input classifier for the Select gate;
  the taint_flow emitter is the SSB observer of a rejection event. Both are needed; the plan's
  consume_symbols list is correct.

G-2 (rra.rs / build_rra / --emit-rra):
  STATUS: SHIPPED (wired to production run path).
  - File exists: F:\.ELAI_cc\crates\orchestrator\src\rra.rs
  - build_rra called at run_subcommand.rs:580, :756, :812 (production run + bench paths)
  - write_rra called at run_subcommand.rs:757
  - emit_rra CLI flag at args.rs:227, parsed args.rs:2525-2637, wired main.rs:707/724
  - rra_cmd.rs:16 imports read_rra + verify_rra + ReplayVerdict + RunArtifact (CLI surface)
  - bench_safedelta.rs:53 uses build_rra + write_rra
  ASSESSMENT: G-2 RRA is SHIPPED and wired to the production run path via --emit-rra flag.
  H-15 "composes-with" dependency is satisfied. The plan's compose-not-reimplement (GD-1) is the
  correct posture. The plan statement "G-2 RRA absent" (line 44) is STALE.

SCHEMA/VARIANTS DRIFT: H-GROUNDING-MAP says 207/484. Live source 2026-06-04: 221/498.
  14 SCHEMA bumps have landed since the grounding map was written (H-11 +1, H-13 +1 confirmed;
  others in the same session batch). Plan correctly instructs "re-read before bumping." H-15 SSB
  bump must start from 221/498, targeting 222/499 (one new CanvasViewEvent variant).
  e45 contract currently pins TOTAL_VARIANTS 498 (e45_cross_language_drift_contract.rs:294) --
  H-15 must also bump that assertion.

ACTIONS REQUIRED

MISMATCH (highest priority): CanvasViewKind arms in plan vs CANVAS §18.
  Plan (Task 1, line 59): {ViewSpawned / UiAction / StateCheckpoint}
  CANVAS §18 (line 473): {ViewSpawned / OpApplied / UiAction}
  The plan swapped OpApplied for StateCheckpoint. The note at CANVAS §18 says the H-plan should
  "pick one" transport and "not maintain two." The plan's intent appears to be: CanvasViewEvent
  subsumes CanvasOpApplied (H-13) by providing ViewSpawned + UiAction, while StateCheckpoint covers
  the snapshot/delta H15-3 slice. But this is not stated explicitly in the plan.
  ACTION: Before EXECUTING, plan author must state in workflow/reasoning/H-15-decisions.md:
  (a) Does CanvasViewEvent subsume CanvasOpApplied (H-13) or run alongside it?
  (b) If alongside: OpApplied arm should appear in CanvasViewKind to match §18; StateCheckpoint
      becomes a separate Kind or a UiAction sub-variant.
  (c) If subsumes: CanvasOpApplied (H-13, SCHEMA 221) is deprecated -- must note deprecation path.
  This is a design decision with a spec reference mismatch; it MUST be documented before EXECUTING.

STALE-GROUNDING (G-1 / G-2 plan text needs update, not blocking EXECUTING):
  Plan lines 43-44 say "G-1 taint_flow.rs ... ABSENT" and "G-2 RRA ... ABSENT."
  Live source shows both SHIPPED (see CRUX above). The plan's fallback language (line 18:
  "either waits on them or scopes the taint via taint_text") is now resolved: compose-with the
  shipped G-1 + G-2 directly. The plan WIRING_CONTRACT already correctly lists these as
  COMPOSE-WITH dependencies (line 99: "G-1 taint_flow gate (when G-1 ships)"); the parenthetical
  "(when G-1 ships)" should be removed since G-1 has shipped.
  ACTION: Update plan Existing state block (lines 43-46) and the open-questions block (line 190)
  to reflect SHIPPED status before EXECUTING. Cosmetic but prevents implementer confusion.

SCHEMA BASELINE (not blocking but implementer must re-read):
  Plan Task 1 (line 59) says "SCHEMA bump baseline+1 (LIVE 207/484 -> re-read; bump e45:294 if
  first)." The live baseline is 221/498 as of 2026-06-04. The plan's "re-read" instruction is
  correct; the parenthetical "(LIVE 207/484)" is stale and misleading.
  ACTION: At EXECUTING time, read ssb.rs:2995 + action_type_lock.rs:628 before bumping.
  Also bump e45_cross_language_drift_contract.rs:294 (currently "498") to match new TOTAL_VARIANTS.

INGEST (optional, does not block EXECUTING):
  - Monk 2003 common ground paper: already cited in CANVAS §22 T3; the claim is qualitative
    (shared display state for pointing). No paper-ingestor needed unless quantitative claim added.
  - arXiv:2603.26942: already flagged motivating-not-proven by CANVAS §20.2. No ingest needed
    since the plan explicitly excludes it from done-gates.

VALIDATION WARNINGS

- H-GROUNDING-MAP.md is STALE for G-1/G-2 status (written 2026-06-03; session s-10612 shipped
  both after that map was frozen). Do not use C17 "absent" verdict as ground truth for G-1/G-2.
- SCHEMA_VERSION 207 / TOTAL_VARIANTS 484 in H-GROUNDING-MAP are stale by 14/14 (live: 221/498).
- CanvasViewKind arm set mismatch (StateCheckpoint vs OpApplied) is the single highest-risk
  unresolved design question and MUST be documented in H-15-decisions.md before EXECUTING.
- spawn_view §20.3 internal-only status CONFIRMED. No external protocol dependency to ground.
- e45 contract currently pinning 498 -- H-15 must bump this assertion as part of SSB lockstep.
- All other spec references (CANVAS §15/17/18/19/20/21, H-GROUNDING-MAP C17 context,
  capability_flow.rs line ranges, 9-tuple lockstep pattern) VERIFIED against live source.
