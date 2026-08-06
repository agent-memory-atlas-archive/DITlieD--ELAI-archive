# J-82 decisions (measure-first)

## Format choice
ELAI compact sorted-key JSON emission (no TOON crate, no axi-sdk-js). Anti-slopsquat: avoid unverified npm/TOON dep on the hot path. Grammar not invented as an external TOON port; deterministic `serde_json` with sorted keys is the ELAI-roll.

## Legs default
All three legs default OFF via env:
- `ELAI_TOKEN_EMIT_FORMAT=1`
- `ELAI_TOKEN_EMIT_LOG_DICT=1`
- `ELAI_TOKEN_EMIT_PROTECTED_SPAN=1`

KEEP-OFF until `elai bench tool-output` records a positive token delta on heavy returns (GD-4). A marginal delta is a legitimate KEEP-OFF.

## Measurement
Run: `elai bench tool-output --seed 1234 --json`
Record baseline_tokens, format_delta, protected_span_delta, log_dict_tokens. Fill measured numbers here after first real heavy-return corpus run. Hermetic fixture exists for determinism; production KEEP-OFF until positive heavy-return evidence.

## Production seams
- ipc_accept tool-result path after annotate_tool_result (format + protected-span when enabled)
- mcp_budget::enforce_output_budget on compact path (chars/4, DEFAULT_OUTPUT_BUDGET_TOKENS=8000)
- CLI: `elai bench tool-output`

## RESPECT
K-12 must-preserve: structured spans and code streams are not collapsed; protected-span only drops prose fillers outside fences.
