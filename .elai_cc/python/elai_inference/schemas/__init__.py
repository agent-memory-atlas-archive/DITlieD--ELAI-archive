"""E-45 — Pydantic typed-boundary models for the sidecar HTTP surface.

Mirrors the orchestrator's serde-serialized shapes (Anthropic-protocol
`/v1/messages` request + response) so the sidecar can validate inbound
requests + serialize outbound responses with explicit field
declarations instead of dict-literal construction.

Per D-10 DECISION-8, the orchestrator-side Rust structs are the
authoritative shape; the Pydantic models here are the lint-the-mirror
side of the same discipline. The `cross_lang_drift::DriftDetector`
walks both sides via the `sidecar_orchestrator_schemas` pair entry in
`cross-lang-pairs.toml`.

SCHEMA_VERSION pinned to the orchestrator's `SCHEMA_VERSION` constant.
Bump in lockstep with `crates/orchestrator/src/ssb.rs::SCHEMA_VERSION`.
"""

from .messages import (
    SCHEMA_VERSION,
    MessagesRequest,
    MessagesResponse,
    MessageContentBlock,
    InboundMessage,
    ToolDefinition,
    SystemBlock,
)

__all__ = [
    "SCHEMA_VERSION",
    "MessagesRequest",
    "MessagesResponse",
    "MessageContentBlock",
    "InboundMessage",
    "ToolDefinition",
    "SystemBlock",
]
