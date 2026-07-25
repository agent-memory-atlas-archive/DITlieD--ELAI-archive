"""H-50 STEP 3 -- sidecar-side context-reduction journal + measurement layer.

The sidecar is where the model call actually happens and where input tokens
are measured (the engine tokenizes the prompt in ``_generate_impl`` and emits
``usage.input_tokens``). H-50 STEP 3 requires a *measured off-agent proof*
that the model call after a local trim carries FEWER input tokens than
before, that a ``ContextReduction`` row lands, and that the session that
previously hit ``context_window_blocked`` now completes.

This module is the sidecar's contribution to that proof -- the MEASUREMENT
and JOURNAL layer (not the trim executor). The trim itself is applied
client-side by the orchestrator/CLI (``live_cli.rs prepare_turn_runtime`` +
``conversation.rs maybe_auto_compact``); the sidecar only receives the
already-trimmed prompt and records what it measured. The journal is the
off-agent artifact the verifier reads ("from the journal").

Design mirrors the plan's regime split (one window truth feeding
window-derived trim thresholds, LOCAL unload/load vs CLOUD cache-stable):

  * ``effective_context_window(regime, ...)`` -- the ONE window truth.
  * ``trim_at(window, regime)``            -- ``floor(window * fraction)``.
  * ``apply_local_trim(messages, tokenize, window, ...)`` -- the
    cheapest-first LOCAL ladder (drop dead trailing tool-result observations
    -> mask/stub bulky tool-results -> summarize oldest text turns). Used by
    the proof harness AND by the journal's optional would-reduce annotation.
  * ``ContextReductionRecord``             -- the ``ContextReduction`` row
    payload (regime, window, trim_at, tokens_before, tokens_after, dropped,
    masked, summarized, request_id, ts).
  * ``ContextJournal``                     -- append-only JSONL journal,
    env-gated ``ELAI_CONTEXT_JOURNAL_ENABLE`` (default OFF, kill-switch W-2).
  * ``maybe_journal_turn``                 -- the real-caller hook wired into
    ``server.handle_messages`` (the live /v1/messages entrypoint).

Grounding: H-50 plan (docs/thefinalglance/H-plan/H-50_regime_aware_context_management.md);
Anthropic effective-context-engineering (Sep 2025) -- policy must match the
window; cheapest-first reduction (drop -> mask -> summarize). The trim
fractions (LOCAL 0.72, CLOUD 0.85) are [DESIGN] per the plan's "~0.70-0.75"
/ "~0.85" ranges, re-measured per HD-1 by the proof harness.
"""

from __future__ import annotations

import os
import time
from dataclasses import asdict, dataclass, field
from threading import Lock
from typing import Any, Callable, Dict, List, Optional, Tuple

import orjson
import structlog

try:
    from elai_inference.posthog_client import capture as _ph_capture
    _PH_AVAILABLE = True
except ImportError:
    def _ph_capture(event: str, properties=None) -> None:  # type: ignore[misc]
        pass
    _PH_AVAILABLE = False

logger = structlog.get_logger("elai_inference.context_journal")

# ---------------------------------------------------------------------------
# Regime + window truth
# ---------------------------------------------------------------------------

# [DESIGN] per H-50 plan ("~0.70-0.75" local, "~0.85" cloud). Re-measured by
# the proof harness (proof_context_journal.py) against the real tokenizer path.
TRIM_FRACTION_LOCAL = 0.72
TRIM_FRACTION_CLOUD = 0.85

# Default ceilings when the caller does not supply an explicit value. The
# live path reads ELAI_LOCAL_MAX_CONTEXT (the BudgetPolicy::Local ceiling)
# from the Rust client; these are fallbacks for the sidecar standing alone.
DEFAULT_LOCAL_MAX_CONTEXT = 32_000
DEFAULT_CLOUD_CEILING = 200_000

# Kill-switches (W-2). Journal is default-OFF; the would-reduce annotation
# (which runs the ladder on every oversized turn) is a second, more-expensive
# opt-in so the default-enabled path pays no extra tokenization cost.
_ENV_JOURNAL_ENABLE = "ELAI_CONTEXT_JOURNAL_ENABLE"
_ENV_JOURNAL_PATH = "ELAI_CONTEXT_JOURNAL_PATH"
_ENV_JOURNAL_PROOF = "ELAI_CONTEXT_JOURNAL_PROOF"
_ENV_REGIME_OVERRIDE = "ELAI_CONTEXT_REGIME"
_ENV_LOCAL_MAX_CONTEXT = "ELAI_LOCAL_MAX_CONTEXT"
_DEFAULT_JOURNAL_PATH = ".elai_context_journal.jsonl"


class BackendMode:
    """Regime selector. The sidecar derives this from the request ``model``
    field (local model ids carry "local") or the ``ELAI_CONTEXT_REGIME`` env
    override. Mirrors the Rust ``BackendMode`` used by ``ContextPolicy``."""

    LOCAL = "local"
    CLOUD = "cloud"


def effective_context_window(
    regime: str,
    local_max_context: Optional[int] = None,
    cloud_ceiling: Optional[int] = None,
) -> int:
    """The ONE window truth feeding all trim thresholds.

    LOCAL = the BudgetPolicy::Local ceiling (``ELAI_LOCAL_MAX_CONTEXT``);
    CLOUD = the cloud model's real ceiling. Both are read once and feed
    ``trim_at`` so the flat 100_000 / 64K-non-opus gates are replaced by a
    single window-derived value (H-50 plan, REVISED scope Tier-1)."""

    if regime == BackendMode.LOCAL:
        if local_max_context is not None:
            return int(local_max_context)
        env_val = os.environ.get(_ENV_LOCAL_MAX_CONTEXT)
        if env_val and env_val.strip().isdigit():
            return int(env_val)
        return DEFAULT_LOCAL_MAX_CONTEXT
    # CLOUD
    if cloud_ceiling is not None:
        return int(cloud_ceiling)
    return DEFAULT_CLOUD_CEILING


def trim_at(window: int, regime: str) -> int:
    """``floor(window * TRIM_FRACTION)``. LOCAL trims earlier (no cache to
    amortize, small window); CLOUD trims late (keep the cache-stable prefix)."""

    fraction = TRIM_FRACTION_LOCAL if regime == BackendMode.LOCAL else TRIM_FRACTION_CLOUD
    return int(window * fraction)


def derive_regime(model_name: str) -> str:
    """Derive the regime from the request ``model`` field or the
    ``ELAI_CONTEXT_REGIME`` env override. Local model ids carry "local"."""

    override = os.environ.get(_ENV_REGIME_OVERRIDE)
    if override and override.strip().lower() in (BackendMode.LOCAL, BackendMode.CLOUD):
        return override.strip().lower()
    if model_name and "local" in model_name.lower():
        return BackendMode.LOCAL
    return BackendMode.CLOUD


# ---------------------------------------------------------------------------
# ContextReduction record (the "ContextReduction row" payload)
# ---------------------------------------------------------------------------


@dataclass
class ContextReductionRecord:
    """The ``ContextReduction`` row. Reused payload (GD-1, no new SSB
    variant) -- the orchestrator emits the canonical SSB row; this is the
    sidecar-measured mirror that lands in the JSONL journal."""

    request_id: str
    regime: str
    window: int
    trim_at: int
    tokens_before: int
    tokens_after: int
    dropped_turns: int = 0
    masked_turns: int = 0
    summarized_turns: int = 0
    completed: bool = False
    ts: float = field(default_factory=time.time)

    @property
    def reduced_by(self) -> int:
        return self.tokens_before - self.tokens_after

    @property
    def reduced_pct(self) -> float:
        if self.tokens_before <= 0:
            return 0.0
        return (self.reduced_by / self.tokens_before) * 100.0

    def to_dict(self) -> Dict[str, Any]:
        d = asdict(self)
        d["reduced_by"] = self.reduced_by
        d["reduced_pct"] = round(self.reduced_pct, 4)
        d["event"] = "ContextReduction"
        return d


# ---------------------------------------------------------------------------
# Cheapest-first LOCAL trim ladder (drop -> mask -> summarize)
# ---------------------------------------------------------------------------

# A tool_result block is "bulky" above this many tokens; masking replaces its
# content with a short stub (H-6 deterministic masking, routed into the
# interactive local turn by H-50). [DESIGN] -- re-measured by the harness.
_BULKY_TOOL_RESULT_TOKENS = 256
_MASK_STUB = "[tool_result masked: {n} tokens dropped, reload on demand]"
# Oldest text turns are summarized to one short line (compact_session the
# oldest turns, H-50 plan). The summary preserves the role + a digest of the
# content length so the model retains a breadcrumb (RV-2 lossless-view stub).
_SUMMARY_STUB = "[prior {role} turn summarized: {n} tokens]"


def _content_text(content: Any) -> str:
    """Flatten Anthropic-style content (str | list[block]) to text."""

    if isinstance(content, str):
        return content
    if isinstance(content, list):
        parts: List[str] = []
        for block in content:
            if isinstance(block, dict):
                if block.get("type") == "text":
                    parts.append(str(block.get("text", "")))
                elif block.get("type") == "tool_result":
                    inner = block.get("content", "")
                    parts.append(_content_text(inner))
        return " ".join(parts)
    return str(content)


def _set_content_text(content: Any, new_text: str) -> Any:
    """Replace the text of a content block in-place, preserving shape."""

    if isinstance(content, str):
        return new_text
    if isinstance(content, list):
        for block in content:
            if isinstance(block, dict) and block.get("type") == "text":
                block["text"] = new_text
                return content
            if isinstance(block, dict) and block.get("type") == "tool_result":
                block["content"] = new_text
                return content
        # No text block found -- append one
        content.append({"type": "text", "text": new_text})
        return content
    return new_text


def _count_messages_tokens(
    messages: List[Dict[str, Any]],
    tokenize: Callable[[str], int],
) -> int:
    """Count tokens across a message list using the supplied tokenizer."""

    total = 0
    for msg in messages:
        if msg is None:
            continue
        total += tokenize(_content_text(msg.get("content", "")))
        # role tag overhead (~1 token) -- conservative
        total += 1
    return total


def _is_tool_result(msg: Dict[str, Any]) -> bool:
    content = msg.get("content")
    if isinstance(content, list):
        return any(
            isinstance(b, dict) and b.get("type") == "tool_result"
            for b in content
        )
    return False


def apply_local_trim(
    messages: List[Dict[str, Any]],
    tokenize: Callable[[str], int],
    window: int,
    regime: str = BackendMode.LOCAL,
    trim_fraction: Optional[float] = None,
    request_id: str = "proof",
    max_output_tokens: int = 1024,
) -> Tuple[List[Dict[str, Any]], ContextReductionRecord]:
    """Cheapest-first LOCAL trim ladder (H-50 plan Tier-1):

      1. DROP dead trailing tool-result observations (oldest first -- least
         likely to be re-needed; the most recent tool_result is kept so the
         model retains the live observation it just acted on).
      2. MASK bulky tool-result content -> short stub (H-6 deterministic
         masking). The stub records how many tokens were dropped so the model
         can ask to reload selectively (recover_stubs becomes SELECTIVE).
      3. SUMMARIZE oldest user/assistant text turns -> one-line breadcrumb
         (compact_session the oldest turns, RV-2 lossless-view).

    Iterates until ``tokens_after <= trim_at`` or no further reduction is
    possible. Returns the trimmed messages (a NEW list; the input is not
    mutated) and a ``ContextReductionRecord``. ``completed`` is True when the
    trimmed prompt + ``max_output_tokens`` fits the window (the session that
    previously hit ``context_window_blocked`` now completes)."""

    if trim_fraction is None:
        trim_fraction = TRIM_FRACTION_LOCAL if regime == BackendMode.LOCAL else TRIM_FRACTION_CLOUD
    target = int(window * trim_fraction)

    # Deep-ish copy: clone message dicts + content lists so the caller's
    # messages are not mutated.
    trimmed: List[Dict[str, Any]] = []
    for msg in messages:
        content = msg.get("content")
        if isinstance(content, list):
            new_content = []
            for b in content:
                if isinstance(b, dict):
                    nb = dict(b)
                    if isinstance(b.get("content"), list):
                        nb["content"] = list(b["content"])
                    new_content.append(nb)
                else:
                    new_content.append(b)
            trimmed.append({**msg, "content": new_content})
        else:
            trimmed.append(dict(msg))

    tokens_before = _count_messages_tokens(messages, tokenize)
    dropped = 0
    masked = 0
    summarized = 0

    def _now_tokens() -> int:
        return _count_messages_tokens(trimmed, tokenize)

    # Step 1: DROP oldest tool_result observations, keeping the most recent.
    tool_result_idx = [i for i, m in enumerate(trimmed) if _is_tool_result(m)]
    # Keep the last tool_result (most recent live observation); drop the rest
    # oldest-first until under target.
    for i in tool_result_idx[:-1]:
        if _now_tokens() <= target:
            break
        trimmed[i] = None  # type: ignore[assignment]
        dropped += 1
    trimmed = [m for m in trimmed if m is not None]

    # Step 2: MASK bulky tool_result content still present.
    for i, m in enumerate(trimmed):
        if _now_tokens() <= target:
            break
        if not _is_tool_result(m):
            continue
        text = _content_text(m.get("content", ""))
        n = tokenize(text)
        if n > _BULKY_TOOL_RESULT_TOKENS:
            stub = _MASK_STUB.format(n=n)
            trimmed[i]["content"] = _set_content_text(m.get("content", ""), stub)
            masked += 1

    # Step 3: SUMMARIZE oldest user/assistant text turns (not tool_results).
    text_idx = [
        i for i, m in enumerate(trimmed)
        if not _is_tool_result(m) and m.get("role") in ("user", "assistant")
    ]
    for i in text_idx:
        if _now_tokens() <= target:
            break
        m = trimmed[i]
        text = _content_text(m.get("content", ""))
        n = tokenize(text)
        if n <= 0:
            continue
        stub = _SUMMARY_STUB.format(role=m.get("role", "turn"), n=n)
        trimmed[i]["content"] = _set_content_text(m.get("content", ""), stub)
        summarized += 1

    tokens_after = _now_tokens()
    completed = (tokens_after + max_output_tokens) <= window
    record = ContextReductionRecord(
        request_id=request_id,
        regime=regime,
        window=window,
        trim_at=target,
        tokens_before=tokens_before,
        tokens_after=tokens_after,
        dropped_turns=dropped,
        masked_turns=masked,
        summarized_turns=summarized,
        completed=completed,
    )
    return trimmed, record


# ---------------------------------------------------------------------------
# JSONL journal (env-gated, default OFF -- kill-switch W-2)
# ---------------------------------------------------------------------------


class ContextJournal:
    """Append-only JSONL journal of context-reduction + per-turn measurement
    rows. The off-agent verifier reads this file ("from the journal").

    Gated by ``ELAI_CONTEXT_JOURNAL_ENABLE`` (default OFF). Path from
    ``ELAI_CONTEXT_JOURNAL_PATH`` (default ``.elai_context_journal.jsonl``).
    Thread-safe (the sidecar serves concurrent requests)."""

    def __init__(self, path: Optional[str] = None) -> None:
        self._enabled = os.environ.get(_ENV_JOURNAL_ENABLE, "").strip().lower() in (
            "1", "true", "yes", "on",
        )
        if path is not None:
            self._path = path
        else:
            self._path = os.environ.get(_ENV_JOURNAL_PATH, _DEFAULT_JOURNAL_PATH)
        self._lock = Lock()

    @property
    def enabled(self) -> bool:
        return self._enabled

    @property
    def path(self) -> str:
        return self._path

    def append(self, row: Dict[str, Any]) -> None:
        if not self._enabled:
            return
        try:
            line = orjson.dumps(row, default=str)
            with self._lock, open(self._path, "ab") as fh:
                fh.write(line)
                fh.write(b"\n")
        except OSError as exc:
            # Journal is an observation layer -- never fail the request on it.
            logger.warning("context_journal_write_failed", error=str(exc))


# Module-level singleton used by the live path.
_JOURNAL = ContextJournal()


# ---------------------------------------------------------------------------
# Real-caller hook (wired into server.handle_messages)
# ---------------------------------------------------------------------------


def maybe_journal_turn(
    prompt: str,
    messages: List[Dict[str, Any]],
    model_name: str,
    tokenizer: Any,
    request_id: Optional[str] = None,
    max_output_tokens: int = 1024,
) -> Optional[ContextReductionRecord]:
    """Measurement hook called from the live /v1/messages entrypoint.

    Records one per-turn row to the journal with the measured
    ``input_tokens`` + regime + window + trim_at + ``exceeds_trim_at``. When
    the turn exceeds ``trim_at`` AND ``ELAI_CONTEXT_JOURNAL_PROOF=1``, also
    runs ``apply_local_trim`` to record the would-reduce ``ContextReduction``
    row (the off-agent artifact). The prompt sent to the model is NOT
    modified -- the trim is the orchestrator/CLI's job (H-50 plan); the
    sidecar only MEASURES and JOURNALS.

    Returns the ``ContextReductionRecord`` when a would-reduce was computed,
    else ``None``. No-op (returns ``None``) when the journal is disabled."""

    if not _JOURNAL.enabled:
        return None
    regime = derive_regime(model_name)
    window = effective_context_window(regime)
    target = trim_at(window, regime)

    # Tokenize the built prompt the same way the engine does.
    try:
        input_tokens = _tokenizer_count(tokenizer, prompt)
    except Exception as exc:  # pragma: no cover -- defensive on the live path
        logger.warning("context_journal_tokenize_failed", error=str(exc))
        return None

    rid = request_id or f"turn-{int(time.time() * 1000)}"
    turn_row: Dict[str, Any] = {
        "event": "TurnMeasured",
        "request_id": rid,
        "regime": regime,
        "window": window,
        "trim_at": target,
        "input_tokens": input_tokens,
        "exceeds_trim_at": input_tokens > target,
        "model": model_name,
        "messages": len(messages),
        "ts": time.time(),
    }
    _JOURNAL.append(turn_row)
    _ph_capture("context_journal_recorded", {
        "regime": regime,
        "input_tokens": input_tokens,
        "trim_at": target,
        "exceeds_trim_at": input_tokens > target,
        "message_count": len(messages),
    })

    # Optional would-reduce annotation (second, more-expensive kill-switch).
    proof = os.environ.get(_ENV_JOURNAL_PROOF, "").strip().lower() in (
        "1", "true", "yes", "on",
    )
    if not proof or input_tokens <= target:
        return None

    def tokenize(text: str) -> int:
        return _tokenizer_count(tokenizer, text)
    _, record = apply_local_trim(
        messages, tokenize, window, regime=regime,
        request_id=rid, max_output_tokens=max_output_tokens,
    )
    _JOURNAL.append(record.to_dict())
    logger.info(
        "context_reduction_journaled",
        request_id=rid,
        regime=regime,
        tokens_before=record.tokens_before,
        tokens_after=record.tokens_after,
        reduced_by=record.reduced_by,
        completed=record.completed,
    )
    return record


def _tokenizer_count(tokenizer: Any, text: str) -> int:
    """Count tokens using the real engine tokenizer (or any HF tokenizer)."""

    enc = tokenizer(text, return_tensors=None)
    ids = enc.get("input_ids") if isinstance(enc, dict) else getattr(enc, "input_ids", None)
    if ids is None:
        return 0
    if hasattr(ids, "__len__") and not isinstance(ids, (list, tuple)):
        # tensor-like -- take len of first dim
        try:
            return int(len(ids))
        except TypeError:
            return 0
    return len(ids)
