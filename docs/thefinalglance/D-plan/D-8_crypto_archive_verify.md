# D-8: Cryptographic archive + `elai archive verify` CLI

Objective: surface ArchiveChain hash-chain integrity check (from D-1
ArchiveChain.verify_chain) as a user-facing CLI subcommand + CI workflow;
emit ArchiveVerifyResult SSB rows; harden against lobotomy-attack where agent
rewrites its own audit history. Per arxiv:2507.05619 (T1) cryptographic-
checksum recommendation + DGM §5 archive practice (companion §3.8 lines
319-337).
Estimated effort: 1-2 sessions.
Pre-requisites: D-1 (ArchiveChain.verify_chain exists). spec-grounder verifies.
Parallel-safe with: D-2..D-7 (D-8 only READS ArchiveChain; does not extend
  schema beyond verify reporting).

## Context

Companion §3.8 (lines 319-337): every mutation candidate gets blake3 content
hash of (template_body, parent_template_hash, mutation_proposal_text). Archive
append-only, indexed by hash. Prevents lobotomy attack. Hash chain
independently verifiable.

D-1 already provides ArchiveEntry { self_hash, parent_hash, template,
staged_eval_result, promoted_at, retired_at, retirement_reason } + ArchiveChain.
verify_chain() method. D-8 wraps these in user-facing CLI + CI integration.

The verify_chain walk: for each ArchiveEntry, recompute
blake3(template_body || parent_hash || mutation_proposal_text); compare to
self_hash field; check parent_hash references an existing prior entry; report
BrokenLink / HashMismatch / DanglingParent errors.

DGM §5 archive practice: lineage exposes diff. Without lineage, DGM's
hallucination-marker-removal incident (§H) would have gone undetected.

Partial foundation present and usable:
- D-1 ArchiveChain.verify_chain method.
- C-27 `elai wiring audit` CLI pattern (model for new subcommand structure).
- C-27 wiring-audit.yml CI workflow pattern (model for archive-verify CI).

## Reads

- F:\.ELAI_workflow\docs\specs\guide\reasoning-improvement-grounded-2026-05-19.md
  (§3.8 archive design — lines 319-337)
- F:\.ELAI_workflow\workflow\papers\dgm-interface.md (§5 archive practice)
- F:\.ELAI_workflow\workflow\papers\reward-hacking-detection-2507.05619-interface.md (D-4)
- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-27_testing_trust_t0_wiring_contract.md
  (CLI + CI pattern model)
- .elai_cc/crates/orchestrator/src/archive_chain.rs (D-1 verify_chain)
- .elai_cc/crates/cli/src/wiring_audit.rs (C-27 CLI pattern)
- .elai_cc/.github/workflows/wiring-audit.yml (C-27 CI pattern)

## Tasks

1. **ArchiveVerifyCmd** in new file
   .elai_cc/crates/cli/src/archive_verify.rs:
   - `struct ArchiveVerifyCmd { archive_path: PathBuf, json_output: bool,
     fail_fast: bool }`
   - `struct ArchiveVerifyReport { total_entries: u32, verified: u32, errors:
     Vec<ArchiveVerifyError> }`
   - `enum ArchiveVerifyError { HashMismatch { entry_index: u32, expected:
     String, computed: String }, BrokenParent { entry_index: u32, parent_hash:
     String }, DuplicateHash { entry_index: u32, hash: String, prior_index:
     u32 }, MalformedJson { entry_index: u32, error: String } }`
   - `fn run(&self) -> Result<ExitCode, ArchiveVerifyError>`:
     - Load ArchiveChain from archive_path.
     - Call ArchiveChain.verify_chain (D-1).
     - Aggregate errors; if !fail_fast continue collecting; if fail_fast
       short-circuit on first.
     - Print human-readable report or JSON if json_output.
     - Exit 0 on zero errors; exit 1 otherwise.
     - Emit ArchiveVerifyResult SSB row regardless of exit code.

2. **CLI dispatch** in
   .elai_cc/crates/cli/src/commands.rs:
   - Add `SlashCommand::ArchiveVerify` variant.
   - Wire dispatch in main.rs: `elai archive verify [--path PATH]
     [--json] [--fail-fast]`.
   - Default path: .elai_cc/archive/reasoning-templates.jsonl (matches D-1
     default).

3. **SSB action type** in ssb.rs (SCHEMA_VERSION 53→54):
   - `ArchiveVerifyResult { audit_id: Uuid, total_entries: u32, verified:
     u32, error_count: u32, errors: Vec<String> }`
   - Round-trip test.

4. **CI workflow** at
   .elai_cc/.github/workflows/archive-verify.yml:
   - Trigger: push + pull_request to main; nightly schedule (cron).
   - Steps: checkout → cargo build -p elai_cli (release) → `elai archive verify
     --path .elai_cc/archive/reasoning-templates.jsonl --fail-fast` → exit
     non-zero on integrity violation.
   - Runner: windows-latest (matches existing CI pattern); also matrix
     ubuntu-latest for cross-platform smoke.
   - Skip if `[ci skip]` in commit message.

5. **Tamper-detection contract test** in
   .elai_cc/crates/contracts/tests/archive_verify_contract.rs:
   - Create synthetic ArchiveChain with 5 entries in tmpdir.
   - Tamper scenario A: modify template_body of entry 3 in JSONL on disk →
     ArchiveVerifyCmd.run reports HashMismatch on entry 3; exit 1.
   - Tamper scenario B: change parent_hash of entry 4 to reference a non-
     existent hash → BrokenParent; exit 1.
   - Tamper scenario C: duplicate self_hash across entries 2 and 5 →
     DuplicateHash; exit 1.
   - Tamper scenario D: malformed JSON on a single line → MalformedJson +
     ability to continue if !fail_fast.
   - Clean scenario: untampered archive → exit 0; ArchiveVerifyResult row
     with error_count=0.

6. **Performance test** in
   .elai_cc/crates/orchestrator/tests/archive_verify_perf.rs (cfg-gated to
   ignored by default; manual run):
   - Synthesize 10K-entry ArchiveChain.
   - verify_chain runtime ≤ 5 seconds wall-clock on dev hardware (anti-DoS
     bound; D-1 implementation must be O(n) not O(n²)).

7. **Documentation update** in .elai_cc/README.md or docs/:
   - Document `elai archive verify` CLI usage.
   - Document CI integration (archive-verify.yml).
   - Document tamper-recovery procedure: if verify fails, the agent's last
     N actions are suspect; rollback via ArchiveChain forensic walk.

8. **Decision doc** workflow/reasoning/D-8-decisions.md (3 decisions):
    - (a) Why CLI + CI separately (not just CLI): CI catches drift in shared
      repo; CLI is dev-side debugging; both serve.
    - (b) Why blake3 (not sha256 or sha3): matches D-1 + C-27 + CLAUDE.md HARD
      RULE; blake3 is faster on commodity hardware (~5GB/s vs ~1GB/s sha256
      on AVX2) so 10K-entry walk fits the 5s budget.
    - (c) Why fail-fast vs continue: dev mode wants full report; CI mode wants
      early-exit. Both supported via --fail-fast flag.

## WIRING_CONTRACT

produces_symbols:
  - ArchiveVerifyCmd + ArchiveVerifyReport + ArchiveVerifyError
  - SlashCommand::ArchiveVerify variant + dispatch
  - SsbActionType::ArchiveVerifyResult + payload
  - SCHEMA_VERSION bump 53→54
  - CI workflow .github/workflows/archive-verify.yml
consumes_symbols:
  - ArchiveChain (D-1) + verify_chain method
  - SlashCommand enum (existing)
  - SsbWriter (existing)
reachable_from:
  - `elai archive verify` CLI subcommand
  - .github/workflows/archive-verify.yml CI step
  - manual `cargo run --bin elai -- archive verify` during dev
stub_ban:
  - ArchiveVerifyCmd MUST NOT use unwrap on IO; thiserror enum
  - On clean archive MUST exit 0 (defends against test pass-rate inflation
    from "always exit 1")
  - On tampered archive MUST exit 1 (defends against silent acceptance)
  - --fail-fast and --json flags MUST be functional (not stub)
observable_effect:
  trigger: `elai archive verify` on clean repo vs after manual tamper
  expect_ssb: 1 ArchiveVerifyResult row per run (clean: error_count=0; tampered:
              error_count>=1)
  expect_artifact: archive-verify.yml CI workflow exists; runs on PR; reports
                   pass/fail on commit status

## DETERMINISTIC_CHECK

```bash
# 1. SCHEMA_VERSION 54
rg "SCHEMA_VERSION: i64 = 54" .elai_cc/crates/orchestrator/src/ssb.rs

# 2. CLI subcommand
rg "ArchiveVerify\|archive.verify" .elai_cc/crates/cli/src/commands.rs

# 3. Archive verify types
rg "struct ArchiveVerifyCmd|struct ArchiveVerifyReport|enum ArchiveVerifyError" \
  .elai_cc/crates/cli/src/archive_verify.rs

# 4. CI workflow file
test -f .elai_cc/.github/workflows/archive-verify.yml && echo OK

# 5. SSB variant
rg "ArchiveVerifyResult" .elai_cc/crates/orchestrator/src/ssb.rs

# 6. Clean-archive smoke
test -f .elai_cc/archive/reasoning-templates.jsonl && \
  cargo run --bin elai -- archive verify --fail-fast && echo CLEAN-OK

# 7. Tamper-then-verify (synthetic; should exit 1)
cp .elai_cc/archive/reasoning-templates.jsonl /tmp/_clean_archive.jsonl
echo '{"corrupt":"line"}' >> .elai_cc/archive/reasoning-templates.jsonl
cargo run --bin elai -- archive verify --fail-fast && echo "ERROR: should have failed" || echo TAMPER-DETECT-OK
mv /tmp/_clean_archive.jsonl .elai_cc/archive/reasoning-templates.jsonl

# 8. Build + tests
cargo check --workspace 2>&1 | tail -3
cargo test -p elai_cc_contracts -- archive_verify 2>&1 | grep "test result"

# 9. Performance test (manual; ignored by default)
cargo test -p elai_orchestrator --test archive_verify_perf -- --ignored 2>&1
```

## ACCEPTANCE

- [ ] SCHEMA_VERSION_PUBLIC == 54
- [ ] ArchiveVerifyResult SSB variant + payload round-trip
- [ ] ArchiveVerifyCmd compiles; `elai archive verify` subcommand dispatch
- [ ] 5 tamper scenarios (HashMismatch, BrokenParent, DuplicateHash,
      MalformedJson, clean) tested in contract test; all 4 tamper scenarios
      detected with exit 1
- [ ] Clean repo: exit 0; ArchiveVerifyResult row with error_count=0
- [ ] --json flag emits parseable JSON (jq round-trip)
- [ ] --fail-fast vs default: full-collect mode reports ≥2 errors when ≥2
      tampers present
- [ ] archive-verify.yml CI workflow present; triggers on push + PR + nightly
- [ ] Performance test (10K entries, ignored by default): verify_chain ≤ 5s
- [ ] cargo check --workspace: zero new errors
- [ ] cargo test -p elai_cc_contracts -- archive_verify: all pass
- [ ] Cloud smoke test A: live cloud /task with template promotion;
      `elai archive verify` exit 0 after promotion (clean integrity)
- [ ] Cloud smoke test B: manual `sed -i` on one JSONL line to corrupt
      template_body; `elai archive verify` exit 1 with HashMismatch error
- [ ] Documentation in README.md or docs/ explains tamper-recovery procedure
- [ ] Decision doc workflow/reasoning/D-8-decisions.md (3 decisions)

## Files touched

- ADD: .elai_cc/crates/cli/src/archive_verify.rs
- MODIFY: .elai_cc/crates/cli/src/commands.rs (SlashCommand::ArchiveVerify)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs
    (SCHEMA_VERSION 53→54; ArchiveVerifyResult variant)
- ADD: .elai_cc/.github/workflows/archive-verify.yml
- ADD: .elai_cc/crates/contracts/tests/archive_verify_contract.rs
- ADD: .elai_cc/crates/orchestrator/tests/archive_verify_perf.rs (cfg ignore)
- ADD: workflow/reasoning/D-8-decisions.md
- MODIFY: .elai_cc/README.md or docs/ (tamper-recovery section)

## Out-of-scope

- D-25 (full 10-scenario tamper test suite as benchmark)
- D-9 (LockedKeysAuditor enforcing locked-key access; orthogonal mechanism)
- Archive compaction / sled migration (deferred; JSONL scales to ~10K entries
  cleanly per D-1 design)

## Next

D-9 (L3 expansion + frozen surfaces) is the final substrate piece. After D-9,
Wave 1 substrate is complete and Wave 2 (real-problem-solving modes) can begin.
