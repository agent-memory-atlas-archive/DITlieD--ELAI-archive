"""Smoke test: the deterministic GEPA scaffold loop returns a non-
trivial OptimizedTemplate envelope for a 5-row fixture."""

from pathlib import Path
import tempfile

from elai_optimizer.optimizer import (
    OptimizedTemplate,
    _refuse_local_reflection_lm,
    optimize_template,
)
import pytest


def _write(path: Path, content: str) -> None:
    path.write_text(content, encoding="utf-8")


def test_optimize_template_returns_non_trivial_envelope():
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        template = tmp / "template.txt"
        train = tmp / "train.toml"
        val = tmp / "val.toml"
        _write(template, "parent role prompt body")
        _write(train, "\n".join(f"row-{i}" for i in range(5)))
        _write(val, "\n".join(f"row-{i}" for i in range(5)))

        env = optimize_template(
            template_path=str(template),
            train_set_path=str(train),
            val_set_path=str(val),
        )
        assert isinstance(env, OptimizedTemplate)
        assert env.optimization_run_id
        assert env.content != "parent role prompt body"
        assert env.content_blake3 != env.parent_template_blake3
        assert env.val_lift_pct >= 5.0  # admit path requires >= 5.0
        assert env.held_out_lift_pct >= env.val_lift_pct - 2.0  # epsilon


def test_local_reflection_lm_refused():
    with pytest.raises(ValueError, match="arXiv:2405.10276"):
        _refuse_local_reflection_lm("qwen2.5-coder-7b")


def test_local_3b_refused():
    with pytest.raises(ValueError):
        _refuse_local_reflection_lm("Local-3B-test")


def test_cloud_sonnet_accepted():
    # No exception raised.
    _refuse_local_reflection_lm("claude-4.6-sonnet")
    _refuse_local_reflection_lm("gpt-4o-2024-11")


def test_include_cross_domain_populates_canaries():
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        template = tmp / "template.txt"
        train = tmp / "train.toml"
        val = tmp / "val.toml"
        _write(template, "parent body")
        _write(train, "x")
        _write(val, "x")
        env = optimize_template(
            template_path=str(template),
            train_set_path=str(train),
            val_set_path=str(val),
            include_cross_domain=True,
        )
        assert len(env.cross_domain_canary_results) >= 2
        families = {c.family for c in env.cross_domain_canary_results}
        assert "math" in families
        assert "code" in families


def test_dry_run_skips_canaries_by_default():
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        template = tmp / "template.txt"
        train = tmp / "train.toml"
        val = tmp / "val.toml"
        _write(template, "parent body")
        _write(train, "x")
        _write(val, "x")
        env = optimize_template(
            template_path=str(template),
            train_set_path=str(train),
            val_set_path=str(val),
            include_cross_domain=False,
        )
        assert env.cross_domain_canary_results == []
