# Held-out task fixtures: 2026-05

D-6 seed fixtures for the time-windowed held-out task store. Date-anchored
directory (`{YYYY-MM}/`) holds raw JSONL task bodies; the rusqlite-backed
index sibling is `../held_out.sqlite` (lazy-created on first
`HeldOutStore::open`).

LiveCodeBench (arxiv:2403.07974, T1) §3.2 published 9-grams + Jaccard 0.3
as the contamination-detection sweet spot. This seed corpus is intentionally
small (8 fixtures across 3 sources) so the contamination filter has a
realistic baseline at first run. Production ingest paths extend this with:

- GitHubIssue: fresh issues post-cutoff with verifiable recency
- ARC-AGI public tasks (re-coded as BenchmarkFixture)
- Hand-curated math/prose problems

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

- `seed_synthetic.jsonl`: 5 hand-authored synthetic tasks, diverse 9-gram
  fingerprints, eligibility_window_ms=0 (active immediately).
- `seed_benchmark.jsonl`: 2 ARC-AGI-style structural transformation tasks
  re-coded as `benchmark_fixture` sources.
- `seed_github_issue.jsonl`: 1 issue-shaped task (placeholder; production
  ingest pulls from the GitHub API).
