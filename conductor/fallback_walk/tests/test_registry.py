import subprocess
import unittest

import walk_controller_registry as registry


class RegistryTests(unittest.TestCase):
    def test_all_shared_controllers_are_registered(self):
        self.assertEqual(
            set(registry.CONTROLLER_PATTERNS),
            {"opus-orchestrator", "glm-orchestrator", "fallback-orchestrator", "elai-walk"},
        )

    def test_unknown_controller_is_rejected(self):
        with self.assertRaises(ValueError):
            registry.controller_pids("unknown")

    def test_shell_command_text_is_not_a_controller(self):
        process = subprocess.Popen(
            [
                "bash",
                "-c",
                "while :; do sleep 1; done # fallback_walk/fallback_supervisor.py "
                "glm_walk/glm_supervisor.py opus_walk/opus_supervisor.py",
            ]
        )
        try:
            active = registry.active_controllers()
            self.assertNotIn(process.pid, {
                pid for pids in active.values() for pid in pids
            })
        finally:
            process.terminate()
            process.wait(timeout=5)


if __name__ == "__main__":
    unittest.main()
