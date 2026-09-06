"""ENTRY 10 F-3: destructive-git coverage for guard.OutputGuard.

The OutputGuard scans WORKER + codex stdout/stderr (worker_providers.post_guard_block) and is the
STRUCTURAL git-safety layer, because codex cannot reliably honor the prose git-read-only preamble.
These tests pin, per destructive-git pattern, that a real command is BLOCKED, and pin the ALLOW
regressions that must never break: the K-27 `2>/dev/null` narrowing, prose/quoted mentions of the
new patterns, and the plain fast-forward-safe git commands workers legitimately emit.

unittest.TestCase so it runs under both `python3 -m pytest` and `python3 -m unittest`.
"""
from __future__ import annotations

import unittest

import guard


def _blocked(text: str) -> bool:
    return guard.OutputGuard().check(text).blocked


class DestructiveCommandGuardTests(unittest.TestCase):
    def test_rm_rf_command_positions_block(self):
        for command in (
            "rm -rf /",
            "$ rm -rf /",
            "    rm -rf /",
            "true && rm -rf /",
        ):
            with self.subTest(command=command):
                self.assertTrue(_blocked(command))

    def test_rm_rf_prose_and_quoted_mentions_allow(self):
        for text in (
            "No rm -rf /",
            "Never run rm -rf /",
            'quoted command: "rm -rf /"',
            "Markdown example: `rm -rf /`",
        ):
            with self.subTest(text=text):
                self.assertFalse(_blocked(text))


class DestructiveGitBlockTests(unittest.TestCase):
    """One BLOCK assertion per new destructive-git pattern (command-position emitted command)."""

    def test_push_force_blocks(self):
        self.assertTrue(_blocked("git push --force origin main"))

    def test_push_f_short_flag_blocks(self):
        self.assertTrue(_blocked("git push -f origin main"))

    def test_push_force_with_lease_blocks(self):
        self.assertTrue(_blocked("git push --force-with-lease origin main"))

    def test_push_colon_refspec_delete_blocks(self):
        # remote-branch delete via colon refspec
        self.assertTrue(_blocked("git push origin :feature-branch"))

    def test_push_delete_blocks(self):
        self.assertTrue(_blocked("git push --delete origin feature-branch"))

    def test_push_mirror_blocks(self):
        self.assertTrue(_blocked("git push --mirror origin"))

    def test_bare_push_blocks(self):
        # deliverable-2 backstop: a worker/codex is never a legitimate push origin
        self.assertTrue(_blocked("git push origin main"))

    def test_branch_force_delete_blocks(self):
        self.assertTrue(_blocked("git branch -D main"))

    def test_tag_delete_blocks(self):
        self.assertTrue(_blocked("git tag -d v1.0.0"))

    def test_reflog_expire_blocks(self):
        self.assertTrue(_blocked("git reflog expire --expire=now --all"))

    def test_filter_branch_blocks(self):
        self.assertTrue(_blocked("git filter-branch --force --tree-filter 'rm x' HEAD"))

    def test_filter_repo_blocks(self):
        self.assertTrue(_blocked("git filter-repo --path secrets --invert-paths"))

    def test_gc_prune_now_blocks(self):
        self.assertTrue(_blocked("git gc --prune=now"))

    def test_chained_command_after_separator_blocks(self):
        # command-position anchor also fires after a shell separator, so a chained evasion is caught
        self.assertTrue(_blocked("cd /tmp && git push --force"))
        self.assertTrue(_blocked("true ; git branch -D main"))

    def test_indented_and_prompt_prefixed_command_blocks(self):
        self.assertTrue(_blocked("    git push --force"))       # indented command block
        self.assertTrue(_blocked("$ git push --mirror origin"))  # shell-prompt prefixed


class GitSafetyAllowRegressionTests(unittest.TestCase):
    """ALLOW regressions: text that must NOT be blocked, or a worker turn dies for no reason."""

    def test_dev_null_redirect_allows(self):
        # K-27 (owner-approved): `2>/dev/null` must stay ALLOW; regressing it discarded EVERY diff
        self.assertFalse(_blocked("cargo test 2>/dev/null | tail -5"))
        self.assertFalse(_blocked("grep foo bar.txt 2>/dev/null"))

    def test_prose_mention_of_new_patterns_allows(self):
        # a doc/prose line MENTIONING the command is not a command -> must not false-positive
        self.assertFalse(_blocked("Never run git filter-repo on the shared history."))
        self.assertFalse(_blocked("The runbook says to avoid git push --mirror at all costs."))
        self.assertFalse(_blocked("Do not use `git branch -D main` on protected branches."))
        self.assertFalse(_blocked("Contributors sometimes reach for git tag -d to fix a typo."))

    def test_markdown_list_and_diff_hunk_mention_allows(self):
        self.assertFalse(_blocked("  - git push --delete origin x   # (documented, do not do this)"))
        self.assertFalse(_blocked("+git push --mirror origin"))   # diff added line, + at column 0
        self.assertFalse(_blocked("-git reflog expire --all"))    # diff removed line

    def test_fast_forward_safe_git_commands_allow(self):
        # a bare `git push` emitted as a command IS blocked by design (worker denied + opus_land not
        # scanned), so it is intentionally absent here; these read-only/local commands must stay ALLOW.
        for cmd in (
            "git status",
            "git diff HEAD~1",
            "git log --oneline -10",
            "git add .",
            'git commit -m "wire the production hook"',
            "git fetch origin",
        ):
            with self.subTest(cmd=cmd):
                self.assertFalse(_blocked(cmd))

    def test_benign_prose_allows(self):
        self.assertFalse(_blocked("added a test for the bridge env"))
        self.assertFalse(_blocked("refactored the scheduler and updated the changelog"))


class SystemRedirectGuardTests(unittest.TestCase):
    """K-27 allowlist + marathon 2026-07-16 fail-open fix for bare / :>/ redirects."""

    def test_k27_dev_null_and_tmp_still_allow(self):
        # The original landmine: stderr redirects must never discard a codex diff.
        self.assertFalse(_blocked("cargo test 2>/dev/null | tail -5"))
        self.assertFalse(_blocked("grep foo bar.txt 2>/dev/null"))
        self.assertFalse(_blocked("cmd 2>/tmp/out.log"))
        self.assertFalse(_blocked("cmd >/tmp/out.log"))
        self.assertFalse(_blocked("cmd >/home/researcher/out"))
        self.assertFalse(_blocked("cmd >/mnt/f/work/out"))
        self.assertFalse(_blocked("ls >/dev/null"))

    def test_system_dir_redirect_blocks_bare_and_fd(self):
        # FAIL-OPEN fixed: bare >/etc and classic :>/etc must block (not only 2>/etc).
        self.assertTrue(_blocked(">/etc/passwd"))
        self.assertTrue(_blocked(":>/etc/passwd"))
        self.assertTrue(_blocked(":> /etc/hosts"))
        self.assertTrue(_blocked("echo secrets >/etc/shadow"))
        self.assertTrue(_blocked("2>/etc/cron.d/evil"))
        self.assertTrue(_blocked("1>/var/log/auth.log"))
        self.assertTrue(_blocked(">>/usr/local/bin/backdoor"))
        self.assertTrue(_blocked("&>/root/.ssh/authorized_keys"))
        # no-space form (cmd>/path) is legal shell and must still block system dirs
        self.assertTrue(_blocked("echo>/etc/passwd"))
        self.assertTrue(_blocked("cmd>/var/log/x"))

    def test_raw_disk_redirect_blocks(self):
        self.assertTrue(_blocked(">/dev/sda"))
        self.assertTrue(_blocked("2>/dev/hda"))
        self.assertTrue(_blocked(":>/dev/sda1"))
        # /dev/null remains ALLOW (not a disk device; also out of system-dir list)
        self.assertFalse(_blocked(">/dev/null"))

    def test_legacy_git_and_reboot_prose_allows(self):
        # Marathon: unanchored pre-F-3 rules false-positived on mentions.
        self.assertFalse(_blocked("please reboot later after the window"))
        self.assertFalse(_blocked("Never git push --force on shared branches."))
        self.assertFalse(_blocked("document: git reset --hard is bad"))
        self.assertFalse(_blocked("use git clean -fd carefully in docs only"))
        self.assertFalse(_blocked("admin ran kill -9 once in an incident report"))
        self.assertFalse(_blocked("wiki: git checkout -- . is destructive"))
        self.assertFalse(_blocked("hint: git restore --staged ."))
        self.assertFalse(_blocked("docs say git revert --no-edit"))

    def test_legacy_git_and_reboot_commands_still_block(self):
        self.assertTrue(_blocked("git push --force origin main"))
        self.assertTrue(_blocked("git reset --hard HEAD~1"))
        self.assertTrue(_blocked("git clean -fd"))
        self.assertTrue(_blocked("reboot"))
        self.assertTrue(_blocked("kill -9 1234"))
        self.assertTrue(_blocked("pkill -9 chrome"))
        self.assertTrue(_blocked("shutdown -h now"))


if __name__ == "__main__":
    unittest.main()
