F-1 Tier-1 VibeGate decisions (session s-6344, Unit A co-ship with F-2a)

What I will implement
Unified fail-closed PreToolUse write-gate, Tier-1 axes only: code-vuln (AST/grep via security-domain
rules) + secret (regex scan of the write payload). One compound evaluator fans the write across the
two axes and BLOCKs fail-closed if either fires. Backed by a "vibesec"/security rule domain seeded
mutable=0 so per-project edits cannot disable security. New SSB VibeGateBlocked Pattern-2 variant.
Frontend VibeGateDrawer pair.

Key decisions (grounded)
- Injection site: InProcessHookDispatcher::dispatch_pre_tool_use at hook_dispatcher.rs:862 (impl block
  confirmed at L861 via grep). The E-30 argv early-return runs first; vibegate runs on the fallthrough
  after self.analyze(). NOT hook_dispatcher.rs:1477 (that impl block at L1476 is IpcHookAsyncAdapter).
- Secret axis: new free-fn evaluate_vibegate in crates/orchestrator/src/vibegate.rs + a SECRET_PATTERNS
  regex set in crates/orchestrator/src/vibegate_secrets.rs. regex confirmed in orchestrator Cargo.toml.
  Patterns infallible on hot path: compile once (LazyLock), init-panic on bad regex, never HookError at
  call time (GAP-6).
- Code-vuln axis: rides the EXISTING T1Hook/DefaultRuleEngine path via security-domain TOML seed rows.
  ALWAYS_INCLUDED_DOMAINS already contains "security" (rule_engine.rs:187), so no phase mask needed.
  This is shared with F-2a (same seed rows). No new dispatcher path.
- mutable=0 seed: rows inserted with mutable=0; the immune-rule path (lifecycle.rs mutable=0 bypass) is
  already tested-working. verify_vibesec_seed_integrity computes a blake3 digest of the mutable=0 seed
  rows at dispatcher startup (blake3 confirmed in both orchestrator and rules Cargo.toml; never blake2/sha2).
- SSB: ONE SsbActionType::VibeGateBlocked variant + VibeGateBlockedKind 4-arm enum
  (CodeVuln, Secret = live Tier-1 producers; DepHallucinated, Skill = reserved parse-only, no producer
  this session) cloning the RouteDecisionEventKind Pattern-2 precedent at ssb.rs:23884. Payload carries
  kind, rule_id, file_path, source_blake3, citation, fsm_state. as_sql_str: vibesec_code_vuln_blocked,
  vibesec_secret_blocked (+ 2 reserved strings).
- SCHEMA_VERSION 195->196 (confirmed live value at ssb.rs:2726; CLAUDE.md's 191 is stale). Co-shipped
  with F-2a's SecurityRuleBlocked in one atomic bump. TOTAL_VARIANTS 471->473 (+2 variants).
  Follow the ssb.rs:530 9-lockstep-site checklist + action_type_lock.golden + activity_entry coverage.
- Frontend pair (W-4): VibeGateDrawer.svelte (DrawerKey 'vibegate') + commands_vibegate.rs::
  list_vibegate_blocks. Both manually-maintained unions updated: types.ts DrawerKey AND Rail.svelte
  DrawerKeyUpper ('VIBEGATE'). DrawerHost exhaustive chain + src-tauri lib.rs mod + generate_handler.

Scope boundaries (documented, not stubs)
- GAP-1: the per-tool path from conversation.rs goes through HookRunner (hooks.rs), NOT
  InProcessHookDispatcher. VibeGate fires on the task_runner baby-step pre-gate path (the production
  W-3 anchor: task_runner drive_task ~L6475 -> dispatch_pre_tool_use:862 -> is_block short-circuit).
  Extending to the conversation.rs per-tool path is a separate follow-up; Tier-1 boundary.
- GAP-7: IpcHookAsyncAdapter (IPC mode) is a separate impl; VibeGate does not fire there in Tier-1.
- F-2b (InsecureRateMeasured + SecCodePltRunner + bench seccodeplt) is NOT in Unit A; deferred to Tier-2
  fixture per the plan's tiering. Only reserved parse arms here.

Expected test outcome
cargo check --workspace GREEN. New contract tests: vibegate_per_tool_contract (Edit/Write/NotebookEdit/
Bash each BLOCKed on a known SQLi + secret), vibegate_ssb_roundtrip_contract (VibeGateBlocked encode/
decode), and the F-2a security-rule contracts. action_type_lock count + golden pass with +2 variants.
observable_effect: a baby step embedding a SQLi string + an AWS key is refused before disk; two SSB rows
(vibesec_code_vuln_blocked + vibesec_secret_blocked) in the same superstep; VibeGateDrawer lists the block.

Unknowns / risks
- Exact merged-findings return shape at L862 (read L862-960 before editing; infallible vibegate).
- The 9 lockstep SSB sites must all be touched (ssb.rs:530 comment is the checklist).
- False-positive bound: the secret regex must not fire on the synthetic AKIA example only; bound FP on
  the 20 most-used real ELAI deps + clean orchestrate runs (CodeGate died of FP friction).

Fixer pass addendum (session s-6344, F-Unit-A audit BLOCK remediation, 2026-06-01)
The unit-level code compiled clean but failed the W-1..W-10 done-gate (wire-dark / default-off). Fixes:

B3 (W-3/W-7) language-agnostic code-vuln axis + the Tier-1 vs Tier-1.5 boundary.
The live production caller is task_runner baby-step pre-gate: HookInvocation::for_pre_tool_use passes the
task-SUMMARY text with file_path=None (hook_dispatcher.rs for_pre_tool_use sets file_path=None). With an
empty file_path, the rule engine's per-language GrepPattern filter (rule_engine.rs detect_language_from_ext
-> None) skipped every language-scoped F-2a security rule, so the code-vuln axis silently never fired on the
one live path. Fix: added EvaluationContext.language_agnostic; the GrepPattern lang filter is bypassed when
true; vibegate::run_code_vuln_axis sets it true when file_path is empty OR the detected language is unknown.
All other ~30 EvaluationContext construction sites set it false (no behaviour change). The F-1 observable_effect
(task text embeds a SQLi string + AWS key, NO env override) now reaches the gate over the task text.
BOUNDARY: the byte-level per-Edit/Write content gate (conversation.rs run_pre_tool_use -> elai_plugins
HookRunner in hooks.rs) is a DIFFERENT path that never reaches InProcessHookDispatcher::dispatch_pre_tool_use.
That per-tool byte gate (carrying real file_content + a populated file_path) is a Tier-1.5 follow-up, NOT in
F-Unit-A. The plan's conversation.rs:483 reachable_from anchor was a hallucinated integration point for Tier-1.

B2 (W-3/W-5/W-8) wired the SSB writer + seed-integrity into the sole production dispatcher build.
build_production_hook_dispatcher now takes (ssb, vibegate_seed_ok) and installs .with_vibegate_ssb(ssb,
RUNNER_AGENT_ID, RUNNER_SUPERSTEP) + .with_vibegate_seed_ok(seed_ok). All three prod callers (cli task_cmd,
cli run_subcommand, dashboard lib.rs) pass wiring.ssb.clone() + wiring.vibegate_seed_ok. So VibeGateBlocked /
SecurityRuleBlocked rows now actually emit in production.

B1 (W-2) flipped default-OFF env gates to a single default-ON kill-switch.
Removed the ELAI_VIBE_GATE_ENABLE / ELAI_SECURITY_RULE_BLOCKED_ENABLE default-off pair from
emit_vibegate_blocked_to_ssb. The ONLY env is now ELAI_VIBEGATE_DISABLED (default unset = enabled); set truthy
to disable (kill-switch, F-1 Task 6). Emission + the SecurityRuleBlocked beacon are ON by default. The two
legacy ENABLE consts remain DEFINED in ssb.rs only for the existing SSB round-trip tests (they no longer gate
the live path). Dashboard commands_vibegate / commands_f2 env_active now means "gate enabled" (= !disabled).

B5 (W-3/W-8) gave the mutable=0 seeder + blake3 integrity guard a production caller — and the DB-vs-in-memory
DECISION. build_task_runner_wiring now calls ProjectRuleSeeder::seed_security_rules(&rules_conn, clock) at
rules.db init (idempotent; inserts the 12 mutable=0 source=SEED security rows) and runs
check_vibegate_seed_integrity(&rules_conn) once, threading the bool onto RuntimeWiring.vibegate_seed_ok.
DECISION: the LIVE VibeGate code-vuln axis evaluates the EMBEDDED in-memory ruleset (rule_loader::
load_security_seed_rules_builtin), NOT the DB rules table. The DB seed + blake3 integrity guard is the
defence-in-depth copy: it proves the on-disk security rows have not been tampered (mutable=0 is not truly
immune on a user-writable SQLite — synthesis C1 REVISED SCOPE), and a mismatch forces the dispatcher to block
all code-vuln writes fail-closed (vibegate_seed_ok=false). The mutable=0 claim is therefore meaningful for the
DB copy's integrity story, not decorative — but the gate itself does not read the DB rows.

W-a: load_security_seed_rules_builtin now embeds the 4 security TOMLs via include_str! at compile time instead
of resolving CARGO_MANIFEST_DIR/../rules/seed/security at runtime. A relocated/shipped binary now loads the
full ruleset, so the code-vuln axis never spuriously fail-closes (vibesec-seed-load-failed) on a missing dir.
The TOML files remain the single source of truth (the embedded bytes are exactly those files).

W-b: removed the A3 dead-code surface (commands_f1.rs, components/VibeGatePanel.svelte, the orphan 'vibesec'
DrawerKey + its A3 types in types.ts, the commands_f1 mod + generate_handler entry in lib.rs). Kept the A5
split (commands_vibegate + commands_f2 + VibeGateDrawer.svelte + SecurityRulesPanel.svelte + DrawerKey
'vibegate'/'security-rules'). One surface, not two.

W-c (left as-is, noted): action_type_lock ALL_VARIANTS locks ONE arm per Pattern-2 variant
(VibeGateBlocked(CodeVuln)) per the RouteDecisionEvent precedent; the test passes. vibesec_secret_blocked is a
live wire string (present in the SQL CHECK + parse arm + as_sql_str) but is not separately golden-locked,
consistent with the Pattern-2 single-entry precedent. A future rename of the Secret arm string would go
uncaught — acceptable per precedent, flagged for a possible later golden expansion.

W-d: added two explicit pairs (vibegate_drawer: commands_vibegate.rs <-> VibeGateDrawer.svelte;
security_rules_drawer: commands_f2.rs <-> SecurityRulesPanel.svelte) to the PAIRS list the harness
.claude/tools/cross_lang_diff.py walks. Left .elai_cc/cross-lang-pairs.toml untouched (its name maps 1:1 to a
PairKind enum variant; adding a row would force a Rust enum + golden change out of F-Unit-A scope).
