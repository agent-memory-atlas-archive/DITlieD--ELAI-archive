"""S-9 symbol_graph scenario: drives `elai symbol_graph build` on a tiny
source fixture and asserts the STDOUT reports a NON-EMPTY graph (symbols>0).
Off-agent artifact = the bench's STDOUT 'Symbol graph built' line + the
resulting dkg.db; the CLI build path emits NO DkgRebuild SSB row (the plan's
SSB-row assumption is grounded-wrong, those rows are emitted only via
knowledge_ssb_sink's emit_dkg_rebuild trait, not the CLI build path)."""

from __future__ import annotations

import os
import subprocess
import sys

from pathlib import Path

from runner import RunResult, register

# The runner resolves the binary via runner.DEFAULT_BINARY (walks up to find
# .elai_cc/crates). If absent there, scenarios fall back to the env override
# ELAI_E2E_BINARY or the warm cargo target.
def _resolve_binary() -> Path:
    import runner
    if runner.DEFAULT_BINARY.exists():
        return runner.DEFAULT_BINARY
    env_bin = os.environ.get("ELAI_E2E_BINARY")
    if env_bin and Path(env_bin).exists():
        return Path(env_bin)
    warm = Path.home() / ".cache" / "cargo-target-elai" / "debug" / "elai"
    if warm.exists():
        return warm
    return runner.DEFAULT_BINARY  # let the runner's own error fire


@register("s9_symbol_graph", ["symbol_graph", "build", "--project-root", "."])
def s9_symbol_graph(result: RunResult) -> None:
    # The static-argv register above would fail (no indexes dir in a fresh
    # home). Drive correctly here: tiny fixture + pre-created indexes dir in
    # the home the runner gave us.
    home = result.home
    src_dir = home / "src"
    src_dir.mkdir(parents=True, exist_ok=True)
    # TWO source files so there are cross-file edges (a single file yields
    # edges=0, which is valid but a weaker functional assertion).
    (src_dir / "a.rs").write_text(
        "pub fn add(a: i32, b: i32) -> i32 { a + b }\n"
        "pub struct Point { x: f64, y: f64 }\n",
        encoding="utf-8",
    )
    (src_dir / "b.rs").write_text(
        "use crate::a::{add, Point};\n"
        "pub fn dist(p: Point) -> f64 { (p.x*p.x + p.y*p.y).sqrt() }\n",
        encoding="utf-8",
    )
    # GROUNDED FIX: the build path does NOT create .elai_cc/indexes/, so it
    # fails on a fresh project. Pre-create it.
    (home / ".elai_cc" / "indexes").mkdir(parents=True, exist_ok=True)

    binary = _resolve_binary()
    proc = subprocess.run(
        [str(binary), "symbol_graph", "build", "--project-root", str(home)],
        cwd=str(home), capture_output=True, text=True, timeout=120,
    )
    assert proc.returncode == 0, \
        f"expected exit 0, got {proc.returncode}; stderr={proc.stderr[:300]}"
    # The off-agent artifact is the STDOUT verdict line.
    assert "Symbol graph built" in proc.stdout, \
        f"FUNCTIONAL FAIL: 'Symbol graph built' line missing from stdout; " \
        f"got stdout={proc.stdout[:400]!r}"
    # Functional content assertion: the graph is NON-EMPTY (real symbols).
    # Parse the 'symbols=N edges=N' fields out of the printed line.
    import re
    m = re.search(r"symbols=(\d+)\s+edges=(\d+)", proc.stdout)
    assert m, \
        f"FUNCTIONAL FAIL: could not parse symbols/edges counts from stdout; " \
        f"got {proc.stdout[:400]!r}"
    symbols = int(m.group(1))
    assert symbols > 0, \
        f"FUNCTIONAL FAIL: empty rebuild, symbols={symbols} (a completion row " \
        f"with an empty graph is a real gap, recorded not passed); " \
        f"stdout={proc.stdout[:300]!r}"
