# I-19: tdd enforcer preflight

- Track 15 — DETERMINISTIC GATES + ANTI-GAMING VERIFICATION | lever CANT-WRECK | grounding STRONG | nature EXTENSION | effort S-M | WAVE 3 | IC-12
- Premise: TESTING_FIRST FSM gating (the state machine refuses execution before tests exist) and E-13 anti-over-mock
  already ship. F-1 VibeGate already sits in the PreToolUse compound hook path. what is absent is the SYMBOL-LEVEL
  leg: an Edit that introduces a new pub fn with no corresponding test-symbol delta passes today without protest.
  i-19 closes only that gap — src-edit-requires-test-delta, with a syntax-only dry-run and a coverage-delta exemption
  for pure refactors. FP-bounded before default-ON (GD-4); the FP rate is the gate.

## Steal/build
SYMBOL-LEVEL PRECOMPILE CHECK via tree-sitter AST: when a PreToolUse fires on a Rust/Python/TS source file in
EXECUTING state, parse the pending diff, extract modified or added pub fn/fn/def/class symbols, and require that the
test file set contains at least one symbol whose name shares a substring with the modified symbol (or is listed in a
test-attribution map). this is the MINIMUM viable legibility check — sufficient to block "add fn X with no test at
all" without requiring full coverage instrumentation.
SYNTAX-ONLY DRY-RUN: before the write lands, run cargo check --tests (Rust) or pytest --collect-only (Python) on
the staged content. this is a COMPILE-ONLY check — no test execution (scoped-tests rule: auditor runs tests). the
dry-run catches import/type errors that would fail the test even if symbolically matched.
REFACTOR EXEMPTION via coverage-delta: a pure rename/move/reformat (no new public surface) is exempt. the check is:
if the diff introduces NO new pub symbol, the symbol-level gate is skipped. a coverage-delta cross-check (line
delta vs name delta) disambiguates structural refactors from new-API additions.
FP BOUND BEFORE DEFAULT-ON: this gate generates friction. GD-4 mandates measuring the false-positive rate on >=3
real orchestrate runs (runs where the gate fires on a CORRECT edit). the gate stays default-OFF (registered in
wire-dark-allowlist.toml with type=experiment, expiry_date, promotion_plan_id=i-19) until the FP rate is recorded
and bounded. the allowlist row ships WITH the gate (R9 compliance).
IC-12 is the binding correction: Consolidated i-19 implied the gate was net-new; the corrected scope is this
symbol-level PreToolUse leg only, composing the existing TESTING_FIRST + E-13 + F-1 seam.
EXCLUDED: full coverage instrumentation in the pre-flight (too slow for per-edit friction); forced test generation
(not a gate, a generator concern); any design that blocks a refactor Edit.

## Existing ground truth
- TESTING_FIRST FSM state — SHIPPED; i-19 adds an axis to the compound hook, never replaces the FSM gate
- E-13 anti-over-mock — SHIPPED; i-19 composes (F-1 seam already has E-13 as an axis — GD-1)
- F-1 VibeGate compound evaluator seam (hooks/run_pre_tool_use) — SHIPPED; i-19 adds one more predicate, does not fork the hook
- Phase 4C hook tier structure — SHIPPED; the PreToolUse compound is the right tier (PLAN-asserted; executor re-greps hook_dispatcher or equivalent at EXECUTING, W-7)
- knowledge tree-sitter grammars — SHIPPED (IS-17 workspace crates include knowledge crate); i-19 uses the Rust/Python/TS grammars for diff parsing
- IC-12 (binding correction): TESTING_FIRST FSM gating + E-13 already exist; i-19 scope = symbol-level leg only
- GD-4: gate default-ON requires FP rate bounded on >=3 real runs; wire-dark-allowlist row required

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- tree-sitter diff parser in the PreToolUse hook (extracts added/modified pub symbols from the pending edit)
- symbol-name cross-reference check against test files (substring match + optional attribution map)
- cargo check --tests / pytest --collect-only dry-run on staged content before the write lands
- refactor exemption (no new pub symbol -> gate skipped)
- wire-dark-allowlist.toml row for the gate (type=experiment, expiry + promotion plan)
- TddPreflightBlocked SSB action_type (Kind: MissingTestDelta / SyntaxDryRunFailed / RefactorExempted)
- FP measurement harness: 3-run fixture set that triggers the gate on correct edits; FP rate recorded to a TOML
DEFER (Tier-2 / gated):
- default-ON flip: happens only after FP rate is recorded and bounded (GD-4); tracked as a separate patch-N
- full coverage-delta instrumentation (line-level coverage diff): deferred; the symbol-name check is Tier-1

## Integration
modify: the run_pre_tool_use compound hook (F-1 seam) — add TddPreflight as a new predicate after existing axes.
new: crates/orchestrator/src/tdd_preflight.rs (the predicate: tree-sitter parse + symbol check + dry-run invocation).
modify: wire-dark-allowlist.toml (new row for ELAI_TDD_PREFLIGHT_ENABLE with type=experiment).
SSB action_type: +TddPreflightBlocked (Kind: MissingTestDelta / SyntaxDryRunFailed / RefactorExempted) — baseline+1.
frontend_pair: block reasons surfaced in the existing gate panel (the F-1 VibeGate panel already shows block reasons;
i-19 adds a row to the same panel for TDD block events). no new drawer. cross_lang_diff PASS required (W-4).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/tdd_preflight.rs::TddPreflight (the predicate struct + check(&Edit) -> PreflightResult)
  - TddPreflightBlocked SSB action_type with MissingTestDelta / SyntaxDryRunFailed / RefactorExempted discriminators
  - wire-dark-allowlist.toml row for ELAI_TDD_PREFLIGHT_ENABLE
consumes_symbols:
  - hooks/run_pre_tool_use (F-1 compound seam — why: TddPreflight is added as a new predicate; GD-1 do NOT fork the hook)
  - E-13 anti-over-mock surface (why: existing axis in the compound; TddPreflight must not conflict; GD-1)
  - knowledge crate tree-sitter grammar bindings (why: diff parsing; GD-1 do NOT re-bundle grammars)
  - cargo check --tests / pytest --collect-only (external toolchain calls, deterministic, scoped to staged content)
reachable_from:
  - the PreToolUse hook firing on any Edit to a .rs/.py/.ts source file during EXECUTING FSM state  [PRODUCTION hook dispatch]
  (contracts/ callers do not satisfy R7; the production hook dispatch is the reachable_from chain)
stub_ban:
  - NO full test execution in the pre-flight (cargo test is BANNED here — auditor only; dry-run is compile-only)
  - NO default-ON flip without the GD-4 FP bound recorded on >=3 real runs (the allowlist row enforces the gate is OFF until then)
  - NO blocking a pure-rename refactor (the exemption is mandatory, not optional)
  - NO re-implementing tree-sitter grammar loading (use the knowledge crate's existing grammar bindings — GD-1)
  - NO .unwrap()/.expect() on production paths; NO fabricated FP measurements
frontend_pair:
  block reasons added to the existing gate panel (F-1 VibeGate panel). the same Tauri IPC surface that delivers F-1
  block reasons delivers TDD block reasons as an additional entry. no new drawer, no new IPC endpoint. the
  TddPreflightBlocked SSB row is the off-agent artifact. cross_lang_diff PASS required (W-4). Justified.
observable_effect:
  trigger: "elai run (EXECUTING state, with ELAI_TDD_PREFLIGHT_ENABLE=1 set for the FP-measurement run)"
  expect_ssb: "TddPreflightBlocked(MissingTestDelta) row when an Edit adds a new pub fn with no test-symbol delta. TddPreflightBlocked(SyntaxDryRunFailed) when the staged content fails cargo check --tests. TddPreflightBlocked(RefactorExempted) when a rename-only diff passes through (no new pub symbol)."
  expect_artifact: "the Edit with a matching failing test (a test file with a symbol name substring-matching the new fn) LANDS without a block row (positive arm). the FP harness 3-run report writes metrics/tdd_preflight_fp.toml with measured FP rate before any default-ON promotion. a --skip-preflight broken arm bypasses the gate and FAILS the contract test (R2/GD-4)."

## Grounding
- IC-12 (binding correction): TESTING_FIRST + E-13 already exist; i-19 = symbol-level leg only; the summary and map both confirm this is EXTENSION not BUILD
- GD-4: FP/over-block rate bounded before default-ON — the mandatory posture for a friction-generating gate
- F-1 VibeGate seam (PLAN-asserted file:line; executor re-greps run_pre_tool_use or hook_dispatcher at EXECUTING, W-7)
- Phase 4C hook tier structure (PLAN-asserted; re-grep at EXECUTING)
- IS-17: knowledge crate exists in workspace — tree-sitter grammar bindings confirmed at workspace level
- Consolidated Part IV + review 1.1 "symbol-level TDD gate as EXTENSION of shipped TESTING_FIRST"
- SSB: +TddPreflightBlocked (Kind: MissingTestDelta/SyntaxDryRunFailed/RefactorExempted) — baseline+1 from LIVE 221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9).

## COMPOSES-WITH
F-1 VibeGate compound evaluator (EXTENDS the hook with one predicate — do NOT fork it); E-13 anti-over-mock (the
compound already carries it — do NOT interact with its logic, avoid conflicts); TESTING_FIRST FSM gate (prerequisite
state — i-19 adds within EXECUTING, does not touch the FSM transitions); knowledge tree-sitter grammars (uses the
existing bindings — do NOT re-bundle or recompile grammars). the FP measurement harness is net-new.

End of I-19.
