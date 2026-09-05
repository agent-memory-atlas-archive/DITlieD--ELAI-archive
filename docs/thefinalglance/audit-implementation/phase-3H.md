PHASE 3H AUDIT — Session Handoff (CMV-derived DAG)
====================================================

Auditor: hostile implementation auditor
Spec: docs/thefinalglance/summary/phase-3-summary.md L788-893
Code: .elai_cc/crates/orchestrator/src/session_dag.rs (3090 lines)
Wiring: .elai_cc/crates/cli/src/main.rs L3027/3552/3624/4034-4187/4957-4990/5136-5159
Contracts: .elai_cc/crates/contracts/tests/session_dag_contract.rs

OVERALL VERDICT: PARTIAL — DATA LAYER COMPLETE / RUNTIME WIRING DEAD ON ARRIVAL

The SessionDag library implementation (3H.1 through 3H.6) is materially complete and well-tested at the unit/contract level. However, the integration into the CLI runtime is a hollow shell: stub_ssb_seq is hard-coded to None in every message handed to trim_context, the create_handoff path is never invoked from the REPL, recover_stubs is never called, branch merge / close are never wired, and the legacy JSONL session persistence (orchestrator/src/session.rs) still drives all on-disk session state. Phase 3H ships a working module that does not actually replace anything.

---

3H.1 SQLITE SCHEMA AND TYPES — VERDICT: PASS (with documented divergence)

Evidence:
- session_dag.rs:158-193 — DDL for snapshots, snapshot_context, indexes
- session_dag.rs:106-152 — Snapshot, TrimmedMessage, Stub, BranchRef, MergeResult
- session_dag.rs:91-100 — MergeResult::Winner / Conflict variants both present
- session_dag.rs:241-242 — PRAGMA foreign_keys=ON, journal_mode=DELETE
- session_dag.rs:308-312 — PRAGMA integrity_check on open -> DatabaseCorrupted
- Tests: schema_has_exactly_three_tables (L1416), schema_has_three_indexes (L1435), open_corrupted_returns_database_corrupted, open_nonexistent_returns_not_found

Notes:
- Implementation uses 3 tables (snapshots, snapshot_context, schema_version) — spec only describes 2; the schema_version table is undocumented in spec but harmless and forward-compatible (version=1 enforced at L301-306). Schema test asserts count==3 to account for it (L1431).
- snapshot_context follows SPEC AMENDMENT #1 (per-message rows with seq, role, content BLOB, is_stubbed, stub_ssb_seq) — not the alternate BLOB-per-snapshot variant. Matches the resolved amendment.
- ON DELETE CASCADE added to snapshot_context.snap_id FK (L181) — undocumented but defensive; acceptable since spec §3H.4 says snapshots are immutable so cascade should never trigger in normal operation.
- snap_id generated via UUID v4 from EntropyProvider (L381-387). Sets RFC 4122 version+variant bits correctly. Deterministic replay confirmed in tests via SeededRng.

3H.1 VERDICT: PASS

---

3H.2 SNAPSHOT SAVE AND LOAD — VERDICT: PASS

Evidence:
- save_snapshot (L394-459): single-transaction BEGIN IMMEDIATE / explicit ROLLBACK on error / COMMIT. Pre-validates all i64/JSON conversions BEFORE opening txn (L400-406) so a partial-data error cannot leave a half-written transaction.
- load_snapshot (L462-543): JOIN-equivalent — separate snapshot row read then ordered seq ASC scan of snapshot_context.
- latest_snapshot (L546-621): tag-filtered or any-tag, ORDER BY created_at DESC LIMIT 1.
- parent_chain (L624-691): visited HashSet (L629) + MAX_PARENT_DEPTH=1000 cap (L633) -> CycleDetected. Both belt-and-suspenders.
- Tests: save_load_snapshot_round_trip, snapshot_not_found, save_atomic_rollback_on_partial_write (L1651), parent_chain_returns_newest, parent_chain_walks_linear, parent_chain_cycle_detected (L1734), parent_chain_1000_messages_performance (L1769).

ADVERSARIAL CHECKS:
- Atomicity: explicit BEGIN IMMEDIATE + ROLLBACK on inner error and on COMMIT failure. PASS.
- Cycle detection: visited HashSet AND depth cap. PASS.
- seq ordering: SQL `ORDER BY seq ASC` (L512), not insertion order. PASS.

3H.2 VERDICT: PASS

---

3H.3 STRUCTURALLY LOSSLESS 3-PASS TRIMMING — VERDICT: LIBRARY PASS / RUNTIME DEAD

Evidence (library):
- trim_context (L707-833) implements 3 passes:
  - Pass 1 (L740-766): skip non-Tool, skip already-stubbed, skip seq <= PROMPT_CACHE_PREFIX_BOUNDARY (=3), skip if stub_ssb_seq.is_none(), then check token_counts[i] > stub_threshold.
  - Pass 2 (L768-797): blake3::hash on raw content (L778), build Stub, msgpack-encode via rmp_serde (L786), replace content.
  - Pass 3 (L799-814): for each candidate, if !valid_ssb_seqs.contains(&ssb_seq) restore original (UNSTUB).
  - Budget check (L817-830): sum tokens, return TrimBudgetExceeded if > target_tokens.
- Slot guard: PROMPT_CACHE_PREFIX_BOUNDARY = 3 (L19). User/Assistant guard at L749 (only Tool eligible), so user/assistant are NEVER stubbed regardless of seq.
- Range guard: stub_threshold validated [100, 1000] at L713-720.
- TokenCountFn (L200): pluggable; default heuristic_token_count = bytes/4 (L204), settable via set_token_counter (L275).
- Tests: trim_context_no_stubs_when_below_threshold, user_assistant_never_stubbed (L1851 / L1878), prompt_cache_prefix_never_stubbed, trim_context_validates_ssb_seq_in_set (L1949), unstubbed_when_ssb_missing (L1980), trim_context_deterministic_same_seed, trim_context_500_messages_performance, etc.
- blake3 grounding: blake3 0.5+ used directly (L778, L1198, L1259); CLAUDE.md mandates blake3 over blake2 — COMPLIANT. The spec flagged "blake3 hashing UNGROUNDED in spec" but the choice is consistent with project hard rule.

ADVERSARIAL CHECKS:
- Cache prefix: assertion via `msg.seq <= PROMPT_CACHE_PREFIX_BOUNDARY` (L756). PASS.
- Hashing: blake3 (32-byte digest stored on Stub.summary_hash). PASS.
- Min/max threshold: enforced. PASS.

CRITICAL RUNTIME FAILURE — TRIM IS DEAD CODE IN PRODUCTION:
- cli/main.rs:4044-4063 builds TrimmedMessage from session.messages with `stub_ssb_seq: None` hard-coded (L4061).
- cli/main.rs:4131-4153 (try_save_snapshot) does the same — `stub_ssb_seq: None` (L4151).
- session_dag.rs:759-762: Pass 1 SKIPS any message where stub_ssb_seq is None.
- Net effect: at runtime, trim_context will identify ZERO candidates, produce ZERO stubs, write ZERO archived content to SSB. The branch L4077 returns the input unchanged. The "[stubbed — content archived in SSB]" cosmetic replacement at L4082-4088 is unreachable.
- There is no code anywhere in the CLI/runtime that PRE-WRITES tool-result payloads to SSB and stamps the seq onto the in-memory message envelope. Until that pre-write loop exists, trim_context does nothing useful at runtime.

3H.3 VERDICT: LIBRARY PASS / RUNTIME COMPLETELY UNWIRED

---

3H.4 BRANCH AND MERGE OPERATIONS — VERDICT: PASS (library) / PARTIAL (runtime)

Evidence:
- create_branch (L847-939): validates name (alphanumeric + hyphens, max 64, L839-845, L854), rejects duplicates (L858), enforces MAX_ACTIVE_BRANCHES=3 (L862-866 with BranchLimitExceeded{max:3}), copies snapshot row + snapshot_context with `tag=branch:{name}` (L895-917), snapshot_context copy is in same txn.
- merge_branch (L945-1069): Conflict detection by overlapping code_change paths (L957-987) returns MergeResult::Conflict; otherwise creates NEW snapshot (immutable DAG) with `parent_id = winner_tip` and `tag = NULL` for the merged trunk row, then UPDATEs all branch tags from `branch:{name}` to `abandoned:{name}` (L1028-1040). Active branches drained.
- close_branch (L1071-1107): UPDATE tag to abandoned + remove from active_branches.
- list_branches (L941): returns active_branches slice.
- extract_code_change_paths (L1109-1147): JSON-decodes code_changes BLOB, extracts keys (Object) or "path" fields (Array). NB: the spec says findings/code_changes/tests_written are MSGPACK BLOBs, but the conflict-detection path uses serde_json::from_slice. Currently the CLI passes `code_changes: None` (L4164) so this never runs in production, masking the format mismatch — divergence flagged.
- SPEC AMENDMENT #2 (SSB action_type for branch ops): NOT IMPLEMENTED. merge_branch never writes to SSB. The spec says winner/abandoned/snap_ids should be recorded as `action_type='checkpoint'` with payload {event:"branch_merge",...}. Missing.
- SPEC AMENDMENT #3 (SSB cleanup on merge): N/A while #2 missing.

Tests: merge_branch_creates_new_trunk_snapshot, merge_branch_with_overlapping_paths_returns_conflict, merge_branch_not_found, branch_limit_exceeded, branch_naming validation tests (L2660-2672), branches_persist_across_open (L2721) — confirms BranchRef rebuilt from DB tags on open() (L324-366).

CLI WIRING (partial):
- cli/main.rs:5132-5166 — /session fork delegates to dag.create_branch with sane error mapping (BranchLimitExceeded -> user error). PARTIAL.
- merge_branch / close_branch / list_branches: NOT CALLED from CLI. No /session merge or /session close commands found. Branches can be created but never closed or merged through the user interface.

3H.4 VERDICT: PARTIAL — library complete except SSB record-keeping; runtime exposes only create_branch.

---

3H.5 STUB RECOVERY FROM SSB — VERDICT: LIBRARY PASS / RUNTIME UNWIRED

Evidence:
- recover_stubs (L1162-1212) and recover_single_stub (L1216-1272):
  - rmp_serde::from_slice deserializes Stub from msg.content (L1174, L1235)
  - safe_i64 conversion of ssb_seq (L1181)
  - SsbReader::query_by_seq (L1187, L1248)
  - blake3::hash(&entry.payload) compared against stub.summary_hash (L1198-1204, L1259-1265) -> StubRecoveryFailed{reason:"integrity check failed"} on mismatch
  - On success: msg.content = entry.payload.to_vec(); is_stubbed=false; stub_ssb_seq=None
- Doc comment (L1156-1161) explicitly documents the contract: SSB entry must contain RAW content bytes (Ssb::write, not write_payload) — i.e. caller must pre-write the payload such that blake3(written_bytes) matches the hash trim_context computed.
- Tests: recover_stubs_no_stubs_unchanged, recover_stubs_signature_callable, recover_single_stub_non_stubbed_unchanged, round_trip_trim_save_load_recover (contracts L471-540).

ADVERSARIAL CHECKS:
- Integrity verification: blake3 hash compare with explicit error. PASS.
- Missing SSB entry: explicit StubRecoveryFailed{reason:"SSB entry missing"}. PASS.
- Mismatch: explicit StubRecoveryFailed{reason:"integrity check failed"}. PASS.

CRITICAL RUNTIME FAILURE:
- Grep for `.recover_stubs(` in production code: ZERO matches outside tests.
- Grep for `.recover_single_stub(` in production code: ZERO matches outside tests.
- The CLI cosmetic `[stubbed — content archived in SSB]` replacement (L4082-4088) is permanent — there is no recovery path to swap it back. If trim_context were ever wired (which it isn't), the user would see the cosmetic stub forever.
- The pre-write-to-SSB invariant the doc comment describes is never satisfied anywhere. No code path writes raw tool_result bytes to SSB and threads the seq into TrimmedMessage.stub_ssb_seq.

3H.5 VERDICT: LIBRARY PASS / RUNTIME 0% INTEGRATED

---

3H.6 HANDOFF AND INTEGRATION — VERDICT: LIBRARY PASS / RUNTIME ABSENT

Evidence:
- create_handoff (L1282-1330): trims with derived valid_ssb_seqs (filter_map of stub_ssb_seq) — see note below — then save_snapshot with tag="handoff", parent_id from latest_snapshot(None).
- load_handoff (L1337-1354): branch-tip-first (active_branches.first()), else latest_snapshot(Some("handoff")). Returns Option.
- Tests: test_create_handoff_tag (L2823), test_load_handoff_empty_returns_none (L2847), test_load_handoff_no_branch_returns_trunk (L2859), test_load_handoff_with_active_branch_returns_branch_tip (L2885), test_handoff_integration_lifecycle (L2917-3089) — full 50-message + 2-branch + merge + acyclic-chain + commit_hashes-preservation cycle.
- Contract tests mirror these (session_dag_contract.rs:596-810).

NB on valid_ssb_seqs derivation (L1292-1295): valid set is computed FROM the messages themselves — i.e. if a caller passes in messages with stub_ssb_seq=None for everything (which the CLI does), the valid set is empty, Pass 1 has zero candidates, Pass 3 has nothing to validate. Functionally this means create_handoff degrades to "save_snapshot with tag" when called from the current runtime.

85% TRIGGER:
- cli/main.rs:198 — `const TRIM_TRIGGER_RATIO: f64 = 0.85;`
- cli/main.rs:4042-4043 — `let threshold = (model_window * TRIM_TRIGGER_RATIO) as usize; if estimated_tokens > threshold && !session.messages.is_empty() { ... trim_context ... }`
- This satisfies the spec's "85% triggers; 84.9% no trigger; 100% triggers" check, BUT the trim itself is a no-op (3H.3 finding above). The trigger fires; the trimming does nothing.

CHECKPOINT TRIGGERS (every GRC step / tool call / phase transition):
- Snapshot save (try_save_snapshot, cli/main.rs:4119) is invoked ONLY at run_turn end (L4325), AFTER persist_session.
- There is NO hook for GRC step boundaries.
- There is NO hook for individual tool-call completion.
- There is NO hook for phase_transition (FSM state change is DETECTED at L4173-4181 to trigger plan propagation, but does not itself trigger an additional snapshot — it piggy-backs on the same per-turn snapshot).
- Spec says "Checkpoint triggers: every GRC step, tool call, phase transition." Implementation reduces this to "once per REPL turn." Substantial divergence; gating loose enough that ~10-20 tool calls inside a single agent turn produce ONE snapshot, not N.

CLI HANDOFF WIRING:
- create_handoff: ZERO call sites outside tests.
- load_handoff: ZERO call sites outside tests.
- /resume in cli/main.rs:4942-5001 calls SessionDag::open and pulls latest_snapshot(None) for last_snap_id (L4984), but loads the legacy JSON session via Session::load_from_path (L4959). The DAG handoff payload is not used to reconstruct the message stream — it is discarded.
- In other words: the on-disk format the CLI actually round-trips through is JSONL (session.rs:159-180 save_to_path/load_from_path -> render_jsonl_snapshot/from_jsonl). The DAG's snapshot_context is written by try_save_snapshot but never read for replay.

3H.6 VERDICT: LIBRARY PASS / RUNTIME WIRING IS A NO-OP STAPLE

---

CROSS-CUTTING FINDINGS

JSON vs MSGPACK ON DISK:
- Session messages on disk: JSONL (session.rs:159 save_to_path -> render_jsonl_snapshot at L396; load via from_jsonl at L317). Phase 0 audit gap (msgpack for disk sessions, item 0.7b) remains UNRESOLVED.
- DAG snapshot_context.content: stored as raw BLOB (whatever bytes the caller hands in — for the CLI, UTF-8 text bytes from joined ContentBlock::Text).
- DAG findings / code_changes / tests_written: declared as BLOB in DDL (session_dag.rs:171-173), spec calls for msgpack Vec<...> payloads, but in the CLI all three are written as None (cli/main.rs:4163-4165). When set, the merge-conflict detector uses serde_json::from_slice on code_changes (session_dag.rs:1128) — JSON, not msgpack. Format inconsistency; never exercised at runtime.
- DAG snapshots.commit_hashes: serialized as JSON string (session_dag.rs:400, 492-493). Spec says "comma-separated SHAs" — divergence (more flexible / quoting-safe).
- DAG Stub records (the bytes inside snapshot_context.content for stubbed messages): MSGPACK via rmp_serde (session_dag.rs:786, 1174, 1235). Correct per spec.
- FSM state file: msgpack via .msgpack extension (cli/main.rs:3597 `.fsm-state.msgpack`).
- Net answer: on-disk session messages = JSON. DAG snapshot envelope columns = mixed (BLOB, JSON commit_hashes string). Stub payloads inside DAG = msgpack. Spec's msgpack-everywhere intent is partially honored.

BLAKE3 IN USE:
- YES. session_dag.rs:778 (trim Pass 2), L1198 (recover_stubs), L1259 (recover_single_stub). All hash sites use blake3, no blake2 fallback. Compliant with CLAUDE.md anti-stale rule.

WIRED TO REPLACE RAW SESSION JSONL:
- NO. The CLI continues to write the legacy JSONL session via persist_session at every turn end (cli/main.rs:4324) and reads it at /resume via Session::load_from_path (L4959). The DAG runs as a parallel write-only ledger: try_save_snapshot writes a row, but no code path reads snapshot_context to reconstruct a session. SessionDag::open is called at /resume (L4957) only to retrieve last_snap_id; the message stream is loaded from JSONL, not from snapshot_context.

UNGROUNDED CLAIMS RESOLVED:
- "blake3 hashing UNGROUNDED in spec": the choice is grounded in CLAUDE.md project rule (blake3 over blake2 always). Implementation honors it consistently. RESOLVED — defensible.
- "MergeResult::Conflict variant UNGROUNDED": variant exists (L97-99) and the code overlap-check actually uses it (L976-984). Conflict semantics are heuristic (file-path overlap in code_changes), not a true 3-way merge — but the variant is wired and tested. RESOLVED — implemented as overlap heuristic; document the heuristic.

PHASE 0 KNOWN ISSUE (msgpack for disk sessions, 0.7b): STILL OPEN. session.rs uses JSONL exclusively. This phase did not address it.

---

SUMMARY TABLE

3H.1 Schema + types         | PASS (schema_version table is undocumented but harmless)
3H.2 Save/load + parent     | PASS
3H.3 3-pass trim            | LIBRARY PASS / RUNTIME DEAD (stub_ssb_seq always None at call site)
3H.4 Branch / merge         | LIBRARY PASS / RUNTIME PARTIAL (only create_branch wired; SSB record-keeping per AMENDMENT #2 missing)
3H.5 Stub recovery          | LIBRARY PASS / RUNTIME 0% (no call sites)
3H.6 Handoff + integration  | LIBRARY PASS / RUNTIME ABSENT (no caller; per-turn snapshot only; legacy JSONL still authoritative)

OVERALL: PARTIAL. The orchestrator crate ships a clean, well-tested CMV-derived DAG library that implements the spec at the API level (with the 3 documented amendments mostly resolved or downgraded). The CLI integration is a token gesture: it instantiates SessionDag, writes hollow snapshots once per turn, and reads back only last_snap_id for branching. The trim/stub/recover/handoff pipeline is never exercised on real session data because (a) no code populates stub_ssb_seq, (b) no code calls create_handoff or load_handoff, (c) no code calls recover_stubs or merge_branch / close_branch from the user interface, and (d) the legacy JSONL session in orchestrator/src/session.rs remains the source of truth for replay.

REQUIRED FOLLOW-UPS BEFORE 3H CAN BE CALLED COMPLETE:
1. Add a pre-trim SSB write loop in cli/main.rs that, for every Tool message above some byte threshold, writes the payload to SSB via Ssb::write and stamps the returned seq onto the in-memory message before passing to dag.trim_context.
2. Call dag.create_handoff (not just dag.save_snapshot) when TRIM_TRIGGER_RATIO is exceeded; persist the resulting snap_id; on /resume call dag.load_handoff and reconstruct the runtime session from snapshot_context (or call recover_stubs first).
3. Wire merge_branch and close_branch to /session merge and /session close commands.
4. Implement SPEC AMENDMENT #2: write SSB action_type='checkpoint' (with payload describing the merge event) inside merge_branch.
5. Reconcile findings/code_changes/tests_written serialization: pick msgpack OR JSON consistently, then update extract_code_change_paths accordingly.
6. Add additional checkpoint triggers per spec — at minimum, on FSM phase_transition and after each tool_call dispatch, not just at REPL turn boundary.
7. Resolve Phase 0 item 0.7b: migrate session.rs disk format to msgpack OR explicitly mark JSONL as the chosen format and amend the spec.
