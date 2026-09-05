# C-27: Testing-Trust T(-1) Determinism + T-0 Wiring Contract

Objective: implement the two foundational tiers of the testing-trust stack —
Tier T(-1) DeterminismConfig (hardware-class lock, seeded RNG, blake3
prefix-cache keys, DETERMINISM_DRIFT SSB) and Tier T-0 WiringContract
(WiringContract struct, EntrypointSet, StubBanAuditor, DkgReachabilityAuditor,
ObservableEffectRunner, WIRING_EVIDENCE + WIRING_VIOLATION SSB types, `elai
wiring audit` CLI subcommand, CI workflow).
Estimated effort: 2-3 sessions.
Pre-requisites: C-15 (ClockProvider + entropy stable; SeededClock/SeededRng
  required by DeterminismConfig), C-17 (DKG resilience; DkgReachabilityAuditor
  walks the symbol graph produced by C-17).
Parallel-safe with: C-21..C-26 (non-overlapping crate scope; no shared mut on
  ssb.rs except the SCHEMA_VERSION bump, which is the sole serialisation point).

## Context

testing-trust-design.report.md §Mechanisms MISSING (2026-05-17) confirms:

Layer -1 (Inference Determinism): DeterminismConfig absent; ELAI_SEED seeds
SeededRng only (core/src/lib.rs:157) — no hardware-class lock, no blake3
prefix-cache keys, no system_fingerprint pin, no DETERMINISM_DRIFT SSB type.
Searched DeterminismConfig, hardware_class, nvml, DETERMINISM_DRIFT — no
evidence (report:47-55).

Layer 0 (Wiring Contract): WiringContract struct absent; StubBanEntry absent;
ObservableEffect/ObservableEffectRunner absent; EntrypointSet + [wiring.
entrypoints] in elai_cc.toml absent; WiringEvidence/WiringViolation SSB types
absent; ArchCodex DSL absent; WiringContractValidator absent (report:58-66).

Partial foundation present and usable:
- SeededClock (core/src/clock.rs:76) + SeededRng (core/src/entropy.rs) — C-15
  will stabilise; DeterminismConfig wraps both.
- DKG symbol graph (knowledge/src/symbol_graph/types.rs) — DkgReachabilityAuditor
  queries it.
- stub_ban as code-comment convention (~15 files) — StubBanAuditor operationalises.
- SCHEMA_VERSION 46 in ssb.rs:184 — C-27 bumps to 47.
- SsbActionType enum (ssb.rs) — extend with WiringEvidence, WiringViolation,
  DeterminismDrift variants.
- elai_cc.toml loaded by runtime_config.rs at startup — add [wiring.entrypoints].

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\guide\testing-trust-design.report.md
  (§Layer -1, §Layer 0 MISSING blocks — source of record for gap list)
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\guide\post-phase4-implementation-order.report.md
  (T(-1)/T-0 rows)
- .elai_cc/crates/orchestrator/src/ssb.rs (SCHEMA_VERSION bump region:50-184;
  SsbActionType enum for insertion point)
- .elai_cc/crates/core/src/clock.rs (SeededClock:76; ClockProvider trait:33)
- .elai_cc/crates/core/src/lib.rs (SeededRng export:99; TestContext:120)
- .elai_cc/crates/knowledge/src/symbol_graph/types.rs (DKG node/edge types
  consumed by DkgReachabilityAuditor)
- .elai_cc/elai_cc.toml (insertion point for [wiring.entrypoints])
- .elai_cc/.github/workflows/windows.yml (pattern for new CI step)
- .elai_cc/crates/cli/src/commands.rs (SlashCommand enum; wiring subcommand
  insertion point)

## Tasks

1. **SCHEMA_VERSION bump + SSB variants** in ssb.rs. Bump SCHEMA_VERSION 46->47.
   Add three SsbActionType variants with payloads and SsbPayload arms:
   - WiringEvidence { entrypoint: String, produces_symbols_verified: Vec<String>,
     reachable_count: u32, audit_id: Uuid }
   - WiringViolation { violation_kind: WiringViolationKind, file: String,
     line: u32, pattern: String, audit_id: Uuid }
   - DeterminismDrift { seed: u64, run_a_hash: String, run_b_hash: String,
     first_divergence_index: u32, audit_id: Uuid }
   Add WiringViolationKind enum { StubPattern, UnreachableSymbol, BrokenObservable }.
   Derive Debug/Clone/Serialize/Deserialize on all types. Verify uuid in Cargo.toml
   before use (already present via existing SSB payloads).

2. **DeterminismConfig** in crates/orchestrator/src/determinism.rs (new file).
   Fields: hardware_class: HardwareClass, seed: u64, blake3_cache_prefix: [u8; 8],
   system_fingerprint_pin: Option<String>.
   HardwareClass { Cpu, Gpu(GpuSku), Unknown }; GpuSku is a String newtype.
   from_env(): reads ELAI_SEED (default 0), ELAI_HARDWARE_CLASS (default Unknown);
   blake3_cache_prefix = first 8 bytes of blake3::hash(seed.to_le_bytes()) (verify
   blake3 in Cargo.toml before use — present since B-1).
   prefix_cache_key(prompt_hash: &[u8]) -> [u8; 32]: blake3::keyed_hash (pad
   blake3_cache_prefix to 32 bytes). matches_fingerprint: true when pin is None.
   DeterminismError: thiserror { EnvParseError(String), HashKeyError }. No unwrap.

3. **DeterminismDrift detector** in determinism.rs.
   ActionTypeSequence(Vec<String>) newtype. DriftComparator { seed, ssb_tx, audit_id }.
   compare(run_a, run_b): zip sequences; on first differing index emit DeterminismDrift
   SSB row with run_a_hash + run_b_hash (blake3 of joined action_types) + seed.
   Zero rows when sequences match. No unwrap; SSB via unbounded channel.

4. **WiringContract core types** in crates/orchestrator/src/wiring_contract.rs.
   StubBanEntry { pattern: String, description: String }.
   ObservableEffect { trigger, expect_ssb: Option<String>, expect_artifact: Option<String>,
   timeout_secs: u32 }.
   WiringContract { bn_id, produces_symbols, consumes_symbols, reachable_from,
   stub_ban, frontend_pair: Option<String>, observable_effects }.
   WiringContractError: thiserror { ParseError, IoError(#[from] io::Error), RegexError }.
   All types: Derive Debug/Clone/Serialize/Deserialize.

5. **EntrypointSet + EntrypointSetLoader** in wiring_contract.rs.
   EntrypointSet { entrypoints: Vec<String> }. EntrypointSetLoader { config_path }.
   load(): read elai_cc.toml (std::fs::read_to_string); parse TOML (verify toml in
   Cargo.toml before use; present in runtime_config.rs); extract [wiring.entrypoints]
   symbols array; missing section -> empty EntrypointSet + tracing::warn!, no panic.
   Add comment template to elai_cc.toml: `# [wiring.entrypoints]\n# symbols = [...]`.

6. **StubBanAuditor** in wiring_contract.rs. Fields: patterns: Vec<(Regex,
   StubBanEntry)>, ssb_tx, audit_id. new(): compile each pattern via regex::Regex::new
   (verify regex in Cargo.toml before use — grep existing codebase first).
   scan_file(path, contents): per-line pattern test; emit WiringViolation(StubPattern)
   with file + line on match. scan_production_paths(root): walkdir (verify in
   Cargo.toml); exclude paths containing "/tests/", "_test.rs", "_contract.rs";
   collect all scan_file results. Default forbidden patterns: todo!(), unimplemented!(),
   "deferred", "stub", Ok(Default::default()).

7. **DkgReachabilityAuditor** in wiring_contract.rs. Fields: dkg: Arc<SymbolGraph>,
   ssb_tx, audit_id. is_reachable(entrypoints, symbol): BFS from each entrypoint
   node; short-circuit on first hit. audit_contract(contract, entrypoints): for each
   produces_symbol — reachable -> WiringEvidence; unreachable -> WiringViolation
   (UnreachableSymbol). Use knowledge crate's existing DKG types; no duplication.

8. **ObservableEffectRunner** in wiring_contract.rs. Fields: ssb_db_path, timeout_secs,
   ssb_tx, audit_id. run_effect(effect, bn_id): if expect_ssb Some — poll rusqlite
   SSB DB (verify rusqlite in Cargo.toml; present in ssb.rs persistence path) at 1s
   interval via tokio::time::timeout; timeout -> WiringViolation(BrokenObservable) +
   Err. if expect_artifact Some — check path::exists() after deadline; absent ->
   WiringViolation. No panic; propagate via ?.

9. **`elai wiring audit` CLI subcommand** in crates/cli/src/wiring_audit.rs.
   WiringAuditCmd { config_path, contracts_dir, ssb_db_path, seed }.
   run(): load EntrypointSet; glob contracts_dir *.md (verify globset in Cargo.toml;
   present per workspace); parse WIRING_CONTRACT markdown blocks for produces_symbols /
   stub_ban / observable_effects; run StubBanAuditor + DkgReachabilityAuditor +
   ObservableEffectRunner; aggregate WiringAuditReport { evidence, violations, audit_id };
   print "WIRING AUDIT PASS" or "WIRING AUDIT FAIL (N violations)" with file:line;
   exit 0 / exit 1. Wire SlashCommand::WiringAudit in commands.rs; expose as
   `elai wiring audit [--config PATH] [--contracts PATH]` in main.rs.

10. **CI workflow** .github/workflows/wiring-audit.yml. Steps: checkout -> cargo
    build -p elai_cli (release) -> `elai wiring audit --config elai_cc.toml
    --contracts .elai_cc/crates/orchestrator/tests/` -> exit non-zero. Trigger:
    push + pull_request to main. Runner: windows-latest (matches windows.yml pattern).
    Skip if [ci skip] in commit message.

11. **Unit tests** (#[cfg(test)] in determinism.rs + wiring_contract.rs):
    prefix_cache_key stable across calls with same seed; DriftComparator emits row
    on mismatch, zero on identical; StubBanAuditor detects todo!() in synthetic content,
    skips test paths; EntrypointSetLoader returns empty (no panic) on missing section;
    ObservableEffectRunner emits BrokenObservable on timeout (mock SSB DB, no row).

12. **Contract test** crates/contracts/tests/wiring_contract_tier0_contract.rs (or
    orchestrator/tests/ — grep existing contract test locations before writing):
    WiringEvidence + WiringViolation in SsbActionType; SCHEMA_VERSION_PUBLIC == 47;
    DeterminismConfig::from_env() ok with ELAI_SEED=42; DriftComparator zero rows
    for identical sequences.

13. **Decision doc** workflow/reasoning/C-27-decisions.md: (a) blake3 keyed_hash for
    cache keys (CLAUDE.md mandate; server.py sha256 is sidecar HTTP boundary, not
    production Rust); (b) BFS over DFS in reachability (bounded depth on large DKG);
    (c) 1s poll over event-driven in ObservableEffectRunner (SQLite has no NOTIFY;
    polling is deterministic and unit-testable).

## WIRING_CONTRACT

produces_symbols:
  - DeterminismConfig + HardwareClass + GpuSku + DeterminismError
  - DriftComparator + ActionTypeSequence
  - WiringContract + WiringContractError
  - StubBanEntry + StubBanAuditor
  - ObservableEffect + ObservableEffectRunner
  - EntrypointSet + EntrypointSetLoader
  - DkgReachabilityAuditor
  - WiringAuditCmd + WiringAuditReport + WiringAuditError
  - SsbActionType::WiringEvidence + ::WiringViolation + ::DeterminismDrift + payloads
  - WiringViolationKind enum
  - SlashCommand::WiringAudit
  - SCHEMA_VERSION bump 46->47
consumes_symbols:
  - SeededClock (core/src/clock.rs:76) + SeededRng (core/src/lib.rs:99) — C-15
  - DKG SymbolGraph (knowledge/src/symbol_graph/types.rs) — C-17
  - SsbActionType enum (orchestrator/src/ssb.rs) — extended here
  - rusqlite SSB DB (already used by ssb.rs persistence path)
  - runtime_config.rs elai_cc.toml loader — EntrypointSetLoader reads same file
  - SlashCommand enum (cli/src/commands.rs) — gains WiringAudit variant
reachable_from:
  - elai_cc.toml [wiring.entrypoints] (closed entrypoint list)
  - CLI `elai wiring audit` subcommand
  - .github/workflows/wiring-audit.yml CI step
  - DeterminismConfig::from_env() (reads ELAI_SEED, ELAI_HARDWARE_CLASS)
stub_ban:
  - DeterminismConfig::prefix_cache_key MUST use blake3, never sha256 or md5
  - StubBanAuditor::scan_production_paths MUST exclude tests/ and _contract files
  - DriftComparator::compare MUST emit zero SSB rows when sequences are identical
  - ObservableEffectRunner MUST emit WiringViolation (not panic) on timeout
  - EntrypointSetLoader::load MUST NOT panic on missing [wiring.entrypoints] section
  - wiring audit exit code MUST be 0 on zero violations, 1 on any violation
observable_effect:
  trigger: `elai wiring audit` after wiring lands
  expect_ssb: WiringEvidence rows appear for each configured entrypoint (zero
              violations for core orchestrator); manually introduce `Ok(Default::
              default())` in non-test path, rerun — >=1 WiringViolation row with
              file:line; run `elai /task --mode cloud "simple add function"` twice
              with ELAI_SEED=42, assert DriftComparator emits zero DeterminismDrift
              rows; submit broken ObservableEffect (nonexistent SSB action_type),
              assert WiringViolation kind=BrokenObservable within timeout+5s
  expect_artifact: wiring-audit.yml CI step exits 0 on clean repo

## DETERMINISTIC_CHECK

```bash
# 1. SCHEMA_VERSION bumped to 47
rg "SCHEMA_VERSION: i64 = 47" .elai_cc/crates/orchestrator/src/ssb.rs

# 2. New SSB variants present
rg "WiringEvidence|WiringViolation|DeterminismDrift" \
  .elai_cc/crates/orchestrator/src/ssb.rs

# 3. WiringViolationKind enum
rg "enum WiringViolationKind" .elai_cc/crates/orchestrator/src/ssb.rs

# 4. DeterminismConfig in determinism.rs
rg "struct DeterminismConfig|fn from_env|fn prefix_cache_key" \
  .elai_cc/crates/orchestrator/src/determinism.rs

# 5. blake3 use (not sha256)
rg "blake3" .elai_cc/crates/orchestrator/src/determinism.rs

# 6. WiringContract core types
rg "struct WiringContract|struct StubBanEntry|struct ObservableEffect|struct EntrypointSet" \
  .elai_cc/crates/orchestrator/src/wiring_contract.rs

# 7. StubBanAuditor excludes test paths
rg "tests/|_contract\|_test\.rs" .elai_cc/crates/orchestrator/src/wiring_contract.rs

# 8. DkgReachabilityAuditor BFS
rg "fn is_reachable|fn audit_contract" \
  .elai_cc/crates/orchestrator/src/wiring_contract.rs

# 9. CLI subcommand registered
rg "WiringAudit|wiring.audit" .elai_cc/crates/cli/src/commands.rs

# 10. CI workflow file present
powershell -c "Test-Path .elai_cc/.github/workflows/wiring-audit.yml"

# 11. Build + scoped tests
cargo check --workspace 2>&1 | tail -3
cargo test -p elai_orchestrator -- determinism wiring_contract 2>&1 | grep "test result"
cargo test -p elai_cc_contracts -- wiring_contract_tier0 2>&1 | grep "test result"

# 12. DriftComparator zero-drift assertion (unit test)
cargo test -p elai_orchestrator -- drift_comparator_no_drift 2>&1 | grep "PASSED\|ok"
```

## ACCEPTANCE

- [ ] SCHEMA_VERSION_PUBLIC == 47; old SSB DB with version 46 emits migration warn
- [ ] WiringEvidence, WiringViolation, DeterminismDrift in SsbActionType; payloads
      serialise/deserialise (round-trip test passes)
- [ ] WiringViolationKind { StubPattern, UnreachableSymbol, BrokenObservable }
- [ ] DeterminismConfig::from_env() succeeds with ELAI_SEED=42; blake3 keyed_hash
      used for prefix_cache_key (grep confirms, not sha256)
- [ ] DriftComparator: identical sequences -> zero SSB rows; divergent sequences ->
      one DeterminismDrift row with correct first_divergence_index
- [ ] StubBanAuditor: detects `todo!()`, `unimplemented!()`, `Ok(Default::default())`
      in synthetic non-test content; skips test/ paths
- [ ] EntrypointSetLoader: returns empty EntrypointSet with tracing::warn! when
      [wiring.entrypoints] absent; no panic
- [ ] DkgReachabilityAuditor: emits WiringEvidence for reachable symbols; emits
      WiringViolation(UnreachableSymbol) for missing symbols
- [ ] ObservableEffectRunner: emits WiringViolation(BrokenObservable) on timeout;
      does NOT panic; returns Err
- [ ] `elai wiring audit` exits 0 on clean orchestrator; exits 1 after manual
      stub injection; violation output includes file:line
- [ ] Cloud smoke test A: `elai wiring audit` after wiring — WiringEvidence rows
      present for each entrypoint; zero WiringViolation for core orchestrator
- [ ] Cloud smoke test B: inject `Ok(Default::default())` in non-test path; rerun
      audit; >=1 WiringViolation row with file:line matching injection site
- [ ] Cloud smoke test C: two runs of `elai /task --mode cloud "simple add function"`
      with ELAI_SEED=42; DriftComparator emits zero DeterminismDrift rows
- [ ] Cloud smoke test D: broken ObservableEffect (nonexistent action_type); runner
      timeouts; WiringViolation(BrokenObservable) row emitted within timeout+5s
- [ ] wiring-audit.yml CI step present; triggers on push + pull_request to main
- [ ] cargo check --workspace: zero new errors
- [ ] cargo test -p elai_orchestrator -- determinism wiring_contract: all pass,
      >=10 new test functions
- [ ] cargo test -p elai_cc_contracts -- wiring_contract_tier0: all pass
- [ ] Decision doc workflow/reasoning/C-27-decisions.md (3 decisions)

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/determinism.rs
    (DeterminismConfig, HardwareClass, GpuSku, DeterminismError,
     DriftComparator, ActionTypeSequence)
- ADD: .elai_cc/crates/orchestrator/src/wiring_contract.rs
    (WiringContract, StubBanEntry, StubBanAuditor, ObservableEffect,
     ObservableEffectRunner, EntrypointSet, EntrypointSetLoader,
     DkgReachabilityAuditor, WiringAuditCmd, WiringAuditReport, WiringAuditError)
- ADD: .elai_cc/crates/cli/src/wiring_audit.rs
    (CLI dispatch layer for wiring_audit subcommand)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs
    (SCHEMA_VERSION 46->47; WiringEvidence, WiringViolation, DeterminismDrift
     SsbActionType variants + payloads; WiringViolationKind enum)
- MODIFY: .elai_cc/crates/cli/src/commands.rs
    (SlashCommand::WiringAudit variant + dispatch)
- MODIFY: .elai_cc/elai_cc.toml
    (add [wiring.entrypoints] comment template block)
- ADD: .elai_cc/.github/workflows/wiring-audit.yml
- ADD: .elai_cc/crates/contracts/tests/wiring_contract_tier0_contract.rs
    (or crates/orchestrator/tests/; confirm location by grepping existing
     contract test files before writing)
- ADD: workflow/reasoning/C-27-decisions.md

## Next

C-28 (Testing-Trust T-1 Intent Formalization) or T-2 Property/Metamorphic Tests
depending on prioritization. T(-1) and T-0 land together in this session as they
share the SSB schema bump.
