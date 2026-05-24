"""C-37 Actor / Evaluator / Reflector role implementations.

Paper Section 3.3 (verbatim mapping):
  Actor (M_a):      policy function; generates a reasoning Trajectory
                    pi_Theta(y|x) given input context. Decoding free-text.
  Evaluator (M_e):  semantic-equivalence judge; binary verdict only.
                    Decoding free-text.
  Reflector (M_r):  root-cause diagnoser; constrained via Outlines FSM
                    over the pydantic schema in types.ReflectorOutput.

Paper Section 4.3 (verbatim decoding params):
  All roles: temperature=0.1, max_new_tokens=1024
  Reflector additionally: constrained=Outlines + ReflectorOutput pydantic

Model: Qwen3-8B per paper Section 4.3 ("a uniform model for both the Actor
and Evaluator roles prevents capability mismatches and mitigates intra-
model scoring bias"). ELAI's remote sidecar default is
Qwen2.5-Coder-7B-Instruct-abliterated; using that instead of Qwen3-8B is a
documented deviation — the structure-snowballing finding MAY transfer but
the 0.38 / 96% numbers are NOT portable per the source interface map.

Sidecar transport: each role calls the Anthropic-protocol endpoint at
http://127.0.0.1:8100/v1/messages (ELAI sidecar). The Reflector additionally
attaches a response_format=json_object hint AND a separate Outlines pass
when running locally with HF transformers — full Outlines wiring is a
RUNPOD-PENDING step because it requires the model loaded in-process with
a logit-processor hook (not the streaming HTTP path).
"""

from __future__ import annotations

import json
import time
import urllib.error
import urllib.request
from dataclasses import dataclass
from typing import Any, Optional

from elai_inference.c37_snowballing.types import (
    ERROR_TYPES,
    ReflectorOutput,
)


# Paper Section 4.3 — uniform decoding hyperparameters.
PAPER_TEMPERATURE: float = 0.1
PAPER_MAX_NEW_TOKENS: int = 1024
PAPER_MODEL_DEFAULT: str = "Qwen/Qwen3-8B"

# Default sidecar endpoint matches CLAUDE.md remote-sidecar block + the
# local tunnel forwarder at workflow/remote-setup/start-tunnel.ps1.
DEFAULT_SIDECAR_URL: str = "http://127.0.0.1:8100"


@dataclass(frozen=True)
class RoleConfig:
    """Per-role decoding configuration.

    Paper-grounded values for every role: temperature=0.1, max_new_tokens=1024.
    `constrained` is True only for the Reflector (paper Section 3.3).
    """

    role: str  # one of: "actor", "evaluator", "reflector"
    model: str
    temperature: float
    max_new_tokens: int
    constrained: bool


def paper_role_config(role: str, model: str = PAPER_MODEL_DEFAULT) -> RoleConfig:
    """Construct a paper-grounded RoleConfig for the given role.

    All roles use the same temperature + max_new_tokens; only the Reflector
    flips `constrained=True`.
    """
    if role not in ("actor", "evaluator", "reflector"):
        raise ValueError(
            f"role must be one of actor/evaluator/reflector, got {role!r}"
        )
    return RoleConfig(
        role=role,
        model=model,
        temperature=PAPER_TEMPERATURE,
        max_new_tokens=PAPER_MAX_NEW_TOKENS,
        constrained=(role == "reflector"),
    )


def _sidecar_messages_request(
    sidecar_url: str,
    model: str,
    system: str,
    user: str,
    temperature: float,
    max_new_tokens: int,
    response_format: Optional[dict[str, Any]] = None,
    timeout_s: float = 120.0,
) -> str:
    """POST to /v1/messages (ELAI Anthropic-protocol sidecar) and return
    the assistant content as a string.

    No retries: caller's max_trials loop handles trial-level retries; this
    function performs exactly one round-trip per call so timing measurements
    in the JSONL output remain attributable.

    Raises urllib.error.URLError on transport failure (caller handles).
    """
    body: dict[str, Any] = {
        "model": model,
        "max_tokens": max_new_tokens,
        "temperature": temperature,
        "system": system,
        "messages": [{"role": "user", "content": user}],
    }
    if response_format is not None:
        # The ELAI sidecar accepts the response_format hint at
        # server.py:1564-1641 (per source interface map). Reflector calls
        # pass this to opt into json_object enforcement; non-Reflector
        # callers omit it for free-text decoding.
        body["response_format"] = response_format

    payload = json.dumps(body).encode("utf-8")
    req = urllib.request.Request(
        f"{sidecar_url.rstrip('/')}/v1/messages",
        data=payload,
        method="POST",
        headers={
            "Content-Type": "application/json",
            "anthropic-version": "2023-06-01",
        },
    )
    with urllib.request.urlopen(req, timeout=timeout_s) as resp:
        raw = resp.read().decode("utf-8")
    parsed = json.loads(raw)
    # Anthropic envelope: {content: [{type: "text", text: "..."}], ...}
    if "content" in parsed and isinstance(parsed["content"], list):
        chunks = [
            c.get("text", "")
            for c in parsed["content"]
            if isinstance(c, dict) and c.get("type") == "text"
        ]
        return "".join(chunks)
    # Fallback path: some sidecar builds expose a flat `text` field.
    if "text" in parsed and isinstance(parsed["text"], str):
        return parsed["text"]
    raise ValueError(
        f"Sidecar response missing content/text fields: keys={list(parsed.keys())}"
    )


class Actor:
    """Reasoning-step generator. Free-text decoding (paper Section 3.3)."""

    def __init__(self, config: RoleConfig, sidecar_url: str = DEFAULT_SIDECAR_URL):
        if config.role != "actor":
            raise ValueError(f"Actor requires role=='actor', got {config.role!r}")
        if config.constrained:
            raise ValueError("Actor must run unconstrained (paper Section 3.3)")
        self.config = config
        self.sidecar_url = sidecar_url

    def generate(
        self,
        question: str,
        contexts: list[str],
        episodic_memory: list[ReflectorOutput] | None = None,
    ) -> str:
        """Generate an answer trajectory string given question + contexts.

        Episodic memory (paper Section 3.3) is the accumulated list of
        correction rules from prior failed trials; it is prepended to the
        Actor prompt verbatim. None or empty list means trial 1.
        """
        context_block = "\n\n".join(contexts)
        memory_block = ""
        if episodic_memory:
            rules = "\n".join(
                f"- [{m.error_type}] {m.correction_rule}" for m in episodic_memory
            )
            memory_block = (
                "Prior failed attempts produced these correction rules; "
                "apply them this trial:\n" + rules + "\n\n"
            )
        system = (
            "You are a multi-hop reasoning agent. Read the provided paragraphs, "
            "reason step by step, and produce a final answer in the LAST line "
            "as exactly: Answer: <your answer>"
        )
        user = (
            f"{memory_block}Question: {question}\n\n"
            f"Paragraphs:\n{context_block}\n\n"
            "Reason step by step, then output: Answer: <answer>"
        )
        return _sidecar_messages_request(
            self.sidecar_url,
            model=self.config.model,
            system=system,
            user=user,
            temperature=self.config.temperature,
            max_new_tokens=self.config.max_new_tokens,
        )


class Evaluator:
    """Semantic-equivalence judge. Free-text decoding (paper Section 4.3:
    "strictly limit the evaluator to verifying semantic equivalence,
    avoiding subjective quality ranking").
    """

    def __init__(self, config: RoleConfig, sidecar_url: str = DEFAULT_SIDECAR_URL):
        if config.role != "evaluator":
            raise ValueError(f"Evaluator requires role=='evaluator', got {config.role!r}")
        if config.constrained:
            raise ValueError("Evaluator must run unconstrained (paper Section 4.3)")
        self.config = config
        self.sidecar_url = sidecar_url

    def evaluate(self, question: str, gold: str, predicted: str) -> bool:
        """Return True iff `predicted` is semantically equivalent to `gold`.

        Paper Section 4.3 bounds this to entailment-only — no quality
        ranking, no partial credit, no preference comparison.
        """
        system = (
            "You are a strict semantic-equivalence judge. Compare a predicted "
            "answer against the gold answer for the same question. Output a "
            "single token on the LAST line: YES if the predicted answer is "
            "semantically equivalent to the gold (same entity, same number, "
            "same date, formatting differences allowed); NO otherwise."
        )
        user = (
            f"Question: {question}\n"
            f"Gold answer: {gold}\n"
            f"Predicted answer: {predicted}\n\n"
            "Output the verdict (YES/NO) on the last line."
        )
        text = _sidecar_messages_request(
            self.sidecar_url,
            model=self.config.model,
            system=system,
            user=user,
            temperature=self.config.temperature,
            max_new_tokens=self.config.max_new_tokens,
        )
        # Parse last non-empty line; case-insensitive match for YES.
        for line in reversed(text.strip().splitlines()):
            stripped = line.strip().upper()
            if stripped in ("YES", "NO"):
                return stripped == "YES"
            if stripped.startswith("YES"):
                return True
            if stripped.startswith("NO"):
                return False
        # No clear verdict — treat as NO (conservative: forces another trial).
        return False


class Reflector:
    """Root-cause diagnoser. Constrained decoding (paper Section 3.3:
    Outlines FSM over the 5-class ReflectorOutput pydantic schema).

    Implementation note on `constrained_mode`:
      - "schema_hint": passes response_format={"type":"json_schema"} to
        the sidecar. Sidecar applies lm-format-enforcer (per
        elai_cc.toml:998-1004) if available; otherwise falls back to
        json_object hint with a regex post-parse here. This is the
        sidecar-compatible path that works against the live ELAI rig.
      - "outlines": signals a future in-process Outlines path. RUNPOD-
        PENDING because Outlines requires the model loaded with a logit
        processor (HF transformers generate() path), not the streaming
        HTTP endpoint the sidecar exposes today.
    """

    def __init__(self, config: RoleConfig, sidecar_url: str = DEFAULT_SIDECAR_URL):
        if config.role != "reflector":
            raise ValueError(f"Reflector requires role=='reflector', got {config.role!r}")
        if not config.constrained:
            raise ValueError(
                "Reflector must run constrained (paper Section 3.3 — "
                "this is the structure-snowballing trigger condition)"
            )
        self.config = config
        self.sidecar_url = sidecar_url

    def diagnose(
        self,
        trial_outputs: list[str],
        evaluator_verdicts: list[bool],
        constrained_mode: str = "schema_hint",
    ) -> ReflectorOutput:
        """Diagnose the FIRST upstream error in the failed trajectory.

        Paper Section 3.2 (upstream-first attribution): the reflector MUST
        identify only the FIRST fatal error in the trajectory, even if
        downstream errors are also present. Cite: BIG-Bench Mistake (Tyen
        et al. 2024).

        Returns a ReflectorOutput with error_type in the 5-class enum and
        a non-empty correction_rule. Raises pydantic ValidationError if
        the underlying generation produced an unparseable result.
        """
        if constrained_mode not in ("schema_hint", "outlines"):
            raise ValueError(
                f"constrained_mode must be schema_hint or outlines, got {constrained_mode!r}"
            )

        trajectory_block = "\n".join(
            f"  Trial {i + 1}: {out!r} (evaluator={'PASS' if v else 'FAIL'})"
            for i, (out, v) in enumerate(zip(trial_outputs, evaluator_verdicts))
        )
        enum_block = ", ".join(ERROR_TYPES)
        system = (
            "You are a reflection agent diagnosing the FIRST upstream error in a "
            "failed multi-hop reasoning trajectory. Classify exactly one error "
            f"type from this 5-class enum: {enum_block}. Then propose a single "
            "correction rule (free text, one sentence) the actor should apply "
            "on the next trial. Output ONLY valid JSON matching the schema "
            '{"error_type": "<one-of-5>", "correction_rule": "<text>"}. No '
            "prose outside the JSON."
        )
        user = (
            "Failed trajectory:\n"
            f"{trajectory_block}\n\n"
            "Identify the FIRST upstream error and emit the JSON object."
        )

        response_format: Optional[dict[str, Any]]
        if constrained_mode == "schema_hint":
            response_format = {
                "type": "json_schema",
                "schema": ReflectorOutput.model_json_schema(),
            }
        else:
            # "outlines" path — RUNPOD-PENDING. Falls back to schema_hint
            # transport here so the harness still produces an output; the
            # caller is expected to swap in a real Outlines-bound generator
            # when running the full paper replication.
            response_format = {"type": "json_object"}

        text = _sidecar_messages_request(
            self.sidecar_url,
            model=self.config.model,
            system=system,
            user=user,
            temperature=self.config.temperature,
            max_new_tokens=self.config.max_new_tokens,
            response_format=response_format,
        )
        # Locate the first balanced JSON object in the response.
        decoded = _extract_json_object(text)
        if decoded is None:
            raise ValueError(
                f"Reflector response did not contain a JSON object: {text[:200]!r}"
            )
        # Pydantic-validate against the schema; raises ValidationError on
        # 5-class violation or empty correction_rule.
        return ReflectorOutput.model_validate(decoded)


def _extract_json_object(text: str) -> dict[str, Any] | None:
    """Return the first top-level JSON object embedded in `text`.

    Used by Reflector.diagnose to tolerate sidecar responses that wrap the
    JSON in prose (e.g. "Here is the diagnosis: { ... }"). Returns None
    when no balanced object is found.

    No regex: uses brace-counting so nested objects do not confuse the
    parser. Strings containing unescaped braces are NOT a concern here
    because the FSM / schema_hint constrains the response shape.
    """
    depth = 0
    start: int | None = None
    for i, ch in enumerate(text):
        if ch == "{":
            if depth == 0:
                start = i
            depth += 1
        elif ch == "}":
            if depth == 0:
                continue
            depth -= 1
            if depth == 0 and start is not None:
                candidate = text[start : i + 1]
                try:
                    parsed = json.loads(candidate)
                    if isinstance(parsed, dict):
                        return parsed
                except json.JSONDecodeError:
                    start = None
                    continue
    return None
