wiring-elai-phase0-grounding.md

Phase 0 grounding gate for WIRING-FIX-ELAI. Date 2026-06-02. Scope: verify every symbol the four implementation phases depend on against real .elai_cc source, classify each claim, and gate PROCEED vs STOP_REPLAN. Acceptance: ANY blocking absence of a depended-on symbol => STOP_REPLAN.

OVERALL VERDICT: PROCEED_WITH_RESCOPE

Rationale: no symbol that a later phase depends on is ABSENT. The single ABSENT artifact (.github/workflows/wiring-audit.yml) has a live functional equivalent (CI step in windows.yml:112-119), so it is a re-scope item, not a hard blocker. However, the design doc was WRONG about the current code in several load-bearing places (REFUTED/DIFFERENT verdicts below), and the central thesis -- activate existing machinery + make every leg fail-CLOSED -- is confirmed: the gate legs exist but are fail-open in exactly the targeted spots. The plan must adapt to the corrections before EXECUTING.

CLASSIFICATION TABLE (NAME | STATUS | file:line | claim_verdict)

Subsystem A -- wiring_contract.rs core types + auditors (C-27)
WiringContract | VERIFIED | crates/orchestrator/src/wiring_contract.rs:154-169 | CONFIRMED
StubBanEntry | VERIFIED | wiring_contract.rs:135-139 | NO_CLAIM
ObservableEffect(contract-decl) | VERIFIED | wiring_contract.rs:141-149 | NO_CLAIM
WiringEntrypointSet | VERIFIED | wiring_contract.rs:202-219 | DIFFERENT (real name, doc says 'EntrypointSet')
WiringEntrypointSetLoader | VERIFIED | wiring_contract.rs:225-270 | CONFIRMED (empty AND absent both FAIL-OPEN)
StubBanAuditor | VERIFIED | wiring_contract.rs:395-527 | CONFIRMED (anchored regexes; test files skipped; unsound name-grep)
DkgReachabilityAuditor | VERIFIED | wiring_contract.rs:564-659 | CONFIRMED (W-8 UNENFORCED; graph fabricated by driver 903-914)
ObservableEffectRunner | VERIFIED | wiring_contract.rs:671-798 | DIFFERENT (manual deadline loop; runner Errs+emits; DRIVER drops Err at 945)

Subsystem B -- wiring-audit CLI + CI surface
CliAction::WiringAudit | VERIFIED | crates/cli/src/args.rs:362-366 | CONFIRMED
main.rs dispatch WiringAudit | VERIFIED | crates/cli/src/main.rs:775-782 | CONFIRMED
wiring_audit::run | VERIFIED | crates/cli/src/wiring_audit.rs:59-108 | CONFIRMED (exit 1 on !is_clean)
WiringAuditCmd::run | VERIFIED | wiring_contract.rs:827-959 | CONFIRMED (invokes all 3 auditors)
DkgReachability skip-on-empty-entrypoints | VERIFIED | wiring_contract.rs:897-901 | DIFFERENT (FAIL-OPEN warn-and-skip; live toml block commented)
WiringAuditError (zero-contracts) | VERIFIED | wiring_contract.rs:1038-1046 | REFUTED (no Empty variant; zero contracts => exit 0)
ObservableEffectRunner best-effort swallow | VERIFIED | wiring_contract.rs:942-945 | DIFFERENT (let _ drops Err; violation row still counted)
.github/workflows/wiring-audit.yml | ABSENT | searched both workflow dirs | REFUTED (no such file)
CI step 'Wiring contract audit (C-27 T-0)' | VERIFIED | .elai_cc/.github/workflows/windows.yml:112-119 | CONFIRMED (build-failing; Windows-only; [ci skip] bypass)

Subsystem C -- SSB (off-agent observable trace; Layer-4 evidence)
Ssb (writer) | VERIFIED | crates/orchestrator/src/ssb.rs:14145 | CONFIRMED (single-writer WAL, append-only, ClockProvider)
SsbActionType | VERIFIED | ssb.rs:4650 | DIFFERENT (482 variants not ~100; NOT non_exhaustive; SCHEMA 205)
SSB db path resolution | VERIFIED | ssb.rs:14758-14771 | CONFIRMED (CWD-rooted default, no env var)
Ssb::write_payload / write / SsbWriteRequest | VERIFIED | ssb.rs:14265 | CONFIRMED (Layer-4 assertion target; fail-closed primary path)
SsbEntry / SsbConfig / SsbError | VERIFIED | ssb.rs:13935 | NO_CLAIM

Subsystem D -- FSM evidence gate (3A) + autonomous loop (4G)
TestFlippedGate::check | VERIFIED | crates/orchestrator/src/evidence.rs:449-472 | CONFIRMED (predicate any flipped_to_pass; unit-test proxy)
TestOutcome | VERIFIED | evidence.rs:60-69 | CONFIRMED (flipped_to_pass bool; in-boundary)
AutoAdvanceEvaluator | VERIFIED | crates/orchestrator/src/autonomous_gate.rs:621 | CONFIRMED (AutoAdvance defers to Evidence::all_passed)
EvidenceCheck | VERIFIED | autonomous_gate.rs:579-590 | CONFIRMED (artifact_path display-only, never verified)
Evidence::test_flipped / all_passed | VERIFIED | autonomous_gate.rs:246-248,351-367 | CONFIRMED (2nd unit-test-proxy instance)
derive_autonomous_evidence | VERIFIED | crates/orchestrator/src/task_runner.rs:13456-13542 | CONFIRMED (FAIL-OPEN: empty rows => test_flipped vacuously true)
emit_test_outcome | VERIFIED | task_runner.rs:12476-12494 | CONFIRMED (in-boundary; producer==grader, R4)
DiagnosticProbeGate | VERIFIED | evidence.rs:684-728 | DIFFERENT (real LSP artifact but FAIL-OPEN on None/TimedOut/ToolMissing)
AntiFabricationGate | VERIFIED | evidence.rs:752-788 | DIFFERENT (off-agent stdout_hash but AUDITING->MEMORIZING + FAIL-OPEN on absent rows)
check_transition / gates_for_transition / all_gates | VERIFIED | evidence.rs:797-850 | DIFFERENT (empty gate set => Ok/PASS, R1 risk)

Subsystem E -- Phase 4H self-modification locks
AllowedConfigKey | VERIFIED | crates/orchestrator/src/self_mod_layer1a.rs:202-208 | CONFIRMED (closed 2-variant; Parameter always rejected at L1A)
LockedKeysAuditor | VERIFIED | crates/orchestrator/src/locked_keys_auditor.rs:335-477 | CONFIRMED (fail-closed unreadable registry; empty registry fail-open by bootstrap policy)
self_mod_safety 4-layer module | VERIFIED | crates/orchestrator/src/self_mod_safety.rs:1-23 | DIFFERENT (only Dispatch+Git live in-tree; IPC+fs-sandbox deferred/external)
git pre-commit hook (Layer 4) | VERIFIED | .elai_cc/.git/hooks/pre-commit + scripts/locked_keys_check.sh | CONFIRMED (drives elai audit-locked-keys --staged, fail-closed)
check_lock_list_read / LOCKED_KEYS | VERIFIED | self_mod_safety.rs:230-236,156-175 | CONFIRMED (structurally prevented; lock_list self-protecting)
StructuralChange / apply_structural_change (L3) | VERIFIED | self_mod_types.rs:417-425 + self_mod_safety.rs:242-254 | CONFIRMED (human-only tier; AllowedConfigKeyAddition variant)
locked_keys_registry.toml (extension point) | VERIFIED | .elai_cc/scripts/locked_keys_registry.toml:1-132 | CONFIRMED (new path lock = TOML append; INNER repo only)

Subsystem F -- internal dispatch enums (Plan Phase 1.2)
SsbActionType | VERIFIED | ssb.rs:4650 | CONFIRMED no non_exhaustive => strip is NO-OP; exhaustive as_sql_str 8364
FsmSubstate | VERIFIED | crates/orchestrator/src/fsm.rs:125 | CONFIRMED no non_exhaustive => strip is NO-OP; exhaustive as_str 175-187
RouteDecisionEventKind | VERIFIED | ssb.rs:24375 | CONFIRMED no non_exhaustive => strip is NO-OP; doc says 'Closed enum'
SkillEventPhase | VERIFIED | ssb.rs:21235 | CONFIRMED no non_exhaustive => strip is NO-OP; exhaustive as_str 21244-21254

Subsystem G -- cross-language wiring (E-45) + seed files
cross_lang_diff.py | VERIFIED | .claude/tools/cross_lang_diff.py:1-568 | CONFIRMED (14 hardcoded pairs; action-type leg OPT-IN R3; advisory/unwired)
cross-lang-pairs.toml registry | VERIFIED | .elai_cc/cross-lang-pairs.toml:1-49 | CONFIRMED (6 rows 1:1 PairKind)
PairKind (E-45) | VERIFIED | crates/orchestrator/src/cross_lang_drift/types.rs:12-58 | CONFIRMED
elai drift CLI entrypoint | VERIFIED | crates/cli/src/main.rs:523-524 + drift_cmd.rs:236 | CONFIRMED (sound Rust leg)
action_type_lock.golden | VERIFIED | crates/orchestrator/tests/action_type_lock.golden | CONFIRMED (HARD-GATE, panics, append-only, fail-closed on missing)
TOTAL_VARIANTS / e45_f7 staleness | VERIFIED | action_type_lock.rs:534 vs contracts/tests/e45_cross_language_drift_contract.rs:294 | DIFFERENT (live 482 vs e45_f7 asserts 394 => stale/broken contract)
Tauri generate_handler! | VERIFIED | crates/dashboard-v2/src-tauri/src/lib.rs:430 | CONFIRMED (real, bidirectional invoke())
sidecar HTTP routes + Rust callers | VERIFIED | python/elai_inference/server.py:3209-4654 + providers/runtime/sidecar.rs | DIFFERENT (callers NOT in runtime_client.rs)
smoke-manifest.toml (seed) | VERIFIED | .elai_cc/smoke-manifest.toml:1-31 | DIFFERENT (exists but ZERO active [[feature]] entries)
wire-dark-allowlist.toml (seed) | VERIFIED | .elai_cc/wire-dark-allowlist.toml:1-62 | DIFFERENT (exists but ZERO active exemptions)
[wiring.entrypoints] in elai_cc.toml (seed) | VERIFIED | .elai_cc/elai_cc.toml:106-110 | REFUTED (commented out; fail-open dormant template)

Subsystem H -- external tooling (anti-slopsquatting + anti-stale)
cargo-llvm-cov | VERIFIED | crates.io v0.8.7 (2026-05-13) | DIFFERENT (JSON flag is --json not --format=json)
cargo-machete | VERIFIED | crates.io v0.9.2 (2026-04-15) | CONFIRMED (watch typo-squat cargo-machete-nk)
knip | VERIFIED | npm v6.15.0 | DIFFERENT (--production confirmed; 'failure-on-zero-modules' not native, needs wrapper)
vulture | VERIFIED | pypi v2.16 (2026-03-25) | CONFIRMED (--min-confidence + --make-whitelist)
tracing-test | VERIFIED | crates.io v0.2.6 (2026-02-07) | CONFIRMED (logs_contain -> bool, does NOT auto-fail)
cargo-mutants | VERIFIED | crates.io v27.0.0 (2026-03-07) + install_dev_tools.sh:20 | CONFIRMED (--in-diff exists; no --shard; exits 0 even on survived mutants)
rustc dead_code(pub exempt)+instrument-coverage | VERIFIED | rust-lang/rust#120079 OPEN + rustc book | CONFIRMED (pub items exempt; -C instrument-coverage instruments unused fns)

BLOCKING ABSENCES

None that gate STOP_REPLAN. Detail:

1. .github/workflows/wiring-audit.yml is ABSENT in both .elai_cc/.github/workflows/ and repo-root .github/workflows/. The design-doc filename claim is REFUTED. The functional equivalent is LIVE as the 'Wiring contract audit (C-27 T-0)' step in .elai_cc/.github/workflows/windows.yml:112-119 (no continue-on-error, no || true, so nonzero exit FAILS the build). No implementation phase depends on the exact filename. Consequence: the gate is Windows-only and bypassable via '[ci skip]'. Re-scope item, not a blocker.

2. (Not an absence but a forward-dep) There is no real DKG/SymbolGraphCache-backed reachability graph. DkgReachabilityAuditor::is_reachable (wiring_contract.rs:581) is a genuine BFS, but the audit driver fabricates the graph (wiring_contract.rs:903-914) by adding a direct edge from every entrypoint to every declared produces_symbol -> every produces_symbol is trivially reachable, UnreachableSymbol can never fire from a real run, and there is zero test-vs-production caller classification (W-8/R7 moot, R5 'sound reference' unmet). Any phase relying on sound reachability or test-caller exclusion must FIRST wire elai_knowledge::symbol_graph::SymbolGraphCache into the graph builder. This is BUILD-NEW work, surfaced now so it is not discovered mid-implementation.

PER-PHASE DELTA (exists vs build/change; fail-open -> fail-closed edits)

Phase 1 -- Layer 1+2 (compile-time static + cross-language wiring contract)
EXISTS: all four Phase-1.2 enums already lack #[non_exhaustive] and are already exhaustively matched at production as_str/as_sql_str sites (SsbActionType ssb.rs:4650/as_sql_str 8364, FsmSubstate fsm.rs:125/as_str 175, RouteDecisionEventKind ssb.rs:24375/as_str 24393, SkillEventPhase ssb.rs:21235/as_str 21244). action_type_lock.golden is a live hard-gate (panics on removal/rename, fail-closed on missing). cross_lang_diff.py (14 pairs), cross-lang-pairs.toml (6 rows), PairKind (6 variants) all exist. elai drift CLI is the sound Rust leg.
CHANGE/REFUTED-PREMISE: the design-doc '1.2 strip non_exhaustive' is a NO-OP -- there is nothing to strip on any of the four. RE-SCOPE 1.2 to (i) a compile-time exhaustiveness regression test locking the no-wildcard invariant, or (ii) auditing other dispatch enums.
FAIL-OPEN -> FAIL-CLOSED: (a) WiringAuditError has no Empty/ZeroContracts variant (wiring_contract.rs:1038-1046) -> a present-but-empty contracts dir returns exit 0; ADD a ZeroContracts error/violation so it FAILS (R1 failOnEmptyShould). (b) cross_lang_diff.py action-type leg is OPT-IN via --check-action-types (R3 violation) and unwired into any gate -> make it run unconditionally and wire it into CI. (c) reconcile the three independent pair lists (py / toml / PairKind) so they cannot drift.
PATCH: e45_cross_language_drift_contract.rs:294 asserts TOTAL_VARIANTS==394 vs live 482 -> stale/broken contract assertion, fix it. CLAUDE.md 191/464 is stale vs live SCHEMA 205 / 482 variants.

Phase 2 -- Layer 3 entrypoint coverage (instrumented-binary, NEW-fn count==0 = BLOCK)
EXISTS: the `elai wiring audit` CLI chain is real and invokes all three auditors (args.rs:362-366 -> main.rs:775-782 -> wiring_audit.rs:59-108 -> WiringAuditCmd::run wiring_contract.rs:827-959), exit 1 on !is_clean. External coverage tooling verified current: cargo-llvm-cov v0.8.7 (run subcmd + --fail-under-lines confirmed), rustc -C instrument-coverage (instruments unused fns -> count==0 valid signal), rustc -D dead_code (confirmed blind to unused pub items per OPEN #120079).
BUILD NEW: (1) the actual Layer-3 coverage runner -- build instrumented binary, drive under production-default env, parse JSON, BLOCK any new function with count==0. The current driver checks declared contracts, not binary coverage; this runner does not exist. (2) smoke-manifest.toml enforcement -- file exists+frozen but has ZERO active [[feature]] entries; require and populate per-feature entries. (3) knip --production wrapper that fails on a zero-module analysis ('failure-on-zero-modules' is not native to knip). (4) a Linux/cross-platform runner leg or a dedicated wiring-audit.yml so the gate is not Windows-only.
TOOL FIX: cargo-llvm-cov JSON flag is --json (with --output-path), NOT --format=json -- correct any checker that hard-codes the wrong spelling.

Phase 3 -- Layer 4 runtime trace assertion (drive real entrypoint, assert off-agent SSB row; db-not-found/zero-rows = FAIL)
EXISTS (substrate is real and grounded): Ssb single-writer WAL append-only writer (ssb.rs:14145); Ssb::write_payload/write -> Result<i64,SsbError> (ssb.rs:14265/14249) returns durable seq before the agent unblocks (WARN-6) -- correct Layer-4 assertion target; CWD-rooted db path (ssb.rs:14758, identical to core/paths.rs:161); contract ObservableEffect decl (wiring_contract.rs:141-149) and ObservableEffectRunner (671-798) with 1s poll; tracing-test v0.2.6 confirmed (logs_contain -> bool, does NOT auto-fail).
FAIL-OPEN -> FAIL-CLOSED: (1) the DRIVER drops the runner Err at wiring_contract.rs:945 (`let _ = runner.run_effect(...)`). The violation still increments report.violation_count, but verify WiringAuditReport::is_clean()/exit-code (line 826) actually surfaces a BrokenObservable as a nonzero exit; harden any path that lets it slip. (2) ssb_has_row treats a missing DB as keep-polling -> make 'db never appeared by deadline' an explicit FAIL, not just poll-exhaustion. (3) do not rely on ReflectEventSink::emit (ssb.rs:14784) as sole evidence emitter -- it is fire-and-forget and logs+drops write failures (fail-open bridge).
BUILD NEW: an AUDITOR-authored trace-assertion harness (per the wiring role boundary -- not written by the feature agent) that drives the real entrypoint and asserts the SSB row in-window, using SimpleSpanProcessor only (BatchSpanProcessor banned in test setup). Auditor must verify every #[traced_test] body wraps logs_contain in assert! (un-asserted = silent pass).

Phase 4 -- Autonomous loop (4G evidence gate) + freeze (4H self-mod locks)
EXISTS / CENTRAL FAIL-OPEN: the EXECUTING->AUDITING gate keys on a unit-test proxy in two reinforcing places -- TestFlippedGate::check (evidence.rs:458, any flipped_to_pass) and AutoAdvanceEvaluator's AutoAdvance check deferring to Evidence::all_passed() which ANDs Evidence::test_flipped (autonomous_gate.rs:355). derive_autonomous_evidence (task_runner.rs:13499) makes test_flipped vacuously TRUE on zero test rows. TestOutcome rows are in-boundary (emit_test_outcome task_runner.rs:12483; producer==grader, R4).
FAIL-OPEN -> FAIL-CLOSED: (1) flip the absent/empty-data path on the EXECUTING->AUDITING axis from PASS to FAIL (the autonomous derive path task_runner.rs:13499 is the vacuous-true hole). (2) require an off-agent artifact on this transition analogous to AntiFabricationGate's blake3 stdout_hash (evidence.rs:766-786) -- which today only guards AUDITING->MEMORIZING and itself fail-opens on absent rows (flip its absent-rows `Ok(())` to Err). (3) harden DiagnosticProbeGate (evidence.rs:696-706) None/TimedOut/ToolMissing default-PASS to necessary-not-sufficient or FAIL. (4) REGISTER the new requirement as a gate -- check_transition (evidence.rs:847) returns Ok for an empty gate set, so an unregistered requirement silently passes.
FREEZE (4H) -- machinery real: LockedKeysAuditor (fail-closed on unreadable registry, locked_keys_auditor.rs), git pre-commit hook (.git/hooks/pre-commit drives elai audit-locked-keys --staged), L3 human-only StructuralChange tier (self_mod_types.rs:417), lock-list self-protection (self_mod_safety.rs:234). EXTENSION POINT: scripts/locked_keys_registry.toml accepts a new [[locked_keys]] row as a pure data append (no auditor code change). TWO CAVEATS the plan must handle: (a) registry+hook scoped to the INNER .elai_cc/ git repo, but the .claude harness wiring-gate files (gate-manifest.txt, checker, smoke harness, allowlist) live in the OUTER F:/.ELAI_workflow/ repo -> a parallel registry+hook at the outer root OR relocating gate files under .elai_cc/ is required for real freeze coverage (BUILD NEW). (b) an empty registry is fail-open by bootstrap policy -> live entries must be populated, not assumed.

RESIDUAL RISKS (carried from the design doc, confirmed by this sweep)
1. No sound static reachability for Rust + std + tokio + dyn Trait (no Go-style deadcode). Static legs (Layer 1/2) are approximations: false positives on trait-object/IPC-string/event-bus dispatch, false negatives on dynamic paths. The runtime leg (Layer 3/4) is the only sound backstop and is load-bearing, not optional. Confirmed by rustc #120079 (pub items exempt from dead_code) -- the static dead_code lint cannot be the sole reachability gate.
2. The smoke harness only proves what it exercises; smoke-manifest.toml is currently empty so it covers nothing. Auditor owns the manifest and must measure manifest coverage itself.
3. Wire-dark has no external benchmark. Instrument the local wire-dark rate (wire-dark plans / total) so the redesign is judged on measured deltas.
4. tracing-test logs_contain does NOT auto-fail; an un-asserted call silently passes (fail-open hazard, Layer 4). cargo-mutants exits 0 even when mutants survive (parse stdout, not exit code). cargo-llvm-cov JSON flag is --json not --format=json. knip does not self-fail on empty analysis (needs a wrapper). cargo-machete has a typo-squat fork cargo-machete-nk -- pin bnjbvr/cargo-machete.
5. The DkgReachabilityAuditor graph is fabricated (entrypoint->every produces_symbol); reachability self-certifies. Until SymbolGraphCache is wired in, the producer-without-consumer check (R6) is structurally inert even when [wiring.entrypoints] is populated.
6. Two of three auditor legs are conditionally fail-open with the live default config: [wiring.entrypoints] is commented out (elai_cc.toml:106-110) so the reachability leg is skipped-with-warn; only StubBan + ObservableEffect actually gate today, and the latter has its driver Err dropped.