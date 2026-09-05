B-16E — Decision Doc
=====================
Phase: B-16E (as-cast narrowing → try_into per CLAUDE.md hard rule)
Session: s-27276 (registry s-34160)

WHAT
----
Replace narrowing `as`-casts with `.try_into()` patterns in production .rs files.
Re-baselined: ~175 actual narrowing casts (NOT 422 — many of the original count were
pointer / widening / enum-discriminant-indexing, which are acceptable per spec).

CHUNK 1 SCOPE (this phase only):
  cli/render.rs, tools/, plugins/, reasoning/, ~12 actual fixes across ~25 files
  (most touched files are skippable — widening or already-correct).

DEFERRED HIGH-risk (NOT in scope; logged as B-16E-HIGH follow-up):
  - crates/providers/src/turboquant.rs — 27 hits, f64→f32; std lacks TryFrom<f64> for f32.
    Mark with `// NARROWING-OK: f64→f32 has no TryFrom; quantization math accepts
    precision loss as design`.
  - crates/orchestrator/src/sandbox.rs — HRESULT i32→u32 reinterpretation + Win32 HANDLE
    pointer casts; intentional FFI conversions.
  - crates/rules/src/self_discovery.rs — f64→u8 score clamping at lines 247/258/266;
    fixable but needs runtime-bounds verification first.

ALSO DEFERRED:
  - context_budget.rs ~40 ContextSlot/AdaptiveLevel `as usize` for enum discriminant
    indexing — structurally safe per #[repr(u8)] + const assert at line 640. DO NOT replace.

KEY DECISIONS
-------------
D1. Chunk 1 only this phase. Subsequent chunks (orchestrator, knowledge, reasoning) for
    next session.
D2. Pattern: `value as TargetType` →
      Result fn: `value.try_into()?`
      Option chain: `value.try_into().ok()?`
      Bounded init: `u32::try_from(value).expect("rationale")`
      Cap on overflow: `u32::try_from(value).unwrap_or(u32::MAX)`
      (matches existing precedent at task_runner.rs:5017)
D3. Lint script tightening: add `--glob '!**/turboquant.rs' --glob '!**/sandbox.rs'`
    exclusions to check_anti_stale.sh; split int-narrow vs f64→usize vs u128→u64
    categories. Tighten in this phase.

EXPECTED TEST OUTCOMES
----------------------
- cargo check --workspace PASS.
- cargo test --no-run PASS.
- check_anti_stale.sh narrowing count drops by ~12.

UNKNOWNS
--------
- Whether some Chunk 1 sites need an additional Ord/Hash derive on a key type. Mitigation:
  agent applies trivial derives if compile breaks; logs as DEVIATION.

ROLLBACK
--------
git restore on touched files.

ACCEPTANCE
----------
A1: cargo check --workspace PASS.
A2: cargo test --no-run touched crates PASS.
A3: ~12 narrowing casts replaced.
A4: Lint script tightened with exclusions for HIGH-risk files.
A5: B-16E-HIGH follow-up logged (turboquant.rs, sandbox.rs, self_discovery.rs).


B-16E CONTINUATION — Chunks 2-4 + HIGH (s-330056)
==================================================
Phase: B-16E continuation
Session: s-330056 (continuation of s-27276)
FSM: PLANNING -> TESTING_FIRST -> EXECUTING (sequential 4-agent execution)

WHAT
----
Convert remaining ~163 narrowing as-casts (post Chunk 1) to try_into / try_from per
CLAUDE.md HARD RULE. Re-baselined via context-compiler enumeration, not the s-27276 estimate.

ENUMERATED SITE COUNT (from context-package-B-16E.md):
  elai-knowledge:    ~89 NARROWING, ~33 WIDENING (26 files)
  elai-orchestrator: ~75 NARROWING, ~50 WIDENING, ~18 FFI/skip (25 files)
  elai-reasoning:    1 narrowing (b'A'+i as u8 — annotate-only ASCII-bounded)
  HIGH-risk:         sandbox 2 real + 10 FFI/skip; self_discovery 6; turboquant 0 real
                     (all f64->f32 algorithmic, annotate-only)

CHUNK PLAN (sequential, not parallel — shared knowledge crate Cargo.toml)
  Agent 1: Chunk 2+3 combined  -> knowledge crate    ~62 sites in ~10 files
  Agent 2: Chunk 4a            -> orch first half    ~40 sites in ~9 files
  Agent 3: Chunk 4b            -> orch second half   ~35 sites in ~9 files
  Agent 4: HIGH chunk          -> sandbox / self_discovery / turboquant + annotations

PATH CORRECTIONS (vs HANDOFF.md):
  turboquant.rs is in crates/providers/src/   (not orchestrator/)
  self_discovery.rs is in crates/rules/src/    (not orchestrator/)

CONVERSION PATTERNS (canonical, locked):
  expr as Tnarrow (production fallible)  -> Tnarrow::try_from(expr).map_err(|e| anyhow!(...))?
  expr as Tnarrow (cap on overflow)      -> Tnarrow::try_from(expr).unwrap_or(Tnarrow::MAX)
  expr as Tnarrow (#[cfg(test)]/main)    -> Tnarrow::try_from(expr).expect("rationale")
  intentional FFI / bit-twiddle           -> annotate `// NARROWING-OK: <reason>`
  ASCII-bounded counter (b'A'+i as u8)    -> annotate-only
  ORT tensor shape (count -> i64)         -> annotate, established pattern in reranker.rs

KEY DECISIONS
-------------
D1. Sequential not parallel — shared crate Cargo.toml + import graph cascade risk.
D2. Each agent runs `cargo check --workspace` after its chunk and stops if it fails.
D3. If a function would need to become Result-returning, agent STOPS and asks orchestrator.
    No silent signature changes.
D4. HIGH chunk runs LAST so sandbox FFI skip-list is the smallest possible blast radius.
D5. NO new tests, NO refactoring beyond cast conversions.

EXPECTED TEST OUTCOMES
----------------------
A1: cargo check --workspace PASS after each chunk + at end.
A2: cargo test --no-run -p elai-knowledge -p elai-orchestrator -p elai-reasoning PASS.
A3: ~163 narrowing casts replaced or annotated (NARROWING-OK).
A4: Pre-existing 2-warning baseline preserved; no new warnings.
A5: check_entropy_quarantine.sh count unchanged.

UNKNOWNS
--------
- Whether any chunk-4 file has a Result-cascade we missed in enumeration. Mitigation:
  agent stops and asks instead of guessing.
- elai.map stale; agents fall back to ripgrep.

ROLLBACK
--------
git restore on touched files.

NO COMMITS PER ORCHESTRATION RULE.
NO REFACTORING beyond cast conversions.

