WIRING FIX -- ELAI ITSELF (what .elai_cc must build going forward)
Written 2026-06-02. Grounded in WIRING-ENFORCEMENT-RESEARCH-2026-06-02.md (citations [n] -> its SOURCES)
and in ELAI spec/code read this session (C-27 wiring_contract.rs, Phase 3A/3B FSM+SSB, Phase 4G autonomous
gate, Phase 4H self-mod, F-plan W-1..W-10, E-45 cross-lang lock). Companion: WIRING-FIX-CLAUDE-CODE-WORKFLOW.md.
This is GROUNDED DESIGN, not yet built. It is C/F-plan-sized. Label every claim's evidence before acting.

==============================================================================
0. THE ONE-SENTENCE DIAGNOSIS (for ELAI's own runtime)
==============================================================================
ELAI already ships the entire wiring substrate -- C-27's DkgReachabilityAuditor + StubBanAuditor +
ObservableEffectRunner, the SSB event log, `elai wiring audit`, wiring-audit.yml CI -- but it is WIRE-DARK
ITSELF in the fail-open sense: EntrypointSetLoader warns instead of failing on an empty entrypoint set,
ObservableEffectRunner emits BrokenObservable non-fatally on timeout, and the SSB-row proof is not the
authoritative verdict input. The fix is not to invent machinery; it is to ACTIVATE the machinery that
exists and make every leg fail-CLOSED, anchored to the SSB row -- the one artifact a self-modifying agent
does not author. [research Rule 1/2/4][2][6][8]

==============================================================================
1. WHAT ELAI ALREADY HAS (and where it is fail-open) -- verified from spec
==============================================================================
  - C-27 wiring_contract.rs: WiringContract / StubBanEntry / ObservableEffect / EntrypointSet /
    EntrypointSetLoader (TOML [wiring.entrypoints], "empty + warn on absent" <- FAIL-OPEN) /
    StubBanAuditor (regex on todo!/unimplemented!/Ok(Default::default())) / DkgReachabilityAuditor (BFS from
    each entrypoint over DKG edges) / ObservableEffectRunner (1s SQLite poll, tokio::timeout, BrokenObservable
    on timeout -- no panic <- FAIL-OPEN). CLI `elai wiring audit`; CI .github/workflows/wiring-audit.yml.
  - SSB (Phase 3B): single-writer SQLite WAL, append-only, typed action_types, created_at via ClockProvider,
    one Ssb bound to (project_id, session_id). THIS is the off-agent observable trace -- the right anchor.
  - FSM (Phase 3A): EXECUTING->AUDITING evidence gate (">=1 FAIL->PASS"); 7 DAIRA evidence gates.
  - Autonomous (Phase 4G): AutoAdvanceEvaluator maps each transition to a deterministic EvidenceCheck.
    EXECUTING->AUDITING currently keys on "test flipped FAIL->PASS" -- a unit-test proxy, the exact blind spot.
  - Dispatch enums: SsbActionType, FsmSubstate, RouteDecisionEventKind, SkillEventPhase (Pattern-2 discriminators).
  - E-45: cross_lang_diff.py + action_type golden-file lock (append-only) -- the schema-drift leg already exists.
  - Phase 4H self-mod: AllowedConfigKey closed enum, LockedKeysAuditor, 4 enforcement layers
    (dispatch + IPC + filesystem + git), self-mod agent CANNOT read the lock list, L3 = human-only.
  - DKG (Phase 1A) + SCIP elai.map: the static reachability source -- but NEITHER stores a sound call graph
    for trait-object / IPC-string dispatch. [research HIGH risk]

==============================================================================
2. THE FOUR-LAYER FAIL-CLOSED GATE, MAPPED ONTO ELAI
==============================================================================
Defense-in-depth (research section 5): a sub-phase cannot reach MEMORIZING unless ALL layers pass. Any layer
returning non-zero / not-found / empty-evidence = FAIL. Cheapest first; a Layer-1 fail blocks 2-4.

------------------------------------------------------------------------------
LAYER 1 -- COMPILE-TIME (every commit, ~seconds, static, cannot be gamed cheaply)
------------------------------------------------------------------------------
  1a. RUSTFLAGS="-D dead_code" cargo check --workspace as a CI floor. CAVEAT: rustc's dead_code lint EXEMPTS
      pub items entirely (it assumes an external crate may call them) -- open issues #120079 (workspace
      analysis) and #74970 (binary-crate pub-as-pub(crate)) remain unresolved [10][11]. So adopt the
      pub(crate) DISCIPLINE: anything not part of a genuine cross-crate API surface is pub(crate), at which
      point dead_code fires on unused internal items. This is the floor, not the ceiling.
  1b. Enum exhaustiveness as a free always-on fitness function [research 3.4]: REMOVE #[non_exhaustive] from
      internal dispatch enums (SsbActionType dispatch, FsmSubstate, RouteDecisionEventKind, SkillEventPhase).
      Adding a variant without a match arm becomes a COMPILE ERROR -- a producer that is not dispatched
      cannot compile. Note the limit: this proves a dispatch ARM exists, not that the dispatcher is reached
      from a production entrypoint (Layer 3 covers that).
  1c. dashboard-v2: knip --production from the real Svelte entrypoints (test entrypoints excluded from the
      root set) [15]. Unreachable export from production root = BLOCK. Gate must exit nonzero on zero modules.
  1d. python/elai_inference: vulture --min-confidence with a checked-in whitelist for decorator-registered
      routes (@app.post etc., which vulture cannot see) [16]. Whitelist entries require justification comments.
  1e. cargo-machete (v0.9.2 [28]) for dead Cargo deps. Catches dead crates, not dead fns -- complementary only.

------------------------------------------------------------------------------
LAYER 2 -- CROSS-LANGUAGE WIRING CONTRACT (every commit, ~seconds, static)
------------------------------------------------------------------------------
This is the C-27 wiring_contract.rs layer, made fail-closed:
  2a. EntrypointSetLoader: empty / absent entrypoint set = FAIL, not warn. A wiring audit with no roots
      proves nothing; vacuous pass is the same bug class as the harness SKIP-pass. [research Rule 1, ArchUnit
      failOnEmptyShould 25]
  2b. DkgReachabilityAuditor producer-without-(non-test)-caller = FAIL, propagated to the audit verdict.
      Test-only callers (tests/, #[cfg(test)]) are a SEPARATE root set and do NOT satisfy production
      reachability [9 Go deadcode -test insight]. This enforces W-8, which is currently unenforced.
  2c. Tauri symmetry: every name in generate_handler!() requires >=1 invoke() in dashboard-v2 production TS;
      every @app.post/@router route in server.py requires >=1 caller in crates/cli runtime_client.rs.
      Missing either side = FAIL. Reuse cross_lang_diff.py (E-45) as the executor.
  2d. action_type golden lock (E-45): new SsbActionType variants must flow through the append-only golden
      file; cross_lang_diff --strict green. Already exists -- make it a hard gate, not advisory.
  2e. HONEST LIMIT: DKG/SCIP have no call-graph edges for dyn-Trait / IPC-string / event-bus dispatch
      [research HIGH risk]. A vtable-dispatched method shows zero references and is FALSELY flagged dead.
      MITIGATION: [wiring.entrypoints] in elai_cc.toml must enumerate trait-dispatch sites; a produces that
      "implements dispatched trait T, T reachable from entrypoint E" is accepted as wired. Layer 3 is the
      real backstop for everything static analysis cannot see.

------------------------------------------------------------------------------
LAYER 3 -- ENTRYPOINT COVERAGE (gated before EXECUTING->AUDITING, ~minutes, runtime binary)
------------------------------------------------------------------------------
The compensating control for Rust's unsound static reachability. This is the layer ELAI does NOT yet have.
  3a. Build the real binary instrumented: RUSTFLAGS="-C instrument-coverage" cargo build --bin elai.
      Run the smoke scenarios (auditor-owned, section 4) against the REAL binary with PRODUCTION-DEFAULT env
      (all ELAI_*_ENABLE at shipped defaults, no test overrides). Merge .profraw; cargo-llvm-cov export
      --format=json [21]. Parse functions with count==0; diff against functions introduced in the current
      plan's git diff; any NEW function with count==0 = BLOCK. Suppression: #[cfg(test)] (not in prod binary)
      OR a registered // WIRE-DARK[B-N]: <justification> stub per the existing stub-tracking.md registry.
  3b. python/elai_inference: coverage run -m uvicorn elai_inference.server:app (the REAL entrypoint, not
      pytest); coverage json; zero-hit new functions = BLOCK.
  3c. A feature behind a default-OFF flag will correctly show zero coverage under production defaults -- that
      IS the wire-dark signal, and it is correct behaviour. The flag must be in wire-dark-allowlist.toml
      (section 5) to be exempt; otherwise BLOCK. [research 3.3 fail-open note]
  3d. smoke-manifest.toml (per B-N): plan, entrypoint, expect_ssb, flag_default. Auditor verifies the entry
      exists and the smoke run drives the declared entrypoint. Missing entry for a new feature = BLOCK.

------------------------------------------------------------------------------
LAYER 4 -- RUNTIME TRACE ASSERTION (every PR, in-process, the un-gameable layer)
------------------------------------------------------------------------------
ELAI's SSB IS the OTel-InMemorySpanExporter equivalent [29] -- an in-process, off-agent artifact. Fix
ObservableEffectRunner to use it as a hard oracle:
  4a. Drive the REAL entrypoint (CLI subcommand / FSM transition / IPC handler), NOT the produced function
      directly. A test that calls the function directly reproduces the unit-test proxy and proves nothing.
  4b. Bind an in-process Ssb to a temp db BEFORE invoking production code; after the entrypoint returns,
      query for the contract's expect_ssb action_type within the time window. Zero rows OR db-not-found =
      FAIL (fix the BrokenObservable-non-fatal-on-timeout and the path bug -- missing evidence is FAIL, not
      a soft signal). [research Rule 1/2; the InMemory pattern makes "sink missing" structurally impossible 29]
  4c. The Rust log/span leg: tracing-test (v0.2.6) #[traced_test] + an EXPLICIT assert!(logs_contain("<key>"))
      -- the macro does NOT auto-fail, so a test without the assert silently passes [31]. The auditor rule
      must reject any #[traced_test] integration test that lacks a logs_contain/logs_assert call.
  4d. If/when OTel spans are added: SimpleSpanProcessor + InMemorySpanExporter ONLY. BatchSpanProcessor is
      BANNED in test setup -- its flush timing causes spans to miss the assertion -> silent fail-open [30].

==============================================================================
3. F-PLAN W-1..W-10, CORRECTED AND MAPPED TO LAYERS
==============================================================================
  W-1 (no-wire-dark done-gate)         -> enforced by Layer 2 (reachability) + Layer 3 (coverage) +
                                          Layer 4 (SSB row). SSB row MUST be FAIL-if-absent, not UNVERIFIED.
  W-2 (no default-off terminal state)  -> Layer 1/3 + wire-dark-allowlist.toml (section 5). FAIL if a new
                                          flag defaults off and is not in the allowlist.
  W-3 (production entrypoint required) -> Layer 2 (test-callers are a separate root set) + Layer 3.
  W-4 (frontend pair mandatory)        -> Layer 2c Tauri symmetry + E-45 cross_lang_diff.
  W-5 (observable_effect is live E2E)  -> Layer 3 (drive real binary) + Layer 4 (assert off-agent SSB row).
                                          CORRECTION: the trigger must drive a real entrypoint, and the
                                          artifact is the SSB row, not stdout.
  W-6 (dashboard integration checkpt)  -> Layer 3c (Playwright E2E coverage if dashboard in scope).
  W-7 (no hallucinated integration pt) -> Layer 1b enum exhaustiveness + Layer 2 reachability at author time.
  W-8 (produces/consumes symmetry)     -> Layer 2b, now FAIL-feeding-the-verdict (was unenforced).
  W-9 (SSB variant + golden lock)      -> Layer 2d (E-45 action_type golden), made a hard gate.
  W-10 (tiered gate; no fake fail-closed)-> the whole point: each layer is genuinely fail-closed; network/
                                          subprocess legs (Layer 3 build/run) are separately gated but their
                                          absence is FAIL, never an advertised-but-fake pass.

==============================================================================
4. THE AUTONOMOUS-LOOP FIX (your explicit concern: never leave dead code behind)
==============================================================================
The Phase 4G AutoAdvanceEvaluator is exactly where wire-dark slips in: its EXECUTING->AUDITING EvidenceCheck
keys on "cargo test exit==0 AND test flipped FAIL->PASS" -- a unit-test proxy with zero causal link to wiring
[research section 2]. CHANGE: the EXECUTING->AUDITING (and AUDITING->MEMORIZING) EvidenceCheck must require
Layer 3 (entrypoint coverage of new functions) AND Layer 4 (SSB-row assertion) to pass. cargo-green-alone
can NEVER auto-advance. This single change is what makes an autonomous ELAI run structurally unable to ship an
orphan: the gate now measures an artifact the run did not author, produced by a real entrypoint. [4][7]

==============================================================================
5. FREEZE THE GATE -- COMPOSE WITH PHASE 4H SELF-MOD LOCKS
==============================================================================
Research Rule 8: an agent that can rewrite its own gate writes itself to PASS [2][6][8]. ELAI already has the
mechanism -- reuse it:
  - Add the wiring gate fileset (wiring_contract.rs gate logic, the smoke harness, smoke-manifest.toml,
    wire-dark-allowlist.toml, [wiring.entrypoints]) to the Phase 4H AllowedConfigKey LOCKED set / LockedKeysAuditor.
    The self-modifying agent CANNOT read or edit the lock list, and the pre-commit hook + git-diff filter
    already BLOCK commits touching LOCKED_KEYS-owning files. The wiring gate becomes L3-frozen by construction.
  - wire-dark-allowlist.toml rows require type (kill_switch | experiment), owner, expiry_date,
    promotion_plan_id, and an ON-path smoke test [37][38][40]. A flag with no promotion path is wire-dark by
    design and is BLOCKED.

==============================================================================
6. COST / WHERE EACH LAYER RUNS
==============================================================================
  Every commit (cheap, static):  Layer 1 (compile-time), Layer 2 (cross-lang contract).
  Gated before AUDITING (runtime): Layer 3 (entrypoint coverage), Layer 4 (SSB trace assertion).
  Order: Layer 1 fail blocks 2-4; Layer 2 fail blocks 3-4; 3 and 4 parallel after 1+2 pass.
  This matches ELAI's existing scoped-test discipline (per-crate -p during EXECUTING, full gate at AUDITING).

==============================================================================
7. SUGGESTED PLAN SHAPE (this is C/F-plan-sized, not a patch-N)
==============================================================================
  Phase 1 (Layer 1+2, low risk): pub(crate) sweep + -D dead_code CI; strip #[non_exhaustive] from internal
    dispatch enums; knip --production + vulture whitelist; fix wiring_contract.rs EntrypointSetLoader empty=FAIL
    + DkgReachabilityAuditor producer-without-caller=FAIL feeding verdict; Tauri/sidecar symmetry via cross_lang_diff.
  Phase 2 (Layer 3): cargo-llvm-cov entrypoint-coverage runner + smoke-manifest.toml + count==0-new-fn=BLOCK
    + WIRE-DARK[B-N] stub registry integration (stub-tracking.md).
  Phase 3 (Layer 4): fix ObservableEffectRunner to drive real entrypoint + assert SSB row + FAIL-if-absent;
    tracing-test mandatory-assert auditor rule; SimpleSpanProcessor-only if OTel added.
  Phase 4 (autonomous + freeze): AutoAdvanceEvaluator EvidenceCheck requires Layer 3+4; gate fileset added to
    Phase 4H LOCKED set; wire-dark-allowlist.toml with expiry/promotion enforcement.
  Each phase's OWN done-gate is the gate it builds (dogfood: the wiring gate must pass its own wiring contract,
    driven from `elai wiring audit`, leaving a real SSB row).

==============================================================================
8. RESIDUAL RISK (carried verbatim from the research; do not oversell)
==============================================================================
  - No sound RTA tool for Rust with std + tokio + dyn Trait [research OPEN QUESTIONS]. Layer 1/2 are
    approximations with false-positives on trait dispatch and false-negatives on dynamic paths. Layer 3
    runtime coverage is the only sound backstop -- treat it as load-bearing, not optional.
  - SCIP/DKG have no call-graph edges; [wiring.entrypoints] trait-dispatch enumeration can go stale. Audit it.
  - Smoke coverage is only as good as the manifest; an under-specified manifest under-covers. Auditor owns it;
    measure coverage of the manifest itself.
  - Wire-dark has no external benchmark [45]; ELAI's 40-45% is self-measured. Instrument the rate
    (wire-dark plans / total, via an SSB action_type) so this redesign is judged on measured deltas, not faith.
End of WIRING-FIX-ELAI.md.
