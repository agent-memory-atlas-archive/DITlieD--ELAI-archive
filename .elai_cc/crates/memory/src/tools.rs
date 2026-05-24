//! E-1 Task 5 — Chat-memory tool definitions exposed to the model in
//! the [`crate::chat_memory_reflect`] PRE-stage (MemGPT contract, Packer
//! et al. arXiv:2310.08560 §3.2).
//!
//! Two tools:
//!   * `core_memory_replace(section, new_text)` — model-driven edit of
//!     a named section of the pinned [`crate::CoreBlock`]. Invalid
//!     `section` names route to a silent no-op + SSB warning (per E-1
//!     plan Task 5 typed-validator contract).
//!   * `archival_search(query, top_k)` — PPR-seeded top-K retrieval
//!     over the A-MEM `memory_notes` graph. Thin wrapper around
//!     [`crate::ppr_seeded_top_k`].
//!
//! These structs are JSON-schema-shaped (the same shape as
//! `elai_plugins::mcp_bridge::McpToolSpec` but defined locally to avoid
//! a circular dep between `elai-memory` and `elai-plugins`). The
//! orchestrator side maps `ChatMemoryToolDef -> McpToolSpec` at the
//! ToolGate registration boundary (C-35).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Canonical name of the MemGPT-style `core_memory_replace` tool.
pub const CORE_MEMORY_REPLACE_TOOL_NAME: &str = "core_memory_replace";

/// Canonical name of the MemGPT-style `archival_search` tool.
pub const ARCHIVAL_SEARCH_TOOL_NAME: &str = "archival_search";

/// Mirror of `elai_plugins::mcp_bridge::McpToolSpec` (name +
/// description + JSON-schema input). Local definition keeps
/// `elai-memory` free of plugin / mcp deps; the orchestrator boundary
/// converts to McpToolSpec for ToolGate registration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMemoryToolDef {
    /// Stable snake_case tool name (matches the literal the model
    /// emits in tool-call JSON).
    pub name: String,
    /// One-sentence description rendered into the gated-summaries
    /// section of the prompt by the ToolGate.
    pub description: String,
    /// JSON-schema-shaped input descriptor. Always an object schema
    /// with `properties` + `required` per the OpenAI / Anthropic
    /// tool-call convention.
    pub input_schema: Value,
}

/// Build the `core_memory_replace` tool definition.
///
/// Schema:
/// ```text
/// section: enum["user_style", "user_expertise", "active_goal", "opinions"]
/// new_text: string (<= CORE_BLOCK_MAX_BYTES; longer routes to silent no-op)
/// ```
pub fn core_memory_replace_tool_def() -> ChatMemoryToolDef {
    ChatMemoryToolDef {
        name: CORE_MEMORY_REPLACE_TOOL_NAME.to_owned(),
        description: "Replace a named section of the pinned core memory \
                      (user_style | user_expertise | active_goal | opinions). \
                      Caps at CORE_BLOCK_MAX_BYTES. Invalid section is a \
                      no-op."
            .to_owned(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "section": {
                    "type": "string",
                    "enum": [
                        "user_style",
                        "user_expertise",
                        "active_goal",
                        "opinions",
                    ],
                    "description": "Section name to replace.",
                },
                "new_text": {
                    "type": "string",
                    "description": "Replacement body for the section. \
                                    Longer than CORE_BLOCK_MAX_BYTES \
                                    routes to a silent no-op + SSB warning.",
                },
            },
            "required": ["section", "new_text"],
        }),
    }
}

/// Build the `archival_search` tool definition.
///
/// Schema:
/// ```text
/// query: string (chat-side entity / keyword)
/// top_k: integer (1..=32, defaults to 8 client-side)
/// ```
pub fn archival_search_tool_def() -> ChatMemoryToolDef {
    ChatMemoryToolDef {
        name: ARCHIVAL_SEARCH_TOOL_NAME.to_owned(),
        description: "Top-K Personalised-PageRank retrieval over the \
                      A-MEM note store (Zettelkasten links + tag-Jaccard \
                      edges). Returns rows with body + tags + last \
                      accessed."
            .to_owned(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Entity / keyword from the current turn. \
                                    Lower-cased and used as the PPR seed.",
                },
                "top_k": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 32,
                    "description": "Number of top-ranked notes to return.",
                },
            },
            "required": ["query"],
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_memory_replace_def_is_well_formed() {
        let def = core_memory_replace_tool_def();
        assert_eq!(def.name, CORE_MEMORY_REPLACE_TOOL_NAME);
        let props = def.input_schema.get("properties").expect("properties");
        assert!(props.get("section").is_some());
        assert!(props.get("new_text").is_some());
        let section_enum = props
            .get("section")
            .and_then(|s| s.get("enum"))
            .and_then(|e| e.as_array())
            .expect("enum");
        assert_eq!(section_enum.len(), 4);
    }

    #[test]
    fn archival_search_def_is_well_formed() {
        let def = archival_search_tool_def();
        assert_eq!(def.name, ARCHIVAL_SEARCH_TOOL_NAME);
        let props = def.input_schema.get("properties").expect("properties");
        let top_k = props.get("top_k").expect("top_k");
        assert_eq!(top_k.get("maximum").and_then(Value::as_u64), Some(32));
        assert_eq!(top_k.get("minimum").and_then(Value::as_u64), Some(1));
    }

    #[test]
    fn tool_defs_serialize_roundtrip() {
        let def = core_memory_replace_tool_def();
        let s = serde_json::to_string(&def).expect("ser");
        let back: ChatMemoryToolDef = serde_json::from_str(&s).expect("de");
        assert_eq!(back, def);
    }
}
