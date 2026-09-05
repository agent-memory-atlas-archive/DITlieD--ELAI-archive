H-31 decisions -- user-decision bi-temporal ledger ACTIVATION
Session: s-8404
Date: 2026-06-06
FSM: EXECUTING

WHAT WAS BUILT
Activated the wire-dark crates/memory bi-temporal user-memory substrate into the LIVE path. Three
production wires + the off-agent SSB beacon + the Personalization-Trap firewall, all default-OFF
behind ELAI_USER_MEMORY_ENABLE (R9):
  (a) CAPTURE: `/remember <text>` REPL command + `elai remember "<text>"` CLI subcommand ->
      crates/cli/src/user_memory_capture.rs::capture_remember. Runs the secret/PII reject FIRST
      (elai_memory::classify, defense.rs -- both Block and SkipSensitive => NOT stored), builds a
      Preference ExtractedFact, calls elai_memory::dispatch_one (its FIRST production caller; ADD or
      UPDATE-via-supersede), reads the DispatchOp, emits the UserDecisionSuperseded SSB row.
  (b) RETRIEVAL-INTO-PROMPT: report_fns::build_system_prompt now threads AgentRole; for an
      allowlisted role + flag-ON it appends a MEMORY_SECTION_TAG section from get_relevant_facts
      (the list_current valid_to_ms IS NULL hard-filter -- a superseded row is structurally absent,
      never narrated [OUTDATED]). Registered ContextSlot::Memory (SLOT_COUNT 8->9) as the budgeted
      slot.
  (c) SSB BEACON: +1 Pattern-2 action_type UserDecisionSuperseded (inner 3-arm UserDecisionEventKind
      Captured/Superseded/ProposalSurfaced). SCHEMA_VERSION 239->240, TOTAL_VARIANTS 516->517.

KEY DECISIONS

DECISION-1 (firewall = compile-time allowlist over BOTH enums, fail-CLOSED). The #1 load-bearing
safety constraint. crates/orchestrator/src/user_memory_firewall.rs exposes agent_role_gets_user_memory
(AgentRole) + consensus_role_gets_user_memory (ConsensusRole), each a CLOSED exhaustive match with NO
wildcard. ALLOW = {AgentRole::Executor} and {ConsensusRole::Worker} ONLY. A new role variant added
later FAILS TO COMPILE until an explicit allow/deny decision is made (leak-by-omission is structurally
impossible). The retrieval site checks the firewall FIRST -- a denied role returns None before
get_relevant_facts is ever called, so no user-memory can reach a reviewer/auditor/critic/adversary
prompt. Chose allowlist over deny-list because a default-inject-with-deny-list silently leaks to any
role a future contributor forgets to deny.

DECISION-2 (CITATION DISCIPLINE -- the grounding pass caught TWO bad cites). The firewall is framed
HONESTLY:
  - DO cite arXiv:2603.00024 (Kelley & Riedl, "Personalization Increases Affective Alignment but Has
    Role-Dependent Effects on Epistemic Independence in LLMs") for the role-dependent mechanism:
    personalization has role-dependent effects on epistemic independence (it DECREASES epistemic
    independence in a social-peer role). The adversarial-reviewer/auditor user-memory exclusion is
    CONSERVATIVE DEFENSIVE ENGINEERING consistent with that mechanism -- the auditor role itself was
    NOT directly tested.
  - DO NOT cite arXiv:2510.09905 as judge-exclusion support. It is REAL ("The Personalization Trap:
    How User Memory Alters Emotional Reasoning in LLMs", Fang et al.) but WRONG DOMAIN -- it studies
    emotional reasoning in a RESPONDENT role, not a judge/auditor evaluating external content. It does
    not measure judge-objectivity loss; the mismatch would fail a citation-check.
  - DO NOT cite the "+68% token narration trap" number -- it is FABRICATED (not in Zep arXiv:2501.13956).
  - DO cite Zep arXiv:2501.13956 ONLY for the real bi-temporal soft-close semantics (t_invalid set to
    t_valid of the superseding fact). No token-percentage is attached.
  Verification source: workflow/papers/personalization-trap-verify.md (this session's WebSearch-grounded
  verdict).

DECISION-3 (ContextSlot::Memory APPENDED, not inserted). Memory = 8 is appended AFTER OutputReserve = 7,
NOT inserted before it. Inserting before would renumber OutputReserve = 8 and break the hardcoded
`OutputReserve as u8 == 7` assert + the repr(u8) ordinal contract every `slot as usize` index relies
on. Appending keeps every existing ordinal stable; the 6 churn sites (SLOT_COUNT const, the enum
variant, the new() fixed array, the two slot_assembly_config exhaustive matches, the u8 ordinal test,
plus the context_budget_contract.rs compile-time SLOT_COUNT==9 assert) are all compile-LOUD if missed
(R5). The Memory slot is bounded small (k<=8 decisions, MICRO=300 / LARGE=500 tokens) so it cannot
crowd out the local ~25-32K window.

DECISION-4 (bi-temporal soft-close; DELETE BANNED). A contradicting `/remember` on the same key drives
dispatch_one's UPDATE path -> FactStore::supersede, which soft-closes the old row (valid_to_ms =
now_ms, superseded_by set) and inserts the new row (valid_from_ms = now_ms). The observable invariant
old.valid_to_ms == new.valid_from_ms holds because build_fact_from_candidate sets valid_from_ms =
ctx.now_ms. DELETE is never issued (it would break SSB replay). The superseded row stays readable via
list_including_closed but is structurally excluded from get_relevant_facts (list_current's
valid_to_ms IS NULL filter).

DECISION-5 (the SSB row carries NO value). UserDecisionSupersededPayload carries {kind, old_fact_id,
new_fact_id, fact_type, key, evidence_blake3} -- NEVER the fact value. No secrets reach the journal;
the secret/PII reject (defense.rs classify) already runs at the capture boundary, and even a clean
value is omitted from the off-agent row on principle.

DECISION-6 (live SCHEMA re-read at edit time). Confirmed live SCHEMA_VERSION=239 (ssb.rs:3271),
TOTAL_VARIANTS=516 (action_type_lock.rs:817), e45 literal "516" -- re-read immediately before editing
per the concurrent-session warning. Bumped to 240/517. No concurrent variant landed between read and
write.

GROUNDING SUMMARY
  - GROUNDED (code read this session): the full crates/memory substrate (store.rs supersede/list_current,
    mem0_dispatcher.rs dispatch_one/DispatchOp, retrieval.rs get_relevant_facts, defense.rs classify,
    extractor.rs ExtractedFact, fact.rs FactType), both role enums, the DriftInterlock SSB template,
    context_budget.rs ContextSlot churn surface, the CLI wiring templates (drift_check_cmd.rs,
    args.rs, commands.rs, repl_command.rs, live_cli.rs FactStore handle).
  - GROUNDED (verified): the citation verdict (2603.00024 honest framing, NOT 2510.09905-as-judge, NO
    +68%) from workflow/papers/personalization-trap-verify.md.
  - UNGROUNDED/UNVERIFIED (carried over, NOT cited as fact): arXiv:2605.06527 (future-dated staleness
    ref, the plan self-flagged it). arXiv:2504.19413 (Mem0) and arXiv:2505.07509 (HALO) are not
    ingested but corroborated by ELAI's own soft_close/decay code -- cited for design lineage only.

VERIFICATION
  cargo check -p elai-orchestrator: PASS (14 pre-existing warnings, none in H-31 files).
  cargo check -p elai-cli: PASS (1 pre-existing `empire` cfg warning, unrelated).
  cargo check -p elai-memory: PASS.
  cargo check -p elai_cc_contracts --tests: PASS (after the context_budget_contract SLOT_COUNT 8->9 fix).
  cargo check --workspace: PASS (no error, no missed SLOT_COUNT / action_type drift anywhere).
  Tests authored (auditor runs them): h31_user_memory_contract.rs (8 tests over a real Ssb + FactStore),
  user_memory_firewall.rs (5 unit tests, both enums), user_memory_capture.rs (6 unit tests).
  The action_type_lock golden was regenerated by hand (insert user_decision_superseded at ordinal 0493,
  renumber 0493..0515 -> 0494..0516, count 516->517); the auditor should confirm via the regen test
  `cargo test -p elai-orchestrator --test action_type_lock regen_golden_file -- --ignored`.

OPEN / DEFERRED
  - wire-dark-allowlist.toml + smoke-manifest.toml rows added by this executing run; they trip
    gate_freeze_check.py until the controller re-locks OUT-OF-BAND
    (`python .claude/tools/gate_freeze_check.py --lock`). Sanctioned flag registration (H-3/G-9 precedent).
  - The propose-on-repeated-correction (ProposalSurfaced) path: the SSB Kind + surface_proposal emitter
    are SHIPPED; the per-(key) repeated-correction COUNTER that triggers it is deferred (the live tick's
    chat_memory_turn_count is the precedent pattern; not wired this slice).
  - HIGH-STAKES-FLIP confirmation (H-37 InteractiveGate) for a flip that changes a model-routing /
    dependency decision: deferred (the plan scopes it; the gate seam exists).
  - Ranked retrieval (DKG+rerank) is H-32; H-31 ships the trust-floor + last_observed ordering.

FIXER ADDENDUM (s-8404 fixer, 2026-06-06) -- B-1 BLOCK + W-1/W-3 remediation

DECISION-7 (W-1 -- topic-key derivation so a live contradiction SUPERSEDES; the narrowest sound
version). The audit proved the live `/remember` could NEVER reach the Superseded arm: the shipped
extract_facts heuristic keys "i prefer X over Y" on the STANCE (prefers_X_over_Y), so the genuine
contradiction "i prefer Y over X" keys differently (prefers_Y_over_X) and dispatch_one (which
reconciles on (user_id, scope, fact_type, KEY)) always found no matching current row => ADD => two
Captured rows, never a supersede. FIX (in crates/cli/src/user_memory_capture.rs, NOT the shared
extractor.rs E-5 substrate): build_candidate now calls rekey_prefer_over_to_topic, which detects the
two-named-option shape EXACTLY (value["matched"]=="prefer_over" with string subject+alternative
fields the heuristic stamps) and rewrites the key to topic_{a}_vs_{b}, where a,b are the two option
slugs sorted lexicographically. So "tracing over log" and "log over tracing" both => one key
topic_log_vs_tracing, with a DIFFERENT value (raw text + which option is preferred flip), so the
second remember drives dispatch_one's UPDATE-via-supersede arm (same key, changed value, equal 0.85
confidence so NOT lower_confidence) => a Superseded SSB row + the bi-temporal soft-close.
APPROACH TAKEN = the narrowest sound version (the explicit-shape heuristic, option (a)+(b) hybrid of
the audit's REMEDY OPTIONS): re-key ONLY the unambiguous two-named-option "prefer X over Y" shape;
single-option "i prefer X" (prefers_X) and the free-form blake3 fallback (remembered_<hash>) keys are
left UNCHANGED. A topic extractor over arbitrary free-form text would be ambiguous and risk a false
supersede that drops a distinct decision, which is strictly worse than a miss, so it was NOT
implemented.
RESIDUAL (documented, honest): a single-option flip ("i prefer rust" then "i prefer go") still gets
distinct prefers_rust / prefers_go keys and does NOT supersede -- the two statements name no shared
topic, so collapsing them would be a guess. Free-form ("remember: ...") likewise stays per-text. The
live supersede now FIRES for the clear two-option contradiction case (the plan-headline behavior);
the residual is a conservative miss, never a false merge.

DECISION-8 (W-2 -- consensus_role_gets_user_memory is FORWARD-DEFENSIVE, not a live leak). The
firewall gates BOTH enums by design, but only AgentRole is LIVE-wired today: the only production
get_relevant_facts-into-prompt caller is report_fns::build_system_prompt -> build_system_prompt_for_
role (default Executor), which gates on agent_role_gets_user_memory. agent_pool.rs (the consensus /
adversary path) has ZERO user-memory references -- there is no ConsensusRole prompt-assembler yet, so
consensus_role_gets_user_memory gates nothing live and its only callers are tests. This is the
CORRECT fail-closed forward-guard: it fires the instant a consensus assembler is added (a new
ConsensusRole variant forces a compile error, leak-by-omission impossible). It is labeled
forward-defensive here, NOT presented as an active second live firewall leg. The live leak surface is
AgentRole via build_system_prompt, which IS gated.

FIXER VERIFICATION (B-1 / W-1 / W-3)
  B-1: crates/orchestrator/src/ssb.rs:~38328 -- escaped the two literal brace pairs in the H-31
    assert_eq! message ({{AgentRole::Executor, ConsensusRole::Worker}} and {{kind, old_fact_id ...}})
    so assert_eq!'s format-string arg no longer parses them as placeholders. The elai-orchestrator
    lib-test target now compiles.
  W-1: the topic-rekey above + a live-producer contract test
    (user_memory_capture::tests::live_producer_contradiction_supersedes_with_soft_close_invariant)
    that drives the REAL capture_remember producer (ELAI_USER_MEMORY_ENABLE=1, isolated tempdir CWD,
    stepping clock) with two contradicting statements and asserts the SECOND yields
    CaptureOutcome::Superseded with old.valid_to_ms == new.valid_from_ms against the live FactStore
    (old NOT deleted), plus a substrate-level topic-key contract test
    (h31_user_memory_contract::topic_keyed_contradiction_supersedes_not_double_adds).
  W-3: wire-dark-allowlist.toml ELAI_USER_MEMORY_ENABLE smoke_test repointed from the non-existent
    crates/contracts/.../superseded_fact_never_reaches_assembled_prompt to the REAL ON-path test
    crates/cli/src/user_memory_capture.rs::tests::live_producer_contradiction_supersedes_with_soft_
    close_invariant (the only test that flips the flag ON and exercises capture + supersede).
  CONSTRAINTS honored: no SCHEMA/TV bump (still 240/517); firewall logic untouched; no DELETE on
    supersede (soft-close only); no new dependency; no .unwrap()/.expect() in lib code; ClockProvider
    (SteppingClock in test) not SystemTime.
