Phase: B-16D magic numbers → named consts (Chunk 1)
Session: s-1327164
Date: 2026-04-29

What
  Convert production-code magic-number literals to named consts in the SMALLEST crate
  identified by context-compiler. HANDOFF estimate "<200 sites" was acknowledged inflated;
  context-compiler re-baselined per-crate; warnings show several "magic" hits are inline-test
  false positives (web_validator: 4 raw → 1 prod; plugins: 26 raw → ~10-12 prod).

Chunking strategy
  Pick the SMALLEST production hit count chunk, capped at 30 sites for one phase.
  Execution agent reads context-package-phase4.md, picks the smallest chunk, justifies pick
  in execution report. Converts all qualifying magic numbers to named const at module/file
  level (preferred) or fn scope (only if locality dominates).

Naming convention
  - SCREAMING_SNAKE_CASE
  - File-local: `const FOO_BAR_MS: u64 = 5000;` near top of file
  - Crate-shared: only if multiple files use the same value; otherwise file-local
  - Doc-comment one-liner explaining what the value means and unit (e.g., `/// HTTP request timeout in milliseconds`)
  - Type explicit (no inferred i32) — match the use site

Skipped patterns
  - 0, 1, -1, 2 in indexing/sentinel context
  - Hex literals already next to an obvious named-bit operation
  - Test-only literals (#[cfg(test)] blocks)
  - Const initializers themselves (already named)
  - #[derive] / #[cfg] attribute literals

Expected test outcome (NOTE: cargo deferred this round due to RAM constraint)
  Source-level audit only this round.
  Auditor verifies: (a) no remaining unnamed magic-numbers in chunk's files (sample-check),
  (b) named consts are typed and doc-commented, (c) no semantic change.
  cargo check --workspace deferred until RAM ≥20GB free; will be batched in a later audit.

Unknowns
  - Whether smallest crate is web_validator (1 hit — barely a phase) or plugins (10-12 hits).
    Execution agent decides per context-package guidance.
  - Whether some "magic" hits are actually documented constants in the wild (e.g., HTTP 200,
    port 80) — execution agent applies judgment, documents skips in report.

Out of scope
  - Renaming consts that already exist (only NEW names for unnamed literals)
  - Cross-crate const-shared modules (file-local only this chunk)
