"""phase-3J-BASELINE Token-Efficiency Benchmark Harness.

Standalone stdlib-only harness that measures the token economy of every shipped
ModelCapabilityProfile tier and writes metrics/capability_profile.toml (+ a
sibling .csv) as the canonical input the runtime consumes when sizing slots.

Spec: docs/specs/guide/steps/phase-3J-BASELINE.txt

DO NOT confuse this with run_benchmark.py — that file is the pre-existing
C-26/A-1 attention/few-shot/structured measurement harness owned by other
phases (3I, 4D-eval). The 3J-BASELINE harness is a SEPARATE entry point so
the two roles do not collide.

Usage:
    python baseline_harness.py --profile micro_7b_12gb
    python baseline_harness.py --profile all
    python baseline_harness.py --profile cloud_200k --skip-repair-context

Exit codes:
    0  all gates pass
    1  at least one gate (G1..G4) failed
    2  harness error (sidecar unreachable, fixture/config missing, etc.)

Imports: stdlib only per spec stub_ban (urllib.request + tomllib + json + csv
+ argparse + time + statistics + pathlib + sys + os + dataclasses + datetime).
No `requests`, no `orjson`, no in-tree crate imports, no source-simplifier
import (the runtime AST simplifier is the runtime's concern, not the bench's).

Stub-ban rationale and compression_ratio methodology
----------------------------------------------------

Spec requires a `compression_ratio` column derived from the runtime source
simplifier but ALSO bans importing in-tree crates (the simplifier lives in
Rust at crates/knowledge/src/shortcoder.rs:101 and there is no
/v1/shortcoder/preview sidecar endpoint as of this writing — verified by
exploration at workflow/sessions/s-25120/completion-drive/exploration/
phase-3J-BASELINE-exploration.md GAP-2).

Resolution: this harness emits `compression_ratio_estimate` computed as
tokens_out / tokens_in across the fixture (output_token_ratio method),
explicitly annotated with measurement_method = "output_token_ratio" in the
output TOML's [methodology] table. The runtime-side simplifier ratio is a
DIFFERENT semantic (simplified_bytes / original_bytes of source code) and
is the runtime's measurement to take, not the bench harness's.

Tier roster (matches ModelCapabilityProfile constructors in
crates/orchestrator/src/context_budget.rs:309-657):
    micro_8gb, micro_3b_12gb, micro_7b_12gb,
    awq_3b_8gb, awq_3b_12gb, awq_7b_12gb,
    large_32b_48gb, cloud_200k.

Cloud pricing
-------------

Cloud-tier rows compute cost_per_run_usd from the CLOUD_PRICING table below.
The numbers are placeholder/literal entries — the harness flags any cloud
row whose pricing tier is not pinned in CLOUD_PRICING and writes
`cost_per_run_usd = 0.0` with `cost_pricing_unknown = true` rather than
silently inventing a number. Update CLOUD_PRICING after verifying current
Anthropic pricing at api.anthropic.com/pricing before relying on cost rows.
"""

import argparse
import csv
import json
import os
import statistics
import sys
import time
import urllib.error
import urllib.request
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

# tomllib is stdlib since Python 3.11. Fall back to tomli for older interpreters
# but DO NOT silently degrade — emit a clear error if neither is present.
try:
    import tomllib as _toml_reader
    _TOML_SOURCE = "tomllib (stdlib 3.11+)"
except ModuleNotFoundError:
    try:
        import tomli as _toml_reader  # type: ignore[no-redef]
        _TOML_SOURCE = "tomli (Python <3.11 fallback)"
    except ModuleNotFoundError:
        print(
            "[baseline_harness] ERROR: neither tomllib (stdlib 3.11+) nor "
            "tomli is available; install Python 3.11+ or `pip install tomli`",
            file=sys.stderr,
        )
        raise SystemExit(2)


SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parent.parent  # .../.elai_cc/
DEFAULT_CONFIG = REPO_ROOT / "elai_cc.toml"
DEFAULT_OUTPUT_TOML = REPO_ROOT / "metrics" / "capability_profile.toml"
DEFAULT_OUTPUT_CSV = REPO_ROOT / "metrics" / "capability_profile.csv"

REQUEST_TIMEOUT_S = 180
HEALTH_TIMEOUT_S = 10

EXIT_OK = 0
EXIT_GATE_FAILED = 1
EXIT_HARNESS_ERROR = 2

# Cloud pricing per-MTok input/output (USD). Update after verifying current
# Anthropic pricing — see top-of-file comment block.
CLOUD_PRICING: dict[str, dict[str, float]] = {
    "claude-sonnet-4-6": {"input_per_mtok": 3.0, "output_per_mtok": 15.0},
}


# ---------------------------------------------------------------------------
# Config + fixture loaders
# ---------------------------------------------------------------------------


@dataclass
class ProfileConfig:
    """One [model_capability.profiles.<tier>] entry from elai_cc.toml."""

    tier: str
    model: str
    runtime: str
    quant_format: str
    prompt_tokens_target: int
    output_reserve: int
    effective_ceiling: int
    expected_slot_utilization_floor: float
    expected_compression_ratio_floor: float
    fixture: str


@dataclass
class HarnessConfig:
    sidecar_endpoint: str
    cloud_endpoint: str
    output_toml: Path
    output_csv: Path
    default_profiles: list[str]
    repair_context_total: int
    repair_context_threshold_pct: float
    constrained_mode: str
    constrained_invalid_allowed: int
    constrained_fixture: str
    profiles: dict[str, ProfileConfig] = field(default_factory=dict)


def _load_toml(path: Path) -> dict[str, Any]:
    if not path.is_file():
        print(
            f"[baseline_harness] ERROR: config file not found at {path}",
            file=sys.stderr,
        )
        raise SystemExit(EXIT_HARNESS_ERROR)
    with path.open("rb") as f:
        return _toml_reader.load(f)


def load_harness_config(config_path: Path) -> HarnessConfig:
    """Load [model_capability.*] tables from elai_cc.toml.

    Hard-fails with exit code 2 (harness error) if the section is absent or
    malformed; the spec stub_ban forbids guessing tier values.
    """
    raw = _load_toml(config_path)
    mc = raw.get("model_capability")
    if mc is None:
        print(
            f"[baseline_harness] ERROR: [model_capability] section missing "
            f"from {config_path}. Add it per "
            f"docs/specs/guide/steps/phase-3J-BASELINE.txt §Spec references.",
            file=sys.stderr,
        )
        raise SystemExit(EXIT_HARNESS_ERROR)

    profiles_table = mc.get("profiles")
    if not isinstance(profiles_table, dict) or not profiles_table:
        print(
            "[baseline_harness] ERROR: [model_capability.profiles.*] is empty "
            f"in {config_path}; cannot iterate tiers.",
            file=sys.stderr,
        )
        raise SystemExit(EXIT_HARNESS_ERROR)

    repair = mc.get("repair_context", {})
    constrained = mc.get("constrained_decoding", {})

    profiles: dict[str, ProfileConfig] = {}
    for tier, entry in profiles_table.items():
        try:
            profiles[tier] = ProfileConfig(
                tier=tier,
                model=str(entry["model"]),
                runtime=str(entry["runtime"]),
                quant_format=str(entry["quant_format"]),
                prompt_tokens_target=int(entry["prompt_tokens_target"]),
                output_reserve=int(entry["output_reserve"]),
                effective_ceiling=int(entry["effective_ceiling"]),
                expected_slot_utilization_floor=float(
                    entry["expected_slot_utilization_floor"]
                ),
                expected_compression_ratio_floor=float(
                    entry["expected_compression_ratio_floor"]
                ),
                fixture=str(entry["fixture"]),
            )
        except (KeyError, TypeError, ValueError) as exc:
            print(
                f"[baseline_harness] ERROR: [model_capability.profiles."
                f"{tier}] malformed: {exc!r}",
                file=sys.stderr,
            )
            raise SystemExit(EXIT_HARNESS_ERROR) from exc

    return HarnessConfig(
        sidecar_endpoint=str(mc.get("sidecar_endpoint", "http://127.0.0.1:8100")),
        cloud_endpoint=str(mc.get("cloud_endpoint", "https://api.anthropic.com")),
        output_toml=REPO_ROOT / str(mc.get("output_path", "metrics/capability_profile.toml")),
        output_csv=REPO_ROOT / str(mc.get("output_csv_path", "metrics/capability_profile.csv")),
        default_profiles=list(mc.get("default_profiles", list(profiles.keys()))),
        repair_context_total=int(repair.get("total_token_budget", 1400)),
        repair_context_threshold_pct=float(repair.get("regression_threshold_pct", 15.0)),
        constrained_mode=str(constrained.get("mode", "json_schema")),
        constrained_invalid_allowed=int(constrained.get("invalid_outputs_allowed", 0)),
        constrained_fixture=str(
            constrained.get(
                "json_schema_fixture",
                "benches/model_capability/fixtures/json_schema.toml",
            )
        ),
        profiles=profiles,
    )


def load_fixture(path: Path) -> dict[str, Any]:
    raw = _load_toml(path)
    fix = raw.get("fixture")
    if fix is None or not isinstance(fix, dict):
        print(
            f"[baseline_harness] ERROR: {path} is missing [fixture] section.",
            file=sys.stderr,
        )
        raise SystemExit(EXIT_HARNESS_ERROR)
    prompts = fix.get("prompts", [])
    if not isinstance(prompts, list) or len(prompts) < 1:
        print(
            f"[baseline_harness] ERROR: {path} has no prompts.",
            file=sys.stderr,
        )
        raise SystemExit(EXIT_HARNESS_ERROR)
    return fix


# ---------------------------------------------------------------------------
# HTTP transport (urllib only — spec stub_ban forbids `requests`)
# ---------------------------------------------------------------------------


def _http_request(
    url: str,
    *,
    method: str = "GET",
    body: bytes | None = None,
    headers: dict[str, str] | None = None,
    timeout: float = REQUEST_TIMEOUT_S,
) -> tuple[int, bytes]:
    """Stdlib HTTP. Returns (status_code, body_bytes). Never silently swallows."""
    req = urllib.request.Request(url, data=body, method=method)
    if headers:
        for k, v in headers.items():
            req.add_header(k, v)
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:  # noqa: S310 (sidecar URL is operator-controlled)
            return resp.status, resp.read()
    except urllib.error.HTTPError as e:
        # HTTPError IS a response — read it so callers can surface server errors.
        return e.code, e.read()
    except (urllib.error.URLError, TimeoutError, OSError) as e:
        print(
            f"[baseline_harness] ERROR: transport failure to {url}: "
            f"{type(e).__name__}: {e}",
            file=sys.stderr,
        )
        raise SystemExit(EXIT_HARNESS_ERROR) from e


def preflight_health(endpoint: str) -> None:
    """GET /health on the sidecar; fail-fast on any non-200 or transport error.

    Per spec stub_ban + anti-lazy rule 7: this MUST NOT silently swallow — a
    sidecar that is unreachable is an unambiguous harness-error condition
    (exit 2) and operators need a clear cause printed to stderr.
    """
    url = endpoint.rstrip("/") + "/health"
    status, body = _http_request(url, method="GET", timeout=HEALTH_TIMEOUT_S)
    if status != 200:
        print(
            f"[baseline_harness] ERROR: sidecar /health returned status="
            f"{status} body={body!r} — boot the sidecar (see CLAUDE.md "
            f"'Remote sidecar (canonical)' block) before re-running.",
            file=sys.stderr,
        )
        raise SystemExit(EXIT_HARNESS_ERROR)
    try:
        payload = json.loads(body)
    except json.JSONDecodeError as e:
        print(
            f"[baseline_harness] ERROR: /health returned non-JSON body "
            f"{body!r}: {e}",
            file=sys.stderr,
        )
        raise SystemExit(EXIT_HARNESS_ERROR) from e
    if payload.get("status") != "healthy":
        print(
            f"[baseline_harness] ERROR: sidecar /health status != 'healthy' "
            f"(got {payload!r})",
            file=sys.stderr,
        )
        raise SystemExit(EXIT_HARNESS_ERROR)


def query_loaded_model(endpoint: str) -> str | None:
    """GET /v1/models for telemetry. Returns model id or None if unavailable."""
    url = endpoint.rstrip("/") + "/v1/models"
    try:
        status, body = _http_request(url, method="GET", timeout=HEALTH_TIMEOUT_S)
    except SystemExit:
        # /v1/models is a nice-to-have; transport error here should NOT abort
        # the whole run if /health was already green. Re-raise only on the
        # health probe; downgrade to None here.
        return None
    if status != 200:
        return None
    try:
        payload = json.loads(body)
        data = payload.get("data", [])
        if data:
            return str(data[0].get("id", "")) or None
    except json.JSONDecodeError:
        return None
    return None


# ---------------------------------------------------------------------------
# Prompt dispatch + measurement
# ---------------------------------------------------------------------------


@dataclass
class PromptResult:
    prompt_id: str
    tokens_in: int
    tokens_out: int
    latency_ms: float
    json_invalid: bool
    error: str | None


def _build_messages_payload(
    *,
    model: str,
    prompt_text: str,
    max_tokens: int,
    temperature: float,
    schema: str | None,
) -> dict[str, Any]:
    payload: dict[str, Any] = {
        "model": model,
        "messages": [{"role": "user", "content": prompt_text}],
        "max_tokens": max_tokens,
        "temperature": temperature,
    }
    if schema is not None:
        # Schema is stored as a TOML string of JSON for fixture readability;
        # parse it now so the sidecar receives a dict, matching the shape
        # _build_constrained_prefix_fn expects at server.py:1874-1888.
        try:
            schema_dict = json.loads(schema)
        except json.JSONDecodeError as e:
            print(
                f"[baseline_harness] ERROR: fixture schema is not valid JSON: "
                f"{schema!r} ({e})",
                file=sys.stderr,
            )
            raise SystemExit(EXIT_HARNESS_ERROR) from e
        payload["response_format"] = {
            "type": "json_schema",
            "json_schema": {"schema": schema_dict},
        }
    return payload


def _extract_text_and_usage(body: bytes) -> tuple[str, int, int]:
    """Parse Anthropic /v1/messages response. Returns (text, tokens_in, tokens_out)."""
    payload = json.loads(body)
    parts: list[str] = []
    for block in payload.get("content", []):
        if isinstance(block, dict) and block.get("type") == "text":
            parts.append(str(block.get("text", "")))
    text = "".join(parts)
    usage = payload.get("usage", {}) or {}
    tokens_in = int(usage.get("input_tokens", 0))
    tokens_out = int(usage.get("output_tokens", 0))
    return text, tokens_in, tokens_out


def dispatch_prompt(
    *,
    endpoint: str,
    model: str,
    prompt: dict[str, Any],
    default_max_tokens: int,
    default_temperature: float,
    api_key: str | None = None,
) -> PromptResult:
    """POST a single prompt to /v1/messages and capture the measurement row."""
    schema_raw = prompt.get("schema")
    schema = str(schema_raw) if schema_raw is not None else None
    expect_json = bool(prompt.get("expect_json", False))
    max_tokens = int(prompt.get("max_tokens", default_max_tokens))
    temperature = float(prompt.get("temperature", default_temperature))
    payload = _build_messages_payload(
        model=model,
        prompt_text=str(prompt["text"]),
        max_tokens=max_tokens,
        temperature=temperature,
        schema=schema if expect_json else None,
    )
    body_bytes = json.dumps(payload).encode("utf-8")
    headers = {"Content-Type": "application/json"}
    if api_key:
        # Anthropic cloud auth — only attach when an explicit key is provided
        # via env. Sidecar accepts unauthenticated requests on the loopback
        # tunnel per CLAUDE.md sidecar boot block.
        headers["x-api-key"] = api_key
        headers["anthropic-version"] = "2023-06-01"

    url = endpoint.rstrip("/") + "/v1/messages"
    t0 = time.perf_counter()
    status, resp_body = _http_request(
        url, method="POST", body=body_bytes, headers=headers,
        timeout=REQUEST_TIMEOUT_S,
    )
    latency_ms = (time.perf_counter() - t0) * 1000.0

    if status != 200:
        return PromptResult(
            prompt_id=str(prompt["id"]),
            tokens_in=0,
            tokens_out=0,
            latency_ms=latency_ms,
            json_invalid=expect_json,
            error=f"http_status={status} body={resp_body[:256]!r}",
        )

    try:
        text, tokens_in, tokens_out = _extract_text_and_usage(resp_body)
    except (json.JSONDecodeError, ValueError, KeyError) as exc:
        return PromptResult(
            prompt_id=str(prompt["id"]),
            tokens_in=0,
            tokens_out=0,
            latency_ms=latency_ms,
            json_invalid=expect_json,
            error=f"response_parse_failed: {type(exc).__name__}: {exc}",
        )

    json_invalid = False
    if expect_json:
        stripped = text.strip()
        # Strip optional ```json fences a model might wrap output in
        if stripped.startswith("```"):
            lines = stripped.splitlines()
            if len(lines) >= 2:
                stripped = "\n".join(lines[1:-1]) if lines[-1].startswith("```") else "\n".join(lines[1:])
        try:
            json.loads(stripped)
        except json.JSONDecodeError:
            json_invalid = True

    return PromptResult(
        prompt_id=str(prompt["id"]),
        tokens_in=tokens_in,
        tokens_out=tokens_out,
        latency_ms=latency_ms,
        json_invalid=json_invalid,
        error=None,
    )


# ---------------------------------------------------------------------------
# Per-tier measurement aggregation
# ---------------------------------------------------------------------------


@dataclass
class TierMeasurement:
    tier: str
    model: str
    runtime: str
    quant_format: str
    samples: int
    total_tokens_in: int
    total_tokens_out: int
    slot_utilization: float
    compression_ratio_estimate: float
    ttft_ms_median: float
    p95_latency_ms: float
    p99_latency_ms: float
    cost_per_run_usd: float
    cost_pricing_unknown: bool
    json_invalid_count: int
    repair_context_delta_pct: float | None
    repair_context_regression: bool
    measured_at_utc: str
    gates: dict[str, bool]
    errors: list[str]
    measurement_method_compression: str = "output_token_ratio"


def _percentile(values: list[float], pct: float) -> float:
    if not values:
        return 0.0
    if len(values) == 1:
        return values[0]
    ordered = sorted(values)
    # statistics.quantiles requires n>=2; use exclusive method for tail percentiles.
    if pct >= 100.0:
        return ordered[-1]
    if pct <= 0.0:
        return ordered[0]
    k = max(1, min(99, int(round(pct))))
    qs = statistics.quantiles(ordered, n=100, method="exclusive")
    return float(qs[k - 1])


def _compute_cost_usd(model: str, tokens_in: int, tokens_out: int) -> tuple[float, bool]:
    pricing = CLOUD_PRICING.get(model)
    if pricing is None:
        return 0.0, True
    cost = (
        tokens_in / 1_000_000.0 * pricing["input_per_mtok"]
        + tokens_out / 1_000_000.0 * pricing["output_per_mtok"]
    )
    return cost, False


def run_tier(
    *,
    profile: ProfileConfig,
    cfg: HarnessConfig,
    fixture_path: Path,
    repair_context_active: bool,
    api_key: str | None,
) -> tuple[list[PromptResult], dict[str, Any]]:
    """Execute all prompts in a fixture for the given tier."""
    fix = load_fixture(fixture_path)
    default_max_tokens = int(fix.get("default_max_tokens", 256))
    default_temperature = float(fix.get("default_temperature", 0.0))

    # Endpoint selection: cloud-tier rows use the cloud_endpoint; everything
    # else hits the local sidecar tunnel.
    endpoint = cfg.cloud_endpoint if profile.runtime == "cloud" else cfg.sidecar_endpoint

    results: list[PromptResult] = []
    for prompt in fix.get("prompts", []):
        prompt_copy = dict(prompt)
        if repair_context_active:
            # G3 simulation: prepend a 1400-token repair-context proxy ASCII
            # block so tokens_in grows. The sidecar does NOT currently honour
            # a request-level REPAIR_CONTEXT toggle (verified: orchestrator/
            # src/context_budget.rs:130 constant lives runtime-side, no HTTP
            # surface) — see exploration GAP-6. This emulates the budget
            # delta by sending an extra context block.
            proxy = ("REPAIR_CONTEXT proxy block. " * 100)[: cfg.repair_context_total * 4]
            prompt_copy["text"] = proxy + "\n\n" + str(prompt_copy["text"])
        results.append(
            dispatch_prompt(
                endpoint=endpoint,
                model=profile.model,
                prompt=prompt_copy,
                default_max_tokens=default_max_tokens,
                default_temperature=default_temperature,
                api_key=api_key if profile.runtime == "cloud" else None,
            )
        )

    meta = {
        "default_max_tokens": default_max_tokens,
        "default_temperature": default_temperature,
        "endpoint": endpoint,
    }
    return results, meta


def aggregate_tier(
    *,
    profile: ProfileConfig,
    cfg: HarnessConfig,
    results_baseline: list[PromptResult],
    results_repair: list[PromptResult] | None,
) -> TierMeasurement:
    total_in = sum(r.tokens_in for r in results_baseline)
    total_out = sum(r.tokens_out for r in results_baseline)
    samples = len(results_baseline)
    errors = [f"{r.prompt_id}: {r.error}" for r in results_baseline if r.error]

    target = max(1, profile.prompt_tokens_target)
    avg_in = total_in / max(1, samples)
    slot_utilization = avg_in / target

    compression_ratio = (total_out / total_in) if total_in > 0 else 0.0

    latencies = [r.latency_ms for r in results_baseline if r.error is None]
    ttft_median = statistics.median(latencies) if latencies else 0.0
    p95 = _percentile(latencies, 95.0)
    p99 = _percentile(latencies, 99.0)

    if profile.runtime == "cloud":
        cost, pricing_unknown = _compute_cost_usd(profile.model, total_in, total_out)
    else:
        cost, pricing_unknown = 0.0, False

    json_invalid_count = sum(1 for r in results_baseline if r.json_invalid)

    repair_delta_pct: float | None = None
    repair_regression = False
    if results_repair is not None:
        total_in_repair = sum(r.tokens_in for r in results_repair)
        if total_in > 0:
            repair_delta_pct = (total_in_repair - total_in) / total_in * 100.0
            repair_regression = repair_delta_pct > cfg.repair_context_threshold_pct
        else:
            repair_delta_pct = 0.0

    gates = {
        "G2_slot_utilization": slot_utilization >= profile.expected_slot_utilization_floor,
        "G2_compression_ratio": compression_ratio >= profile.expected_compression_ratio_floor,
        "G3_repair_context": (
            not repair_regression
            if results_repair is not None
            else True
        ),
    }

    return TierMeasurement(
        tier=profile.tier,
        model=profile.model,
        runtime=profile.runtime,
        quant_format=profile.quant_format,
        samples=samples,
        total_tokens_in=total_in,
        total_tokens_out=total_out,
        slot_utilization=round(slot_utilization, 4),
        compression_ratio_estimate=round(compression_ratio, 4),
        ttft_ms_median=round(ttft_median, 2),
        p95_latency_ms=round(p95, 2),
        p99_latency_ms=round(p99, 2),
        cost_per_run_usd=round(cost, 6),
        cost_pricing_unknown=pricing_unknown,
        json_invalid_count=json_invalid_count,
        repair_context_delta_pct=(
            round(repair_delta_pct, 2) if repair_delta_pct is not None else None
        ),
        repair_context_regression=repair_regression,
        measured_at_utc=datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        gates=gates,
        errors=errors,
    )


# ---------------------------------------------------------------------------
# Output writers (TOML + CSV — stdlib only; tomllib reads but does not write,
# so we serialise by hand into a strict subset of TOML)
# ---------------------------------------------------------------------------


def _toml_escape(value: str) -> str:
    return value.replace("\\", "\\\\").replace('"', '\\"')


def _toml_scalar(value: Any) -> str:
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, int):
        return str(value)
    if isinstance(value, float):
        return repr(value)
    if value is None:
        # TOML has no null. Use an explicit absent-marker string so the
        # runtime loader can distinguish "not measured" from "measured 0".
        return '"__null__"'
    return f'"{_toml_escape(str(value))}"'


def write_capability_profile_toml(
    cfg: HarnessConfig,
    measurements: list[TierMeasurement],
    constrained_invalid: int,
) -> None:
    cfg.output_toml.parent.mkdir(parents=True, exist_ok=True)
    now_utc = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    lines: list[str] = []
    lines.append(
        "# metrics/capability_profile.toml — written by "
        "benches/model_capability/baseline_harness.py"
    )
    lines.append(f"# Generated: {now_utc}")
    lines.append("")
    lines.append("[schema]")
    lines.append("version = 1")
    lines.append('generator = "baseline_harness.py"')
    lines.append("")
    lines.append("[methodology]")
    lines.append(
        'compression_ratio_estimate = "output_token_ratio (tokens_out/tokens_in '
        'over the fixture); see baseline_harness.py top-of-file comment block"'
    )
    lines.append(
        'slot_utilization = "avg_tokens_in / prompt_tokens_target from '
        '[model_capability.profiles.<tier>]"'
    )
    lines.append(
        'constrained_decoding_mode = "json_schema only — grammar/EBNF DEFERRED '
        'per phase-3J-BASELINE.txt"'
    )
    lines.append("")
    lines.append("[constrained_decoding_gate]")
    lines.append(f"invalid_outputs = {constrained_invalid}")
    lines.append(f"invalid_outputs_allowed = {cfg.constrained_invalid_allowed}")
    lines.append(
        "passed = "
        + ("true" if constrained_invalid <= cfg.constrained_invalid_allowed else "false")
    )
    lines.append("")

    for m in measurements:
        lines.append(f"[profiles.{m.tier}]")
        lines.append(f"tier = {_toml_scalar(m.tier)}")
        lines.append(f"model = {_toml_scalar(m.model)}")
        lines.append(f"runtime = {_toml_scalar(m.runtime)}")
        lines.append(f"quant_format = {_toml_scalar(m.quant_format)}")
        lines.append(f"samples = {_toml_scalar(m.samples)}")
        lines.append(f"total_tokens_in = {_toml_scalar(m.total_tokens_in)}")
        lines.append(f"total_tokens_out = {_toml_scalar(m.total_tokens_out)}")
        lines.append(f"slot_utilization = {_toml_scalar(m.slot_utilization)}")
        lines.append(
            f"compression_ratio_estimate = {_toml_scalar(m.compression_ratio_estimate)}"
        )
        lines.append(
            f"measurement_method_compression = "
            f"{_toml_scalar(m.measurement_method_compression)}"
        )
        lines.append(f"ttft_ms_median = {_toml_scalar(m.ttft_ms_median)}")
        lines.append(f"p95_latency_ms = {_toml_scalar(m.p95_latency_ms)}")
        lines.append(f"p99_latency_ms = {_toml_scalar(m.p99_latency_ms)}")
        lines.append(f"cost_per_run_usd = {_toml_scalar(m.cost_per_run_usd)}")
        lines.append(f"cost_pricing_unknown = {_toml_scalar(m.cost_pricing_unknown)}")
        lines.append(f"json_invalid_count = {_toml_scalar(m.json_invalid_count)}")
        lines.append(
            f"repair_context_delta_pct = {_toml_scalar(m.repair_context_delta_pct)}"
        )
        lines.append(
            f"repair_context_regression = {_toml_scalar(m.repair_context_regression)}"
        )
        lines.append(f"measured_at_utc = {_toml_scalar(m.measured_at_utc)}")
        for gate, ok in m.gates.items():
            lines.append(f"gate_{gate} = {_toml_scalar(ok)}")
        if m.errors:
            lines.append("errors = [")
            for err in m.errors:
                lines.append(f"  {_toml_scalar(err)},")
            lines.append("]")
        else:
            lines.append("errors = []")
        lines.append("")

    cfg.output_toml.write_text("\n".join(lines) + "\n", encoding="utf-8")


def write_capability_profile_csv(
    cfg: HarnessConfig, measurements: list[TierMeasurement]
) -> None:
    cfg.output_csv.parent.mkdir(parents=True, exist_ok=True)
    fieldnames = [
        "tier",
        "model",
        "runtime",
        "quant_format",
        "samples",
        "total_tokens_in",
        "total_tokens_out",
        "slot_utilization",
        "compression_ratio_estimate",
        "measurement_method_compression",
        "ttft_ms_median",
        "p95_latency_ms",
        "p99_latency_ms",
        "cost_per_run_usd",
        "cost_pricing_unknown",
        "json_invalid_count",
        "repair_context_delta_pct",
        "repair_context_regression",
        "measured_at_utc",
        "gate_G2_slot_utilization",
        "gate_G2_compression_ratio",
        "gate_G3_repair_context",
    ]
    with cfg.output_csv.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        for m in measurements:
            writer.writerow(
                {
                    "tier": m.tier,
                    "model": m.model,
                    "runtime": m.runtime,
                    "quant_format": m.quant_format,
                    "samples": m.samples,
                    "total_tokens_in": m.total_tokens_in,
                    "total_tokens_out": m.total_tokens_out,
                    "slot_utilization": m.slot_utilization,
                    "compression_ratio_estimate": m.compression_ratio_estimate,
                    "measurement_method_compression": m.measurement_method_compression,
                    "ttft_ms_median": m.ttft_ms_median,
                    "p95_latency_ms": m.p95_latency_ms,
                    "p99_latency_ms": m.p99_latency_ms,
                    "cost_per_run_usd": m.cost_per_run_usd,
                    "cost_pricing_unknown": m.cost_pricing_unknown,
                    "json_invalid_count": m.json_invalid_count,
                    "repair_context_delta_pct": m.repair_context_delta_pct,
                    "repair_context_regression": m.repair_context_regression,
                    "measured_at_utc": m.measured_at_utc,
                    "gate_G2_slot_utilization": m.gates.get(
                        "G2_slot_utilization", False
                    ),
                    "gate_G2_compression_ratio": m.gates.get(
                        "G2_compression_ratio", False
                    ),
                    "gate_G3_repair_context": m.gates.get("G3_repair_context", False),
                }
            )


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------


def _resolve_fixture(fixture_relative: str) -> Path:
    p = Path(fixture_relative)
    if not p.is_absolute():
        p = REPO_ROOT / p
    return p


def parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        prog="baseline_harness.py",
        description=(
            "phase-3J-BASELINE token-efficiency benchmark harness — measures "
            "ModelCapabilityProfile tiers and writes metrics/capability_profile.toml"
        ),
    )
    p.add_argument(
        "--profile",
        default="micro_7b_12gb",
        help="tier slug from [model_capability.profiles.*], 'all' for the full roster",
    )
    p.add_argument(
        "--config",
        type=Path,
        default=DEFAULT_CONFIG,
        help=f"path to elai_cc.toml (default: {DEFAULT_CONFIG})",
    )
    p.add_argument(
        "--skip-repair-context",
        action="store_true",
        help="skip the G3 REPAIR_CONTEXT regression run (saves ~50% runtime)",
    )
    p.add_argument(
        "--skip-constrained",
        action="store_true",
        help="skip the G4 constrained-decoding json_schema fixture",
    )
    p.add_argument(
        "--skip-preflight",
        action="store_true",
        help="skip GET /health (use only when probing a known-good endpoint)",
    )
    return p.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv or sys.argv[1:])
    cfg = load_harness_config(args.config)

    if args.profile == "all":
        targets = list(cfg.default_profiles)
    elif args.profile in cfg.profiles:
        targets = [args.profile]
    else:
        print(
            f"[baseline_harness] ERROR: --profile {args.profile!r} not in "
            f"[model_capability.profiles.*] (known: "
            f"{sorted(cfg.profiles.keys())})",
            file=sys.stderr,
        )
        return EXIT_HARNESS_ERROR

    # Preflight: at least one local profile needs a healthy sidecar. If the
    # caller asked only for cloud_200k we can skip the sidecar probe.
    needs_sidecar = any(cfg.profiles[t].runtime != "cloud" for t in targets)
    if needs_sidecar and not args.skip_preflight:
        preflight_health(cfg.sidecar_endpoint)
        loaded = query_loaded_model(cfg.sidecar_endpoint)
        print(f"[baseline_harness] sidecar healthy, loaded_model={loaded!r}")

    api_key = os.environ.get("ANTHROPIC_API_KEY")
    if any(cfg.profiles[t].runtime == "cloud" for t in targets) and not api_key:
        print(
            "[baseline_harness] WARNING: cloud profile requested but "
            "ANTHROPIC_API_KEY is not set — cloud requests will fail.",
            file=sys.stderr,
        )

    measurements: list[TierMeasurement] = []
    overall_pass = True

    for tier in targets:
        profile = cfg.profiles[tier]
        fixture_path = _resolve_fixture(profile.fixture)
        print(f"[baseline_harness] tier={tier} fixture={fixture_path}")
        results, meta = run_tier(
            profile=profile,
            cfg=cfg,
            fixture_path=fixture_path,
            repair_context_active=False,
            api_key=api_key,
        )
        results_repair: list[PromptResult] | None = None
        if not args.skip_repair_context:
            results_repair, _ = run_tier(
                profile=profile,
                cfg=cfg,
                fixture_path=fixture_path,
                repair_context_active=True,
                api_key=api_key,
            )

        m = aggregate_tier(
            profile=profile,
            cfg=cfg,
            results_baseline=results,
            results_repair=results_repair,
        )
        measurements.append(m)
        all_gates = all(m.gates.values())
        overall_pass = overall_pass and all_gates
        print(
            f"[baseline_harness]   slot_utilization={m.slot_utilization:.3f} "
            f"compression_ratio_estimate={m.compression_ratio_estimate:.3f} "
            f"p99={m.p99_latency_ms:.1f}ms gates={m.gates} errors={len(m.errors)}"
        )

    # G4 constrained-decoding gate runs once over the json_schema fixture
    # against the sidecar tier (cloud handled separately if requested).
    constrained_invalid = 0
    if not args.skip_constrained and needs_sidecar:
        constrained_path = _resolve_fixture(cfg.constrained_fixture)
        # Use the first sidecar target as the model carrier for the gate.
        carrier_tier = next(
            (t for t in targets if cfg.profiles[t].runtime != "cloud"), None
        )
        if carrier_tier is not None:
            carrier = cfg.profiles[carrier_tier]
            print(
                f"[baseline_harness] G4 constrained-decoding fixture="
                f"{constrained_path} carrier_tier={carrier_tier}"
            )
            constrained_results, _ = run_tier(
                profile=carrier,
                cfg=cfg,
                fixture_path=constrained_path,
                repair_context_active=False,
                api_key=None,
            )
            constrained_invalid = sum(1 for r in constrained_results if r.json_invalid)
            constrained_pass = constrained_invalid <= cfg.constrained_invalid_allowed
            overall_pass = overall_pass and constrained_pass
            print(
                f"[baseline_harness]   constrained invalid={constrained_invalid} "
                f"allowed={cfg.constrained_invalid_allowed} pass={constrained_pass}"
            )

    write_capability_profile_toml(cfg, measurements, constrained_invalid)
    write_capability_profile_csv(cfg, measurements)
    print(
        f"[baseline_harness] wrote {cfg.output_toml} and {cfg.output_csv}; "
        f"overall_pass={overall_pass}"
    )
    return EXIT_OK if overall_pass else EXIT_GATE_FAILED


if __name__ == "__main__":
    raise SystemExit(main())
