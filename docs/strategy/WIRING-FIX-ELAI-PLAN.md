WIRING-FIX-ELAI -- EXECUTABLE PLAN (the Rust runtime redesign)
Drafted 2026-06-02. Plan only; NO Rust edits yet. Grounds: WIRING-FIX-ELAI.md (design),
WIRING-ENFORCEMENT-RESEARCH-2026-06-02.md (citations [n]). C/F-plan-sized. Run via b-plan-runner /
orchestrate, NOT as a patch-N. Each phase's OWN done-gate is the gate it builds (dogfood): the wiring
gate must pass its own WIRING_CONTRACT, driven from `elai wiring audit`, leaving a real SSB row.

Canonical location note: this file sits with its design doc in docs/strategy/. Move/copy into the
canonical ELAI plan dir (docs/thefinalglance/{C,F}-plan/) before orchestrating if the runner expects it there.

=========================================================================================
PHASE 0 -- GROUNDING GATE (no code; must pass before Phase 1) [VERIFY, blocking]
=========================================================================================
The design doc asserts these C-27 / Phase-3/4 symbols exist "from spec". This plan does NOT trust that;
it verifies first. Spawn context-compiler + integration-explorer-elai (orchestrate protocol) to CONFIRM,
via elai.map / source read, the real path + signature of each before any edit:
  - wiring_contract.rs: WiringContract, StubBanEntry, ObservableEffect, EntrypointSet, EntrypointSetLoader,
    StubBanAuditor, DkgReachabilityAuditor, ObservableEffectRunner; CLI `elai wiring audit`;
    .github/workflows/wiring-audit.yml.
  - SSB (Phase 3B): the Ssb writer type, its action_type enum (SsbActionType), the real on-disk db path.
  - FSM (Phase 3A) EXECUTING->AUDITING evidence gate; AutoAdvanceEvaluator (Phase 4G) + its EvidenceCheck.
  - Phase 4H: AllowedConfigKey, LockedKeysAuditor, the 4 enforcement layers.
  - Dispatch enums: SsbActionType, FsmSubstate, RouteDecisionEventKind, SkillEventPhase.
DELIVERABLE: workflow/reasoning/wiring-elai-phase0-grounding.md classifying each symbol GROUNDED/VERIFIED/
ABSENT. ACCEPTANCE: every symbol VERIFIED with a real file:line, OR the plan is re-scoped around what
actually exists. ANY ABSENT symbol that the later phases depend on = STOP, re-plan. No edits in Phase 0.

=========================================================================================
PHASE 1 -- LAYER 1 + 2 (compile-time + cross-language contract, low risk)
=========================================================================================
GOAL: make the static legs fail-closed; they are the cheap floor that blocks the dynamic legs.
DELIVERABLES:
  1.1  RUSTFLAGS="-D dead_code" cargo check --workspace as a CI floor + adopt pub(crate) discipline
       (rustc dead_code EXEMPTS pub items -- issues #120079/#74970 open [10][11]; pub(crate) makes the lint
       fire on unused internal items). Floor, not ceiling.
  1.2  Strip #[non_exhaustive] from INTERNAL dispatch enums (SsbActionType dispatch, FsmSubstate,
       RouteDecisionEventKind, SkillEventPhase) so a producer variant with no match arm is a COMPILE error.
       NOTE limit: proves a dispatch ARM exists, not that the dispatcher is reached (Phase 3 covers that).
  1.3  EntrypointSetLoader: empty / absent entrypoint set = FAIL, not warn (ArchUnit failOnEmptyShould [25]).
  1.4  DkgReachabilityAuditor: producer-without-(non-test)-caller = FAIL, propagated to the audit verdict.
       Test-only callers (tests/, #[cfg(test)]) are a SEPARATE root set [9]. Enforces W-8 (was unenforced).
  1.5  Tauri symmetry (generate_handler! <-> TS invoke()) + sidecar route <-> Rust client symmetry via
       cross_lang_diff.py (E-45); action_type golden lock made a HARD gate (was advisory).
  1.6  HONEST LIMIT: DKG/SCIP have no call-graph edges for dyn-Trait/IPC dispatch [research HIGH risk].
       [wiring.entrypoints] in elai_cc.toml must enumerate trait-dispatch sites; "implements dispatched trait
       T reachable from entrypoint E" is accepted as wired. Layer 3 is the real backstop.
ACCEPTANCE: a deliberately-orphaned pub(crate) fn fails -D dead_code; an empty entrypoint set fails the
audit; a producer with only a test caller fails the audit verdict (not a warning); cross_lang_diff --strict
green. Risk: false positives on trait dispatch (mitigated by 1.6).

=========================================================================================
PHASE 2 -- LAYER 3 (entrypoint coverage; the missing compensating control)
=========================================================================================
GOAL: catch what unsound Rust static analysis cannot -- run the real binary, prove new code executed.
DELIVERABLES:
  2.1  Entrypoint-coverage runner: RUSTFLAGS="-C instrument-coverage" cargo build --bin elai; run the
       auditor-owned smoke scenarios against the REAL binary with PRODUCTION-DEFAULT env (all ELAI_*_ENABLE
       at shipped defaults); merge .profraw; cargo-llvm-cov export --format=json [21]. (anti-slopsquatting:
       confirm cargo-llvm-cov version live before adding.)
  2.2  Gate logic: parse functions with count==0, diff against functions introduced in the plan's git diff;
       any NEW function with count==0 = BLOCK. Suppression: #[cfg(test)] OR a registered
       // WIRE-DARK[B-N]: <justification> stub per stub-tracking.md.
  2.3  python/elai_inference: coverage run -m uvicorn elai_inference.server:app (REAL entrypoint, not
       pytest); coverage json; zero-hit new functions = BLOCK.
  2.4  smoke-manifest.toml (already seeded at .elai_cc/smoke-manifest.toml) becomes load-bearing: the
       auditor verifies a [[feature]] entry exists per new feature and the smoke run drives the declared
       entrypoint. Missing entry = BLOCK. A default-OFF flag correctly shows zero coverage -> must be in
       wire-dark-allowlist.toml (already seeded) or BLOCK.
ACCEPTANCE: a new fn never called from the smoke run is BLOCK; the same fn with a WIRE-DARK[B-N] registered
stub passes; a new feature with no smoke-manifest entry is BLOCK. Risk: an under-covering smoke manifest
(auditor owns it; measure coverage of the manifest itself).

=========================================================================================
PHASE 3 -- LAYER 4 (runtime trace assertion; the un-gameable leg)
=========================================================================================
GOAL: the SSB row is the off-agent oracle. Make ObservableEffectRunner fail-CLOSED on it.
DELIVERABLES:
  3.1  Drive the REAL entrypoint (CLI subcommand / FSM transition / IPC handler), NOT the produced fn directly.
  3.2  Bind an in-process Ssb to a temp db BEFORE invoking production code; after the entrypoint returns,
       query for the contract's expect_ssb action_type in the window. Zero rows OR db-not-found = FAIL.
       Fix the BrokenObservable-non-fatal-on-timeout and the path bug -- missing evidence is FAIL, not soft
       (the InMemory pattern makes "sink missing" structurally impossible [29]).
  3.3  Rust log/span leg: tracing-test (v0.2.6) #[traced_test] + an EXPLICIT assert!(logs_contain("<key>"))
       -- the macro does NOT auto-fail [31]. Auditor rule REJECTS any #[traced_test] test lacking the assert.
  3.4  If/when OTel spans added: SimpleSpanProcessor + InMemorySpanExporter ONLY; BatchSpanProcessor BANNED
       in test setup (flush timing -> silent fail-open [30]). Use real gen_ai.* attribute names, not fabricated.
ACCEPTANCE: a feature whose entrypoint runs but emits no SSB row FAILs; a #[traced_test] with no logs_contain
is rejected by the auditor rule; db-not-found returns FAIL not BrokenObservable-soft.

=========================================================================================
PHASE 4 -- AUTONOMOUS LOOP + FREEZE (the change that stops orphan auto-ship)
=========================================================================================
GOAL: an autonomous ELAI run becomes structurally unable to ship wire-dark.
DELIVERABLES:
  4.1  AutoAdvanceEvaluator EvidenceCheck for EXECUTING->AUDITING (and AUDITING->MEMORIZING) MUST require
       Layer 3 (entrypoint coverage of new fns) AND Layer 4 (SSB-row assertion). cargo-green-alone can NEVER
       auto-advance. This single change makes the gate measure an artifact the run did not author [4][7].
  4.2  Add the wiring gate fileset (wiring_contract.rs gate logic, smoke harness, smoke-manifest.toml,
       wire-dark-allowlist.toml, [wiring.entrypoints]) to the Phase 4H AllowedConfigKey LOCKED set /
       LockedKeysAuditor. The self-mod agent CANNOT read/edit the lock list; pre-commit + git-diff filter
       BLOCK commits touching LOCKED-owning files. The gate becomes L3-frozen by construction (R8).
  4.3  wire-dark-allowlist.toml rows enforced: type/owner/expiry_date/promotion_plan_id + ON-path smoke test.
       A flag past expiry, or default-OFF without a row, is a build break [37][38][40].
ACCEPTANCE: a run that is cargo-green but emits no SSB row cannot auto-advance EXECUTING->AUDITING; a commit
touching a LOCKED gate file is blocked; an expired allowlist flag breaks the build.

=========================================================================================
W-1..W-10 MAPPING (corrected, from WIRING-FIX-ELAI.md section 3) -- which phase enforces each
=========================================================================================
  W-1 -> Phase 1 (reachability) + 2 (coverage) + 3 (SSB row, FAIL-if-absent)
  W-2 -> Phase 1/2 + wire-dark-allowlist.toml (Phase 4.3)
  W-3 -> Phase 1 (test-callers a separate root set) + Phase 2
  W-4 -> Phase 1.5 Tauri/sidecar symmetry + E-45
  W-5 -> Phase 2 (drive real binary) + Phase 3 (SSB row, not stdout)
  W-6 -> Phase 2.x Playwright E2E coverage if dashboard in scope
  W-7 -> Phase 1.2 enum exhaustiveness + Phase 1 reachability
  W-8 -> Phase 1.4 (FAIL feeding verdict; was unenforced)
  W-9 -> Phase 1.5 action_type golden, hard gate
  W-10 -> the whole stack: each layer genuinely fail-closed; absence is FAIL, never advertised-but-fake pass

=========================================================================================
SEQUENCING / RISK
=========================================================================================
  Order: Phase 0 (grounding) -> 1 (cheap static) -> 2 (coverage) -> 3 (SSB oracle) -> 4 (autonomous+freeze).
  Layer 1 fail blocks 2-4; Layer 2 fail blocks 3-4; 3 and 4 of the same feature run after 1+2.
  No sound RTA for Rust (std+tokio+dyn Trait) -- Layer 1/2 are approximations; Layer 3 runtime coverage is
  the load-bearing backstop, not optional. SCIP/DKG have no call-graph edges; [wiring.entrypoints] trait-
  dispatch enumeration can go stale -- audit it. Wire-dark has no external benchmark [45]; instrument the
  rate (wire-dark plans / total via an SSB action_type) so this redesign is judged on measured deltas.
  This plan is NOT built. It is grounded design + decomposition. Build via orchestrate/b-plan after Phase 0.
End of WIRING-FIX-ELAI-PLAN.md.
