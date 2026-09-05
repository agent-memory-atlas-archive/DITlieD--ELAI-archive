# E-21: Peer-grounded harness gap closes (SBFL, ACI, repo-as-state, critic-rerank)
Objective: Translate the 2026-05-19 peer-systems survey into seven scoped, evidence-grounded harness improvements that close the measurable gaps where Agentless, AutoCodeRover, SWE-agent, OpenHands, Aider, Codex CLI, and Cursor 2.0 beat ELAI in benchmarks while keeping ELAI's unique stack (typed ReasoningTemplate + blake3 chain + BSP supersteps + GRC + rules-as-data) intact.
Estimated effort: 3 sessions, ~10-12 h
Pre-requisites: B-1 (DONE, path unification), B-22 (DONE, autonomous mode + StagnationDetector), B-37 (DONE, RuntimeOrchestrator + PerRoleRuntime), B-41 (DONE, per-level auditor), D-1 PLANNED (typed ReasoningTemplate archive — only referenced, not blocked)
Parallel-safe with: E-22 (sandbox / SKILL.md surface), E-24 (MCP harden — different IPC layer), E-25 (compaction ladder — different state surface)
Source report: workflow/improvement-sweep-2026-05-19/reports/21-peer-systems-llm-as-worker.md

## Context

Peer benchmarks now publish hard numbers: Agentless 32% SWE-bench at $0.70/issue (arXiv:2407.01489), AutoCodeRover 37.3% SWE-lite pass@1 with spectrum-based fault localization (arXiv:2404.05427), OpenHands 72.8% SWE-Bench-Verified via inference-time scaling + critic-model rerank (openhands.dev Nov 2025 blog), SWE-agent ACI 12.29% with LM-shaped commands (arXiv:2405.15793 NeurIPS 2024). ELAI has no published SWE-bench number to position against. The report also documents that pipeline / "agentless" scaffolds match or beat ReAct loops at a fraction of token cost, and Kimi-Dev (arXiv:2509.23045) frames the agentless mode as a skill prior — meaning the harness-first philosophy that ELAI is built on now has external validation.

Three peer failures directly inform ELAI guardrails already in place: Devin's rabbit-hole compounding (Answer.AI 3/20, theregister.com 2025/01/23) is countered by the existing anti-spiral skill and reasoning-after-action rule; ChatDev's superficial-check-passes-runtime-fails pattern (arXiv:2307.07924, FC3 verification) is countered by auditor ≥1-FINDING and per-level audit (B-41 SHIPPED); Cline issue #3934 documents the failure of rules-as-prose-in-system-prompt under context pressure, which the rules-as-data approach in `.claude/rules/` and SQLite already addresses. The report's argument is that ELAI is uniquely defensible on these axes but is genuinely missing six peer-proven techniques: tree-sitter PageRank repo-map, SBFL on failing tests, inference-time critic-rerank, ACI command shaping, repo-as-state-store, and multi-provider routing.

The report also flags that the Rust-native harness market is contested (Cursor 2.0 RL-trained-in-harness Composer, Codex CLI Rust 96%, warp 59k, cc-switch 74.5k) but no peer combines FSM + decomposition tree + GRC + rules-as-data + Rust. The window remains open. E-21 captures the six steals plus an externally publishable SWE-bench-Verified number that lets ELAI defend its positioning claim quantitatively.

## Existing state (B/C/D coverage)

- B-22 SHIPPED (s-26716, ~480 LOC + patch-4G.1 ~90): Phase 4G autonomous mode + 4-pattern StagnationDetector port (FOLLOWUP B-22b). DELTA: E-21 critic-rerank reuses StagnationDetector's measurement infra but adds candidate-patch scoring rather than only stagnation classification.
- B-23 PLANNED: self-modification / DGM / CEL / DarwinNet. EXTENSION: E-21 SBFL output is a natural input to B-23's self-mod proposal generator (high-suspicion lines = candidate self-mod targets).
- B-33 PLANNED: Interactive Decomposition + Symbol Graph Viewer (Mermaid + cytoscape). EXTENSION: E-21 PageRank pass over SCIP_REF graph feeds B-33's graph viewer as a "top-K context-relevant files" overlay.
- B-37 SHIPPED (commit fd5c2e5): Multi-Runtime Backend Abstraction Phase 4M (11 backends, RuntimeAdapter, PerRoleRuntime, RuntimeOrchestrator). EXTENSION: E-21 multi-provider routing equivalent reuses PerRoleRuntime; no new abstraction needed. The Aider/LiteLLM "100-provider" pattern is materially equivalent to PerRoleRuntime — what is missing is exposed CLI surface and benchmark fixtures.
- B-41 SHIPPED: per-level auditor modes. DELTA: E-21 critic-rerank is auditor-adjacent but distinct (rerank is a quality signal on N parallel patches; auditor is the gate on a chosen patch). They compose; no replacement.
- D-1 PLANNED (typed ReasoningTemplate + blake3 chain): the unique-pattern foundation. E-21 does NOT block on D-1 but tags any new persisted artifact (repo-state, SBFL ranking) for future D-1 archive ingestion via TODO[D-1-followup] markers per stub-tracking rule.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\21-peer-systems-llm-as-worker.md` (full)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\runtime_orchestrator.rs` (B-37 entrypoint; PerRoleRuntime surface)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\indexing_service.rs` (DKG / SCIP entry; PageRank consumer)
- `F:\.ELAI_workflow\.elai_cc\crates\cli\src\symbol_graph.rs` (existing graph CLI; PageRank attach point)
- `F:\.ELAI_workflow\.claude\tools\elai.map` (Grep only; SCIP_REF / CALL clusters that PageRank operates on)
- `F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md` lines 290-470 (B-15..B-23 harness hardening context)
- `F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-22.md` only if B-22 SSB row set is needed (otherwise rely on B-PLAN-SUMMARY)
- `F:\.ELAI_workflow\.elai_cc\crates\agent\src\stagnation.rs` (StagnationDetector — reused by critic-rerank measurement layer)
- arXiv:2404.05427 AutoCodeRover (workflow/papers/ if ingested; otherwise WebFetch — required for SBFL implementation)
- arXiv:2405.15793 SWE-agent ACI (NeurIPS 2024 — required for command-shape audit)
- openhands.dev blog "SOTA on SWE-Bench-Verified" Nov 2025 (critic-rerank method)
- aider.chat/docs/repomap.html (tree-sitter + PageRank repo-map method)

## Tasks

1. Build a PageRank-over-SCIP pass at `crates/symbol_graph/src/pagerank.rs` (new module under existing symbol_graph crate; do NOT create a new crate). Consume the SCIP_REF + CALL rows already produced by `.claude/tools/codebase_mapper.py` and the live SCIP indices at `.claude/tools/scip_indices/`. Compute a damping-0.85 PageRank with `max_iters=64` and `tolerance=1e-6`. Output a `FileRank { path, rank, in_degree, out_degree }` table persisted via `core::paths::artifacts_dir_path` / `pagerank.bincode`. Aider's repo-map (aider.chat/docs/repomap.html) provides the reference algorithm. Surface a CLI subcommand `elai symbol-graph rank --top K` and a Tauri command `get_pagerank_top` already stubbed under B-3 / B-33 territory; this task wires the real reader.

2. Implement spectrum-based fault localization (SBFL) at `crates/auditor/src/sbfl.rs` (new module). Pull per-test coverage from `cargo tarpaulin --skip-clean --out Json` (verify the crate name and CLI flag via WebSearch before adding; do NOT trust training memory per anti-slopsquatting rule). Compute Ochiai = passed_executions(line) / sqrt(total_failing_tests * total_executing_tests(line)) per AutoCodeRover (arXiv:2404.05427). When the auditor reports BLOCK with failing tests, the SBFL ranking is attached to the auditor JSON output as `suspicious_lines: [{path, line, score, ...}]` and consumed by the fixer agent. Side benefit: ranking informs B-23 self-mod target selection.

3. Inference-time critic-rerank pipeline. New module `crates/orchestrator/src/critic_rerank.rs`. When `RuntimeOrchestrator::execute_step` runs in `mode=critic_rerank`, generate N=3-5 parallel candidate patches via PerRoleRuntime (sidecar can spawn parallel calls because the local Qwen 3B is single-instance — fall back to sequential with N=3 if sidecar reports only one worker available; cite sidecar-protocol skill for boot detail). Score each candidate with a critic role wired through PerRoleRuntime (default critic = same local Qwen 3B with a critic-system-prompt; cloud Sonnet 4.6 critic optional via `--critic-runtime cloud`). Pick the argmax. OpenHands Nov 2025 blog is the reference — `openhands.dev/blog/sota-on-swe-bench-verified-with-inference-time-scaling-and-critic-model`. Emit SSB rows `critic_candidates_generated`, `critic_score_computed`, `critic_pick_committed`.

4. ACI command-shape audit. Run a one-time audit of every tool exposed to the model via the existing tool registry (consumes_symbols list of `crates/agent/src/tool_dispatch.rs` and `crates/plugins/src/mcp_bridge.rs`). For each tool, score on three axes: (a) is the output human-readable verbose or LM-terse, (b) does the tool emit structured fields the LM can parse without regex, (c) does the tool default to a sane scope (e.g., glob narrowed, head_limit set). Output `workflow/audits/aci-tool-shape-2026-05-19.md` with per-tool grades and a fixer queue. Reference: SWE-agent ACI paper (arXiv:2405.15793). This is an audit, not a refactor — the refactor of any failing tool is a follow-up E-N or B-N at the user's discretion.

5. Repo-as-state-store. Create `.elai_cc/.elai/state/` (note: this is INSIDE .elai_cc and is gitignored at the workflow root but TRACKED inside .elai_cc — confirm with the user before adding to .gitignore). The directory holds: `current_phase.md` (one-line), `last_decisions.md` (rolling 10), `stub_registry.toml` (per CLAUDE.md stub-tracking rule), `runtime_choices.toml` (which RuntimeAdapter was used per task). Survives compaction because it's on disk; survives `git clean -fdx` because it's a tracked path. The `live_cli.rs` startup writes its first row to `current_phase.md`; the PreCompact hook persists `last_decisions.md` from in-memory FSM state. Reference: Codex CLI state-in-repo pattern (medium.com/jonathans-musings inside-the-agent-harness 2026).

6. Multi-provider routing exposure. PerRoleRuntime already supports per-role backend per B-37; this task adds a `--runtime-table <path>` CLI flag pointing at `elai_cc.toml` `[runtime.table]` rows that map `role -> RuntimeAdapter`. Concrete adapters to confirm wired (verify via the existing 11-backend list in `crates/providers/src/runtime/`): `anthropic_local_sidecar`, `anthropic_cloud`, `ollama`, `openai_compatible`, `vllm_openai_compatible`. NO LiteLLM dependency added — the report's Open Question on Aider/LiteLLM is answered by the existing PerRoleRuntime; we just expose it. Anti-slopsquatting check: `litellm` PyPI exists but we do NOT use it; we keep the Rust-native PerRoleRuntime.

7. SWE-bench-Verified benchmark subcommand `elai bench swebench-verified`. Pull the SWE-bench-Verified fixture set (verify URL and license via WebSearch; current public source is `github.com/SWE-bench/SWE-bench/blob/main/data/swebench-verified.json` — confirm before committing). Run via the same `RuntimeOrchestrator::execute_step` path that production tasks use. Emit aggregate stats `passed`, `failed_compile`, `failed_test`, `failed_semantic` (the last bucket per memory feedback_verify_semantics: a 511-byte splash != a built feature). Persist to `.elai_cc/.elai/state/benchmarks/swebench-verified-{date}.json`. Report's TL;DR notes that "without a number we can't position against Agentless 32% / AutoCodeRover 37%" — this closes that gap.

## WIRING_CONTRACT

produces_symbols:
  - symbol_graph::pagerank::compute_pagerank(scip_index: &ScipIndex) -> Vec<FileRank>
  - symbol_graph::pagerank::FileRank { path, rank, in_degree, out_degree }
  - orchestrator::peer_systems::sbfl::compute_ochiai(coverage: &CoverageReport) -> Vec<SuspiciousLine>
  - orchestrator::peer_systems::sbfl::SuspiciousLine { path, line, score, passed, failed }
  - orchestrator::peer_systems::critic_rerank::CriticRerank::pick(candidates: Vec<Candidate>) -> Candidate
  - orchestrator::peer_systems::critic_rerank::Candidate { patch, runtime_id, score }
  - cli::bench::swebench_verified_run(opts: SweBenchOpts) -> SweBenchReport
consumes_symbols:
  - crates/orchestrator/src/runtime_orchestrator.rs::PerRoleRuntime (existing B-37)
  - crates/orchestrator/src/indexing_service.rs::IndexingService (DKG / SCIP source)
  - crates/agent/src/stagnation.rs::StagnationDetector (B-22b measurement)
  - crates/auditor/src/lib.rs::Auditor (BLOCK output augments with sbfl field)
  - crates/cli/src/symbol_graph.rs (PageRank CLI surface)
reachable_from:
  - crates/cli/src/main.rs::main (new `bench swebench-verified` and `symbol-graph rank` subcommands)
  - crates/orchestrator/src/task_runner.rs::drive_task (critic_rerank optional path)
  - crates/auditor/src/lib.rs::run_audit (sbfl auto-attached on BLOCK with failing tests)
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_pagerank_top (existing stub now reads real data)
stub_ban:
  - no "TODO" without registered babystep id per stub-tracking rule
  - no `unimplemented!()` in production paths
  - no `.unwrap()` outside test code and main() bootloader
  - no manual coverage parser; use cargo-tarpaulin JSON output only (verify via WebSearch before adding dep)
frontend_pair: crates/dashboard-v2/src/lib/windows/CommandWindow.svelte (PageRank top-K overlay panel; or new PagerankPanel.svelte under B-33's umbrella)
observable_effect:
  trigger: "elai symbol-graph rank --top 5" on a project with indexed DKG
  expect_ssb: "pagerank_computed" within 5000ms with non-empty file list and ranks > 0
  expect_artifact: ".elai_cc/.elai_cc/artifacts/pagerank.bincode" exists and deserializes to >100 FileRank entries on the .elai_cc tree itself

## DETERMINISTIC_CHECK

```bash
# 1. PageRank module exists and reads SCIP_REF rows
rg "fn compute_pagerank" .elai_cc/crates/symbol_graph --type rust
# Expected: 1 hit at crates/symbol_graph/src/pagerank.rs

# 2. SBFL module exists under peer_systems; consumed by auditor BLOCK path
rg "use crate::peer_systems::sbfl::compute_ochiai|peer_systems::sbfl::compute_ochiai" .elai_cc/crates/orchestrator --type rust
# Expected: at least 1 hit at the auditor BLOCK handler consume site

# 3. Critic-rerank wired through PerRoleRuntime; SSB row emitted
rg "critic_pick_committed" .elai_cc/crates --type rust
# Expected: ≥2 hits (emit site + consumer/observer)

# 4. Repo-as-state-store directory written by live_cli startup
test -d .elai_cc/.elai_cc/.elai/state && test -f .elai_cc/.elai_cc/.elai/state/current_phase.md
# Expected: directory + current_phase.md exist after first `elai` invocation

# 5. SWE-bench-Verified subcommand visible
.elai_cc/elai.bat bench swebench-verified --help
# Expected: help output documenting --fixture-path, --max-tasks, --runtime-role

# 6. ACI audit artifact exists and lists every registered tool
test -f workflow/audits/aci-tool-shape-2026-05-19.md
# Expected: file exists with one row per tool in tool_dispatch + mcp_bridge
```

## ACCEPTANCE

- [ ] `compute_pagerank` returns non-empty FileRank list on .elai_cc tree; top-5 files are sensible (orchestrator / cli / dashboard-v2 expected at top)
- [ ] SBFL Ochiai ranking attaches to auditor BLOCK output as `suspicious_lines` field
- [ ] Critic-rerank with N=3 picks a candidate; SSB rows emitted; failure path (only one runtime available) falls back to N=1 gracefully with `critic_rerank_skipped` SSB row
- [ ] ACI audit artifact exists with grades A/B/C/D for every registered tool
- [ ] `.elai_cc/.elai/state/` populated after first session and survives PreCompact
- [ ] `--runtime-table` CLI flag accepts a TOML and overrides PerRoleRuntime for the current run
- [ ] SWE-bench-Verified subcommand runs at least 10 tasks end-to-end and emits a JSON report
- [ ] No new dependencies beyond cargo-tarpaulin (CONFIRM with user before adding); anti-slopsquatting check run on tarpaulin

## Tests to add

- `crates/symbol_graph/tests/pagerank_correctness.rs`: 3-node star graph, verify centre dominates
- `crates/orchestrator/src/peer_systems/sbfl.rs` (or sibling `crates/orchestrator/tests/sbfl_ochiai_correctness.rs`): synthetic coverage table, hand-computed Ochiai matches
- `crates/orchestrator/tests/critic_rerank_picks_argmax.rs`: 3 candidates with known scores, picked = argmax
- `crates/cli/tests/bench_swebench_smoke.rs`: dry-run with `--max-tasks 1 --fixture-path tests/fixtures/swebench-mini.json`
- `tests/contracts/repo_state_store_survives_compaction.rs`: write to `.elai/state/`, fire PreCompact, verify content intact
- `tests/contracts/aci_audit_covers_all_registered_tools.rs`: fail if any registered tool absent from `workflow/audits/aci-tool-shape-*.md`

## Files touched

- CREATE: `crates/symbol_graph/src/pagerank.rs` (~250 LOC)
- CREATE: `crates/orchestrator/src/peer_systems/sbfl.rs` (~200 LOC)
- CREATE: `crates/orchestrator/src/peer_systems/critic_rerank.rs` (~300 LOC)
- CREATE: `crates/cli/src/bench.rs` (~250 LOC)
- CREATE: `workflow/audits/aci-tool-shape-2026-05-19.md` (audit artifact, ~1-2 KB)
- MODIFY: `crates/cli/src/symbol_graph.rs` (+~40 LOC for `rank` subcommand)
- MODIFY: `crates/cli/src/main.rs` (+~30 LOC for `bench` subcommand wiring)
- MODIFY: `crates/orchestrator/src/runtime_orchestrator.rs` (+~30 LOC for critic-rerank path)
- MODIFY: `crates/auditor/src/lib.rs` (+~20 LOC for sbfl attachment on BLOCK)
- MODIFY: `crates/cli/src/live_cli.rs` (+~15 LOC for state-store init)
- MODIFY: `crates/orchestrator/src/session_dag.rs` (+~10 LOC for state-store PreCompact write)
- MODIFY: `crates/dashboard-v2/src-tauri/src/commands.rs` (+~20 LOC, `get_pagerank_top` reads real data)
- CREATE: `workflow/reasoning/E-21-decisions.md`

## Citations / Grounding

- arXiv:2407.01489 (Agentless, Lingming Zhang et al.) — 32% SWE-bench at $0.70/issue; cited in report — grounds the "harness-first is competitive" claim
- arXiv:2404.05427 (AutoCodeRover, ISSTA 2024) — 37.3% SWE-lite pass@1 with SBFL — grounds Task 2 SBFL design
- arXiv:2405.15793 (SWE-agent ACI, NeurIPS 2024) — LM-shaped commands — grounds Task 4 audit
- arXiv:2509.23045 (Kimi-Dev) — agentless training as skill prior — grounds the philosophy alignment
- arXiv:2503.13657 (Cemri et al., "Why Do Multi-Agent LLM Systems Fail?") — FC3 verification weakness — grounds why ELAI's ≥1-FINDING rule stays
- openhands.dev Nov 2025 blog (SOTA via inference-time scaling + critic) — grounds Task 3 critic-rerank
- aider.chat/docs/repomap.html — tree-sitter + PageRank repo-map — grounds Task 1 PageRank choice
- medium.com/jonathans-musings inside-the-agent-harness — Codex state-in-repo pattern — grounds Task 5
- arXiv:2510.08996 + arXiv:2603.00520 (SWE-bench mutation / SWE-ABS) — 19.78% of top-30 "solved" tasks semantically incorrect — grounds Task 7 `failed_semantic` bucket per memory feedback_verify_semantics

## Open questions / risks

- Tarpaulin Windows support: CONFIRM via WebSearch before committing — if Windows-only blocker exists, fall back to `cargo llvm-cov` (also requires verification per anti-slopsquatting). Risk: SBFL on Windows-only dev box.
- Critic-rerank with a single local Qwen 3B sidecar serializes candidates; N=5 multiplies wall time by ~5. Acceptance threshold should tolerate `critic_rerank_skipped` when only one runtime is available — do not block production on multi-runtime hardware.
- Repo-as-state-store inside .elai_cc creates a path-collision risk with the `dkg_db_path` helper from B-1; verify the helper does not write outside `.elai_cc/indexes/` and the new state dir stays under `.elai_cc/.elai/state/`. Confirm with the user that committing `.elai/state/` to the elai_cc git repo is desired before adding it to the tracked set.
- SWE-bench-Verified runs require >1 hour wall time at N=500 tasks even on cloud Sonnet; default `--max-tasks 50` for first run, document via `--help`.
- ACI audit subjectivity: grading axes a/b/c are author-judgment. Open question for the user: should the grading be auditor-agent-driven (defer to test-attributer + auditor) rather than hand-graded?

## Next

E-22 (sandbox + SKILL.md) shares no source surface but introduces microsandbox boot which Task 4's ACI audit will want to grade. E-24 (MCP harden) tightens the same tool-dispatch surface Task 4 audits — coordinate to avoid two simultaneous tool_dispatch refactors. E-25 (compaction ladder) will inherit `.elai/state/` from Task 5 as a persistence target. After E-21 closes, B-23 (self-mod) can consume SBFL `suspicious_lines` directly as candidate self-mod targets — note this as an unblock in B-23's Pre-reqs when that file is updated.
