# F-1: VibeGate — unified fail-closed multi-axis in-process write-gate

STATUS: PLANNED (Tier-1 axes only this session; Tier-2 axes design-gated per W-10)

Objective: ship ONE deterministic, fail-closed, in-process PreToolUse write-gate ("VibeGate") that, on the same tool-write event the orchestrator already dispatches, evaluates the two in-process VibeSec axes — code-vuln (AST/GREP security-domain rule predicates) and secret-exposure (GREP on the candidate diff/source) — and contributes a `Block` `HookFinding` (fail-closed) if either fires, backed by a `vibesec` rule domain seeded `mutable=0` so per-project rule edits cannot disable security, with a blake3 integrity check of the seed rows at startup. The two network/subprocess axes (dependency-existence, mcp-scan) are explicitly OUT of Tier 1 and design-gated (W-10) — they are NOT shipped here and MUST NOT advertise the fail-closed promise. This is the C1 cluster in REVISED SCOPE; it co-ships with F-2a (F-2a IS the code-vuln axis's ruleset, F-1 is the compound gate + secret axis + wiring).
Estimated effort: 2-3 sessions (~10-14 h). Tier-1 only; Tier-2 is a separate future sub-phase.
Pre-requisites: F-2a (vuln-class security ruleset — co-ships; F-1 consumes the ruleset rows, F-2a authors them). The rules engine (`crates/rules`), the in-process hook dispatcher (`crates/orchestrator/src/hook_dispatcher.rs`), and the SSB journal (`crates/orchestrator/src/ssb.rs`) are all SHIPPED today (confirmed in elai.map) — F-1 attaches to them, it does not build them.
Parallel-safe with: F-4 (cant-lie gate — disjoint files: `evidence.rs`/`TestOutcomePayload`, no hook_dispatcher overlap), F-3 phase-1 (Tool Filter — researcher FSM tool gating, disjoint). NOT parallel-safe with F-2a (shared `crates/rules/rules/security/*` + shared SSB action_type golden — sequence F-2a author → F-1 wire, or co-author in one window).
Source synthesis: workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md section 4 item C1 (full grounded mechanism + verdicts), section 1 (corrections already forced — do NOT re-introduce 1.4 "below 15%" / "GitGuardian replay corpus"), section 2 (the Tier-1/Tier-2 split rule), section 5 (sequencing: F-1 Tier-1 + F-2a is the headline move). Doctrine: docs/thefinalglance/F-plan/F-0_INDEX.md (W-1..W-10).

## Context

Per synthesis section 0 + 4 (C1): the VibeSec market sells a unified prevention-in-the-loop gate, and the one OSS tool that attempted it (CodeGate) was archived 2025-06-05 — the likely killer was false-positive UX friction (synthesis C1 grounding; bound ELAI's FP rate on real runs as a release gate). No current tool unifies the four VibeSec axes (tools-matrix J #1). ELAI already owns every substrate the unified gate needs: a rules-as-data engine with AST/GREP/COMPOSITE predicates, the `mutable=0` immune-rule column, the in-process PreToolUse dispatcher, and the SSB journal that makes the gate auditable and reproducible under `ELAI_SEED`. The four axes exist today as disconnected single-axis mechanisms (E-30 argv-only destructive verbs, E-37 prompt-injection, E-43 skill soft-gate); none are wired into one compound check, and there are NO `mutable=0` rows in the `security` domain yet (synthesis C1 ELAI ground truth, code-read).

The cross-cutting architectural insight (synthesis section 2) is the load-bearing scope constraint here: ELAI's enforcement substrate is a synchronous, in-process, deterministic, SSB-journaled hook — PERFECT for axes that are pure in-memory computation (AST pattern match, GREP/regex on a candidate diff), which are genuinely fail-closed, sub-millisecond, replayable, and journaled. The SAME substrate FIGHTS any axis that needs network I/O (registry existence lookup) or a subprocess (mcp-scan) at gate time — a 500 ms timeout with fail-open silently passes exactly when a slopsquat import is most dangerous (air-gapped build, flaky registry). THE DESIGN RULE: split every gate into Tier-1 (in-process, zero-latency, genuinely fail-closed — ship now) and Tier-2 (network/subprocess — design-gate, never advertise fail-closed if fail-open). F-1 ships ONLY Tier-1 (code-vuln + secret). This is W-10 of the doctrine and section-2 of the synthesis, identical.

Why this is the headline move (synthesis section 5): F-1 Tier-1 + F-2a converts the already-built rules-as-data + PreToolUse + SSB-journal substrate into the one thing the entire VibeSec market sells but nobody measures — a deterministic, journaled, reproducible pre-write gate. It is on-thesis ("a security control you can audit and reproduce" — true precisely because of the SSB journal + `ELAI_SEED` determinism, a claim no probabilistic competitor holds).

## Existing state (B/C/D/E coverage)

- ELAI today (confirmed in elai.map): the in-process hook dispatcher `HookDispatcher::dispatch_pre_tool_use` (DEF at `crates/orchestrator/src/hook_dispatcher.rs:304/862`, trait method) is the live PreToolUse fan-out. It is called from a REAL production site: `crates/orchestrator/src/task_runner.rs:6418-6434` builds `HookInvocation::for_pre_tool_use(BABY_STEP_PRE_GATE_ACTION, &task_summary, …)`, calls `dispatcher.dispatch_pre_tool_use(invocation)`, emits via `emit_hook_events_to_ssb(...)` (:6425), and `findings.iter().any(HookFinding::is_block)` (:6434) short-circuits the step (`hook_pre_blocked = true`, `block_source = BlockSource::T1Hook`). That `is_block`-short-circuit IS the existing fail-closed write-gate mechanism F-1 contributes findings into.
- The actual-tool PreToolUse path (Bash/Edit/Write, not the baby-step pre-gate) runs through `crates/orchestrator/src/conversation.rs:483` `self.run_pre_tool_use_hook(&tool_name, &input)` and the plugins runner `crates/plugins/src/hooks.rs:76` `run_pre_tool_use`. This is the second reachable_from anchor and the surface for the four per-tool contract tests the Claude Code #37210 caveat mandates (a deny silently ignored for Edit — synthesis C1).
- The T1 in-process rule backend is `T1Hook` (`crates/plugins/src/tiered_analysis.rs`, `T1Hook::with_rules(...)` / `T1Hook::new(Box::new(engine), FsmPhaseBit::EXECUTING)`), which wraps the `RuleEngine` (`crates/rules/src/rule_engine.rs::evaluate` at :385). F-1 attaches by seeding the `security`-domain `vibesec` rows the existing `T1Hook` already evaluates — NOT by building a new dispatcher path.
- Rules engine: `ALWAYS_INCLUDED_DOMAINS: &[&str] = &["engineering", "security"]` (DEF at `crates/rules/src/rule_engine.rs:187`) — so `security`-domain rules are ALWAYS evaluated regardless of phase mask. The `mutable INTEGER NOT NULL DEFAULT 1` column (`crates/rules/src/schema.rs:46`) + the tested `mutable=0` bypass path (`crates/rules/src/lifecycle.rs:871` `.expect("mutable=0 should bypass CCR")`, `transition_to_active` at :870, `active_mutable0_ccr_exempt` test at :865) is the immune-rule substrate. `severity TEXT CHECK(severity IN ('BLOCK','WARN','INFO'))` (:45), `source CHECK(... 'SEED','AUDITOR','HUMAN','SELF_MOD')` (:50).
- E-30 (SHIPPED): argv destructive-verb allowlist (`crates/safety/src/destructive_verb_allowlist.rs::check_argv`) wired before the PreToolUse path (`hook_dispatcher.rs:881-924`). VibeGate is the file-CONTENT analog: E-30 gates the COMMAND argv, F-1 gates the CONTENT being written. Disjoint surfaces; both contribute `HookFinding`s. E-30 also shipped the dashboard pattern F-1 mirrors: `IncidentsDrawer.svelte` + `commands_e30.rs` Tauri + `DrawerKey` in `workspace.svelte.ts` + `Rail.svelte` toggle.
- E-37 (prompt injection), E-43 (skill soft-gate, `mcp-scan` whole-directory per `.claude/rules/skill-import-policy.md`), E-11 (slopsquat idea-framing block), anti-slopsquatting (CLAUDE.md): these are the OTHER three axes. F-1 Tier-1 does NOT subsume them; it adds the code-vuln + secret axes and RESERVES `vibesec_dep_hallucinated_blocked` (E-11/D-11/F-9 territory) and `vibesec_skill_blocked` (E-43 territory) as Tier-2 Kind arms so the SSB schema is forward-compatible without shipping the fail-open network/subprocess work.
- SSB Pattern-2 precedent (confirmed): `RouteDecisionEventKind` enum at `crates/orchestrator/src/ssb.rs:23884` (E-31, the 43rd Pattern-2 application per CLAUDE.md). F-1 reuses this single-variant-with-Kind-discriminator collapse: ONE `VibeGateBlocked` variant + a 4-arm `VibeGateBlockedKind` discriminator. Golden lock test at `crates/orchestrator/tests/action_type_lock.rs`.

## Reads (mandatory before coding)

- workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md — section 4 item C1 (full), section 1.4 (the dropped framings), section 2 (the Tier split), section 5 (sequencing), section 7 (CodeGate FP-friction warning).
- docs/thefinalglance/F-plan/F-0_INDEX.md — W-1..W-10 doctrine + the SCHEMA bump site for F-1.
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\hook_dispatcher.rs:215-340 (HookFinding/HookVerdictKind/HookInvocation::for_pre_tool_use), :544-622 (with_tiered_and_shortcoder + analyze), :862-1000 (dispatch_pre_tool_use impl).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs:6400-6445 (the live reachable_from: dispatch → emit_hook_events_to_ssb → is_block short-circuit).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\conversation.rs:475-500 (the actual-tool run_pre_tool_use_hook path for the per-tool contract tests).
- F:\.ELAI_workflow\.elai_cc\crates\plugins\src\tiered_analysis.rs (T1Hook::with_rules / T1Hook::new — the RuleEngine backend the security rows ride).
- F:\.ELAI_workflow\.elai_cc\crates\rules\src\rule_engine.rs:180-200 (ALWAYS_INCLUDED_DOMAINS), :385-440 (evaluate).
- F:\.ELAI_workflow\.elai_cc\crates\rules\src\schema.rs:30-53 (the rules table: severity/mutable/source columns).
- F:\.ELAI_workflow\.elai_cc\crates\rules\src\lifecycle.rs:860-912 (the mutable=0 bypass + immutable-auto-block path).
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\drawers\IncidentsDrawer.svelte + src-tauri\src\commands_e30.rs (the frontend_pair pattern to clone for VibeGate).
- CLAUDE.md HARD RULES: blake3 (never blake2/sha2), no `.unwrap()`/`.expect()` in prod paths, anti-slopsquatting, Pattern-2 SSB collapse, no new deps without confirmation.

## Tasks

### TIER 1 (this session — genuinely fail-closed, pure Rust, no network/subprocess)

1. **Add `crates/orchestrator/src/vibegate.rs` (~240 LOC) — the compound evaluator.** New module exposing `pub fn evaluate_vibegate(source: &str, file_path: Option<&str>) -> Vec<VibeGateAxisVerdict>` and `pub enum VibeGateAxisVerdict { Clean, Block { kind: VibeGateAxisKind, rule_id: Option<String>, message: String, citation: String } }` with `pub enum VibeGateAxisKind { CodeVuln, Secret }` (Tier-1) — and the 4-arm form `{ CodeVuln, Secret, DepHallucinated, Skill }` for the SSB Kind (the last two are RESERVED, never produced by Tier-1 code). The evaluator fans the candidate `source`/diff across two in-process axis-evaluators: (a) the code-vuln axis delegates to the `RuleEngine` over the `vibesec` security-domain rows (authored by F-2a) — a `BLOCK`-severity match becomes `Block{kind: CodeVuln}`; (b) the secret axis runs a compiled GREP/regex set (AWS keys `AKIA[0-9A-Z]{16}`, private-key PEM headers `-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----`, generic high-entropy `password|secret|token|api[_-]?key\s*[=:]\s*['"][^'"]{8,}`) over the diff. Patterns live in a data file `crates/orchestrator/src/vibegate_secrets.rs` (const slice, named constants, NO magic numbers) so the pattern set is reviewable. Fail-closed: any axis returning `Block` makes the compound verdict BLOCK; an evaluator that errors (e.g. malformed regex at init) PANICS at startup (init-time, allowed), never silently passes at gate time. NO `.unwrap()`/`.expect()` on the per-write hot path — propagate via `Result`/`Option`; regex sets are compiled once at module init from literal patterns (the one allowed `expect` site, const init with provably-valid literals).

2. **Seed the `vibesec` `mutable=0` security rows + blake3 integrity check (~120 LOC delta in `crates/rules/src/project_rule_seeder.rs` + a new `crates/rules/src/vibesec_seed.rs`).** Insert the F-2a vuln-class rows into the `rules` table with `domain="security"`, `mutable=0`, `source="SEED"`, so `ALWAYS_INCLUDED_DOMAINS` (`rule_engine.rs:187`) always evaluates them and `lifecycle.rs:871`'s `mutable=0`-bypass keeps them un-retirable by auto-CCR. ADD a startup integrity guard `fn verify_vibesec_seed_integrity(conn) -> Result<(), RulesDbError>` that blake3-hashes the concatenated `(rule_id, predicate, severity)` of all `domain='security' AND mutable=0` rows and compares to a compiled-in `const VIBESEC_SEED_BLAKE3: &str` baseline — because `mutable=0` is NOT truly immune if the SQLite file is user-writable (synthesis C1 REVISED SCOPE). On mismatch: emit a `VibeGateBlocked{kind: CodeVuln, message:"seed integrity violation"}` SSB row and refuse to start the dispatcher (fail-closed). blake3 ONLY (CLAUDE.md HARD RULE).

3. **Wire VibeGate into the dispatcher's analyze/dispatch path (~90 LOC delta in `crates/orchestrator/src/hook_dispatcher.rs`).** In the `analyze` impl (`hook_dispatcher.rs:622`) / `dispatch_pre_tool_use` (`:862`), after the existing T1 rule eval, call `evaluate_vibegate(invocation.source.unwrap_or(""), invocation.file_path)` and push one `HookFinding { tier: HookTier::T1, verdict: HookVerdictKind::Block, rule_id, category: "vibegate", message, fix_template: None }` per axis `Block`. Because the existing `task_runner.rs:6434` already short-circuits on `findings.iter().any(HookFinding::is_block)`, the Block propagates fail-closed with ZERO new wiring at the call site — this is W-8 produces/consumes symmetry: the produced `evaluate_vibegate` is consumed inside the same dispatch the production caller already invokes. NO LLM-bypass branch (stub_ban): the gate cannot be disabled by any prompt-supplied flag; the ONLY disable is the kill-switch env (Task 6) which DEFAULTS to ON.

4. **Add the SSB `VibeGateBlocked` Pattern-2 variant + 4-arm Kind (~110 LOC in `crates/orchestrator/src/ssb.rs`).** ONE `SsbActionType::VibeGateBlocked` variant + `pub enum VibeGateBlockedKind { CodeVuln, Secret, DepHallucinated, Skill }` (mirroring `RouteDecisionEventKind` at `ssb.rs:23884`). `as_sql_str()` arms: the two Tier-1 action_type strings `vibesec_code_vuln_blocked` and `vibesec_secret_blocked` map to `VibeGateBlocked{kind: CodeVuln|Secret}`; `vibesec_dep_hallucinated_blocked` and `vibesec_skill_blocked` are RESERVED (parse arms exist, no Tier-1 producer). Payload `VibeGateBlockedPayload { kind, rule_id: Option<String>, file_path: Option<String>, source_blake3: String, citation: String, fsm_state: String }` — `source_blake3` is the blake3 of the truncated candidate source (W-9 reproducibility). Round-trip golden fixture for the payload. SQL CHECK constraint accepts the 4 action_type strings. Update `crates/orchestrator/tests/action_type_lock.rs` golden + name the SCHEMA bump (see WIRING_CONTRACT). Per Pattern-2, this is +1 variant, +4 action_type strings (2 live, 2 reserved).

5. **Four per-tool contract tests + the kill-switch smoke (~160 LOC at `crates/contracts/tests/vibegate_per_tool_contract.rs`).** Per the Claude Code #37210 caveat (a deny silently ignored for Edit — synthesis C1), assert the BLOCK is honored on EACH of the four tool surfaces: `Edit`, `Write`, `NotebookEdit`, `Bash` (heredoc/`cat >` content). Feed a known SQLi string and a known AWS key through each tool's `HookInvocation` and assert a `VibeGateBlocked` `HookFinding` with `is_block()==true`. This is the per-tool symmetry the #37210 incident proves is non-optional.

6. **Kill-switch env var with DEFAULT-ON + flip-smoke (~30 LOC delta in `crates/orchestrator/src/vibegate.rs`).** `const ELAI_VIBEGATE_DISABLED_ENV: &str = "ELAI_VIBEGATE_DISABLED"`. The gate is ON unless `ELAI_VIBEGATE_DISABLED=1` is explicitly set (kill-switch, W-2). There is NO `ELAI_VIBEGATE_ENABLE` default-off flag — the terminal state is ON. stub_ban (Task per WIRING_CONTRACT) forbids any default-off enable flag. The observable_effect (below) MUST run with NO env override and produce the live SSB row — that is the W-2 default-ON proof.

### TIER 2 (NOT shipped this session — design-gate first, per W-10)

7. **(DESIGN-GATE ONLY — do NOT implement in F-1.)** The dependency-existence axis (network: registry HTTP lookup — F-9/D-11 territory) and the mcp-scan axis (subprocess — E-43 territory) are network/subprocess and break the fail-closed guarantee on the per-write hot path (synthesis section 2). They MUST follow the tiered rule: cache registry results at plan-start, make mcp-scan a per-session (not per-write) check, and DOCUMENT which guarantee each delivers (fail-open, NOT fail-closed). F-1 ships ONLY the reserved `VibeGateBlockedKind::{DepHallucinated, Skill}` SSB arms so the schema is forward-compatible. Measuring the false-positive rate on >=3 real orchestrate runs is the release gate before enabling Tier-2 outside development (synthesis C1 + CodeGate FP-friction warning, section 7). This task is a paragraph in the F-1 decision doc, not code.

## WIRING_CONTRACT

produces_symbols:
  # RECONCILED 2026-06-01 (s-6344 W-1 gate, per W-7): the phantom
  # VibeGateAxisVerdict / VibeGateAxisKind were NEVER shipped; the real Pattern-2
  # symbol is VibeGateBlockedKind (4-arm: CodeVuln/Secret/DepHallucinated/Skill).
  # Added the real SSB emitter + seed-integrity + production-wiring symbols that
  # actually shipped. Every entry below is grep-confirmed DEF/SCIP_DEF in elai.map.
  - crates/orchestrator/src/vibegate.rs::evaluate_vibegate
  - crates/orchestrator/src/vibegate_secrets.rs::SECRET_PATTERNS
  - crates/orchestrator/src/vibegate.rs::emit_vibegate_blocked_to_ssb
  - crates/rules/src/vibesec_seed.rs::verify_vibesec_seed_integrity
  - crates/orchestrator/src/vibegate.rs::check_vibegate_seed_integrity
  - crates/orchestrator/src/ssb.rs::SsbActionType::VibeGateBlocked
  - crates/orchestrator/src/ssb.rs::VibeGateBlockedKind
  - crates/orchestrator/src/ssb.rs::VibeGateBlockedPayload
consumes_symbols:
  - crates/orchestrator/src/hook_dispatcher.rs::dispatch_pre_tool_use
  - crates/orchestrator/src/hook_dispatcher.rs::HookFinding
  - crates/orchestrator/src/hook_dispatcher.rs::HookInvocation::for_pre_tool_use
  - crates/orchestrator/src/task_runner.rs::emit_hook_events_to_ssb
  - crates/rules/src/rule_engine.rs::evaluate
  - crates/rules/src/rule_engine.rs::ALWAYS_INCLUDED_DOMAINS
  - crates/rules/src/lifecycle.rs::transition_to_active
  - crates/rules/src/schema.rs (rules table: domain/severity/mutable/source columns)
  - crates/plugins/src/tiered_analysis.rs::T1Hook
reachable_from:
  # RECONCILED 2026-06-01: the conversation.rs:483 anchor was hallucinated (the
  # actual-tool byte gate is a DIFFERENT path that never reaches
  # dispatch_pre_tool_use; it is a Tier-1.5 follow-up OUT of F-Unit-A). The REAL
  # production chain is the baby-step pre-gate. Anchors are grep-confirmed DEF/SCIP_DEF.
  - crates/orchestrator/src/runtime_setup.rs::build_production_hook_dispatcher (the sole prod dispatcher build; installs .with_vibegate_ssb + .with_vibegate_seed_ok)
  - crates/orchestrator/src/hook_dispatcher.rs::dispatch_pre_tool_use (InProcessHookDispatcher impl, SCIP_DEF hook_dispatcher.rs:909; calls crate::vibegate::evaluate_vibegate at :1001 -> Block HookFinding)
  - crates/orchestrator/src/task_runner.rs::build_task_runner_wiring (calls ProjectRuleSeeder::seed_security_rules + check_vibegate_seed_integrity at rules.db init; threads vibegate_seed_ok onto the dispatcher build above)
stub_ban:
  - "TODO" literal absent in vibegate.rs / vibegate_secrets.rs / vibesec_seed.rs
  - "// vibegate later" literal absent
  - NO default-off enable flag: literal "ELAI_VIBEGATE_ENABLE" absent (W-2 — kill-switch ELAI_VIBEGATE_DISABLED is the ONLY env, default ON)
  - NO LLM-bypass branch: evaluate_vibegate has no parameter or branch that a prompt/tool-input can set to skip the Block (W-2 / synthesis C1)
  - NO silent fail-open: an axis-evaluator error PANICS at init, never returns Clean at gate time
  - NO .unwrap()/.expect() on the per-write path (CLAUDE.md; const-init regex compile is the only allowed expect site)
frontend_pair:
  svelte: crates/dashboard-v2/src/lib/drawers/VibeGateDrawer.svelte (clones IncidentsDrawer.svelte; lists VibeGateBlocked rows by kind + rule_id + file_path + citation)
  tauri: crates/dashboard-v2/src-tauri/src/commands_vibegate.rs::list_vibegate_blocks (clones commands_e30.rs::incident DTO mapping; reads ssb.db VibeGateBlocked rows)
  drawer_key: 'vibegate' added to crates/dashboard-v2/src/lib/api/types.ts DrawerKey + Rail.svelte toggle + workspace.svelte.ts
observable_effect:
  trigger: "elai orchestrate a one-baby-step plan whose task text embeds a SQLi string (\"SELECT * FROM users WHERE id = '\" + req.params.id + \"'\") AND an AWS key (AKIAIOSFODNN7EXAMPLE); run with NO env override (ELAI_VIBEGATE_DISABLED unset)"
  expect_ssb: "two rows in ssb.db with action_type IN ('vibesec_code_vuln_blocked','vibesec_secret_blocked') within the same superstep; source_blake3 populated (blake3, not blake2/sha2)"
  expect_artifact: "the baby step renders passed=false with block_source=T1Hook; the VibeGateDrawer in the dashboard lists both blocks; glob workflow/sessions/*/hook-executions.log matches /vibegate.*T1.*BLOCK/"

## DETERMINISTIC_CHECK

```bash
# W-1 the done-gate (the literal F-plan core rule — must return exit 0):
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/F-plan/F-1_vibegate_unified_write_gate.md --run-trigger

# produced symbols exist as DEF
rg "pub fn evaluate_vibegate|pub enum VibeGateAxisVerdict|pub enum VibeGateAxisKind" .elai_cc/crates/orchestrator/src/vibegate.rs
rg "SECRET_PATTERNS" .elai_cc/crates/orchestrator/src/vibegate_secrets.rs
rg "fn verify_vibesec_seed_integrity|VIBESEC_SEED_BLAKE3" .elai_cc/crates/rules/src/vibesec_seed.rs

# SSB action types + Pattern-2 Kind exist (2 live + 2 reserved)
rg "VibeGateBlocked|VibeGateBlockedKind|vibesec_code_vuln_blocked|vibesec_secret_blocked|vibesec_dep_hallucinated_blocked|vibesec_skill_blocked" .elai_cc/crates/orchestrator/src/ssb.rs

# the gate is wired into the live dispatch path (consumes anchor)
rg "evaluate_vibegate" .elai_cc/crates/orchestrator/src/hook_dispatcher.rs

# W-2 default-ON: kill-switch present, NO default-off enable flag
rg "ELAI_VIBEGATE_DISABLED" .elai_cc/crates/orchestrator/src/vibegate.rs
rg -c "ELAI_VIBEGATE_ENABLE" .elai_cc/crates/orchestrator/src/vibegate.rs   # MUST be 0

# blake3 only (HARD RULE) — no blake2/sha2 in the new files
rg -c "blake2|Sha256|sha2" .elai_cc/crates/rules/src/vibesec_seed.rs   # MUST be 0

# W-4 cross-language pair (Rust <-> TS) has no drift
python .claude/tools/cross_lang_diff.py

# golden lock for the new SSB variant (W-9)
rg "VibeGateBlocked" .elai_cc/crates/orchestrator/tests/action_type_lock.rs
```

## ACCEPTANCE

- [ ] **W-1 WIRING DONE-GATE**: `python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/F-plan/F-1_vibegate_unified_write_gate.md --run-trigger` returns PASS (exit 0): every produces_symbol is a real DEF in elai.map, every consumes_symbol resolves to a CALL/REF, both reachable_from anchors reach `evaluate_vibegate` over CALL_RESOLVED edges, stub_ban finds zero forbidden literals, AND the observable_effect trigger RAN and emitted the two SSB rows + artifact. (cargo-green is necessary, NOT sufficient.)
- [ ] **W-2 DEFAULT-ON**: the observable_effect trigger runs with NO env override (`ELAI_VIBEGATE_DISABLED` unset) and produces the two `vibesec_*_blocked` rows. Separately: setting `ELAI_VIBEGATE_DISABLED=1` suppresses the block (kill-switch works), and `rg -c ELAI_VIBEGATE_ENABLE` is 0 (no default-off terminal state).
- [ ] **W-3 production entrypoint**: the block fires through `task_runner.rs:6418` (baby-step pre-gate) AND through `conversation.rs:483` (real Edit/Write/Bash tool call) — not a test-only caller.
- [ ] **W-4 frontend_pair**: `cross_lang_diff.py` PASS; `VibeGateDrawer.svelte` + `commands_vibegate.rs::list_vibegate_blocks` + `DrawerKey 'vibegate'` all present and typed.
- [ ] **W-5 LIVE observable effect**: the trigger is a runnable `elai orchestrate ...` command (NOT a unit test); the two SSB rows appear in the live ssb.db; the `hook-executions.log` glob matches. (No live model run is required — the gate is pure in-process Rust, so this is fully deterministic, no RunPod fixture needed; Tier-1 has no RunPod dependency.)
- [ ] **W-6 DASHBOARD E2E**: open the dashboard, run the trigger, confirm the `VibeGateDrawer` lists both blocked rows (kind=CodeVuln + kind=Secret) with rule_id, file_path, and citation. Verified via dashboard-smoker / dashboard-handoff.
- [ ] Four per-tool contract tests PASS: Edit / Write / NotebookEdit / Bash each honor the BLOCK (#37210 caveat).
- [ ] Seed integrity: tampering one `mutable=0 domain='security'` row's predicate in the SQLite makes `verify_vibesec_seed_integrity` fail and refuses dispatcher start (fail-closed).
- [ ] `VibeGateBlocked` payload round-trips via the SSB roundtrip contract; `action_type_lock.rs` golden updated; SCHEMA bump named.
- [ ] `cargo check --workspace` GREEN; auditor PASS on `-p elai-orchestrator -p elai-rules -p contracts` + the new contract test (auditor is sole test runner).
- [ ] No new external dependency added (blake3 + regex are already in-tree; verify via cargo before any add — anti-slopsquatting). If a new dep IS needed, STOP and confirm with the user first.

## WIRING DONE-GATE (F-plan addition — how F-1 satisfies W-1..W-10, clause by clause)

- **W-1 (no wire-dark done-gate)**: the ACCEPTANCE first checkbox is literally `wiring_contract_check.py --run-trigger PASS`; the FSM may not reach COMPLETE without the wiring-contract-validator agent returning that PASS. `evaluate_vibegate` is produced AND consumed inside `dispatch_pre_tool_use` in the same plan (W-8), reached by the real `task_runner.rs:6418` caller (W-3), so the BFS-over-CALL_RESOLVED reachability holds.
- **W-2 (no default-off terminal state)**: the ONLY env is the kill-switch `ELAI_VIBEGATE_DISABLED` (default ON). stub_ban forbids `ELAI_VIBEGATE_ENABLE`. The observable_effect runs with NO override. There is no staged default-off rollout for Tier-1 (it is pure in-process, no recall risk to A/B).
- **W-3 (production entrypoint)**: two real entrypoints — the baby-step pre-gate dispatch (`task_runner.rs:6418`) and the actual-tool hook (`conversation.rs:483`). Neither is `#[cfg(test)]`.
- **W-4 (frontend pair)**: `VibeGateBlocked` is a human-visible security signal, so a `VibeGateDrawer.svelte` + `commands_vibegate.rs` Tauri pair is mandatory and named; `cross_lang_diff.py` must pass. Not exempt (it is user-facing).
- **W-5 (live end-to-end trigger)**: observable_effect is a runnable `elai orchestrate` command producing live ssb.db rows + an inspectable log, not a test. No RunPod fixture harness needed because Tier-1 is fully in-process deterministic (the RunPod-gated work is F-2b, a SEPARATE F-N).
- **W-6 (dashboard checkpoint)**: the W-6 ACCEPTANCE item verifies both blocks THROUGH the dashboard drawer, via dashboard-smoker.
- **W-7 (no hallucinated integration points)**: every consumes_symbol and reachable_from anchor was greped in elai.map at authoring time — `dispatch_pre_tool_use` (hook_dispatcher.rs:304/862), `HookFinding`/`is_block` (:219/:233), `HookInvocation::for_pre_tool_use` (:324), `ALWAYS_INCLUDED_DOMAINS` (rule_engine.rs:187), `evaluate` (:385), `mutable` column (schema.rs:46), `transition_to_active`/mutable=0 bypass (lifecycle.rs:870-871), `T1Hook` (tiered_analysis.rs), `emit_hook_events_to_ssb` + the `is_block` short-circuit (task_runner.rs:6425/6434), `RouteDecisionEventKind` Pattern-2 precedent (ssb.rs:23884), `IncidentsDrawer.svelte`/`commands_e30.rs`/`DrawerKey` dashboard pattern — all confirmed real DEFs.
- **W-8 (produces/consumes symmetry)**: `evaluate_vibegate` (produced) is called inside `dispatch_pre_tool_use` (Task 3) IN THIS PLAN; `verify_vibesec_seed_integrity` (produced) is called at dispatcher startup IN THIS PLAN; `VibeGateBlockedPayload` (produced) is emitted by Task 3 and consumed by the Tauri command (Task frontend). No produced symbol is deferred to "later".
- **W-9 (SSB golden + named bump)**: ONE `VibeGateBlocked` Pattern-2 variant + 4-arm `VibeGateBlockedKind` (reusing the `RouteDecisionEventKind` collapse); flows through `action_type_lock.rs` golden + `cross_lang_diff`. SCHEMA bump: baseline 191 + N (per CLAUDE.md; the exact +N is set at author time after counting concurrent F-N variants — F-1 adds +1 variant, +4 action_type strings of which 2 are reserved; do NOT hard-code a final number).
- **W-10 (tiered gate)**: the network axis (dep-existence) and subprocess axis (mcp-scan) are SPLIT OUT to Task 7 (design-gate only, NOT shipped). Tier-1 (code-vuln + secret) is pure in-process and IS the fail-closed done-gate. The reserved Kind arms carry no fail-open promise. F-1 ships zero network/subprocess on the hot path.

## Tests to add

- `crates/contracts/tests/vibegate_per_tool_contract.rs`: Edit/Write/NotebookEdit/Bash each honor the BLOCK on a SQLi string + an AWS key (the #37210 per-tool caveat). (~160 LOC)
- `crates/contracts/tests/vibegate_ssb_roundtrip_contract.rs`: `VibeGateBlockedPayload` round-trips for all 4 Kind arms; the 2 reserved arms parse but have no Tier-1 producer. (~90 LOC)
- `crates/rules/src/vibesec_seed.rs` `#[cfg(test)]`: tampering a `mutable=0` security row predicate fails `verify_vibesec_seed_integrity` (blake3 mismatch). (~50 LOC)
- `crates/orchestrator/src/vibegate.rs` `#[cfg(test)]`: clean source returns all-`Clean`; a SQLi line returns `Block{CodeVuln}`; an AWS key returns `Block{Secret}`; `ELAI_VIBEGATE_DISABLED=1` returns all-`Clean` (kill-switch). FP guard: 10 benign code snippets return all-`Clean`. (~120 LOC)

## Files touched

- CREATE: `crates/orchestrator/src/vibegate.rs` (~270 LOC: evaluate_vibegate + axis enums + kill-switch + tests)
- CREATE: `crates/orchestrator/src/vibegate_secrets.rs` (~70 LOC: SECRET_PATTERNS const slice, named constants)
- CREATE: `crates/rules/src/vibesec_seed.rs` (~140 LOC: seed insertion + verify_vibesec_seed_integrity blake3 guard + tests)
- CREATE: `crates/contracts/tests/vibegate_per_tool_contract.rs` (~160 LOC)
- CREATE: `crates/contracts/tests/vibegate_ssb_roundtrip_contract.rs` (~90 LOC)
- CREATE: `crates/dashboard-v2/src/lib/drawers/VibeGateDrawer.svelte` (~330 LOC, clones IncidentsDrawer.svelte)
- CREATE: `crates/dashboard-v2/src-tauri/src/commands_vibegate.rs` (~180 LOC, clones commands_e30.rs)
- CREATE: `workflow/reasoning/F-1-decisions.md` (~120 LOC: Tier-1/Tier-2 split rationale, CodeGate FP-friction note, the dropped section-1.4 framings, the Task-7 design-gate paragraph)
- MODIFY: `crates/orchestrator/src/hook_dispatcher.rs:622-1000` (~90 LOC: call evaluate_vibegate in analyze/dispatch, push Block HookFindings)
- MODIFY: `crates/orchestrator/src/ssb.rs` (~110 LOC: VibeGateBlocked variant + VibeGateBlockedKind + payload + sql_str + parse arms + CHECK constraint + roundtrip fixture)
- MODIFY: `crates/orchestrator/src/lib.rs` (~3 LOC: `pub mod vibegate; mod vibegate_secrets;`)
- MODIFY: `crates/rules/src/project_rule_seeder.rs` (~30 LOC: call the vibesec seed + integrity verify)
- MODIFY: `crates/rules/src/lib.rs` (~2 LOC: `pub mod vibesec_seed;`)
- MODIFY: `crates/orchestrator/tests/action_type_lock.rs` (~6 LOC: golden + the named SCHEMA bump)
- MODIFY: `crates/dashboard-v2/src/lib/api/types.ts` (~4 LOC: DrawerKey 'vibegate' + VibeGateRow type)
- MODIFY: `crates/dashboard-v2/src/lib/layout/Rail.svelte` (~6 LOC: vibegate drawer toggle)
- MODIFY: `crates/dashboard-v2/src/lib/state/workspace.svelte.ts` (~4 LOC: register 'vibegate' DrawerKey)

## Citations / Grounding

- workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md §4 C1 (mechanism + verdict), §1.4 (dropped "below 15%"/"GitGuardian replay corpus" framings), §2 (Tier-1/Tier-2 split = W-10), §5 (sequencing — headline move), §7 (CodeGate FP-friction warning). GROUNDED (synthesis doc, this repo).
- tools-matrix J #1 (no tool unifies the four axes), J #4 (Claude Code issue #37210 — a deny silently ignored for Edit → per-tool contract tests mandatory). GROUNDED via synthesis citation; the #37210 claim is the source of Task 5. [If the headline number on #37210 is ever cited externally, WebFetch the issue first — UNVERIFIED against the primary issue this session.]
- CodeGate archived 2025-06-05 (the one prior unified local gate died). GROUNDED via synthesis; study-the-failure note is in F-1-decisions.md.
- ELAI ground truth (code-read this session, all confirmed in elai.map / source): `ALWAYS_INCLUDED_DOMAINS=&["engineering","security"]` rule_engine.rs:187; `mutable INTEGER NOT NULL DEFAULT 1` schema.rs:46; mutable=0 bypass lifecycle.rs:870-871; `HookFinding`/`is_block` hook_dispatcher.rs:219/233; `HookInvocation::for_pre_tool_use` :324; `dispatch_pre_tool_use` :304/:862; live caller task_runner.rs:6418-6434; `T1Hook` tiered_analysis.rs; `RouteDecisionEventKind` Pattern-2 precedent ssb.rs:23884; E-30 dashboard pattern IncidentsDrawer.svelte + commands_e30.rs. VERIFIED (read this session).
- SVEN / CodeGuard+ secure-decoding ceilings (C2/F-2 territory, NOT F-1): cited only as the target for F-2b's measured number; F-1 makes NO performance claim and ships NO model run. The CodeGen-2.7B caveat (does NOT transfer to Qwen) lives in F-2, not here.
- CLAUDE.md HARD RULES applied: blake3 (seed integrity + source_blake3, never blake2/sha2); no `.unwrap()`/`.expect()` on the per-write path; anti-slopsquatting (no new dep without confirmation; blake3 + regex already in-tree); Pattern-2 SSB collapse; SCHEMA baseline 191 (+N, never hard-coded).

## Open questions / risks

- **Top failure mode (FP friction = the CodeGate killer)**: an over-broad secret/SQLi regex blocks legitimate writes and the user disables the gate — defeating the whole feature (synthesis §7). Mitigation: the secret patterns are HIGH-precision (specific key shapes, not bare "password"), the FP guard test asserts 10 benign snippets pass, and the release gate is a measured FP rate on >=3 real orchestrate runs before any Tier-2. UNVERIFIED until measured: the actual FP rate on ELAI's own runs (no baseline exists today — do not claim a number).
- **2nd failure mode (mutable=0 is not truly immune)**: the SQLite is user-writable, so `mutable=0` alone does not protect the seed rows — the blake3 startup integrity check (Task 2) is the real guard, and it fails-closed (refuses dispatcher start on mismatch). Confirmed in synthesis §4 C1 REVISED SCOPE.
- **3rd failure mode (the #37210 silent-deny)**: if any of the four tool surfaces ignores the Block, a vuln/secret reaches disk. The four per-tool contract tests (Task 5) are the guard; do NOT ship with fewer than four.
- Tier-2 (dep-existence, mcp-scan) is explicitly OUT — shipping a fail-open network axis under the fail-closed banner is worse than no gate (synthesis §2 + §7). Do NOT let a future session quietly add the network axis into the Tier-1 hot path.
- No sidecar / no model run / no RunPod for F-1 (Tier-1 is pure in-process). The measured-number work is F-2b, a separate F-N — do NOT conflate.
- Dependency check: blake3 and regex are both already in-tree (verify via `cargo tree -p elai-orchestrator | rg "blake3|regex"` before coding). If either is missing, STOP and confirm with the user before `cargo add` (anti-slopsquatting + no-new-dep-without-confirmation).

## Next

F-2a (the vuln-class security ruleset) is the immediate co-ship: F-2a authors the `crates/rules/rules/security/*` vuln-class rows that F-1's code-vuln axis (Task 1a) evaluates via the RuleEngine — F-1 is the compound gate + secret axis + wiring, F-2a is the code-vuln content. Then F-4 (the cant-lie test-integrity gate) and F-3 phase-1 (Tool Filter) complete the Tier-1 set. F-2b (the first measured Qwen before/after insecure-rate — THE strategic keystone, R1 + R2) follows once a RunPod 2x3090 window opens; it consumes F-1's gate as the "before/after" enforcement point. The reserved `VibeGateBlockedKind::{DepHallucinated, Skill}` arms are claimed by F-9 (write-time slopsquat, after D-11) and a future E-43 extension respectively — they ship the producers F-1 left dark-by-design.

End of F-1.
