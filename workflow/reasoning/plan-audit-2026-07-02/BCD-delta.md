B/C/D PLAN DELTA RE-VERIFICATION
baseline: workflow/sessions/s-plan-audit-2026-06-04/FINISH/_MASTER-INDEX.md, "VERIFIED FULL-SYSTEM RE-AUDIT - 2026-06-12" block (session s-107b022a)
this pass: 2026-07-02, git-log delta + direct source grep/Read, NOT a full re-sweep

================================================================================
(a) COMMITS SINCE 2026-06-12 MAPPED TO PLANS
================================================================================
30 commits total, 2026-06-13 .. 2026-07-02 (git log --oneline --since=2026-06-12). Only
commits with a B/C/D touch point are listed; the rest (I-plan waves, J-1 perception gateway,
H-plan memory hardening, ssb.rs module split, sidecar context journal, dashboard-v2 visual
pass, bench q-delta, provider tool-input repair, elai-index-build, clippy cleanup) are
I/J/H/E/G-scoped or general infra and out of B/C/D delta scope.

  df18539  06-13  i-plan checkpoint                                    -- I-scope, no B/C/D touch
  9b698b2  06-15  stage-5 punch across i/c/b plans                     -- C-31, C-45, C-47, B-7 (dark-register); grammar gate/tdd/tool-seq/i-47/rra (C/I misc)
  edd1ed2  06-28  C-45 agent-role dropper (arXiv:2605.13295)           -- C-45 (new production wiring, crates/agent)
  981a412  06-28  C-15 entropy quarantine batch 1 (30 sites)           -- C-15
  675918f  06-28  C-15 entropy quarantine batch 2 (24 sites)           -- C-15
  1f32886  06-28  C-15 entropy quarantine batch 3 (15 sites, dashboard)-- C-15
  3d40940  06-28  C-15 entropy quarantine batch 4 (14 sites, final)    -- C-15, claims 0 leaks
  dab4368  06-28  check_entropy_quarantine.sh Windows-path fix         -- C-15 tooling
  7b6d710  07-01  unify metrics/rules.db path lookups (self_mod_layer1a touched) -- not B/C/D scoped (B-23 self-mod module, already SHIPPED_WIRED; infra bugfix only)
  15bfa3b/29358e6  07-01/07-02  B-3 plan-doc resync (duplicate commits)-- B-3 (docs-only resync, no source change)
  eacd440  07-02  sweep of B-2/B-4/B-5/B-6/B-7 uncommitted work        -- B-2, B-4, B-5, B-6, B-7 (real code, entangled per commit's own disclosure)

UNCOMMITTED AT SESSION START (git status): .elai_cc/crates/orchestrator/{grc.rs,lib.rs,
research_tree.rs,task_runner.rs}, tests/safety_gate_integration.rs, indexes/watch.lock.
Content = B-8 W1/W2/W3 test-production symbol-drift fixes (CSQ_CRITICAL_BLOCK_REASON pub
const, truncate_for_log made pub, double-Finding-emission guard) + a comment-only update to
the B-7 WIRE-DARK markers (still dark, clarifies producer data-flow deferred to a B-7-followup
plan). Not yet committed as of this audit.

NOTABLE: eacd440's own commit message discloses a live infra bug in the glm-orchestrator
controller (orchestrator.py): a "dirt-baseline" capture bug silently absorbed fixes from
FAILED attempts (gate never passed, S10-commit never fired) into the next attempt's baseline,
losing ~10 hours of B-2/B-4/B-5/B-6/B-7 work across multiple attempts until manually swept
into one commit. Work could not be cleanly attributed per-B-N because the bug entangled
several items' edits in one working tree. Flagged for the controller as a process-integrity
finding, not a wiring finding.

================================================================================
(b) PER-PLAN FLIP TABLE (evidence-grounded, file:line anchors)
================================================================================

B-4_knowledge_producer_wiring | 06-12: PARTIAL (3 gaps: get_hyde_query_recent absent;
DkgStats.failed_files absent; DEEP MRR unresolved) | 07-02: PARTIAL, 2 of 3 CLOSED --
  - get_hyde_query_recent: NEW pub fn, real Tauri command, registered in generate_handler!
    (commands.rs:1628, lib.rs:645). WIRED.
  - DkgStats.failed_files: producer AND consumer now both real. DkgReadyPayload gained
    `failed_files: u64` (#[serde(default)]) at orchestrator/src/ssb/payloads.rs:1689;
    populated from `summary.dkg_files_failed` (a real derived count, indexing_service.rs:754)
    at the non-test call site task_runner.rs:5987 (inside the real run_full_index completion
    path, not #[cfg(test)]). Dashboard consumer at commands.rs:2432 reads it defensively.
    WIRED end-to-end. NOTE: commands.rs:2427 comment ("DkgEngine::failed_files() ... never
    plumbed") is now STALE/WRONG -- cosmetic doc drift, not a functional gap; flag for a
    one-line comment fix, no code risk.
  - DEEP MRR monotonicity: UNCHANGED, still 0.5017 < 0.6000 STANDARD 0.6000 per the
    hardcoded fallback at commands.rs:2404-2406. Still open.

B-5_reasoning_producer_wiring | 06-12: flip PARTIAL->WIRED already claimed in master-index
  notable-flips list (grounds unclear from 06-04 fragment) | 07-02: verified CsqBlockCard.svelte
  EXISTS and is MOUNTED (ReasoningDrawer.svelte:10 import, :293 `<CsqBlockCard />`) and
  get_agent_grc_config has a real Svelte consumer (ReasoningDrawer.svelte:14,145,319-335,
  fetched on load, rendered as a collapsible per-role GRC table). Both were still-open 06-04
  punch items; both closed via the 07-02 eacd440 sweep (dated commit content, "B-5 GAP-1"
  anti-rubber-stamp counter card + the GRC config table). Confirms/strengthens the 06-12
  WIRED verdict with direct evidence the 06-04 fragment lacked.

B-7_pipeline_factory_producer_wiring | 06-12: PARTIAL (controller-attention: "2 of 4 emitters
  absent") | 07-02: UNCHANGED. Verified via grep: SsbTreePositionChangedEmitter (task_runner.rs
  :7488 `.with_tree_position_emitter(...)`) and SsbFailureRecordedEmitter (task_runner.rs:7483
  `.with_failure_emitter(...)`) ARE wired (non-test). SsbPackageRefVerifiedEmitter and
  SsbTechStackDecidedEmitter remain ONLY defined in research_tree.rs, zero task_runner.rs call
  sites -- still WIRE-DARK[B-7]. The uncommitted diff (see (a) above) only updated the marker
  comment text (clarifying the producer-side PackageRef/TechStack materialization site does
  not exist in drive_task's decomposition flow), did not close the gap. Registered dark
  (WIRE-DARK[B-7] marker present, stubs.md/allowlist backing per 9b698b2's "dark-register").

C-31_code_completeness_file_plan | 06-12 controller-attention: "set_mistake_recorder zero
  production callers in orchestrator/; 1 of 3 gaps still open" | 07-02: STILL DARK, but now
  REGISTERED not accidental. grep confirms zero non-test callers of set_mistake_recorder
  (tiered_analysis.rs:551,886 -- only the two setter definitions, no call site). 9b698b2 added
  a WIRE-DARK[C-31] marker directly above both definitions (tiered_analysis.rs:550,885) plus a
  wire-dark-allowlist.toml row (name="set_mistake_recorder", type=experiment, owner=maintainer,
  expiry=2027-06-01, promotion_plan_id=C-31-promotion, smoke_test=c31_file_plan_contract.rs).
  Verdict flips accidental-DARK -> REGISTERED-DARK (WARN per production-caller-gate.md), not
  wired.

C-45_cantante_contrastive_credit_attribution | 06-12 controller-attention: "current_credit_
  weights not called from routing/policy.rs (grep 0 hits); CLI path real but credit scores
  don't influence live routing" | 07-02: current_credit_weights (credit_attribution/
  integration.rs:46) STILL has zero non-test callers -- confirmed all 4 call sites
  (integration.rs:113,115,120,121,126,132) are inside the #[cfg(test)] mod. STILL DARK on the
  routing-weight leg, now REGISTERED (WIRE-DARK[C-45] marker at integration.rs:44 + allowlist
  row name="agent.credit_attribution.enabled", promotion_plan_id=C-45, added by 9b698b2).
  SEPARATE NEW WORK (edd1ed2, 2026-06-28): the agent-side "dropper" ablation primitive
  (crates/agent/src/dropper.rs, 599 lines, AgentRoleDropper::from_env() gated on
  ELAI_C45_DROPPED_ROLE) IS wired at a real production entrypoint: crates/agent/src/main.rs:315
  constructs it and passes it into handle_message (main.rs:147-152), non-test. This is a
  DIFFERENT C-45 leg (contrastive ablation for the credit-attribution eval harness) from the
  routing-consumer leg (current_credit_weights) -- the dropper landing does NOT close the
  controller-attention item, which is specifically about routing consumption. Net: C-45 gains
  one genuinely-wired sub-component; the flagged gap remains open and registered.

C-47_case_based_calibration_tool_use | 06-12 controller-attention: "OrchestratorCaseBaseSsb
  Emitter not attached (WIRE-DARK[C-47] runtime_builder.rs:407-435); NoopCaseBaseEmitter in
  prod; R9 settled but structural emitter absent" | 07-02: FLIPPED -- WIRED on a real seam.
  cli/src/runtime_builder.rs:411-416 still documents the REPL path as intentionally not
  attaching it (interactive ConversationRuntime path has no McpClientDispatch), BUT
  cli/src/run_subcommand.rs:290-300 now attaches the real emitter unconditionally:
  `mcp.with_calibration_ssb_emitter(Arc::new(OrchestratorCaseBaseSsbEmitter::new(
  wiring.ssb.clone())))` under `#[cfg(feature = "mcp")]`, and "mcp" is a DEFAULT-ON cargo
  feature (cli/Cargo.toml:75 `default = ["mcp"]`). This is a genuine non-test production
  wire on the `--via-mcp` CLI dispatch path. allowlist row (line ~728-733) now documents this
  precisely: "WIRED via the --via-mcp dispatch path only (in-process dispatch has no MCP seam
  by design)". VERDICT: C-47 flips SHIPPED_WIRE_DARK -> WIRED (on the MCP-dispatch entrypoint;
  the interactive-REPL entrypoint documented as intentionally out of scope, not a residual
  gap).

C-15_phase0_debt_clock_entropy | 06-12: PARTIAL ("entropy/hashmap regressed post-close")
  | 07-02: MIXED. 5 commits (981a412/675918f/1f32886/3d40940/dab4368, all 2026-06-28) quarantine
  83 -> 0 leak sites plus fix a Windows drive-letter false-positive in the checker's awk
  comment-stripper. 3d40940's commit message claims "check_entropy_quarantine.sh now exits 0,
  clean, 0 leaks." VERIFIED BY RUNNING THE SCRIPT (2026-07-02): `bash .elai_cc/scripts/
  check_entropy_quarantine.sh` exits 1, NOT 0 -- "FOUND 1 leak site(s): task_runner.rs:15303
  `let emit_start = std::time::Instant::now();`" (inside emit_reasoning, tagged "B-5 GAP-2" in
  the surrounding comment, part of the 07-02 eacd440 sweep). This is a fresh regression
  introduced AFTER the C-15 closure commits, by unrelated B-5 work landing in the same file
  (task_runner.rs) without an entropy-allow annotation. Category-wise this looks like the
  "monotonic Instant duration math" class the C-15 commits explicitly treated as entropy-allow
  elsewhere in the same file -- almost certainly benign, but the gate is not currently green
  as claimed. VERDICT: C-15 was genuinely CLOSED as of 3d40940 (06-28) but is RE-OPENED
  (1-site regression) as of the 07-02 eacd440 commit. Needs either the entropy-allow comment
  annotation or a ClockProvider thread at task_runner.rs:15303.

================================================================================
(c) PUNCH-ITEM RE-CHECK RESULTS
================================================================================

B DARK-3 (06-12: B-16, B-25, B-28 -- inferred by elimination: 06-04's 6 SHIPPED_WIRE_DARK
entries B-14/B-16/B-22/B-25/B-28/B-40 minus the 3 flips B-14/B-22/B-40 the master-index
names explicitly = B-16, B-25, B-28):
  - B-16 (check_anti_stale.sh CI false-positive on ppm_health.rs:65): not touched by any
    post-06-12 commit. UNVERIFIED this pass (out of delta scope, no commit to re-check against).
  - B-25 (Empire integration, external codebase gate): wire-dark-allowlist.toml carries the
    `empire` cargo-feature row (lines 413-420) confirming R9 is settled; the 47 EMPIRE-PENDING
    stubs remain gated on external Empire codebase access per design. UNCHANGED, correctly
    registered-dark, no drift.
  - B-28 (3 post-execute hooks claimed "never instantiated in production, None-only"):
    DISCREPANCY FOUND. Direct read of task_runner.rs:3763-3778 (inside `impl ResolvedConfig {
    fn from(cfg: TaskRunnerConfig) -> Self` at line 3629, the real config-resolution path, not
    a test) shows `focus_tunnel_pre_assembler: Some(Arc::new(AsyncMutex::new(
    PipelinePreAssembler::new())))` and `focus_tunnel_file_router: Some(Arc::new(AsyncMutex::
    new(SpeculativeFileRouter::new(2))))` -- both instantiated unconditionally with real
    defaults, non-test. `git log -L` on this line range shows the last touch was e38cd1a
    (2026-06-11), i.e. this code PREDATES the 06-12 audit baseline. This is not a post-06-12
    flip -- it appears to be a FALSE NEGATIVE in the 06-12 B-28 controller-attention entry
    itself (2 of 3 hooks were already wired when that audit ran; only focus_tunnel_dep_resolver
    stays conditionally-None, per its own comment, "only if the HTTP probe build failed").
    Flagged for the controller as a baseline-audit correction, not a new-work flip.

C DARK-4 (06-12; per R9-status text "8 gaps settled in C-31..C-59 range"): C-45 and C-47
  directly re-verified above (C-47 flips to WIRED, C-45 stays registered-dark). C-31 (was
  PARTIAL not WIRE_DARK in the 06-04 baseline, but the controller-attention item frames it as
  a dark-callsite gap) re-verified above: registered-dark, unchanged code-wise. C-22, C-34,
  C-41, C-49, C-50 (06-04 SHIPPED_WIRE_DARK entries not named in any 06-12 flip or controller-
  attention item, and not touched by any commit since 06-12 per the git-log scan): UNCHANGED,
  not re-verified this pass (outside delta scope -- no commit exists to check against).

D DARK-11 (06-12: WIRED_STATIC=16/PARTIAL=3/DARK=11, all WIRED_STATIC upgraded from prior
  runs' 06-07b tally, runtime leg UNVERIFIED-no-journal): ZERO commits since 06-12 touch any
  D-plan file (the only near-miss, 7b6d710, only touches self_mod_layer1a.rs which belongs to
  B-23 phase_4H_self_modification, already SHIPPED_WIRED in the B tally -- it is a path-helper
  bugfix, not a D-plan wiring change). D tally is UNCHANGED this pass; no re-verification
  performed (nothing to check against).

C 8 USER-DEFERRED external harnesses (C-51, C-53, C-54, C-55, C-56, C-57, C-58, C-59 --
  Aider-polyglot, SWE-bench-verified, NF4-kernel-counter, DKG-31-repo-eval, OpenHands-index,
  AgentForge-RRK, cross-config-master-report, all NOT_STARTED/PLANNED, RunPod/external-corpus
  dependent): none named in any of the 30 commits since 06-12. CONFIRMED still intentionally
  deferred; no drift into accidental-dark (they remain simply not-started, not mis-registered
  as something else).

================================================================================
(d) UPDATED PER-LETTER TALLY (06-12 baseline adjusted by verified flips only)
================================================================================
  letter  06-12(WIRED/PARTIAL/DARK/BENCH/OTHER)   07-02 verified deltas          adjusted read
  B       30 / 11 / 3 / 2 / 0                     B-4 1-of-3-items-closed (still PARTIAL);
                                                   B-5 evidence strengthened (already WIRED,
                                                   now grounded not just claimed); B-7 confirmed
                                                   unchanged PARTIAL; B-28 baseline-correction
                                                   (was miscounted DARK, likely PARTIAL --
                                                   2-of-3 hooks wired predates 06-12, not a
                                                   07-02 flip)                   30/11/3/2/0,
                                                                                 net tally
                                                                                 unchanged (no
                                                                                 verdict-class
                                                                                 crossed a
                                                                                 bucket boundary
                                                                                 on the evidence
                                                                                 gathered)
  C       25 / 21 / 4 / 1 / 8                     C-47 DARK->WIRED (1 flip, grounded);
                                                   C-15 CLOSED-then-REGRESSED (1-site entropy
                                                   leak, net still counts as PARTIAL not WIRED
                                                   since the gate does not pass green);
                                                   C-31/C-45 registered-dark (no bucket change,
                                                   still DARK, now WARN-class not BLOCK-class)
                                                                                 WIRED 26/21,
                                                                                 DARK 3/21,
                                                                                 (25->26 WIRED,
                                                                                 4->3 DARK)
  D       16 / 3 / 11 / 0 / 0                     no commits touch D; unchanged             16/3/11/0/0
                                                                                 (unverified
                                                                                 this pass)

SYSTEM B+C+D (134 plans): 06-12 = 71 WIRED / 35 PARTIAL / 18 DARK / 3 BENCH / 8 OTHER.
  07-02 verified: 72 WIRED / 35 PARTIAL / 17 DARK / 3 BENCH / 8 OTHER (net +1 WIRED, -1 DARK,
  from the single grounded C-47 flip; all other deltas found either stayed within their
  06-12 bucket [B-4, B-5, C-31, C-45] or are baseline corrections predating 06-12 [B-28] rather
  than post-06-12 work product).

================================================================================
(e) TOP FINDINGS FOR THE CONTROLLER
================================================================================
1. C-47 (case-based tool-use calibration) is a genuine, evidence-grounded flip to WIRED:
   OrchestratorCaseBaseSsbEmitter fires on the real `--via-mcp` CLI path (run_subcommand.rs:
   298-299, mcp feature default-on). Safe to reclassify.

2. glm-orchestrator process-integrity bug (disclosed in eacd440's own commit message): a
   dirt-baseline capture bug silently absorbed fixes from FAILED attempts into the next
   attempt's baseline, losing ~10 hours of B-plan work across B-2/B-4/B-5/B-6/B-7 until
   manually swept today. The bug itself is NOT yet fixed (orchestrator.py), only worked
   around by this one-off sweep commit. Recommend prioritizing the orchestrator.py fix before
   the next multi-attempt B/C/D run, or expect more silent-loss incidents.

3. C-15 (phase-0 entropy quarantine) is NOT currently green despite the 06-28 closure commits'
   claim. `check_entropy_quarantine.sh` exits 1 today (1 leak at task_runner.rs:15303,
   introduced by unrelated 07-02 B-5 work in the same file). Quick fix: add the entropy-allow
   annotation (monotonic Instant class, matches precedent already in the same file) or thread
   a ClockProvider.

4. B-28's controller-attention item ("3 hooks never instantiated, None-only") appears to be a
   FALSE NEGATIVE from the 06-12 audit itself -- 2 of 3 hooks (pre-assembler, file-router) were
   already unconditionally instantiated with real defaults in ResolvedConfig::from as of
   e38cd1a (2026-06-11), predating the audit. Only focus_tunnel_dep_resolver stays
   conditionally-None (HTTP-probe-dependent, documented). Recommend re-scoping B-28's DARK
   verdict to reflect this — likely PARTIAL not DARK.

5. B-4's DkgStats.failed_files gap is closed end-to-end (producer DkgReadyPayload.failed_files
   + real derivation at indexing_service.rs:754 + non-test emit call at task_runner.rs:5987),
   but commands.rs:2427's inline comment still claims "never plumbed" -- stale doc-comment,
   zero functional risk, one-line fix recommended so the next reader/auditor isn't misled.

6. C-31 and C-45's routing-consumer legs remain genuinely dark (zero non-test callers,
   verified by direct grep), but are now correctly REGISTERED-DARK (WIRE-DARK[C-N] marker +
   wire-dark-allowlist.toml row + promotion_plan_id), which is the intended non-BLOCK state
   per production-caller-gate.md. No action required beyond the already-named promotion plans
   (C-31-promotion, C-45).

7. B DARK-3 (B-16, B-25, B-28) and D DARK-11 were largely OUT of this delta's verifiable scope
   because no commits since 06-12 touch them (except the B-28 correction in item 4). A future
   session wanting a fresh D-plan number needs either new D-plan commits to anchor a delta
   check, or a scoped full re-sweep (outside this task's remit).
