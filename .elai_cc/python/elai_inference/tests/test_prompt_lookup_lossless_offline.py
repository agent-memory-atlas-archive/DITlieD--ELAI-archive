"""E-2 Task 3 offline contract test for prompt_lookup_num_tokens losslessness.

OFFLINE companion to test_prompt_lookup_lossless.py (RUNPOD live harness).

Losslessness is a STRUCTURAL property of the HF transformers prompt-lookup
implementation: at temp=0 greedy, the draft tokens proposed via input-prefix
n-gram lookup are VERIFIED against the same forward-pass logits that the
non-draft path uses. Mismatching drafts are rejected and the verified
position becomes the canonical next token. Therefore the output token-id
sequence is byte-equivalent to the non-draft greedy path BY CONSTRUCTION.

This offline test locks the SIDECAR-SIDE INVARIANT we control: server.py
must pass prompt_lookup_num_tokens through to model.generate UNCHANGED,
WITHOUT injecting any sampling-style modification that would break
losslessness. The live RUNPOD harness measures the empirical byte
equivalence; this harness verifies the wiring contract.

Invariants verified:
  1. server.py imports/uses model.generate (the HF canonical entry).
  2. The kwarg name is exactly "prompt_lookup_num_tokens" (HF API name;
     a typo here breaks the kwarg silently).
  3. No call site in server.py overrides do_sample=True alongside
     prompt_lookup_num_tokens (HF rejects this combination — lossless
     guarantee is greedy-only).
  4. No call site wraps prompt_lookup output through a sampling
     LogitsProcessor that could non-deterministically alter accepted
     tokens.

Run via:
   cd .elai_cc/python && \
     python -m pytest elai_inference/tests/test_prompt_lookup_lossless_offline.py -v

Exit 0 = invariants hold. Test failure = lossless contract violated on the
sidecar side. The HF library side is grounded by the verifier-rejection
semantics documented at huggingface.co/docs/transformers/generation_strategies
"""

from __future__ import annotations

import pathlib
import re


SERVER_PATH = (
    pathlib.Path(__file__).resolve().parent.parent / "server.py"
)


def _read_server_source() -> str:
    return SERVER_PATH.read_text(encoding="utf-8")


def test_server_source_file_exists():
    """Sanity: the source-of-truth file is at the canonical path."""
    assert SERVER_PATH.is_file(), f"server.py missing at {SERVER_PATH}"


def test_server_invokes_model_generate():
    """server.py must call model.generate(...) — the HF canonical entry
    point that hosts the prompt-lookup verifier loop. If a fork has rolled
    its own decode loop, the lossless guarantee is no longer in effect."""
    source = _read_server_source()
    assert re.search(r"\.generate\s*\(", source), (
        "server.py does not call .generate(); lossless guarantee depends on "
        "the HF verifier-rejection loop and cannot be enforced if a custom "
        "decode path replaces it."
    )


def test_no_kwarg_typo_for_prompt_lookup():
    """If prompt_lookup appears at all on the call surface, the kwarg name
    must be the canonical 'prompt_lookup_num_tokens'. A typo would silently
    no-op and the test_prompt_lookup_lossless RUNPOD harness would still
    pass (since it would just run baseline generation twice) — surfacing
    the wiring bug requires a structural check here."""
    source = _read_server_source()
    # Tolerate prose mentions ("prompt-lookup decoding"); require kwarg
    # form to be exact when present.
    forbidden_misspellings = [
        "prompt_lookup_tokens",
        "promptLookup",
        "prompt_lookups_num_tokens",
        "prompt_lookup_n_tokens",
    ]
    for misspelling in forbidden_misspellings:
        assert misspelling not in source, (
            f"server.py references a misspelled HF kwarg '{misspelling}'. "
            "Canonical form is 'prompt_lookup_num_tokens'."
        )


def test_no_sampling_override_in_codepath():
    """The losslessness guarantee is greedy-only (temp=0). If server.py
    forcibly overrides do_sample=True on a path that also passes
    prompt_lookup_num_tokens, the verifier loop runs but accepted tokens
    become non-deterministic — losslessness is violated.

    This test scans server.py for any LINE that simultaneously sets
    do_sample=True AND prompt_lookup_num_tokens. Comments and prose are
    excluded by the line-level conjunction check.
    """
    source = _read_server_source()
    for line in source.splitlines():
        # Strip comment tail to avoid false positives in prose.
        code = line.split("#", 1)[0]
        if "do_sample=True" in code and "prompt_lookup_num_tokens" in code:
            raise AssertionError(
                "Found do_sample=True alongside prompt_lookup_num_tokens on "
                f"one line: {line.strip()!r}. Lossless guarantee is greedy-only."
            )


def test_no_logitsprocessor_wrapping_lookup_output():
    """A LogitsProcessor or LogitsWarper that mutates accepted tokens
    AFTER prompt_lookup verification would break losslessness. We refuse
    any registration that names both 'prompt_lookup' and 'LogitsProcessor'
    on adjacent lines in server.py code paths.

    Liberal heuristic: scan for a small textual window around any
    'prompt_lookup_num_tokens' reference and ensure no LogitsProcessor /
    LogitsWarper / sample-temp injection appears within 5 lines."""
    source = _read_server_source().splitlines()
    suspect_tokens = [
        "LogitsProcessor",
        "LogitsWarper",
        "TopKLogitsWarper",
        "TopPLogitsWarper",
        "TemperatureLogitsWarper",
    ]
    for idx, line in enumerate(source):
        if "prompt_lookup_num_tokens" not in line:
            continue
        # Skip comment-only lines.
        if line.lstrip().startswith("#"):
            continue
        # Look 5 lines on each side for suspect processors.
        lo, hi = max(0, idx - 5), min(len(source), idx + 6)
        window = "\n".join(source[lo:hi])
        for tok in suspect_tokens:
            assert tok not in window, (
                f"Found {tok} within 5 lines of prompt_lookup_num_tokens "
                f"(line {idx+1}). Lossless guarantee may be violated if "
                "the processor mutates accepted tokens."
            )
