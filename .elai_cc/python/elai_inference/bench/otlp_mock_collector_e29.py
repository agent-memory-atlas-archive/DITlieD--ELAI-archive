"""E-29 -- minimal local OTLP/HTTP collector mock (for sidecar-G live-flow proof).

Listens on 127.0.0.1:4318 (the OTLP/HTTP default port; :4317 is gRPC). Accepts
POST /v1/traces (resource spans) and POST /v1/metrics. Logs the payload byte
length + a parsed-span summary to stdout AND appends a JSON record to the
file specified by ELAI_OTLP_MOCK_LOG (default: ./otlp_mock_log.jsonl).

This replaces the heavyweight otelcol-contrib install for the wire-flow proof.
The orchestrator's ssb-otel-export crate uses gRPC by default; for the Python
mock we accept OTLP/HTTP (protobuf or JSON) because Python protoc support is
already in opentelemetry-proto on pypi. To keep this minimal, we accept ANY
content-type and log only the wire size + headers; the goal is to verify the
TRANSPORT path, not to deserialize the proto.

Run:
  python otlp_mock_collector_e29.py --port 4318

Probe:
  python otlp_mock_collector_e29.py --probe http://127.0.0.1:4318/v1/traces
"""
from __future__ import annotations

import argparse
import json
import os
import sys
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


LOG_FILE = os.environ.get("ELAI_OTLP_MOCK_LOG", "otlp_mock_log.jsonl")


class OtlpHandler(BaseHTTPRequestHandler):
    server_version = "elai-otlp-mock/0.1"

    def do_POST(self):  # noqa: N802
        length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(length) if length else b""
        record = {
            "ts_unix_ms": int(time.time() * 1000),
            "path": self.path,
            "method": "POST",
            "content_type": self.headers.get("Content-Type", ""),
            "content_length": length,
            "user_agent": self.headers.get("User-Agent", ""),
            "body_preview_hex": body[:64].hex(),
            "client_addr": self.client_address[0],
        }
        with open(LOG_FILE, "a", encoding="utf-8") as f:
            f.write(json.dumps(record) + "\n")
        sys.stdout.write(
            f"OTLP-MOCK recv path={self.path} ct={record['content_type']} "
            f"bytes={length} from={record['client_addr']}\n"
        )
        sys.stdout.flush()
        # Per OTLP/HTTP spec, return Partial Success (200, empty json).
        resp = b'{"partialSuccess":{}}'
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(resp)))
        self.end_headers()
        self.wfile.write(resp)

    def log_message(self, fmt, *args):  # silence default access log
        return


def serve(port: int) -> None:
    httpd = ThreadingHTTPServer(("127.0.0.1", port), OtlpHandler)
    print(f"OTLP-MOCK listening on 127.0.0.1:{port} log={LOG_FILE}", flush=True)
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        httpd.server_close()


def probe(url: str) -> None:
    """Send a minimal OTLP/HTTP traces payload (JSON shape) to verify wire."""
    import httpx
    payload = {
        "resourceSpans": [{
            "resource": {"attributes": [{"key": "service.name", "value": {"stringValue": "elai-cli"}}]},
            "scopeSpans": [{
                "scope": {"name": "elai.sidecar-G"},
                "spans": [{
                    "traceId": "0123456789abcdef0123456789abcdef",
                    "spanId": "0123456789abcdef",
                    "name": "sidecar_G.e29_probe",
                    "kind": 1,
                    "startTimeUnixNano": str(int(time.time() * 1e9)),
                    "endTimeUnixNano": str(int(time.time() * 1e9) + 1_000_000),
                    "attributes": [
                        {"key": "elai.session", "value": {"stringValue": "s-25120"}},
                        {"key": "elai.eval", "value": {"stringValue": "E-29"}},
                    ],
                    "status": {"code": 1},
                }],
            }],
        }],
    }
    r = httpx.post(url, json=payload, timeout=5.0)
    r.raise_for_status()
    print(f"PROBE ok status={r.status_code} resp={r.text}", flush=True)


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--port", type=int, default=4318)
    p.add_argument("--probe", default=None, help="if set, POST a sample payload to URL and exit")
    args = p.parse_args()

    if args.probe:
        probe(args.probe)
        return 0
    serve(args.port)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
