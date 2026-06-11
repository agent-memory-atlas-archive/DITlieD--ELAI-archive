"""E-45 — Pydantic typed boundary for /v1/messages.

Mirrors the orchestrator's Anthropic-protocol request/response shapes.
Used by `server.handle_messages` as the validation gate at the HTTP
boundary; replaces the prior dict.get-with-defaults pattern with
explicit fields that match the Rust side declared in
`crates/providers/src/providers/anthropic.rs`.

Schema discipline:
* `extra="allow"` on the request model so unknown fields are forwarded
  to the engine without breaking older callers; the matching Rust
  struct uses `#[serde(other)]` capture for forward-compat.
* All numeric fields are typed `Optional[int]` / `Optional[float]` so
  the strict-bool rejection in `handle_messages` (server.py:3351) keeps
  working without coercion surprises.
* `SCHEMA_VERSION` MUST equal the orchestrator's `SCHEMA_VERSION` const
  at `crates/orchestrator/src/ssb.rs`. `DriftDetector` flags mismatch
  via the `schema_version_mismatch` SSB beacon.
"""

from typing import Any, List, Optional, Union

from pydantic import BaseModel, ConfigDict, Field

# E-45: keep in lockstep with crates/orchestrator/src/ssb.rs::SCHEMA_VERSION.
# `cross_lang_drift::DriftDetector` reads this integer and compares
# against the Rust-side constant; a divergence emits
# `SsbActionType::SchemaVersionMismatch`.
#
# Tranche 8B (2026-05-31): 138 -> 191 to match
# `SCHEMA_VERSION_PUBLIC` re-exported from ssb.rs after the cumulative
# Tranche 6 (E-31) + earlier bumps. Exploration tranche8-E45-wiring.md
# flagged the 53-version drift as MEDIUM severity; the drift was real
# (Pydantic side had not been touched since E-45 land at 138). The
# bump is purely a constant update: no schema-shape changes on either
# side here -- the schemars derives (Tranche 8A) cover that surface
# separately via the JSON Schema snapshot fixtures.
#
# E-47 (2026-05-31): 191 -> 192 in lockstep with the
# LoraShipGateEvent SINGLE-VARIANT Pattern-2 (44th application) bump
# at ssb.rs:SCHEMA_VERSION. 3-arm LoraShipGateEventPhase discriminator
# (refused / calibration_missing / calibration_passed). Wire-dark
# unless ELAI_LORA_SHIP_GATE_ENABLE=1. Producer:
# crates/orchestrator/src/lora_ship_gate.rs::notify_lora_ship_gate +
# `elai lora-gate-check` CLI subcommand. The Pydantic side does not
# carry the typed envelope (no Anthropic-protocol surface area), so
# this lockstep bump is purely a constant update to keep the
# DriftDetector quiet.
#
# E-40 Tranche B-2 (2026-05-31): 192 -> 193 in lockstep with the
# SymbolGraphEvent SINGLE-VARIANT Pattern-2 (45th application) bump
# at ssb.rs:SCHEMA_VERSION. 4-arm SymbolGraphEventKind discriminator
# (symbol_dsl_executed / dataflow_impact_computed /
# agent_impact_claim_refuted / scope_graph_resolved). Wire-dark
# unless ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE=1 (or other per-event
# gates upstream). Producer:
# crates/orchestrator/src/symbol_graph/mod.rs::notify_symbol_graph_event +
# the new symbol_dsl_query / symbol_dataflow_impact ToolDefinitions.
# Pydantic side does not carry the typed envelope (no Anthropic-protocol
# surface area), so this lockstep bump is purely a constant update.
#
# Tranche B-6 quad (2026-05-31): 193 -> 194 in lockstep with four new
# SSB substrates landing in one coordinated orchestrate run:
#   * C-42 McpAuthzEvent (3-arm Kind authz_check_passed /
#     authz_check_failed / corpus_replayed). Wire-dark unless
#     ELAI_C42_MCP_AUTHZ_EMIT_ENABLE=1. Producer:
#     crates/plugins/src/authz/compose.rs::compose_with_tool_gate.
#   * C-47 ToolCalibrationEvent (6-arm Kind hint_emitted /
#     drift_detected / call_logged / case_added / case_selected /
#     calibration_applied). Wire-dark unless
#     ELAI_C47_CASE_BASE_ENABLE=1.
#   * B-39 ReflectBudgetSoftWarn (single-add). Wire-dark unless
#     ELAI_B39_SOFTWARN_ENABLE=1. Producer:
#     crates/reasoning/src/chat_reflect.rs ~L454.
#   * C-50 ContractCodingEvent (3-arm Kind retry_attempted /
#     retry_resolved / retry_abandoned). Wire-dark unless
#     ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE=1. Producer:
#     crates/orchestrator/src/task_runner.rs::fix_and_rerun_until_green.
# Pydantic side does not carry the typed envelope; constant-only bump.
# See workflow/reasoning/B6-quad-tranche-decisions.md.
#
# Tranche B-7 triple (2026-05-31): 194 -> 195 in lockstep with three
# coordinated SSB extensions landing in one orchestrate run:
#   * E-13 SHIPPABLE-NOW: NEW TddVerificationEvent (4-arm Kind
#     property_gate_pass / mutation_gate_survivor_block /
#     anti_mock_block / assertion_only_diff_block). Pattern-2 (46th
#     application). Wire-dark unless
#     ELAI_E13_TDD_VERIFICATION_EMIT_ENABLE=1.
#   * E-12 carry: AssertionOnlyDiffBlock collapses into arm #4 of
#     TddVerificationEventKind (NO new top-level variant; producer
#     wiring DEFERRED -- substrate-only this tranche).
#   * E-50 extension: SaasArchPatternEventKind gains 12th arm
#     saas_archetype_selected. NO additional SCHEMA bump (within-variant
#     arm extension). Wire-dark unless
#     ELAI_E50_ARCHETYPE_SELECTED_EMIT_ENABLE=1.
# Pydantic side does not carry the typed envelope; constant-only bump.
# See workflow/reasoning/B7-triple-tranche-decisions.md.
# H-47 (SCHEMA_VERSION 225, 2026-06-11): EvalFirewallEvent -- DGM eval firewall
# beacon. Composes D-6 date-gate + E-54 template-regen + H-19 Thresholdout
# budget binding. Pattern-2 single-variant 3-arm EvalFirewallEventKind:
# PreUpdateItemRejected | TemplateRegenerated | IndistinguishabilityProbe.
# Pydantic side does not carry the typed envelope; constant-only bump.
SCHEMA_VERSION: int = 225


class MessageContentBlock(BaseModel):
    """One block of message content (text / image / tool_use / tool_result).

    Mirrors the Anthropic content-block discriminated union. We do not
    enforce the discriminator here because /v1/messages accepts an
    open-ended block list — the engine inspects `type` itself.
    """

    model_config = ConfigDict(extra="allow")

    type: str = Field(..., description="Block discriminator (text, image, tool_use, tool_result, ...)")
    text: Optional[str] = None
    # `content` is used by tool_result blocks for the inner payload.
    content: Optional[Any] = None


class InboundMessage(BaseModel):
    """A single entry in the request `messages` array.

    Per Anthropic API, `content` can be a string OR a list of blocks.
    """

    model_config = ConfigDict(extra="allow")

    role: str = Field(..., description="user | assistant | system")
    content: Union[str, List[MessageContentBlock]]


class SystemBlock(BaseModel):
    """One entry of a list-form system prompt (Anthropic permits a
    structured `system` field beyond plain string)."""

    model_config = ConfigDict(extra="allow")

    type: str = Field(..., description="Typically 'text'")
    text: Optional[str] = None


class ToolDefinition(BaseModel):
    """Anthropic-protocol tool entry. Schema is engine-opaque; we just
    pass it through after validating the wrapper shape."""

    model_config = ConfigDict(extra="allow")

    name: str
    description: Optional[str] = None
    input_schema: Optional[Any] = None


class MessagesRequest(BaseModel):
    """E-45 — typed request body for POST /v1/messages.

    Mirrors the Rust serde shape on the orchestrator side. Fields
    marked Optional align with the dict.get(..., default) defaults in
    `server.handle_messages` so the migration is byte-identical at the
    semantics level. `model_config.extra='allow'` preserves forward
    compatibility with new fields the orchestrator may add before the
    sidecar regenerates this mirror.
    """

    model_config = ConfigDict(extra="allow")

    model: str = Field(default="elai-local")
    messages: List[InboundMessage] = Field(default_factory=list)
    system: Union[str, List[SystemBlock]] = ""
    max_tokens: Optional[int] = None
    stream: bool = False
    tools: List[ToolDefinition] = Field(default_factory=list)
    temperature: float = 0.0
    seed: Optional[int] = None
    fitm: Optional[bool] = None


class MessagesResponse(BaseModel):
    """E-45 — typed response body for POST /v1/messages.

    Anthropic-protocol response shape. Engine constructs the dict and
    we validate-then-emit so any field drift surfaces at serialization
    time rather than as a downstream parse error on the orchestrator.
    """

    model_config = ConfigDict(extra="allow")

    id: str
    type: str = "message"
    role: str = "assistant"
    model: str
    content: List[MessageContentBlock]
    stop_reason: Optional[str] = None
    stop_sequence: Optional[str] = None
    usage: Optional[Any] = None
