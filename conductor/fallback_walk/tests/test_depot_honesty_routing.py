"""BURN-U1: import-level contract that dirty elai never selects CI (fallback suite)."""
from __future__ import annotations

import sys
from pathlib import Path

TOOLS = Path("/home/researcher/teikoku/Ougi/tools")
if str(TOOLS) not in sys.path:
    sys.path.insert(0, str(TOOLS))

import depot_offload  # noqa: E402


def _cfg(**overrides):
    base = dict(
        mode="remote",
        org="example-org",
        teikoku_repo="example/automation-workspace",
        elai_repo="example/ELAI",
        project_id="example-project",
        workflow_teikoku=".depot/workflows/conductor-cargo.yml",
        workflow_elai=".depot/workflows/elai-cargo.yml",
        timeout_sec=60,
        depot_bin="depot",
        ci_repos=("example/ELAI",),
        build_fallback=True,
    )
    base.update(overrides)
    return depot_offload.OffloadConfig(**base)


def _state(*, head="abc", dirty=0, at_main_head=True, worktree="/wt"):
    return {
        "head": head,
        "dirty": dirty,
        "at_main_head": at_main_head,
        "worktree": worktree,
        "git_root": worktree,
        "main_sha": head if at_main_head else "mainsha",
    }


def test_select_remote_path_dirty_elai_is_build_not_ci():
    cfg = _cfg()
    dirty = _state(dirty=1, at_main_head=True)
    assert depot_offload.select_remote_path("elai", dirty, cfg) == "build"
    line = depot_offload.format_depot_provenance("build", "abc", 1, "/wt")
    assert line.startswith("DEPOT_PROVENANCE ")
    assert "dirty=1" in line
    assert "path=ci" not in line


def test_select_remote_path_clean_main_elai_may_use_ci():
    """Clean main-HEAD elai keeps the faster CI path (AC2)."""
    cfg = _cfg()
    clean = _state(dirty=0, at_main_head=True)
    assert depot_offload.select_remote_path("elai", clean, cfg) == "ci"
    assert depot_offload.elai_ci_eligible(clean) is True


def test_select_remote_path_non_main_head_elai_is_build():
    """Non-main-HEAD elai must not take CI-on-main (D-257)."""
    cfg = _cfg()
    diverged = _state(dirty=0, at_main_head=False, head="deadbeef")
    assert depot_offload.select_remote_path("elai", diverged, cfg) == "build"
    assert depot_offload.elai_ci_eligible(diverged) is False


def test_select_remote_path_dirty_refuses_when_build_fallback_off():
    cfg = _cfg(build_fallback=False)
    dirty = _state(dirty=1, at_main_head=False)
    assert depot_offload.select_remote_path("elai", dirty, cfg) == "refuse"


def test_provenance_line_shape_for_build_and_refuse():
    build = depot_offload.format_depot_provenance(
        "build", "sha1", 1, "/workspace/ELAI"
    )
    assert build == (
        "DEPOT_PROVENANCE path=build head=sha1 dirty=1 "
        "worktree=/workspace/ELAI"
    )
    refuse = depot_offload.format_depot_provenance("refuse", "sha1", 1, "/wt")
    assert "path=refuse" in refuse
    assert "path=ci" not in refuse
