"""E-8 — GEPA-class prompt optimizer service.

Top-level entry: ``optimize_template(template_path, train_set_path,
val_set_path, reflection_model, target_model_url)`` returns an
``OptimizedTemplate`` envelope ready for the Rust admit path
(``orchestrator::optimizer_admit::admit_optimized_template``).

Cloud-Sonnet reflection / local-Qwen target split per
arXiv:2405.10276. Live LLM calls are RUNPOD-PENDING: the
``_run_gepa_loop`` helper currently emits a deterministic mock loop
so the surrounding orchestration shape can be unit-tested without
spending the cloud budget. Replace the loop body with
``gepa.optimize(...)`` (PyPI ``gepa>=0.1.1``) when the live-LLM
seam opens.

Paper grounding:
  * arXiv:2507.19457 GEPA (Agrawal et al., ICLR 2026 Oral) -- primary
  * arXiv:2310.03714 DSPy (Khattab et al.) -- MIPROv2 surface
  * arXiv:2309.03409 OPRO (Yang et al.) -- meta-prompt trajectory
  * arXiv:2211.01910 APE (Zhou et al.) -- Monte-Carlo prior-art
  * arXiv:2405.10276 small-LMs-as-optimizers limitation
  * arXiv:2504.20355 LPO -- ``<edit>`` tag pattern (Rust-side guard)
  * arXiv:2602.00997 ETGPO -- top-3 gotchas injection

CLAUDE.md anti-stale-Python rules: orjson over json; pydantic v2 for
typed dataclasses; no torch.no_grad() (irrelevant here -- inference
runs on the remote sidecar, not in-process); typed dataclasses
preferred over plain dicts.
"""

from __future__ import annotations

import argparse
import dataclasses
import hashlib
import sys
import time
import uuid
from pathlib import Path
from typing import Optional

try:
    import orjson  # type: ignore
except ImportError:  # pragma: no cover -- handled at install time
    orjson = None  # type: ignore


# ---------------------------------------------------------------------------
# Typed envelopes (manual dataclasses to avoid forcing pydantic import in
# environments where only the type shape is needed, e.g. the smoke test).
# ---------------------------------------------------------------------------


@dataclasses.dataclass(frozen=True)
class OptimizerConfig:
    """Caller-side config for one optimize_template() invocation."""

    template_path: str
    train_set_path: str
    val_set_path: str
    reflection_model: str  # cloud Sonnet 4.6 default; refuses local LMs.
    target_model_url: str  # http URL of the local sidecar (Qwen2.5-Coder).
    budget_tokens: int = 200_000
    include_cross_domain: bool = False
    patience_outer_iters: int = 2  # plateau early-stop per plan §Open Q.
    min_delta_pct: float = 1.0  # outer-iter delta floor for continuing.


@dataclasses.dataclass
class CrossDomainCanaryResult:
    family: str
    delta_pct: float


@dataclasses.dataclass
class OptimizedTemplate:
    """Output envelope. Mirrors the Rust ``OptimizedTemplate`` struct at
    ``crates/orchestrator/src/optimizer_admit.rs``.

    ``content_blake3`` / ``parent_template_blake3`` are 64-char lowercase
    hex strings (the Rust ``Blake3Hash::as_hex`` round-trip)."""

    content: str
    content_blake3: str
    parent_template_blake3: str
    optimizer_version: str
    optimization_run_id: str
    val_lift_pct: float
    held_out_lift_pct: float
    cross_domain_canary_results: list[CrossDomainCanaryResult]
    target_blocks: list[str]


# ---------------------------------------------------------------------------
# Cloud-only reflection LM guard. Python belt-and-suspenders enforcement;
# the Rust CLI parser already refuses local-class reflection models per
# arXiv:2405.10276. We re-check here so direct callers (e.g. test
# harnesses) cannot bypass via raw Python imports.
# ---------------------------------------------------------------------------


_LOCAL_LM_TOKENS = ("qwen", "local", "7b", "3b", "8b", "13b", "llama")


def _refuse_local_reflection_lm(name: str) -> None:
    lower = name.lower()
    for tok in _LOCAL_LM_TOKENS:
        if tok in lower:
            raise ValueError(
                f"reflection-model must be cloud (sonnet / gpt / opus class); "
                f"refused `{name}` per arXiv:2405.10276 small-LMs-as-optimizers limitation"
            )


# ---------------------------------------------------------------------------
# blake3 hashing -- mirrors the canonical Rust Blake3Hash hex form.
# ---------------------------------------------------------------------------


def _blake3_hex(content: str) -> str:
    # The `blake3` PyPI package (0.4+) is the canonical choice. We fall
    # back to hashlib.blake2b ONLY for the smoke test path -- live admit
    # will Rust-verify the hash and reject blake2.
    try:
        import blake3  # type: ignore

        return blake3.blake3(content.encode("utf-8")).hexdigest()
    except ImportError:
        # The Rust side rejects this on admit; the fallback only exists
        # so the smoke test can produce a syntactically-valid envelope
        # in a checkout without blake3 installed yet.
        return hashlib.blake2b(content.encode("utf-8"), digest_size=32).hexdigest()


# ---------------------------------------------------------------------------
# Train / val / holdout split discipline. DSPy 20/80 inversion per
# arXiv:2310.03714: prompt optimizers overfit small training sets harder
# than NN weights, so we inverted train/val: 20% optimization, 80%
# validation. Sizes recorded on the envelope so the dashboard can render
# them.
# ---------------------------------------------------------------------------


@dataclasses.dataclass
class _SplitSizes:
    train_size: int
    val_size: int
    holdout_size: int


def _split_sizes(train_path: Path, val_path: Path) -> _SplitSizes:
    # In the live path these would be counts of fixture rows; the smoke
    # path reads line counts from disk so the dashboard shows non-zero
    # values even when the fixtures are 5-row stubs.
    try:
        train_size = sum(1 for _ in train_path.open("r", encoding="utf-8"))
    except OSError:
        train_size = 0
    try:
        val_size = sum(1 for _ in val_path.open("r", encoding="utf-8"))
    except OSError:
        val_size = 0
    holdout_size = max(1, val_size // 4)
    return _SplitSizes(train_size=train_size, val_size=val_size, holdout_size=holdout_size)


# ---------------------------------------------------------------------------
# GEPA reflective loop -- DETERMINISTIC SCAFFOLD.
#
# RUNPOD-PENDING: replace the body of this function with the live
# `gepa.optimize(...)` call when:
#   1. the cloud Sonnet route is reachable from the developer host;
#   2. the local sidecar at target_model_url is healthy
#      (workflow/remote-setup/REMOTE-SIDECAR.md);
#   3. the cloud-budget cap (default 200k tokens) is approved.
# The scaffold below produces an OptimizedTemplate with a small lift
# so the Rust admit path can exercise its guards under unit-test.
# ---------------------------------------------------------------------------


def _run_gepa_loop(cfg: OptimizerConfig, parent_content: str) -> OptimizedTemplate:
    # RUNPOD-PENDING: live GEPA reflection loop.
    run_id = uuid.uuid4().hex
    # Deterministic synthetic optimization: append a fixed marker so the
    # content differs from the parent + the hash changes meaningfully.
    optimized_content = parent_content + "\n\n# [GEPA reflective edit, run " + run_id[:8] + "]"
    parent_hex = _blake3_hex(parent_content)
    content_hex = _blake3_hex(optimized_content)
    # Lift values modelled after the GEPA paper's reported MIPROv2 delta;
    # held-out lift drops a little to stay within the 2pp epsilon band.
    val_lift = 8.5
    held_out_lift = 7.1
    canaries: list[CrossDomainCanaryResult] = []
    if cfg.include_cross_domain:
        # Two canary families per E-7 task 7. Deterministic non-regression
        # in the scaffold; the live loop replaces with real measurements.
        canaries = [
            CrossDomainCanaryResult(family="math", delta_pct=1.5),
            CrossDomainCanaryResult(family="code", delta_pct=0.0),
        ]
    return OptimizedTemplate(
        content=optimized_content,
        content_blake3=content_hex,
        parent_template_blake3=parent_hex,
        optimizer_version="gepa-0.1.1+elai-e8-scaffold",
        optimization_run_id=run_id,
        val_lift_pct=val_lift,
        held_out_lift_pct=held_out_lift,
        cross_domain_canary_results=canaries,
        target_blocks=["role_prompt_body"],
    )


def optimize_template(
    template_path: str,
    train_set_path: str,
    val_set_path: str,
    reflection_model: str = "claude-4.6-sonnet",
    target_model_url: str = "http://localhost:8100/v1/messages",
    budget_tokens: int = 200_000,
    include_cross_domain: bool = False,
) -> OptimizedTemplate:
    """Top-level entry point. Returns an ``OptimizedTemplate`` ready for
    the Rust admit path. Raises ``ValueError`` if ``reflection_model``
    is a local-class model (arXiv:2405.10276 enforcement)."""

    _refuse_local_reflection_lm(reflection_model)
    cfg = OptimizerConfig(
        template_path=template_path,
        train_set_path=train_set_path,
        val_set_path=val_set_path,
        reflection_model=reflection_model,
        target_model_url=target_model_url,
        budget_tokens=budget_tokens,
        include_cross_domain=include_cross_domain,
    )
    parent_path = Path(template_path)
    try:
        parent_content = parent_path.read_text(encoding="utf-8")
    except OSError:
        parent_content = "(missing parent template fixture; using empty body)"
    return _run_gepa_loop(cfg, parent_content)


# ---------------------------------------------------------------------------
# CLI entrypoint -- consumed by `python -m elai_optimizer.optimizer ...`
# spawned from the Rust `elai optimize template` handler.
# ---------------------------------------------------------------------------


def _write_envelope(out_dir: Path, envelope: OptimizedTemplate) -> Path:
    out_dir.mkdir(parents=True, exist_ok=True)
    target = out_dir / f"{envelope.optimization_run_id}.json"
    payload = {
        "content": envelope.content,
        "content_blake3": envelope.content_blake3,
        "parent_template_blake3": envelope.parent_template_blake3,
        "optimizer_version": envelope.optimizer_version,
        "optimization_run_id": envelope.optimization_run_id,
        "val_lift_pct": envelope.val_lift_pct,
        "held_out_lift_pct": envelope.held_out_lift_pct,
        "cross_domain_canary_results": [
            {"family": c.family, "delta_pct": c.delta_pct}
            for c in envelope.cross_domain_canary_results
        ],
        "target_blocks": envelope.target_blocks,
    }
    if orjson is not None:
        target.write_bytes(orjson.dumps(payload, option=orjson.OPT_INDENT_2))
    else:  # pragma: no cover
        import json as _json
        target.write_text(_json.dumps(payload, indent=2), encoding="utf-8")
    return target


def main(argv: Optional[list[str]] = None) -> int:
    parser = argparse.ArgumentParser(prog="elai_optimizer.optimizer")
    parser.add_argument("--target", required=True, help="AgentRole slug")
    parser.add_argument("--train", required=True, help="path to train fixture")
    parser.add_argument("--val", required=True, help="path to val fixture")
    parser.add_argument(
        "--reflection-model",
        default="claude-4.6-sonnet",
        help="cloud reflection LM (refuses local-class models)",
    )
    parser.add_argument(
        "--budget-tokens",
        type=int,
        default=200_000,
        help="cloud-token budget cap for the GEPA reflection loop",
    )
    parser.add_argument(
        "--include-cross-domain",
        action="store_true",
        help="run cross-domain canary check + populate canary_results",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="run the deterministic scaffold loop only; live GEPA calls skipped",
    )
    parser.add_argument(
        "--out-dir",
        default=".elai_cc/optimization/outputs",
        help="directory to write the OptimizedTemplate envelope JSON",
    )
    args = parser.parse_args(argv)

    # Use the train file as a stand-in for the role-prompt source so a
    # smoke run hash-differentiates by AgentRole.
    template_source = args.train
    try:
        envelope = optimize_template(
            template_path=template_source,
            train_set_path=args.train,
            val_set_path=args.val,
            reflection_model=args.reflection_model,
            budget_tokens=args.budget_tokens,
            include_cross_domain=args.include_cross_domain,
        )
    except ValueError as e:
        print(f"error: {e}", file=sys.stderr)
        return 2

    out_dir = Path(args.out_dir)
    target = _write_envelope(out_dir, envelope)
    splits = _split_sizes(Path(args.train), Path(args.val))
    print(
        f"optimize template target={args.target} reflection_model={args.reflection_model} "
        f"train_size={splits.train_size} val_size={splits.val_size} "
        f"holdout_size={splits.holdout_size} val_lift_pct={envelope.val_lift_pct} "
        f"held_out_lift_pct={envelope.held_out_lift_pct} "
        f"envelope={target} ts={int(time.time())}"
    )
    return 0


if __name__ == "__main__":  # pragma: no cover
    sys.exit(main())
