# Verification Report: B-16_anti_stale_enforcement.md
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-16_anti_stale_enforcement.md
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary

B-16 anti-stale enforcement is substantially complete across its core objectives.
The knowledge crate KnowledgeError thiserror enum replaced all 14 Box<dyn Error>
production sites in bm25.rs, vector/indexer.rs, and pagerank/mod.rs. The CI lint
scripts (check_anti_stale.sh + .bat) exist and report STRICT-clean as of 2026-05-09.
Two gaps remain: (1) ort is still pinned at =2.0.0-rc.12 under an active waiver
(stable not published on crates.io as of 2026-05-09), and (2) the lint script is
present locally but not wired into any GitHub Actions CI workflow file.

## Claims verified DONE

- KnowledgeError thiserror enum defined (replaces per-module Bm25Error/VectorError/PagerankError design) — evidence: .elai_cc/crates/knowledge/src/error.rs:10 (#[derive(Debug, thiserror::Error)])

- 8 Box<dyn Error> in bm25.rs replaced — evidence: zero grep hits for "Box<dyn.*Error" in bm25.rs; B-16A-decisions.md:148-155 lists all 8 migrated sites

- 5 Box<dyn Error> in vector/indexer.rs replaced — evidence: zero grep hits in vector/indexer.rs; B-16A-decisions.md:156-160 lists all 5 migrated sites

- 1 Box<dyn Error> in pagerank/mod.rs replaced — evidence: zero grep hits in pagerank/mod.rs; B-16A-decisions.md:161 confirms migration

- reranker.rs:465 .expect("semaphore closed") fixed — evidence: .elai_cc/crates/knowledge/src/reranker.rs:464-474 uses match+Ok/Err with tracing::warn!; B-16A-decisions.md:37-55 confirms DECISION-B

- error.rs created with typed variants (Tantivy, Json, SchemaMismatch, Index) — evidence: .elai_cc/crates/knowledge/src/error.rs:1-61

- CI lint script check_anti_stale.sh created — evidence: .elai_cc/scripts/check_anti_stale.sh:1-273 (STRICT + REPORT-ONLY modes, B-16E category split)

- CI lint script check_anti_stale.bat created (Windows sibling) — evidence: .elai_cc/scripts/check_anti_stale.bat:1-154

- Zero use log:: in any production Rust file — evidence: grep across .elai_cc/crates/ returns no hits; B-16-decisions.md:49 confirms lint EXIT 0

- Blake3 used in knowledge crate (never blake2) — evidence: .elai_cc/crates/knowledge/Cargo.toml:39 (blake3 = "1.8"); no blake2 in crates/ source

- Spec text updated re: blake3 — evidence: docs/specs/guide/steps/phase-3C.txt:399,401 and docs/specs/guide/anti-staleness-enforcement.txt:13-14,27-28 updated; B-16A-decisions.md:DECISION-E documents lines amended vs preserved

- ort stable waiver documented — evidence: workflow/reasoning/B-16A-decisions.md:DECISION-A; re-verified 2026-05-09 (rc.12 still latest); lint script checks for waiver file presence before failing

- .unwrap() audit executed (B-16B) — evidence: workflow/reasoning/B-16B-decisions.md records Chunks A+D+B converted ~67 production hits; Chunk C deferred by explicit decision

- Narrowing as-cast audit executed (B-16E) — evidence: workflow/reasoning/B-16E-decisions.md; B-16-decisions.md:11-33 lists 15 production cast sites addressed (CAT-A annotated, CAT-C/D replaced); check_anti_stale.sh splits into 4 named cast categories per B-16E

- rand >= 0.9 used (not 0.8) — evidence: .elai_cc/crates/core/Cargo.toml:13 (rand = "0.9"); no rand 0.8 in any Cargo.toml

- getrandom >= 0.3 used — evidence: .elai_cc/crates/agent/Cargo.toml:13, orchestrator/Cargo.toml:9, providers/Cargo.toml:15 (all getrandom = "0.3")

- No async-trait crate dependency — evidence: grep for async.trait/async_trait in all Cargo.toml returns no hits

- No r2d2 or bare glob crate — evidence: grep returns no hits across all crates/Cargo.toml files

- workflow/reasoning/B-16A-decisions.md created — evidence: file exists, 170 lines with 6 DECISION blocks

- workflow/reasoning/B-16B-decisions.md created — evidence: file exists with chunk plan and key decisions

- workflow/reasoning/B-16E-decisions.md created — evidence: file exists with chunk plan and cast patterns

## Claims PARTIAL

- Zero unjustified .unwrap()/.expect() in production (B-16B Chunk C deferred)
  - exists: .elai_cc/crates/knowledge/src/reranker.rs:779,781 (.unwrap() in project_root() helper, confirmed non-test but covered by DECISION-B rationale in B-16A-decisions.md:57-61); Chunks A+D+B completed per B-16B-decisions.md
  - gap: Chunk C (deterministic_auditor + dkg + misc, ~12 hits with signature cascade) explicitly deferred to next session per B-16B-decisions.md:24; user approval required before proceeding; B-16-decisions.md:50 shows REPORT count still at 1387 (though many are test/LazyLock exempt)

- All narrowing as-casts replaced with .try_into() or justified (B-16E HIGH follow-up open)
  - exists: B-16E Chunks 1-4 completed; CAT-C residual=0, CAT-D residual=0 per B-16-decisions.md:52-53; turboquant.rs/sandbox.rs/context_budget.rs excluded from cast scans with documented rationale
  - gap: self_discovery.rs (crates/rules/src/) — f64->u8 score clamping at L247/258/266 flagged as B-16E-HIGH follow-up in B-16E-decisions.md:18; still unresolved; dashboard-v2 casts (commands.rs:49,556,3479) declared out of scope but not annotated

- ort stable version (never -rc)
  - exists: .elai_cc/crates/knowledge/Cargo.toml:46 (ort = "=2.0.0-rc.12"); waiver documented in B-16A-decisions.md:DECISION-A; re-verified 2026-05-09 (crates.io confirms stable not published)
  - gap: rule not satisfied — rc.12 pin retained; waiver is time-limited by crates.io publication; bump trigger steps documented but not yet actionable

## Claims MISSING

- ADD CI step running the anti-stale lint — searched: .github/workflows/*.yml for "anti_stale", "anti-stale", "check_anti"; no evidence; neither windows.yml nor gate-liveness.yml references check_anti_stale.sh or .bat

## Claims UNVERIFIABLE

- "84 std::collections::HashMap imports — covered by B-15" — reason: HashMap ban is B-15 scope; not verified here; check_hashmap_ban.sh IS wired in windows.yml but that is B-15's concern

- Magic number audit completeness (Task 11) — reason: B-16-decisions.md:5-8 records one specific T2_CIRCUIT_BREAKER_LINES const added; full audit coverage across all crates cannot be confirmed from decision docs alone; no dedicated "named constants" check script exists

- Spec files phase-3C.txt and anti-staleness-enforcement.txt amend correctness — reason: spec files are gitignored per CLAUDE.md project boundary; amendments are local-only and cannot be version-controlled or diff-verified from codebase state

## What must be done (action list)

- [ ] Wire check_anti_stale.sh into a GitHub Actions workflow (e.g., add step to .elai_cc/.github/workflows/windows.yml or create a new anti-stale.yml); currently scripts/check_anti_stale.sh exists but has no CI trigger
- [ ] Execute B-16B Chunk C: ~12 production .unwrap() sites in deterministic_auditor + dkg + misc that require signature cascade changes; requires explicit user approval before proceeding (per B-16B-decisions.md:24)
- [ ] Resolve B-16E-HIGH follow-up: self_discovery.rs L247/258/266 (f64->u8 score clamping); verify runtime bounds then convert or annotate NARROWING-OK
- [ ] Monitor crates.io for ort 2.0.0 stable publication; when published, execute bump trigger in B-16A-decisions.md:DECISION-A steps 1-5 and delete waiver

## Notes

The sha2 = "0.10" in crates/core/Cargo.toml:12 may appear to violate the "never sha2" rule, but
core/src/project_id.rs:12 contains a comment explicitly noting the anti-stale rule and sha2 is
referenced in a gitignored-doc context. This should be audited if sha2 is used for non-RFC purposes
in core crate production paths.
The reranker.rs project_root() helper at L776-783 uses two .unwrap() calls on PathBuf::parent().
Per CLAUDE.md these are boundary panics (always fails at startup if wrong) but the function is not
main/entrypoint; annotating with CLAUDE-md-EXCEPTION or converting to expect("invalid crate layout")
would improve clarity.
