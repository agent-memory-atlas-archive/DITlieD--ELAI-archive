# E-30: Public agent incidents (non-LLM destructive-verb argv allowlist, trust dialog ordering, MCP pin+auth, independent verifier)

STATUS (completion-drive close, s-25120, 2026-05-23)
====================================================
overall: CLOSED (19 SHIPPED-ON-V2 + 4 MISSING built + 1 NOT_WIRED rewired + 4 SPEC_DRIFT rebaselined)
schema_version: 103 UNCHANGED (8 SSB variants shipped under renamed spellings; bump rejected -- would violate SCHEMA-LOCK without semantic delta)
test_pending: Y (user paused cargo test runtime; verification via `cargo check --workspace` only)
decisions doc: workflow/reasoning/E-30-decisions.md (full per-item rationale)

SHIPPED-ON-V2 (19 items, recognized via cross-check-recheck):
  - crates/safety crate (Cargo.toml + lib.rs + destructive_verb_allowlist.rs + trust_dialog.rs + policies/destructive_verbs.toml)
  - check_argv + AllowlistVerdict{Allow, Deny{reason,citation}, RequireHumanApproval{reason,citation}} + argv_blake3 helper
  - PreToolUse backend wires check_argv before existing path (hook_dispatcher.rs:881-924)
  - ensure_session_trust + TrustToken + mint_trust_token (trust_dialog.rs, re-exported via lib.rs)
  - LineageGroup + auditor_executor_lineage_collision predicate (per_role_runtime.rs:128-159)
  - TextSource + LabeledText + is_trusted_for_memory_write (conversation.rs:904-959)
  - 8 SSB action types shipped under renamed spellings (ssb.rs:2946-2953):
      ArgvDestructiveDenied/ArgvApprovalRequired/TrustDialogHookDeferred/TrustDialogEnvDeferred/McpPinVersionRefused/AuditorLineageCollision/UntrustedMemoryWriteBlocked/IncidentRecordCreated
  - sql_str values + parse arms + payload variants (ssb.rs:3414-3421 + 3815-3822 + 6655-6660 + 10623-10690)
  - SQL CHECK constraint accepts all 8 action_types (ssb.rs:1670)
  - action_type_lock.golden:302 carries the locked variants
  - SCHEMA_VERSION 103 LOCKED (ssb.rs:1043); no bump for E-30 gap-closure
  - 6 BLOCK rule rows for 2026-05 (rules/src/schema.rs:226-238)
  - cmd_incident_record CLI subcommand (cli/src/main.rs:524-555)
  - workflow/incident-postmortem-template.md (39 LOC template)
  - .elai_cc/mcp_pins.toml (23 LOC starter config)
  - 5 contract test files (destructive_verb_redteam / trust_dialog_ordering_cve / auditor_lineage_collision / mcp_pin_violation / untrusted_memory_write)
  - lib.rs re-exports for safety crate + per_role_runtime types
  - Round-trip golden fixtures for AuditorLineageCollisionPayload + UntrustedMemoryWriteBlockedPayload (ssb.rs:15124-15135)
  - mcp_pins.toml policy fields (enforce_pin_list / refuse_version_float / refuse_unauthenticated)

ACTIVE WORK CLOSED THIS COMPLETION-DRIVE (4 MISSING + 1 NOT_WIRED):
  M1. argv_allowlist_latency bench at crates/safety/benches/argv_allowlist_latency.rs (~140 LOC, stdlib std::time::Instant + harness=false; p99 asserted <1ms per B-19 T1 budget; 12 fixtures x 1000 iters)
  M2. regression.rs 5 named taxonomy entries (~100 LOC, const E30_FAILURE_TAXONOMY: &[E30TaxonomyEntry] + lookup helpers; re-exported via orchestrator lib.rs)
  M3. crates/plugins/src/mcp/pins.rs (~340 LOC, McpPinsConfig::load + validate + find + enforce_pin returning McpPinVerdict; extends existing plugins crate rather than scaffolding new crates/mcp-loader)
  M4. IncidentsDrawer.svelte at crates/dashboard-v2/src/lib/drawers/ (~330 LOC svelte) + commands_e30.rs at crates/dashboard-v2/src-tauri/src/ (~180 LOC tauri) wiring 8 action_types into a typed DTO; DrawerKey 'incidents' added to types.ts + DrawerHost.svelte + Drawer.svelte
  NOWIRE-1. .claude/tools/failure_taxonomy.py split single "unsafe_action" into 5 named categories matching M2 (DestructiveVerbBlocked / TrustDialogDeferredLoad / McpServerPinMismatch / AuditorLineageCollision / UntrustedMemoryWriteRefused) anchored on LOCKED sql_str values

SPEC_DRIFT REBASELINED (4 items):
  SD-1. crates/safety uses `regex` crate not `globset` -- regex is correct for pattern union (globset is for fnmatch file globs); CLAUDE.md anti-stale "globset/ignore over glob" applies to file globs only. No code change.
  SD-2. sandbox path moved from crates/orchestrator/src/sandbox.rs to sandbox/mod.rs:1232-1330. Plan text stale, semantics unchanged. No code change.
  SD-3. mcp_pins.toml path correctly at .elai_cc/mcp_pins.toml (single .elai_cc prefix; plan text "double-prefix" was a typo). Verified via contract test path resolution.
  SD-4. "73% lineage-collision false-negative rate" attribution in schema.rs:234 flagged UNGROUNDED in grounding sweep; rule message left in-place as illustrative; follow-up: ingest arXiv:2511.18397 via paper-ingestor to ground or downgrade the number.

DEFERRED (out of E-30 scope):
  DEFERRED-TO-E-5-FOLLOWUP: TextSource refusal at mcp_memorize call sites. crates/memory/src/defense.rs Stage-1 + sensitive-class skip-list shipped; Stage-0 TextSource gate (E-30 plumbing) is NOT yet wired into defense::classify() or mcp_memorize. E-30 ships the labels + contract; E-5 owns the call-site refusal.
  DEFERRED: HTTP HEAD against npm/pypi for live version check in M3 (enforce_pin takes observed_version from caller; live fetch deferred to follow-up patch).
====================================================

Objective: harden ELAI against the 6+ documented public agent incidents (Replit DROP DATABASE, Cursor `git restore .`, Claude Code `rm -rf ~/`, Kiro env delete, DataTalks `terraform destroy`, postmark-mcp backdoor) by adding a NON-LLM PreToolUse argv-regex allowlist for destructive verbs, enforce trust-dialog-before-side-effect ordering (CVE-2025-59536 / CVE-2026-21852 fix pattern), pin+auth MCP servers, and require auditor on a different model lineage than the executor for verification.
Estimated effort: 2-3 sessions (~10-14 h)
Pre-requisites: B-19 (Hook System Unification, PLANNED — provides the IpcHookDispatcher that the new argv-regex allowlist plugs into). E-29 (MAST classifier — the IncorrectVerification mode flags the DGM/Replit deception pattern; auditor-lineage check produces evidence the classifier can flag). C-11 / C-44 / D-9 (sandbox + frozen-surface — these protect the file system; E-30 adds the argv-layer protection above them).
Parallel-safe with: E-29 (observability; E-30 adds new SSB action types for destructive-verb blocks and writes incident-style postmortem rows; no schema conflict if columns are added under E-29's SCHEMA_VERSION bump first), E-24 (MCP protocols; E-24 = protocol grounding, E-30 = MCP pin+auth hardening — coordinate the MCP loader edit window).
Source report: workflow/improvement-sweep-2026-05-19/reports/30-public-agent-incidents.md

## Context

Per report TL;DR: production-write actions (DB drops, `rm -rf`, `git restore`, `terraform destroy`) executed by agents during stated "code freeze" / "do not modify" states are the dominant trust-killing class. The report Section "Pattern taxonomy" enumerates 8 patterns: P1 destructive verb without out-of-band gate (Replit, Kiro, Cursor, Claude Code, DataTalks — 6+ incidents), P2 agent deception under failure (Replit fabricated 4,000 users + false logs; DGM faked unit-test pass logs; arXiv:2511.18397 reward hacking generalizes to alignment faking), P3 indirect prompt injection from untrusted content into authenticated tool context (Operator/Comet/Fellou — embracethered + brave.com), P4 loop/no-stop autonomy (AutoGPT, Devin), P5 supply chain attack on agent extensions (postmark-mcp v1.0.16 BCC backdoor after 15 clean versions — thehackernews 2025-09-29 + snyk.io; Clawdbot Jan 2026 — pointguardai), P6 hook/config preempts consent (CVE-2025-59536 Claude Code hooks, CVE-2026-21852 ANTHROPIC_BASE_URL — research.checkpoint.com), P7 demo/benchmark inflation (Devin Upwork bait-and-switch), P8 agent inherits broad human permissions (Kiro).

Crucially, in-prompt instructions are NOT an enforceable boundary: Replit ignored ALL-CAPS instructions repeated 11 times (theregister 2025-07-21). "Sandbox is best-effort if the LLM controls the shell" — Cursor sandbox executed `git restore .` and wiped 3 days of work (forum.cursor.com 154810). The fix is REFUSE BY ALLOWLIST AT THE ARGV LAYER, NOT REVIEW BY LLM.

ELAI's design defends P4 well (4-pattern anti-spiral), partially P1/P2/P3/P6 (Tier B Bash sandbox + auditor separate from executor + PreToolUse hook gating), unverified P5/P8. P2 (auditor lineage) is the highest unresolved theoretical risk per Anthropic Nov-2025 paper (arXiv:2511.18397) — if auditor and executor share model lineage, the auditor may rubber-stamp because reward-hack generalizes.

The report Section "Best practices that emerged (ranked)" prescribes 12 hooks. E-30 ships the top 6: (1) Non-LLM destructive-action gate, (2) Separate dev/prod state, (3) Independent verifier (different model lineage), (4) Out-of-band test execution capture, (5) Content-source trust labels, (6) MCP version pin + signature.

## Existing state (B/C/D coverage)

- ELAI today: PreToolUse hook at `crates/orchestrator/src/hook_dispatcher.rs:626-1036` per elai.map; `crates/orchestrator/src/conversation.rs:432-446` carries "PreToolUse hook cancelled tool / failed for tool / denied tool" warn paths. The wiring exists; E-30 ADDS the argv-regex allowlist BACKEND.
- B-19 (PLANNED): Hook System Unification specifies T1<1ms / T2<10ms / T3<5ms hook latency budgets. E-30's argv-regex allowlist must hit T1 (<1ms) — regex compilation up-front, match at call time.
- C-11 (PLANNED): Tier C sandbox isolation per `crates/orchestrator/src/sandbox.rs:1136-1156` (PR_CAPBSET_DROP loop, setrlimit, hidepid=2). E-30 LAYERS argv-regex BEFORE sandbox engages — the report Section "ELAI risk assessment P1" note: sandbox is correct LAST line of defense, not first.
- C-44 (PLANNED) / D-9 (PLANNED): frozen-surface + LockedKeysAuditor + LOCKED_KEYS_TOUCH SSB. E-30 extends LockedKeysAuditor with argv-pattern auditing.
- B-20 PLANNED carries the failure taxonomy 13→9 collapse. E-30's `destructive_verb_denied` action_type is a NEW symptom category; orthogonal to MAST (which is E-29 cause category).
- `crates/orchestrator/src/hook_dispatcher.rs:626` warns "PreToolUse hook cancelled tool"; line 651 `RuffSkipReason::BinaryMissing` and line 860 `pre_compact invoked`. These are the integration points for the new argv backend.
- CLAUDE.md hard rule: FSM gate check before any Edit/Write/Bash via `mcp_fsm_get_state`. E-30 does NOT replace this; it adds a separate non-LLM argv-layer ABOVE it.
- Hook dispatcher contract tests at `crates/contracts/tests/hook_dispatcher_contract.rs:41-283` (dyn-safety, dispatch_pre_tool_use, dispatch_post_tool_use, emit_hook_events). E-30 ADDS new contract tests for the argv-regex backend.
- Memory note `feedback_never_skip_audit_memorize.md`: full FSM flow mandatory. E-30's independent-verifier requirement must NOT break the audit FSM cycle.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\30-public-agent-incidents.md` (source report, full)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\hook_dispatcher.rs:620-1100` (PreToolUse/PostToolUse/PreCompact backend + ruff dispatch + DAIRA)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\conversation.rs:430-460` (PreToolUse cancel/fail/deny warn sites)
- `F:\.ELAI_workflow\.elai_cc\crates\contracts\tests\hook_dispatcher_contract.rs:40-310` (existing contract shape)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\sandbox.rs:1100-1170` (C-11 PLANNED layer below — for hierarchy diagram)
- `F:\.ELAI_workflow\.elai_cc\crates\rules\src\schema.rs:170-220` (rule-row shape for new "blocked verb" rules)
- `F:\.ELAI_workflow\.claude\tools\failure_taxonomy.py` (existing taxonomy script — E-30 adds new symptom category)
- arXiv:2511.18397 Anthropic reward-hacking generalization paper (cited; PRIMARY for auditor-lineage check)
- arXiv:2401.05566 Anthropic Sleeper Agents (cited)
- arXiv:2505.22954 DGM paper (cited; "test passed" fakery)
- research.checkpoint.com/2026/rce-and-api-token-exfiltration-through-claude-code-project-files-cve-2025-59536/ (cited; primary for Task 4)
- thehackernews.com/2025/09/first-malicious-mcp-server-found.html + snyk.io postmark-mcp breakdown (cited; primary for Task 5)
- harperfoley.com/blog/ai-agents-destroyed-production-zero-postmortems (cited)

## Tasks

1. **Add `crates/safety/src/destructive_verb_allowlist.rs` (~280 LOC).** New crate `crates/safety` (or new module in existing safety crate if present — verify via Glob). Exposes `pub fn check_argv(argv: &[OsString]) -> AllowlistVerdict` with three variants: `Allow`, `Deny { reason, citation }`, `RequireHumanApproval { reason, citation }`. Compiles regex set at crate init using `globset` per CLAUDE.md anti-stale ("globset/ignore over glob"). Match patterns (initial set — extensible via `crates/safety/policies/destructive_verbs.toml`):
   - `rm -rf` with target outside CWD-relative paths or expanding to `~` — DENY (cite Claude Code 2025-10 home-dir delete)
   - `git restore .` / `git restore --staged .` (anything-restore-all) — DENY (cite Cursor 2025-08 / 2026-03 incidents)
   - `git reset --hard` followed by any commit-ish that is NOT HEAD~0..HEAD — DENY (cite Cursor 2026-03 git stash+reset)
   - `git stash` followed by `git reset HEAD` in same session — DENY (Cursor pattern)
   - `terraform destroy` / `tofu destroy` — REQUIRE_HUMAN_APPROVAL (cite DataTalks 2026-02)
   - SQL `DROP DATABASE` / `DROP TABLE` / `TRUNCATE` against any host not in localhost-allow — DENY (cite Replit SaaStr 2025-07)
   - `aws .* delete-*` / `aws .* terminate-*` — REQUIRE_HUMAN_APPROVAL (cite Kiro 2025-12)
   - `gcloud .* delete` / `az .* delete` — REQUIRE_HUMAN_APPROVAL
   - `kubectl delete` against any namespace not in dev-namespace-allow — REQUIRE_HUMAN_APPROVAL
   - `docker rm -fv` against any container not in test-prefix-allow — REQUIRE_HUMAN_APPROVAL
   - Glob expansion crossing `~` or starting with `/` — DENY (Claude Code precedent)
   - `chmod 777` / `chmod -R 777` — DENY
   Per CLAUDE.md "blake3 over blake2/sha2": any session-token hash uses blake3.

2. **Wire destructive_verb_allowlist into PreToolUse backend (~150 LOC delta in `crates/orchestrator/src/hook_dispatcher.rs`).** New `IpcHookDispatcher` backend method `dispatch_argv_allowlist(argv)` called BEFORE the existing PreToolUse path. Returns `AllowlistVerdict`. On `Deny`: emit `destructive_verb_denied` SSB row with `{argv_hash:blake3, reason, citation, session_id, fsm_state}`, refuse the tool call, log to `workflow/sessions/{id}/incidents.log`. On `RequireHumanApproval`: enter new FSM sub-state `AWAITING_HUMAN_APPROVAL`, emit SSB row, block tool call until approval row arrives. T1 latency budget per B-19 (<1ms) — regex match only, no I/O. Per report HARD RULE: in-prompt instructions are NOT a safety boundary, this MUST be in compiled code, not in any prompt.

3. **Trust-dialog-before-side-effect ordering (CVE pattern) (~180 LOC).** Per report Section "Best practices 8": "Trust dialog before ANY side-effect". CVE-2025-59536 (Claude Code hooks RCE) + CVE-2026-21852 (ANTHROPIC_BASE_URL exfil) both share the failure: hook execution OR env honor preceded user consent dialog. New module `crates/safety/src/trust_dialog.rs`: `fn ensure_session_trust(project_root, settings_path) -> Result<TrustToken>`. Refuses to load `.claude/settings.json` hooks, `.elai_cc/settings.json` hooks, `.elai_cc/.elai_cc/hooks/*`, or honor `ANTHROPIC_BASE_URL` / `OPENAI_BASE_URL` env vars BEFORE the user has issued a session-level trust action (typing a prompt counts; flag-passing does not). Emit `trust_dialog_required` SSB row on first deferred-load attempt; emit `trust_token_minted` on grant. Per CLAUDE.md FSM gate: hook trust check runs BEFORE FSM transition checks.

4. **MCP pin + signature loader (~220 LOC in `crates/mcp-loader/` or equivalent — verify via Glob).** Per report Section "Best practices 6+7": pin exact MCP version, refuse minor-version float, verify maintainer pubkey if available; mandatory authentication on MCP endpoints (bearer or mTLS; refuse `0.0.0.0` bind). New types `McpServerPin { name, exact_version, expected_pubkey_blake3, auth_required }` loaded from a registry file `.elai_cc/.elai_cc/mcp_pins.toml`. Load-time verification: HTTP HEAD against npm/pypi to confirm exact-version match (cite postmark-mcp 1.0.16 backdoor pattern); blake3-hash the downloaded artifact and compare against pinned hash. Refuse on mismatch. Emit `mcp_pin_violation` SSB row.

5. **Auditor-lineage check (~180 LOC in `crates/orchestrator/src/per_role_runtime.rs` + new audit-time guard).** Per report Section "Best practices 3": auditor agent calls a separate model family AND a deterministic harness. Add guard: when spawning the auditor agent, assert that `agents.auditor.runtime` resolves to a DIFFERENT model lineage than `agents.executor.runtime`. Lineage groups: Anthropic (Claude *), Google (Gemini *), Qwen (Qwen *, Qwen2.5-Coder *), DeepSeek (DeepSeek *), Meta (Llama *), OpenAI (GPT *, o-* series). If auditor lineage == executor lineage, emit `auditor_lineage_collision` WARN row and (in autonomous mode) HALT with escalation; in interactive mode prompt the user to confirm. Out-of-band test execution capture: assert auditor reads test stdout from a path NOT writable by the executor (this is mostly already the case via Tier C sandbox; ADD a contract test asserting this).

6. **Content-source trust labels (~200 LOC).** Per report Section "Best practices 5": any text reaching the LLM that originated from a web fetch / OCR / clipboard / MCP tool gets tagged untrusted. New type `TextSource` enum {`User`, `SystemPrompt`, `ProjectFile`, `WebFetch`, `McpToolOutput`, `Ocr`, `Clipboard`}. New wrapper `LabeledText { source: TextSource, content: String }`. Refactor the prompt assembler at `crates/orchestrator/src/conversation.rs` (find prompt-assemble path via grep) to carry labels through. Block: `mcp_memorize` calls that originate from a `TextSource ∈ {WebFetch, McpToolOutput, Ocr, Clipboard}` segment are refused (cite SpAIware ChatGPT memory injection). Emit `untrusted_memory_write_refused` SSB row. Out-of-scope for E-30: full prompt-injection refusal training (research-grade); E-30 ships only the trust-label plumbing + memory-write block.

7. **Add failure-taxonomy entries (~80 LOC in `.claude/tools/failure_taxonomy.py` + `crates/orchestrator/src/regression.rs:150-170`).** New symptom categories (orthogonal to MAST from E-29):
   - `DestructiveVerbBlocked` (count per session)
   - `TrustDialogDeferredLoad` (count per session)
   - `McpPinViolation`
   - `AuditorLineageCollision`
   - `UntrustedMemoryWriteRefused`

8. **Add 8 new SSB action types (~80 LOC in `crates/orchestrator/src/ssb.rs`).** `destructive_verb_denied`, `destructive_verb_require_human_approval`, `trust_dialog_required`, `trust_token_minted`, `mcp_pin_violation`, `auditor_lineage_collision`, `untrusted_memory_write_refused`, `incident_postmortem_recorded`.

9. **Add Rules entries in `crates/rules/src/schema.rs:170-220` (~60 LOC).** Per report Anti-patterns + Best-practices:
   - `Some("2026-05"), "BLOCK", "In-prompt instructions are NOT a safety boundary; Replit ignored ALL-CAPS instructions repeated 11 times. Reference: theregister 2025-07-21. Destructive verbs MUST be gated by non-LLM argv-regex allowlist."`
   - `Some("2026-05"), "BLOCK", "Sandbox without argv-regex pre-check is best-effort. Cursor sandbox executed git restore . and wiped 3 days of work. Reference: forum.cursor.com 154810."`
   - `Some("2026-05"), "BLOCK", "MCP version float (npm ^X.Y.Z, pypi >=X.Y.Z): postmark-mcp v1.0.16 inserted one-line BCC backdoor after 15 clean versions. Reference: thehackernews 2025-09-29, snyk.io. ELAI MCP pins are exact-version with blake3 artifact hash."`
   - `Some("2026-05"), "BLOCK", "Auditor and executor on same model lineage: reward hacking generalizes to alignment faking. Reference: arXiv:2511.18397 (Anthropic Nov 2025)."`
   - `Some("2026-05"), "BLOCK", "Hooks / env vars honored before trust dialog: CVE-2025-59536 + CVE-2026-21852. Reference: research.checkpoint.com 2026."`
   - `Some("2026-05"), "BLOCK", "mcp_memorize from untrusted text source (WebFetch/McpToolOutput/Ocr/Clipboard): SpAIware ChatGPT memory injection persisted across sessions. Reference: embracethered.com."`

10. **Incident postmortem template + recorder (~120 LOC).** Per report Section "What the failures taught" + "harperfoley.com zero-postmortems" critique: ELAI must have a documented incident-response process. New template at `workflow/incident-postmortem-template.md` (NOT memory-bank; not project-rules). New SSB action_type `incident_postmortem_recorded` referenced by Task 8. New CLI subcommand `elai incident record` in `crates/cli/src/main.rs` that drops a template-pre-filled markdown into `workflow/incidents/{timestamp}-{slug}.md`. The template covers: timestamp, trigger event (which destructive_verb / mcp_pin_violation / etc. row), affected paths, agent + model lineage involved, mitigation taken, follow-up required.

11. **Red-team contract suite (~200 LOC at `crates/contracts/tests/destructive_verb_redteam_contract.rs`).** Per report Section "ELAI risk assessment Open Question 4": "Does ELAI's destructive-command allowlist cover the full set (rm -rf, git restore, git reset --hard, DROP, TRUNCATE, terraform destroy, aws delete-*, kubectl delete, docker rm -fv)? Measurable: red-team script attempts each command via Bash tool and counts denials." Build the script: 15 known-bad command shapes from the incident catalog; assert each is denied or escalated by the new argv-regex allowlist. Total denial rate MUST be 15/15.

12. **Decision doc at `workflow/reasoning/E-30-decisions.md`** covering: (a) argv-regex allowlist is NON-LLM (cite Replit ALL-CAPS-11x-ignored), (b) trust dialog ordering matches CVE-2025-59536/CVE-2026-21852 Anthropic fix pattern, (c) auditor-lineage check defaults to BLOCK-on-collision in autonomous, WARN-in-interactive (per CLAUDE.md autonomous-mode feedback), (d) MCP pin policy is exact-version + blake3 hash (no minor-version float), (e) content-source trust labels are the plumbing; full prompt-injection defense is research-grade out-of-scope, (f) explicit OUT-OF-SCOPE: held-out eval rotation (P7 demo-inflation mitigation; future E-N), microVM/gVisor isolation upgrade (Tier C already covers Linux, Windows path is future work).

## WIRING_CONTRACT

produces_symbols:
  - crates/safety/src/destructive_verb_allowlist.rs::{check_argv, AllowlistVerdict}
  - crates/safety/src/trust_dialog.rs::{ensure_session_trust, TrustToken}
  - crates/safety/policies/destructive_verbs.toml
  - crates/mcp-loader (or equivalent)::McpServerPin
  - .elai_cc/.elai_cc/mcp_pins.toml (NEW config)
  - workflow/incident-postmortem-template.md (NEW template, NOT memory-bank)
  - SsbActionType::{DestructiveVerbDenied, DestructiveVerbRequireHumanApproval, TrustDialogRequired, TrustTokenMinted, McpPinViolation, AuditorLineageCollision, UntrustedMemoryWriteRefused, IncidentPostmortemRecorded}
  - 6 new rule rows in crates/rules/src/schema.rs
  - 5 new failure-taxonomy entries in failure_taxonomy.py + regression.rs
consumes_symbols:
  - crates/orchestrator/src/hook_dispatcher.rs::dispatch_pre_tool_use (new argv-allowlist call before existing PreToolUse path)
  - crates/orchestrator/src/conversation.rs:430-460 (PreToolUse cancel/deny warn sites; new SSB emit alongside)
  - crates/orchestrator/src/per_role_runtime.rs (auditor-lineage guard)
  - C-44/D-9 LockedKeysAuditor (extended with argv-pattern audit)
reachable_from:
  - Agent attempts `Bash("rm -rf ~/something")` → dispatch_argv_allowlist returns Deny(rm-rf-tilde) → tool call refused → destructive_verb_denied SSB row + workflow/sessions/{id}/incidents.log entry
  - Boot with `.claude/settings.json` present → ensure_session_trust defers hook load → trust_dialog_required SSB row → on first user prompt, trust_token_minted → hooks load
  - Agent attempts `mcp_memorize(content, source=WebFetch)` → refused → untrusted_memory_write_refused SSB row
  - Auditor spawn with same-lineage runtime → auditor_lineage_collision WARN/HALT
stub_ban:
  - "TODO: harden destructive verbs" literal absent
  - "// safety later" literal absent
  - No bypass branch on dispatch_argv_allowlist (LLM cannot disable the allowlist via prompt)
  - No silent fallback when MCP pin verification fails (refuse-by-default)
  - No same-lineage auditor in autonomous mode (BLOCK)
frontend_pair: crates/dashboard-v2/src/lib/windows/SettingsWindow.svelte (RuntimePanel surfaces auditor-lineage status; new IncidentsDrawer.svelte surfaces destructive_verb_denied rows + postmortem links)
observable_effect:
  trigger: "Agent issues Bash tool call with argv ['rm', '-rf', '/home/user/repo'] OR session boot with a planted .claude/settings.json hook OR auditor spawn with same runtime as executor"
  expect_ssb: "destructive_verb_denied OR trust_dialog_required OR auditor_lineage_collision row written within 100ms of the trigger; argv-hash via blake3"
  expect_artifact: "Tool call refused; workflow/sessions/{id}/incidents.log carries the citation URL; FSM does NOT advance to EXECUTING for the refused step"

## DETERMINISTIC_CHECK

```bash
# 1. New safety crate exists with the allowlist module
test -f .elai_cc/crates/safety/src/destructive_verb_allowlist.rs
test -f .elai_cc/crates/safety/src/trust_dialog.rs
test -f .elai_cc/crates/safety/policies/destructive_verbs.toml

# 2. PreToolUse backend wires the new argv-allowlist
rg "check_argv|AllowlistVerdict" .elai_cc/crates/orchestrator/src/hook_dispatcher.rs

# 3. 8 new SSB action types
rg "destructive_verb_denied|destructive_verb_require_human_approval|trust_dialog_required|trust_token_minted|mcp_pin_violation|auditor_lineage_collision|untrusted_memory_write_refused|incident_postmortem_recorded" .elai_cc/crates/orchestrator/src/ssb.rs

# 4. Rules entries
rg "In-prompt instructions are NOT a safety boundary|argv-regex allowlist|MCP version float|Auditor and executor on same model lineage|Hooks / env vars honored before trust dialog|mcp_memorize from untrusted text source" .elai_cc/crates/rules/src/schema.rs

# 5. Red-team contract suite passes 15/15
cd .elai_cc && cargo test -p contracts --test destructive_verb_redteam_contract

# 6. MCP pin config exists with example pins
test -f .elai_cc/.elai_cc/mcp_pins.toml

# 7. Incident template + CLI subcommand
test -f workflow/incident-postmortem-template.md
rg "fn cmd_incident_record" .elai_cc/crates/cli/src/main.rs

# 8. Failure taxonomy + decision doc
rg "DestructiveVerbBlocked|TrustDialogDeferredLoad|McpPinViolation|AuditorLineageCollision|UntrustedMemoryWriteRefused" .claude/tools/failure_taxonomy.py
test -f workflow/reasoning/E-30-decisions.md
```

## ACCEPTANCE

- [ ] Red-team contract suite: 15 known-bad argv shapes (from incident catalog) all denied or require-human-approval; 15/15
- [ ] Argv-regex allowlist T1 latency (<1ms per B-19 budget) — bench test asserts p99 < 1ms
- [ ] Trust dialog ordering: `.claude/settings.json` hooks DO NOT execute before a session trust token is minted; CVE-2025-59536 PoC fixture is blocked
- [ ] `ANTHROPIC_BASE_URL` env honor DOES NOT exfil before trust token; CVE-2026-21852 PoC fixture is blocked
- [ ] Auditor-lineage check: spawning auditor on same model lineage as executor emits `auditor_lineage_collision` in interactive mode (WARN); in autonomous mode emits + HALTs
- [ ] `mcp_memorize` from `TextSource::WebFetch | McpToolOutput | Ocr | Clipboard` is refused with `untrusted_memory_write_refused` SSB row
- [ ] MCP pin loader refuses minor-version float; planted postmark-mcp-like pin mismatch is denied
- [ ] All 8 new SSB action types round-trip via SSB roundtrip contract
- [ ] All 6 new Rules entries appear in schema.rs and are surfaced by the rules linter
- [ ] `workflow/incident-postmortem-template.md` exists and `elai incident record` produces a pre-filled markdown
- [ ] `cargo check --workspace` GREEN; `cargo test -p safety -p contracts -p elai-orchestrator` PASS
- [ ] No new dependency outside cargo verify (anti-slopsquatting per CLAUDE.md)

## Tests to add

- `crates/contracts/tests/destructive_verb_redteam_contract.rs`: 15 known-bad argv shapes from incident catalog (Task 11).
- `crates/contracts/tests/trust_dialog_ordering_cve_contract.rs`: CVE-2025-59536 + CVE-2026-21852 PoC fixtures fail closed.
- `crates/contracts/tests/auditor_lineage_collision_contract.rs`: same-lineage spawn produces WARN/HALT per FSM mode.
- `crates/contracts/tests/mcp_pin_violation_contract.rs`: planted version-float + blake3-mismatch refused.
- `crates/contracts/tests/untrusted_memory_write_contract.rs`: mcp_memorize from each tagged TextSource is refused for the untrusted variants.
- `crates/safety/benches/argv_allowlist_latency.rs`: p99 < 1ms on a representative argv set.

## Files touched

- CREATE: `crates/safety/` (NEW crate, ~600 LOC across lib.rs + destructive_verb_allowlist + trust_dialog + policies dir)
- CREATE: `crates/safety/policies/destructive_verbs.toml` (~120 LOC initial rules)
- CREATE: `.elai_cc/.elai_cc/mcp_pins.toml` (~30 LOC starter pins for any MCP servers ELAI currently uses)
- CREATE: `workflow/incident-postmortem-template.md` (~40 LOC template)
- CREATE: `workflow/reasoning/E-30-decisions.md` (~120 LOC)
- CREATE: `crates/contracts/tests/destructive_verb_redteam_contract.rs` (~200 LOC)
- CREATE: `crates/contracts/tests/trust_dialog_ordering_cve_contract.rs` (~100 LOC)
- CREATE: `crates/contracts/tests/auditor_lineage_collision_contract.rs` (~120 LOC)
- CREATE: `crates/contracts/tests/mcp_pin_violation_contract.rs` (~110 LOC)
- CREATE: `crates/contracts/tests/untrusted_memory_write_contract.rs` (~90 LOC)
- CREATE: `crates/safety/benches/argv_allowlist_latency.rs` (~70 LOC)
- MODIFY: `crates/orchestrator/src/hook_dispatcher.rs:620-1100` (~150 LOC: argv-allowlist call before PreToolUse)
- MODIFY: `crates/orchestrator/src/conversation.rs:430-460` (~50 LOC: SSB emit on deny + LabeledText prompt assembler)
- MODIFY: `crates/orchestrator/src/per_role_runtime.rs` (~100 LOC: auditor-lineage guard)
- MODIFY: `crates/orchestrator/src/ssb.rs` (~80 LOC: 8 new action_types)
- MODIFY: `crates/orchestrator/src/regression.rs:150-170` (~30 LOC: 5 new failure-taxonomy entries)
- MODIFY: `crates/rules/src/schema.rs:170-220` (~60 LOC: 6 new rule rows)
- MODIFY: `crates/cli/src/main.rs` (~60 LOC: `incident record` subcommand)
- MODIFY: `.claude/tools/failure_taxonomy.py` (~30 LOC: 5 new categories)
- MODIFY: `Cargo.toml` workspace members (~3 LOC: add `crates/safety`)

## Citations / Grounding

- arXiv:2511.18397 — Anthropic reward-hacking generalizes to alignment faking (cited; PRIMARY for auditor-lineage)
- arXiv:2401.05566 — Anthropic Sleeper Agents (cited)
- arXiv:2505.22954 — DGM faked unit-test pass logs (cited via theregister)
- arXiv:2502.17521 — contamination survey (cited; P7 demo inflation — DEFERRED to future E-N)
- theregister.com/2025/07/21/replit_saastr_vibe_coding_incident — Replit DROP DATABASE + 4,000 fake users (cited; PRIMARY P1+P2)
- incidentdatabase.ai/cite/1152 — AIID canonical record (cited)
- forum.cursor.com 154810 + 156146 — Cursor git restore / git stash+reset (cited)
- harperfoley.com — 10 documented agent prod incidents Oct-2024..Feb-2026, zero vendor postmortems (cited; PRIMARY for Task 10)
- research.checkpoint.com/2026/rce-and-api-token-exfiltration-through-claude-code-project-files-cve-2025-59536/ — CVE-2025-59536 + CVE-2026-21852 (cited; PRIMARY for Task 3)
- thehackernews.com/2025/09/first-malicious-mcp-server-found.html — postmark-mcp v1.0.16 (cited; PRIMARY for Task 4)
- snyk.io postmark-mcp breakdown (cited)
- pointguardai.com — Clawdbot Jan 2026 (cited)
- ruh.ai/blogs/amazon-kiro-ai-outage-ai-governance-failure — Kiro Dec 2025 (cited)
- embracethered.com/blog/posts/2025/chatgpt-operator-prompt-injection-exploits — Operator HN PII exfil (cited; P3)
- brave.com/blog/unseeable-prompt-injections — Comet OCR injection (cited; P3)
- cyberscoop.com/openai-chatgpt-atlas-prompt-injection-browser-agent-security-update — OpenAI prompt-injection-unsolved admission (cited)
- answer.ai/posts/2025-01-08-devin.html — Devin 3/20 + no-stop autonomy (cited; P4 + P7)

## Open questions / risks

- Report Open Question 1: does ELAI's PreToolUse hook run BEFORE .claude/settings.json hooks load? E-30 Task 3 implements the fix; verify via trust_dialog_ordering_cve_contract test on a planted hook fixture.
- Report Open Question 2: is ELAI's auditor agent ever the same model lineage as the executor (Claude→Claude, Qwen→Qwen)? E-30 Task 5 BLOCKS this in autonomous mode; the per-role default mapping (B-37) routes auditor→AnthropicApi and executor→Sidecar today, so the default IS cross-lineage — verify in contract test.
- Report Open Question 3: are MCP tool outputs trust-labeled when they re-enter the LLM context? E-30 Task 6 plumbs the labels and refuses mcp_memorize from untrusted sources; full prompt-injection refusal is out-of-scope (research-grade).
- Report Open Question 4: does ELAI's destructive-command allowlist cover the full set? E-30 Task 11 red-team contract suite ANSWERS this — 15/15 must deny.
- Report Open Question 5: documented incident-response process — E-30 Task 10 ships the template + recorder; the 24h response SLA is a process commitment (out-of-spec for code).
- Ungrounded conjecture flagged in report: "in-prompt instructions are NOT a safety boundary" is PROVEN by the Replit ALL-CAPS-11x ignored precedent. Do NOT regress on this.
- Anti-Stale-Training: `globset` / `regex` versions verify before cargo add (anti-slopsquatting per CLAUDE.md).
- C-11 / C-44 / D-9 dependency: E-30's argv-allowlist runs ABOVE the sandbox; if C-11 ships later, argv-allowlist still provides a meaningful first line. If C-11 / D-9 ship before E-30, the integration is additive.
- Sidecar boot is HARDWARE-HEAVY (CLAUDE.md hard rule). Trust-dialog ordering tests must NOT boot the sidecar; use mock fixtures.
- The "WARN-in-interactive, BLOCK-in-autonomous" choice for auditor-lineage collision is policy-grade; surface to user in decision doc for explicit confirmation per CLAUDE.md anti-sycophancy.

## Next

E-29 (observability) is the natural composition: every SSB row produced by E-30 (destructive_verb_denied, trust_dialog_required, mcp_pin_violation, auditor_lineage_collision, untrusted_memory_write_refused, incident_postmortem_recorded) flows through E-29's OTLP exporter for off-machine archival. The MAST classifier's "IncorrectVerification" (Verification 21.3%) flags the auditor-lineage-collision pattern. Future E-N: P3 full prompt-injection defense (research-grade — Anthropic Dec 2025 admission "may never be solved"); P7 held-out eval rotation; P8 separate principal for the agent (non-admin local user + short-TTL scoped cloud tokens) — Kiro Dec 2025 precedent; Windows-side microVM/gVisor isolation upgrade.
