from __future__ import annotations

import os
import pytest
import subprocess
import sys
from pathlib import Path
from unittest import mock

CONDUCTOR = Path(__file__).resolve().parents[1]
if str(CONDUCTOR) not in sys.path:
    sys.path.insert(0, str(CONDUCTOR))

from opus_walk import depot_cargo


class TestDepotCargo:
    def test_local_non_cargo_passthrough(self, monkeypatch, tmp_path):
        monkeypatch.setenv("TEIKOKU_DEPOT_OFFLOAD", "local")
        completed = subprocess.CompletedProcess(["cargo", "check"], 0, "ok", "")
        with mock.patch.object(depot_cargo.subprocess, "run", return_value=completed) as run:
            result = depot_cargo.run_gate_command(["cargo", "check"], tmp_path, 10)
        run.assert_called_once()
        assert result.args == completed.args
        assert result.returncode == 0
        assert result.provider == "local"

    def test_remote_heavy_calls_offload(self, monkeypatch, tmp_path):
        monkeypatch.setenv("TEIKOKU_DEPOT_OFFLOAD", "remote")
        raw = {
            "provider": "depot-build",
            "exit_code": 0,
            "output": "remote output",
            "request_id": "build-123",
            "offload_mode": "remote",
        }
        with mock.patch.object(depot_cargo.depot_offload, "run_offloaded", return_value=raw) as offload:
            with mock.patch.object(depot_cargo.subprocess, "run") as local:
                result = depot_cargo.run_gate_command(["cargo", "check"], tmp_path, 10)
        offload.assert_called_once()
        local.assert_not_called()
        assert result.returncode == 0
        assert result.depot_id == "build-123"
        assert result.stdout == "remote output"

    def test_remote_preflight_failure_fails_closed(self, monkeypatch, tmp_path):
        monkeypatch.setenv("TEIKOKU_DEPOT_OFFLOAD", "remote")
        remote_error = depot_cargo.depot_offload.DepotOffloadError("offline")
        with mock.patch.object(
            depot_cargo.depot_offload,
            "run_offloaded",
            side_effect=remote_error,
        ) as offload:
            with mock.patch.object(depot_cargo.subprocess, "run") as local:
                result = depot_cargo.run_gate_command(["cargo", "check"], tmp_path, 10)
        offload.assert_called_once()
        local.assert_not_called()
        assert result.returncode != 0
        assert "FAIL-CLOSED" in result.stdout
        assert "offline" in result.stdout

    def test_prefer_falls_back_to_local(self, monkeypatch, tmp_path):
        monkeypatch.setenv("TEIKOKU_DEPOT_OFFLOAD", "prefer")
        remote_error = depot_cargo.depot_offload.DepotOffloadError("remote unavailable")
        local_result = subprocess.CompletedProcess(["cargo", "check"], 0, "local", "")
        with mock.patch.object(depot_cargo.depot_offload, "run_offloaded", side_effect=remote_error):
            with mock.patch.object(depot_cargo.subprocess, "run", return_value=local_result) as local:
                result = depot_cargo.run_gate_command(["cargo", "check"], tmp_path, 10)
        local.assert_called_once()
        assert result.returncode == 0
        assert result.stdout == "local"

    def test_opus_land_gate_leg_calls_adapter(self, monkeypatch, tmp_path):
        """NON-heavy gate commands still route through the depot adapter."""
        sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
        import opus_walk.opus_land as opus_land

        calls = []
        completed = subprocess.CompletedProcess(["cargo", "fmt"], 0, "gate", "")
        completed.depot_id = "run-456"

        def fake_gate(cmd, cwd, timeout_s, *, env=None):
            calls.append((cmd, cwd, timeout_s, env))
            return completed

        monkeypatch.setattr(opus_land.depot_cargo, "run_gate_command", fake_gate)
        entries, error = opus_land.run_gate_leg(
            ["cargo fmt --check"], tmp_path, {"PATH": "test"}, timeout_s=17
        )
        assert error is None
        assert entries[0]["depot_id"] == "run-456"
        assert calls == [(["cargo", "fmt", "--check"], tmp_path, 17, {"PATH": "test"})]


class TestGateCandidateTreeLaw:
    """Heavy gate builds run LOCALLY CAGED on the candidate worktree.

    The depot-ci remote path runs a fixed job (elai-core-check) against the
    MAIN checkout — it never judges the candidate tree (J-102 landed green,
    broke main). An uncaged local build is killed by rustc_tripwire v7
    (rc=-15). The gate therefore runs the real command in a systemd
    run-*.scope cage: honest tree, bounded RAM, breaker-exempt.
    """

    def _land(self):
        sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
        import opus_walk.opus_land as opus_land
        return opus_land

    @pytest.fixture(autouse=True)
    def _disable_shared_target_trim(self, monkeypatch):
        opus_land = self._land()
        monkeypatch.setattr(opus_land, "_trim_gate_target_dir", lambda *args: False)

    def test_translate_nextest_to_cargo_test(self):
        opus_land = self._land()
        assert opus_land._translate_nextest(
            ["cargo", "nextest", "run", "-p", "elai-orchestrator", "j80_worktree_pool_lifecycle"]
        ) == ["cargo", "test", "-p", "elai-orchestrator", "j80_worktree_pool_lifecycle"]
        assert opus_land._translate_nextest(
            ["cargo", "nextest", "run", "--test", "reward_tamper_admit_observable_smoke"]
        ) == ["cargo", "test", "--test", "reward_tamper_admit_observable_smoke"]

    def test_translate_leaves_other_commands_alone(self):
        opus_land = self._land()
        for cmd in (["cargo", "check", "-p", "x"], ["cargo", "test"], ["python3", "x.py"],
                    ["cargo", "nextest"]):
            assert opus_land._translate_nextest(list(cmd)) == cmd

    def test_heavy_gate_runs_local_caged_not_depot(self, monkeypatch, tmp_path):
        opus_land = self._land()
        ran = []
        completed = subprocess.CompletedProcess(["x"], 0, "built", "")

        def fake_run(cmd, cwd, timeout=None, env=None):
            ran.append((cmd, env))
            return completed

        monkeypatch.setattr(opus_land, "_run", fake_run)
        monkeypatch.setattr(opus_land.shutil, "which", lambda name: "/usr/bin/systemd-run")
        depot = mock.Mock()
        monkeypatch.setattr(opus_land.depot_cargo, "run_gate_command", depot)
        safe_target = tmp_path / "cargo-target-elai"

        entries, error = opus_land.run_gate_leg(
            ["cargo check -p elai-orchestrator --all-targets"],
            tmp_path,
            {"PATH": "p", "CARGO_TARGET_DIR": str(safe_target)},
        )
        assert error is None
        depot.assert_not_called()
        argv, env = ran[0]
        assert argv[0] == "systemd-run" and "--scope" in argv
        assert any(a.startswith("MemoryMax=") for a in argv)
        assert argv[-5:] == ["cargo", "check", "-p", "elai-orchestrator", "--all-targets"]
        assert env["CARGO_TARGET_DIR"] == str(safe_target)
        assert entries[0]["provider"] == "local-caged"

    def test_candidate_gate_trim_never_receives_shared_target(self, monkeypatch, tmp_path):
        opus_land = self._land()
        ran = []
        trim_paths = []
        completed = subprocess.CompletedProcess(["x"], 0, "", "")

        monkeypatch.setattr(
            opus_land,
            "_run",
            lambda cmd, cwd, timeout=None, env=None: (ran.append((cmd, env)), completed)[1],
        )
        monkeypatch.setattr(opus_land.shutil, "which", lambda name: "/usr/bin/systemd-run")
        monkeypatch.setattr(
            opus_land,
            "_trim_gate_target_dir",
            lambda path, budget_gb=None: (trim_paths.append(Path(path)), False)[1],
        )
        safe_target = tmp_path / "cargo-target-elai"

        entries, error = opus_land.run_gate_leg(
            ["cargo check", "sh /tmp/i22_gate.sh"],
            tmp_path,
            {"PATH": "p", "CARGO_TARGET_DIR": str(safe_target)},
        )

        assert error is None
        assert len(entries) == 2
        assert trim_paths == [safe_target]
        assert Path("/home/researcher/.cache/cargo-target-elai") not in trim_paths

    def test_bare_assignment_prefix_is_caged(self, monkeypatch, tmp_path):
        opus_land = self._land()
        ran = []
        completed = subprocess.CompletedProcess(["x"], 0, "ok", "")
        monkeypatch.setattr(
            opus_land, "_run",
            lambda cmd, cwd, timeout=None, env=None: (ran.append((cmd, env)), completed)[1],
        )
        monkeypatch.setattr(opus_land.shutil, "which", lambda name: "/usr/bin/systemd-run")
        monkeypatch.setattr(opus_land, "_trim_gate_target_dir", lambda *args: False)
        depot = mock.Mock()
        monkeypatch.setattr(opus_land.depot_cargo, "run_gate_command", depot)
        command = ["cargo", "check", "-p", "elai-orchestrator", "--all-targets"]

        entries, error = opus_land.run_gate_leg(
            ["CARGO_TARGET_DIR=/custom cargo check -p elai-orchestrator --all-targets"],
            tmp_path,
            {"CARGO_TARGET_DIR": "/prior", "PATH": "p"},
        )

        assert error is None
        assert opus_land.depot_offload_is_heavy(command)
        depot.assert_not_called()
        argv, env = ran[0]
        assert argv[0] == "systemd-run" and "--scope" in argv
        assert argv[-5:] == command
        assert env["CARGO_TARGET_DIR"] == "/custom"
        assert entries[0]["provider"] == "local-caged"

    def test_env_assignment_prefix_is_caged(self, monkeypatch, tmp_path):
        opus_land = self._land()
        ran = []
        completed = subprocess.CompletedProcess(["x"], 0, "ok", "")
        monkeypatch.setattr(
            opus_land, "_run",
            lambda cmd, cwd, timeout=None, env=None: (ran.append((cmd, env)), completed)[1],
        )
        monkeypatch.setattr(opus_land.shutil, "which", lambda name: "/usr/bin/systemd-run")
        monkeypatch.setattr(opus_land, "_trim_gate_target_dir", lambda *args: False)

        entries, error = opus_land.run_gate_leg(
            ["env FOO=bar CARGO_TARGET_DIR=/env cargo check"], tmp_path, {"PATH": "p"}
        )

        assert error is None
        assert ran[0][0][-2:] == ["cargo", "check"]
        assert ran[0][1]["FOO"] == "bar"
        assert ran[0][1]["CARGO_TARGET_DIR"] == "/env"
        assert entries[0]["provider"] == "local-caged"

    def test_command_assignment_overrides_gate_environment(self, monkeypatch, tmp_path):
        opus_land = self._land()
        seen = {}
        completed = subprocess.CompletedProcess(["x"], 0, "ok", "")
        monkeypatch.setattr(
            opus_land, "_run",
            lambda cmd, cwd, timeout=None, env=None: (seen.update(env or {}), completed)[1],
        )
        monkeypatch.setattr(opus_land.shutil, "which", lambda name: "/usr/bin/systemd-run")
        monkeypatch.setattr(opus_land, "_trim_gate_target_dir", lambda *args: False)

        opus_land.run_gate_leg(
            ["CARGO_TARGET_DIR=/local cargo check"],
            tmp_path,
            {"CARGO_TARGET_DIR": "/inherited"},
        )

        assert seen["CARGO_TARGET_DIR"] == "/local"

    def test_assignment_only_gate_fails_closed_without_subprocess(self, monkeypatch, tmp_path):
        opus_land = self._land()
        subprocess_run = mock.Mock(side_effect=AssertionError("subprocess must not run"))
        monkeypatch.setattr(opus_land.subprocess, "run", subprocess_run)
        depot = mock.Mock()
        monkeypatch.setattr(opus_land.depot_cargo, "run_gate_command", depot)

        entries, error = opus_land.run_gate_leg(
            ["CARGO_TARGET_DIR=/custom"], tmp_path, {"CARGO_TARGET_DIR": "/prior"}
        )

        assert error is not None
        assert "no executable" in error
        assert entries[0]["rc"] is None
        subprocess_run.assert_not_called()
        depot.assert_not_called()

    def test_malformed_and_non_assignment_prefixes_stay_unchanged(self):
        opus_land = self._land()
        inherited = {"PATH": "p"}
        for command in (
            ["BAD-NAME=value", "cargo", "check"],
            ["env", "cargo", "check"],
            ["env", "BAD-NAME=value", "cargo", "check"],
        ):
            normalized, command_env = opus_land._normalize_gate_command(command, inherited)
            assert normalized == command
            assert command_env == inherited

    def test_shell_wrapper_gate_runs_caged(self, monkeypatch, tmp_path):
        """A `sh some-script.sh` gate leg must be CAGED: the script can run cargo inside,
        invisible to the argv heavy-detector. Live 2026-08-11: an uncaged `sh /tmp/i22_gate.sh`
        gate had its inner cargo killed by the breaker 7 times, each recorded as a gate FAIL."""
        opus_land = self._land()
        ran = []
        completed = subprocess.CompletedProcess(["x"], 0, "ok", "")
        monkeypatch.setattr(opus_land, "_run",
                            lambda cmd, cwd, timeout=None, env=None: (ran.append(cmd), completed)[1])
        monkeypatch.setattr(opus_land.shutil, "which", lambda name: "/usr/bin/systemd-run")
        depot = mock.Mock()
        monkeypatch.setattr(opus_land.depot_cargo, "run_gate_command", depot)
        safe_target = tmp_path / "cargo-target-elai"
        entries, error = opus_land.run_gate_leg(
            ["sh /tmp/i22_gate.sh"],
            tmp_path,
            {"PATH": "p", "CARGO_TARGET_DIR": str(safe_target)},
        )
        assert error is None
        depot.assert_not_called()
        argv = ran[0]
        assert argv[0] == "systemd-run" and "--scope" in argv
        assert argv[-2:] == ["sh", "/tmp/i22_gate.sh"]
        assert entries[0]["provider"] == "local-caged"

    def test_non_shell_light_command_still_uncaged(self, monkeypatch, tmp_path):
        """`python3 checker.py` stays on the plain run_gate_command path (no cage regression)."""
        opus_land = self._land()
        depot = mock.Mock(return_value=subprocess.CompletedProcess(["x"], 0, "ok", ""))
        monkeypatch.setattr(opus_land.depot_cargo, "run_gate_command", depot)
        entries, error = opus_land.run_gate_leg(["python3 check.py"], tmp_path, {})
        assert error is None
        depot.assert_called_once()

    def test_cage_wrap_without_systemd_run_is_passthrough(self, monkeypatch):
        opus_land = self._land()
        monkeypatch.setattr(opus_land.shutil, "which", lambda name: None)
        cmd = ["cargo", "check", "-p", "x"]
        assert opus_land._cage_wrap(cmd) == cmd

    def test_gate_env_respects_existing_target_dir(self, monkeypatch, tmp_path):
        opus_land = self._land()
        seen = {}
        completed = subprocess.CompletedProcess(["x"], 0, "", "")

        def fake_run(cmd, cwd, timeout=None, env=None):
            seen.update(env)
            return completed

        monkeypatch.setattr(opus_land, "_run", fake_run)
        monkeypatch.setattr(opus_land.shutil, "which", lambda name: "/usr/bin/systemd-run")
        opus_land.run_gate_leg(["cargo check"], tmp_path, {"CARGO_TARGET_DIR": "/custom"})
        assert seen["CARGO_TARGET_DIR"] == "/custom"


class TestGateTargetBudget:
    """The shared gate CARGO_TARGET_DIR is budgeted, not unbounded.

    Cargo target dirs have no GC: the shared incremental dir grew 0 -> 41G in
    ~10h of lands and was marching at the walk's own disk-guard floor. The
    trim wipes it at gate start (serialized lands = never mid-build) when it
    exceeds TEIKOKU_GATE_TARGET_BUDGET_GB (default 20).
    """

    def _land(self):
        sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
        import opus_walk.opus_land as opus_land
        return opus_land

    def _dir_with_file(self, tmp_path, name, size_b):
        d = tmp_path / name
        d.mkdir()
        (d / "blob.bin").write_bytes(b"x" * size_b)
        return d

    def test_over_budget_shared_dir_is_wiped(self, tmp_path):
        opus_land = self._land()
        d = self._dir_with_file(tmp_path, "cargo-target-elai", 4096)
        # budget of ~1KiB in GB — the 4KiB blob is over it
        assert opus_land._trim_gate_target_dir(d, budget_gb=1024 / 1024 ** 3) is True
        assert not d.exists()

    def test_under_budget_dir_is_kept(self, tmp_path):
        opus_land = self._land()
        d = self._dir_with_file(tmp_path, "cargo-target-elai", 1024)
        assert opus_land._trim_gate_target_dir(d, budget_gb=1.0) is False
        assert d.exists()

    def test_foreign_dir_name_is_never_wiped(self, tmp_path):
        """Own-artifact guard: a misconfigured env pointing anywhere else is a no-op."""
        opus_land = self._land()
        d = self._dir_with_file(tmp_path, "important-user-data", 4096)
        assert opus_land._trim_gate_target_dir(d, budget_gb=1024 / 1024 ** 3) is False
        assert d.exists()

    def test_cachedir_tagged_dir_counts_as_cargo_target(self, tmp_path):
        opus_land = self._land()
        d = self._dir_with_file(tmp_path, "custom-target", 4096)
        (d / "CACHEDIR.TAG").write_text("Signature: 8a477f597d28d172789f06886806bc55")
        assert opus_land._trim_gate_target_dir(d, budget_gb=1024 / 1024 ** 3) is True
        assert not d.exists()

    def test_missing_dir_is_noop(self, tmp_path):
        opus_land = self._land()
        assert opus_land._trim_gate_target_dir(tmp_path / "absent", budget_gb=0.0) is False

    def test_env_budget_overrides_default(self, tmp_path, monkeypatch):
        opus_land = self._land()
        d = self._dir_with_file(tmp_path, "cargo-target-elai", 4096)
        monkeypatch.setenv(opus_land.GATE_TARGET_BUDGET_GB_ENV,
                           str(1024 / 1024 ** 3))
        assert opus_land._trim_gate_target_dir(d) is True
        assert not d.exists()

    def test_gate_leg_trims_once_before_first_heavy_build(self, monkeypatch, tmp_path):
        opus_land = self._land()
        completed = subprocess.CompletedProcess(["x"], 0, "", "")
        monkeypatch.setattr(opus_land, "_run",
                            lambda cmd, cwd, timeout=None, env=None: completed)
        monkeypatch.setattr(opus_land.shutil, "which", lambda name: "/usr/bin/systemd-run")
        trims = []
        monkeypatch.setattr(opus_land, "_trim_gate_target_dir",
                            lambda p, budget_gb=None: trims.append(str(p)))
        opus_land.run_gate_leg(["cargo check -p a", "cargo test -p a"], tmp_path, {})
        assert len(trims) == 1
        assert trims[0].endswith("cargo-target-elai")


class TestKeepForLandRelease:
    """Landed plans must release their .KEEP_FOR_LAND markers.

    opus_dispatch marks trees so disk_hygiene never reaps a tree the land
    still needs; nothing cleared the marker after MERGED, so landed plans'
    trees were protected forever (13/23 marked, 2026-08-08)."""

    def _land(self):
        sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
        import opus_walk.opus_land as opus_land
        return opus_land

    def test_release_clears_only_matching_plan_markers(self, monkeypatch, tmp_path):
        opus_land = self._land()
        monkeypatch.setattr(opus_land, "WORKTREE_ROOT", tmp_path)
        mine = tmp_path / "opuswalk-K-99-EXECUTE-123-abc_def0"
        other = tmp_path / "opuswalk-K-100-EXECUTE-456-fff_1111"
        for d in (mine, other):
            d.mkdir()
            (d / ".KEEP_FOR_LAND").write_text("hold")
        opus_land._release_keep_for_land_markers("K-99")
        assert not (mine / ".KEEP_FOR_LAND").exists()
        assert (other / ".KEEP_FOR_LAND").exists()
        assert mine.exists()  # marker release never removes the tree itself

    def test_release_is_failsoft_and_skips_empty_plan(self, monkeypatch, tmp_path):
        opus_land = self._land()
        monkeypatch.setattr(opus_land, "WORKTREE_ROOT", tmp_path / "absent-root")
        opus_land._release_keep_for_land_markers("K-99")  # no raise on missing root
        opus_land._release_keep_for_land_markers("")      # no glob on empty plan id


class TestCageIoCap:
    """Owner-approved 2026-08-12: gate cage gets an IO write cap, guarded on the
    io controller actually being delegated to the user slice (fail-closed to
    no-cap: an undelegated property would be silently ignored by systemd)."""

    def test_io_cap_added_when_delegated(self, monkeypatch):
        from opus_walk import opus_land
        monkeypatch.setattr(opus_land, "_io_controller_delegated", lambda uid: True)
        cmd = opus_land._cage_wrap(["cargo", "check"])
        joined = " ".join(cmd)
        assert "IOWriteBandwidthMax=/dev/nvme0n1 200M" in joined
        assert "MemoryMax=" in joined  # existing caps kept

    def test_io_cap_skipped_when_not_delegated(self, monkeypatch):
        from opus_walk import opus_land
        monkeypatch.setattr(opus_land, "_io_controller_delegated", lambda uid: False)
        cmd = opus_land._cage_wrap(["cargo", "check"])
        assert "IOWriteBandwidthMax" not in " ".join(cmd)

    def test_io_cap_disabled_by_zero_env(self, monkeypatch):
        from opus_walk import opus_land
        monkeypatch.setattr(opus_land, "_io_controller_delegated", lambda uid: True)
        monkeypatch.setenv("TEIKOKU_GATE_IOMAX", "0")
        cmd = opus_land._cage_wrap(["cargo", "check"])
        assert "IOWriteBandwidthMax" not in " ".join(cmd)
