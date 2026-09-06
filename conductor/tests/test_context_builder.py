from __future__ import annotations

import unittest
from dataclasses import replace

from context_builder import build_context_packet_for_task
from tests.helpers import TempDirMixin
from tests.test_worker_adapter import scoped_task


class ContextBuilderTests(TempDirMixin, unittest.TestCase):
    def test_context_packet_is_bounded_deterministic_and_redacted(self) -> None:
        temp = self.make_tempdir()
        repo = temp / "repo"
        (repo / "src").mkdir(parents=True)
        (repo / "src" / "lib.rs").write_text(
            "pub fn original() {}\napi_key=SHOULD_NOT_LEAK\n" + ("line\n" * 1000),
            encoding="utf-8",
        )
        task = scoped_task("CTX-1")
        first = build_context_packet_for_task(task, repo, max_total_chars=1600, max_file_chars=500)
        second = build_context_packet_for_task(task, repo, max_total_chars=1600, max_file_chars=500)
        self.assertEqual(first.digest, second.digest)
        self.assertLessEqual(len(first.text), 1700)
        self.assertTrue(first.truncated)
        self.assertNotIn("SHOULD_NOT_LEAK", first.text)
        self.assertIn("api_key=<REDACTED>", first.text)

    def test_context_packet_task_section_carries_constraint_block(self) -> None:
        # M9 (teikoku-merge-addon-M8-M9-2026-07-05.md): the imperative CONSTRAINT BLOCK
        # must survive in the Task section (context_builder._task_section), the FIRST
        # section in _cap_sections' ordered list -- a generous budget here proves the
        # content lands, independent of the separate truncation test above.
        temp = self.make_tempdir()
        repo = temp / "repo"
        repo.mkdir(parents=True)
        task = scoped_task("CTX-2")
        packet = build_context_packet_for_task(task, repo, max_total_chars=4000, max_file_chars=500)
        self.assertIn("CONSTRAINT BLOCK", packet.text)
        self.assertIn("ALLOWED FILES:", packet.text)
        self.assertIn("src/lib.rs::changed", packet.text)
        self.assertIn("production_caller_check and python_wire_check", packet.text)

    def test_context_packet_constraint_block_exempt_from_redaction(self) -> None:
        # M89 BLOCK-2: redaction.py's LONG_TOKEN_RE matches any run of 40+ path-safe chars,
        # so a real allowed_write_set path (paths are not secrets) inside the M9 CONSTRAINT
        # BLOCK was being replaced with <REDACTED_TOKEN>. The block is now rendered and
        # appended to packet_text AFTER redact_text() runs (context_builder._constraint_block_
        # text), so the literal path survives while an actual long secret-shaped token
        # elsewhere in the packet still redacts.
        temp = self.make_tempdir()
        repo = temp / "repo"
        (repo / "src").mkdir(parents=True)
        long_path = "crates/dashboard-v2/src-tauri/src/ipc_handlers.rs"
        self.assertGreaterEqual(len(long_path), 40)
        long_secret = "token_holder_" + ("z" * 40)
        (repo / "src" / "lib.rs").write_text(
            f"pub fn original() {{}}\n{long_secret}\n", encoding="utf-8"
        )
        task = replace(scoped_task("CTX-3"), allowed_write_set=[long_path, "src/lib.rs"])
        packet = build_context_packet_for_task(task, repo, max_total_chars=8000, max_file_chars=500)
        constraint_block = packet.text[packet.text.index("CONSTRAINT BLOCK"):]
        self.assertIn(long_path, constraint_block)
        self.assertNotIn("<REDACTED_TOKEN>", constraint_block)
        self.assertNotIn(long_secret, packet.text)
        self.assertIn("<REDACTED_TOKEN>", packet.text)


if __name__ == "__main__":
    unittest.main()
