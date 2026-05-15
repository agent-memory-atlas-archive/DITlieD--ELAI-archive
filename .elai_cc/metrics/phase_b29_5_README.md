# B-29.5 Scale Benchmark Reproduction Guide

## Overview

This directory holds measurement artifacts from the B-29.5 scale benchmark.
The benchmark proves SymbolGraphCache + the four traversal tools (compute_impact_radius,
compute_call_flow, compute_symbol_context, list_symbols) meet p99 < 500 ms on four
open-source codebases up to Linux-kernel scale (~36M LOC).

## Hardware target

The official measurement run targets any developer workstation meeting:
- CPU: 8+ cores, x86-64
- RAM: 16+ GB (contract threshold: peak RSS <= 2048 MB during 1000-query workload)
- Disk: NVMe SSD recommended for indexing (affects index-build time, not query latency)

Runs on different hardware produce a sibling file `phase_b29_5_scale_<hostname>.toml`
rather than overwriting the official file.

## Tier commits (pinned)

| Tier         | Tag / Branch     | Approx LOC |
|--------------|------------------|------------|
| linux_kernel | v6.7             | ~36M       |
| chromium     | 121.0.6167.184   | ~35M       |
| vscode       | 1.94.2           | ~2.45M     |
| postgres     | REL_17_0         | ~1.5M      |
| elai_self    | current workspace| ~140K      |

## Step 1 — Clone fixtures

```bash
export ELAI_SCALE_FIXTURE_ROOT="${HOME}/elai_scale_fixtures"
bash scripts/fixtures/clone_scale_fixtures.sh
```

Expected wall time: 10-60 minutes depending on network speed (chromium and linux are
large shallow clones; chromium mirror may be slow).

## Step 2 — Build indices per tier

```bash
export ELAI_SCALE_FIXTURE_ROOT="${HOME}/elai_scale_fixtures"
for tier in linux chromium vscode postgres; do
    cargo run --release -- symbol_graph build \
        --project-root "${ELAI_SCALE_FIXTURE_ROOT}/${tier}" \
        --project-id "${tier}"
done
```

Expected wall time: 1-3 hours for linux + chromium combined (tree-sitter extraction
is CPU-bound; B-31 resumable indexing handles interrupts).

Index location: `${ELAI_SCALE_FIXTURE_ROOT}/<tier>/.elai_cc/indexes/dkg.db`

The ELAI workspace (elai_self) is indexed during normal development — no separate step.

## Step 3 — Run measurement benches

```bash
# symbol_graph_scale: per-tool latency (20 samples each)
cargo bench -p elai-knowledge --bench symbol_graph_scale 2>&1 | tee /tmp/bench_scale.log

# workload: 1000-query realistic mix
cargo bench -p elai-knowledge --bench workload 2>&1 | tee /tmp/bench_workload.log

# scale_memory: RSS snapshots
cargo bench -p elai-knowledge --bench scale_memory 2>&1 | tee /tmp/bench_memory.log
```

Results written to:
- `metrics/phase_b29_5_bench_raw.json`
- `metrics/phase_b29_5_workload_<tier>.toml` (one per tier)

## Step 4 — Populate metrics/phase_b29_5_scale.toml

Copy the measured numbers from the workload TOML outputs into the master
`metrics/phase_b29_5_scale.toml` file and set:

```toml
[meta]
populated_at = "2026-MM-DDTHH:MM:SSZ"
hardware = "your CPU model, RAM, disk type"
```

## Step 5 — Run contract tests

```bash
# p99 budget contract (reads phase_b29_5_scale.toml)
cargo test -p elai_cc_contracts scale_p99_budget

# Determinism regression (opt-in, ~2 s on ELAI self)
cargo test -p elai-knowledge --test scale_determinism -- --include-ignored
```

## File inventory

| File                                   | Purpose                                |
|----------------------------------------|----------------------------------------|
| phase_b29_5_scale.toml                 | Master measurement artifact (schema)   |
| phase_b29_5_fixture_inventory.toml     | Tier commit pins + LOC estimates       |
| phase_b29_5_bench_raw.json             | Raw per-tool JSON output (generated)   |
| phase_b29_5_workload_<tier>.toml       | Per-tier workload output (generated)   |
| phase_b29_5_README.md                  | This file                              |

## CI policy

Scale benchmarks do NOT run on every commit — they require pre-built indices
and take hours. They run:
- Manually before B-29 release
- Nightly on the CI schedule (future; see B-29.5 decisions doc)
- After any change to crates/knowledge/src/symbol_graph/

The p99 contract test (`scale_p99_budget_contract.rs`) runs on every CI pass but
skips gracefully when `populated_at = "null"`.
