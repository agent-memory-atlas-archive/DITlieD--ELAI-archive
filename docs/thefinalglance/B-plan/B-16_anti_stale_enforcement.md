# B-16: Anti-Stale Rule Enforcement
Objective: enforce CLAUDE.md Anti-Stale-Training hard rules across the
codebase: ort stable (not -rc), thiserror over Box<dyn Error> in 15+
public APIs, blake3 spec-text update, .unwrap/.expect removal, .try_into()
over as-casts.
Estimated effort: 1-2 sessions.
Pre-requisites: none.
Parallel-safe with: all other B-N.

REBASELINE NOTE (s-25120, 2026-05-22)
=====================================
Rebaselined against SHIPPED v2 state across B-16A/B/D/E sub-phases. Markers:
  - SHIPPED-ON-V2          = task closed in prior sub-phase, evidence at file:line
  - SHIPPED-ON-DIFFERENT-SURFACE = closed but on a different artifact than originally planned
  - PENDING-UPSTREAM       = blocked on external constraint (analogous to RUNPOD-PENDING)
  - ORT-PENDING            = bump to ort 2.0.0 when crates.io ships stable
Closing decisions in workflow/reasoning/B-16-decisions.md (Session s-25120 block).
Regression doc in workflow/reasoning/B-16-followup.md.

## Context

Per audit-implementation/_AGGREGATE.md "ANTI-STALENESS RULE VIOLATIONS":
- ort = "=2.0.0-rc.12" (knowledge/Cargo.toml:54) — should be stable
  [PENDING-UPSTREAM: ort 2.0.0 stable not on crates.io as of 2026-05-22]
- Box<dyn std::error::Error> in 15+ public APIs across knowledge crate
  [SHIPPED-ON-V2 via B-16A: KnowledgeError enum at crates/knowledge/src/error.rs:11]
- .expect("semaphore closed") in reranker.rs:465 (production hot path) —
  fixed in B-4 then re-verified in B-16A [SHIPPED-ON-V2: reranker.rs:606 graceful match]
- 84 std::collections::HashMap imports — covered by B-15 [SHIPPED-ON-V2 by B-15]
- Phase 3C audit confirmed blake3 vs spec's blake2s — DELIBERATE divergence
  per CLAUDE.md; spec text updated [SHIPPED-ON-V2: phase-3C.txt L399-401, L443]

CLAUDE.md hard rules (Rust):
- blake3 (never blake2/sha2 for non-RFC uses)            [SHIPPED-ON-V2 via B-16A]
- rand>=0.9 (never 0.8)                                  [DEFERRED to B-16-lint-extension]
- getrandom>=0.3                                         [DEFERRED to B-16-lint-extension]
- tracing (never log crate)                              [SHIPPED-ON-V2: zero use log:: hits]
- thiserror/anyhow (never Box<dyn Error>)                [SHIPPED-ON-V2 via B-16A]
- tokio::sync::Mutex in async (sync-only otherwise)      [DEFERRED to B-16-lint-extension]
- Native async fn in traits (never async-trait crate)    [DEFERRED to B-16-lint-extension]
- ort stable (never -rc)                                 [PENDING-UPSTREAM]
- deadpool/bb8 over r2d2                                 [DEFERRED to B-16-lint-extension]
- globset/ignore over glob                               [DEFERRED to B-16-lint-extension]
- Named constants (no magic numbers)                     [SHIPPED-ON-DIFFERENT-SURFACE: plugins T2 AST lint]
- .try_into() over as-casts for narrowing                [SHIPPED-ON-V2: B-16E categorized + B-16D Chunk 2]

## Reads

- F:\.ELAI_workflow\CLAUDE.md Anti-Stale-Training section (authoritative)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\error.rs (KnowledgeError enum)
- F:\.ELAI_workflow\.elai_cc\scripts\check_anti_stale.sh (current CI lint surface)
- F:\.ELAI_workflow\.elai_cc\crates\plugins\src\tiered_analysis.rs L1454-1545 (T2 magic-number AST lint)
- F:\.ELAI_workflow\workflow\reasoning\B-16-decisions.md (cross-session decisions)
- F:\.ELAI_workflow\workflow\reasoning\B-16A-decisions.md (ort waiver DECISION-A)
- F:\.ELAI_workflow\workflow\reasoning\B-16B-decisions.md (.unwrap chunks A+B+D + D5 LazyLock exemption)
- F:\.ELAI_workflow\workflow\reasoning\B-16E-decisions.md (cast scan classification)
- F:\.ELAI_workflow\workflow\reasoning\B-16-followup.md (regression + deferred items)

REBASELINE NOTE: original Reads cited audit-implementation/_AGGREGATE.md and
.elai_cc/outdated-tech-audit.md — both files MISSING (grounding sweep s-25120).
Reads section above replaces them with extant authoritative sources.

## Tasks

### ort stable

1. **Find ort stable version.** [PENDING-UPSTREAM: bump to 2.0.0 when crates.io
   ships stable]
   ORT-PENDING. WebSearch 2026-05-09 (s-91480) + non-change-since (s-25120):
   ort 2.0.0 stable STILL not published; latest rc.12 (2026-03-05). Waiver
   doc workflow/reasoning/B-16A-decisions.md DECISION-A active; lint
   ORT_FAIL=0 while waiver file present. Bump trigger checklist in
   B-16-followup.md (2).

2. **Update knowledge/Cargo.toml.** [PENDING-UPSTREAM]
   ORT-PENDING. Cannot bump until Task 1 resolves. Pins remain at
   knowledge/Cargo.toml:54 and orchestrator/Cargo.toml:62 = "=2.0.0-rc.12".

### thiserror

3. **Define KnowledgeError enum.** [SHIPPED-ON-V2]
   Evidence: crates/knowledge/src/error.rs:11 `pub enum KnowledgeError` with
   12 thiserror variants and #[from] bridges. Per-module narrower at
   crates/knowledge/src/vector/mod.rs:14 `VectorError` for the EmbedderProvider
   trait boundary. Decision: unified KnowledgeError + per-trait narrower
   (rather than per-module split as the plan originally suggested) — ergonomics
   win, all observed source types bridged via #[from]. See B-16A-decisions.md.

4. **Replace 8 Box<dyn Error> in bm25.rs.** [SHIPPED-ON-V2]
   Evidence: crates/knowledge/src/bm25.rs imports KnowledgeError at L11; all
   8 public methods (new/index_file/index_directory/query/delete_file/
   update_file + internal commit + chunk_file) now return
   Result<_, KnowledgeError>. Original line cites (L155, 217, 243, 262, 314,
   329, 362, 403) are SUPERSEDED — those line numbers no longer exist
   post-refactor. Grep "Box<dyn.*Error" bm25.rs returns zero hits.

5. **Replace 5 Box<dyn Error> in vector/indexer.rs.** [SHIPPED-ON-V2]
   Evidence: crates/knowledge/src/vector/indexer.rs imports KnowledgeError at
   L8; 7 pub fn/method sites converted (load, save, index_file, search,
   search_code, plus chunk_file free fn). Original line cites (L57, 71, 89,
   194, 218) are SUPERSEDED. Grep returns zero Box<dyn ...Error> hits in
   vector/ (only Box<dyn Fn> closures at vector/mod.rs:156-157, which are
   legitimate non-error trait objects).

6. **Replace 1 Box<dyn Error> in pagerank/mod.rs.** [SHIPPED-ON-V2]
   Evidence: crates/knowledge/src/pagerank/mod.rs imports KnowledgeError at
   L5; DkgSnapshot::from_engine returns Result<_, KnowledgeError>. Original
   L28 cite SUPERSEDED. Grep returns zero hits.

7. **Replace any remaining Box<dyn Error> in knowledge crate.** [SHIPPED-ON-V2]
   Evidence: full crate sweep shows only error.rs:6 (doc comment referring to
   pre-B-16A historical state) and Pin<Box<dyn Future>>/Box<dyn Fn> non-error
   trait objects. The 1A scope mention is closed.

### .unwrap/.expect

8. **Audit production .unwrap() and .expect() calls.** [SHIPPED-ON-V2]
   Evidence: B-16B Chunks A+D+B shipped (Mutex registries + LazyLock annots +
   mcp_bridge/cli/lane_events; ~67 sites converted). Chunk C ~12 deferred
   for "signature cascade" — INVESTIGATED s-25120: all candidate sites
   ALREADY converted to LazyLock<Regex>::new(|| regex::Regex::new(LITERAL).unwrap())
   static-init blocks with #[allow(clippy::unwrap_used)] + CLAUDE-md-EXCEPTION
   comment per B-16B D5 (which is the CLAUDE.md-approved exception, NOT a
   violation). See B-16-decisions.md Session s-25120 DECISION-B for
   per-site evidence table.
   Lint raw count (1589) includes test/bench/static-exempt sites by design;
   per-file-cfg-test-aware audit confirms ZERO unjustified production unwraps.

9. **Verify reranker.rs:465 .expect("semaphore closed") fixed.** [SHIPPED-ON-V2]
   Evidence: crates/knowledge/src/reranker.rs:606 — comment confirms prior
   .expect removed in B-16A; current code uses graceful match block. The
   string "semaphore closed" no longer appears as .expect() body.

### Tracing not log

10. **Grep for `use log::`.** [SHIPPED-ON-V2]
    Evidence: grep "use log::" across .elai_cc/crates returns zero hits.
    Lint STRICT mode enforces no regression.

### Named constants

11. **Audit magic numbers.** [SHIPPED-ON-DIFFERENT-SURFACE]
    SHIPPED via plugins T2 AST lint (not via scripts/check_anti_stale.sh as
    originally planned). Evidence:
      - crates/plugins/src/tiered_analysis.rs:1458 check_magic_numbers_rust
      - crates/plugins/src/tiered_analysis.rs:1471 walk_magic_numbers_rust
      - crates/plugins/src/tiered_analysis.rs:1506 check_magic_numbers_python
      - crates/plugins/src/tiered_analysis.rs:1519 walk_magic_numbers_python
    Allow-list: {0, 1, 2, -1, 0.0, 1.0}; ALLOW context: const/static decls +
    test files. tree-sitter AST scope-aware, exempts const/static blocks
    deterministically (bash grep cannot). Also: web_validator/src/lib.rs:514
    fn constants_are_named_not_magic (web-domain test).
    s-25120 added REPORT echo block to scripts/check_anti_stale.sh + .bat
    citing the T2 AST lint surface for CI visibility.

### .try_into over as-casts

12. **Audit `as` narrowing casts.** [SHIPPED-ON-V2]
    Evidence: scripts/check_anti_stale.sh CAT-A/B/C/D split per B-16E
    (categorisation + HIGH-risk file exclusions). 15 production cast sites
    addressed in s-91480 (NARROWING-OK annotations + CAT-C/D try_from
    replacements). CAT-D residual = 0. CAT-C residual = 35 (per s-25120
    re-scan; mostly dashboard-v2 commands out of B-16 scope + degradation.rs
    test-only). Migration policy is REPORT-only per CLAUDE.md "per-site
    judgment" rule — blanket find-and-replace would regress.

### Spec amendment

13a. **Update spec text re: blake3 in phase-3C.txt.** [SHIPPED-ON-V2]
     Evidence: docs/specs/guide/steps/phase-3C.txt:399-401 + :443 reference
     blake3 with explicit note "previously the project specced blake2s —
     superseded by blake3 per Anti-Stale-Training rule, B-16A 2026-04-26".
     Original Task 13 cited stale line range; current is L399-401, L443.

13b. **Spec amendment anti-staleness-enforcement.txt.** [SHIPPED-ON-V2]
     Evidence: docs/specs/guide/anti-staleness-enforcement.txt EXISTS (at
     docs/specs/guide/, not docs/specs/guide/steps/ as the original cross-
     check searched). File references blake3 throughout at L14, 28, 182,
     303, 367, 375, 743-744 including explicit "DO NOT USE blake2. Use
     blake3" and "project itself migrated to blake3".

### Anti-stale CI lint

14. **Add CI lint script.** [SHIPPED-ON-V2]
    Evidence: scripts/check_anti_stale.sh + scripts/check_anti_stale.bat
    both present, exit 0 STRICT clean after s-25120 cosmetic fix at
    media_validator/types.rs:539 (comment rephrase to drop banned literal).
    STRICT modes: Box<dyn .*Error, use log::, ort = "...-rc..." (waived
    via DECISION-A presence check). REPORT modes: .unwrap(), as-cast
    CAT-A/B/C/D, magic-numbers (cites T2 AST lint surface).
    Companion CI wire at .github/workflows/windows.yml:71-76.

### Future-lint extension (DEFERRED to a future B-N)

15. **Extend lint to cover remaining CLAUDE.md rules.** [DEFERRED]
    Out-of-scope for this session (no scope creep). See B-16-followup.md (3).
    Coverage gaps: rand>=0.9, getrandom>=0.3, async_trait import ban,
    deadpool/bb8 over r2d2, globset/ignore over glob, tokio::sync::Mutex
    async-context vs std::sync::Mutex sync-only (needs AST).
    Owner: a future B-16-lint-extension plan.

## WIRING_CONTRACT

produces_symbols:
  - crates/knowledge/src/error.rs::KnowledgeError
  - crates/knowledge/src/vector/mod.rs::VectorError
  - scripts/check_anti_stale.sh
consumes_symbols:
  - all knowledge crate callers (signature changes from Box<dyn Error> to KnowledgeError)
  - all production .unwrap/.expect callers (per-site exemption or conversion)
reachable_from:
  - cargo build --workspace
  - bash scripts/check_anti_stale.sh (CI wire)
stub_ban:
  - Zero Box<dyn Error> in non-test public APIs                       [STRICT-enforced]
  - Zero .unwrap/.expect in non-acceptable production code             [per-file-cfg-test-aware; LazyLock-static-init exemption per B-16B D5]
  - Zero `as` narrowing casts                                          [per-site judgment per B-16E categorisation; CAT-D=0]
  - Zero `use log::`                                                   [STRICT-enforced]
  - ort stable, not -rc                                                [PENDING-UPSTREAM; waiver via B-16A DECISION-A]
frontend_pair: null
observable_effect:
  trigger: cd .elai_cc && bash scripts/check_anti_stale.sh
  expect_ssb: plan_state_derived
  expect_artifact: exit 0; STRICT clean; REPORT counts informational

## DETERMINISTIC_CHECK

```bash
# 1. ort -rc (waived until 2.0.0 stable ships on crates.io)
grep "^ort\\s*=" .elai_cc/crates/knowledge/Cargo.toml
# Current: ort = "=2.0.0-rc.12" (PENDING-UPSTREAM; waiver active)

# 2. No Box<dyn Error> in knowledge public APIs
rg "Box<dyn.*Error" .elai_cc/crates/knowledge/src \
   --glob '!*test*' --glob '!*tests*'
# Expected: zero hits.

# 3. No .unwrap in production hot paths (per-file-cfg-test-aware)
# scripts/check_anti_stale.sh REPORT count is raw; per-file audit:
#   - skip first .unwrap() AFTER first #[cfg(test)] line per file
#   - skip LazyLock<Regex>::new(...).unwrap() static-init exemption
#   - skip benches/*.rs (test-equivalent)
# Expected: zero unjustified prod hits.

# 4. No log::
rg "use log::" .elai_cc/crates --type rust
# Expected: zero hits.

# 5. CI lint clean
bash .elai_cc/scripts/check_anti_stale.sh
# Expected: exit 0 STRICT clean.
```

## ACCEPTANCE

- [PENDING-UPSTREAM] ort stable version (waived via B-16A DECISION-A)
- [MET] Zero Box<dyn Error> in knowledge crate public APIs
- [MET] Zero unjustified .unwrap/.expect in production (per-file-cfg-test-aware audit)
- [MET] Zero `use log::`
- [MET] All `as` narrowing casts replaced with .try_into() or per-site categorised (B-16E)
- [MET] Spec text updated re: blake3 (matches impl) — phase-3C.txt + anti-staleness-enforcement.txt
- [MET] CI anti-stale lint script in place (check_anti_stale.sh + .bat + CI wire)
- [MET] All tests still pass after refactor

## Files touched

- MODIFY: crates/knowledge/Cargo.toml (ort version)                                       [DEFERRED: PENDING-UPSTREAM]
- ADD:    crates/knowledge/src/error.rs (KnowledgeError unified + VectorError trait)      [SHIPPED via B-16A]
- MODIFY: ~30+ files in knowledge crate (signature changes)                                [SHIPPED via B-16A]
- MODIFY: ~50+ files for .unwrap removals + as → .try_into                                [SHIPPED via B-16B/D/E]
- MODIFY: spec files phase-3C.txt + anti-staleness-enforcement.txt                        [SHIPPED via B-16A]
- ADD:    scripts/check_anti_stale.sh + .bat                                              [SHIPPED + s-25120 magic-number REPORT block]
- ADD:    CI step running the lint (.github/workflows/windows.yml:71-76)                  [SHIPPED]
- MODIFY: crates/orchestrator/src/media_validator/types.rs:539 comment                    [SHIPPED s-25120 cosmetic]
- CREATE: workflow/reasoning/B-16-decisions.md (+ B-16A/B/D/E sub-session docs)           [SHIPPED]
- CREATE: workflow/reasoning/B-16-followup.md (regression + deferred items)               [SHIPPED s-25120]

## Next

B-17 (Ground the vacuous gates).
