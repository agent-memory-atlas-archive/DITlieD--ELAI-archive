# D-9: L3 expansion + frozen-surface access-control enforcement

Objective: document and STRUCTURALLY ENFORCE the expanded L3-frozen surface
list — every L1A/L2/L3 boundary per companion §2.3 + §4 (lines 156-194 +
347-362). Per Everitt current-RF principle (arxiv:1908.04734, T1): the agent
must not be able to modify the function that grades it. Add pre-commit hook +
LockedKeysAuditor + LOCKED_KEYS_TOUCH SSB type + git push BLOCK on attempted
edits.
Estimated effort: 1-2 sessions.
Pre-requisites: D-1..D-8 (the surfaces being frozen must exist first).
  spec-grounder verifies all DONE.
Parallel-safe with: D-10..D-18 (Wave 2 consumes L3-frozen surfaces but does
  not write to them). NOT parallel-safe with C-27 (WiringContract StubBanAuditor
  is partial L3 enforcement; D-9 extends with LockedKeysAuditor; coordinate
  hook order in .pre-commit hook chain).

## Context

Companion §2.3 (lines 182-194) + §4 (lines 347-362): the agent must NOT
modify L3 surfaces. Specifically L3 contains:
1. Staged-eval harness code
2. Tripwire threshold values + detection code (D-4)
3. MSI / BAI scoring function code (D-5)
4. Archive maintenance + parent-selection algorithm code (D-1 + D-8)
5. Held-out task selection algorithm (D-6)
6. Cross-domain canary task family definitions (D-7)
7. Cryptographic hash function selection (blake3 only)
8. The bound on AllowedConfigKey itself (cannot self-add new variants)
9. The verifier_required field on every ReasoningTemplate (cannot self-weaken)
10. Domain enum (cannot self-add new domains)
11. Tripwire thresholds in elai_cc.toml (kl_spec_tau, rho_delta, iqr_mult,
    ppl_sigma, ddm_thresholds, consensus_fire_count, window_size)
12. BAI parameters in elai_cc.toml (eval_budget, ci_alpha, per-family delta)
13. Held-out parameters in elai_cc.toml (ttl_months, ngram_size, jaccard_threshold)
14. Canary parameters in elai_cc.toml (regression_threshold, min_samples)

Cohen Boxed Myopic (arxiv:1908.04734 / AI Mag 43(3), T1) + Hubinger Risks
from Learned Optimization (arxiv:1906.01820, T1) + Everitt Reward Tampering
(arxiv:1908.04734, T1) — three independent T1 sources converge: the agent
must not edit the function that grades it.

Enforcement is three-layer:
- Layer 1: documentation (this file's task list + frozen-surface registry)
- Layer 2: pre-commit hook (rejects any commit touching listed paths/symbols)
- Layer 3: runtime auditor (LockedKeysAuditor walks SsbWriter writes for
  LOCKED_KEYS_TOUCH events)

Partial foundation present and usable:
- C-21 already wires check_anti_stale.sh CI + git pre-commit LOCKED_KEYS hook
  for separate locked-key class. D-9 extends with reasoning-surface paths.
- C-27 StubBanAuditor pattern (production-path stub detection); LockedKeysAuditor
  follows same pattern.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\reasoning-improvement-grounded-2026-05-19.md
  (§2.3 + §4 L3 expansion — lines 182-194 + 347-362)
- F:\.ELAI_workflow\workflow\papers\statistical-limits-2510.04399-interface.md (D-1)
- .elai_cc/scripts/locked_keys_check.sh (existing from C-21; extend with new paths)
- .elai_cc/.git/hooks/pre-commit (existing chain; extend)
- .elai_cc/crates/orchestrator/src/wiring_contract.rs (C-27 StubBanAuditor pattern)
- All Wave-1 D-N files (D-1..D-8) for the surface list

## Paper-ingestor pre-step

Spawn paper-ingestor with mode=paper for:
- arxiv:1908.04734 Everitt Reward Tampering (current-RF principle)
- arxiv:1906.01820 Hubinger Risks from Learned Optimization (mesa-optimization)
- Cohen, Hutter, Osborne 2022 AI Mag 43(3) DOI:10.1002/aaai.12064
  (Boxed Myopic; paper-ingestor handles DOI fetches)

Verify workflow/papers/{everitt-reward-tampering-1908.04734,hubinger-mesa-
1906.01820,cohen-boxed-myopic-aimag43-3}-interface.md.

## Tasks

1. **Locked-surface registry** at
   .elai_cc/scripts/locked_keys_registry.toml (new file):
   - List EVERY L3-frozen path + symbol per companion §2.3 + §4.
   - Format:
     ```toml
     [[locked_path]]
     path = ".elai_cc/crates/orchestrator/src/tripwire.rs"
     reason = "tripwire detection code per companion §4; arxiv:1908.04734 current-RF"
     human_override_key = "L3_OVERRIDE:tripwire"

     [[locked_path]]
     path = ".elai_cc/crates/orchestrator/src/bai.rs"
     reason = "BAI scoring function per companion §4"
     ...

     [[locked_symbol]]
     symbol = "AllowedConfigKey"
     file = ".elai_cc/crates/orchestrator/src/config.rs"
     reason = "agent cannot add new variants per companion §4"

     [[locked_toml_key]]
     file = ".elai_cc/elai_cc.toml"
     section = "tripwire"
     keys = ["kl_spec_tau", "rho_delta_threshold", "iqr_outlier_multiplier",
             "ppl_anomaly_sigma", "ddm_warning_threshold", "ddm_drift_threshold",
             "consensus_fire_count", "window_size"]
     reason = "tripwire thresholds per arxiv:2507.05619"
     ```
   - Populate with all 14 surface items above + the per-file/per-symbol
     specifics from companion §4.

2. **Extend locked_keys_check.sh** (C-21 existing) with reasoning-surface
   paths:
   - Parse locked_keys_registry.toml; for each [[locked_path]], grep the
     commit diff via `git diff --cached --name-only`; if any locked path
     appears AND no L3_OVERRIDE:<key> commit message tag present → exit 1
     with structured error.
   - Similarly for [[locked_symbol]] (grep diff body for symbol).
   - Similarly for [[locked_toml_key]] (parse TOML diff for section.key
     changes).

3. **LockedKeysAuditor** in
   .elai_cc/crates/orchestrator/src/locked_keys_auditor.rs (new file):
   - `struct LockedKeysAuditor { registry: LockedKeysRegistry, ssb_tx: SsbSender, audit_id: Uuid }`
   - `struct LockedKeysRegistry { paths: Vec<LockedPath>, symbols: Vec<LockedSymbol>, toml_keys: Vec<LockedTomlKey> }`
   - `fn audit_diff(diff: &GitDiff) -> Vec<LockedKeysTouchEvent>`: parse diff;
     for each touched file/symbol/key, look up registry; if present and no
     override → emit event.
   - `fn audit_runtime_write(write: &PendingWrite) -> Option<LockedKeysTouchEvent>`:
     analogous for runtime write attempts (e.g. orchestrator-side
     SafeConfigEditor before disk).

4. **Runtime LOCKED_KEYS_TOUCH dispatch**:
   - Wire LockedKeysAuditor into orchestrator/src/task_runner.rs pre-Write
     hook (where Tool::Write is dispatched); if auditor returns event,
     BLOCK the write + emit LOCKED_KEYS_TOUCH SSB row.
   - Per CLAUDE.md hook-enforcement pattern: existing PreToolUse hook chain;
     D-9 adds LockedKeysAuditor as a hook entry.

5. **SSB action types** in ssb.rs (SCHEMA_VERSION 54→55):
   - `LockedKeysTouch { path: String, symbol: Option<String>, toml_section:
     Option<String>, toml_key: Option<String>, override_present: bool,
     audit_id: Uuid }`
   - `L3OverrideRequested { override_key: String, requester: String,
     justification: BoundedString<512> }` (when human operator commits with
     L3_OVERRIDE:* tag; informational log row, NOT a permission grant
     mechanism in code — only documents the human decision)
   - Round-trip tests.

6. **L3_OVERRIDE commit message convention**:
   - Format: `<commit subject line>` + body containing `L3_OVERRIDE:<key>:
     <justification>` block.
   - Example:
     ```
     update tripwire kl_spec_tau from 0.3 to 0.25

     L3_OVERRIDE:tripwire.kl_spec_tau:
       Empirical evidence from D-19 PRMBench shows kl_spec_tau=0.3 causes
       false positives on math-proof family. Lowering to 0.25 per
       arxiv:2507.05619 §5.3 acknowledged tuning band.
     ```
   - Pre-commit hook parses body; if L3_OVERRIDE present + valid registry
     key + ≥40 char justification → allow; otherwise BLOCK.

7. **Documentation update** in CLAUDE.md or docs/:
   - Add "L3 frozen surfaces" section listing the 14 surface items.
   - Reference locked_keys_registry.toml as authoritative.
   - Document L3_OVERRIDE protocol.
   - Reference Everitt arxiv:1908.04734 + Cohen Boxed Myopic + Hubinger
     arxiv:1906.01820 for grounding.

8. **Unit tests** in locked_keys_auditor.rs:
   - audit_diff on synthetic diff touching .elai_cc/crates/orchestrator/src/
     tripwire.rs without override → returns 1 LockedKeysTouch event.
   - audit_diff on same diff WITH L3_OVERRIDE:tripwire tag → returns 0 events
     OR returns 1 event with override_present=true.
   - audit_diff on diff touching elai_cc.toml [tripwire] kl_spec_tau → returns
     event with toml_section + toml_key populated.
   - audit_diff on diff touching unrelated path → returns 0 events.

9. **Integration test** in
   .elai_cc/crates/orchestrator/tests/locked_keys_integration.rs:
   - Create tmpdir git repo; stage commit touching a locked path; run
     locked_keys_check.sh hook; assert exit 1.
   - Same with L3_OVERRIDE tag; assert exit 0.

10. **CI workflow** at .elai_cc/.github/workflows/locked-keys-audit.yml:
    - Trigger: push + pull_request to main.
    - Steps: checkout → run scripts/locked_keys_check.sh against PR diff →
      exit non-zero on violation.
    - This is a GUARDRAIL: even if local hooks are bypassed, CI catches.

11. **Decision doc** workflow/reasoning/D-9-decisions.md (3 decisions):
    - (a) Why structural enforcement (not just documentation): documentation-
      only is bypassable; agent could quietly edit. Hook + auditor + CI
      makes the surface impractical to bypass.
    - (b) Why L3_OVERRIDE requires ≥40-char justification (not just any
      string): forces operator to articulate the why; prevents trivial "yes"
      bypasses; matches arxiv:1908.04734 current-RF principle that override
      is HUMAN decision with rationale.
    - (c) Why both pre-commit + runtime auditor: pre-commit catches dev-time
      edits; runtime catches orchestrator-side writes from a hypothetical
      future "L3-edit tool" the agent might try to invent.

## WIRING_CONTRACT

produces_symbols:
  - LockedKeysAuditor + LockedKeysRegistry + LockedPath + LockedSymbol +
    LockedTomlKey + LockedKeysTouchEvent
  - audit_diff + audit_runtime_write
  - SsbActionType::LockedKeysTouch + ::L3OverrideRequested + payloads
  - SCHEMA_VERSION bump 54→55
  - .elai_cc/scripts/locked_keys_registry.toml (canonical registry)
  - .elai_cc/.github/workflows/locked-keys-audit.yml CI step
  - L3_OVERRIDE commit message convention (documented in CLAUDE.md)
consumes_symbols:
  - C-21 existing locked_keys_check.sh script (extended here)
  - C-21 existing pre-commit hook chain (extended here)
  - All Wave-1 D-N files (the surfaces being locked exist)
  - GitDiff trait or git2 crate (verify in Cargo.toml; if absent, shell-out
    to git command per CLAUDE.md no-new-deps-without-confirmation)
  - SsbWriter
reachable_from:
  - .git/hooks/pre-commit local hook
  - .github/workflows/locked-keys-audit.yml CI
  - Orchestrator pre-write hook chain (PreToolUse)
stub_ban:
  - LockedKeysAuditor MUST NOT silently allow on registry parse error; fail
    closed (treat as locked if registry unreadable)
  - L3_OVERRIDE MUST require ≥40-char justification; shorter overrides BLOCK
  - audit_diff MUST cover paths + symbols + toml_keys (all three layers)
  - pre-commit hook MUST exit 1 on violation; exit 0 only when clean OR valid
    override
observable_effect:
  trigger: dev attempts commit touching tripwire.rs without L3_OVERRIDE tag
  expect_ssb: 1 LockedKeysTouch row; pre-commit hook exit 1; commit BLOCKED;
              orchestrator runtime if attempted via tool dispatch → same
  expect_artifact: .elai_cc/scripts/locked_keys_registry.toml present + parses;
                   locked-keys-audit.yml CI workflow present

## DETERMINISTIC_CHECK

```bash
# 1. SCHEMA_VERSION 55
rg "SCHEMA_VERSION: i64 = 55" .elai_cc/crates/orchestrator/src/ssb.rs

# 2. SSB variants
rg "LockedKeysTouch|L3OverrideRequested" .elai_cc/crates/orchestrator/src/ssb.rs

# 3. Registry file present + ≥14 locked items
test -f .elai_cc/scripts/locked_keys_registry.toml && \
  rg "^\[\[locked_(path|symbol|toml_key)\]\]" .elai_cc/scripts/locked_keys_registry.toml | wc -l
# Expected: >= 14

# 4. LockedKeysAuditor + locked_keys_check.sh
rg "struct LockedKeysAuditor|fn audit_diff" .elai_cc/crates/orchestrator/src/locked_keys_auditor.rs
test -f .elai_cc/scripts/locked_keys_check.sh

# 5. CI workflow
test -f .elai_cc/.github/workflows/locked-keys-audit.yml

# 6. Pre-commit hook updated
rg "locked_keys_check.sh" .elai_cc/.git/hooks/pre-commit 2>/dev/null || \
  rg "locked_keys_check.sh" .elai_cc/scripts/install-hooks.sh

# 7. CLAUDE.md or docs/ updated
rg "L3 frozen surfaces|L3_OVERRIDE" F:/.ELAI_workflow/CLAUDE.md \
  .elai_cc/docs/ 2>/dev/null

# 8. Build + tests
cargo check --workspace 2>&1 | tail -3
cargo test -p elai_orchestrator -- locked_keys 2>&1 | grep "test result"

# 9. Paper-ingestor outputs
test -f workflow/papers/everitt-reward-tampering-1908.04734-interface.md && \
  test -f workflow/papers/hubinger-mesa-1906.01820-interface.md && \
  test -f workflow/papers/cohen-boxed-myopic-aimag43-3-interface.md && echo OK
```

## ACCEPTANCE

- [ ] SCHEMA_VERSION_PUBLIC == 55
- [ ] LockedKeysTouch + L3OverrideRequested round-trip
- [ ] locked_keys_registry.toml present with ≥14 entries covering all 14 L3
      surface items from companion §2.3 + §4
- [ ] LockedKeysAuditor compiles; unit tests pass
- [ ] locked_keys_check.sh extended with reasoning-surface paths
- [ ] Pre-commit hook chain includes locked_keys_check.sh
- [ ] locked-keys-audit.yml CI workflow triggers on push + PR
- [ ] L3_OVERRIDE commit-message convention enforced (≥40-char justification)
- [ ] cargo check --workspace: zero new errors
- [ ] cargo test -p elai_orchestrator -- locked_keys: all pass, ≥8 new tests
- [ ] CLAUDE.md or docs/ documents L3 frozen surfaces + override protocol
- [ ] Cloud smoke test A: synthetic commit touching tripwire.rs without
      L3_OVERRIDE → pre-commit hook BLOCKS; exit 1; LockedKeysTouch SSB row
- [ ] Cloud smoke test B: same commit with valid L3_OVERRIDE tag → allowed;
      L3OverrideRequested SSB row logged
- [ ] Cloud smoke test C: orchestrator simulates Tool::Write to bai.rs via
      task_runner; LockedKeysAuditor BLOCKS pre-tool-use; LockedKeysTouch SSB
      row with override_present=false
- [ ] All 3 paper-ingestor outputs present
- [ ] Decision doc workflow/reasoning/D-9-decisions.md (3 decisions)

## Files touched

- ADD: .elai_cc/scripts/locked_keys_registry.toml
- ADD: .elai_cc/crates/orchestrator/src/locked_keys_auditor.rs
- ADD: .elai_cc/.github/workflows/locked-keys-audit.yml
- MODIFY: .elai_cc/scripts/locked_keys_check.sh (extend with reasoning paths)
- MODIFY: .elai_cc/scripts/install-hooks.sh (verify pre-commit chain order)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs
    (SCHEMA_VERSION 54→55; 2 new variants + payloads)
- MODIFY: .elai_cc/crates/orchestrator/src/task_runner.rs (LockedKeysAuditor
    as PreToolUse hook entry)
- MODIFY: .elai_cc/crates/orchestrator/src/lib.rs (pub mod locked_keys_auditor)
- ADD: .elai_cc/crates/orchestrator/tests/locked_keys_integration.rs
- MODIFY: F:/.ELAI_workflow/CLAUDE.md or .elai_cc/docs/L3-FROZEN-SURFACES.md
    (documentation)
- ADD: workflow/reasoning/D-9-decisions.md
- ADD: workflow/papers/{everitt-reward-tampering-1908.04734,hubinger-mesa-
       1906.01820,cohen-boxed-myopic-aimag43-3}-interface.md

## Out-of-scope

- D-25 (tamper test suite includes some LockedKeysTouch scenarios; this is
  D-9 + D-8 combined contract)
- Cryptographic signing of registry (deferred; relies on git commit signing)
- Multi-party human approval for L3 override (deferred; single-operator
  override sufficient for current threat model)

## Next

Wave 2 (D-10..D-18) begins. Wave 1 substrate complete: typed templates + PPM
+ consensus + tripwires + BAI + held-out + canary + archive verify + L3
enforcement.
