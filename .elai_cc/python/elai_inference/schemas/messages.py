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
SCHEMA_VERSION: int = 138


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
