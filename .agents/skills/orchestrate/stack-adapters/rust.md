STACK ADAPTER — rust

Loaded by orchestrate when detect-stack.ps1 reports stack="rust" or when stack starts with "rust+" (the multi-stack case loads this adapter AND cross-stack-rust-ts.md).

Tools required pod-side / dev-machine-side (the launched session verifies presence on first sweep):
- cargo (workspace command, ships with toolchain)
- cargo-modules (https://github.com/regexident/cargo-modules) — symbol-level graph + orphans + acyclic check
- cargo-machete (https://crates.io/crates/cargo-machete) — unused-dep audit, stable toolchain
- cargo-public-api (https://github.com/cargo-public-api/cargo-public-api) — public-API surface tracking (nightly toolchain required)
- cargo-nextest (optional, faster test runner if installed; fallback to cargo test)
- ripgrep (rg) for grep-based delta-coverage detection

Install commands (orchestrate-launcher.ps1 surfaces these to user if missing; does NOT auto-install):
  cargo install cargo-modules
  cargo install cargo-machete
  cargo install cargo-public-api  # needs nightly: rustup toolchain install nightly
  cargo install cargo-nextest

THE 6 GENERIC GATES, RUST FLAVOR

GATE 1 — BUILD GREEN
  cargo build --workspace --all-features --release
  Expected: exit 0
  Per sub-phase: scope to the crate(s) touched: cargo build -p <crate> --all-features
  Full sweep: --workspace

GATE 2 — TEST SUITE GREEN + DELTA-COVERED
  cargo test --workspace --all-features  (or cargo nextest run --workspace if installed)
  Expected: exit 0, no skipped or ignored without explicit justification
  Delta coverage: for each new public symbol in WIRING_CONTRACT.produces_symbols, grep test files for the symbol name:
    rg "<symbol_short_name>" crates/*/tests/ crates/*/src/**/tests.rs
    Expected: at least 1 match per symbol
  Missing matches surface in the report as "uncovered new public symbol: <symbol>"

GATE 3 — DEP-GRAPH INTEGRITY
  cargo modules dependencies --workspace --acyclic
  Expected: exit 0 (acyclic check fails on any cycle)
  cargo modules orphans --workspace
  Expected: empty list (no unreachable modules)

GATE 4 — PUBLIC-API SURFACE DIFF
  cargo public-api --workspace --diff-git-checkouts HEAD~1 HEAD
  (requires nightly toolchain; orchestrate-launcher.ps1 sets RUSTUP_TOOLCHAIN=nightly for this command)
  Output: list of added / removed / changed public items
  For each ADDED item: check if it appears in WIRING_CONTRACT.produces_symbols OR has at least one consumer in the codebase:
    rg "<added_item_short_name>" --type rust
  Items with NO match and NOT in produces_symbols = "intentional new surface unregistered" = block

GATE 5 — MANIFEST + LOCKFILE CONSISTENCY
  cargo metadata --format-version 1 --no-deps --locked
  Expected: exit 0 (the --locked flag fails if Cargo.lock would need updating)
  cargo machete
  Expected: zero "unused dependencies" lines OR each unused dep is explicitly listed in a known-OK list in compat.machete_allowlist
  (note: machete may false-flag macro-only deps; cargo-udeps is more accurate but nightly + slow; default to machete and overrride via compat allowlist when needed)

GATE 6 — DOCUMENTATION DRIFT
  For each new public symbol in WIRING_CONTRACT.produces_symbols:
    rg "<symbol_short_name>" workflow/memory-bank/**/changelog.md workflow/memory-bank/**/systemPatterns.md workflow/memory-bank/**/projectIndex.md
  Expected: at least 1 match per symbol (the memory-writer agent should have added the entry)

RUST-SPECIFIC EXTRA GATES (beyond the generic 6)

GATE R1 — STUB BAN (CLAUDE.md rule for ELAI + this rule applies anywhere Rust)
  rg "\.unwrap\(\)" crates/*/src --type rust --glob "!**/tests/**" --glob "!**/test_*.rs"
  Expected: zero matches (force-unwrap in non-test source is BLOCK per ELAI CLAUDE.md hard rule)
  rg "todo!\(\)|unimplemented!\(\)|unreachable!\(\)" crates/*/src --type rust --glob "!**/tests/**"
  Expected: every match must reference a registered stub id (TODO[X.Y.Z] embedded in the macro arg OR a stubs.md row pointing at this line)

GATE R2 — STAGE-BUILD CHECK (catches "compiles but emits warnings critical to logic")
  cargo build --workspace --all-features --release 2>&1 | rg "^warning:"
  Expected: zero matches OR each warning is listed in compat.clippy_allowlist
  cargo clippy --workspace --all-features -- -D warnings
  Expected: exit 0

GATE R3 — DETERMINISTIC OUTPUT (where the plan's WIRING_CONTRACT.stub_ban specifies deterministic ordering, e.g. B-30 CS namespace map)
  For each row in plan.deterministic_check that involves a determinism assertion:
    run the test 5x, hash the stdout
  Expected: identical hashes across runs

GATE R4 — TREE-SITTER GRAMMAR VERSION CHECK (B-30 specific, generalizable)
  If plan touches crates/*/src/**/language/*.rs (tree-sitter grammar files):
    cargo tree --workspace --invert tree-sitter
  Expected: single tree-sitter version pinned at workspace level (no multiple versions)


PER-SUB-PHASE SCOPING

Each sub-phase touches a subset of crates. Phase-end sweep at sub-phase boundary scopes gates to:
  - touched_crates = union of plan.sub_phase.files mapped to their containing crate
  - cargo build -p <crate> for each touched_crate
  - cargo test -p <crate> for each touched_crate
  - GATE 3 / 4 / 5 still run at workspace level (cheap enough)
  - GATE 6 still runs at workspace level

This keeps per-sub-phase sweep wall-clock under 2 min on a typical workspace. Full sweep at end runs all gates at full workspace.


COMMAND TEMPLATES (orchestrate-launcher.ps1 fills $WORKSPACE_ROOT and $TOUCHED_CRATES)

per-sub-phase:
  Push-Location $WORKSPACE_ROOT
  $crates = $TOUCHED_CRATES -split ','
  foreach ($c in $crates) {
    cargo build -p $c --all-features --release; if ($LASTEXITCODE -ne 0) { exit 1 }
    cargo test -p $c --all-features; if ($LASTEXITCODE -ne 0) { exit 2 }
  }
  cargo modules dependencies --workspace --acyclic; if ($LASTEXITCODE -ne 0) { exit 3 }
  cargo modules orphans --workspace; if ($LASTEXITCODE -ne 0) { exit 4 }
  Pop-Location

full sweep:
  Push-Location $WORKSPACE_ROOT
  cargo build --workspace --all-features --release; if ($LASTEXITCODE -ne 0) { exit 1 }
  cargo test --workspace --all-features; if ($LASTEXITCODE -ne 0) { exit 2 }
  cargo modules dependencies --workspace --acyclic; if ($LASTEXITCODE -ne 0) { exit 3 }
  cargo modules orphans --workspace; if ($LASTEXITCODE -ne 0) { exit 4 }
  $env:RUSTUP_TOOLCHAIN = 'nightly'
  cargo public-api --workspace --diff-git-checkouts HEAD~1 HEAD > $sweepdir/public-api-diff.txt 2>&1
  $env:RUSTUP_TOOLCHAIN = $null
  cargo metadata --format-version 1 --no-deps --locked > $null; if ($LASTEXITCODE -ne 0) { exit 5 }
  cargo machete; if ($LASTEXITCODE -ne 0) { exit 6 }
  cargo clippy --workspace --all-features -- -D warnings; if ($LASTEXITCODE -ne 0) { exit 7 }
  Pop-Location

  # stub-ban grep (PowerShell)
  $unwrap_hits = Get-ChildItem $WORKSPACE_ROOT\crates\*\src -Recurse -Include *.rs | Select-String -Pattern '\.unwrap\(\)' -Exclude '**/tests/**','**/test_*.rs'
  if ($unwrap_hits) { exit 8 }


REPORTING

Each gate result writes to workflow/sessions/<sid>/phase-reports/<sub_phase_id>.md per templates/phase-report.md "GATES" section.

Failures include:
- gate name + exit code
- captured stdout/stderr (truncated to 200 lines)
- recommended next action: rerun / fix / bubble-up

END stack-adapters/rust.md
