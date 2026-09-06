from __future__ import annotations

import tempfile
from pathlib import Path


class TempDirMixin:
    def make_tempdir(self) -> Path:
        temp = tempfile.TemporaryDirectory(prefix="conductor-test-")
        self.addCleanup(temp.cleanup)
        return Path(temp.name)
