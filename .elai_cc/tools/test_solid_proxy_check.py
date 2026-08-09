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
        for row in rows:
            if len(row) >= 3 and ":" in row[2]:
                self._src(row[2].rsplit(":", 1)[0], "source\n")
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
        self.assertEqual(payload_bad["unreadable_files"], [])

    def test_11_map_base_matches_workflow_and_map_relative_names(self):
        rows = [("DEF_STRUCT", "God", "crates/x/big.rs:1", "", "", "struct God")]
        rows += [("TYPE_REF", f"God::m{i}", "src/caller.rs:1", "", "", f"God::m{i}")
                 for i in range(11)]
        rows += [("DEF_METHOD", f"God::m{i}", "crates/x/big.rs:10", "", "", f"fn m{i}")
                 for i in range(11)]
        map_path = self.dir / ".claude" / "tools" / "elai.map"
        map_path.parent.mkdir(parents=True)
        map_path.write_text(_map_text(rows), encoding="utf-8")
        source = self.dir / ".elai_cc" / "crates" / "x" / "big.rs"
        source.parent.mkdir(parents=True)
        source.write_text("\n".join(f"line {i}" for i in range(201)) + "\n", encoding="utf-8")

        common = ["--map", str(map_path), "--json", "--root", str(self.dir)]
        workflow = _run([*common, "--files", ".elai_cc/crates/x/big.rs"])
        map_relative = _run([*common, "--files", "crates/x/big.rs"])
        self.assertEqual(workflow.returncode, 1, (workflow.returncode, workflow.stdout, workflow.stderr))
        self.assertEqual(map_relative.returncode, 1, (map_relative.returncode, map_relative.stdout, map_relative.stderr))
        workflow_payload = json.loads(workflow.stdout)
        map_relative_payload = json.loads(map_relative.stdout)
        self.assertTrue(workflow_payload["findings"])
        self.assertEqual(workflow_payload["findings"], map_relative_payload["findings"])

    def _equivalence_fixture(self):
        rows = [("DEF_STRUCT", "God", "crates/x/big.rs:1", "", "", "struct God")]
        rows += [("TYPE_REF", f"God::m{i}", "src/caller.rs:1", "", "", f"God::m{i}") for i in range(11)]
        rows += [("DEF_METHOD", f"God::m{i}", "crates/x/big.rs:10", "", "", f"fn m{i}") for i in range(11)]
        mp = self._map(rows)
        self._src("crates/x/big.rs", "\n".join(f"line {i}" for i in range(201)) + "\n")
        return mp

    def test_12_flat_layout_finding(self):
        mp = self._equivalence_fixture()
        r = _run(["--map", str(mp), "--files", "crates/x/big.rs", "--json", "--root", str(self.dir)])
        self.assertEqual(r.returncode, 1, (r.returncode, r.stdout, r.stderr))
        self.assertTrue(json.loads(r.stdout)["findings"])

    def test_13_unresolvable_file_is_residue(self):
        mp = self._map([("DEF_STRUCT", "Known", "src/known.rs:1", "", "", "struct Known")])
        r = _run(["--map", str(mp), "--files", "no/such/file.rs", "--json", "--root", str(self.dir)])
        self.assertEqual(r.returncode, 0, (r.returncode, r.stdout, r.stderr))
        payload = json.loads(r.stdout)
        self.assertIn("no/such/file.rs", payload["residue"])
        self.assertEqual(payload["findings"], [])

    def test_14_wrong_base_is_scan_config_error(self):
        mp = self._map([("DEF_STRUCT", "Known", "src/known.rs:1", "", "", "struct Known")])
        (self.dir / "src/known.rs").unlink()
        r = _run(["--map", str(mp), "--map-base", str(self.dir / "wrong"), "--files", "src/known.rs", "--json"])
        self.assertEqual(r.returncode, 2, (r.returncode, r.stdout, r.stderr))
        self.assertIn("map base", r.stderr)
        self.assertIn("wrong", r.stderr)

    def test_15_existing_file_with_zero_map_rows_is_error(self):
        mp = self._map([("DEF_STRUCT", "Known", "src/known.rs:1", "", "", "struct Known")])
        self._src("src/other.rs", "source\n")
        r = _run(["--map", str(mp), "--files", "src/other.rs", "--root", str(self.dir)])
        self.assertEqual(r.returncode, 2, (r.returncode, r.stdout, r.stderr))
        self.assertIn("src/other.rs", r.stderr)
        self.assertIn("zero map rows", r.stderr)

    def test_16_explicit_map_base_controls_resolution(self):
        mp = self._map([("DEF_STRUCT", "Known", "src/known.rs:1", "", "", "struct Known")])
        good = _run(["--map", str(mp), "--map-base", str(self.dir), "--files", "src/known.rs", "--json"])
        self.assertEqual(good.returncode, 0, (good.returncode, good.stdout, good.stderr))
        bad = _run(["--map", str(mp), "--map-base", str(self.dir / "wrong"), "--files", "src/known.rs"])
        self.assertEqual(bad.returncode, 2, (bad.returncode, bad.stdout, bad.stderr))

    def test_17_map_relative_flat_finding(self):
        mp = self._map([("DEF_STRUCT", "Known", "src/known.rs:1", "", "", "struct Known")])
        r = _run(["--map", str(mp), "--files", "src/known.rs", "--json", "--root", str(self.dir)])
        self.assertEqual(r.returncode, 0, (r.returncode, r.stdout, r.stderr))
        self.assertEqual(json.loads(r.stdout)["findings"], [])


if __name__ == "__main__":
    unittest.main()
