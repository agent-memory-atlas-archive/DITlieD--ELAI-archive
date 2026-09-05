#!/usr/bin/env python3
"""Regression tests for the public snapshot exporter."""

import hashlib
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


EXPORTER = Path(__file__).with_name("export_public_snapshot.py")


class ExportPublicSnapshotTests(unittest.TestCase):
    def setUp(self):
        self.tempdir = tempfile.TemporaryDirectory(prefix="elai-export-test-", dir="/tmp")
        self.root = Path(self.tempdir.name)
        self.addCleanup(self.tempdir.cleanup)

    def _run(self, args, cwd):
        env = os.environ.copy()
        env.update(GIT_CONFIG_NOSYSTEM="1", GIT_TERMINAL_PROMPT="0")
        return subprocess.run(
            args,
            cwd=cwd,
            env=env,
            text=True,
            capture_output=True,
        )

    def _git(self, repo, *args):
        result = self._run(["git", *args], repo)
        self.assertEqual(result.returncode, 0, result.stderr)
        return result

    def _repo(self, name):
        repo = self.root / name
        repo.mkdir()
        self._git(repo, "init", "--initial-branch=main")
        self._git(repo, "config", "user.name", "Archive Test")
        self._git(repo, "config", "user.email", "archive-test@example.invalid")
        return repo

    def _export(self, repo, destination):
        return self._run([sys.executable, str(EXPORTER), str(destination)], repo)

    def _write(self, path, content):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)

    def test_index_only_export_and_manifest_collision(self):
        repo = self._repo("index-only")
        self._write(repo / "staged.txt", "staged content\n")
        self._write(repo / "dirty.txt", "staged version\n")
        self._write(repo / "ARCHIVE-MANIFEST.sha256", "source manifest sentinel\n")
        self._git(repo, "add", "-A")
        self._write(repo / "dirty.txt", "unstaged dirty content\n")
        self._write(repo / "untracked.txt", "untracked content\n")
        self._git(repo, "remote", "add", "origin", "https://example.invalid/archive.git")

        destination = self.root / "exported"
        result = self._export(repo, destination)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((destination / "dirty.txt").read_text(), "staged version\n")
        self.assertFalse((destination / "untracked.txt").exists())
        self.assertFalse((destination / ".git").exists())

        manifest = destination / "ARCHIVE-MANIFEST.sha256"
        rows = {}
        for line in manifest.read_text().splitlines():
            digest, relative = line.split("  ", 1)
            rows[relative] = digest
        self.assertNotIn("ARCHIVE-MANIFEST.sha256", rows)

        exported = {
            path.relative_to(destination).as_posix()
            for path in destination.rglob("*")
            if path.is_file() and path != manifest
        }
        self.assertEqual(set(rows), exported)
        for relative, digest in rows.items():
            actual = hashlib.sha256((destination / relative).read_bytes()).hexdigest()
            self.assertEqual(digest, actual, relative)

    def test_destination_symlink_parent_is_rejected(self):
        repo = self._repo("destination-boundary")
        self._write(repo / "safe.txt", "safe\n")
        self._git(repo, "add", "safe.txt")

        alias = self.root / "repo-alias"
        alias.symlink_to(repo, target_is_directory=True)
        destination = alias / "export"

        result = self._export(repo, destination)

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("outside the development repository", result.stderr)
        self.assertFalse((repo / "export").exists())

    def test_symlink_archive_member_is_rejected(self):
        repo = self._repo("archive-symlink")
        self._write(repo / "safe.txt", "safe\n")
        link = repo / "member-link"
        try:
            link.symlink_to("safe.txt")
        except OSError as error:
            self.skipTest(f"symlinks unavailable: {error}")
        self._git(repo, "add", "-A")

        destination = self.root / "symlink-export"
        result = self._export(repo, destination)

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("unsupported archive member", result.stderr)
        self.assertFalse(destination.exists())


if __name__ == "__main__":
    unittest.main()
