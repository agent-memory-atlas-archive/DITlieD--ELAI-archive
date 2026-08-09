#!/usr/bin/env python3
"""Contract tests for solid_proxy_check.py; stdlib unittest only."""
import json, subprocess, sys, tempfile, unittest
from pathlib import Path

TOOL = Path(__file__).with_name("solid_proxy_check.py")


def _run(argv):
    return subprocess.run([sys.executable, str(TOOL), *argv], capture_output=True, text=True)


def _map_text(rows):
    return "\n".join("\t".join(r) for r in rows) + "\n"


class _SolidProxyCheckContract(unittest.TestCase):
    def setUp(self):
        self._dir = tempfile.TemporaryDirectory()
        self.dir = Path(self._dir.name)

    def tearDown(self):
        self._dir.cleanup()

    def _map(self, rows, name="elai.map"):
        path = self.dir / name
        path.write_text(_map_text(rows), encoding="utf-8")
        return path

    def _src(self, rel, text):
        path = self.dir / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        return path

    def test_01_srp_god_object_trips_block(self):
        rows = [("DEF_STRUCT", "God", "src/god.rs:1", "", "", "struct God")]
        rows += [("TYPE_REF", f"God::m{i}", "src/caller.rs:1", "", "", f"God::m{i}") for i in range(11)]
        rows += [("DEF_METHOD", f"God::m{i}", "src/god.rs:10", "", "", f"fn m{i}") for i in range(11)]
        mp = self._map(rows)
        self._src("src/god.rs", "struct God;\n")
        r = _run(["--map", str(mp), "--files", "src/god.rs", "--root", str(self.dir)])
        self.assertEqual(r.returncode, 1, (r.returncode, r.stdout, r.stderr))
        self.assertIn("C-MAINTAINABILITY-SRP", r.stdout)
        self.assertIn("BLOCK", r.stdout)

    def test_02_srp_just_under_every_threshold_no_findings(self):
        rows = [("DEF_STRUCT", "Small", "src/small.rs:1", "", "", "struct Small")]
        rows += [("TYPE_REF", f"Small::m{i}", "src/caller.rs:1", "", "", f"Small::m{i}") for i in range(10)]
        rows += [("DEF_METHOD", f"Small::m{i}", "src/small.rs:10", "", "", f"fn m{i}") for i in range(10)]
        mp = self._map(rows)
        self._src("src/small.rs", "".join(f"l{i}\n" for i in range(200)))
        r = _run(["--map", str(mp), "--files", "src/small.rs", "--root", str(self.dir)])
        self.assertEqual(r.returncode, 0, (r.returncode, r.stdout, r.stderr))
        self.assertNotIn("C-MAINTAINABILITY-SRP", r.stdout)

    def test_03_dip_fixture_warn_exit_zero(self):
        rows = [
            ("DEF_TRAIT", "Trait", "src/trait.rs:1", "", "", "trait Trait"),
            ("IMPL_TRAIT", "Impl:Trait", "src/impl.rs:1", "", "", "impl Trait for Concrete"),
            ("TYPE_REF", "Concrete", "src/caller.rs:1", "", "", "Concrete"),
        ]
        mp = self._map(rows)
        r = _run(["--map", str(mp), "--files", "src/caller.rs", "--root", str(self.dir)])
        self.assertEqual(r.returncode, 0, (r.returncode, r.stdout, r.stderr))
        self.assertIn("C-MAINTAINABILITY-DIP", r.stdout)
        self.assertIn("WARN", r.stdout)

    def test_04_dip_dedup_one_finding_per_symbol_per_file(self):
        rows = [
            ("DEF_TRAIT", "Trait", "src/trait.rs:1", "", "", "trait Trait"),
            ("IMPL_TRAIT", "Impl:Trait", "src/impl.rs:1", "", "", "impl Trait for Concrete"),
            ("TYPE_REF", "Concrete", "src/caller.rs:1", "", "", "Concrete"),
            ("TYPE_REF", "Concrete", "src/caller.rs:5", "", "", "Concrete"),
        ]
        mp = self._map(rows)
        r = _run(["--map", str(mp), "--files", "src/caller.rs", "--json", "--root", str(self.dir)])
        self.assertEqual(r.returncode, 0, (r.returncode, r.stdout, r.stderr))
        payload = json.loads(r.stdout)
        dip = [f for f in payload["findings"] if f["tag"] == "C-MAINTAINABILITY-DIP"]
        self.assertEqual(len(dip), 1, dip)

    def test_05_isp_fixture_inside_changeset_warn(self):
        rows = [("DEF_TRAIT", "BigTrait", "src/big.rs:1", "", "", "trait BigTrait")]
        rows += [("DEF_METHOD", f"BigTrait::m{i}", "src/big.rs:10", "", "", f"fn m{i}") for i in range(11)]
        rows += [("IMPL_TRAIT", "Impl:BigTrait", "src/impl.rs:1", "", "", "impl BigTrait for Impl")]
        rows += [("DEF_METHOD", f"Impl::u{i}", "src/impl.rs:10", "", "", f"fn u{i}") for i in range(3)]
        mp = self._map(rows)
        r = _run(["--map", str(mp), "--files", "src/big.rs", "--root", str(self.dir)])
        self.assertEqual(r.returncode, 0, (r.returncode, r.stdout, r.stderr))
        self.assertIn("C-MAINTAINABILITY-ISP", r.stdout)
        self.assertIn("WARN", r.stdout)

    def test_06_isp_scoping_outside_changeset_no_findings(self):
        rows = [("DEF_TRAIT", "BigTrait", "src/big.rs:1", "", "", "trait BigTrait")]
        rows += [("DEF_METHOD", f"BigTrait::m{i}", "src/big.rs:10", "", "", f"fn m{i}") for i in range(11)]
        rows += [("IMPL_TRAIT", "Impl:BigTrait", "src/impl.rs:1", "", "", "impl BigTrait for Impl")]
        rows += [("DEF_METHOD", f"Impl::u{i}", "src/impl.rs:10", "", "", f"fn u{i}") for i in range(3)]
        mp = self._map(rows)
        r = _run(["--map", str(mp), "--files", "src/other.rs", "--json", "--root", str(self.dir)])
        self.assertEqual(r.returncode, 0, (r.returncode, r.stdout, r.stderr))
        payload = json.loads(r.stdout)
        isp = [f for f in payload["findings"] if f["tag"] == "C-MAINTAINABILITY-ISP"]
        self.assertEqual(isp, [], isp)

    def test_07_missing_or_unparseable_map_exit_two_names_path(self):
        missing = self.dir / "nope.map"
        r = _run(["--map", str(missing), "--files", "src/x.rs"])
        self.assertEqual(r.returncode, 2, (r.returncode, r.stdout, r.stderr))
        self.assertIn(str(missing), r.stderr)
        bad = self._map([("badrow",)], name="bad.map")
        r2 = _run(["--map", str(bad), "--files", "src/x.rs"])
        self.assertEqual(r2.returncode, 2, (r2.returncode, r2.stdout, r2.stderr))
        self.assertIn(str(bad), r2.stderr)

    def test_08_json_output_uncovered_names_ocp_and_lsp(self):
        rows = [("DEF_STRUCT", "God", "src/god.rs:1", "", "", "struct God")]
        mp = self._map(rows)
        r = _run(["--map", str(mp), "--files", "src/god.rs", "--json", "--root", str(self.dir)])
        self.assertEqual(r.returncode, 0, (r.returncode, r.stdout, r.stderr))
        payload = json.loads(r.stdout)
        self.assertIn("OCP", payload["uncovered"])
        self.assertIn("LSP", payload["uncovered"])

    def test_09_calibrate_no_files_exit_zero_has_distribution_keys(self):
        rows = [("DEF_STRUCT", "God", "src/god.rs:1", "", "", "struct God")]
        mp = self._map(rows)
        r = _run(["--map", str(mp), "--calibrate", "--json"])
        self.assertEqual(r.returncode, 0, (r.returncode, r.stdout, r.stderr))
        payload = json.loads(r.stdout)
        cal = payload["calibration"]
        self.assertIn("fan_out", cal)
        self.assertIn("method_count", cal)
        self.assertIn("trait_method_count", cal)

    def test_10_root_resolution_correct_vs_wrong(self):
        rows = [("DEF_STRUCT", "God", "src/god.rs:1", "", "", "struct God")]
        rows += [("TYPE_REF", f"God::m{i}", "src/caller.rs:1", "", "", f"God::m{i}") for i in range(11)]
        mp = self._map(rows)
        self._src("src/god.rs", "".join(f"line {i}\n" for i in range(201)))
        r_ok = _run(["--map", str(mp), "--files", "src/god.rs", "--json", "--root", str(self.dir)])
        payload_ok = json.loads(r_ok.stdout)
        self.assertEqual(payload_ok["unreadable_files"], [], payload_ok["unreadable_files"])
        srp_ok = [f for f in payload_ok["findings"] if f["tag"] == "C-MAINTAINABILITY-SRP"]
        self.assertEqual(len(srp_ok), 1, srp_ok)
        self.assertIn("lines=201", srp_ok[0]["message"])
        r_bad = _run(["--map", str(mp), "--files", "src/god.rs", "--json", "--root", "/nonexistent"])
        payload_bad = json.loads(r_bad.stdout)
        self.assertIn("src/god.rs", payload_bad["unreadable_files"])


if __name__ == "__main__":
    unittest.main()
