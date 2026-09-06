from __future__ import annotations

import subprocess
import unittest
from pathlib import Path
from unittest.mock import patch

from worker_adapter import SubprocessWorkerProvider


class TimeoutCleanupProcess:
    pid = 12345
    returncode = None

    def __init__(self) -> None:
        self.communicate_calls = 0
        self.cancelled = False

    def communicate(self, *args, **kwargs):
        self.communicate_calls += 1
        if self.communicate_calls == 1:
            raise subprocess.TimeoutExpired(
                cmd=["worker"],
                timeout=1,
                output=b"primary stdout",
                stderr=b"primary stderr",
            )
        raise subprocess.TimeoutExpired(
            cmd=["worker"],
            timeout=15,
            output=b"cleanup stdout",
            stderr=b"cleanup stderr",
        )

    def poll(self):
        return None

    def send_signal(self, _signal):
        self.cancelled = True

    def wait(self, timeout):
        self.returncode = -15
        return self.returncode

    def terminate(self):
        self.cancelled = True

    def kill(self):
        self.returncode = -9


class WorkerTimeoutCleanupTests(unittest.TestCase):
    def test_second_timeout_returns_partial_output(self) -> None:
        process = TimeoutCleanupProcess()
        provider = SubprocessWorkerProvider()

        with patch("worker_adapter.subprocess.Popen", return_value=process), \
                patch("worker_adapter.os.killpg", side_effect=ProcessLookupError), \
                patch.object(provider, "cancel", wraps=provider.cancel) as cancel:
            result = provider.run(
                ["worker"],
                Path("."),
                timeout_seconds=1,
                env={},
            )

        cancel.assert_called_once_with()
        self.assertEqual(result.exit_code, -1)
        self.assertTrue(result.timed_out)
        self.assertEqual(result.stdout, "primary stdoutcleanup stdout")
        self.assertEqual(result.stderr, "primary stderrcleanup stderr")
        self.assertTrue(process.cancelled)
        self.assertEqual(process.communicate_calls, 2)


if __name__ == "__main__":
    unittest.main()
