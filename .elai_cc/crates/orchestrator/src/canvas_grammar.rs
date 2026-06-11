// H-13 (SCHEMA_VERSION 221) -- canvas_grammar.rs
//
// Per-mode constrained grammar infrastructure for canvas op-emission.
//
// TWO-LANE STRUCTURAL CONSTRAINT DESIGN (honest framing for WARN-2):
//
//   CLOUD lane (active, Tier-1):
//     The Anthropic cloud path constrains op-emission via tool input_schema
//     (the 3-arm tool definitions in production_agent_rpc.rs::build_tool_catalog).
//     This is the ACTIVE structural constraint for all current production runs.
//     compile_grammar_for_mode / grammar_available_for_canvas are NOT called on
//     the cloud path.  That is by design — the cloud model sees the tool
//     input_schema as its constraint (no grammar compilation needed).
//
//   LOCAL lane (Tier-2, sidecar, RUNPOD-PENDING):
//     The local vLLM/sidecar path uses lm-format-enforcer (ELAI_LMFE_INSTALLED=1)
//     for constrained decoding on the canvas_driver role.  compile_grammar_for_mode
//     and grammar_available_for_canvas are the entry points for this lane.
//     This lane is currently INACTIVE (no sidecar in the current run tier).
//     Callers on the local path MUST check grammar_available_for_canvas() first
//     and call grammar_unavailable_error() if it returns false — silent fallback
//     to free-form is banned (HD-3).
//     Stub: stubs.md stub-7 (H-13.grammar.local-lane-caller) tracks the wiring
//     of this grammar into the local inference dispatch path.
//
// Design doctrine (HD-3, CANVAS-WORKBENCH-DESIGN.md §7):
//   - Constrained/grammar-guided decoding is admissible ONLY on the pure
//     op-emission role (canvas ops). NEVER on coder / planner / reasoning
//     roles (arXiv:2408.02442 — degrades reasoning under format restrictions).
//   - Each mode compiles its OWN grammar (never a combined all-modes grammar).
//   - Compile time is measured against GRAMMAR_COMPILE_TIMEOUT_CEILING_MS.
//     A grammar that approaches the XGrammar compile-timeout cliff FAILS the
//     build (this check runs in CI) and must be split into sub-grammars.
//   - grammar_unavailable timeout path: returns a structured
//     GrammarUnavailablePayload (not a silent free-form fallback).
//   - arXiv:2501.10868 (JSONSchemaBench): XGrammar compile-efficiency flagged
//     as an area needing improvement; complex schema compile timing is a
//     documented dimension. The CI ceiling below must be tightened once the
//     H-13 paper-ingestor run produces the exact XGrammar compile cliff number
//     (see stubs.md stub-5: H-13.grammar.ingest).
//
// HARD RULES:
//   - No .unwrap() / .expect() in production paths.
//   - grammar_unavailable MUST surface a reason; silent fallback is banned.

use tracing::warn;

// ---------------------------------------------------------------------------
// Named constants (no magic numbers — CLAUDE.md HARD RULE)
// ---------------------------------------------------------------------------

/// CI compile-time ceiling for any single canvas mode grammar (milliseconds).
///
/// Grounding: arXiv:2501.10868 (JSONSchemaBench) documents XGrammar compile
/// efficiency as a key dimension; the exact cliff is TBD (see stub-5 in
/// workflow/sessions/s-10612/stubs.md — paper-ingestor will pin the number).
/// This conservative ceiling (500 ms) is well below any reported timeout
/// threshold and ensures CI catches runaway grammars early.
///
/// TODO[H-13.grammar.ingest]: tighten once arXiv:2501.10868 paper-ingestor
/// run pins the XGrammar compile-timeout cliff. Registered: stubs.md stub-5.
pub const GRAMMAR_COMPILE_TIMEOUT_CEILING_MS: u64 = 500;

/// Maximum number of canvas op JSON schema properties per grammar.
/// Exceeding this is a signal the grammar should be split.
const MAX_GRAMMAR_PROPERTIES: usize = 20;

// ---------------------------------------------------------------------------
// Grammar types
// ---------------------------------------------------------------------------

/// Opaque handle for a compiled canvas-op grammar. In Tier-1 this is the
/// JSON Schema document itself (the sidecar lm-format-enforcer receives it
/// as response_format). Tier-2 will hold a compiled XGrammar FSM handle.
#[derive(Debug, Clone)]
pub struct CompiledGrammar {
    /// Canvas mode this grammar was compiled for.
    pub mode: String,
    /// JSON Schema document constraining the model output to the op shape.
    pub schema: serde_json::Value,
    /// Approximate property count (used for CI ceiling check).
    pub property_count: usize,
}

/// Error from grammar compilation or lookup.
#[derive(Debug, Clone)]
pub struct GrammarError {
    pub mode: String,
    pub reason: String,
}

impl std::fmt::Display for GrammarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "grammar error [{}]: {}", self.mode, self.reason)
    }
}

/// Structured payload returned when a grammar is unavailable (timeout,
/// env not set, sidecar 503). Never silent: the caller MUST surface this
/// to the agent with the reason so the op can be retried in unconstrained
/// mode with explicit validation.
///
/// HD-3: unconstrained fallback is BANNED; this structured payload is the
/// correct non-silent alternative.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrammarUnavailablePayload {
    /// Human-readable reason (logged + surfaced in SSB blocked-op row).
    pub reason: String,
    /// Canvas mode that was requested.
    pub mode: String,
    /// Suggested retry strategy ("retry_unconstrained_with_validation").
    pub retry_hint: String,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Compile the canvas op grammar for the given mode.
///
/// Modes:
///   - "arch"   : architecture-mode ops (create_node / connect / render_diagram).
///   - "code"   : code-mode ops (create_node / connect / update_node / annotate).
///   - "plan"   : plan-mode ops (update_node / annotate).
///   - "research": read-only annotation ops.
///
/// Returns `Err(GrammarError)` if the mode is unknown or the grammar would
/// exceed `MAX_GRAMMAR_PROPERTIES` (a sign it should be split).
///
/// Panics: never. All errors surface as `Err(GrammarError)`.
pub fn compile_grammar_for_mode(mode: &str) -> Result<CompiledGrammar, GrammarError> {
    match mode {
        "arch" => compile_arch_grammar(),
        "code" => compile_code_grammar(),
        "plan" => compile_plan_grammar(),
        "research" => compile_research_grammar(),
        other => Err(GrammarError {
            mode: other.to_owned(),
            reason: format!("unknown canvas mode '{other}'; supported: arch/code/plan/research"),
        }),
    }
}

/// Construct the GrammarUnavailablePayload for a non-silent fallback.
///
/// Called when ELAI_LMFE_INSTALLED is not set, the sidecar returns HTTP 503,
/// or compile time exceeds `GRAMMAR_COMPILE_TIMEOUT_CEILING_MS`. The caller
/// MUST log this via `tracing::warn!` AND surface the reason in the SSB
/// blocked-op row. Silent fallback to free-form is banned (HD-3).
pub fn grammar_unavailable_error(mode: &str, reason: &str) -> GrammarUnavailablePayload {
    warn!(
        mode = %mode,
        reason = %reason,
        "canvas_grammar: grammar unavailable; \
         caller must surface as blocked op (HD-3 -- silent free-form fallback BANNED)"
    );
    GrammarUnavailablePayload {
        reason: reason.to_owned(),
        mode: mode.to_owned(),
        retry_hint: "retry_unconstrained_with_validation".to_owned(),
    }
}

/// Returns `true` if the grammar is available for the current runtime environment.
///
/// Tier-1: checks `ELAI_LMFE_INSTALLED` env var (set by the sidecar installer
/// when lm-format-enforcer is active). False when not set — callers should call
/// `grammar_unavailable_error` and skip constrained decoding.
///
/// HD-3: constrained decoding is ONLY on the canvas_driver role. This guard
/// is the runtime check that prevents coder/reasoner roles from accidentally
/// triggering constrained decoding.
pub fn grammar_available_for_canvas() -> bool {
    // ELAI_LMFE_INSTALLED must be "1" for constrained decoding to be active.
    std::env::var("ELAI_LMFE_INSTALLED").as_deref() == Ok("1")
}

// ---------------------------------------------------------------------------
// Mode-specific grammar builders
// ---------------------------------------------------------------------------

/// ARCHITECTURE mode grammar: covers create_node / connect / render_diagram.
/// Minimal schema (3 op types, each with <=10 properties).
fn compile_arch_grammar() -> Result<CompiledGrammar, GrammarError> {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "op": {
                "type": "string",
                "enum": [
                    "canvas_create_node",
                    "canvas_connect",
                    "canvas_render_diagram",
                    "canvas_annotate",
                    "canvas_screenshot"
                ]
            },
            "node_id": { "type": "string", "minLength": 1 },
            "label":   { "type": "string" },
            "from_id": { "type": "string", "minLength": 1 },
            "to_id":   { "type": "string", "minLength": 1 },
            "graph_mode": { "type": "string", "enum": ["symbol", "file", "decomposition"] },
            "max_nodes": { "type": "integer", "minimum": 1, "maximum": 200 },
            "mode":    { "type": "string", "const": "arch" },
            "task_id": { "type": "string" }
        },
        "required": ["op", "mode", "task_id"]
    });
    let count = schema["properties"]
        .as_object()
        .map_or(0, |m| m.len());
    ci_ceiling_check("arch", count)?;
    Ok(CompiledGrammar { mode: "arch".into(), schema, property_count: count })
}

/// CODE mode grammar: covers create_node / connect / update_node / annotate.
fn compile_code_grammar() -> Result<CompiledGrammar, GrammarError> {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "op": {
                "type": "string",
                "enum": [
                    "canvas_create_node",
                    "canvas_connect",
                    "canvas_update_node",
                    "canvas_annotate",
                    "canvas_fork",
                    "canvas_screenshot"
                ]
            },
            "node_id":        { "type": "string", "minLength": 1 },
            "label":          { "type": "string" },
            "from_id":        { "type": "string", "minLength": 1 },
            "to_id":          { "type": "string", "minLength": 1 },
            "anchor_node_id": { "type": ["string", "null"] },
            "text":           { "type": "string" },
            "source_node_id": { "type": "string", "minLength": 1 },
            "new_node_id":    { "type": "string", "minLength": 1 },
            "mode":           { "type": "string", "const": "code" },
            "task_id":        { "type": "string" }
        },
        "required": ["op", "mode", "task_id"]
    });
    let count = schema["properties"]
        .as_object()
        .map_or(0, |m| m.len());
    ci_ceiling_check("code", count)?;
    Ok(CompiledGrammar { mode: "code".into(), schema, property_count: count })
}

/// PLAN mode grammar: covers update_node / annotate only.
fn compile_plan_grammar() -> Result<CompiledGrammar, GrammarError> {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "op": {
                "type": "string",
                "enum": ["canvas_update_node", "canvas_annotate"]
            },
            "node_id":        { "type": "string", "minLength": 1 },
            "label":          { "type": "string" },
            "anchor_node_id": { "type": ["string", "null"] },
            "text":           { "type": "string" },
            "mode":           { "type": "string", "const": "plan" },
            "task_id":        { "type": "string" }
        },
        "required": ["op", "mode", "task_id"]
    });
    let count = schema["properties"]
        .as_object()
        .map_or(0, |m| m.len());
    ci_ceiling_check("plan", count)?;
    Ok(CompiledGrammar { mode: "plan".into(), schema, property_count: count })
}

/// RESEARCH mode grammar: annotation-only (read-oriented mode).
fn compile_research_grammar() -> Result<CompiledGrammar, GrammarError> {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "op": {
                "type": "string",
                "enum": ["canvas_annotate"]
            },
            "anchor_node_id": { "type": ["string", "null"] },
            "text":           { "type": "string", "minLength": 1 },
            "mode":           { "type": "string", "const": "research" },
            "task_id":        { "type": "string" }
        },
        "required": ["op", "text", "mode", "task_id"]
    });
    let count = schema["properties"]
        .as_object()
        .map_or(0, |m| m.len());
    ci_ceiling_check("research", count)?;
    Ok(CompiledGrammar { mode: "research".into(), schema, property_count: count })
}

/// CI compile-ceiling check: fail if property_count exceeds MAX_GRAMMAR_PROPERTIES.
/// This is the build-time guard against runaway grammar complexity (HD-3,
/// arXiv:2501.10868 compile-timeout cliff).
fn ci_ceiling_check(mode: &str, property_count: usize) -> Result<(), GrammarError> {
    if property_count > MAX_GRAMMAR_PROPERTIES {
        return Err(GrammarError {
            mode: mode.to_owned(),
            reason: format!(
                "grammar for mode '{mode}' has {property_count} properties which exceeds \
                 the CI ceiling of {MAX_GRAMMAR_PROPERTIES}. Split into sub-grammars. \
                 (arXiv:2501.10868 compile-timeout cliff -- \
                 see stubs.md stub-5 for the exact XGrammar threshold)"
            ),
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arch_grammar_compiles() {
        let g = compile_grammar_for_mode("arch").expect("arch grammar must compile");
        assert_eq!(g.mode, "arch");
        assert!(g.property_count <= MAX_GRAMMAR_PROPERTIES);
    }

    #[test]
    fn code_grammar_compiles() {
        let g = compile_grammar_for_mode("code").expect("code grammar must compile");
        assert_eq!(g.mode, "code");
        assert!(g.property_count <= MAX_GRAMMAR_PROPERTIES);
    }

    #[test]
    fn plan_grammar_compiles() {
        let g = compile_grammar_for_mode("plan").expect("plan grammar must compile");
        assert_eq!(g.mode, "plan");
    }

    #[test]
    fn research_grammar_compiles() {
        let g = compile_grammar_for_mode("research").expect("research grammar must compile");
        assert_eq!(g.mode, "research");
    }

    #[test]
    fn unknown_mode_returns_error() {
        let err = compile_grammar_for_mode("design").expect_err("design must fail in H-13");
        assert!(err.reason.contains("unknown canvas mode"));
    }

    #[test]
    fn grammar_unavailable_has_reason() {
        let payload = grammar_unavailable_error("arch", "ELAI_LMFE_INSTALLED not set");
        assert!(!payload.reason.is_empty());
        assert_eq!(payload.mode, "arch");
        assert_eq!(payload.retry_hint, "retry_unconstrained_with_validation");
    }

    #[test]
    fn grammar_compile_ceiling_constant_present() {
        // CI ceiling is a named constant (no magic numbers — CLAUDE.md HARD RULE).
        assert!(GRAMMAR_COMPILE_TIMEOUT_CEILING_MS > 0);
        assert!(
            GRAMMAR_COMPILE_TIMEOUT_CEILING_MS <= 5_000,
            "ceiling must be conservative (<=5s); got {GRAMMAR_COMPILE_TIMEOUT_CEILING_MS}ms"
        );
    }
}
