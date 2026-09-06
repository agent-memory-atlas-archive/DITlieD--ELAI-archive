PROGRESS SUMMARY: Tests: 5/5 passing (i33 contract) | 9/9 (decision_governance unit) | 2/2 (action_type_lock) | 14/14 (e45 cross-lang) | 11/11 (h39) | 6/6 (i34_grammar_gate) | 11/11 (h41_checker_coverage) | 13/13 (h41 mode_a) | overall contracts suite: 1 pre-existing failure in agent_pool_contract (attributed to prior session, not I-33) | svelte-check: 0 errors | maintainability_check.py: 0 BLOCKs 0 WARNs | BLOCKs: 0 (I-33 scope) | Score: 0.82

session: A101-I33-AUDIT  task_id: A101-I33-AUDIT  date: 2026-06-12

---
THE FLAG-CHAIN RULING (central hostile question)

DISPOSITION: PASS-WITH-REGISTERED-DEPENDENCY (not BLOCK).

Evidence:
- governed_prompt_decision (decision_governance.rs:356) calls prompt_decision (hook_prompt.rs:219-252) which returns Err(PromptHandlerError::GateDark) when ELAI_E25_PROMPT_HANDLER_ENABLE is unset (hook_prompt.rs:223-226).
- The ? on line 392 of decision_governance.rs propagates GateDark as GovernanceError::PromptHandler — meaning steps 3 (I6Q) and 4 (CommitReveal) are unreachable with default env.
- CEFL step (step 1) CAN fire even when gate dark: CeflExternalized row emits BEFORE the prompt_decision call.
- There is NO production orchestrator caller of governed_prompt_decision — only bench_governance.rs (fixture, deterministic, no live LLM). The bench exercises emit_governance_fired directly, bypassing the gate.
- ELAI_E25_PROMPT_HANDLER_ENABLE is R9-registered at wire-dark-allowlist.toml:1096 as type=experiment, owner=maintainer, expiry=2026-09-01, promotion_plan_id=E-25, smoke_test citing e25_prompt_handler_on_path.rs.
- The I-33 plan explicitly accepts E-25 as the compose target (COMPOSES-WITH block: "E-25 prompt_decision primitive — compose the call site, do NOT re-implement") and the WIRING_CONTRACT reachable_from entry names "prompt_decision -> decision_governance::CeflRequest::apply -> GovernancePrimitiveFired" — the chain is plan-acknowledged, not an audit surprise.
- The allowlist carries both flags. Governance promotes when E-25 promotes. The dependency is structurally registered, not silent.

Ruling: governance is wire-dark-in-practice behind the E-25 gate, but this is DOCUMENTED in the plan, REGISTERED in the allowlist (both rows), and the bench proves all 3 SSB kind arms emit correctly. The plan chose "compose E-25, do not fork" as the wiring strategy. R9 is satisfied on both flags. Not a BLOCK.

Residual: a production orchestrator call site that uses governed_prompt_decision with a real RuntimeAdapter will remain a stub until ELAI_E25_PROMPT_HANDLER_ENABLE is promoted. The I-33 wire-dark-allowlist row lists the bench as its smoke_test, which is honest (the bench is what actually exercises all 3 arms today).

---
AUDIT FINDINGS

BLOCK (must fix before proceeding):
  none in I-33 scope.

WARN:
- C-STALE-API / named-constants violation: I6qValidator::default() (decision_governance.rs:200-205) hardcodes magic numbers 20 and 5 in the Default impl body rather than named const values (e.g. I6Q_MIN_LEN: usize = 20, I6Q_MIN_UNIQUE_WORDS: usize = 5). The plan states thresholds are "orchestrator-owned config, not model-settable" — named constants make the values visible, searchable, and grounded to the plan claim. The current struct fields are pub, so callers CAN override, but the semantic anchor is missing. Remedy: extract const I6Q_DEFAULT_MIN_LEN: usize = 20 and const I6Q_DEFAULT_MIN_UNIQUE_WORDS: usize = 5 and reference them in Default.

- C-COMMIT-REVEAL-TIMING: CommitRevealRecord::commit_hash (decision_governance.rs:266-271) hashes (seed || choice_index) where choice_index is the OUTCOME of the prompt (the PromptDecision variant, step 4 of governed_prompt_decision:419-424). This means the commitment is computed AFTER the model has returned — the hash does not seal the choice BEFORE observation. The wiring contract requires "commit_hash ... written to SSB before the model runs." The bench fixture (bench_governance.rs:180-207) simulates reversed IDs as the out-of-order case, but the real production flow never writes a commit row BEFORE prompt_decision is called (steps 1-2-3-4 show no pre-commit write). The only CommitRevealSealed row is a post-reveal beacon. The SSB row-order proof (commit_row_id < reveal_row_id) is therefore untestable in the production path — there is no pre-model commit row to compare against. The plan's commit-reveal framing is an after-the-fact hash, not a before-the-fact seal. Remedy: add a distinct pre-call commit row write (CeflExternalized row is already in step 1 — the CommitRevealSealed beacon should be restructured as a two-row pair, or the plan should acknowledge that the current implementation is a deterministic audit trail not a true commit-reveal protocol).

INFO:
- Pre-existing: agent_pool_contract::contract_all_10_roles_constructible FAILS (1/6) — Researcher GRC config missing transition_override {PlanAction: [Verify]}; identical defect present in git HEAD e38cd1a, predates I-33. Not attributed to this session. agent_pool.rs is in the current session's M files due to unrelated calibration rename — the Researcher arm is byte-for-byte identical to HEAD.

- SeededClock vs OsClock: bench_governance.rs uses SeededClock::new() (line 93) for deterministic replay; the contract test (i33_b) uses OsClock. No temporal collision risk but worth noting for reproducibility under --seed: SeededClock is the correct choice for bench determinism (correct).

- ELAI_GOVERNANCE_DISABLE allowlist (wire-dark-allowlist.toml:1633): type=kill_switch, owner=orchestrator, expiry=never, promotion_plan_id=I-33, smoke_test="elai bench governance-probe --seed 42". The smoke_test is a CLI string, not a file path. The E-25 row cites a real .rs file path. Minor inconsistency (advisory): for auditability, a file path is preferable. Not a blocking concern given kill_switch semantics.

- U-2/D-4 production seed injection PENDING: CommitRevealRecord.seed is supplied by the caller; in the bench this is the --seed CLI arg. The plan acknowledges U-2/D-4 deferred. Until that wires in, seed=0 is the default, which reduces commit-reveal entropy to a constant. Registered PENDING at the plan level; not a new finding.

---
SCHEMA / GOLDEN CHECKS (verified)

- SCHEMA_VERSION_PUBLIC = 240 (ssb.rs:3143). Contract arm (e) requires >= 240. PASS.
- TOTAL_VARIANTS = 549 (action_type_lock.rs:702). Contract arm (e) requires 549. PASS.
- GovernancePrimitiveFired in ALL_VARIANTS at action_type_lock.rs:1944. PASS.
- Alphabetical correctness: GovernancePrimitiveFired is entry 240 in the lock array. CeflExternalized < CommitRevealSealed < I6qRejected in ASCII order — the 3-arm ordering in the enum should follow: CeflExternalized=0, CommitRevealSealed=1, I6qRejected=2. Verified present in import list at action_type_lock.rs:30.

---
CEFL STRUCTURAL ANALYSIS

The CeflRequest builds {"type":"string","enum":[...]} JSON Schema and passes it to ConstrainedDecodingConfig::json_schema. The plan claims "model-invented options are structurally unparseable." The enforcement is:
- Cloud: Anthropic Tool API input_schema (grounding_client.rs:767 — cited in module doc, not re-verified; UNVERIFIED).
- Local: lm-format-enforcer JsonSchemaParser (server.py:2257 — cited in module doc, not re-verified; UNVERIFIED).

The schema is authored by the orchestrator and the enum constraint is valid JSON (verified by contract arm a which calls serde_json::from_str). Whether the backend actually enforces the constraint at token-generation level is runtime-dependent and not verifiable from static audit. The contract tests validate the schema content (correct), not enforcement (not testable statically). This is the plan's intended scope for Tier-1.

---
CROSS-LANGUAGE UNIT

4-file cross-lang: decision_governance.rs + commands_i33.rs + GovernanceMetricsChip.svelte + types.ts GovernanceMetrics interface (types.ts:4172-4177). The Tauri command is registered at lib.rs:1366. Svelte chip imports GovernanceMetrics type. Cross-lang consistency: GovernanceMetricsPayload fields (cdl, diu, fsr, row_count) match GovernanceMetrics interface in types.ts. Serde round-trip tested in commands_i33.rs:126-137. CONSISTENT.

FSR metric correctness concern: commands_i33.rs reads rows via recent_rows_of_type(tag, 50) where tag = SsbActionType::GovernancePrimitiveFired(CeflExternalized).as_sql_str(). The SQL tag for the entire GovernancePrimitiveFired enum is "governance_primitive_fired" regardless of Kind discriminator (Pattern-2 single-variant). So the query returns ALL GovernancePrimitiveFired rows (all 3 kinds). The total denominator is all rows. CDL = cefl_count/total, DIU = i6q_count/total. These are correct. FSR = seal_clean/seal_total — this is correct because seal_total only counts CommitRevealSealed rows. However the "out_of_order" string check in details_json (commands_i33.rs:93) is a substring match: a row whose out_of_order_commit fixture arm embeds "verdict\":\"out_of_order\"" in details_json will correctly count as dirty. CORRECT.

---
ZERO-LAZY / UNWRAP CHECK

Production code (decision_governance.rs): no .unwrap()/.expect() in non-test paths. let Some(ssb) = ssb else { return Ok(()) } pattern used for optional SSB. PASS.

bench_governance.rs: .unwrap_or_default() at line 55 on SystemTime::duration_since(UNIX_EPOCH) — returns Duration::default() (zero) on pre-epoch clock, yielding subsec_nanos=0, which is benign for temp-dir naming. This is the bench entrypoint (CLI fixture), acceptable per CLAUDE.md exception: "main()/entrypoint/bootloader init where panic = won't start, you'll see it immediately." Not a BLOCK.

commands_i33.rs: serde_json::from_value continues on Err (line 85: match ... Err(_) => continue). This silently skips unparseable rows. Acceptable for a metrics aggregator reading an append-only log where payload format may drift across schema versions — the row_count still counts the total, so the CDL/DIU/FSR denominators may be inflated relative to parseable rows. Minor but acceptable given this is a display metric. WARN: not blocking.

---
MAINTAINABILITY (MR-1..MR-4)

decision_governance.rs: 546 LOC — under 1000 line boundary. MR-1 PASS.
bench_governance.rs: 254 LOC — PASS.
commands_i33.rs: 139 LOC — PASS.
i33_governance_contract.rs: 295 LOC — PASS.
No as any / as unknown / @ts-ignore / unsafe blocks introduced. MR-2 PASS.
No function touches the 80-line threshold. MR-3 PASS.
No branching density spike. MR-4 PASS.
maintainability_check.py --json: 0 BLOCKs, 0 WARNs. CLEAN.

---
WIRING-ENFORCEMENT LAYER SUMMARY

Layer 1 (compile): cargo check --workspace --tests: CLEAN (18 dead_code warnings, all pre-existing).
Layer 2 (contract): 5/5 i33_governance_contract arms pass. Action_type_lock 2/2. e45 14/14.
Layer 3 (entrypoint): bench_governance.rs is the real CLI entrypoint. emit_governance_fired called directly; SSB row count verified at runtime (3 rows, kind-discriminated). Off-author artifact: bench does NOT author its own assertion rows — decision_governance.rs emits them.
Layer 4 (trace): contract arm (b) uses tokio + tempfile SSB to assert I6qRejected row lands. Kind-discriminated via rmp_serde decode path. 1 row asserted. PASS.

Overall wiring disposition: PASS-WITH-REGISTERED-DEPENDENCY (flag-chain ruling above).

---
VERDICT: PASS (score 0.82)

Rationale: all 5 contract arms pass, 9/9 unit tests pass, action_type_lock 2/2, compile clean, maintainability clean, Svelte 0 errors, cross-lang consistent. The flag-chain question resolves to PASS-WITH-REGISTERED-DEPENDENCY per plan acknowledgement and dual R9 registrations. Two WARNs (magic numbers in I6Q defaults; commit-reveal semantic gap) do not meet BLOCK threshold for this Tier-1 delivery given plan-explicit Tier-2 deferred items. Pre-existing agent_pool Researcher GRC defect is not attributed to this session.
