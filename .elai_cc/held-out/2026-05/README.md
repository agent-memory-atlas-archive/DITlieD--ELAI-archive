# Held-out task fixtures: 2026-05

D-6 seed fixtures for the time-windowed held-out task store. Date-anchored
directory (`{YYYY-MM}/`) holds raw JSONL task bodies; the rusqlite-backed
index sibling is `../held_out.sqlite` (lazy-created on first
`HeldOutStore::open`).

LiveCodeBench (arxiv:2403.07974, T1) §3.2 published 9-grams + Jaccard 0.3
as the contamination-detection sweet spot. D-7 (cross-domain transfer
canary) raised this corpus to 60 fixtures across all 5 `TaskSource`
families (12 per family) so the canary has the per-family floor it needs
(>=10 per family; >=50 total, per D-7 ACCEPTANCE). Bodies deliberately
span the cross-domain axes (code-fix / math-proof / prose-reason /
planning-decomposition / tool-use) via the `expected_outcome` kind so the
corpus exercises every CanaryFamily, not a single axis. Per
workflow/reasoning/D-7-decisions.md DECISION-4 the 5th canary family is
ToolUse, NOT KnowledgeQa. Production ingest paths extend this further with:

- GitHubIssue: fresh issues post-cutoff with verifiable recency
- ARC-AGI / LiveCodeBench public tasks (re-coded as BenchmarkFixture)
- ArxivRepro: reproducibility tasks tied to a paper release date
- HumanCurated: hand-curated math/prose problems

## Format

Each `*.jsonl` file is one JSON object per line, matching the
`HeldOutTask` envelope serialized by the orchestrator. Required keys:

- `task_id` (32-char simple-hex uuid)
- `source` (tagged enum: `github_issue` / `benchmark_fixture` / `synthetic`)
- `body` (the raw normalized task body — the store re-normalizes on ingest)
- `expected_outcome` (tagged enum: `code` / `math` / `prose` with `value`)
- `ingested_ts_ms` (UNIX ms)
- `eligibility_window_ms` (ms before task becomes active for BAI eval)

Hash + n-gram signature are computed deterministically on ingest by
`held_out::build_task`; the fixture file does NOT carry them (they would
drift if the body is edited).

## Sources

One file per `TaskSource` family (5 families, 12 rows each = 60 total). The
file list is the source-of-truth the D-7 corpus contract
(`crates/contracts/tests/d7_held_out_corpus_contract.rs`) parses + validates
against the real orchestrator `TaskSource` / `ExpectedOutcome` enums.

- `seed_synthetic.jsonl`: 12 hand-authored synthetic tasks, diverse 9-gram
  fingerprints, eligibility_window_ms=0 (active immediately).
- `seed_benchmark.jsonl`: 12 ARC-AGI / LiveCodeBench / PRMBench / HotpotQA /
  PlanBench / AlfWorld / ToolBench-style fixtures re-coded as
  `benchmark_fixture` sources.
- `seed_github_issue.jsonl`: 12 issue-shaped tasks (placeholder repo;
  production ingest pulls from the GitHub API for verifiable recency).
- `seed_arxiv_repro.jsonl`: 12 arXiv reproducibility tasks, each carrying an
  `arxiv_id` + `fetched_at_ms` for the temporal date-windowing defense.
- `seed_human_curated.jsonl`: 12 hand-curated tasks, each carrying a
  `curator_id` + `curated_at_ms`.

## Validation

`crates/contracts/tests/d7_held_out_corpus_contract.rs` reads these files
on disk and is the off-agent done-gate for the corpus: it deserializes every
row into the real `TaskSource` + `ExpectedOutcome` enums (so a schema drift
is a compile/runtime FAIL), and asserts >=10 per family, >=50 total, all 5
families present, and all 3 outcome kinds (code/math/prose) present.
