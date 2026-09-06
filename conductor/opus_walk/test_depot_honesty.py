"""BURN-U1: dirty elai worktrees never take CI-on-main; full land-gate logs persist."""
from __future__ import annotations

import importlib
import subprocess
import sys
from pathlib import Path
from unittest import mock

import pytest

CONDUCTOR = Path(__file__).resolve().parents[1]
TOOLS = Path("/home/researcher/teikoku/Ougi/tools")
if str(CONDUCTOR) not in sys.path:
    sys.path.insert(0, str(CONDUCTOR))
if str(TOOLS) not in sys.path:
    sys.path.insert(0, str(TOOLS))

import depot_offload  # noqa: E402
import opus_walk.opus_land as opus_land  # noqa: E402


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


def _state(*, head="aaa", dirty=0, at_main_head=True, worktree="/tmp/wt"):
    return {
        "head": head,
        "dirty": dirty,
        "at_main_head": at_main_head,
        "worktree": worktree,
        "git_root": worktree,
        "main_sha": head if at_main_head else "bbb",
    }


class TestSelectRemotePath:
    """Routing decision table: clean-main / dirty / non-main-HEAD."""

    def test_clean_main_head_elai_selects_ci(self):
        st = _state(dirty=0, at_main_head=True)
        assert depot_offload.select_remote_path("elai", st, _cfg()) == "ci"
        assert depot_offload.elai_ci_eligible(st) is True

    def test_dirty_elai_selects_build(self):
        st = _state(dirty=1, at_main_head=True)
        assert depot_offload.select_remote_path("elai", st, _cfg()) == "build"
        assert depot_offload.elai_ci_eligible(st) is False

    def test_non_main_head_elai_selects_build(self):
        st = _state(dirty=0, at_main_head=False, head="deadbeef")
        assert depot_offload.select_remote_path("elai", st, _cfg()) == "build"
        assert depot_offload.elai_ci_eligible(st) is False

    def test_dirty_elai_refuses_when_build_fallback_off(self):
        st = _state(dirty=1, at_main_head=False)
        assert (
            depot_offload.select_remote_path("elai", st, _cfg(build_fallback=False))
            == "refuse"
        )

    def test_zenov_prefers_build(self):
        st = _state(dirty=0, at_main_head=True)
        assert depot_offload.select_remote_path("zenov", st, _cfg()) == "build"


class TestProvenanceFormat:
    def test_line_shape(self):
        line = depot_offload.format_depot_provenance(
            "build", "abc123", 1, "/workspace/ELAI"
        )
        assert line.startswith("DEPOT_PROVENANCE ")
        assert "path=build" in line
        assert "head=abc123" in line
        assert "dirty=1" in line
        assert "worktree=/workspace/ELAI" in line
        assert "path=ci" not in line

    def test_emit_prints_and_returns(self, capsys):
        st = _state(head="fff", dirty=0, worktree="/wt")
        line = depot_offload.emit_depot_provenance("ci", st)
        captured = capsys.readouterr()
        assert "DEPOT_PROVENANCE path=ci" in captured.out
        assert line == captured.out.strip()


class TestRunRemoteRouting:
    """run_remote must never call CI for dirty / non-main-HEAD elai."""

    def test_dirty_elai_never_calls_ci(self, tmp_path):
        cfg = _cfg()
        st = _state(dirty=1, at_main_head=True, worktree=str(tmp_path))
        build_ret = {
            "provider": "depot-build",
            "exit_code": 0,
            "output": "DEPOT_PROVENANCE path=build head=aaa dirty=1 worktree="
            + str(tmp_path),
            "request_id": "b1",
        }
        with mock.patch.object(depot_offload, "classify_cargo_cwd", return_value="elai"):
            with mock.patch.object(depot_offload, "inspect_worktree", return_value=st):
                with mock.patch.object(
                    depot_offload, "run_remote_depot_ci", side_effect=AssertionError("CI")
                ) as ci:
                    with mock.patch.object(
                        depot_offload, "run_remote_depot_build", return_value=build_ret
                    ) as build:
                        out = depot_offload.run_remote(
                            ["cargo", "check"], tmp_path, cfg=cfg
                        )
        ci.assert_not_called()
        build.assert_called_once()
        assert out["provider"] == "depot-build"
        assert "path=build" in out["output"]
        assert "dirty=1" in out["output"]

    def test_non_main_head_elai_never_calls_ci(self, tmp_path):
        cfg = _cfg()
        st = _state(dirty=0, at_main_head=False, head="branchsha", worktree=str(tmp_path))
        build_ret = {
            "provider": "depot-build",
            "exit_code": 1,
            "output": "DEPOT_PROVENANCE path=build head=branchsha dirty=0 worktree=x",
        }
        with mock.patch.object(depot_offload, "classify_cargo_cwd", return_value="elai"):
            with mock.patch.object(depot_offload, "inspect_worktree", return_value=st):
                with mock.patch.object(
                    depot_offload, "run_remote_depot_ci", side_effect=AssertionError("CI")
                ):
                    with mock.patch.object(
                        depot_offload, "run_remote_depot_build", return_value=build_ret
                    ):
                        out = depot_offload.run_remote(
                            ["cargo", "check"], tmp_path, cfg=cfg
                        )
        assert out["provider"] == "depot-build"

    def test_clean_main_elai_may_use_ci(self, tmp_path):
        cfg = _cfg()
        st = _state(dirty=0, at_main_head=True, worktree=str(tmp_path))
        ci_ret = {
            "provider": "depot-ci",
            "exit_code": 0,
            "output": "DEPOT_PROVENANCE path=ci head=aaa dirty=0 worktree=x",
        }
        with mock.patch.object(depot_offload, "classify_cargo_cwd", return_value="elai"):
            with mock.patch.object(depot_offload, "inspect_worktree", return_value=st):
                with mock.patch.object(
                    depot_offload, "run_remote_depot_ci", return_value=ci_ret
                ) as ci:
                    with mock.patch.object(
                        depot_offload,
                        "run_remote_depot_build",
                        side_effect=AssertionError("build"),
                    ):
                        out = depot_offload.run_remote(
                            ["cargo", "check"], tmp_path, cfg=cfg
                        )
        ci.assert_called_once()
        assert out["provider"] == "depot-ci"

    def test_ci_hard_guard_refuses_dirty_elai(self, tmp_path, capsys):
        """Refuse dirty CI AND still emit DEPOT_PROVENANCE (AC2; never path=ci)."""
        cfg = _cfg()
        st = _state(dirty=1, at_main_head=True, worktree=str(tmp_path))
        with mock.patch.object(depot_offload, "classify_cargo_cwd", return_value="elai"):
            with mock.patch.object(depot_offload, "inspect_worktree", return_value=st):
                with pytest.raises(depot_offload.DepotOffloadError) as ei:
                    depot_offload.run_remote_depot_ci(
                        ["cargo", "check"], tmp_path, cfg=cfg
                    )
        assert "D-257" in str(ei.value) or "refusing depot CI" in str(ei.value)
        out = capsys.readouterr().out
        assert "DEPOT_PROVENANCE" in out
        assert "dirty=1" in out
        assert "path=ci" not in out
        assert "path=refuse" in out
        # Exception body also carries the line so land gates that capture errors see it.
        assert "DEPOT_PROVENANCE" in str(ei.value)
        assert "path=ci" not in str(ei.value).split("DEPOT_PROVENANCE", 1)[-1].split("\n", 1)[0]

    def test_refuse_when_build_fallback_disabled(self, tmp_path, capsys):
        """run_remote refuse path also prints provenance before raising."""
        cfg = _cfg(build_fallback=False)
        st = _state(dirty=1, at_main_head=False, worktree=str(tmp_path))
        with mock.patch.object(depot_offload, "classify_cargo_cwd", return_value="elai"):
            with mock.patch.object(depot_offload, "inspect_worktree", return_value=st):
                with pytest.raises(depot_offload.DepotOffloadError) as ei:
                    depot_offload.run_remote(["cargo", "check"], tmp_path, cfg=cfg)
        assert "build_fallback" in str(ei.value) or "D-257" in str(ei.value)
        out = capsys.readouterr().out
        assert "DEPOT_PROVENANCE" in out
        assert "dirty=1" in out
        assert "path=ci" not in out
        assert "DEPOT_PROVENANCE" in str(ei.value)

    def test_dirty_elai_empty_command_emits_refuse_not_indexerror(self, tmp_path, capsys):
        """Empty command on dirty elai must not IndexError before DEPOT_PROVENANCE.

        Real entry path: run_remote → select build → run_remote_depot_build.
        Invalid input refuses with path=refuse (never path=ci, never IndexError).
        """
        cfg = _cfg()
        st = _state(dirty=1, at_main_head=True, worktree=str(tmp_path))
        with mock.patch.object(depot_offload, "classify_cargo_cwd", return_value="elai"):
            with mock.patch.object(depot_offload, "inspect_worktree", return_value=st):
                with mock.patch.object(
                    depot_offload, "run_remote_depot_ci", side_effect=AssertionError("CI")
                ):
                    with pytest.raises(depot_offload.DepotOffloadError) as ei:
                        depot_offload.run_remote([], tmp_path, cfg=cfg)
        out = capsys.readouterr().out
        assert "DEPOT_PROVENANCE" in out
        assert "path=refuse" in out
        assert "dirty=1" in out
        assert "path=ci" not in out
        assert "DEPOT_PROVENANCE" in str(ei.value)
        assert "path=refuse" in str(ei.value)
        assert "IndexError" not in type(ei.value).__name__

    def test_build_path_empty_command_emits_refuse(self, tmp_path, capsys):
        """Direct run_remote_depot_build([]) also refuses with provenance."""
        cfg = _cfg()
        st = _state(dirty=1, worktree=str(tmp_path))
        with mock.patch.object(depot_offload, "inspect_worktree", return_value=st):
            with pytest.raises(depot_offload.DepotOffloadError) as ei:
                depot_offload.run_remote_depot_build([], tmp_path, cfg=cfg)
        out = capsys.readouterr().out
        assert "path=refuse" in out
        assert "DEPOT_PROVENANCE" in str(ei.value)


class TestRemoteFailureProvenance:
    """Timeout / missing-binary must keep DEPOT_PROVENANCE through the gate adapter.

    execute_gate_via_offload only surfaces str(exc); if the exception body drops
    the line, land gates never see it (AC2 / skeptic gap at build:610-615).
    """

    def test_build_timeout_exception_carries_provenance(self, tmp_path, capsys):
        cfg = _cfg()
        st = _state(dirty=1, at_main_head=True, worktree=str(tmp_path))
        with mock.patch.object(depot_offload, "inspect_worktree", return_value=st):
            with mock.patch.object(
                depot_offload.subprocess,
                "run",
                side_effect=subprocess.TimeoutExpired(cmd=["depot"], timeout=1),
            ):
                with pytest.raises(depot_offload.DepotOffloadError) as ei:
                    depot_offload.run_remote_depot_build(
                        ["cargo", "check"], tmp_path, cfg=cfg, timeout_sec=1
                    )
        msg = str(ei.value)
        assert "DEPOT_PROVENANCE" in msg
        assert "path=build" in msg
        assert "dirty=1" in msg
        assert "timed out" in msg
        printed = capsys.readouterr().out
        assert "DEPOT_PROVENANCE path=build" in printed

    def test_build_missing_binary_exception_carries_provenance(self, tmp_path, capsys):
        cfg = _cfg(depot_bin="depot-bin-does-not-exist-burn-u1")
        st = _state(dirty=1, at_main_head=False, worktree=str(tmp_path), head="dead")
        with mock.patch.object(depot_offload, "inspect_worktree", return_value=st):
            with mock.patch.object(
                depot_offload.shutil, "which", return_value=None
            ):
                with mock.patch.object(
                    depot_offload.subprocess,
                    "run",
                    side_effect=FileNotFoundError("depot"),
                ):
                    with pytest.raises(depot_offload.DepotOffloadError) as ei:
                        depot_offload.run_remote_depot_build(
                            ["cargo", "check"], tmp_path, cfg=cfg
                        )
        msg = str(ei.value)
        assert "DEPOT_PROVENANCE" in msg
        assert "path=build" in msg
        assert "dirty=1" in msg
        assert "binary missing" in msg or "depot" in msg.lower()

    def test_execute_gate_adapter_surfaces_timeout_provenance(self, tmp_path, monkeypatch):
        """Real adapter boundary: execute_gate_via_offload output includes provenance."""
        cfg = _cfg(mode="remote")
        st = _state(dirty=1, at_main_head=True, worktree=str(tmp_path))
        monkeypatch.setattr(
            depot_offload.OffloadConfig, "load", staticmethod(lambda path=None: cfg)
        )
        with mock.patch.object(
            depot_offload, "preflight", return_value={"ready": True, "org": cfg.org}
        ):
            with mock.patch.object(depot_offload, "classify_cargo_cwd", return_value="elai"):
                with mock.patch.object(depot_offload, "inspect_worktree", return_value=st):
                    with mock.patch.object(
                        depot_offload.subprocess,
                        "run",
                        side_effect=subprocess.TimeoutExpired(
                            cmd=["depot", "build"], timeout=2
                        ),
                    ):
                        out = depot_offload.execute_gate_via_offload(
                            "cargo_check",
                            ["cargo", "check"],
                            tmp_path,
                            timeout_sec=2,
                        )
        assert out["exit_code"] == 2
        assert "FAIL-CLOSED" in out["output"]
        assert "DEPOT_PROVENANCE" in out["output"]
        assert "path=build" in out["output"]
        assert "dirty=1" in out["output"]
        assert "path=ci" not in out["output"]
        # offload_error field is the same surface land consumers may log
        assert "DEPOT_PROVENANCE" in (out.get("offload_error") or "")


class TestInspectWorktree:
    def test_dirty_flag_from_porcelain(self, tmp_path):
        subprocess.run(["git", "init"], cwd=tmp_path, check=True, capture_output=True)
        subprocess.run(
            ["git", "config", "user.email", "t@t"],
            cwd=tmp_path,
            check=True,
            capture_output=True,
        )
        subprocess.run(
            ["git", "config", "user.name", "t"],
            cwd=tmp_path,
            check=True,
            capture_output=True,
        )
        (tmp_path / "f").write_text("a")
        subprocess.run(
            ["git", "add", "f"], cwd=tmp_path, check=True, capture_output=True
        )
        subprocess.run(
            ["git", "commit", "-m", "i"],
            cwd=tmp_path,
            check=True,
            capture_output=True,
        )
        clean = depot_offload.inspect_worktree(tmp_path)
        assert clean["dirty"] == 0
        assert clean["head"] not in ("", "unknown")
        (tmp_path / "f").write_text("dirty")
        dirty = depot_offload.inspect_worktree(tmp_path)
        assert dirty["dirty"] == 1


class TestLandGateLogPersistence:
    def test_full_log_written_and_tail_truncated(self, monkeypatch, tmp_path):
        """Full output lands on disk; JSON tail stays at most 1500 chars."""
        long_body = ("LINE-%05d-abcdef\n" % i for i in range(400))
        full = "".join(long_body)
        assert len(full) > 1500
        completed = subprocess.CompletedProcess(
            ["python3", "check.py"], 0, full, ""
        )
        completed.provider = "local"
        monkeypatch.setattr(
            opus_land.depot_cargo,
            "run_gate_command",
            lambda *a, **k: completed,
        )
        unit = "BURN-U1-TEST"
        entries, error = opus_land.run_gate_leg(
            ["python3 check.py"],
            tmp_path,
            {},
            plan_id=unit,
            log_root=tmp_path / "evidence",
        )
        assert error is None
        assert len(entries) == 1
        assert entries[0]["tail"] == full[-1500:]
        assert len(entries[0]["tail"]) == 1500
        log_path = Path(entries[0]["log_path"])
        assert log_path.is_file()
        assert log_path.read_text(encoding="utf-8") == full
        assert "land-gate-logs" in str(log_path)
        assert unit in str(log_path)

    def test_persist_helper_slug_and_index(self, tmp_path):
        p = opus_land.persist_gate_command_log(
            "UNIT-X",
            0,
            "cargo check -p elai-ipc-types",
            "hello-full-log",
            root=tmp_path,
        )
        assert p is not None
        assert p.name.startswith("0_")
        assert "cargo" in p.name
        assert p.read_text() == "hello-full-log"

    def test_no_plan_id_skips_disk_but_keeps_tail(self, monkeypatch, tmp_path):
        completed = subprocess.CompletedProcess(["x"], 0, "abc" * 600, "")
        monkeypatch.setattr(
            opus_land.depot_cargo,
            "run_gate_command",
            lambda *a, **k: completed,
        )
        entries, error = opus_land.run_gate_leg(
            ["python3 check.py"], tmp_path, {}, plan_id=None
        )
        assert error is None
        assert "log_path" not in entries[0]
        assert entries[0]["tail"] == ("abc" * 600)[-1500:]
