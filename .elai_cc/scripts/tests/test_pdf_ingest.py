"""Unit tests for .elai_cc/scripts/pdf_ingest.py.

Stdlib unittest only. Mocks the HTTP layer (urllib.request.urlopen) so no
live MinerU service is required. Runnable from the repo root:

    python -m unittest .elai_cc/scripts/tests/test_pdf_ingest.py

or from `.elai_cc` cwd:

    PYTHONPATH=. python -m unittest scripts/tests/test_pdf_ingest.py
"""
from __future__ import annotations

import io
import json
import os
import sys
import unittest
import urllib.error
from pathlib import Path
from unittest import mock

# Make `scripts.pdf_ingest` importable from either cwd.
REPO_ROOT = Path(__file__).resolve().parents[3]
ELAI_CC = REPO_ROOT / ".elai_cc"
if str(ELAI_CC) not in sys.path:
    sys.path.insert(0, str(ELAI_CC))

from scripts.pdf_ingest import ingest  # noqa: E402  (path setup above)


def _fake_pdf() -> bytes:
    return b"%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n" \
           b"2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj\n" \
           b"3 0 obj<</Type/Page/Parent 2 0 R/MediaBox[0 0 200 80]" \
           b"/Contents 4 0 R/Resources<</Font<</F1 5 0 R>>>>>>endobj\n" \
           b"4 0 obj<</Length 44>>stream\nBT /F1 12 Tf 10 60 Td (E=mc^2) Tj ET\n" \
           b"endstream\nendobj\n" \
           b"5 0 obj<</Type/Font/Subtype/Type1/BaseFont/Helvetica>>endobj\n" \
           b"xref\n0 6\n0000000000 65535 f \ntrailer<</Size 6/Root 1 0 R>>\n" \
           b"startxref\n0\n%%EOF\n"


class _FakeResponse:
    def __init__(self, status: int, body: bytes):
        self.status = status
        self._body = body
        self._read = False

    def read(self, _amt: int = -1) -> bytes:
        if self._read:
            return b""
        self._read = True
        return self._body

    def __enter__(self):
        return self

    def __exit__(self, *exc):
        return False


class PdfIngestTests(unittest.TestCase):
    def setUp(self):
        self.tmp_pdf = Path(self._tmp_pdf_path())
        self.tmp_pdf.write_bytes(_fake_pdf())
        self.addCleanup(self._cleanup_tmp)

    def _tmp_pdf_path(self) -> Path:
        import tempfile
        fd, path = tempfile.mkstemp(suffix=".pdf", prefix="j33_test_")
        os.close(fd)
        return Path(path)

    def _cleanup_tmp(self):
        try:
            self.tmp_pdf.unlink(missing_ok=True)
        except Exception:
            pass

    def _set_env(self, enable: str, base_url: str):
        patcher = mock.patch.dict(
            os.environ,
            {"ELAI_DOC_PARSER_ENABLE": enable, "ELAI_DOC_PARSER_BASE_URL": base_url},
            clear=False,
        )
        patcher.start()
        self.addCleanup(patcher.stop)

    def test_happy_path_writes_markdown_from_json(self):
        self._set_env("1", "")
        body = json.dumps({"markdown": "$E=mc^2$"}).encode("utf-8")
        with mock.patch(
            "scripts.pdf_ingest.urllib.request.urlopen",
            return_value=_FakeResponse(200, body),
        ) as urlopen_mock:
            out = Path(self.tmp_pdf.with_suffix(".md").as_posix())
            written, status = ingest(
                self.tmp_pdf, out, "http://127.0.0.1:8080", timeout=5
            )
        self.assertEqual(status, "parsed")
        self.assertTrue(written.is_file())
        self.assertIn("$E=mc^2$", written.read_text(encoding="utf-8"))
        urlopen_mock.assert_called_once()

    def test_connection_failure_degrades_without_fake_tex(self):
        self._set_env("1", "")
        def raise_url_error(*_args, **_kw):
            raise urllib.error.URLError("connection refused")
        with mock.patch(
            "scripts.pdf_ingest.urllib.request.urlopen",
            side_effect=raise_url_error,
        ):
            out = Path(self.tmp_pdf.with_suffix(".out.md").as_posix())
            written, status = ingest(
                self.tmp_pdf, out, "http://127.0.0.1:8080", timeout=5
            )
        self.assertEqual(status, "passthrough")
        self.assertTrue(written.is_file())
        text = written.read_text(encoding="utf-8")
        self.assertIn("# pdf_ingest: passthrough", text)
        self.assertIn("connection error", text)
        # No fabricated TeX content.
        self.assertNotIn("$E=mc^2$", text)
        self.assertNotIn("E=mc^2", text)

    def test_flag_off_degrades_to_passthrough(self):
        # ELAI_DOC_PARSER_ENABLE explicitly off; no HTTP call should fire.
        with mock.patch.dict(
            os.environ,
            {"ELAI_DOC_PARSER_ENABLE": "0", "ELAI_DOC_PARSER_BASE_URL": "http://x"},
            clear=False,
        ):
            with mock.patch(
                "scripts.pdf_ingest.urllib.request.urlopen",
                side_effect=AssertionError("must not call HTTP when disabled"),
            ):
                out = Path(self.tmp_pdf.with_suffix(".off.md").as_posix())
                written, status = ingest(
                    self.tmp_pdf, out, None, timeout=5
                )
        self.assertEqual(status, "passthrough")
        self.assertTrue(written.is_file())
        text = written.read_text(encoding="utf-8")
        self.assertIn("# pdf_ingest: passthrough", text)
        self.assertIn("ELAI_DOC_PARSER_ENABLE not truthy", text)

    def test_non2xx_degrades_to_passthrough(self):
        self._set_env("1", "")
        body = b"internal error"
        with mock.patch(
            "scripts.pdf_ingest.urllib.request.urlopen",
            side_effect=urllib.error.HTTPError(
                url="http://x/file_parse",
                code=500,
                msg="Internal Server Error",
                hdrs=None,
                fp=io.BytesIO(body),
            ),
        ):
            out = Path(self.tmp_pdf.with_suffix(".err.md").as_posix())
            written, status = ingest(
                self.tmp_pdf, out, "http://127.0.0.1:8080", timeout=5
            )
        self.assertEqual(status, "passthrough")
        text = written.read_text(encoding="utf-8")
        self.assertIn("non-2xx", text)


if __name__ == "__main__":
    unittest.main()
