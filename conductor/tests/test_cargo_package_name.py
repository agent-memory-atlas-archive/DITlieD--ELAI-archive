"""Regression test: _cargo_package_name returns the [package] name from Cargo.toml,
not the directory name (fixes the cargo test -p <X> ERROR that used the dir name)."""
from __future__ import annotations

import unittest
from pathlib import Path

from gate_runner import _cargo_package_name
from tests.helpers import TempDirMixin


class CargoPkgNameTests(TempDirMixin, unittest.TestCase):
    # ------------------------------------------------------------------ helpers
    def _make_crate(self, base: Path, dir_name: str, pkg_name: str) -> Path:
        crate = base / dir_name
        crate.mkdir(parents=True, exist_ok=True)
        (crate / "Cargo.toml").write_text(
            f'[package]\nname = "{pkg_name}"\nversion = "0.1.0"\nedition = "2021"\n',
            encoding="utf-8",
        )
        return crate

    # ------------------------------------------------------------------ tests
    def test_returns_package_name_not_dir_name(self) -> None:
        base = self.make_tempdir()
        crate = self._make_crate(base, dir_name="orchestrator", pkg_name="elai-orchestrator")
        result = _cargo_package_name(crate / "Cargo.toml")
        self.assertEqual(result, "elai-orchestrator")
        self.assertNotEqual(result, "orchestrator")

    def test_returns_package_name_when_equals_dir_name(self) -> None:
        base = self.make_tempdir()
        crate = self._make_crate(base, dir_name="mylib", pkg_name="mylib")
        result = _cargo_package_name(crate / "Cargo.toml")
        self.assertEqual(result, "mylib")

    def test_fallback_no_cargo_toml_returns_none(self) -> None:
        base = self.make_tempdir()
        missing = base / "no-manifest" / "Cargo.toml"
        result = _cargo_package_name(missing)
        self.assertIsNone(result)

    def test_cargo_toml_missing_package_section_returns_none(self) -> None:
        base = self.make_tempdir()
        crate = base / "workspace-root"
        crate.mkdir()
        (crate / "Cargo.toml").write_text(
            "[workspace]\nmembers = [\"crates/*\"]\n", encoding="utf-8"
        )
        result = _cargo_package_name(crate / "Cargo.toml")
        self.assertIsNone(result)

    def test_cargo_toml_name_with_single_quotes(self) -> None:
        base = self.make_tempdir()
        crate = base / "mypkg"
        crate.mkdir()
        (crate / "Cargo.toml").write_text(
            "[package]\nname = 'my-package'\nversion = \"0.1.0\"\n", encoding="utf-8"
        )
        result = _cargo_package_name(crate / "Cargo.toml")
        self.assertEqual(result, "my-package")


if __name__ == "__main__":
    unittest.main()
