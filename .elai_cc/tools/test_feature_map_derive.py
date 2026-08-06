import hashlib
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).parent
spec = importlib.util.spec_from_file_location("feature_map_derive", HERE / "feature_map_derive.py")
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)


class _FeatureMapDeriveTests(unittest.TestCase):
    def fixture(self, fmap):
        td = tempfile.TemporaryDirectory()
        root = Path(td.name)
        (root / ".elai_cc/crates/orchestrator/src").mkdir(parents=True)
        (root / ".elai_cc/crates/orchestrator/tests").mkdir(parents=True)
        (root / ".elai_cc/crates/orchestrator/src/ssb.rs").write_text(
            "const SCHEMA_VERSION_DDL_EPOCH: i64 = -10;\n"
            "pub const SSB_ACTION_TYPES: &[SsbActionType] = &[\n"
            " SsbActionType::One,\n SsbActionType::Two,\n];\n")
        (root / ".elai_cc/crates/orchestrator/tests/action_type_lock.rs").write_text(
            "const TOTAL_VARIANTS: usize = ALL_VARIANTS.len();\n")
        path = root / "map.md"
        path.write_text(fmap)
        return td, root, path

    def test_schema_and_stale_stamp(self):
        td, root, path = self.fixture("SCHEMA_VERSION 1\n## SECTION 1\n[WIRED] Thing\n")
        self.addCleanup(td.cleanup)
        report = mod.derive_report(root, path)
        self.assertEqual(report["schema"]["schema_version"], -8)
        self.assertEqual(report["schema"]["total_variants"], 2)
        self.assertTrue(any(x["flag"] == "schema_stamp_stale" for x in report["disagreements"]))

    def test_status_counts(self):
        td, root, path = self.fixture("## SECTION 1\n[WIRED] A\n[DARK(foo)] B\n[PARTIAL] C\n[PLANNED(X-1)] D\n[DEFERRED] E\n[BENCH-PENDING] F\n")
        self.addCleanup(td.cleanup)
        counts = mod.derive_report(root, path)["status_counts"]
        self.assertEqual({k: counts[k] for k in ("WIRED", "DARK", "PARTIAL", "PLANNED", "DEFERRED", "BENCH_PENDING")}, {"WIRED": 1, "DARK": 1, "PARTIAL": 1, "PLANNED": 1, "DEFERRED": 1, "BENCH_PENDING": 1})

    def test_edges(self):
        section = "A EXTENDS B\nC composes with D\nE GATED-on F\n"
        edges = mod._edges(section)
        self.assertEqual([(e["consumer"], e["dependency"], e["kind"]) for e in edges], [("A", "B", "EXTENDS"), ("C", "D", "COMPOSES"), ("E", "F", "GATED-on")])

    def test_missing_map_exit_two(self):
        self.assertEqual(mod.main(["--repo-root", tempfile.gettempdir(), "--feature-map", "/no/such/map"]), 2)

    def test_does_not_mutate_map(self):
        td, root, path = self.fixture("## SECTION 1\n[WIRED] A\n")
        self.addCleanup(td.cleanup)
        before = (hashlib.sha256(path.read_bytes()).hexdigest(), path.stat().st_mtime_ns)
        mod.derive_report(root, path)
        self.assertEqual(before, (hashlib.sha256(path.read_bytes()).hexdigest(), path.stat().st_mtime_ns))


if __name__ == "__main__":
    unittest.main()
