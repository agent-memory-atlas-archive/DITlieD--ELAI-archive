B-15 Tranche P Item P.2 -- providers crate .unwrap()/.expect() burndown NULL-RESULT
Session: s-22600 sub-phase trancheP | Date: 2026-06-01

Verdict: NULL-RESULT (no source edits). Providers crate segment of B15-unwrap-burndown is VERIFIED-COMPLIANT.

Scope examined
- .elai_cc/crates/providers/src/ across all .unwrap() and .expect() sites.

DI6 scope summary (per context-package-trancheP.md)
- providers .unwrap: raw=62  / prod=0 / exempt=0 / migration_target=0
- providers .expect: raw=159 / prod=1 / exempt=1 / migration_target=0
- combined:          raw=221 / prod=1 / exempt=1 / migration_target=0

Prod .unwrap sites
- ZERO real prod .unwrap sites. Three raw matches are doc-comment false positives:
    runtime/composer2.rs L29:                 `//!   - .unwrap()/.expect() restricted to test code only.`
    runtime/sidecar_xgrammar_hook.rs L25:     `//!   no .unwrap() outside #[cfg(test)])`
    runtime/sidecar_xgrammar_hook.rs L93:     `/// snake_case label ... No .unwrap()`
- All other 59 .unwrap raw matches sit inside #[cfg(test)] modules (line# > cutoff per file).

Prod .expect sites (1 site, already case-3 exempt)
- providers/anthropic.rs L674: `last_error.expect("retry loop must capture an error")`
  - Post-retry-loop fallback. The `for attempt in 0..max_attempts` loop guarantees `last_error` is `Some` when the loop exits via "retries exhausted" path (loop body always assigns last_error before continuing).
  - `#[allow(unused_assignments)]` on the `let mut last_error: Option<ApiError> = None;` init confirms author's deliberate over-init for compiler well-formedness.
  - Documented invariant. Exempt under CLAUDE.md case-3 (provably-valid-by-loop-invariant).
- L610 raw match is a comment "// init exists so the post-loop last_error.expect(..) is well-defined" (false positive).

Outcome
- Providers crate B-15 .unwrap/.expect substrate VERIFIED-COMPLIANT.
- Zero production-side migration targets (the single prod .expect site is already documented case-3 exempt).
- No source edits this tranche.
