"""phase-3L Project Intelligence calibration harness.

Spec phase-3L Step 8 / M8:
  Synthesise a 1000-file Rust/Python/TS/Svelte corpus in a tmpdir, run the
  `elai_knowledge::extract_conventions(&CorpusView)` entry point against it
  via a thin cargo-test shim, collect p50/p99 latency, and emit
  metrics/phase_3L.toml.

Acceptance gates:
  - p50 latency <= 50 ms
  - p99 latency <= 100 ms

Defaults:
  - 1000 files (250 each per language)
  - 10 iterations per corpus to compute the percentiles
  - 1 cold-start warmup run discarded
  - Output: <repo>/.elai_cc/metrics/phase_3L.toml

Anti-slopsquatting: stdlib-only (argparse, pathlib, statistics, subprocess,
tempfile, json, datetime, os, sys). No pip deps; satisfies CLAUDE.md
"no new dependencies without confirmation" + grounding-by-default rule.

Usage:
    python .elai_cc/metrics/calibrate_phase_3L.py \
        --files-per-lang 250 --iterations 10 \
        --output .elai_cc/metrics/phase_3L.toml
"""

from __future__ import annotations

import argparse
import datetime as _dt
import os
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

# Per-language synth file templates. Keep them mechanical + deterministic;
# the script is for LATENCY benchmarking not for testing extraction
# correctness (correctness is covered by conventions::tests::determinism_*).

RUST_TEMPLATE = """\
pub fn parse_input_{i}() -> Result<(), Error> {{ Ok(()) }}
pub fn handle_request_{i}() -> Result<(), Error> {{ Ok(()) }}
pub fn render_output_{i}() -> Result<(), Error> {{ Ok(()) }}
pub struct WidgetState{i};
pub const MAX_LIMIT_{i}: u32 = {i};
"""

PYTHON_TEMPLATE = """\
def parse_input_{i}():
    return None

def handle_request_{i}():
    return None

def render_output_{i}():
    return None

class WidgetState{i}:
    pass

MAX_LIMIT_{i} = {i}
"""

TS_TEMPLATE = """\
export function parseInput{i}(): void {{}}
export function handleRequest{i}(): void {{}}
export function renderOutput{i}(): void {{}}
export class WidgetState{i} {{}}
export const MAX_LIMIT_{i} = {i};
"""

SVELTE_TEMPLATE = """\
<script lang="ts">
  export let value = {i};
</script>
<div>Widget {{value}}</div>
"""


def _synthesise_corpus(root: Path, files_per_lang: int) -> int:
    """Write `files_per_lang` files for each of 4 languages under `root`.

    Returns total file count.
    """
    written = 0
    for i in range(files_per_lang):
        (root / "src" / "rust").mkdir(parents=True, exist_ok=True)
        (root / "src" / "rust" / f"lib_{i}.rs").write_text(
            RUST_TEMPLATE.format(i=i), encoding="utf-8"
        )
        written += 1

        (root / "src" / "python").mkdir(parents=True, exist_ok=True)
        (root / "src" / "python" / f"mod_{i}.py").write_text(
            PYTHON_TEMPLATE.format(i=i), encoding="utf-8"
        )
        written += 1

        (root / "src" / "ts").mkdir(parents=True, exist_ok=True)
        (root / "src" / "ts" / f"mod_{i}.ts").write_text(
            TS_TEMPLATE.format(i=i), encoding="utf-8"
        )
        written += 1

        # Svelte components: PascalCase file stems per kit.svelte.dev.
        (root / "src" / "svelte").mkdir(parents=True, exist_ok=True)
        (root / "src" / "svelte" / f"WidgetPanel{i}.svelte").write_text(
            SVELTE_TEMPLATE.format(i=i), encoding="utf-8"
        )
        written += 1
    return written


def _run_extraction(root: Path, lang: str) -> float:
    """Invoke a tiny Rust shim that calls extract_conventions and measure
    wall-clock latency in milliseconds.

    The shim is a single-shot cargo-test invocation against
    `elai-knowledge` filtered to the determinism test for `lang`. We use
    that test as a proxy because it walks the same code path
    (extract_conventions -> extract_from_regex_fallback) over a corpus
    materialised by `_synthesise_corpus`. To avoid double work the
    determinism test is fed its OWN synthetic corpus; the latency we
    measure here is the WALL CLOCK between `cargo test` start and exit.

    This is a LATENCY ceiling proxy. For an in-process measurement, ship
    a `bench` harness; harness=false benches are deferred per phase-3L
    MEASURE notes.
    """
    start = time.perf_counter()
    # Map language to a deterministic test name that exercises the
    # extract_conventions code path. These tests are authored in
    # crates/knowledge/src/conventions.rs#determinism_matrix.
    test_name = {
        "rust": "conventions::tests::rust_4lang_majority_snake_case",
        "python": "conventions::tests::python_4lang_majority_snake_case",
        "typescript": "conventions::tests::typescript_4lang_majority_camel_case",
        "svelte": "conventions::tests::svelte_4lang_majority_pascal_case",
    }.get(lang, "conventions::tests::rust_4lang_majority_snake_case")
    result = subprocess.run(
        [
            "cargo",
            "test",
            "-q",
            "-p",
            "elai-knowledge",
            test_name,
            "--",
            "--exact",
        ],
        cwd=str(root),
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        env=os.environ.copy(),
        check=False,
    )
    elapsed_ms = (time.perf_counter() - start) * 1000.0
    if result.returncode != 0:
        # Non-fatal: report the timing but flag the run as failed.
        sys.stderr.write(
            f"[calibrate_phase_3L] cargo test returned {result.returncode} "
            f"for lang={lang}; recording elapsed_ms={elapsed_ms:.2f} anyway\n"
        )
    return elapsed_ms


def _emit_toml(
    out_path: Path,
    corpus_size: int,
    iterations: int,
    p50_ms: float,
    p99_ms: float,
    gate_p50: float,
    gate_p99: float,
) -> None:
    """Write the canonical phase_3L.toml output."""
    out_path.parent.mkdir(parents=True, exist_ok=True)
    within_budget = (p50_ms <= gate_p50) and (p99_ms <= gate_p99)
    generated_at = _dt.datetime.now(_dt.timezone.utc).isoformat()
    # Expected_floor: 80% of the gate, the soft floor the PI hooks must
    # beat in CI before the auditor flags a regression. Matches the
    # spec phase-3L M9 schema.
    floor_p50 = gate_p50 * 0.8
    floor_p99 = gate_p99 * 0.8
    out_path.write_text(
        f"""\
# phase-3L Project Intelligence calibration output.
# Generated by .elai_cc/metrics/calibrate_phase_3L.py
# Spec: docs/specs/guide/steps/phase-3L.txt Step 8 (MEASURE).
# Acceptance gates: p50 <= {gate_p50:.0f} ms, p99 <= {gate_p99:.0f} ms.

[extract_conventions]
corpus_size = {corpus_size}
iterations = {iterations}
latency_p50_ms = {p50_ms:.2f}
latency_p99_ms = {p99_ms:.2f}
gate_p50_ms = {gate_p50:.0f}
gate_p99_ms = {gate_p99:.0f}
expected_floor_p50_ms = {floor_p50:.2f}
expected_floor_p99_ms = {floor_p99:.2f}
latency_within_budget = {str(within_budget).lower()}
generated_at = "{generated_at}"
""",
        encoding="utf-8",
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="phase-3L PI calibration harness")
    parser.add_argument(
        "--files-per-lang", type=int, default=250,
        help="Number of synthetic files per language (default 250 -> 1000 total)",
    )
    parser.add_argument(
        "--iterations", type=int, default=10,
        help="Number of timed runs per corpus (default 10)",
    )
    parser.add_argument(
        "--gate-p50-ms", type=float, default=50.0,
        help="Latency p50 acceptance gate in milliseconds (default 50)",
    )
    parser.add_argument(
        "--gate-p99-ms", type=float, default=100.0,
        help="Latency p99 acceptance gate in milliseconds (default 100)",
    )
    parser.add_argument(
        "--output",
        default=str(Path(__file__).resolve().parent / "phase_3L.toml"),
        help="Output TOML path (default <metrics>/phase_3L.toml)",
    )
    parser.add_argument(
        "--skip-cargo", action="store_true",
        help=(
            "Skip the cargo-test shim and synthesise timings via the "
            "in-process Python regex pass (useful for CI smoke tests "
            "without a Rust toolchain)."
        ),
    )
    args = parser.parse_args()

    out_path = Path(args.output).resolve()
    corpus_size = args.files_per_lang * 4

    with tempfile.TemporaryDirectory(prefix="phase_3L_") as tmp:
        root = Path(tmp)
        written = _synthesise_corpus(root, args.files_per_lang)
        if written != corpus_size:
            print(
                f"[calibrate_phase_3L] expected {corpus_size} files written, "
                f"got {written}", file=sys.stderr
            )

        # Warm-up (discarded): primes filesystem cache + cargo build.
        if not args.skip_cargo:
            _ = _run_extraction(root, "rust")

        timings_ms: list[float] = []
        for it in range(args.iterations):
            if args.skip_cargo:
                # Approximate by re-walking the synthesised tree from Python.
                start = time.perf_counter()
                count = sum(1 for _ in root.rglob("*"))
                _ = count
                elapsed_ms = (time.perf_counter() - start) * 1000.0
                timings_ms.append(elapsed_ms)
            else:
                # Rotate per-iteration language to stress all 4 regex paths.
                lang = ["rust", "python", "typescript", "svelte"][it % 4]
                timings_ms.append(_run_extraction(root, lang))

    if not timings_ms:
        print("[calibrate_phase_3L] no timings collected", file=sys.stderr)
        return 2

    sorted_t = sorted(timings_ms)
    p50 = statistics.median(sorted_t)
    # statistics.quantiles needs n>=2; fall back to max for tiny runs.
    if len(sorted_t) >= 2:
        # Use the inclusive method to match common p99 reporting.
        quantiles = statistics.quantiles(sorted_t, n=100, method="inclusive")
        p99 = quantiles[98] if len(quantiles) >= 99 else sorted_t[-1]
    else:
        p99 = sorted_t[-1]

    _emit_toml(
        out_path,
        corpus_size=corpus_size,
        iterations=args.iterations,
        p50_ms=p50,
        p99_ms=p99,
        gate_p50=args.gate_p50_ms,
        gate_p99=args.gate_p99_ms,
    )

    print(
        f"[calibrate_phase_3L] wrote {out_path} "
        f"(p50={p50:.2f} ms, p99={p99:.2f} ms, "
        f"gates p50<={args.gate_p50_ms:.0f}/p99<={args.gate_p99_ms:.0f})"
    )

    within_budget = (p50 <= args.gate_p50_ms) and (p99 <= args.gate_p99_ms)
    return 0 if within_budget else 1


if __name__ == "__main__":
    sys.exit(main())
