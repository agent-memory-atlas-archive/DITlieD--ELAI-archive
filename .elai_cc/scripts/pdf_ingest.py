"""Ingest a PDF via the ELAI doc-parser (MinerU) HTTP endpoint.

Mirrors the Rust DocParserClient contract (crates/.../doc_parser_client.rs @c420118):
  - env: ELAI_DOC_PARSER_ENABLE (truthy), ELAI_DOC_PARSER_BASE_URL
  - POST {base_url}/file_parse multipart/form-data field name `file`
  - response JSON: prefer `markdown`, fallback `content`

Degrades to a passthrough markdown stub when the service is disabled,
unreachable, returns non-2xx, or the base URL is missing. The passthrough
file records WHY and points at the absolute PDF path; it NEVER fabricates
parsed TeX content. Exit 0 in both success and degrade paths so callers
(paper-ingestor) still get a markdown file to ingest; the degrade is
signalled on stderr with a WARN line.

Usage:
    python .elai_cc/scripts/pdf_ingest.py <pdf-path> [--out PATH] \\
        [--base-url URL] [--timeout SECS]

Stdlib only (urllib.request) -- no new package deps.
"""
from __future__ import annotations

import argparse
import json
import os
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Optional, Tuple


PROG = "pdf_ingest.py"
BOUNDARY = "----elai_pdf_ingest_boundary"


def _truthy(raw: Optional[str]) -> bool:
    return bool(raw) and raw.strip().lower() not in {"0", "false", "off", "no", ""}


def _multipart_field(field_name: str, filename: str, file_bytes: bytes) -> bytes:
    head = (
        f"--{BOUNDARY}\r\n"
        f'Content-Disposition: form-data; name="{field_name}"; '
        f'filename="{filename}"\r\n'
        f"Content-Type: application/pdf\r\n\r\n"
    ).encode("utf-8")
    return head + file_bytes + f"\r\n--{BOUNDARY}--\r\n".encode("utf-8")


def _parse_response(body: bytes) -> str:
    if not body:
        raise ValueError("empty response body")
    payload = json.loads(body.decode("utf-8"))
    markdown = payload.get("markdown")
    if isinstance(markdown, str) and markdown:
        return markdown
    content = payload.get("content")
    if isinstance(content, str) and content:
        return content
    raise ValueError(f"response JSON missing `markdown` and `content`: {payload!r}")


def _http_post(base_url: str, pdf_path: Path, timeout: float) -> Tuple[int, bytes]:
    data = _multipart_field("file", pdf_path.name, pdf_path.read_bytes())
    url = base_url.rstrip("/") + "/file_parse"
    req = urllib.request.Request(
        url,
        data=data,
        method="POST",
        headers={
            "Content-Type": f"multipart/form-data; boundary={BOUNDARY}",
            "User-Agent": "elai-pdf-ingest/1.0",
        },
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return resp.status, resp.read()
    except urllib.error.HTTPError as exc:
        body = b""
        try:
            body = exc.read()
        except Exception:
            pass
        return exc.code, body
    except (urllib.error.URLError, ConnectionError, TimeoutError, OSError) as exc:
        raise ConnectionError(f"unreachable: {url}: {exc}") from exc


def _write_passthrough(out_path: Path, pdf_path: Path, reason: str) -> Path:
    abs_pdf = str(pdf_path.resolve())
    body = (
        "# pdf_ingest: passthrough\n\n"
        f"reason: {reason}\n\n"
        f"source_pdf: {abs_pdf}\n\n"
        "The ELAI doc-parser (MinerU) was not used; this file is a stub so "
        "downstream ingestors still have a path. No parsed content was "
        "fabricated; the original PDF must be re-ingested when the service "
        "is available.\n"
    )
    out_path.write_text(body, encoding="utf-8")
    return out_path


def _default_out(pdf_path: Path, passthrough: bool) -> Path:
    stem = pdf_path.stem
    suffix = ".passthrough.md" if passthrough else ".md"
    return pdf_path.with_name(stem + suffix)


def ingest(
    pdf_path: Path,
    out_path: Optional[Path],
    base_url: Optional[str],
    timeout: float,
) -> Tuple[Path, str]:
    """Return (written_path, status). status in {"parsed", "passthrough"}."""
    enabled_env = os.environ.get("ELAI_DOC_PARSER_ENABLE")
    base_env = os.environ.get("ELAI_DOC_PARSER_BASE_URL")
    enabled = _truthy(enabled_env)
    effective_base = (base_url or base_env or "").strip()

    if not enabled:
        target = out_path or _default_out(pdf_path, passthrough=True)
        p = _write_passthrough(target, pdf_path, "ELAI_DOC_PARSER_ENABLE not truthy")
        return p, "passthrough"

    if not effective_base:
        target = out_path or _default_out(pdf_path, passthrough=True)
        p = _write_passthrough(
            target, pdf_path, "ELAI_DOC_PARSER_ENABLE truthy but base_url missing"
        )
        return p, "passthrough"

    try:
        status, body = _http_post(effective_base, pdf_path, timeout)
    except ConnectionError as exc:
        target = out_path or _default_out(pdf_path, passthrough=True)
        p = _write_passthrough(target, pdf_path, f"connection error: {exc}")
        return p, "passthrough"

    if not (200 <= status < 300):
        target = out_path or _default_out(pdf_path, passthrough=True)
        p = _write_passthrough(
            target, pdf_path, f"non-2xx response: status={status}"
        )
        return p, "passthrough"

    try:
        markdown = _parse_response(body)
    except (ValueError, json.JSONDecodeError) as exc:
        target = out_path or _default_out(pdf_path, passthrough=True)
        p = _write_passthrough(
            target, pdf_path, f"malformed response body: {exc}"
        )
        return p, "passthrough"

    target = out_path or _default_out(pdf_path, passthrough=False)
    target.write_text(markdown, encoding="utf-8")
    return target, "parsed"


def main(argv: Optional[list] = None) -> int:
    parser = argparse.ArgumentParser(
        prog=PROG,
        description=(
            "Ingest a PDF via the ELAI doc-parser (MinerU). Degrades to a "
            "passthrough stub when the service is disabled or unreachable."
        ),
    )
    parser.add_argument("pdf_path", help="Path to the PDF to ingest.")
    parser.add_argument(
        "--out",
        default=None,
        help="Output markdown path. Defaults to <stem>.md or <stem>.passthrough.md.",
    )
    parser.add_argument(
        "--base-url",
        default=None,
        help="MinerU base URL. Overrides ELAI_DOC_PARSER_BASE_URL.",
    )
    parser.add_argument(
        "--timeout",
        type=float,
        default=60.0,
        help="HTTP timeout in seconds (default 60).",
    )
    args = parser.parse_args(argv)

    pdf_path = Path(args.pdf_path).expanduser()
    if not pdf_path.is_file():
        print(f"pdf_ingest: not a file: {pdf_path}", file=sys.stderr)
        return 2

    out_path = Path(args.out).expanduser() if args.out else None
    written, status = ingest(
        pdf_path, out_path, args.base_url, args.timeout
    )
    if status == "passthrough":
        print(
            f"pdf_ingest: WARN passthrough -- service unavailable or disabled; "
            f"see {written}",
            file=sys.stderr,
        )
    print(written)
    return 0


if __name__ == "__main__":
    sys.exit(main())
