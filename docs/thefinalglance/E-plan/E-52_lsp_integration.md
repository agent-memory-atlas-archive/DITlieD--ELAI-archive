# E-52: LSP-as-verifier (NOT retriever) — diagnostic-probe PostToolUse hook + headless rust-analyzer SCIP refresh + multilspy on-demand find-refs/goto-def
Objective: Add LSP as an OPT-IN precision oracle for type-aware and compile-aware questions only — NOT as a default retrieval layer. Highest-ROI hook is a diagnostic-probe pre-AUDITING gate via B-19 PostToolUse: after every Rust/Python/TS Write/Edit, shell out to a one-shot (`rust-analyzer diagnostics`, `pyright --outputjson`, `tsc --noEmit`) on the affected files; empty → continue, non-empty → inject the diagnostic JSON into the auditor context BEFORE LLM judgment. Refresh elai.map SCIP enrichment via headless `rust-analyzer scip` after large Rust changes (closing B-29's SCIP-already-half-done gap per CLAUDE.md). Wrap multi-language find-refs / goto-def via microsoft/multilspy as an on-demand sidecar service used by context-compiler / impact-mapper agents when the symbol is overloaded or method-resolution-sensitive. NO persistent daemon-for-all-languages — RAM hogs (jdtls ~2-4GB, rust-analyzer 1.5-3GB on 500K LOC) blow the existing 25K-ctx ~2.85GB sidecar envelope.
Estimated effort: 2-3 sessions, ~10-12 h
Pre-requisites: B-19 PLANNED (Hook System Unification — provides the PostToolUse infrastructure this E-N hooks into; per B-PLAN-SUMMARY line 349; sequential after B-18 IPC accept loop). B-29 PLANNED (Symbol-Level Impact Analysis API — provides the symbol graph this E-N's SCIP refresh updates; per B-PLAN-SUMMARY line 571). `.claude/tools/elai.map` already SCIP-enriched (CLAUDE.md: "SCIP_REF/SCIP_DEF rows carry fully-resolved canonical paths from rust-analyzer / scip-python / scip-typescript"; existing `.claude/tools/scip_indices/` directory).
Parallel-safe with: E-51 (edge inference — disjoint), E-53 (formal verification — different precision-oracle tier, complementary), E-54 (eval corpus — disjoint), E-55 (agentic RL — disjoint), E-40 (symbol graph tools — coordinates on elai.map refresh; E-52 adds the refresh-on-edit cadence). DEPENDS ON B-19 PostToolUse hook surface and B-29 symbol graph API.
Source report: workflow/improvement-sweep-2026-05-19/reports/52-lsp-integration.md

## Context

The Oct 2025 Code Retrieval in Coding Agents study (preprints.org/manuscript/202510.0924) and the Lanser-CLI / RL-from-Compiler+LSP-feedback paper (arXiv:2510.22907) together establish that LSP is the wrong primitive for retrieval — but the right primitive for verification. Anthropic shipped exactly this pattern in Claude Code 2.0.74 (Dec 2025, per HN thread item 46355165): native LSP for 11 languages with goto-def, find-refs, and diagnostics, where diagnostics-after-edit is reported as the single highest-value signal. The Monitor-Guided Decoding paper (Agrawal et al., arXiv:2306.10763) — SantaCoder-1.1B + MGD beats text-davinci-003 on next-identifier match and compilation rate — is the foundational verifier-not-retriever framing. rust-analyzer ships CLI subcommands (`analysis-stats`, `scip`, `lsif`, `ssr`, `diagnostics`; per DeepWiki rust-analyzer 7.2 + 9.2) that produce LSP-equivalent semantic data WITHOUT keeping a daemon resident — the headless mode is precisely what ELAI's existing `.claude/tools/scip_indices/` already half-targets. multilspy (microsoft/multilspy) is MSR's canonical multi-language LSP wrapper (12 servers, hand-tuned configs; used for MGD) — the right primitive when ELAI does need a daemon, with on-demand spawn + idle shutdown. Per-language RAM costs (markaicode 2025 bench + amazingcto warnings): rust-analyzer 1.5-3GB on 500K LOC steady-state with 23.4s cold-start on Servo; pyright 500MB-1GB; jdtls 2-4GB; clangd 1-3GB. Together these exceed the existing 25K-ctx Qwen 3B NF4 envelope (~2.85GB total per CLAUDE.md). The If-T benchmark (arXiv:2508.03830) catalogs type-narrowing logic that tree-sitter provably cannot do across TS/Flow/Typed Racket/mypy/Pyright. SCIP replaces LSIF (sourcegraph.com/blog/announcing-scip): protobuf vs JSON, ~4x smaller compressed, ~10x faster CI indexing.

## Existing state (B/C/D coverage)

- B-19 PLANNED (Hook System Unification): provides the PostToolUse surface E-52 hooks into. B-19 already specifies "Ruff PostToolUse subprocess on Python writes (5s timeout, content-hash cache)" per B-PLAN-SUMMARY line 356 — E-52 adds 3 more dispatch entries (`rust-analyzer diagnostics` for `.rs`, `pyright --outputjson` for `.py`, `tsc --noEmit` for `.ts`/`.svelte`). EXTENSION: B-19 surface unchanged; E-52 registers additional dispatchers via the existing PostToolUse extension API.
- B-29 PLANNED (Symbol-Level Impact Analysis API): provides the symbol graph backing `.claude/tools/elai.map`. EXTENSION: E-52 adds an incremental SCIP refresh step that runs `rust-analyzer scip` on affected crates after large Rust changes and merges output into `.claude/tools/scip_indices/`; existing `codebase_mapper.py` rebuild pathway is the fallback.
- B-19 W4 tag in elai.map (line 2530 + 4887 + 10472 + 12327 + 14545 + 14573 + 14574 + 15921 — `task_runner.rs::run_validate_dispatch`): existing B-19 validation dispatch infrastructure is the integration point. EXTENSION: diagnostic probe becomes one entry in this dispatch.
- `.claude/tools/codebase_mapper.py` (existing per CLAUDE.md): regen script for elai.map. EXTENSION: E-52 adds an `--incremental` flag that only refreshes SCIP for crates that changed since last run (uses git diff against last-recorded sha).
- Tree-sitter (already in use per E-50 dependency check; verify presence): used for syntactic boundaries; E-52 explicitly DOES NOT touch tree-sitter, only adds LSP-as-verifier alongside.
- `crates/orchestrator/src/auditor.rs` (existing AUDITING substate): E-52 wires diagnostic-probe results into the auditor's context BEFORE LLM judgment via a new `DiagnosticProbe` evidence variant. EXTENSION: anti-rubber-stamp inputs gain a deterministic signal.
- NO existing LSP daemon code, NO multilspy dependency, NO ts-morph, NO ssr usage: all NEW additions.
- E-40 (symbol graph tools): coordinates on elai.map. E-52's SCIP refresh produces the same output format E-40 reads.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\52-lsp-integration.md` (full)
- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\E-plan\E-40_symbol_graph_tools.md` (elai.map refresh cadence; coordinate)
- `F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md` lines 349-360 (B-19 Hook System Unification) + lines 571-660 (B-29 + B-29.5 Symbol-Level Impact Analysis)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs` lines 6680-6770 (`run_validate_dispatch` + B-19 W4 emit_tool_dispatch / emit_validation_tier; per elai.map lines 12327, 2530, 4887)
- `F:\.ELAI_workflow\.claude\tools\codebase_mapper.py` (full — current mapper logic; extend with `--incremental`)
- `F:\.ELAI_workflow\.claude\tools\elai.map` header (read first 50 lines via `Read` with limit=50; verify column layout for SCIP_REF/SCIP_DEF rows)
- `F:\.ELAI_workflow\.claude\tools\scip_indices\` (directory listing; verify current per-crate SCIP output shape)
- https://deepwiki.com/rust-lang/rust-analyzer/7.2-command-line-interface (WebFetch — analysis-stats, lsif, scip, ssr, diagnostics subcommands)
- https://deepwiki.com/rust-lang/rust-analyzer/9.2-scip-and-lsif-indexing (WebFetch — SCIP/LSIF batch emit; same data as live LSP)
- https://github.com/microsoft/multilspy (WebFetch — Python LSP client, 12 servers; multilspy README + supported-servers list)
- https://sourcegraph.com/blog/announcing-scip (WebFetch — SCIP-vs-LSIF migration rationale)
- https://arxiv.org/pdf/2306.10763 (WebFetch — Monitor-Guided Decoding paper)
- https://arxiv.org/pdf/2510.22907 (WebFetch — Lanser-CLI LSP-feedback-as-process-reward)
- https://www.preprints.org/manuscript/202510.0924 (WebFetch — LSP-as-retrieval underperforms paper; context for not making LSP a default)
- https://arxiv.org/pdf/2508.03830 (WebFetch — If-T type-narrowing benchmark across TS/Flow/Pyright/mypy)
- https://www.amazingcto.com/lsp-in-claude/ (WebFetch — Claude Code LSP writeup; RAM warnings)
- https://markaicode.com/rust-lsp-servers-2025-performance-benchmarks-feature-comparison/ (WebFetch — Servo 23.4s cold-start; markaicode 2025 RAM bench)

## Tasks

1. Diagnostic-probe PostToolUse hook dispatcher. CREATE `.elai_cc/crates/orchestrator/src/diagnostic_probe.rs` (~280 LOC): a dispatch table by file extension — `.rs` → `rust-analyzer diagnostics --output-format=json <crate-root>`; `.py` → `pyright --outputjson <files>`; `.ts`/`.tsx` → `tsc --noEmit --pretty false`; `.svelte` → `svelte-check --output machine` (already in dashboard tooling). The probe wraps each subprocess with a configurable timeout (default 5s for Python, 10s for TS, 15s for Rust per markaicode bench numbers — pyright multi-pass is fast, rust-analyzer is slower on cold cache); on timeout the result is `DiagnosticProbeOutcome::Timeout`, NOT `Pass` (do not silently swallow). Returns `DiagnosticProbeOutcome::{Pass, Diagnostics(Vec<Diagnostic>), Timeout, ToolMissing}`. Each Diagnostic is a structured record with `file`, `line`, `column`, `severity` (Error|Warning|Hint|Info), `code` (e.g. `E0308`, `reportArgumentType`, `2322`), `message`. Tool-missing outcome is a soft warning, NOT a BLOCK — first-time runs without rust-analyzer installed shouldn't bork the workflow.

2. PostToolUse hook registration. MODIFY `crates/orchestrator/src/task_runner.rs` (~50 LOC delta) around `run_validate_dispatch` at line 6694 (per elai.map line 12327): add a `validate_tier_diagnostics` invocation after `emit_tool_dispatch` (line 6736, per elai.map line 15921). The dispatcher resolves the affected files from the most recent Write/Edit (B-19 W4 already tracks this per the emit_tool_dispatch / emit_validation_tier calls), groups by extension, and runs the probe per-language in parallel up to a parallelism cap (default 3 per `mcp_fsm_get_state`-style guard). Emit `diagnostic_probe_dispatched` SSB row with the dispatched languages; emit `diagnostic_probe_completed` per probe with outcome + count; emit `diagnostic_probe_block` if any Error-severity diagnostic returned. Cache by blake3(file_content + tool_version_str) for 30s to prevent re-running on the same content during retry cycles (matches B-19's "content-hash cache" pattern at line 356).

3. Auditor evidence injection. MODIFY `crates/orchestrator/src/evidence.rs` (~40 LOC delta): add a `DiagnosticProbeEvidence { language, file_count, diagnostics: Vec<Diagnostic>, outcome: DiagnosticProbeOutcome }` variant that AUDITING substate consumes BEFORE LLM judgment. The auditor's anti-rubber-stamp gate (per existing D-1 / D-3 trajectory) gains a strict rule: if `diagnostics.iter().any(|d| d.severity == Severity::Error)`, audit verdict is BLOCK with the diagnostic list pre-routed into the fixer prompt as structured input. Hints and Warnings are surfaced to the auditor LLM but DO NOT auto-block — they inform the LLM's verdict.

4. Headless rust-analyzer SCIP refresh on-edit. CREATE `.claude/tools/scip_refresh.py` (~180 LOC stdlib-only Python, matching the existing `.claude/tools/` convention): takes a list of changed Rust files (from `git diff --name-only HEAD`) → resolves them to crates (by walking up to nearest `Cargo.toml`) → for each affected crate, invokes `rust-analyzer scip --output .claude/tools/scip_indices/<crate>.scip <crate-path>` → invokes `codebase_mapper.py --incremental --merge-scip .claude/tools/scip_indices/<crate>.scip` to splice the fresh SCIP rows into elai.map. Idempotent. Logs to `.claude/tools/scip_refresh.log`. Hook surface: a PostToolUse handler on Rust Write/Edit that runs the script in background (or `--queue` flag for the auditor to drain).

5. codebase_mapper.py `--incremental` + `--merge-scip` flags. MODIFY `.claude/tools/codebase_mapper.py` (~120 LOC delta — verify current LOC via Read; CLAUDE.md says stdlib-only Python so no new dependencies): add `--incremental` (only re-scan files changed since last run, tracked via `.claude/tools/.codebase_mapper.last_sha`) and `--merge-scip <path>` (read SCIP protobuf from path and splice SCIP_DEF / SCIP_REF rows into elai.map in canonical 6-col TSV per the existing header). Both flags MUST preserve the existing elai.map header documentation. Idempotent. Note: per CLAUDE.md "stdlib-only Python on .claude/tools/" — SCIP protobuf parsing must NOT add a third-party dep; use the existing approach (verify whether current scip-indices code parses protobuf via stdlib `struct` or via a `protoc`-generated module already vendored under `.claude/tools/scip_proto/`; if absent, ship a minimal protobuf-decoder for the SCIP schema in `.claude/tools/_scip_proto_lite.py` ~150 LOC).

6. multilspy on-demand find-refs / goto-def sidecar service. CREATE `.elai_cc/python/elai_inference/lsp_oracle.py` (~220 LOC) as a NEW Python module loaded by the existing sidecar server (separate from the main inference path; avoids competing for VRAM). Exposes two HTTP endpoints on the existing sidecar port (or a separate `--lsp-oracle-port`): `POST /lsp/find_references {symbol, language, repo_root}` returns `{file, line, kind}` list; `POST /lsp/goto_definition {symbol, language, repo_root}` returns the canonical location. Internally uses multilspy with on-demand server spawn + 60s idle shutdown (multilspy supports this per its README). Servers cached per-language for the session. CRITICAL: jdtls / clangd / rust-analyzer / pyright RAM costs (per report table: 2-4GB / 1-3GB / 1.5-3GB / 0.5-1GB on 500K LOC) MUST be enforced via per-server RAM ceilings — emit `lsp_server_ram_ceiling_exceeded` SSB row + kill the server if it exceeds the ceiling (default 2GB for Rust, 1GB for Python, 1GB for TS).

7. context-compiler / impact-mapper agent integration. MODIFY `.claude/agents/context-compiler.md` and `.claude/agents/impact-mapper.md` (NOT modifying main-LLM-readable files — these are agent definitions; verify via Glob the agents exist at those paths first). Both agents already use `.claude/tools/elai.map` per CLAUDE.md "Prefer `.claude/tools/elai.map` over whole-repo grep". E-52 adds an OPT-IN escalation: when the agent encounters an overloaded symbol (multiple SCIP_DEF rows for the same name) or a question that needs type inference (`is this expression a Result<T,E> or Option<T>?` per If-T benchmark), the agent calls the sidecar `POST /lsp/find_references` or `POST /lsp/goto_definition` endpoint AFTER first trying elai.map. The agent uses the LSP result as a TIE-BREAKER, NOT as the primary signal — matches the report's "verifier not retriever" framing.

8. rust-analyzer `ssr` mechanical refactor surface (deferred to OPT-IN). CREATE `.claude/tools/rust_ssr.py` (~120 LOC stdlib-only Python wrapper): exposes `python rust_ssr.py --pattern '$a.foo($b) ==>> bar($a, $b)' --apply <crate>` for mechanical refactors. NOT wired into the auditor or PostToolUse hook by default — opt-in tool for impact-mapper when rename/signature-shape refactor across >10 sites is detected. Documented in `.claude/tools/_README.md` (NEW or extend if exists). Per CLAUDE.md "No unsolicited refactoring": ssr is opt-in only, requires explicit user or agent invocation.

9. ts-morph / TS bindings drift check stub. CREATE `.elai_cc/crates/dashboard-v2/tools/check_ts_bindings.mjs` (~100 LOC Node.js, no new npm deps if ts-morph is already in package.json — verify per Anti-slopsquatting protocol; if absent, add ts-morph >=22.0 pinned with `npm view ts-morph version` verification first per CLAUDE.md): reads `ts-rs`-generated bindings and validates that Tauri command signatures on the Svelte side match the Rust types from `crates/dashboard-v2/src-tauri/src/commands.rs`. Outputs `{drift: bool, mismatches: [...]}` JSON. Wired into the `cross_lang_diff.py` harness check (per CLAUDE.md `.claude/tools/cross_lang_diff.py`) as an additional gate for Rust↔Svelte pairs.

10. Documentation + decision doc. CREATE `workflow/reasoning/E-52-decisions.md` (~100 LOC) recording: (i) LSP-as-verifier-not-retriever framing per report; (ii) headless-over-daemon default; (iii) the 4 deferred items (MGD decoder constraint, Glean facts DB, LibCST Python codemods, full multilspy daemon-for-every-language); (iv) measured latencies from Task 1 (record p99 wall-time for rust-analyzer diagnostics on `crates/cli` and workspace-root once measurable); (v) Open Questions 1-5 from the report carried forward as TODOs.

## WIRING_CONTRACT

produces_symbols:
  - orchestrator::diagnostic_probe::DiagnosticProbe (struct)
  - orchestrator::diagnostic_probe::DiagnosticProbeOutcome::{Pass, Diagnostics, Timeout, ToolMissing}
  - orchestrator::diagnostic_probe::Diagnostic { file, line, column, severity, code, message }
  - orchestrator::diagnostic_probe::Severity::{Error, Warning, Hint, Info}
  - orchestrator::evidence::EvidenceVariant::DiagnosticProbeEvidence (variant extension)
  - SSB action_types: diagnostic_probe_dispatched, diagnostic_probe_completed, diagnostic_probe_block, lsp_server_spawned, lsp_server_idle_shutdown, lsp_server_ram_ceiling_exceeded, scip_refresh_completed
  - .claude/tools/scip_refresh.py (entrypoint with --incremental --merge-scip)
  - .claude/tools/codebase_mapper.py --incremental + --merge-scip flags (extension)
  - .elai_cc/python/elai_inference/lsp_oracle.py (sidecar HTTP endpoints: POST /lsp/find_references, POST /lsp/goto_definition)
  - .claude/tools/rust_ssr.py (opt-in mechanical refactor wrapper)
  - .elai_cc/crates/dashboard-v2/tools/check_ts_bindings.mjs (TS bindings drift gate)
consumes_symbols:
  - crates/orchestrator/src/task_runner.rs::run_validate_dispatch (B-19 W4; per elai.map line 12327)
  - crates/orchestrator/src/task_runner.rs::emit_tool_dispatch (B-19 W4; per elai.map line 15921)
  - crates/orchestrator/src/task_runner.rs::emit_validation_tier (B-19 W4; per elai.map line 2530)
  - crates/orchestrator/src/evidence.rs::EvidenceGate trait (D-1; per elai.map line 326787 — DEF_TRAIT Send + Sync at evidence.rs:160)
  - .claude/tools/elai.map (SCIP-enriched format; CLAUDE.md "elai.map (~375K lines, SCIP-enriched)")
  - .claude/tools/codebase_mapper.py (existing regen script; CLAUDE.md)
  - .claude/tools/scip_indices/ (existing SCIP per-crate output directory)
  - python/elai_inference/server.py (existing sidecar HTTP server; lsp_oracle.py loads into it)
  - .claude/tools/cross_lang_diff.py (existing harness; check_ts_bindings.mjs wires into it)
  - blake3 (for diagnostic-probe content-hash cache; NOT blake2/sha2)
reachable_from:
  - crates/orchestrator/src/task_runner.rs::run_validate_dispatch (PostToolUse pathway)
  - crates/orchestrator/src/auditor.rs (verify via Grep — auditor consumes DiagnosticProbeEvidence)
  - .claude/agents/context-compiler.md + impact-mapper.md (agents call /lsp/find_references when overloaded symbol detected)
  - workflow/sessions/{id}/hook-executions.log (PostToolUse hooks log here per CLAUDE.md)
stub_ban:
  - no `.unwrap()` outside `#[cfg(test)]` in diagnostic_probe.rs (CLAUDE.md HARD RULE)
  - no `unimplemented!()` / `todo!()` in any new module
  - DiagnosticProbeOutcome::ToolMissing MUST emit a soft warning row + downgrade severity, never silently Pass
  - DiagnosticProbeOutcome::Timeout MUST emit a row + NOT auto-Pass (could mask real Error)
  - lsp_oracle.py MUST enforce per-server RAM ceilings (default 2GB Rust / 1GB Python / 1GB TS); kill + emit row on exceed
  - lsp_oracle.py MUST run multilspy with 60s idle shutdown — no permanent daemons (CLAUDE.md hardware-heavy envelope)
  - no LSP-as-retrieval-by-default — per report anti-pattern, LSP is verifier not primary retrieval signal
  - no LSIF emit — SCIP only (per Sourcegraph migration; LSIF deprecated; ~4x smaller protobuf)
  - rust_ssr.py is OPT-IN; auditor / PostToolUse MUST NOT auto-invoke it (CLAUDE.md "No unsolicited refactoring")
  - blake3 only for content-hash cache; no blake2/sha2 (CLAUDE.md anti-stale-training)
  - tree-sitter usage is UNCHANGED — E-52 explicitly does NOT replace tree-sitter with LSP for structural queries (per report decision matrix)
  - no new third-party Python dep without anti-slopsquatting check (npm/pypi/SPM verification per CLAUDE.md; multilspy verified via pip / pypi.org)
frontend_pair: null (no dashboard component — diagnostic probe is auditor-internal; if visualization desired, defer to E-29 / E-33 dashboard UX phases)
observable_effect:
  trigger: Edit on `.elai_cc/crates/cli/src/main.rs` that introduces a `let x: u32 = "string";` type error
  expect_ssb: ≥1 `diagnostic_probe_dispatched` row with language=rust, ≥1 `diagnostic_probe_completed` row with outcome=Diagnostics(1) and the E0308 code present, ≥1 `diagnostic_probe_block` row preceded by the AUDITING substate consuming DiagnosticProbeEvidence
  expect_artifact: auditor verdict is BLOCK with the rust-analyzer diagnostic JSON pre-injected into the fixer prompt context (verify via the fixer's incoming context dump)

## DETERMINISTIC_CHECK

```bash
# 1. diagnostic_probe module exists and exports the 4 outcome variants
rg "DiagnosticProbeOutcome::Pass|DiagnosticProbeOutcome::Diagnostics|DiagnosticProbeOutcome::Timeout|DiagnosticProbeOutcome::ToolMissing" .elai_cc/crates/orchestrator/src/diagnostic_probe.rs
# Expected: ≥4 hits

# 2. Severity variants
rg "Severity::Error|Severity::Warning|Severity::Hint|Severity::Info" .elai_cc/crates/orchestrator/src/diagnostic_probe.rs
# Expected: ≥4 hits

# 3. Dispatch by extension
rg "\.rs.*rust-analyzer|\.py.*pyright|\.ts.*tsc|\.svelte.*svelte-check" .elai_cc/crates/orchestrator/src/diagnostic_probe.rs
# Expected: ≥4 hits

# 4. Hook registration in task_runner
rg "validate_tier_diagnostics|DiagnosticProbe|diagnostic_probe_dispatched" .elai_cc/crates/orchestrator/src/task_runner.rs
# Expected: ≥3 hits

# 5. Evidence variant added
rg "DiagnosticProbeEvidence" .elai_cc/crates/orchestrator/src/evidence.rs
# Expected: ≥1 hit

# 6. SCIP refresh tool exists + is stdlib-only Python
ls .claude/tools/scip_refresh.py
# Expected: file exists
rg "^import |^from " .claude/tools/scip_refresh.py | rg -v "^import (os|sys|subprocess|pathlib|json|hashlib|argparse|logging|time|re)|^from (os|sys|subprocess|pathlib|json|hashlib|argparse|logging|time|re) "
# Expected: zero hits (stdlib only)

# 7. codebase_mapper.py has new flags
rg "\\-\\-incremental|\\-\\-merge-scip" .claude/tools/codebase_mapper.py
# Expected: ≥2 hits

# 8. lsp_oracle.py exists with the 2 endpoints
rg "find_references|goto_definition" .elai_cc/python/elai_inference/lsp_oracle.py
# Expected: ≥2 hits

# 9. RAM ceiling enforcement
rg "ram_ceiling|RAM_CEILING|2GB|1GB|2 \\* 1024 \\* 1024 \\* 1024" .elai_cc/python/elai_inference/lsp_oracle.py
# Expected: ≥2 hits

# 10. multilspy idle shutdown
rg "idle_shutdown|idle_timeout|60s|60.0" .elai_cc/python/elai_inference/lsp_oracle.py
# Expected: ≥1 hit

# 11. rust_ssr.py opt-in marker
rg "OPT-IN|opt-in|unsolicited refactoring|--apply" .claude/tools/rust_ssr.py
# Expected: ≥2 hits

# 12. TS bindings drift gate
ls .elai_cc/crates/dashboard-v2/tools/check_ts_bindings.mjs
# Expected: file exists
rg "ts-morph|tsmorph" .elai_cc/crates/dashboard-v2/tools/check_ts_bindings.mjs
# Expected: ≥1 hit

# 13. blake3 only (no blake2/sha2)
rg "blake3" .elai_cc/crates/orchestrator/src/diagnostic_probe.rs
# Expected: ≥1 hit
rg "blake2|sha2" .elai_cc/crates/orchestrator/src/diagnostic_probe.rs
# Expected: zero hits

# 14. No .unwrap() outside #[cfg(test)] in new modules
rg "\\.unwrap\\(\\)" .elai_cc/crates/orchestrator/src/diagnostic_probe.rs | rg -v "cfg\\(test\\)|#\\[test\\]|tests/"
# Expected: zero hits

# 15. No LSIF emit
rg "lsif|LSIF" .claude/tools/scip_refresh.py .elai_cc/crates/orchestrator/src/diagnostic_probe.rs .elai_cc/python/elai_inference/lsp_oracle.py
# Expected: zero hits (or only in comments referencing the deprecation)

# 16. Smoke tests
cd .elai_cc && cargo test -p orchestrator diagnostic_probe_dispatches_by_extension
cargo test -p orchestrator diagnostic_probe_blocks_on_rust_error
cargo test -p orchestrator diagnostic_probe_timeout_does_not_auto_pass
# Expected: 3 PASS
```

## ACCEPTANCE

- [ ] `DiagnosticProbe` module dispatches by file extension to rust-analyzer / pyright / tsc / svelte-check with per-language timeouts
- [ ] `DiagnosticProbeOutcome::{Pass, Diagnostics, Timeout, ToolMissing}` — Timeout and ToolMissing do NOT silently Pass
- [ ] PostToolUse hook in `task_runner.rs::run_validate_dispatch` fires on every Rust/Python/TS/Svelte Write/Edit
- [ ] Auditor consumes `DiagnosticProbeEvidence` BEFORE LLM judgment; Error severity = BLOCK with diagnostic JSON pre-routed to fixer
- [ ] `.claude/tools/scip_refresh.py` invokes `rust-analyzer scip` per-affected-crate and merges into `.claude/tools/scip_indices/`
- [ ] `.claude/tools/codebase_mapper.py` gains `--incremental` and `--merge-scip` flags, stdlib-only Python (no new dep)
- [ ] `lsp_oracle.py` exposes `POST /lsp/find_references` + `POST /lsp/goto_definition`; multilspy with 60s idle shutdown + per-server RAM ceiling enforced
- [ ] context-compiler and impact-mapper agents updated to use LSP as TIE-BREAKER, NOT primary retrieval signal
- [ ] `rust_ssr.py` exists as OPT-IN refactor wrapper; NOT auto-invoked
- [ ] `check_ts_bindings.mjs` validates Rust ↔ Svelte type drift via ts-morph (verified via Anti-slopsquatting)
- [ ] No persistent daemons-for-all-languages — on-demand spawn + idle shutdown only
- [ ] No LSIF emit anywhere — SCIP-only per Sourcegraph migration
- [ ] LSP is verifier-not-retriever per report — tree-sitter usage unchanged
- [ ] blake3 only for content-hash cache; no blake2/sha2
- [ ] `cargo check --workspace` is green; `cargo test -p orchestrator` includes the 3 new smoke tests passing

## Tests to add

- `crates/orchestrator/tests/diagnostic_probe_dispatches_by_extension.rs`: 4 file extensions → 4 distinct subprocesses invoked
- `crates/orchestrator/tests/diagnostic_probe_blocks_on_rust_error.rs`: synthetic file with E0308 → BLOCK + DiagnosticProbeEvidence injected
- `crates/orchestrator/tests/diagnostic_probe_timeout_does_not_auto_pass.rs`: mock subprocess that hangs > timeout → DiagnosticProbeOutcome::Timeout, NOT Pass; emit row
- `crates/orchestrator/tests/diagnostic_probe_tool_missing_warns_not_blocks.rs`: missing pyright binary → ToolMissing soft warning; not BLOCK
- `crates/orchestrator/tests/diagnostic_probe_cache_by_blake3.rs`: same content+tool-version → second run hits cache, no subprocess
- `crates/orchestrator/tests/auditor_consumes_diagnostic_probe_evidence_before_llm.rs`: AUDITING substate ingests evidence pre-LLM
- `crates/contracts/tests/lsp_as_verifier_not_retriever_contract.rs`: assert main retrieval pathway (elai.map) is used FIRST; LSP only invoked on tie-break trigger
- `.claude/tools/tests/scip_refresh_idempotent.py`: run twice on same input, output unchanged
- `.claude/tools/tests/codebase_mapper_incremental_round_trip.py`: full rebuild vs --incremental on a fixture crate produce same elai.map rows
- `.elai_cc/python/tests/test_lsp_oracle_ram_ceiling.py`: synthetic huge LSP server → killed + lsp_server_ram_ceiling_exceeded emitted
- `.elai_cc/python/tests/test_lsp_oracle_idle_shutdown.py`: no traffic for 60s → server killed
- `.elai_cc/crates/dashboard-v2/tools/tests/check_ts_bindings_detects_drift.test.mjs`: mismatched Tauri command signature → drift=true

## Files touched

- CREATE: `crates/orchestrator/src/diagnostic_probe.rs` (~280 LOC)
- CREATE: `.claude/tools/scip_refresh.py` (~180 LOC, stdlib-only Python)
- CREATE: `.claude/tools/_scip_proto_lite.py` (~150 LOC IF no existing SCIP protobuf decoder; otherwise SKIP — verify first via Read on `.claude/tools/` for any existing scip_proto module)
- CREATE: `.elai_cc/python/elai_inference/lsp_oracle.py` (~220 LOC)
- CREATE: `.claude/tools/rust_ssr.py` (~120 LOC stdlib-only Python)
- CREATE: `.elai_cc/crates/dashboard-v2/tools/check_ts_bindings.mjs` (~100 LOC)
- CREATE: 12 test files listed above (~30-60 LOC each)
- CREATE: `workflow/reasoning/E-52-decisions.md` (~100 LOC)
- MODIFY: `crates/orchestrator/src/task_runner.rs` (~50 LOC delta — register validate_tier_diagnostics in run_validate_dispatch)
- MODIFY: `crates/orchestrator/src/evidence.rs` (~40 LOC delta — DiagnosticProbeEvidence variant + EvidenceGate impl)
- MODIFY: `crates/orchestrator/src/auditor.rs` (~30 LOC delta — consume DiagnosticProbeEvidence pre-LLM; verify line range via Grep `AUDITING|run_audit` first)
- MODIFY: `.claude/tools/codebase_mapper.py` (~120 LOC delta — --incremental + --merge-scip flags; preserve existing header doc)
- MODIFY: `.claude/agents/context-compiler.md` (~20 LOC delta — add LSP-tie-breaker escalation rule; verify file exists via Glob first)
- MODIFY: `.claude/agents/impact-mapper.md` (~20 LOC delta — same)
- MODIFY: `.claude/tools/cross_lang_diff.py` (~30 LOC delta — wire check_ts_bindings.mjs as Rust↔Svelte check)

## Citations / Grounding

- https://www.preprints.org/manuscript/202510.0924 — LSP-as-retrieval underperforms RAG+lexical; grounds verifier-not-retriever framing in Task 1 + anti-pattern in WIRING_CONTRACT
- https://arxiv.org/pdf/2510.22907 — Lanser-CLI / RL-from-Compiler+LSP-feedback paper Oct 2025; LSP signals as process rewards, deterministic+replayable; grounds Task 1 PostToolUse-gate framing
- https://arxiv.org/pdf/2306.10763 — Monitor-Guided Decoding (Agrawal et al., MSR); SantaCoder-1.1B+MGD beats text-davinci-003 on compilation rate; grounds Open Q3 MGD pilot
- https://news.ycombinator.com/item?id=46355165 — Anthropic shipped LSP in Claude Code 2.0.74 Dec 2025, 11 languages; precedent for the diagnostic-probe-as-default pattern
- https://www.amazingcto.com/lsp-in-claude/ — diagnostics-after-edit highest-value LSP signal; pyright/RA/jdtls RAM heavy on large projects; grounds Task 1 priority + Task 6 RAM ceilings
- https://github.com/microsoft/multilspy — MSR multilspy library, 12 servers, hand-tuned configs; grounds Task 6 wrapper choice
- https://deepwiki.com/rust-lang/rust-analyzer/7.2-command-line-interface — analysis-stats, lsif, scip, ssr, diagnostics CLI subcommands; grounds Task 4 + Task 8 headless mode
- https://deepwiki.com/rust-lang/rust-analyzer/9.2-scip-and-lsif-indexing — batch-emit SCIP/LSIF protobuf, same data as live LSP, no daemon; grounds Task 4 + Task 5
- https://sourcegraph.com/blog/announcing-scip — SCIP protobuf ~4x smaller compressed, ~10x faster CI indexing; grounds SCIP-only stub_ban
- https://markaicode.com/rust-lsp-servers-2025-performance-benchmarks-feature-comparison/ — Servo 500K-LOC time-to-first-completion: rust-analyzer 23.4s cold-start; grounds Task 1 timeouts + Task 6 RAM ceilings
- https://arxiv.org/pdf/2508.03830 — If-T type-narrowing benchmark across TS/Flow/Typed Racket/mypy/Pyright; grounds anti-pattern "tree-sitter for type questions"
- https://github.com/jonrad/lsp-mcp — MCP wrapper exposing LSP requests as MCP tools; pattern for Task 6 daemon mode
- https://engineering.fb.com/2024/12/19/developer-tools/glean-open-source-code-indexing/ — Meta Glean facts DB + Angle query lang; grounds long-term-deferred future tier
- https://github.com/Instagram/LibCST — Lossless Python CST; grounds long-term-deferred Python codemod tier
- https://ts-morph.com/ — TS compiler API wrapper, in-process; grounds Task 9 TS bindings drift gate
- CLAUDE.md anti-stale-training — blake3 only; stdlib-only Python in .claude/tools/; no .unwrap() in lib code; grounds stub_ban
- CLAUDE.md "elai.map (~375K lines, SCIP-enriched)" — grounds Task 4 SCIP refresh framing
- CLAUDE.md "Prefer .claude/tools/elai.map over whole-repo grep" — grounds Task 7 LSP-as-tie-breaker rule

## Open questions / risks

- Open Q1 (from report): p99 wall-time for `rust-analyzer diagnostics --json` on each ELAI crate. Acceptable as PostToolUse hook only if <2s. Mitigation: Task 1 ships with default 15s timeout for Rust; first session measures p99 on `crates/cli` (medium) and `crates/orchestrator` (large) and updates the default. Record measured numbers in `workflow/reasoning/E-52-decisions.md`.
- Open Q2 (from report): pyright (strict mode) recall on sidecar NF4/SDPA path bugs (dtype mismatches, missing await) — runtime-only or compile-time-catchable? Replay last 20 sidecar bugs from gotchas.md through pyright; count caught vs missed. Deferred to first measurement session.
- Open Q3 (from report): MGD payoff for ELAI's main model (Qwen 2.5-Coder-3B-Instruct) — does LSP-constrained decoding move the needle? Pilot on 10 cross-module edits in the cosmos-style verifier loop. Out of scope for E-52 ship; spec only.
- Open Q4 (from report): multilspy stability on Windows hosts proxying through remote-sidecar SSH tunnel. Risk: multilspy hand-tuned configs may target POSIX paths only. Mitigation: lsp_oracle.py runs on the remote sidecar (Linux), with paths translated at the request boundary; document in workflow/reasoning/E-52-decisions.md.
- Open Q5 (from report): SCIP refresh incrementality — does `rust-analyzer scip` produce stable symbol IDs across runs? If not, full-rebuild is the only safe option. Mitigation: Task 5 measures stability on a 3-run roundtrip of the same workspace; fall back to full rebuild if IDs drift.
- Risk 1: per-language RAM cost on a real ELAI workspace (~ to be measured) may exceed the 2GB Rust ceiling — kill+rerun cycle could spiral. Mitigation: emit `lsp_server_ram_ceiling_exceeded` + skip further LSP calls for that crate this session (downgrade to elai.map-only).
- Risk 2: multilspy is a Microsoft Research project — may not be actively maintained. Verify last commit date via WebFetch before pinning per Anti-slopsquatting. If unmaintained, fall back to direct JSON-RPC against rust-analyzer / pyright / tsserver in lsp_oracle.py (~50 LOC extra).
- Risk 3: ts-morph dep addition for Task 9 needs explicit user confirmation per CLAUDE.md "No new dependencies without confirmation". Mitigation: Task 9 ships ts-morph stub IF the user confirms; otherwise defers to a separate patch and ships only the SCIP refresh + diagnostic probe.
- Risk 4: PostToolUse hook running on every edit could add 1-15s to the EXECUTING→AUDITING transition. Mitigation: blake3 content-hash cache eliminates the cost on retry; first-edit cost is the real measurement target for Open Q1.
- Risk 5: `.claude/tools/_scip_proto_lite.py` (Task 5) hand-rolling protobuf decoding is fragile. Mitigation: verify FIRST whether an existing decoder already lives in `.claude/tools/scip_proto/` per the elai.map header that documents SCIP_REF columns; if so, REUSE; if not, the lite decoder targets only the 4-5 SCIP message types ELAI uses (Document, Occurrence, SymbolInformation, Range) — verifiable surface.

## Next

E-52 makes LSP a precision oracle for type-aware and compile-aware questions while keeping elai.map+SCIP as the primary retrieval layer per the report's verifier-not-retriever framing. After E-52, every Rust/Python/TS/Svelte Write/Edit triggers a one-shot diagnostic probe via PostToolUse before AUDITING — the propagate-bad-edit failure mode is cut at its source, matching Claude Code 2.0.74's reported gain. SCIP refresh closes B-29's symbol-graph incrementality gap, eliminating the `elai_map_freshness.py` stale-warning class. multilspy on-demand sidecar handles overloaded-symbol tie-break for context-compiler / impact-mapper without committing to permanent daemons (jdtls+RA+pyright would otherwise blow the 25K-ctx ~2.85GB envelope per CLAUDE.md). Downstream: E-53 (formal verification) consumes the same diagnostic-probe surface as one of its verifier inputs; E-29 / E-33 dashboard UX phases may add a DiagnosticProbe panel later. Deferred: Monitor-Guided Decoding decoder constraint (Open Q3), Glean facts DB (long-term), LibCST Python codemods (mirror of rust_ssr for Python; opt-in tier). Patch E-52.1 candidate: measure Open Q1 + Q2 on first sessions and tune timeouts / strict-mode pyright.
