from __future__ import annotations

import os
import sys
from pathlib import Path

CONDUCTOR = Path(__file__).resolve().parents[1]
if str(CONDUCTOR) not in sys.path:
    sys.path.insert(0, str(CONDUCTOR))

import opus_walk.opus_dispatch as dispatch


class TestDispatchShim:
    def test_remote_prepends_shim_and_removes_target(self, monkeypatch):
        monkeypatch.setenv("TEIKOKU_DEPOT_OFFLOAD", "remote")
        monkeypatch.setenv("PATH", "/usr/bin")
        monkeypatch.setenv("CARGO_TARGET_DIR", "/tmp/shared-target")
        dispatch._configure_worker_cargo_environment()
        assert os.environ["PATH"].split(os.pathsep)[0] == str(dispatch.DEPOT_SHIM_DIR)
        assert "CARGO_TARGET_DIR" not in os.environ
        assert (dispatch.DEPOT_SHIM_DIR / "cargo").is_file()

    def test_local_keeps_cargo_path_and_target(self, monkeypatch):
        monkeypatch.setenv("TEIKOKU_DEPOT_OFFLOAD", "local")
        monkeypatch.setenv("PATH", "/usr/bin")
        monkeypatch.delenv("CARGO_TARGET_DIR", raising=False)
        dispatch._configure_worker_cargo_environment()
        assert os.environ["PATH"].split(os.pathsep)[0] == str(Path.home() / ".cargo" / "bin")
        assert os.environ["CARGO_TARGET_DIR"] == str(Path.home() / ".cache" / "cargo-target-elai")
