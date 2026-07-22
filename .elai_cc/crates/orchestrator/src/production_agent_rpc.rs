// Phase B2c — ProductionAgentRpcServer
//
// Implements the 2-method `AgentRpcServer` trait:
//   - `handle_infer_request`: wraps `AnthropicClient::send_message`.
//   - `handle_tool_request`: translates wire-format `ToolRequest.tool_id` +
//     postcard-encoded `arguments` into the matching `ToolDispatcher::dispatch_*`
//     call via an `Arc<dyn ToolDispatcher>`.
//
// WIRING BACKEND (ELAI_GROUNDING_BACKEND env var, mirrors task_runner.rs:4133):
//   "cloud" -> AnthropicClient::from_env() (reads ANTHROPIC_BASE_URL / auth).
//   Any other value (unset / "sidecar" / empty) ->
//       AnthropicClient::new("local-dummy").with_base_url(ANTHROPIC_BASE_URL
//       env, default http://localhost:8100).
//
// No .unwrap() / .expect() outside #[cfg(test)] or const init.
// All error paths propagate via ToolOutcome::Failed / InferOutcome::Failed.

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_core::tool_dispatcher::{
    EditRequest, FailureInput, GitBranchPolicy, GitCommitRequest, IncrementalRequest,
    ToolDispatcher, ValidateRequest,
};
use elai_ipc_types::{
    EntryPointsRequest, ExportGraphMermaidRequest, IndexArtifactsRequest, ListArtifactsRequest,
    CanvasAnnotateRequest, CanvasConnectRequest, CanvasCreateNodeRequest, CanvasForkRequest,
    CanvasRenderDiagramRequest, CanvasScreenshotRequest, CanvasSpawnViewRequest,
    CanvasSpawnViewResponse, CanvasUpdateNodeRequest, RemoveArtifactsRequest,
    SearchArtifactsRequest,
    SymbolContextRequest, SymbolDataflowImpactRequest, SymbolDslQueryRequest, SymbolFlowRequest,
    SymbolImpactRequest, SymbolListRequest, TOOL_CANVAS_ANNOTATE, TOOL_CANVAS_CONNECT,
    TOOL_CANVAS_CREATE_NODE, TOOL_CANVAS_FORK, TOOL_CANVAS_RENDER_DIAGRAM,
    TOOL_CANVAS_SCREENSHOT, TOOL_CANVAS_SPAWN_VIEW, TOOL_CANVAS_UPDATE_NODE, TOOL_CLASSIFY,
    TOOL_COMPILE_CHECK,
    TOOL_DKG_CALLERS, TOOL_DKG_DATA_FLOW, TOOL_DKG_DEPENDENTS, TOOL_DKG_SUBGRAPH, TOOL_EDIT,
    TOOL_ENTRY_POINTS, TOOL_EXPORT_GRAPH_MERMAID, TOOL_FILE_READ, TOOL_GIT_BRANCH,
    TOOL_GIT_COMMIT, TOOL_GIT_UNDO, TOOL_INDEX_ARTIFACTS, TOOL_LIST_ARTIFACTS,
    TOOL_REMOVE_ARTIFACTS, TOOL_SEARCH_ARTIFACTS, TOOL_SYMBOL_CONTEXT,
    TOOL_SYMBOL_DATAFLOW_IMPACT, TOOL_SYMBOL_DSL_QUERY, TOOL_SYMBOL_FLOW,
    TOOL_SYMBOL_IMPACT, TOOL_SYMBOL_LIST, TOOL_VALIDATE,
    // I-8: voluntary-abort tool (3-arm IS-14 pattern).
    AcknowledgeInfeasibleRequest, TOOL_ACKNOWLEDGE_INFEASIBLE,
    // I-13: on-demand line-range expansion tool (3-arm IS-14 pattern).
    TraceQueryRequest, TOOL_TRACE_QUERY,
    // I-52: agent-callable checkpoint tool (3-arm save/rewind/diff, JSON dispatch).
    TOOL_CHECKPOINT_SAVE, TOOL_CHECKPOINT_REWIND, TOOL_CHECKPOINT_DIFF,
    // I-54: agent-callable structural (AST-aware) edit tool (3-arm search/preview/apply,
    // JSON dispatch over the shipped tree-sitter grammars).
    TOOL_STRUCTURAL_SEARCH, TOOL_STRUCTURAL_PREVIEW, TOOL_STRUCTURAL_APPLY,
};
use elai_knowledge::artifacts::{
    store::ArtifactStore,
    search::artifact_search_with_precomputed_embedding,
};
use elai_knowledge::symbol_graph::{
    cache::SymbolGraphCache,
    entry_points::{detect_entry_points, DetectedEntryPoint, EntryPointReason, EntryPointSummary},
    sharding::SYMBOL_FILE_LRU_SIZE,
    traversal::{compute_call_flow, compute_impact_radius, compute_symbol_context, list_symbols, SymbolFilter},
};
use elai_providers::{
    AnthropicClient, InputMessage, MessageRequest, OutputContentBlock, SystemBlock, ToolChoice,
    ToolDefinition,
};
use serde_json::Value;

use crate::ipc_accept::{AgentRpcServer, InferOutcome, ToolOutcome};
// J-1 — perception gateway (scaffold T2 + router T3 wired at
// handle_infer_request; wire-dark by default via
// ELAI_PERCEPTION_GATEWAY_ENABLE).
use crate::perception;
use crate::media_validator::tier1::ImageValidator;
use crate::media_validator::tier1::Tier1Validator;
use crate::media_validator::types::{
    MediaValidationRequest, MediaType, QualitySpec, ToolCapabilityFlags, ValidationMode,
};
// J-1.T3 controller decision 3: do NOT extend RefusalReason. The gate site
// returns RefusalDecision::Abstain with reason=None (constructed locally;
// decide_and_emit_with_reason is NOT called — that path requires an SSB
// emitter + injected clock not available at this seam, and the router's
// TypedRefusal is deterministic, not score-based).
use crate::calibration::RefusalDecision;

use elai_safety::provenance::{Provenance, Provenanced, TextSource};
use elai_safety::taint_flow::tainted_context;
use elai_safety::{allow_tool_with_origin, CapabilityVerdict};
use elai_safety::blast_radius::{check_fs_target, expand_fs_token, worktree_root_blake3, BlastRadiusVerdict, ExpandedToken};
use crate::ssb::{BlastRadiusKind, TaintFlowKind};
use crate::taint_flow_ssb_adapter::emit_taint_flow;
use crate::blast_radius_ssb_adapter::{blast_radius_gate_disabled, emit_blast_radius_blocked};


/// Default sidecar base URL when ANTHROPIC_BASE_URL is unset.
const DEFAULT_SIDECAR_URL: &str = "http://localhost:8100";

/// G-1: kill-switch env var. When set to a truthy value the data-plane taint
/// gate at the destructive-sink dispatch arms is DISARMED (pass-through). There
/// is NO enable flag (W-2 / R9): the gate is DEFAULT-ON, and this is the only
/// opt-out. Read fresh at each gate call so a long-running process honors a
/// mid-run flip.
const TAINT_PROPAGATE_DISABLE_ENV: &str = "ELAI_TAINT_PROPAGATE_DISABLE";

/// G-1: correlation id for taint_flow rows emitted from the agent-requested
/// tool-dispatch fan. The per-turn task id is not threaded into
/// `translate_tool_request` (it would require a signature change across the
/// IPC `AgentRpcServer` trait); this constant names the seam so a dashboard
/// subscriber can attribute the row to the agent dispatch path. (Per-turn
/// attribution is a documented follow-up, G-1-decisions.md GD-5.)
const TAINT_FLOW_TASK_ID: &str = "agent_tool_dispatch";

/// G-7: kill-switch env var for the lethal-trifecta dual-LLM quarantine gate.
/// When truthy the trifecta gate is DISARMED (pass-through). There is NO enable
/// flag (W-2 / R9): the gate is DEFAULT-ON and this is the only opt-out. Read
/// fresh at each gate call so a long-running process honors a mid-run flip.
const TRIFECTA_QUARANTINE_DISABLE_ENV: &str = "ELAI_TRIFECTA_QUARANTINE_DISABLE";

/// G-7: correlation id for trifecta_quarantined rows emitted from the
/// agent-requested tool-dispatch fan (same seam rationale as
/// [`TAINT_FLOW_TASK_ID`]).
const TRIFECTA_QUARANTINE_TASK_ID: &str = "agent_tool_dispatch";

/// G-1: env var naming the trusted-source workspace root for the file-read
/// path-screen (DD4). A file read resolving under this prefix is treated as
/// first-party Trusted source; anything else is Untrusted. Defaults to
/// `.elai_cc` (the project boundary per CLAUDE.md) when unset.
const TAINT_PROJECT_ROOT_ENV: &str = "ELAI_PROJECT_ROOT";

/// Default trusted-source workspace prefix when `ELAI_PROJECT_ROOT` is unset.
const TAINT_PROJECT_ROOT_DEFAULT: &str = ".elai_cc";

/// I-52: env var naming the checkpoint-tool session scope. Supplied by the run
/// context, NEVER by the agent -- the cross-session rewind guard (an agent
/// cannot reach another session's CAS because it cannot supply session_id, and
/// the CAS root is physically per-session). Falls back to a fixed default scope
/// when unset (single-session use).
const CHECKPOINT_SESSION_ID_ENV: &str = "ELAI_AGENT_SESSION_ID";

/// Default checkpoint session scope when `ELAI_AGENT_SESSION_ID` is unset.
const CHECKPOINT_SESSION_ID_DEFAULT: &str = "agent_default";

/// Resolve the run-context-supplied checkpoint session id (never agent-supplied).
fn checkpoint_session_id() -> String {
    std::env::var(CHECKPOINT_SESSION_ID_ENV)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| CHECKPOINT_SESSION_ID_DEFAULT.to_string())
}

/// I-52: env var overriding the checkpoint CAS root (run-context / test affordance,
/// never agent-supplied). Defaults to the H-49 `SESSION_CHECKPOINT_ROOT` so the
/// agent tool reads/writes the SAME store the H-49 capture seam populates.
const CHECKPOINT_ROOT_ENV: &str = "ELAI_AGENT_CHECKPOINT_ROOT";

/// Resolve the checkpoint CAS root (the H-49 store, or an env override for tests).
fn checkpoint_root() -> std::path::PathBuf {
    std::env::var(CHECKPOINT_ROOT_ENV)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(crate::architecture::timetravel::SESSION_CHECKPOINT_ROOT)
        })
}

/// G-1: resolve the trusted-source workspace root for the file-read path-screen.
fn taint_project_root() -> PathBuf {
    std::env::var(TAINT_PROJECT_ROOT_ENV)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(TAINT_PROJECT_ROOT_DEFAULT))
}

/// G-1: whether the data-plane taint gate is disarmed via the kill-switch.
fn taint_gate_disabled() -> bool {
    std::env::var(TAINT_PROPAGATE_DISABLE_ENV)
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            !(v.is_empty() || v == "0" || v == "false" || v == "no")
        })
        .unwrap_or(false)
}

/// G-1: map a [`TextSource`] to the SSB [`TaintFlowKind`] + a human origin
/// string for the taint_flow row. Only the four UNTRUSTED ingest categories
/// can reach a blocked sink, so the trusted/recallable arms fall back to the
/// closest Kind (they are never emitted in practice — the gate only blocks
/// Untrusted).
fn taint_flow_kind_and_origin(source: &TextSource) -> (TaintFlowKind, String) {
    match source {
        TextSource::Web { url } => (TaintFlowKind::Web, url.clone()),
        TextSource::ToolOutput { server_url, tool_name } => {
            (TaintFlowKind::ToolOutput, format!("{server_url}::{tool_name}"))
        }
        TextSource::File { source_path } => {
            (TaintFlowKind::File, source_path.to_string_lossy().into_owned())
        }
        TextSource::SandboxOutput { sandbox_id } => {
            (TaintFlowKind::Sandbox, sandbox_id.clone())
        }
        // These cannot block (Trusted / Recallable); reported as File for
        // completeness should the gate semantics ever widen to Recallable.
        TextSource::UserPrompt | TextSource::System | TextSource::Memory { .. } => {
            (TaintFlowKind::File, source.label().to_owned())
        }
    }
}

/// G-1 SINK GATE: refuse a destructive sink whose argv CONTAINS an untrusted
/// ingest's content VERBATIM as a substring.
///
/// HONEST SCOPE (s-56068 B2b, GD-5): this gate blocks a destructive sink
/// (git_commit / edit) whose argv contains an untrusted ingest's content
/// VERBATIM as a substring (>= `MIN_TAINT_MATCH_LEN` bytes), within the SAME
/// PROCESS. It does NOT cover transformed / summarized / paraphrased untrusted
/// content (the agent rewording an injected instruction breaks the verbatim
/// substring screen), cross-process sinks, or untrusted content arriving via a
/// non-`McpToolRegistry` MCP transport (those ingest sites are not registered).
/// Do not read this as taint-tracking dataflow; it is a verbatim-substring
/// content match, not a derives-from analysis.
///
/// For each `argv_part` (e.g. a git commit message + the file paths, or an
/// edit's file_path + new_text), look up the process-local taint side-table
/// for an untrusted ingest the part contains verbatim (the dependency
/// screen). If found, build a `Provenanced<String>` and consult the SHIPPED
/// `allow_tool_with_origin(gate_tool, &[..])` gate. On `Reject`, emit a
/// `taint_flow` SSB row (the off-agent artifact) and return `Err` so the
/// destructive dispatch is BLOCKED fail-closed.
///
/// `gate_tool` is the canonical destructive-tool name the gate checks against
/// `DESTRUCTIVE_TOOLS` (e.g. `"bash"` for git commit, `"fs_write"` for edit).
/// `sink_tool` is the human sink label recorded on the SSB row (e.g.
/// `"git_commit"`, `"edit"`). Returns `Ok(())` when the flow is clean or the
/// kill-switch is set.
fn enforce_taint_sink_gate(
    gate_tool: &str,
    sink_tool: &str,
    argv_parts: &[&str],
    ssb: Option<&Arc<crate::ssb::Ssb>>,
) -> Result<(), String> {
    if taint_gate_disabled() {
        return Ok(());
    }
    let ctx = tainted_context();
    for part in argv_parts {
        // HONEST SCOPE (GD-5): `lookup_for_argv` is a VERBATIM substring match
        // (>= MIN_TAINT_MATCH_LEN bytes), same-process only. It does NOT catch
        // transformed/summarized/paraphrased untrusted content, cross-process
        // sinks, or non-McpToolRegistry MCP transports.
        let Some(prov) = ctx.lookup_for_argv(part) else {
            continue;
        };
        // The gate Rejects on Untrusted only (Recallable does not block, by
        // design — capability_gate.rs:96). Build the argv element carrying the
        // joined provenance so the gate sees the trust label.
        let argv: Vec<Provenanced<String>> =
            vec![Provenanced::new((*part).to_owned(), prov.clone())];
        match allow_tool_with_origin(gate_tool, &argv) {
            Ok(CapabilityVerdict::Reject {
                reason,
                offending_trust_label,
                ..
            }) => {
                let (kind, origin) = taint_flow_kind_and_origin(&prov.source);
                if let Some(ssb) = ssb {
                    emit_taint_flow(
                        ssb,
                        kind,
                        &origin,
                        &offending_trust_label,
                        sink_tool,
                        TAINT_FLOW_TASK_ID,
                    );
                }
                return Err(format!(
                    "G-1 taint gate: destructive sink `{sink_tool}` BLOCKED — argv derives \
                     from untrusted ingest ({origin}); {reason}"
                ));
            }
            Ok(CapabilityVerdict::Allow) => {}
            Err(e) => {
                // Empty tool name is the only gate error; treat any gate error
                // as fail-CLOSED (do not let a destructive sink through on a
                // gate malfunction).
                return Err(format!(
                    "G-1 taint gate: destructive sink `{sink_tool}` BLOCKED — gate error: {e:?}"
                ));
            }
        }
    }
    Ok(())
}

/// G-7: whether the lethal-trifecta quarantine gate is disarmed via the
/// kill-switch. Default-ON otherwise (R9).
fn trifecta_quarantine_disabled() -> bool {
    std::env::var(TRIFECTA_QUARANTINE_DISABLE_ENV)
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            !(v.is_empty() || v == "0" || v == "false" || v == "no")
        })
        .unwrap_or(false)
}

/// G-7 LETHAL-TRIFECTA QUARANTINE GATE. Runs BEFORE a destructive / egress
/// dispatch (alongside the G-1 taint gate + G-3 blast-radius gate), fail-closed.
///
/// Detects the lethal trifecta at the tool-call chokepoint by COMPOSING three
/// already-shipped subsystems (GD-1, re-implements none):
///   * (a) SECRET-PRESENT — the G-9 redaction detector
///     ([`crate::redaction::redact`]) matches a secret pattern in the visible
///     argv context. This is the bridge for the confidentiality leg the taint
///     table cannot see (it drops Trusted provenance).
///   * (b) UNTRUSTED SPAN — the G-1 taint table
///     ([`elai_safety::TaintedContext::lookup_for_argv`]) returns an `Untrusted`
///     provenance for an argv part.
///   * (c) EGRESS — the pending `gate_tool` is egress / destructive
///     ([`elai_safety::is_destructive_tool`]).
///
/// On ALL THREE: the untrusted span MUST NOT reach the tool-caller. We BLOCK the
/// destructive dispatch fail-closed (return `Err`) so the injected instruction
/// never drives the real tool, and emit a `trifecta_quarantined` SSB row through
/// the LIVE `Arc<Ssb>` (the off-agent artifact, R4).
///
/// G-7 FIX 2 (GD-5 honesty): at THIS synchronous chokepoint NO sub-LLM runs —
/// the no-tool sub-LLM routing ([`crate::quarantine::query_quarantined`]) is the
/// producer-side primitive but it requires a live `QuarantineDispatcher` that is
/// NOT threaded into `translate_tool_request`. The load-bearing property here is
/// the hard BLOCK — the raw untrusted span + the egress tool never co-flow. The
/// emitted row is therefore HONEST: `quarantined = false`,
/// `sub_llm_verdict = "not_run_blocked"` (it does NOT claim a sub-LLM
/// quarantined the span when it was a hard block). Live-sub-LLM routing at this
/// chokepoint is a DEFERRED follow-up.
///
/// G-7 FIX 3 (GD-5 scope honesty): the secret leg (a) scans the tool ARGV only
/// (commit message + paths), NOT the full live agent context. The live agent
/// messages are not accessible at this free-fn chokepoint. This is fail-closed-
/// SAFE (argv under-detection causes a missed BLOCK, never a false BLOCK) but
/// NARROWER than "secret-in-context": a context-secret not present in this argv
/// is NOT yet detected. Broadening to the full context is a DEFERRED follow-up
/// (it needs the agent messages threaded to this chokepoint).
///
/// Returns `Ok(())` when the trifecta does NOT fire (any leg missing — the FP
/// guard) or the kill-switch is set.
fn enforce_trifecta_quarantine_gate(
    gate_tool: &str,
    sink_tool: &str,
    argv_parts: &[&str],
    ssb: Option<&Arc<crate::ssb::Ssb>>,
) -> Result<(), String> {
    if trifecta_quarantine_disabled() {
        return Ok(());
    }
    // Leg (c): is the pending tool an EGRESS channel? Cheapest check first; if
    // not egress, no trifecta is possible regardless of the other legs. The
    // egress subset (curl/web_fetch/http_post/bash/shell) is STRICTER than the
    // broad destructive-sink set — a local fs_write cannot exfiltrate off-box.
    if !crate::quarantine::is_egress_tool(gate_tool) {
        return Ok(());
    }
    // Leg (a): does any visible argv part contain a secret (G-9 detector)?
    // G-7 FIX 3 SCOPE (GD-5 honesty): this scans ONLY the tool argv parts (the
    // commit message + paths the agent passed), NOT the full live agent
    // context/messages — those are not accessible at this free-fn chokepoint
    // (`translate_tool_request` receives the decoded `wire` argv, not the agent
    // conversation). Fail-closed-SAFE: a secret that is in the agent context but
    // absent from this argv is under-detected (a missed BLOCK), never a false
    // BLOCK. Broadening to the full context is a DEFERRED follow-up (needs the
    // agent messages threaded here).
    let secret_present = argv_parts
        .iter()
        .any(|part| crate::quarantine::context_holds_secret(part));
    if !secret_present {
        return Ok(());
    }
    // Leg (b): does any argv part derive from an untrusted ingest (G-1 taint)?
    let ctx = tainted_context();
    let (components, untrusted_prov) =
        crate::quarantine::detect_trifecta(&ctx, argv_parts, gate_tool, secret_present);
    if !components.is_trifecta() {
        // Missing a leg (no untrusted span reached the argv) — FP guard, allow.
        return Ok(());
    }
    // ALL THREE legs present. Block fail-closed + emit the off-agent artifact.
    let origin = match untrusted_prov.as_ref() {
        Some(prov) => taint_flow_kind_and_origin(&prov.source).1,
        // is_trifecta() guarantees untrusted_span, so prov is Some; this arm is
        // defensive only.
        None => sink_tool.to_owned(),
    };
    if let Some(ssb) = ssb {
        // G-7 FIX 2 (GD-5 honesty): at THIS synchronous chokepoint NO sub-LLM
        // runs — `query_quarantined` requires a live `QuarantineDispatcher`
        // (an AnthropicClient impl) that is NOT threaded into the free
        // `translate_tool_request` chokepoint, only the `dispatcher` + `ssb`
        // are. Wiring the live client here is a large signature-ripple blast
        // radius (every caller + `translate_tool_request_for_test`), so the
        // shipped behaviour is the hard BLOCK below. The emitted row therefore
        // MUST reflect what actually happened: `quarantined = false` (no
        // sub-LLM routed the span) and `sub_llm_verdict = "not_run_blocked"`
        // (the dispatch was hard-blocked, not quarantine-summarized). It would
        // be a lie to claim a sub-LLM ran. The load-bearing security property
        // is unchanged: the destructive dispatch is REFUSED (Err below) so the
        // untrusted span never reaches the real tool. DEFERRED follow-up: route
        // the untrusted span through the LIVE no-tool sub-LLM
        // (production_agent_rpc.rs:457 `client` as a `QuarantineDispatcher`)
        // once the client is threaded to this chokepoint; then this row's
        // `quarantined`/`sub_llm_verdict` reflect the real sub-LLM output.
        crate::trifecta_quarantine_ssb_adapter::emit_trifecta_quarantined(
            ssb,
            components,
            &origin,
            sink_tool,
            false,
            "not_run_blocked",
            TRIFECTA_QUARANTINE_TASK_ID,
        );
    }
    Err(format!(
        "G-7 lethal-trifecta gate: destructive sink `{sink_tool}` BLOCKED — a secret \
         is present AND an untrusted span flows into an egress-capable tool. The \
         untrusted span was quarantined and never reached the tool-caller."
    ))
}

/// G-3: resolve the per-process home directory for `~` expansion in the FS
/// blast-radius gate. Injected into [`expand_fs_token`] so the gate is
/// deterministic. `HOME` (Unix) then `USERPROFILE` (Windows).
fn home_dir_for_expansion() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

/// G-3: FS post-expansion blast-radius gate. Runs BEFORE a real filesystem-
/// mutating dispatch (the `fs_write` edit arm; git_commit paths), fail-closed.
///
/// For each path token the agent issued, expand `~`/`$VAR`/glob Rust-side then
/// [`check_fs_target`] the resolved target against the per-run worktree root.
/// On a [`BlastRadiusVerdict::Block`] the call is REFUSED with an Err and a
/// `blast_radius_blocked{FsEscape}` SSB row is emitted through the LIVE
/// `Arc<Ssb>` already threaded into the dispatch fan (the off-agent artifact,
/// R4). The kill-switch `ELAI_BLAST_RADIUS_DISABLE` disarms the gate (R9
/// default-ON otherwise). COMPOSES with E-30's pre-expansion verb gate
/// (different layer, different subject) — re-implements neither (GD-1).
///
/// `pre_expansion` is the original command/argv string recorded verbatim on the
/// row (the `#10077` forensics fix). `now_ms` is the injected clock value.
fn enforce_fs_blast_radius(
    path_tokens: &[&str],
    worktree_root: &std::path::Path,
    pre_expansion: &str,
    ssb: Option<&Arc<crate::ssb::Ssb>>,
    now_ms: i64,
) -> Result<(), String> {
    if blast_radius_gate_disabled() {
        return Ok(());
    }
    let home = home_dir_for_expansion();
    for token in path_tokens {
        let candidate: PathBuf = match expand_fs_token(token, home.as_deref()) {
            ExpandedToken::Path(p) => p,
            // A glob token: only the literal base before the first wildcard is
            // known. Check containment of that base; a glob whose base could
            // reach outside the root is refused (fail-closed on ambiguity).
            ExpandedToken::GlobBase(base) => {
                // An empty base means a bare `*.rs` relative to cwd; join it
                // onto the worktree root so an in-root glob is allowed.
                if base.as_os_str().is_empty() {
                    worktree_root.to_path_buf()
                } else {
                    base
                }
            }
        };
        let verdict = check_fs_target(&candidate, worktree_root, pre_expansion);
        if let BlastRadiusVerdict::Block {
            kind,
            pre_expansion: pre,
            post_expansion: post,
        } = verdict
        {
            if let Some(ssb) = ssb {
                emit_blast_radius_blocked(
                    ssb,
                    BlastRadiusKind::from(kind),
                    &pre,
                    &post,
                    &candidate.to_string_lossy(),
                    &worktree_root_blake3(worktree_root),
                    "", // pure containment block; no provenance join here
                    now_ms,
                );
            }
            return Err(format!(
                "G-3 blast-radius gate: filesystem target `{}` BLOCKED — resolves OUTSIDE \
                 the worktree root `{}` (fail-closed); resolved={}",
                token,
                worktree_root.display(),
                post
            ));
        }
    }
    Ok(())
}

/// Default model used when constructing a `MessageRequest`.
const DEFAULT_MODEL: &str = "claude-sonnet-4-6";

/// Default max_tokens used when the caller does not override.
const DEFAULT_MAX_TOKENS: u32 = 4096;

/// Outline-mode threshold: files above this many lines get a signatures-only
/// view from `dispatch_file_read_outline`. Matches
/// `elai_tools::dispatcher::OUTLINE_LINE_THRESHOLD`.
const OUTLINE_LINE_THRESHOLD: usize = 300;

/// patch-R5 — canonical system prompt biasing Sonnet into invoking tools
/// from the 6-tool wire catalog. Without this, B3 cloud E2E observed Sonnet
/// returning text-only responses and the agent's `run_react_turn` exiting
/// after one iteration with zero `TOOL_USE` markers parsed and no
/// `FileWrite` SSB row written (see R4 handoff `e2e_cloud_sonnet_run_v9.log`,
/// L737 panic `fw_rows=0`).
///
/// Production call sites (B3 e2e, CLI `elai task`, dashboard "Run" button)
/// pass this constant via `ProductionAgentRpcServer::new(...).with_system_prompt(
/// DEFAULT_BABY_STEP_SYSTEM_PROMPT)`. The wire catalog (R3) only publishes
/// the six tools enumerated below; their exact names match the
/// `elai_ipc_types::TOOL_*` wire IDs.
pub const DEFAULT_BABY_STEP_SYSTEM_PROMPT: &str = include_str!("prompts/baby_step_system.md");

// ---------------------------------------------------------------------------
// ProductionAgentRpcServer
// ---------------------------------------------------------------------------

/// Production implementation of `AgentRpcServer`.
///
/// - `client`: AnthropicClient pointed at either the remote sidecar or the
///   Anthropic cloud API, depending on `ELAI_GROUNDING_BACKEND`.
/// - `dispatcher`: `Arc<dyn ToolDispatcher>` for in-process tool calls
///   (type-erased so production code does not pull in `elai-tools`).
/// - `model`: model identifier forwarded on every `MessageRequest`.
/// - `max_tokens`: token budget forwarded on every `MessageRequest`.
pub struct ProductionAgentRpcServer {
    client: AnthropicClient,
    dispatcher: Arc<dyn ToolDispatcher>,
    model: String,
    max_tokens: u32,
    /// Optional SSB handle used by the B-29 Phase 5 symbol-graph tools to
    /// append SymbolQuery action rows for the dashboard ActivityFeed.
    /// `None` for harness/test paths that do not wire an SSB instance.
    ssb: Option<Arc<crate::ssb::Ssb>>,
    /// Optional VectorModule used by the `search_artifacts` dispatcher arm
    /// (B-35 Phase B). When `None`, artifact search falls back to returning
    /// an empty result set (mirrors `artifact_search` graceful-degrade behavior).
    /// Deferred wiring (B-35 Phase C2): inject via a proper service-locator so
    /// per-project VectorModules can be selected by db_path. Set via
    /// `with_artifact_vm` at server startup.
    artifact_vm: Option<Arc<dyn elai_knowledge::vector::VectorModule + Send + Sync>>,
    /// patch-R5 — optional system prompt forwarded as a one-block
    /// `SystemBlock` array on every `MessageRequest`. `None` (the default)
    /// preserves R3.1 behaviour (`request.system = None`, no leak into the
    /// 10+ existing 4-arg call sites). Production wires
    /// `DEFAULT_BABY_STEP_SYSTEM_PROMPT` via `with_system_prompt(...)` so
    /// Sonnet emits `ToolUse` blocks instead of text-only replies.
    system_prompt: Option<String>,
    /// patch-R3 — per-server stash of `OutputContentBlock::ToolUse.input` JSON
    /// values, keyed by tool name and ordered FIFO for repeated names.
    /// `handle_infer_request` populates this when Sonnet returns ToolUse
    /// blocks (emitting `"TOOL_USE: <name>"` markers in the response text per
    /// agent_loop.rs:79 contract); `handle_tool_request` pops a value when the
    /// agent sends `ToolRequest.arguments = Vec::new()` (agent_loop.rs:170) and
    /// converts JSON -> postcard via `convert_json_input_to_postcard`.
    /// `tokio::sync::Mutex` mandated because the lock is held across `.await`
    /// in `handle_infer_request` (push loop after `send_message` resolves);
    /// `std::sync::Mutex` would prevent `ProductionAgentRpcServer` from being
    /// usable behind `Arc<dyn AgentRpcServer>` whose methods are async.
    pub pending_tool_args: Arc<tokio::sync::Mutex<HashMap<String, VecDeque<Value>>>>,
    /// C-15: ClockProvider used for all timing inside tool dispatch (latency
    /// measurement, artifact indexing `now_secs`). Defaults to `OsClock`;
    /// callers can override via `with_clock()` for deterministic tests.
    clock: Arc<dyn ClockProvider>,
    /// E-40 / s-22600 Tranche B ITEM B.4 — optional SymbolGraph adapter
    /// used by the `TOOL_SYMBOL_DSL_QUERY` + `TOOL_SYMBOL_DATAFLOW_IMPACT`
    /// dispatcher arms. When `None`, both arms continue to return typed
    /// empty responses (preserves the pre-B.4 wire shape for harness /
    /// test paths that do not wire a real cache). When `Some`, the arms
    /// dispatch the DSL / impact query through the adapter and return
    /// real results, while ALSO emitting the existing `emit_symbol_query`
    /// row (additive observability) plus a new `SymbolGraphEvent` row
    /// (SymbolDslExecuted / DataflowImpactComputed) for the dashboard.
    /// Set via `with_symbol_graph_adapter` at CLI startup
    /// (`live_cli::build_local_knowledge_stack`).
    symbol_graph_adapter:
        Option<Arc<dyn elai_knowledge::symbol_graph_adapter::SymbolGraphAdapter>>,
    /// J-13 — process-level `ELAI_SEED` pin threaded from
    /// `DeterminismConfig::from_env()` at server construction
    /// (`runtime_builder.rs`). When `Some`, `handle_infer_request` forwards
    /// it on every `MessageRequest.seed` so same-seed `elai run --local`
    /// invocations reproduce token streams against a seeded sidecar. `None`
    /// preserves the pre-J-13 wire shape (`seed: None` on the wire).
    seed: Option<u64>,
}

impl ProductionAgentRpcServer {
    /// Construct a server with an already-built `AnthropicClient`.
    pub fn new(
        client: AnthropicClient,
        dispatcher: Arc<dyn ToolDispatcher>,
        model: String,
        max_tokens: u32,
    ) -> Self {
        Self {
            client,
            dispatcher,
            model,
            max_tokens,
            ssb: None,
            artifact_vm: None,
            system_prompt: None,
            pending_tool_args: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            clock: Arc::new(OsClock),
            symbol_graph_adapter: None,
            seed: None,
        }
    }

    /// C-15: builder-style setter for the `ClockProvider`. Production keeps the
    /// `OsClock` default; deterministic tests pass `SeededClock`.
    #[must_use]
    pub fn with_clock(mut self, clock: Arc<dyn ClockProvider>) -> Self {
        self.clock = clock;
        self
    }

    /// Builder-style setter for the SSB writer used by symbol-graph tool arms.
    /// Returns `self` so call sites can chain.
    pub fn with_ssb(mut self, ssb: Arc<crate::ssb::Ssb>) -> Self {
        self.ssb = Some(ssb);
        self
    }

    /// patch-R5 — builder-style setter for the system prompt forwarded on
    /// every `MessageRequest.system`. Production passes
    /// `DEFAULT_BABY_STEP_SYSTEM_PROMPT`; tests pass whatever string they
    /// want to assert on. Existing 4-arg `new()` call sites that don't
    /// chain `.with_system_prompt(...)` keep shipping `system: None`,
    /// preserving R3.1 wire behaviour.
    #[must_use]
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Builder-style setter for the VectorModule used by the `search_artifacts`
    /// dispatcher arm (B-35 Phase B). When not set, artifact search returns
    /// an empty result set (graceful degrade — no embedding available).
    pub fn with_artifact_vm(
        mut self,
        vm: Arc<dyn elai_knowledge::vector::VectorModule + Send + Sync>,
    ) -> Self {
        self.artifact_vm = Some(vm);
        self
    }

    /// E-40 / s-22600 Tranche B ITEM B.4 — builder-style setter for the
    /// SymbolGraph adapter used by the `TOOL_SYMBOL_DSL_QUERY` +
    /// `TOOL_SYMBOL_DATAFLOW_IMPACT` dispatcher arms. When not set, both
    /// arms return typed empty responses (pre-B.4 behaviour preserved).
    /// CLI startup chains this on the
    /// `ProductionAgentRpcServer::with_*` builder to plumb a real
    /// `SymbolGraphCacheAdapter` (orchestrator/src/symbol_graph/mod.rs).
    #[must_use]
    pub fn with_symbol_graph_adapter(
        mut self,
        adapter: Arc<dyn elai_knowledge::symbol_graph_adapter::SymbolGraphAdapter>,
    ) -> Self {
        self.symbol_graph_adapter = Some(adapter);
        self
    }

    /// J-13 — builder-style setter for the process `ELAI_SEED` pin forwarded
    /// on every `MessageRequest.seed`. Production reads
    /// `DeterminismConfig::from_env()` at server construction
    /// (`runtime_builder.rs`) and chains `.with_seed(pins.seed)`. Tests pass
    /// whatever u64 they want to assert on. Existing 4-arg `new()` call sites
    /// that don't chain `.with_seed(...)` keep shipping `seed: None`,
    /// preserving the pre-J-13 wire behaviour.
    #[must_use]
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// patch-R3 — Tool catalog published in `MessageRequest.tools` so Sonnet
    /// can decide which tool to call. Names are the wire IDs from
    /// `elai_ipc_types::TOOL_*` constants (NOT dispatcher method names).
    /// `input_schema` keys mirror the matching `elai_ipc_types::*Request`
    /// struct fields verbatim so `convert_json_input_to_postcard` can
    /// `serde_json::from_value::<*Request>` without rename overhead.
    ///
    /// Included with reduced schema (rationale per context-package):
    ///   - `TOOL_FILE_READ`: schema omits `offset`/`limit` because the
    ///     dispatcher arm at `translate_tool_request` rejects either being
    ///     `Some` (windowed reads unsupported).
    ///
    /// C-24 (DECISION-14): the 4 B-29 Phase 5 symbol-graph tools
    /// (`symbol_impact`, `symbol_flow`, `symbol_context`, `symbol_list`) are
    /// NOW INCLUDED. Their dispatcher arms at translate_tool_request
    /// (lines ~907-1072) are wired and `convert_json_input_to_postcard`
    /// also has matching arms — the agent can self-select them and the
    /// JSON->postcard bridge will succeed.
    ///
    /// Excluded entirely (would yield guaranteed-failure tool calls):
    ///   - `TOOL_INDEX_ARTIFACTS`: dispatcher arm returns `Err` (Phase C stub).
    ///   - `TOOL_GIT_UNDO`, `TOOL_GIT_BRANCH`: dispatcher arms return `Err`
    ///     ("not implemented in ProductionAgentRpcServer (B2c)").
    ///   - Artifact-store tools (`list_artifacts`, `search_artifacts`,
    ///     `remove_artifacts`, `entry_points`): deferred to a follow-up
    ///     catalog patch — their dispatcher arms exist but no matching
    ///     `convert_json_input_to_postcard` mapping yet.
    fn build_tool_catalog(&self) -> Vec<ToolDefinition> {
        let edit = ToolDefinition {
            name: TOOL_EDIT.to_string(),
            description: Some(
                "Apply a literal text replacement to a file. Requires a single \
                 exact match of `old_text` inside the file at `file_path`."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Absolute or repo-relative path to the file to edit."
                    },
                    "old_text": {
                        "type": "string",
                        "description": "Exact text fragment to be replaced (must occur once)."
                    },
                    "new_text": {
                        "type": "string",
                        "description": "Replacement text."
                    }
                },
                "required": ["file_path", "old_text", "new_text"]
            }),
            cache_control: None,
        };
        let file_read = ToolDefinition {
            name: TOOL_FILE_READ.to_string(),
            description: Some(
                "Read a file as an outline (signatures + imports for files larger \
                 than 300 lines, otherwise the full content). Windowed reads \
                 (offset/limit) are not supported by this dispatcher."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the file to read."
                    }
                },
                "required": ["file_path"]
            }),
            cache_control: None,
        };
        let validate = ToolDefinition {
            name: TOOL_VALIDATE.to_string(),
            description: Some(
                "Run the tiered validator (determinism / quality-AST / unit) for \
                 the project directory that contains `file_path`."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path inside the project directory to validate."
                    },
                    "language": {
                        "type": "string",
                        "description": "Source language (e.g. rust, python, typescript, go)."
                    }
                },
                "required": ["file_path", "language"]
            }),
            cache_control: None,
        };
        let compile_check = ToolDefinition {
            name: TOOL_COMPILE_CHECK.to_string(),
            description: Some(
                "Run an incremental compile check for the project directory that \
                 contains `file_path` (cargo check / tsc --noEmit / equivalent)."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path inside the project directory to check."
                    },
                    "language": {
                        "type": "string",
                        "description": "Source language (e.g. rust, python, typescript)."
                    },
                    "retry_round": {
                        "type": "integer",
                        "minimum": 0,
                        "maximum": 255,
                        "description": "Retry counter; 0 on first invocation."
                    }
                },
                "required": ["file_path", "language", "retry_round"]
            }),
            cache_control: None,
        };
        let classify = ToolDefinition {
            name: TOOL_CLASSIFY.to_string(),
            description: Some(
                "Classify a raw build / test failure string into one of the known \
                 failure categories. Returns `null` when no classifier matches."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "error_output": {
                        "type": "string",
                        "description": "Raw build / test failure output to classify."
                    },
                    "file_path": {
                        "type": ["string", "null"],
                        "description": "Optional source file the failure refers to."
                    }
                },
                "required": ["error_output"]
            }),
            cache_control: None,
        };
        let git_commit = ToolDefinition {
            name: TOOL_GIT_COMMIT.to_string(),
            description: Some(
                "Commit the supplied paths under the active per-plan branch. \
                 Requires at least one path."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Commit message."
                    },
                    "paths": {
                        "type": "array",
                        "items": {"type": "string"},
                        "minItems": 1,
                        "description": "Repo-relative paths to stage and commit."
                    }
                },
                "required": ["message", "paths"]
            }),
            cache_control: None,
        };
        // C-24 (DECISION-1): the 4 B-29 Phase 5 symbol-graph tools.
        // Schema field names mirror the wire structs in
        // `elai_ipc_types::Symbol*Request` (lines 542-571) verbatim so
        // `convert_json_input_to_postcard` can `serde_json::from_value`
        // without rename overhead — same patch-R3 contract as the 6
        // tools above.
        //
        // `project_root` is required on every request — the dispatcher arms
        // (translate_tool_request L907-1072) Err immediately if the file_name
        // of the path is empty, because that's how `project_id` is derived
        // before opening the SymbolGraphCache. The agent's system prompt
        // surface (DEFAULT_BABY_STEP_SYSTEM_PROMPT) does not currently
        // inject `project_root`; the spawned agent is expected to mirror
        // the absolute project root surfaced via the task brief.
        let symbol_impact = ToolDefinition {
            name: TOOL_SYMBOL_IMPACT.to_string(),
            description: Some(
                "Compute the reverse-dependency blast radius of a symbol (BFS \
                 over the symbol call-graph). Returns the set of files that \
                 transitively call `target`, partitioned by hop distance up \
                 to `depth`."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_root": {
                        "type": "string",
                        "description": "Absolute path to the project root \
                         (used to derive project_id + locate the symbol-graph DB)."
                    },
                    "target": {
                        "type": "string",
                        "description": "Qualified symbol name to start the reverse traversal at."
                    },
                    "depth": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 16,
                        "description": "BFS depth cap (recommended default: 2)."
                    }
                },
                "required": ["project_root", "target", "depth"]
            }),
            cache_control: None,
        };
        let symbol_flow = ToolDefinition {
            name: TOOL_SYMBOL_FLOW.to_string(),
            description: Some(
                "Compute the forward call-flow from an entrypoint symbol (DFS \
                 over the symbol call-graph). Returns the call tree rooted at \
                 `entrypoint_id` truncated to `depth` levels."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_root": {
                        "type": "string",
                        "description": "Absolute path to the project root \
                         (used to derive project_id + locate the symbol-graph DB)."
                    },
                    "entrypoint_id": {
                        "type": "string",
                        "description": "Stable symbol id (typically qualified symbol name) \
                         to start the call-flow traversal at."
                    },
                    "depth": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 16,
                        "description": "DFS depth cap (recommended default: 5)."
                    }
                },
                "required": ["project_root", "entrypoint_id", "depth"]
            }),
            cache_control: None,
        };
        let symbol_context = ToolDefinition {
            name: TOOL_SYMBOL_CONTEXT.to_string(),
            description: Some(
                "Return a 360-degree view of a symbol definition: matching \
                 symbol(s) (by simple or qualified name) with file/line, kind, \
                 and a short callers/callees neighbourhood. Useful before edits \
                 to a symbol whose call-site count you do not yet know."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_root": {
                        "type": "string",
                        "description": "Absolute path to the project root."
                    },
                    "name": {
                        "type": "string",
                        "description": "Symbol name (simple or qualified) to look up."
                    },
                    "file_hint": {
                        "type": ["string", "null"],
                        "description": "Optional file path to disambiguate when multiple \
                         symbols share the same name."
                    }
                },
                "required": ["project_root", "name"]
            }),
            cache_control: None,
        };
        let symbol_list = ToolDefinition {
            name: TOOL_SYMBOL_LIST.to_string(),
            description: Some(
                "List symbols defined in the project, optionally filtered by \
                 file and/or fuzzy query string. Use when planning before edits \
                 to know which symbols already exist. Server caps `limit` at 500."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_root": {
                        "type": "string",
                        "description": "Absolute path to the project root."
                    },
                    "file": {
                        "type": ["string", "null"],
                        "description": "Optional file path to scope the listing."
                    },
                    "query": {
                        "type": ["string", "null"],
                        "description": "Optional substring filter on symbol name."
                    },
                    "limit": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 500,
                        "description": "Result cap (server clamps at 500)."
                    }
                },
                "required": ["project_root", "limit"]
            }),
            cache_control: None,
        };
        // C-30: four new DKG query tools (callers / dependents / data_flow /
        // subgraph). Input schema mirrors the helper signatures on
        // `DkgQuerierAdapter`: `target` (string) + `max_depth` (u8) for
        // callers/dependents/data_flow; `root` (string) + `depth` (u8) for
        // subgraph. `project_root` is included so the dispatcher arm
        // (translate_tool_request) can locate the per-project DKG db.
        let dkg_callers = ToolDefinition {
            name: TOOL_DKG_CALLERS.to_string(),
            description: Some(
                "Return the symbols that CALL `target` (inbound CALLS edges \
                 in the dependency knowledge graph). Useful for change-impact \
                 analysis before editing a function. Depth is capped at 8."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_root": {
                        "type": "string",
                        "description": "Absolute path to the project root \
                         (used to locate the per-project DKG database)."
                    },
                    "target": {
                        "type": "string",
                        "description": "Qualified symbol name to start inbound traversal at."
                    },
                    "max_depth": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 8,
                        "description": "BFS depth cap (recommended default: 3)."
                    }
                },
                "required": ["project_root", "target", "max_depth"]
            }),
            cache_control: None,
        };
        let dkg_dependents = ToolDefinition {
            name: TOOL_DKG_DEPENDENTS.to_string(),
            description: Some(
                "Return the symbols / files that `target` depends on (outbound \
                 IMPORTS / USES edges). Use when planning to refactor a module \
                 to see what it touches transitively. Depth is capped at 8."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_root": {
                        "type": "string",
                        "description": "Absolute path to the project root."
                    },
                    "target": {
                        "type": "string",
                        "description": "Qualified symbol name to start outbound traversal at."
                    },
                    "max_depth": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 8,
                        "description": "BFS depth cap (recommended default: 3)."
                    }
                },
                "required": ["project_root", "target", "max_depth"]
            }),
            cache_control: None,
        };
        let dkg_data_flow = ToolDefinition {
            name: TOOL_DKG_DATA_FLOW.to_string(),
            description: Some(
                "Return the data-flow edges (PARAMETER / RETURN / ASSIGNMENT / \
                 FIELD_ACCESS) reachable from `target`. Each result is a \
                 (source, target, variable, flow_type) tuple. Depth is capped \
                 at 8."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_root": {
                        "type": "string",
                        "description": "Absolute path to the project root."
                    },
                    "target": {
                        "type": "string",
                        "description": "Symbol to start data-flow traversal at."
                    },
                    "max_depth": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 8,
                        "description": "BFS depth cap (recommended default: 3)."
                    }
                },
                "required": ["project_root", "target", "max_depth"]
            }),
            cache_control: None,
        };
        let dkg_subgraph = ToolDefinition {
            name: TOOL_DKG_SUBGRAPH.to_string(),
            description: Some(
                "BFS-extract the subgraph rooted at `root` up to `depth` hops. \
                 Returns nodes + edges for compact agent-context rendering. \
                 Depth is capped at 8."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_root": {
                        "type": "string",
                        "description": "Absolute path to the project root."
                    },
                    "root": {
                        "type": "string",
                        "description": "Symbol to seed the BFS at."
                    },
                    "depth": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 8,
                        "description": "BFS depth cap (recommended default: 2)."
                    }
                },
                "required": ["project_root", "root", "depth"]
            }),
            cache_control: None,
        };
        // E-40 Tranche B-2: SymbolGraph DSL + dataflow tools. Distinct
        // from the B-29 symbol_* family above (those query the
        // SQLite-rooted cache; these query the orchestrator-rooted
        // JSON cache via the new SymbolGraphAdapter). Mirrors the
        // TOOL_SYMBOL_IMPACT input-schema shape (project_root + bounded
        // depth) plus a `dsl` / `symbol` field per tool.
        let symbol_dsl_query = ToolDefinition {
            name: TOOL_SYMBOL_DSL_QUERY.to_string(),
            description: Some(
                "Execute a Datalog-style DSL query (DEF/USE/IMPL/CALL \
                 selectors with lang/file/depth filters) over the \
                 orchestrator-rooted JSON symbol graph. Returns matching \
                 symbols plus any dataflow side-context edges, bounded by \
                 `depth_cap` (default 2)."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_root": {
                        "type": "string",
                        "description": "Absolute path to the project root."
                    },
                    "dsl": {
                        "type": "string",
                        "description": "DSL query string (e.g. DEF(foo) | lang(rust))."
                    },
                    "depth_cap": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 5,
                        "description": "Dataflow side-context hop cap (default 2)."
                    }
                },
                "required": ["project_root", "dsl", "depth_cap"]
            }),
            cache_control: None,
        };
        let symbol_dataflow_impact = ToolDefinition {
            name: TOOL_SYMBOL_DATAFLOW_IMPACT.to_string(),
            description: Some(
                "Compute the dataflow / reverse-call impact radius of a \
                 symbol via BFS over the JSON symbol graph's reverse-call \
                 index, bounded by `depth_cap` (default 2). Returns the set \
                 of transitively-impacted symbol names plus the actual \
                 depth reached and edge count."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_root": {
                        "type": "string",
                        "description": "Absolute path to the project root."
                    },
                    "symbol": {
                        "type": "string",
                        "description": "Symbol name to start the reverse traversal at."
                    },
                    "depth_cap": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 5,
                        "description": "BFS depth cap (default 2)."
                    }
                },
                "required": ["project_root", "symbol", "depth_cap"]
            }),
            cache_control: None,
        };
        // H-13: canvas op tools (model drives the canvas). cache_control: None
        // (inert-by-design per C2; apply_anthropic_cache_markers re-stamps at
        // send_message time). 7 tools; all 3 arms (catalog+convert+dispatch).
        let canvas_create_node = ToolDefinition {
            name: TOOL_CANVAS_CREATE_NODE.to_string(),
            description: Some(
                "Create a new node on the active canvas. Allowed modes: arch, code. \
                 node_id must be unique in the current graph."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "node_id": { "type": "string", "minLength": 1,
                                 "description": "Unique node id (must not already exist)." },
                    "label":   { "type": "string", "description": "Human-readable label." },
                    "mode":    { "type": "string", "description": "Canvas mode (arch or code)." },
                    "x":       { "type": "number", "description": "Optional x-coord 0-1 normalised." },
                    "y":       { "type": "number", "description": "Optional y-coord 0-1 normalised." },
                    "meta":    { "description": "Optional metadata JSON object." },
                    "task_id": { "type": "string", "description": "Triggering task id." }
                },
                "required": ["node_id", "label", "mode", "task_id"]
            }),
            cache_control: None,
        };
        let canvas_connect = ToolDefinition {
            name: TOOL_CANVAS_CONNECT.to_string(),
            description: Some(
                "Connect two existing canvas nodes with a directed edge. \
                 Both from_id and to_id must already exist in the graph."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "from_id": { "type": "string", "minLength": 1,
                                 "description": "Source node id (must exist)." },
                    "to_id":   { "type": "string", "minLength": 1,
                                 "description": "Target node id (must exist)." },
                    "label":   { "type": ["string", "null"],
                                 "description": "Optional edge label." },
                    "mode":    { "type": "string", "description": "Canvas mode (arch or code)." },
                    "task_id": { "type": "string", "description": "Triggering task id." }
                },
                "required": ["from_id", "to_id", "mode", "task_id"]
            }),
            cache_control: None,
        };
        let canvas_update_node = ToolDefinition {
            name: TOOL_CANVAS_UPDATE_NODE.to_string(),
            description: Some(
                "Update the label or metadata of an existing canvas node. \
                 node_id must exist in the graph."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "node_id": { "type": "string", "minLength": 1,
                                 "description": "Node id to update (must exist)." },
                    "label":   { "type": ["string", "null"],
                                 "description": "New label; omit to leave unchanged." },
                    "meta":    { "description": "Metadata patch merged into existing." },
                    "mode":    { "type": "string",
                                 "description": "Canvas mode (arch, code, or plan)." },
                    "task_id": { "type": "string", "description": "Triggering task id." }
                },
                "required": ["node_id", "mode", "task_id"]
            }),
            cache_control: None,
        };
        let canvas_annotate = ToolDefinition {
            name: TOOL_CANVAS_ANNOTATE.to_string(),
            description: Some(
                "Add a text annotation to the canvas, optionally anchored to a node."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "text":           { "type": "string", "minLength": 1,
                                        "description": "Annotation text." },
                    "anchor_node_id": { "type": ["string", "null"],
                                        "description": "Node to anchor to; null for free-floating." },
                    "mode":           { "type": "string",
                                        "description": "Canvas mode." },
                    "task_id":        { "type": "string", "description": "Triggering task id." }
                },
                "required": ["text", "mode", "task_id"]
            }),
            cache_control: None,
        };
        let canvas_fork = ToolDefinition {
            name: TOOL_CANVAS_FORK.to_string(),
            description: Some(
                "Fork an existing node into a new sibling node sharing the same parent edges."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "source_node_id": { "type": "string", "minLength": 1,
                                        "description": "Node to fork (must exist)." },
                    "new_node_id":    { "type": "string", "minLength": 1,
                                        "description": "Id for the forked node (must not exist)." },
                    "label":          { "type": "string", "description": "Label for the forked node." },
                    "mode":           { "type": "string",
                                        "description": "Canvas mode (arch or code)." },
                    "task_id":        { "type": "string", "description": "Triggering task id." }
                },
                "required": ["source_node_id", "new_node_id", "label", "mode", "task_id"]
            }),
            cache_control: None,
        };
        let canvas_render_diagram = ToolDefinition {
            name: TOOL_CANVAS_RENDER_DIAGRAM.to_string(),
            description: Some(
                "Render the current graph as a structured architecture diagram in ARCHITECTURE mode. \
                 Reuses the server-side export_graph_mermaid path for layout data. \
                 Emits CanvasOpKind::LayoutChanged rows. Allowed mode: arch only."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "graph_mode": { "type": "string",
                                    "enum": ["symbol", "file", "decomposition"],
                                    "description": "Graph data mode for layout." },
                    "max_nodes":  { "type": ["integer", "null"], "minimum": 1, "maximum": 200,
                                    "description": "Max nodes; server default when omitted." },
                    "mode":       { "type": "string", "const": "arch",
                                    "description": "Must be arch." },
                    "task_id":    { "type": "string", "description": "Triggering task id." }
                },
                "required": ["graph_mode", "mode", "task_id"]
            }),
            cache_control: None,
        };
        let canvas_screenshot = ToolDefinition {
            name: TOOL_CANVAS_SCREENSHOT.to_string(),
            description: Some(
                "Capture the canvas as a PNG. Emits CanvasOpKind::ScreenshotCaptured SSB row. \
                 Actual pixel capture (canvas.toDataURL) is Tier-2-pending (stub-6); \
                 the Tauri capture API is not yet available in this tier. \
                 VLM/media_validator scoring is also Tier-2-pending (RUNPOD-PENDING). \
                 This call always emits the SSB row; the capture itself returns an Err reason \
                 until the Tier-2 screenshot path is wired."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "mode":    { "type": "string", "description": "Canvas mode at capture time." },
                    "task_id": { "type": "string", "description": "Triggering task id." }
                },
                "required": ["mode", "task_id"]
            }),
            cache_control: None,
        };

        // H-15: spawn_view reply-channel verb (1 tool).
        let canvas_spawn_view = ToolDefinition {
            name: TOOL_CANVAS_SPAWN_VIEW.to_string(),
            description: Some(
                "Spawn a new canvas view of the given view_type, seeded by existing \
                 canvas op ids. view_type selects a registered renderer \
                 (e.g. 'comparison_board'). Emits a CanvasViewEvent(ViewSpawned) SSB row. \
                 Default gated behind routing-confidence; suppress-canvas hatch applies."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "view_type": {
                        "type": "string",
                        "description": "Renderer key (e.g. 'comparison_board')."
                    },
                    "seed_ops": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Op ids (canvas_op_applied) to seed the view's graph."
                    },
                    "mode": {
                        "type": "string",
                        "description": "Canvas mode at spawn time."
                    },
                    "task_id": {
                        "type": "string",
                        "description": "Triggering task id."
                    }
                },
                "required": ["view_type", "seed_ops", "mode", "task_id"]
            }),
            cache_control: None,
        };

        // I-13: TraceQuery on-demand line-range expansion tool (3-arm IS-14 pattern).
        // The model calls this to expand an exact line range from disk when to_llm_content
        // produced a condensed form that omitted needed detail.
        // Dispatcher arm emits MemoryGuardEvent(TraceQueryExpanded) SSB row.
        let trace_query = ToolDefinition {
            name: TOOL_TRACE_QUERY.to_string(),
            description: Some(
                "Expand an exact line range from a file on disk on demand. \
                 Use when a condensed tool result omitted lines you need to see. \
                 Returns the exact lines start_line..=end_line (1-indexed, inclusive) \
                 with total_lines for context. Lines beyond EOF are silently truncated."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Absolute or repo-relative path to the file to read."
                    },
                    "start_line": {
                        "type": "integer",
                        "minimum": 1,
                        "description": "First line to return (1-indexed, inclusive)."
                    },
                    "end_line": {
                        "type": "integer",
                        "minimum": 1,
                        "description": "Last line to return (1-indexed, inclusive, >= start_line)."
                    }
                },
                "required": ["file_path", "start_line", "end_line"]
            }),
            cache_control: None,
        };

        // I-8: voluntary abort tool (aletheia — 3-arm IS-14 pattern).
        // Calling this tool declares the current trajectory infeasible.
        // The dispatcher arm emits TrajectoryAborted(Voluntary) and returns
        // Err(reason) -> ToolOutcome::Failed so the task runner surfaces the
        // abort.  Must be listed in the catalog so the agent can call it.
        let acknowledge_infeasible = ToolDefinition {
            name: TOOL_ACKNOWLEDGE_INFEASIBLE.to_string(),
            description: Some(
                "Declare the current trajectory infeasible and voluntarily abort. \
                 Use when the task cannot be completed with the current approach \
                 and continuing would spiral. Records a TrajectoryAborted(Voluntary) \
                 beacon so the dead-end cache prunes isomorphic re-attempts."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "reason": {
                        "type": "string",
                        "description": "Human-readable explanation of why the trajectory is infeasible."
                    },
                    "trajectory_id": {
                        "type": "string",
                        "description": "Optional trajectory identifier for SSB correlation."
                    }
                },
                "required": ["reason"]
            }),
            cache_control: None,
        };

        // I-53: recipe runners + utility tool pack.
        // NOTE: these tools are dispatched via the JSON path in translate_tool_request
        // (serde_json direct parse, no postcard wire struct — see DECISION-7 in
        // workflow/reasoning/phase-I53-decisions.md). Both-catalogs registration
        // required per I-53 WIRING_CONTRACT ("reachable_from: production agent tool dispatch").
        let recipe_list = ToolDefinition {
            name: "recipe_list".to_string(),
            description: Some(
                "Discover and list build recipes (Make/Just/Cargo targets) in the \
                 current workspace. Returns all runners detected in the given directory."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "cwd": {
                        "type": "string",
                        "description": "Workspace directory to probe for build manifests."
                    }
                },
                "additionalProperties": false
            }),
            cache_control: None,
        };
        let recipe_run = ToolDefinition {
            name: "recipe_run".to_string(),
            description: Some(
                "Run a named build recipe (e.g. `just build`, `make clean`, \
                 `cargo run --package foo --bin bar`). Command is validated through \
                 the bash validation pipeline; blocked commands are never spawned."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Full recipe command to execute."
                    },
                    "cwd": {
                        "type": "string",
                        "description": "Working directory for the recipe."
                    }
                },
                "required": ["command"],
                "additionalProperties": false
            }),
            cache_control: None,
        };
        let archive_read = ToolDefinition {
            name: "archive_read".to_string(),
            description: Some(
                "List or read entries from a plain .tar archive without extracting to disk. \
                 Path traversal is blocked. action=list returns entry metadata; \
                 action=read returns entry content."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path to the .tar archive." },
                    "action": {
                        "type": "string",
                        "enum": ["list", "read"],
                        "description": "`list` to enumerate entries; `read` to fetch content."
                    },
                    "entry": {
                        "type": "string",
                        "description": "Archive-relative entry path (required when action=read)."
                    }
                },
                "required": ["path", "action"],
                "additionalProperties": false
            }),
            cache_control: None,
        };
        let json_tree_query = ToolDefinition {
            name: "json_tree_query".to_string(),
            description: Some(
                "Navigate a JSON file via a dot-path and render a bounded subtree \
                 (depth-capped, line-capped). Prevents large JSON files from flooding context."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file": { "type": "string", "description": "Path to the JSON file." },
                    "path": {
                        "type": "string",
                        "description": "Dot-separated navigation path (e.g. `a.b.c`)."
                    },
                    "max_depth": {
                        "type": "integer",
                        "minimum": 1,
                        "description": "Max render depth (default 2)."
                    },
                    "max_lines": {
                        "type": "integer",
                        "minimum": 1,
                        "description": "Max output lines (default 6)."
                    }
                },
                "required": ["file"],
                "additionalProperties": false
            }),
            cache_control: None,
        };
        // I-52: agent-callable checkpoint tool (3-arm save/rewind/diff) over the
        // H-49 time-travel CAS substrate. JSON dispatch path (no postcard struct),
        // mirroring the I-53 recipe_* tools. session_id is run-context derived
        // (never agent-supplied) -- the cross-session rewind guard.
        let checkpoint_save = ToolDefinition {
            name: TOOL_CHECKPOINT_SAVE.to_string(),
            description: Some(
                "Save a labelled checkpoint of the named files (a cheap content-\
                 addressed snapshot) so you can speculatively edit and rewind \
                 inside this task. Scoped to the current session."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "label": {
                        "type": "string",
                        "description": "Human label to save the checkpoint under (reused later to rewind/diff)."
                    },
                    "paths": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Project-relative file paths to capture into the checkpoint."
                    }
                },
                "required": ["label", "paths"],
                "additionalProperties": false
            }),
            cache_control: None,
        };
        let checkpoint_rewind = ToolDefinition {
            name: TOOL_CHECKPOINT_REWIND.to_string(),
            description: Some(
                "Rewind the working tree to a previously saved checkpoint label. \
                 REFUSES (no file is overwritten) if any covered file changed \
                 outside the checkpointed set since the save -- the diverged \
                 paths are named in the response. Scoped to the current session."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "label": {
                        "type": "string",
                        "description": "The checkpoint label to rewind to (from a prior checkpoint_save)."
                    }
                },
                "required": ["label"],
                "additionalProperties": false
            }),
            cache_control: None,
        };
        let checkpoint_diff = ToolDefinition {
            name: TOOL_CHECKPOINT_DIFF.to_string(),
            description: Some(
                "List the files that have changed since a saved checkpoint label \
                 (read-only). Scoped to the current session."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "label": {
                        "type": "string",
                        "description": "The checkpoint label to diff the working tree against."
                    }
                },
                "required": ["label"],
                "additionalProperties": false
            }),
            cache_control: None,
        };

        // I-54: agent-callable structural (AST-aware) edit tool (search/preview/
        // apply over the shipped tree-sitter grammars; one version source via
        // elai_knowledge::TreeSitterAstEvaluator; ast-grep dep BANNED, SLC-3).
        // JSON dispatch path (no postcard struct), mirroring I-52/I-53. SHIP-NOW
        // languages: rust / python / typescript. Syntactic only (semantic rename
        // is E-52's LSP lane). Apply rides the SHIPPED PreToolUse edit gates.
        let structural_search = ToolDefinition {
            name: TOOL_STRUCTURAL_SEARCH.to_string(),
            description: Some(
                "Search source code structurally with a tree-sitter query (the \
                 grammar's native pattern language) instead of a text/regex match. \
                 Returns every matching node with its file line range and the query \
                 capture it satisfied. Read-only. Languages: rust, python, typescript."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "language": {
                        "type": "string",
                        "enum": ["rust", "python", "typescript"],
                        "description": "Source language of the file/source."
                    },
                    "source": {
                        "type": "string",
                        "description": "The source code to search."
                    },
                    "query": {
                        "type": "string",
                        "description": "A tree-sitter query, e.g. '(function_item name: (identifier) @name)'."
                    }
                },
                "required": ["language", "source", "query"],
                "additionalProperties": false
            }),
            cache_control: None,
        };
        let structural_preview = ToolDefinition {
            name: TOOL_STRUCTURAL_PREVIEW.to_string(),
            description: Some(
                "Preview a structural rewrite: a tree-sitter query selects nodes \
                 and a rewrite template (referencing query captures by @name) \
                 produces the would-be edit set. Returns the per-match diff and a \
                 stable diff hash WITHOUT writing any bytes. A broken query or \
                 template is reported here (no edit happens)."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "language": {
                        "type": "string",
                        "enum": ["rust", "python", "typescript"],
                        "description": "Source language of the file."
                    },
                    "file": {
                        "type": "string",
                        "description": "Project-relative path of the file to preview the edit against."
                    },
                    "query": {
                        "type": "string",
                        "description": "A tree-sitter query selecting the nodes to rewrite."
                    },
                    "rewrite_template": {
                        "type": "string",
                        "description": "Replacement text for each matched node; @name substitutes a query capture."
                    },
                    "whole_node_capture": {
                        "type": "string",
                        "description": "The capture name whose node is the one replaced (e.g. 'name')."
                    }
                },
                "required": ["language", "file", "query", "rewrite_template", "whole_node_capture"],
                "additionalProperties": false
            }),
            cache_control: None,
        };
        let structural_apply = ToolDefinition {
            name: TOOL_STRUCTURAL_APPLY.to_string(),
            description: Some(
                "Apply a structural rewrite previewed with structural_preview. \
                 REFUSES (no bytes written) if the file changed since the preview: \
                 pass back the preview's `source_hash` as `expected_source_hash` \
                 and the apply re-reads the file, re-hashes it, and refuses on any \
                 difference (a stale match), even an edit that left the matched \
                 node text intact. Routes every write through the same edit safety \
                 gates as the Edit tool. Languages: rust, python, typescript."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "language": {
                        "type": "string",
                        "enum": ["rust", "python", "typescript"],
                        "description": "Source language of the file."
                    },
                    "file": {
                        "type": "string",
                        "description": "Project-relative path of the file to edit."
                    },
                    "query": {
                        "type": "string",
                        "description": "The same tree-sitter query used in the preview."
                    },
                    "rewrite_template": {
                        "type": "string",
                        "description": "The same rewrite template used in the preview."
                    },
                    "whole_node_capture": {
                        "type": "string",
                        "description": "The same whole-node capture name used in the preview."
                    },
                    "expected_source_hash": {
                        "type": "string",
                        "description": "The `source_hash` returned by the matching structural_preview. The apply refuses if the file no longer hashes to this (changed since the preview)."
                    }
                },
                "required": ["language", "file", "query", "rewrite_template", "whole_node_capture", "expected_source_hash"],
                "additionalProperties": false
            }),
            cache_control: None,
        };

        vec![
            edit, file_read, validate, compile_check, classify, git_commit,
            // C-24: B-29 Phase 5 symbol-graph tools, now agent-visible.
            symbol_impact, symbol_flow, symbol_context, symbol_list,
            // C-30: dependency-knowledge-graph query tools, now agent-visible.
            dkg_callers, dkg_dependents, dkg_data_flow, dkg_subgraph,
            // E-40 Tranche B-2: SymbolGraph DSL + dataflow tools.
            symbol_dsl_query, symbol_dataflow_impact,
            // H-13: canvas op tools (model drives the canvas). 7 tools.
            canvas_create_node, canvas_connect, canvas_update_node, canvas_annotate,
            canvas_fork, canvas_render_diagram, canvas_screenshot,
            // H-15: spawn_view reply-channel verb. 1 tool.
            canvas_spawn_view,
            // I-8: voluntary abort verb.
            acknowledge_infeasible,
            // I-13: on-demand line-range expansion tool.
            trace_query,
            // I-53: recipe runners + utility tool pack. 4 tools.
            recipe_list, recipe_run, archive_read, json_tree_query,
            // I-52: agent-callable checkpoint tool. 3 tools (save/rewind/diff).
            checkpoint_save, checkpoint_rewind, checkpoint_diff,
            // I-54: agent-callable structural (AST-aware) edit tool. 3 tools
            // (search/preview/apply over the shipped tree-sitter grammars).
            structural_search, structural_preview, structural_apply,
        ]
    }

    /// Construct from environment variables, mirroring the `task_runner.rs`
    /// `ELAI_GROUNDING_BACKEND` switch (L4133):
    ///   - `Some("cloud")` -> `AnthropicClient::from_env()`.
    ///   - Any other value -> sidecar default.
    ///
    /// Returns `None` when the cloud path is requested but credentials are
    /// missing; the caller should fall back to `NoOpAgentRpcServer`.
    pub fn from_env(dispatcher: Arc<dyn ToolDispatcher>) -> Option<Self> {
        let backend = std::env::var("ELAI_GROUNDING_BACKEND").ok();
        let client = if backend.as_deref() == Some("cloud") {
            match AnthropicClient::from_env() {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "ProductionAgentRpcServer: cloud credentials unavailable; \
                         caller should fall back to NoOpAgentRpcServer"
                    );
                    return None;
                }
            }
        } else {
            let base_url = std::env::var("ANTHROPIC_BASE_URL")
                .unwrap_or_else(|_| DEFAULT_SIDECAR_URL.to_string());
            AnthropicClient::new("local-dummy").with_base_url(base_url)
        };

        let model = std::env::var("ELAI_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());
        let max_tokens = std::env::var("ELAI_MAX_TOKENS")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(DEFAULT_MAX_TOKENS);

        Some(Self::new(client, dispatcher, model, max_tokens))
    }
}

// ---------------------------------------------------------------------------
// AgentRpcServer implementation
// ---------------------------------------------------------------------------

impl AgentRpcServer for ProductionAgentRpcServer {
    fn handle_infer_request<'a>(
        &'a self,
        prompt: String,
        max_tokens: u32,
    ) -> Pin<Box<dyn std::future::Future<Output = InferOutcome> + Send + 'a>> {
        Box::pin(async move {
            // Use the frame-supplied max_tokens; fall back to server default
            // when the frame carries 0 (not yet set by caller).
            let effective_max_tokens = if max_tokens == 0 {
                self.max_tokens
            } else {
                max_tokens
            };
            // J-1 — perception gateway (scaffold T2 + router T3). Wire-dark
            // by default (ELAI_PERCEPTION_GATEWAY_ENABLE=1 to enable; GD-4).
            // Runs AFTER effective_max_tokens compute (last cache breakpoint,
            // HD-2) and BEFORE MessageRequest construction — the prompt is the
            // post-assembly volatile tail, so the gate never busts the
            // byte-stable prefix. Router gate runs FIRST so
            // RouteToPixelDiff / TypedRefusal short-circuit (no scaffold call
            // needed on those arms — plan stub_ban: visual-diffs NEVER reach
            // the scaffold's VLM calls).
            if perception::perception_gateway_enabled() {
                let router_input = perception::RouterInput {
                    router_type: perception::classify_prompt(&prompt),
                    has_image: perception::prompt_carries_image(&prompt),
                    image_path: None,
                    reference_path: None,
                    image_count: if perception::prompt_carries_image(&prompt) { 1 } else { 0 },
                    micro_text_confidence: None,
                };
                let outcome = perception::route_outcome(&router_input);
                match outcome.decision {
                    perception::RouterDecision::RouteToPixelDiff => {
                        emit_perception_scaffold_event(
                            self.ssb.as_ref(),
                            &self.clock,
                            crate::ssb::PerceptionScaffoldEventKind::RoutedToSpecialist,
                            None,
                            serde_json::json!({
                                "router_type": "visual_diff",
                                "kind": outcome.kind,
                                "image_path_present": router_input.image_path.is_some(),
                                "reference_path_present": router_input.reference_path.is_some(),
                                "residue": outcome.residue,
                            }),
                        )
                        .await;
                        if let (Some(img_path), Some(ref_path)) =
                            (router_input.image_path.as_ref(), router_input.reference_path.as_ref())
                        {
                            let req = MediaValidationRequest {
                                media_type: MediaType::Image,
                                file_path: img_path.clone(),
                                quality_spec: QualitySpec::default(),
                                reference_path: Some(ref_path.clone()),
                                mode: ValidationMode::Quick,
                                output_dir: None,
                                request_id: uuid::Uuid::new_v4(),
                            };
                            let tier1 = ImageValidator::default();
                            let result = tier1
                                .validate(&req, ToolCapabilityFlags::empty())
                                .await;
                            if !result.pass {
                                return InferOutcome::Failed(
                                    "perception_gateway: visual-diff pixel-diff veto (Tier-1 fail)"
                                        .to_string(),
                                );
                            }
                        }
                    }
                    perception::RouterDecision::RouteToTiledComposite => {
                        emit_perception_scaffold_event(
                            self.ssb.as_ref(),
                            &self.clock,
                            crate::ssb::PerceptionScaffoldEventKind::RouteRuleApplied,
                            None,
                            serde_json::json!({
                                "kind": outcome.kind,
                                "image_count": router_input.image_count,
                                "residue": outcome.residue,
                            }),
                        )
                        .await;
                    }
                    perception::RouterDecision::TypedRefusal => {
                        let _refusal = RefusalDecision::Abstain;
                        return InferOutcome::Failed(
                            "perception_gateway: typed refusal (counting over hard limit, RefusalDecision::Abstain reason=None)"
                                .to_string(),
                        );
                    }
                    perception::RouterDecision::DegradedPassthrough => {
                        emit_perception_scaffold_event(
                            self.ssb.as_ref(),
                            &self.clock,
                            crate::ssb::PerceptionScaffoldEventKind::DegradedPassthrough,
                            None,
                            serde_json::json!({
                                "source": "router",
                                "reason": "router_degraded_passthrough",
                                "kind": outcome.kind,
                                "residue": outcome.residue,
                            }),
                        )
                        .await;
                    }
                    _ => {
                    }
                }
                // J-1.T2 — scaffold gate (two-call flow structure, T5 wires
                // the live sidecar calls). Logged-only at this leg — errors
                // degrade to passthrough (never a silent fallthrough).
                // J-1.T4 — emits ScaffoldEmitted on Ok(ScaffoldEmitted),
                // DegradedPassthrough on Ok(DegradedPassthrough) + Err.
                // NoImagePresent is a no-op (scaffold fires ONLY on
                // image-bearing dispatches, plan line 63).
                let scaffold_input = perception::ScaffoldInput::from_prompt(&prompt);
                match perception::scaffold(&scaffold_input) {
                    Ok(result) => match result.outcome {
                        perception::ScaffoldOutcome::ScaffoldEmitted => {
                            emit_perception_scaffold_event(
                                self.ssb.as_ref(),
                                &self.clock,
                                crate::ssb::PerceptionScaffoldEventKind::SchemaApplied,
                                None,
                                serde_json::json!({
                                    "schema": "v2",
                                    "literal_element_count": result.literal_elements.len(),
                                    "call1_temperature": result.call1_temperature,
                                }),
                            )
                            .await;
                        }
                        perception::ScaffoldOutcome::DegradedPassthrough => {
                            emit_perception_scaffold_event(
                                self.ssb.as_ref(),
                                &self.clock,
                                crate::ssb::PerceptionScaffoldEventKind::DegradedPassthrough,
                                None,
                                serde_json::json!({
                                    "source": "scaffold",
                                    "reason": "degraded_passthrough",
                                }),
                            )
                            .await;
                        }
                        perception::ScaffoldOutcome::NoImagePresent => {
                            // No-op: scaffold fires ONLY on image-bearing
                            // dispatches (plan line 63). No SSB emit.
                        }
                        perception::ScaffoldOutcome::SchemaReprompt => {
                            emit_perception_scaffold_event(
                                self.ssb.as_ref(),
                                &self.clock,
                                crate::ssb::PerceptionScaffoldEventKind::DegradedPassthrough,
                                None,
                                serde_json::json!({
                                    "source": "scaffold",
                                    "reason": "schema_v2_reprompt",
                                }),
                            )
                            .await;
                        }
                        perception::ScaffoldOutcome::TypedRefusal => {
                            emit_perception_scaffold_event(
                                self.ssb.as_ref(),
                                &self.clock,
                                crate::ssb::PerceptionScaffoldEventKind::DegradedPassthrough,
                                None,
                                serde_json::json!({
                                    "source": "scaffold",
                                    "reason": "schema_v2_typed_refusal",
                                }),
                            )
                            .await;
                        }
                    },
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "perception_gateway: scaffold failed (degraded to passthrough)"
                        );
                        emit_perception_scaffold_event(
                            self.ssb.as_ref(),
                            &self.clock,
                            crate::ssb::PerceptionScaffoldEventKind::DegradedPassthrough,
                            None,
                            serde_json::json!({
                                "source": "scaffold",
                                "reason": "scaffold_error",
                            }),
                        )
                        .await;
                    }
                }
            }
            // patch-R5 — wire system_prompt (when set) as a one-block
            // `SystemBlock` array, and pin `tool_choice` to `Auto`. R3 always
            // publishes the 6-tool catalog, so `tool_choice` must always be
            // explicit; `Auto` lets Sonnet interleave reasoning text with
            // ToolUse blocks (per Anthropic docs, `Any` would prefill the
            // assistant turn and suppress natural-language reasoning).
            let system = self
                .system_prompt
                .as_ref()
                .map(|p| vec![SystemBlock::text(p.clone())]);
            let request = MessageRequest {
                model: self.model.clone(),
                max_tokens: effective_max_tokens,
                messages: vec![InputMessage::user_text(prompt)],
                system,
                tools: Some(self.build_tool_catalog()),
                tool_choice: Some(ToolChoice::Auto),
                stream: false,
                seed: self.seed,
                // I-26: production agent RPC uses tool-calling mode, not reasoning;
                // effort is not applicable (HD-3: force-JSON wrapping of reasoning
                // roles is banned; tool-calling and reasoning are mutually exclusive
                // on current Anthropic API).
                effort: None,
                chat_template_kwargs: None,
            };

            match self.client.send_message(&request).await {
                Ok(response) => {
                    // patch-R3 — collapse `response.content` into the agent's
                    // text-based ReAct surface:
                    //   - Each `Text` block contributes its text verbatim.
                    //   - Each `ToolUse` block contributes a `"TOOL_USE: <name>"`
                    //     marker line (agent_loop.rs:79 marker contract) and
                    //     stashes its `input` under `name` for the upcoming
                    //     `ToolRequest` whose `arguments` are empty
                    //     (agent_loop.rs:170).
                    //   - `Thinking` / `RedactedThinking` blocks are dropped
                    //     (the agent has no parser for them today).
                    let mut text_blocks: Vec<String> = Vec::new();
                    let mut markers: Vec<String> = Vec::new();
                    let mut to_stash: Vec<(String, Value)> = Vec::new();

                    for block in response.content.into_iter() {
                        match block {
                            OutputContentBlock::Text { text } => text_blocks.push(text),
                            OutputContentBlock::ToolUse { id: _, name, input } => {
                                markers.push(format!("TOOL_USE: {name}"));
                                to_stash.push((name, input));
                            }
                            OutputContentBlock::Thinking { .. }
                            | OutputContentBlock::RedactedThinking { .. } => {}
                        }
                    }

                    if text_blocks.is_empty() && markers.is_empty() {
                        return InferOutcome::Failed(
                            "provider returned no text or tool_use content blocks"
                                .to_string(),
                        );
                    }

                    // Single lock acquisition for all pushes; dropped before
                    // we build the response string. Never held across the
                    // earlier `send_message().await`.
                    if !to_stash.is_empty() {
                        let mut pending = self.pending_tool_args.lock().await;
                        for (name, value) in to_stash {
                            pending.entry(name).or_default().push_back(value);
                        }
                    }

                    let separator = if !text_blocks.is_empty() && !markers.is_empty() {
                        "\n"
                    } else {
                        ""
                    };
                    let combined = format!(
                        "{}{}{}",
                        text_blocks.join("\n"),
                        separator,
                        markers.join("\n")
                    );
                    InferOutcome::Response(combined)
                }
                Err(e) => InferOutcome::Failed(format!("{e:?}")),
            }
        })
    }

    fn handle_tool_request<'a>(
        &'a self,
        tool_id: String,
        arguments: Vec<u8>,
    ) -> Pin<Box<dyn std::future::Future<Output = ToolOutcome> + Send + 'a>> {
        Box::pin(async move {
            // patch-R3 — `agent_loop` sends `ToolRequest.arguments = Vec::new()`
            // (agent_loop.rs:170) because its in-text `TOOL_USE: <id>` marker
            // carries no payload. When `arguments` is empty, try to pop the
            // corresponding stash entry seeded by `handle_infer_request` and
            // convert the JSON input to the postcard bytes the dispatcher arms
            // expect. A stash miss falls through to `translate_tool_request`
            // with the original empty `arguments`, which yields either
            // `"unknown tool_id:"` (preserves the test-c contract) or a
            // postcard decode error for a real-but-empty tool_id.
            let effective_arguments: Vec<u8> = if arguments.is_empty() {
                let popped = {
                    let mut pending = self.pending_tool_args.lock().await;
                    pending.get_mut(&tool_id).and_then(|q| q.pop_front())
                };
                match popped {
                    Some(value) => match convert_json_input_to_postcard(&tool_id, value) {
                        Ok(bytes) => bytes,
                        Err(reason) => return ToolOutcome::Failed(reason),
                    },
                    None => arguments,
                }
            } else {
                arguments
            };

            match translate_tool_request(
                &tool_id,
                &effective_arguments,
                self.dispatcher.as_ref(),
                self.ssb.as_ref(),
                self.artifact_vm.as_ref().map(Arc::as_ref),
                &self.clock,
                self.symbol_graph_adapter.as_ref(),
            )
            .await
            {
                Ok(bytes) => ToolOutcome::Result(bytes),
                Err(reason) => ToolOutcome::Failed(reason),
            }
        })
    }

    /// I-14: run the deterministic context-compression middleware on a successful
    /// tool result before it enters the agent context window, journaling a reversible
    /// `ContextTransformReceipt` SSB row (ID-3 JOURNALED-MIDDLEWARE).
    ///
    /// When no SSB writer is wired (`ssb: None` -- harness / test paths), this is an
    /// identity passthrough (no transform, no receipt). With SSB wired, a
    /// [`crate::context_transform::ContextTransformer`] runs the Tier-1 pipeline; a
    /// receipt-write failure is logged (non-fatal) and the ORIGINAL bytes are returned
    /// so a journaling fault never corrupts the agent's view of the tool result.
    fn transform_tool_output<'a>(
        &'a self,
        output: Vec<u8>,
        superstep: i64,
    ) -> Pin<Box<dyn std::future::Future<Output = Vec<u8>> + Send + 'a>> {
        Box::pin(async move {
            let Some(ssb) = self.ssb.as_ref() else {
                return output;
            };
            let transformer = crate::context_transform::ContextTransformer::new(
                Arc::clone(ssb),
                Arc::clone(&self.clock),
            );
            match transformer.transform(&output, superstep).await {
                Ok(result) => result.bytes,
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "context_transform: receipt write failed; returning original bytes"
                    );
                    output
                }
            }
        })
    }
}

// ---------------------------------------------------------------------------
// convert_json_input_to_postcard — patch-R3
// ---------------------------------------------------------------------------

/// patch-R3 — Convert a Sonnet `OutputContentBlock::ToolUse.input` JSON value
/// into the postcard-encoded wire bytes the matching dispatcher arm expects.
///
/// Each known `tool_id` is decoded into its `elai_ipc_types::*Request` wire
/// struct first (the typed deserialisation is itself the input-validation
/// gate — attacker-controlled JSON cannot synthesise an arbitrary postcard
/// frame because every field must satisfy the struct's `serde` schema), then
/// `postcard::to_allocvec` produces the byte payload that
/// `translate_tool_request` re-decodes.
///
/// Returns `Err(String)` for an unknown `tool_id`, JSON decode failure, or
/// postcard encode failure. Errors propagate as `ToolOutcome::Failed` so the
/// agent records the failure and continues its ReAct loop.
fn convert_json_input_to_postcard(tool_id: &str, input: Value) -> Result<Vec<u8>, String> {
    match tool_id {
        TOOL_EDIT => {
            let req: elai_ipc_types::EditRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_FILE_READ => {
            let req: elai_ipc_types::FileReadRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_VALIDATE => {
            let req: elai_ipc_types::ValidateRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_COMPILE_CHECK => {
            let req: elai_ipc_types::CompileCheckRequest =
                serde_json::from_value(input).map_err(|e| {
                    format!(
                        "convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}"
                    )
                })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_CLASSIFY => {
            let req: elai_ipc_types::ClassifyRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_GIT_COMMIT => {
            let req: elai_ipc_types::GitCommitRequest =
                serde_json::from_value(input).map_err(|e| {
                    format!(
                        "convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}"
                    )
                })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        // C-24 (DRIFT-12): JSON->postcard bridge for the 4 B-29 Phase 5
        // symbol-graph tools now exposed in the catalog. Field names
        // match the wire structs in ipc-types verbatim — see DECISION-1.
        TOOL_SYMBOL_IMPACT => {
            let req: SymbolImpactRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_SYMBOL_FLOW => {
            let req: SymbolFlowRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_SYMBOL_CONTEXT => {
            let req: SymbolContextRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_SYMBOL_LIST => {
            let req: SymbolListRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        // E-40 Tranche B-2: SymbolGraph DSL + dataflow tools.
        TOOL_SYMBOL_DSL_QUERY => {
            let req: SymbolDslQueryRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_SYMBOL_DATAFLOW_IMPACT => {
            let req: SymbolDataflowImpactRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        // H-13: canvas op tools (model drives the canvas).
        TOOL_CANVAS_CREATE_NODE => {
            let req: CanvasCreateNodeRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_CANVAS_CONNECT => {
            let req: CanvasConnectRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_CANVAS_UPDATE_NODE => {
            let req: CanvasUpdateNodeRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_CANVAS_ANNOTATE => {
            let req: CanvasAnnotateRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_CANVAS_FORK => {
            let req: CanvasForkRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_CANVAS_RENDER_DIAGRAM => {
            let req: CanvasRenderDiagramRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_CANVAS_SCREENSHOT => {
            let req: CanvasScreenshotRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        // H-15: spawn_view reply-channel verb.
        TOOL_CANVAS_SPAWN_VIEW => {
            let req: CanvasSpawnViewRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        // B-34: entry-point detection verb. Mirrors the symbol-graph arms —
        // the typed `EntryPointsRequest` deserialisation is the input-validation
        // gate before postcard encode. The dispatcher arm at TOOL_ENTRY_POINTS
        // re-decodes these bytes via `postcard::from_bytes`.
        TOOL_ENTRY_POINTS => {
            let req: EntryPointsRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        // I-13: TraceQuery JSON->postcard bridge.
        // Decodes TraceQueryRequest (file_path, start_line, end_line) from model JSON input,
        // re-encodes to postcard for translate_tool_request.
        TOOL_TRACE_QUERY => {
            let req: TraceQueryRequest =
                serde_json::from_value(input).map_err(|e| {
                    format!(
                        "convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}"
                    )
                })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!(
                    "convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}"
                )
            })
        }
        // I-8: voluntary-abort tool JSON->postcard bridge.
        // Decodes AcknowledgeInfeasibleRequest (reason: String, trajectory_id: Option<String>)
        // from the model's JSON input, then re-encodes to postcard for translate_tool_request.
        TOOL_ACKNOWLEDGE_INFEASIBLE => {
            let req: AcknowledgeInfeasibleRequest =
                serde_json::from_value(input).map_err(|e| {
                    format!(
                        "convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}"
                    )
                })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!(
                    "convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}"
                )
            })
        }
        unknown => Err(format!(
            "convert_json_input_to_postcard: no mapping for tool_id '{unknown}'"
        )),
    }
}

// ---------------------------------------------------------------------------
// translate_tool_request — private dispatch helper
// ---------------------------------------------------------------------------

/// Deserialise `arguments` (postcard-encoded ipc-types wire struct) for the
/// given `tool_id`, call the matching `ToolDispatcher` method, and serialise
/// the result to JSON bytes.
///
/// Returns `Err(String)` for:
///   - unknown `tool_id`
///   - postcard decode failure
///   - dispatcher error
///   - JSON serialisation failure
/// Symbol-graph tool agent_id used when appending SymbolQuery SSB rows.
/// Distinct from per-baby-step agent IDs so dashboard filters can isolate
/// symbol-graph activity from regular agent tool calls.
const SSB_AGENT_SYMBOL_GRAPH: &str = "symbol_graph_tool";

/// C-8 Task #5: walk up from a `context_*.db` path to the project root.
/// Typical layout is `<project_root>/.elai/indexes/context_*.db`, so the
/// project root is the grandparent's parent. Returns `None` if the path
/// lacks at least three ancestor levels.
fn derive_project_root_from_db_path(db_path: &std::path::Path) -> Option<std::path::PathBuf> {
    db_path
        .parent()? // .elai/indexes
        .parent()? // .elai
        .parent() // <project_root>
        .map(|p| p.to_path_buf())
}

/// C-8 Task #5: derive a stable project_id from the canonical project root
/// path. Mirrors `metadata::project_id_from_path` semantics without pulling
/// the helper into the public surface — we just need the file_name as the
/// project_id label on the SSB payload.
fn derive_project_id_from_root(project_root: &std::path::Path) -> String {
    project_root
        .file_name()
        .and_then(|os| os.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Best-effort SSB write — never fails the tool call. Logs a warn on error.
async fn emit_symbol_query(
    ssb: Option<&Arc<crate::ssb::Ssb>>,
    tool_id: &str,
    target: String,
    hop_count: u32,
    total_files: u32,
    latency_ms: u32,
) {
    let Some(ssb) = ssb else { return };
    use crate::ssb::{SsbPayload, SymbolQueryPayload};
    let payload = SsbPayload::SymbolQuery(SymbolQueryPayload {
        tool: tool_id.to_string(),
        target,
        hop_count,
        total_files,
        latency_ms,
    });
    if let Err(e) = ssb
        .write_payload(SSB_AGENT_SYMBOL_GRAPH, 0, payload, Vec::new())
        .await
    {
        tracing::warn!(
            tool = tool_id,
            error = %e,
            "symbol_query SSB write failed (continuing tool result)"
        );
    }
}

// C-18 D1: visibility raised from private to `pub(crate)` so the in-crate
// integration tests under `crates/orchestrator/tests/` can reach it via
// the public test-helper alias `translate_tool_request_for_test` (gated by
// `test-support` feature). No change to the function body. The crate-public
// surface stays the same for downstream crates (they cannot name `pub(crate)`
// items either way), and the production agent loop continues to call this
// function directly from within the same crate.
pub(crate) async fn translate_tool_request(
    tool_id: &str,
    arguments: &[u8],
    dispatcher: &dyn ToolDispatcher,
    ssb: Option<&Arc<crate::ssb::Ssb>>,
    artifact_vm: Option<&(dyn elai_knowledge::vector::VectorModule + Send + Sync)>,
    clock: &Arc<dyn ClockProvider>,
    symbol_graph_adapter: Option<
        &Arc<dyn elai_knowledge::symbol_graph_adapter::SymbolGraphAdapter>,
    >,
) -> Result<Vec<u8>, String> {
    match tool_id {
        TOOL_EDIT => {
            let wire: elai_ipc_types::EditRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool edit: decode error: {e}"))?;
            // G-1 SINK GATE: block an edit whose target path or new content
            // derives from untrusted ingest (web/tool-output/file/sandbox).
            // `fs_write` is the canonical destructive-tool name the capability
            // gate checks. Runs BEFORE the real dispatch (fail-closed).
            enforce_taint_sink_gate(
                "fs_write",
                "edit",
                &[wire.file_path.as_str(), wire.new_text.as_str()],
                ssb,
            )?;
            // G-3 SINK GATE: block an edit whose EXPANDED target path resolves
            // OUTSIDE the per-run worktree root (the post-expansion FS escape,
            // e.g. file_path = `~/.ssh/authorized_keys` or `../../etc/x`). Runs
            // BEFORE the real dispatch (fail-closed); emits blast_radius_blocked
            // {FsEscape} through the live Ssb. The worktree root is the project
            // boundary (`taint_project_root()`); only the file_path is a path
            // token (new_text is content, not a path).
            enforce_fs_blast_radius(
                &[wire.file_path.as_str()],
                taint_project_root().as_path(),
                &format!("edit {}", wire.file_path),
                ssb,
                i64::try_from(clock.now_ms()).unwrap_or(i64::MAX),
            )?;
            let req = EditRequest {
                file_path: PathBuf::from(wire.file_path),
                old_text: wire.old_text,
                new_text: wire.new_text,
            };
            let outcome = dispatcher
                .dispatch_edit(req)
                .await
                .map_err(|e| format!("tool edit: dispatch error: {e:?}"))?;
            let value = serde_json::json!({
                "tier": format!("{:?}", outcome.tier),
                "bytes_written": outcome.bytes_written,
                "file_path": outcome.file_path.to_string_lossy(),
            });
            serde_json::to_vec(&value)
                .map_err(|e| format!("tool edit: serialise error: {e}"))
        }

        TOOL_VALIDATE => {
            let wire: elai_ipc_types::ValidateRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool validate: decode error: {e}"))?;
            // WARN-1: derive project_dir as the deepest containing directory
            // of file_path, not the file path itself. This matches what
            // dispatch_validate expects (a directory, not a file).
            let file_path = PathBuf::from(&wire.file_path);
            let project_dir = file_path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            let req = ValidateRequest {
                project_dir,
                language: wire.language,
                changed_files: Vec::new(),
                step_id: String::new(),
                tiers: None,
            };
            let outcomes = dispatcher
                .dispatch_validate(req)
                .await
                .map_err(|e| format!("tool validate: dispatch error: {e:?}"))?;
            let arr: Vec<serde_json::Value> = outcomes
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "tier": format!("{:?}", r.tier),
                        "passed": r.passed,
                        "elapsed_ms": r.elapsed_ms,
                        "rules_fired": r.rules_fired,
                    })
                })
                .collect();
            serde_json::to_vec(&serde_json::Value::Array(arr))
                .map_err(|e| format!("tool validate: serialise error: {e}"))
        }

        TOOL_CLASSIFY => {
            let wire: elai_ipc_types::ClassifyRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool classify: decode error: {e}"))?;
            let input = FailureInput {
                raw_output: wire.error_output,
                step_id: String::new(),
                source: wire.file_path.unwrap_or_default(),
            };
            let outcome = dispatcher.dispatch_classify_failure(input).await;
            let value = match outcome {
                Some(cf) => serde_json::json!({
                    "category": format!("{:?}", cf.category),
                    "confidence": cf.confidence,
                    "source": cf.source,
                }),
                None => serde_json::Value::Null,
            };
            serde_json::to_vec(&value)
                .map_err(|e| format!("tool classify: serialise error: {e}"))
        }

        TOOL_GIT_COMMIT => {
            let wire: elai_ipc_types::GitCommitRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool git_commit: decode error: {e}"))?;
            // WARN-5: explicit error message that names the constraint.
            let anchor_path = wire
                .paths
                .first()
                .map(PathBuf::from)
                .ok_or_else(|| {
                    "tool git_commit: git commit requires at least one path in `paths`; \
                     received empty list"
                        .to_string()
                })?;
            // G-1 SINK GATE: block a git commit whose message or any path
            // derives from untrusted ingest. Git shells out, so the canonical
            // destructive-tool name the gate checks is `bash`. Runs BEFORE the
            // real dispatch (fail-closed). Build the argv-parts view before
            // `wire.paths` is consumed below.
            {
                let mut parts: Vec<&str> = Vec::with_capacity(wire.paths.len() + 1);
                parts.push(wire.message.as_str());
                for p in &wire.paths {
                    parts.push(p.as_str());
                }
                enforce_taint_sink_gate("bash", "git_commit", &parts, ssb)?;
                // G-7 LETHAL-TRIFECTA GATE: git shells out (`bash` is an egress
                // channel), so block when a secret is present AND the commit
                // message/paths derive from an untrusted ingest AND the egress
                // sink is reachable. The untrusted span is quarantined and never
                // reaches the tool-caller (the load-bearing security property).
                enforce_trifecta_quarantine_gate("bash", "git_commit", &parts, ssb)?;
                // G-3 SINK GATE: block a git commit whose EXPANDED path tokens
                // resolve OUTSIDE the worktree root (a commit reaching outside
                // the per-run tree). Only the paths are path tokens (the commit
                // message is not). Fail-closed; emits blast_radius_blocked
                // {FsEscape} through the live Ssb.
                enforce_fs_blast_radius(
                    &wire.paths.iter().map(String::as_str).collect::<Vec<&str>>(),
                    taint_project_root().as_path(),
                    &format!("git_commit {}", wire.paths.join(" ")),
                    ssb,
                    i64::try_from(clock.now_ms()).unwrap_or(i64::MAX),
                )?;
            }
            let files: Vec<PathBuf> = wire.paths.into_iter().map(PathBuf::from).collect();
            // WARN-3: use GitBranchPolicy::default() (BranchPerPlan) instead of Main.
            // plan_id: wire GitCommitRequest has no plan_id field; fall back to a
            // UUIDv4 string so the per-plan branch has a unique non-empty name.
            let plan_id = uuid::Uuid::new_v4().to_string(); // entropy-allow: branch-name only, no EntropySource threaded
            let req = GitCommitRequest {
                anchor_path,
                files,
                message: wire.message,
                branch_policy: GitBranchPolicy::default(),
                plan_id,
                step_id: String::new(),
            };
            let outcome = dispatcher
                .dispatch_git_commit(req)
                .await
                .map_err(|e| format!("tool git_commit: dispatch error: {e:?}"))?;
            let value = serde_json::json!({
                "sha": outcome.sha,
                "branch": outcome.branch,
                "files_count": outcome.files_count,
                "message": outcome.message,
            });
            serde_json::to_vec(&value)
                .map_err(|e| format!("tool git_commit: serialise error: {e}"))
        }

        TOOL_FILE_READ => {
            let wire: elai_ipc_types::FileReadRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool file_read: decode error: {e}"))?;
            // WARN-2: do NOT silently drop offset/limit — return an explicit error
            // so agents know windowed reads are unsupported and must retry without them.
            if wire.offset.is_some() || wire.limit.is_some() {
                return Err(
                    "windowed file reads (offset/limit) not yet supported by \
                     dispatch_file_read_outline; pass without offset/limit for full outline"
                        .to_string(),
                );
            }
            let path = PathBuf::from(wire.file_path);
            let view = dispatcher
                .dispatch_file_read_outline(path.clone(), OUTLINE_LINE_THRESHOLD)
                .await
                .map_err(|e| format!("tool file_read: dispatch error: {e:?}"))?;
            // G-1 INGEST: a file read is path-screened. A read of TRACKED
            // in-project source under the workspace root is Trusted (first-party
            // code under version control); any other path (external / untracked
            // / parent-escape) is Untrusted. Register the read content into the
            // taint side-table so a later destructive sink whose argv derives
            // from external file content is gated. The side-table skips Trusted
            // labels, so a tracked .elai_cc/ read is a no-op (the FP-guard case).
            if let Some(content) = view.full_content.as_deref() {
                let prov =
                    Provenance::from_file_screened(path.clone(), taint_project_root().as_path());
                tainted_context().register(content, prov);
            }
            let value = serde_json::json!({
                "outline_only": view.outline_only,
                "total_lines": view.total_lines,
                "signatures": view.signatures,
                "imports": view.imports,
                "full_content": view.full_content,
            });
            serde_json::to_vec(&value)
                .map_err(|e| format!("tool file_read: serialise error: {e}"))
        }

        TOOL_COMPILE_CHECK => {
            let wire: elai_ipc_types::CompileCheckRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool compile_check: decode error: {e}"))?;
            // WARN-1: derive project_dir as the deepest containing directory
            // of file_path, not the file path itself.
            let file_path = PathBuf::from(&wire.file_path);
            let project_dir = file_path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            let req = IncrementalRequest {
                project_dir,
                language: wire.language,
                retry_round: wire.retry_round,
                step_id: String::new(),
            };
            let outcome = dispatcher
                .dispatch_incremental_compile(req)
                .await
                .map_err(|e| format!("tool compile_check: dispatch error: {e:?}"))?;
            let value = serde_json::json!({
                "success": outcome.success,
                "attempts": outcome.attempts,
                "elapsed_ms": outcome.elapsed_ms,
            });
            serde_json::to_vec(&value)
                .map_err(|e| format!("tool compile_check: serialise error: {e}"))
        }

        // B-29 Phase 5 — Symbol-graph traversal tools.
        // Arguments are postcard-encoded typed request structs from ipc-types.
        // Each arm: open SymbolGraphCache, call the matching traversal fn,
        // serialize the result as JSON.

        TOOL_SYMBOL_IMPACT => {
            let wire: SymbolImpactRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool symbol_impact: decode error: {e}"))?;
            let project_root = std::path::PathBuf::from(&wire.project_root);
            let project_id = project_root
                .file_name()
                .and_then(|n| n.to_str())
                .filter(|n| !n.is_empty())
                .ok_or_else(|| {
                    format!(
                        "tool symbol_impact: cannot derive project_id from project_root={:?}",
                        wire.project_root
                    )
                })?
                .to_string();
            let db_path = elai_core::paths::dkg_db_path(&project_root);
            let mut cache = SymbolGraphCache::new(&db_path, &project_id, SYMBOL_FILE_LRU_SIZE)
                .map_err(|e| format!("tool symbol_impact: cache open failed: {e}"))?;
            let t0 = clock.now();
            let result = compute_impact_radius(&mut cache, &wire.target, wire.depth)
                .map_err(|e| format!("tool symbol_impact: traversal error: {e}"))?;
            let latency_ms = clock.elapsed_since(t0).as_millis().try_into().unwrap_or(u32::MAX);
            let total_files_u32: u32 = result.total_files.try_into().unwrap_or(u32::MAX);
            emit_symbol_query(
                ssb,
                TOOL_SYMBOL_IMPACT,
                wire.target.clone(),
                wire.depth,
                total_files_u32,
                latency_ms,
            )
            .await;
            serde_json::to_vec(&result)
                .map_err(|e| format!("tool symbol_impact: serialise error: {e}"))
        }

        TOOL_SYMBOL_FLOW => {
            let wire: SymbolFlowRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool symbol_flow: decode error: {e}"))?;
            let project_root = std::path::PathBuf::from(&wire.project_root);
            let project_id = project_root
                .file_name()
                .and_then(|n| n.to_str())
                .filter(|n| !n.is_empty())
                .ok_or_else(|| {
                    format!(
                        "tool symbol_flow: cannot derive project_id from project_root={:?}",
                        wire.project_root
                    )
                })?
                .to_string();
            let db_path = elai_core::paths::dkg_db_path(&project_root);
            let mut cache = SymbolGraphCache::new(&db_path, &project_id, SYMBOL_FILE_LRU_SIZE)
                .map_err(|e| format!("tool symbol_flow: cache open failed: {e}"))?;
            let t0 = clock.now();
            let result = compute_call_flow(&mut cache, &wire.entrypoint_id, wire.depth)
                .map_err(|e| format!("tool symbol_flow: traversal error: {e}"))?;
            let latency_ms = clock.elapsed_since(t0).as_millis().try_into().unwrap_or(u32::MAX);
            // For flow queries, total_files is set to the number of unique files
            // referenced in the flow tree (root + descendants); 0 if Option<FlowNode> is None.
            let total_files_u32: u32 = match &result {
                Some(_root) => 1u32, // Flow tree starts at one entrypoint file; deeper count via traversal optional in v1
                None => 0u32,
            };
            emit_symbol_query(
                ssb,
                TOOL_SYMBOL_FLOW,
                wire.entrypoint_id.clone(),
                wire.depth,
                total_files_u32,
                latency_ms,
            )
            .await;
            serde_json::to_vec(&result)
                .map_err(|e| format!("tool symbol_flow: serialise error: {e}"))
        }

        TOOL_SYMBOL_CONTEXT => {
            let wire: SymbolContextRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool symbol_context: decode error: {e}"))?;
            let project_root = std::path::PathBuf::from(&wire.project_root);
            let project_id = project_root
                .file_name()
                .and_then(|n| n.to_str())
                .filter(|n| !n.is_empty())
                .ok_or_else(|| {
                    format!(
                        "tool symbol_context: cannot derive project_id from project_root={:?}",
                        wire.project_root
                    )
                })?
                .to_string();
            let db_path = elai_core::paths::dkg_db_path(&project_root);
            let mut cache = SymbolGraphCache::new(&db_path, &project_id, SYMBOL_FILE_LRU_SIZE)
                .map_err(|e| format!("tool symbol_context: cache open failed: {e}"))?;
            let t0 = clock.now();
            let contexts = compute_symbol_context(
                &mut cache,
                &wire.name,
                wire.file_hint.as_deref(),
            )
            .map_err(|e| format!("tool symbol_context: traversal error: {e}"))?;
            let latency_ms = clock.elapsed_since(t0).as_millis().try_into().unwrap_or(u32::MAX);
            let total_files_u32: u32 = contexts.len().try_into().unwrap_or(u32::MAX);
            emit_symbol_query(
                ssb,
                TOOL_SYMBOL_CONTEXT,
                wire.name.clone(),
                0,
                total_files_u32,
                latency_ms,
            )
            .await;
            serde_json::to_vec(&contexts)
                .map_err(|e| format!("tool symbol_context: serialise error: {e}"))
        }

        TOOL_SYMBOL_LIST => {
            let wire: SymbolListRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool symbol_list: decode error: {e}"))?;
            let project_root = std::path::PathBuf::from(&wire.project_root);
            let project_id = project_root
                .file_name()
                .and_then(|n| n.to_str())
                .filter(|n| !n.is_empty())
                .ok_or_else(|| {
                    format!(
                        "tool symbol_list: cannot derive project_id from project_root={:?}",
                        wire.project_root
                    )
                })?
                .to_string();
            let db_path = elai_core::paths::dkg_db_path(&project_root);
            let mut cache = SymbolGraphCache::new(&db_path, &project_id, SYMBOL_FILE_LRU_SIZE)
                .map_err(|e| format!("tool symbol_list: cache open failed: {e}"))?;
            // Bound the limit defensively — agents must not be able to force an
            // unbounded scan of every symbol in the cache.
            const MAX_SYMBOL_LIST_LIMIT: u32 = 500;
            let capped_limit = wire.limit.min(MAX_SYMBOL_LIST_LIMIT);
            let filter = SymbolFilter {
                file: wire.file.clone(),
                query: wire.query.clone(),
                limit: capped_limit as usize,
            };
            let t0 = clock.now();
            let symbols = list_symbols(&mut cache, filter)
                .map_err(|e| format!("tool symbol_list: traversal error: {e}"))?;
            let latency_ms = clock.elapsed_since(t0).as_millis().try_into().unwrap_or(u32::MAX);
            let total_files_u32: u32 = symbols.len().try_into().unwrap_or(u32::MAX);
            let target_label = wire
                .query
                .clone()
                .or(wire.file.clone())
                .unwrap_or_else(|| "<all>".to_string());
            emit_symbol_query(
                ssb,
                TOOL_SYMBOL_LIST,
                target_label,
                0,
                total_files_u32,
                latency_ms,
            )
            .await;
            serde_json::to_vec(&symbols)
                .map_err(|e| format!("tool symbol_list: serialise error: {e}"))
        }

        // E-40 Tranche B-2 + s-22600 Tranche B ITEM B.4: SymbolGraph DSL +
        // dataflow tools. When `symbol_graph_adapter` is Some (CLI startup
        // wired a `SymbolGraphCacheAdapter`), dispatch the query through
        // the adapter and return real results. When None, preserve the
        // pre-B.4 empty-response behaviour so harness / test paths that
        // do not wire a cache continue to see a consistent wire shape.
        // Adapter errors (cache unavailable, DSL parse fail, etc) degrade
        // to empty responses with `tracing::warn` -- the agent loop is
        // best-effort over symbol graph tools.
        TOOL_SYMBOL_DSL_QUERY => {
            let wire: SymbolDslQueryRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool symbol_dsl_query: decode error: {e}"))?;
            let t0 = clock.now();
            let resp = match symbol_graph_adapter {
                Some(adapter) => {
                    let project_root = PathBuf::from(&wire.project_root);
                    match adapter.dsl_query(&wire.dsl, &project_root, wire.depth_cap) {
                        Ok(r) => r,
                        Err(e) => {
                            tracing::warn!(
                                tool = TOOL_SYMBOL_DSL_QUERY,
                                error = %e,
                                "symbol_dsl_query adapter error -- returning empty response"
                            );
                            elai_ipc_types::SymbolDslQueryResponse {
                                matches: Vec::new(),
                                dataflow_edges: Vec::new(),
                                total_scanned: 0,
                            }
                        }
                    }
                }
                None => elai_ipc_types::SymbolDslQueryResponse {
                    matches: Vec::new(),
                    dataflow_edges: Vec::new(),
                    total_scanned: 0,
                },
            };
            let latency_ms = clock.elapsed_since(t0).as_millis().try_into().unwrap_or(u32::MAX);
            // PRESERVE the existing emit_symbol_query beacon (additive
            // observability per s-22300 B-2 contract).
            let result_count: u32 = resp.matches.len().try_into().unwrap_or(u32::MAX);
            emit_symbol_query(
                ssb,
                TOOL_SYMBOL_DSL_QUERY,
                wire.dsl.clone(),
                wire.depth_cap as u32,
                result_count,
                latency_ms,
            )
            .await;
            // s-22600 Tranche B ITEM B.4: also emit the SymbolGraphEvent
            // row (Pattern-2 reuse, SymbolDslExecuted kind, SCHEMA 193).
            // Best-effort: failure to write is logged + swallowed so the
            // tool dispatch result is unaffected.
            if let Some(writer) = ssb {
                let ts_ms = i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX);
                if let Err(e) = crate::symbol_graph::notify_symbol_graph_event(
                    writer.as_ref(),
                    "production_agent_rpc",
                    0,
                    crate::ssb::SymbolGraphEventKind::SymbolDslExecuted,
                    None,
                    serde_json::json!({
                        "dsl": wire.dsl,
                        "depth_cap": wire.depth_cap,
                        "matches": resp.matches.len(),
                        "edges": resp.dataflow_edges.len(),
                    }),
                    ts_ms,
                )
                .await
                {
                    tracing::warn!(
                        tool = TOOL_SYMBOL_DSL_QUERY,
                        error = %e,
                        "symbol_dsl_query: SymbolGraphEvent SSB write failed"
                    );
                }
            }
            serde_json::to_vec(&resp)
                .map_err(|e| format!("tool symbol_dsl_query: serialise error: {e}"))
        }

        TOOL_SYMBOL_DATAFLOW_IMPACT => {
            let wire: SymbolDataflowImpactRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool symbol_dataflow_impact: decode error: {e}"))?;
            let t0 = clock.now();
            let resp = match symbol_graph_adapter {
                Some(adapter) => {
                    let project_root = PathBuf::from(&wire.project_root);
                    match adapter.dataflow_impact(&wire.symbol, &project_root, wire.depth_cap) {
                        Ok(r) => r,
                        Err(e) => {
                            tracing::warn!(
                                tool = TOOL_SYMBOL_DATAFLOW_IMPACT,
                                error = %e,
                                "symbol_dataflow_impact adapter error -- returning empty response"
                            );
                            elai_ipc_types::SymbolDataflowImpactResponse {
                                impacted_symbols: Vec::new(),
                                depth_actual: 0,
                                total_edges: 0,
                            }
                        }
                    }
                }
                None => elai_ipc_types::SymbolDataflowImpactResponse {
                    impacted_symbols: Vec::new(),
                    depth_actual: 0,
                    total_edges: 0,
                },
            };
            let latency_ms = clock.elapsed_since(t0).as_millis().try_into().unwrap_or(u32::MAX);
            let result_count: u32 = resp.impacted_symbols.len().try_into().unwrap_or(u32::MAX);
            emit_symbol_query(
                ssb,
                TOOL_SYMBOL_DATAFLOW_IMPACT,
                wire.symbol.clone(),
                wire.depth_cap as u32,
                result_count,
                latency_ms,
            )
            .await;
            // s-22600 Tranche B ITEM B.4: SymbolGraphEvent row,
            // DataflowImpactComputed kind. Best-effort emit.
            if let Some(writer) = ssb {
                let ts_ms = i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX);
                if let Err(e) = crate::symbol_graph::notify_symbol_graph_event(
                    writer.as_ref(),
                    "production_agent_rpc",
                    0,
                    crate::ssb::SymbolGraphEventKind::DataflowImpactComputed,
                    None,
                    serde_json::json!({
                        "symbol": wire.symbol,
                        "depth_cap": wire.depth_cap,
                        "depth_actual": resp.depth_actual,
                        "impacted_symbols": resp.impacted_symbols.len(),
                        "total_edges": resp.total_edges,
                    }),
                    ts_ms,
                )
                .await
                {
                    tracing::warn!(
                        tool = TOOL_SYMBOL_DATAFLOW_IMPACT,
                        error = %e,
                        "symbol_dataflow_impact: SymbolGraphEvent SSB write failed"
                    );
                }
            }
            serde_json::to_vec(&resp)
                .map_err(|e| format!("tool symbol_dataflow_impact: serialise error: {e}"))
        }

        TOOL_GIT_UNDO | TOOL_GIT_BRANCH => {
            Err(format!(
                "tool {tool_id}: not implemented in ProductionAgentRpcServer (B2c)"
            ))
        }

        // I-8: voluntary abort dispatcher arm.
        //
        // The model calls acknowledge_infeasible when it determines the current
        // trajectory is infeasible (spiral prevention, ID-1 doctrine).
        //
        // Wave-1 behaviour:
        //   1. Decode the AcknowledgeInfeasibleRequest.
        //   2. Emit TrajectoryAborted(Voluntary) SSB row (off-agent evidence; R4).
        //   3. Emit AbstentionScored row crediting the voluntary abstain (H-39
        //      compose, GD-1: no re-implementation of scoring logic).
        //   4. Conditionally call timetravel::materialize() for rollback — skipped
        //      with an honest log entry when no pre-abort snapshot is available
        //      (H-49 full trunk-restore is Wave-2).
        //   5. Return Err(reason) so handle_tool_call maps to ToolOutcome::Failed,
        //      surfacing the abort signal to the task runner.
        //
        // This is NOT a pure-Err-no-emit stub: the SSB row is the observable effect
        // required by wiring-enforcement R4 (missing row = FAIL).
        TOOL_ACKNOWLEDGE_INFEASIBLE => {
            let wire: AcknowledgeInfeasibleRequest =
                postcard::from_bytes(arguments)
                    .map_err(|e| format!("tool acknowledge_infeasible: decode error: {e}"))?;

            let ts_ms = crate::ssb::Ssb::now_ms_via_clock(clock);

            // Step 2: emit TrajectoryAborted(Voluntary) SSB beacon.
            if let Some(ssb_arc) = ssb {
                use crate::ssb::{
                    SsbPayload, TrajectoryAbortedKind, TrajectoryAbortedPayload,
                };
                let abort_payload = SsbPayload::TrajectoryAborted(TrajectoryAbortedPayload {
                    kind: TrajectoryAbortedKind::Voluntary,
                    session_id: String::new(), // caller context; SSB row carries session from write_payload call
                    trajectory_id: wire.trajectory_id.clone(),
                    approach_hash: None,
                    abort_reason_text: Some(wire.reason.clone()),
                    ts_ms,
                });
                if let Err(e) = ssb_arc
                    .write_payload("agent_rpc", 0, abort_payload, Vec::new())
                    .await
                {
                    tracing::warn!(
                        error = %e,
                        "acknowledge_infeasible: TrajectoryAborted SSB write failed (non-fatal)"
                    );
                }

                // Step 3: emit AbstentionScored row crediting voluntary abstain (H-39 compose).
                use crate::abstention_score::{
                    abstention_aware_score, OutcomeTriple, DEFAULT_CONFIDENCE_TARGET,
                };
                use crate::calibration::RefusalDecision;
                let outcome = OutcomeTriple::new(RefusalDecision::Abstain, false);
                let mean_score = abstention_aware_score(outcome, DEFAULT_CONFIDENCE_TARGET);
                let abstention_payload = SsbPayload::AbstentionScored(
                    crate::ssb::AbstentionScoredPayload {
                        suite: "voluntary_abort".into(),
                        mean_score,
                        abstention_rate: 1.0,
                        confident_wrong_rate: 0.0,
                        t: DEFAULT_CONFIDENCE_TARGET,
                        n: 1,
                        seed: 0,
                        corpus_blake3: String::new(),
                    },
                );
                if let Err(e) = ssb_arc
                    .write_payload("agent_rpc", 0, abstention_payload, Vec::new())
                    .await
                {
                    tracing::warn!(
                        error = %e,
                        "acknowledge_infeasible: AbstentionScored SSB write failed (non-fatal)"
                    );
                }
            }

            // Step 4: rollback via timetravel::materialize() if a manifest is available.
            // Wave-1: no pre-abort snapshot linked yet (H-49 trunk-restore is Wave-2).
            // Honest log; never panic on missing manifest.
            tracing::info!(
                "acknowledge_infeasible: rollback skipped: no pre-abort snapshot \
                 (H-49 Wave-2 will wire SessionDag HEAD restore)"
            );

            // Step 5: surface as ToolOutcome::Failed via Err return.
            Err(wire.reason)
        }

        // I-13: TraceQuery dispatcher arm.
        // 1. Decode TraceQueryRequest (file_path, start_line, end_line).
        // 2. Inline expand (elai-tools is dev-dep only; no circular dep allowed).
        //    Logic mirrors elai_tools::trace_query::TraceQuery::expand exactly.
        // 3. Emit MemoryGuardEvent(TraceQueryExpanded) SSB row (off-agent evidence; R4).
        // 4. Return Ok(content) so the model receives the expanded lines.
        TOOL_TRACE_QUERY => {
            let wire: TraceQueryRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool trace_query: decode error: {e}"))?;

            let path = std::path::PathBuf::from(&wire.file_path);
            let start = wire.start_line;
            let end = wire.end_line;

            // struct mirroring TraceQueryResult (no dep on elai-tools in lib code)
            struct ExpandResult {
                file_path: String,
                start_line: u32,
                end_line: u32,
                total_lines: u32,
                content: String,
            }

            let result = tokio::task::spawn_blocking(move || -> Result<ExpandResult, String> {
                if start < 1 || end < start {
                    return Err(format!(
                        "tool trace_query: invalid range start={start} end={end}"
                    ));
                }
                if !path.exists() {
                    return Err(format!(
                        "tool trace_query: file not found: {}",
                        path.display()
                    ));
                }
                let raw = std::fs::read_to_string(&path)
                    .map_err(|e| format!("tool trace_query: io error: {e}"))?;
                let all_lines: Vec<&str> = raw.lines().collect();
                let total_lines = all_lines.len() as u32;
                let zero_start = (start - 1) as usize;
                let zero_end = end.saturating_sub(1) as usize;
                let (actual_end, content) = if zero_start >= all_lines.len() {
                    (start, String::new())
                } else {
                    let clamped = zero_end.min(all_lines.len().saturating_sub(1));
                    let text = all_lines[zero_start..=clamped]
                        .iter()
                        .map(|l| format!("{l}\n"))
                        .collect();
                    ((clamped + 1) as u32, text)
                };
                Ok(ExpandResult {
                    file_path: path.to_string_lossy().into_owned(),
                    start_line: start,
                    end_line: actual_end,
                    total_lines,
                    content,
                })
            })
            .await
            .map_err(|e| format!("tool trace_query: spawn_blocking error: {e}"))?
            .map_err(|e| format!("tool trace_query: expand error: {e}"))?;

            // Emit MemoryGuardEvent(TraceQueryExpanded) SSB row (R4 off-agent artifact).
            if let Some(ssb_arc) = ssb {
                use crate::ssb::{MemoryGuardEventKind, MemoryGuardEventPayload, SsbPayload};
                let ts_ms = crate::ssb::Ssb::now_ms_via_clock(clock);
                let expanded_payload = SsbPayload::MemoryGuardEvent(MemoryGuardEventPayload {
                    kind: MemoryGuardEventKind::TraceQueryExpanded,
                    pattern_class: None,
                    token_delta: None,
                    byte_range: Some(format!(
                        "{}:{}-{}",
                        result.file_path, result.start_line, result.end_line
                    )),
                    ts_ms,
                });
                if let Err(e) = ssb_arc
                    .write_payload("agent_rpc", 0, expanded_payload, vec![result.file_path.clone()])
                    .await
                {
                    tracing::warn!(
                        error = %e,
                        "trace_query: MemoryGuardEvent SSB write failed (non-fatal)"
                    );
                }
            }

            let resp = serde_json::json!({
                "file_path": result.file_path,
                "start_line": result.start_line,
                "end_line": result.end_line,
                "total_lines": result.total_lines,
                "content": result.content,
            });
            serde_json::to_vec(&resp)
                .map_err(|e| format!("tool trace_query: json serialization error: {e}"))
        }

        // B-35 Phase B — Context artifact dispatcher arms.
        // ArtifactStore is sync (std::sync::Mutex); all calls are wrapped in
        // tokio::task::spawn_blocking so the async runtime is not blocked.

        TOOL_LIST_ARTIFACTS => {
            let wire: ListArtifactsRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool list_artifacts: decode error: {e}"))?;
            let db_path = std::path::PathBuf::from(&wire.db_path);
            let chunks = tokio::task::spawn_blocking(move || {
                let store = ArtifactStore::open(&db_path)
                    .map_err(|e| format!("tool list_artifacts: store open failed: {e}"))?;
                store.list_all_chunks()
                    .map_err(|e| format!("tool list_artifacts: list failed: {e}"))
            })
            .await
            .map_err(|e| format!("tool list_artifacts: spawn_blocking error: {e}"))??;
            // Return artifact names (de-duplicated, sorted) rather than raw chunks.
            let mut names: Vec<String> = chunks
                .into_iter()
                .map(|c| c.artifact_name)
                .collect();
            names.sort_unstable();
            names.dedup();
            let value = serde_json::json!({ "artifact_names": names });
            serde_json::to_vec(&value)
                .map_err(|e| format!("tool list_artifacts: serialise error: {e}"))
        }

        TOOL_INDEX_ARTIFACTS => {
            // C-8 Task #5: real implementation (replaces the previous deferred
            // Err stub). Singular per wire frame (IndexArtifactsRequest carries
            // one artifact_name), but the handler treats it as a one-element
            // Vec<IndexArtifactsRequest> internally per C-8 directive so future
            // bulk-frame upgrades only need to change the decode step.
            //
            // Pipeline:
            //   1. Decode IndexArtifactsRequest.
            //   2. Derive project_root from db_path (parent-of-parent walk —
            //      typical layout is <project_root>/.elai/indexes/context_*.db).
            //   3. Load ArtifactManifest::load(project_root). Filter to the
            //      named artifact; missing name => empty result, not an error.
            //   4. Open ArtifactStore + ArtifactIndexer, invoke index_artifact
            //      per filtered spec on a spawn_blocking thread.
            //   5. Emit one ArtifactsIndexed SSB row carrying the indexed +
            //      up_to_date partition + total chunk count + elapsed_ms.
            //   6. Return Ok JSON { project_id, artifact_count, chunk_count,
            //      elapsed_ms, indexed[], up_to_date[] }.
            //
            // Errors from individual specs are non-fatal (mirror
            // index_all_artifacts semantics). A genuine setup error (manifest
            // load failure, store open failure) maps to ToolError so the
            // agent sees the failure.
            let wire: IndexArtifactsRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool index_artifacts: decode error: {e}"))?;
            let pending: Vec<IndexArtifactsRequest> = vec![wire];

            let db_path = std::path::PathBuf::from(&pending[0].db_path);
            let project_root = derive_project_root_from_db_path(&db_path)
                .ok_or_else(|| {
                    format!(
                        "tool index_artifacts: cannot derive project_root from db_path \
                         {} (expected <project_root>/.elai/indexes/context_*.db layout)",
                        pending[0].db_path
                    )
                })?;

            let project_id = derive_project_id_from_root(&project_root);
            let names: Vec<String> = pending.iter().map(|r| r.artifact_name.clone()).collect();

            let start_ms = clock.now();
            // C-15: capture `now_secs` BEFORE spawning the blocking task so the
            // injected `ClockProvider` is the source of truth even inside the
            // `move` closure (we cannot move &Arc<dyn ClockProvider> across the
            // spawn_blocking boundary cheaply).
            let now_secs_for_indexer = clock.now_secs().try_into().unwrap_or(i64::MAX);
            let (indexed_names, up_to_date_names, total_chunks) =
                tokio::task::spawn_blocking({
                    let project_root = project_root.clone();
                    let db_path = db_path.clone();
                    let names = names.clone();
                    move || -> Result<(Vec<String>, Vec<String>, usize), String> {
                        let manifest = elai_knowledge::artifacts::ArtifactManifest::load(
                            &project_root,
                        )
                        .map_err(|e| {
                            format!("tool index_artifacts: manifest load failed: {e}")
                        })?;

                        let store = elai_knowledge::artifacts::store::ArtifactStore::open(
                            &db_path,
                        )
                        .map_err(|e| {
                            format!("tool index_artifacts: store open failed: {e}")
                        })?;
                        // No VectorModule on this RPC path (agents do not pass
                        // it through); embedding-less indexing is the
                        // documented graceful-degrade behaviour (chunks stored
                        // with embedding=None per artifacts/indexer.rs:18-21).
                        let indexer = elai_knowledge::artifacts::indexer::ArtifactIndexer::new(
                            &store, None,
                        );
                        // C-15: now_secs sourced from injected ClockProvider above
                        // (see `now_secs_for_indexer`). Keeps determinism under
                        // SeededClock.
                        let now_secs = now_secs_for_indexer;

                        let target_specs: Vec<&elai_knowledge::artifacts::ArtifactSpec> = manifest
                            .items
                            .iter()
                            .filter(|s| names.iter().any(|n| n == &s.name))
                            .collect();

                        let mut indexed = Vec::new();
                        let mut up_to_date = Vec::new();
                        let mut total = 0usize;
                        for spec in target_specs {
                            match indexer.index_artifact(spec, None, now_secs) {
                                Ok(state) => {
                                    total = total.saturating_add(state.chunk_count);
                                    match state.status {
                                        elai_knowledge::artifacts::ArtifactStatus::Indexed => {
                                            indexed.push(state.name);
                                        }
                                        _ => {
                                            up_to_date.push(state.name);
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        artifact = %spec.name,
                                        error = %e,
                                        "tool index_artifacts: index_artifact failed (non-fatal)"
                                    );
                                }
                            }
                        }
                        Ok((indexed, up_to_date, total))
                    }
                })
                .await
                .map_err(|e| format!("tool index_artifacts: spawn_blocking error: {e}"))??;

            let elapsed_ms = u64::try_from(clock.elapsed_since(start_ms).as_millis()).unwrap_or(u64::MAX);
            let artifact_count = u32::try_from(indexed_names.len() + up_to_date_names.len())
                .unwrap_or(u32::MAX);

            // Emit ONE ArtifactsIndexed SSB row (existing B-34 variant —
            // payload struct fields {project_id, indexed, up_to_date,
            // total_chunks, latency_ms} match exactly). Non-fatal SSB
            // write — log on failure but still return Ok to the agent.
            if let Some(ssb_arc) = ssb {
                let payload = crate::ssb::SsbPayload::ArtifactsIndexed(
                    crate::ssb::ArtifactsIndexedPayload {
                        project_id: project_id.clone(),
                        indexed: indexed_names.clone(),
                        up_to_date: up_to_date_names.clone(),
                        total_chunks,
                        latency_ms: elapsed_ms,
                    },
                );
                if let Err(e) = ssb_arc
                    .write_payload("agent_rpc", 0, payload, Vec::new())
                    .await
                {
                    tracing::warn!(
                        error = %e,
                        "tool index_artifacts: ArtifactsIndexed SSB write failed (non-fatal)"
                    );
                }
            }

            let value = serde_json::json!({
                "project_id": project_id,
                "artifact_count": artifact_count,
                "chunk_count": total_chunks,
                "elapsed_ms": elapsed_ms,
                "indexed": indexed_names,
                "up_to_date": up_to_date_names,
            });
            serde_json::to_vec(&value)
                .map_err(|e| format!("tool index_artifacts: serialise error: {e}"))
        }

        TOOL_SEARCH_ARTIFACTS => {
            let wire: SearchArtifactsRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool search_artifacts: decode error: {e}"))?;
            let limit = wire.limit
                .unwrap_or(elai_ipc_types::DEFAULT_ARTIFACT_SEARCH_LIMIT);
            let db_path = std::path::PathBuf::from(&wire.db_path);
            let query = wire.query.clone();
            let artifact_name = wire.artifact_name.clone();

            if let Some(vm) = artifact_vm {
                // Phase: embed in async context (VectorModule is not Send, cannot move
                // into spawn_blocking).  The embedding call is CPU-bound but short
                // relative to the store I/O that follows.
                let query_embedding = vm
                    .embed_text(&query)
                    .map_err(|e| format!("tool search_artifacts: embed_text failed: {e}"))?;

                // Offload SQLite I/O + cosine scan to a blocking thread.
                // query_embedding is Vec<f32> which is Send.
                let results = tokio::task::spawn_blocking(move || {
                    let store = ArtifactStore::open(&db_path)
                        .map_err(|e| format!("tool search_artifacts: store open failed: {e}"))?;
                    artifact_search_with_precomputed_embedding(
                        &store,
                        &query_embedding,
                        artifact_name.as_deref(),
                        limit,
                    )
                    .map_err(|e| format!("tool search_artifacts: search failed: {e}"))
                })
                .await
                .map_err(|e| format!("tool search_artifacts: spawn_blocking error: {e}"))??;

                // Strip embedding vectors before JSON serialisation: raw f32 arrays
                // bloat the IPC frame (~150 KB for 10 results at 1536-dim) and callers
                // do not need the raw vectors — scores and content are sufficient.
                let response_items: Vec<serde_json::Value> = results
                    .into_iter()
                    .map(|r| {
                        serde_json::json!({
                            "artifact_name": r.chunk.artifact_name,
                            "relative_path": r.chunk.relative_path,
                            "content": r.chunk.content,
                            "start_line": r.chunk.start_line,
                            "end_line": r.chunk.end_line,
                            "score": r.score,
                        })
                    })
                    .collect();
                let value = serde_json::json!({ "results": response_items });
                serde_json::to_vec(&value)
                    .map_err(|e| format!("tool search_artifacts: serialise error: {e}"))
            } else {
                // No VectorModule available — return empty results (graceful degrade).
                tracing::warn!(
                    "tool search_artifacts: no VectorModule injected — \
                     returning empty results (wire with_artifact_vm to enable search)"
                );
                let value = serde_json::json!({ "results": [] });
                serde_json::to_vec(&value)
                    .map_err(|e| format!("tool search_artifacts: serialise error: {e}"))
            }
        }

        TOOL_REMOVE_ARTIFACTS => {
            let wire: RemoveArtifactsRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool remove_artifacts: decode error: {e}"))?;
            let db_path = std::path::PathBuf::from(&wire.db_path);
            let artifact_name = wire.artifact_name.clone();
            let dropped = tokio::task::spawn_blocking(move || {
                let store = ArtifactStore::open(&db_path)
                    .map_err(|e| format!("tool remove_artifacts: store open failed: {e}"))?;
                store.drop_chunks_for_artifact(&artifact_name)
                    .map_err(|e| format!("tool remove_artifacts: drop failed: {e}"))
            })
            .await
            .map_err(|e| format!("tool remove_artifacts: spawn_blocking error: {e}"))??;
            let value = serde_json::json!({
                "artifact_name": wire.artifact_name,
                "chunks_removed": dropped,
            });
            serde_json::to_vec(&value)
                .map_err(|e| format!("tool remove_artifacts: serialise error: {e}"))
        }

        // B-34 — Entry-point detection tool.
        // Loads reverse_file_index once (lazy), then selectively loads payloads
        // for orphan-candidate files + conventional entry-point filenames only.
        // This preserves the B-29 lazy-loading invariant.
        TOOL_ENTRY_POINTS => {
            let wire: EntryPointsRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool entry_points: decode error: {e}"))?;
            let project_root = std::path::PathBuf::from(&wire.project_root);
            // Use canonical blake3-based project_id (WARN-2: replaces basename heuristic).
            let project_id = elai_core::project_id_from_path(&project_root);
            let db_path = elai_core::paths::dkg_db_path(&project_root);

            // All SymbolGraphCache operations are synchronous (no async I/O).
            let t0 = clock.now();
            let result: Result<Vec<DetectedEntryPoint>, String> = (|| {
                let mut cache =
                    SymbolGraphCache::new(&db_path, &project_id, SYMBOL_FILE_LRU_SIZE)
                        .map_err(|e| format!("tool entry_points: cache open failed: {e}"))?;

                // Step 1: load reverse index (lazy, ~5ms first call).
                let reverse_index = cache
                    .ensure_reverse_index()
                    .map_err(|e| format!("tool entry_points: reverse_index load failed: {e}"))?
                    .clone();

                // Step 2: collect all known files from the reverse index
                // (keys = callee files; values = caller files).
                // Union gives every indexed file without loading name shards.
                let mut all_files: std::collections::BTreeSet<String> =
                    reverse_index.keys().cloned().collect();
                for callers in reverse_index.values() {
                    all_files.extend(callers.iter().cloned());
                }

                // Step 3: coarse extension filter — exclude non-code files that can
                // never contain entry points (BLOCK-5: the old orphan+filename filter
                // broke Heuristic 3 by excluding imported route files; we now load
                // payloads for all code-language files; non-code files are excluded for
                // performance. See workflow/reasoning/B-34-decisions.md DECISION-B34-9).
                const CODE_EXTENSIONS: &[&str] = &[
                    "ts", "tsx", "js", "jsx", "py", "rb", "go", "rs", "java", "kt", "php",
                    "c", "cpp", "cs", "swift", "scala", "ex", "exs",
                ];
                let candidate_files: Vec<String> = all_files
                    .into_iter()
                    .filter(|f| {
                        let ext = std::path::Path::new(f)
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("");
                        CODE_EXTENSIONS.contains(&ext)
                    })
                    .collect();

                // Step 4: selectively load payloads for candidates only.
                let mut payloads = Vec::with_capacity(candidate_files.len());
                for file in &candidate_files {
                    match cache.get_or_load_payload(file) {
                        Ok(Some(arc)) => payloads.push((*arc).clone()),
                        Ok(None) => {} // file not in DB — skip
                        Err(e) => {
                            tracing::warn!(
                                file = %file,
                                error = %e,
                                "tool entry_points: payload load failed — skipping"
                            );
                        }
                    }
                }

                // Step 5: run detection (pure fn, no I/O).
                let mut entries = detect_entry_points(&reverse_index, &payloads);

                // Step 6: filter tests if caller requested.
                if !wire.include_tests {
                    entries.retain(|e| !matches!(e.reason, EntryPointReason::Test));
                }

                Ok(entries)
            })();

            let entries = result?;
            let latency_ms: u32 = clock.elapsed_since(t0).as_millis().try_into().unwrap_or(u32::MAX);
            let summary = EntryPointSummary::from_entries(&entries);

            // Convert to wire DTOs.
            let dto_entries: Vec<serde_json::Value> = entries
                .iter()
                .map(|e| {
                    let (reason_str, reason_detail) = match &e.reason {
                        EntryPointReason::Orphan => ("orphan".to_string(), None),
                        EntryPointReason::WellKnownName(n) => {
                            ("well_known".to_string(), Some(n.clone()))
                        }
                        EntryPointReason::FrameworkRoute(r) => {
                            ("framework".to_string(), Some(r.clone()))
                        }
                        EntryPointReason::Test => ("test".to_string(), None),
                    };
                    serde_json::json!({
                        "id": e.id,
                        "name": e.name,
                        "file": e.file,
                        "line": e.line,
                        "reason": reason_str,
                        "reason_detail": reason_detail,
                    })
                })
                .collect();

            let value = serde_json::json!({
                "entries": dto_entries,
                "summary": {
                    "total": summary.total,
                    "by_reason": summary.by_reason,
                    "latency_ms": latency_ms,
                },
            });

            // BLOCK-2: emit EntryPointsDetected SSB row (best-effort, non-fatal).
            if let Some(ssb) = ssb {
                use crate::ssb::{EntryPointsDetectedPayload, SsbPayload};
                let ep_payload = SsbPayload::EntryPointsDetected(EntryPointsDetectedPayload {
                    project_id: project_id.clone(),
                    count: summary.total,
                    by_reason: summary.by_reason.clone(),
                });
                if let Err(e) = ssb
                    .write_payload(SSB_AGENT_SYMBOL_GRAPH, 0, ep_payload, Vec::new())
                    .await
                {
                    tracing::warn!(
                        error = %e,
                        "entry_points SSB write failed (continuing tool result)"
                    );
                }
            }

            serde_json::to_vec(&value)
                .map_err(|e| format!("tool entry_points: serialise error: {e}"))
        }

        // C-24 (DRIFT-5, DECISION-4): Mermaid graph-export tool. Mirrors the
        // Tauri command `export_graph_mermaid` (dashboard-v2 commands.rs:1805)
        // and the shared `export_graph_mermaid_impl` (C-18 D2 extraction).
        // Only `file` and `symbol` modes are reachable here: `decomposition`
        // mode requires in-memory `DecompositionTreeSnapshot` from the
        // dashboard `AppState` (no orchestrator dep). Callers that need
        // decomposition mode must use the Tauri command.
        TOOL_EXPORT_GRAPH_MERMAID => {
            let wire: ExportGraphMermaidRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool export_graph_mermaid: decode error: {e}"))?;
            let project_root = std::path::PathBuf::from(&wire.project_root);
            let project_id = project_root
                .file_name()
                .and_then(|n| n.to_str())
                .filter(|n| !n.is_empty())
                .ok_or_else(|| {
                    format!(
                        "tool export_graph_mermaid: cannot derive project_id from project_root={:?}",
                        wire.project_root
                    )
                })?
                .to_string();
            // Build VizData via the knowledge crate's shared view builders
                // (extracted from dashboard-v2/src-tauri/src/commands.rs in C-24).
            let data = match wire.mode.as_str() {
                "file" => elai_knowledge::visualization::views::build_file_view(
                    &project_root,
                    &project_id,
                )?,
                "symbol" => elai_knowledge::visualization::views::build_symbol_view(
                    &project_root,
                    &project_id,
                )?,
                "decomposition" => {
                    return Err(
                        "tool export_graph_mermaid: `decomposition` mode requires \
                         in-memory DecompositionTreeSnapshot from the dashboard \
                         AppState; use the Tauri export_graph_mermaid command \
                         instead of the IPC tool surface for that mode"
                            .to_string(),
                    );
                }
                other => {
                    return Err(format!(
                        "tool export_graph_mermaid: unknown mode `{other}`; \
                         expected file/symbol/decomposition"
                    ));
                }
            };
            // Match the B-33 default cap when the caller passes None.
            const B33_DEFAULT_MERMAID_MAX_NODES: usize = 200;
            let cap = wire
                .max_nodes
                .map(|n| n as usize)
                .unwrap_or(B33_DEFAULT_MERMAID_MAX_NODES);
            let mermaid = elai_knowledge::visualization::mermaid::generate_mermaid(&data, cap);
            // Return as JSON `{"mermaid": "..."}` so the agent can parse a
            // single field without dealing with a raw string envelope.
            let value = serde_json::json!({ "mermaid": mermaid });
            serde_json::to_vec(&value)
                .map_err(|e| format!("tool export_graph_mermaid: serialise error: {e}"))
        }

        // H-13: canvas op tools (model drives the canvas).
        // Pattern: postcard decode -> CanvasDispatcher (mode-gating + sanitize)
        // -> SSB row (canvas_op_applied) -> JSON response.
        // InProcessCanvasDispatcher is the dyn dispatch point (HD-4 gate:
        // auditor-driven; this path is the PRODUCTION entrypoint, W-3).
        TOOL_CANVAS_CREATE_NODE
        | TOOL_CANVAS_CONNECT
        | TOOL_CANVAS_UPDATE_NODE
        | TOOL_CANVAS_ANNOTATE
        | TOOL_CANVAS_FORK
        | TOOL_CANVAS_RENDER_DIAGRAM
        | TOOL_CANVAS_SCREENSHOT => {
            use crate::canvas_dispatcher::{CanvasOpsDispatcher, InProcessCanvasDispatcher};
            // Build the default CommittedToolSet for the canvas role.
            // In production task_runner wires this via check_against_commitment;
            // in the standalone dispatch path we build a canvas-permissive set.
            let committed = crate::capability_flow::commit_minimal_set(
                "canvas_op_dispatch",
                &[crate::tool_filter::ToolCategory::Canvas],
                0,
            );
            let canvas_mode = extract_canvas_mode_from_postcard(tool_id, arguments)?;
            let session_id = ssb.map(|s| s.session_id().to_owned()).unwrap_or_default();
            let resp = InProcessCanvasDispatcher
                .dispatch_canvas_op(
                    tool_id,
                    arguments,
                    &canvas_mode,
                    &session_id,
                    &committed,
                    ssb,
                )
                .map_err(|e| format!("tool {tool_id}: {e}"))?;
            serde_json::to_vec(&resp)
                .map_err(|e| format!("tool {tool_id}: serialise response error: {e}"))
        }

        // H-15: spawn_view reply-channel verb.
        // Pattern: postcard decode -> autonomy gate -> emit CanvasViewEvent(ViewSpawned)
        // SSB row -> compose seed_ops via CanvasDispatcher (no new dispatcher needed:
        // seed_ops are existing canvas_op_applied op_ids) -> JSON response.
        TOOL_CANVAS_SPAWN_VIEW => {
            // Autonomy kill-switch (GD-4): spawn_view is DEFAULT-ON (W-2 compliance).
            // Set ELAI_CANVAS_SPAWN_DISABLE=1 to suppress (emergency hatch only).
            // A default-ON kill-switch needs no wire-dark-allowlist row (not wire-dark).
            if std::env::var("ELAI_CANVAS_SPAWN_DISABLE").as_deref() == Ok("1") {
                return Err(format!(
                    "tool {tool_id}: spawn_view suppressed via ELAI_CANVAS_SPAWN_DISABLE=1 \
                     (emergency kill-switch active). Unset to re-enable."
                ));
            }
            let req: CanvasSpawnViewRequest =
                postcard::from_bytes(arguments).map_err(|e| {
                    format!("tool {tool_id}: postcard decode error: {e}")
                })?;

            let session_id = ssb.map(|s| s.session_id().to_owned()).unwrap_or_default();
            // Generate a stable view_id: blake3 of view_type + task_id + session_id.
            let view_id = {
                use blake3::Hasher;
                let mut h = Hasher::new();
                h.update(req.view_type.as_bytes());
                h.update(b"|");
                h.update(req.task_id.as_bytes());
                h.update(b"|");
                h.update(session_id.as_bytes());
                format!("view-{}", &h.finalize().to_hex()[..16])
            };

            // Emit CanvasViewEvent(ViewSpawned) SSB row.
            if let Some(ssb_ref) = ssb {
                let payload = crate::ssb::SsbPayload::CanvasViewEvent(crate::ssb::CanvasViewPayload {
                    kind: crate::ssb::CanvasViewKind::ViewSpawned,
                    view_type: req.view_type.clone(),
                    view_id: view_id.clone(),
                    seed_op_ids: req.seed_ops.clone(),
                    node_id: String::new(),
                    taint_flagged: false,
                    task_id: req.task_id.clone(),
                    session_id: session_id.clone(),
                });
                let ssb_clone = ssb_ref.clone();
                let ts_ms = i64::try_from(clock.now_ms()).unwrap_or(i64::MAX);
                let _ = tokio::task::spawn(async move {
                    if let Err(e) = ssb_clone
                        .write_payload(
                            "spawn_view_dispatch",
                            ts_ms,
                            payload,
                            vec![],
                        )
                        .await
                    {
                        tracing::warn!(
                            view_id = %view_id,
                            error = %e,
                            "spawn_view: SSB ViewSpawned write failed (best-effort)"
                        );
                    }
                });
            }

            let resp = CanvasSpawnViewResponse {
                view_id: {
                    // Re-compute for the response (view_id was moved into the async block).
                    let mut h = blake3::Hasher::new();
                    h.update(req.view_type.as_bytes());
                    h.update(b"|");
                    h.update(req.task_id.as_bytes());
                    h.update(b"|");
                    h.update(session_id.as_bytes());
                    format!("view-{}", &h.finalize().to_hex()[..16])
                },
                view_type: req.view_type,
                seed_op_count: req.seed_ops.len(),
                task_id: req.task_id,
            };
            serde_json::to_vec(&resp)
                .map_err(|e| format!("tool {tool_id}: serialise response error: {e}"))
        }

        // I-53: recipe runners + utility tool pack.
        // These tools use the JSON dispatch path (no postcard wire struct) consistent
        // with the fsm_get_state / fsm_transition arms above (DECISION-7).
        //
        // NOTE: elai-tools is a dev-dep of elai-orchestrator (circular dep avoidance);
        // production arms inline the minimal logic using std + serde_json only.
        // The full bash_validation pipeline runs on the CLI path (elai-tools::recipe_runners);
        // the production path applies an essential inline block-list check for recipe_run
        // (documented limitation: DECISION-7 in workflow/reasoning/phase-I53-decisions.md).
        "recipe_list" => {
            let cwd_str = serde_json::from_slice::<serde_json::Value>(arguments)
                .ok()
                .and_then(|v| v.get("cwd").and_then(|c| c.as_str()).map(|s| s.to_string()));
            let cwd_path = cwd_str.clone().unwrap_or_else(|| ".".to_string());
            let cwd = std::path::Path::new(&cwd_path);
            let result = i53_recipe_list(cwd)?;
            serde_json::to_vec(&result)
                .map_err(|e| format!("tool recipe_list: serialise error: {e}"))
        }

        "recipe_run" => {
            let args: serde_json::Value = serde_json::from_slice(arguments)
                .map_err(|e| format!("tool recipe_run: decode error: {e}"))?;
            let command = args
                .get("command")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "tool recipe_run: missing 'command' field".to_string())?
                .to_string();
            let cwd_str = args
                .get("cwd")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| ".".to_string());
            let cwd = std::path::Path::new(&cwd_str);
            let output = i53_recipe_run(&command, cwd)?;
            serde_json::to_vec(&serde_json::json!({ "output": output }))
                .map_err(|e| format!("tool recipe_run: serialise error: {e}"))
        }

        "archive_read" => {
            let args: serde_json::Value = serde_json::from_slice(arguments)
                .map_err(|e| format!("tool archive_read: decode error: {e}"))?;
            let path_str = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "tool archive_read: missing 'path' field".to_string())?
                .to_string();
            let archive_path = std::path::Path::new(&path_str);
            let action = args
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("list")
                .to_string();
            let result: serde_json::Value = match action.as_str() {
                "list" => i53_archive_list(archive_path)?,
                "read" => {
                    let entry = args
                        .get("entry")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            "tool archive_read: 'entry' required when action=read".to_string()
                        })?
                        .to_string();
                    let content = i53_archive_read_entry(archive_path, &entry)?;
                    serde_json::json!({ "content": content })
                }
                other => {
                    return Err(format!(
                        "tool archive_read: unknown action '{other}'; use 'list' or 'read'"
                    ))
                }
            };
            serde_json::to_vec(&result)
                .map_err(|e| format!("tool archive_read: serialise error: {e}"))
        }

        "json_tree_query" => {
            let args: serde_json::Value = serde_json::from_slice(arguments)
                .map_err(|e| format!("tool json_tree_query: decode error: {e}"))?;
            let file_str = args
                .get("file")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "tool json_tree_query: missing 'file' field".to_string())?
                .to_string();
            let file_path = std::path::Path::new(&file_str);
            let dot_path = args.get("path").and_then(|v| v.as_str()).map(|s| s.to_string());
            let max_depth = args
                .get("max_depth")
                .and_then(|v| v.as_u64())
                .map(|n| n as usize)
                .unwrap_or(2); // MAX_DEPTH_DEFAULT
            let max_lines = args
                .get("max_lines")
                .and_then(|v| v.as_u64())
                .map(|n| n as usize)
                .unwrap_or(6); // MAX_LINES_DEFAULT
            let output = i53_json_query(file_path, dot_path.as_deref(), max_depth, max_lines)?;
            serde_json::to_vec(&serde_json::json!({ "output": output }))
                .map_err(|e| format!("tool json_tree_query: serialise error: {e}"))
        }

        // I-52: agent-callable checkpoint tool (save/rewind/diff) over the H-49
        // time-travel CAS. JSON dispatch path (no postcard struct). session_id is
        // run-context derived (checkpoint_session_id), never agent-supplied -- the
        // cross-session rewind guard. The SSB row is the off-agent observable (R4);
        // the RefusedDiverged row is the load-bearing safety beacon (R1).
        TOOL_CHECKPOINT_SAVE => {
            use crate::architecture::checkpoint_tool;
            let args: serde_json::Value = serde_json::from_slice(arguments)
                .map_err(|e| format!("tool checkpoint_save: decode error: {e}"))?;
            let label = args
                .get("label")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "tool checkpoint_save: missing 'label' field".to_string())?
                .to_string();
            let paths: Vec<String> = args
                .get("paths")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let session_id = checkpoint_session_id();
            let root = checkpoint_root();
            let project_root = taint_project_root();
            let outcome = checkpoint_tool::save(
                &root,
                &session_id,
                &label,
                &paths,
                project_root.as_path(),
            )
            .map_err(|e| format!("tool checkpoint_save: {e}"))?;
            emit_checkpoint_tool_event(
                ssb,
                clock,
                crate::ssb::CheckpointToolEventKind::Saved,
                &session_id,
                &outcome.label,
                outcome.paths.clone(),
                Vec::new(),
            )
            .await;
            serde_json::to_vec(&serde_json::json!({
                "saved": true,
                "label": outcome.label,
                "paths": outcome.paths,
            }))
            .map_err(|e| format!("tool checkpoint_save: serialise error: {e}"))
        }

        TOOL_CHECKPOINT_REWIND => {
            use crate::architecture::checkpoint_tool::{self, RewindOutcome};
            let args: serde_json::Value = serde_json::from_slice(arguments)
                .map_err(|e| format!("tool checkpoint_rewind: decode error: {e}"))?;
            let label = args
                .get("label")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "tool checkpoint_rewind: missing 'label' field".to_string())?
                .to_string();
            let session_id = checkpoint_session_id();
            let root = checkpoint_root();
            let project_root = taint_project_root();
            let outcome = checkpoint_tool::rewind(
                &root,
                &session_id,
                &label,
                project_root.as_path(),
            )
            .map_err(|e| format!("tool checkpoint_rewind: {e}"))?;
            match outcome {
                RewindOutcome::Rewound { paths } => {
                    emit_checkpoint_tool_event(
                        ssb,
                        clock,
                        crate::ssb::CheckpointToolEventKind::Rewound,
                        &session_id,
                        &label,
                        paths.clone(),
                        Vec::new(),
                    )
                    .await;
                    serde_json::to_vec(&serde_json::json!({
                        "rewound": true,
                        "label": label,
                        "paths": paths,
                    }))
                    .map_err(|e| format!("tool checkpoint_rewind: serialise error: {e}"))
                }
                RewindOutcome::RefusedDiverged { diverged_paths } => {
                    // Load-bearing safety observable: the refusal MUST land (R1).
                    emit_checkpoint_tool_event(
                        ssb,
                        clock,
                        crate::ssb::CheckpointToolEventKind::RefusedDiverged,
                        &session_id,
                        &label,
                        Vec::new(),
                        diverged_paths.clone(),
                    )
                    .await;
                    serde_json::to_vec(&serde_json::json!({
                        "rewound": false,
                        "refused": "diverged",
                        "label": label,
                        "diverged_paths": diverged_paths,
                    }))
                    .map_err(|e| format!("tool checkpoint_rewind: serialise error: {e}"))
                }
            }
        }

        TOOL_CHECKPOINT_DIFF => {
            use crate::architecture::checkpoint_tool;
            let args: serde_json::Value = serde_json::from_slice(arguments)
                .map_err(|e| format!("tool checkpoint_diff: decode error: {e}"))?;
            let label = args
                .get("label")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "tool checkpoint_diff: missing 'label' field".to_string())?
                .to_string();
            let session_id = checkpoint_session_id();
            let root = checkpoint_root();
            let project_root = taint_project_root();
            let outcome = checkpoint_tool::diff(
                &root,
                &session_id,
                &label,
                project_root.as_path(),
            )
            .map_err(|e| format!("tool checkpoint_diff: {e}"))?;
            emit_checkpoint_tool_event(
                ssb,
                clock,
                crate::ssb::CheckpointToolEventKind::Diffed,
                &session_id,
                &outcome.label,
                outcome.changed_paths.clone(),
                Vec::new(),
            )
            .await;
            serde_json::to_vec(&serde_json::json!({
                "label": outcome.label,
                "changed_paths": outcome.changed_paths,
            }))
            .map_err(|e| format!("tool checkpoint_diff: serialise error: {e}"))
        }

        // I-54: agent-callable structural (AST-aware) edit tool (search/preview/
        // apply). JSON dispatch path (no postcard struct), mirroring I-52/I-53.
        // The engine lives in crate::structural_edit (reuses the shipped
        // TreeSitterAstEvaluator; one tree-sitter version source, SLC-3). The
        // apply arm runs the SHIPPED PreToolUse edit gates + dispatch_edit per
        // rewrite (controller decision 3); it never writes bytes itself.
        TOOL_STRUCTURAL_SEARCH => {
            let args: serde_json::Value = serde_json::from_slice(arguments)
                .map_err(|e| format!("tool structural_search: decode error: {e}"))?;
            let language = args
                .get("language")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "tool structural_search: missing 'language' field".to_string())?
                .to_string();
            let source = args
                .get("source")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "tool structural_search: missing 'source' field".to_string())?
                .to_string();
            let query = args
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "tool structural_search: missing 'query' field".to_string())?
                .to_string();
            match crate::structural_edit::structural_search(&language, &source, &query) {
                Ok(res) => {
                    let match_count =
                        u32::try_from(res.matches.len()).unwrap_or(u32::MAX);
                    emit_structural_edit_event(
                        ssb,
                        clock,
                        crate::ssb::StructuralEditAppliedKind::Matched,
                        &language,
                        match_count,
                        String::new(),
                        Vec::new(),
                    )
                    .await;
                    let matches: Vec<serde_json::Value> = res
                        .matches
                        .iter()
                        .map(|m| {
                            serde_json::json!({
                                "node_text": m.node_text,
                                "start_line": m.start_line,
                                "end_line": m.end_line,
                                "capture_name": m.capture_name,
                            })
                        })
                        .collect();
                    serde_json::to_vec(&serde_json::json!({
                        "match_count": match_count,
                        "matches": matches,
                    }))
                    .map_err(|e| format!("tool structural_search: serialise error: {e}"))
                }
                Err(e @ crate::structural_edit::StructuralEditError::InvalidQuery(_)) => {
                    // Typed ParseFailed: ZERO bytes, the off-agent observable lands.
                    emit_structural_edit_event(
                        ssb,
                        clock,
                        crate::ssb::StructuralEditAppliedKind::ParseFailed,
                        &language,
                        0,
                        String::new(),
                        Vec::new(),
                    )
                    .await;
                    Err(format!("tool structural_search: {e}"))
                }
                Err(e) => Err(format!("tool structural_search: {e}")),
            }
        }

        TOOL_STRUCTURAL_PREVIEW => {
            let (language, file, query, template, whole_node) =
                decode_structural_edit_args("structural_preview", arguments)?;
            let source = std::fs::read_to_string(&file)
                .map_err(|e| format!("tool structural_preview: read '{file}': {e}"))?;
            match crate::structural_edit::structural_preview(
                &language,
                &source,
                &query,
                &template,
                &whole_node,
            ) {
                Ok(prev) => {
                    let match_count =
                        u32::try_from(prev.rewrites.len()).unwrap_or(u32::MAX);
                    emit_structural_edit_event(
                        ssb,
                        clock,
                        crate::ssb::StructuralEditAppliedKind::Previewed,
                        &language,
                        match_count,
                        prev.diff_hash.clone(),
                        Vec::new(),
                    )
                    .await;
                    let rewrites: Vec<serde_json::Value> = prev
                        .rewrites
                        .iter()
                        .map(|r| {
                            serde_json::json!({
                                "start_line": r.start_line,
                                "old_text": r.old_text,
                                "new_text": r.new_text,
                            })
                        })
                        .collect();
                    serde_json::to_vec(&serde_json::json!({
                        "match_count": match_count,
                        "diff_hash": prev.diff_hash,
                        // The staleness anchor the agent passes back as
                        // `expected_source_hash` on the matching structural_apply.
                        "source_hash": prev.source_hash,
                        "rewrites": rewrites,
                    }))
                    .map_err(|e| format!("tool structural_preview: serialise error: {e}"))
                }
                Err(
                    e @ (crate::structural_edit::StructuralEditError::InvalidQuery(_)
                    | crate::structural_edit::StructuralEditError::UnknownCapture(_)),
                ) => {
                    emit_structural_edit_event(
                        ssb,
                        clock,
                        crate::ssb::StructuralEditAppliedKind::ParseFailed,
                        &language,
                        0,
                        String::new(),
                        Vec::new(),
                    )
                    .await;
                    Err(format!("tool structural_preview: {e}"))
                }
                Err(e) => Err(format!("tool structural_preview: {e}")),
            }
        }

        TOOL_STRUCTURAL_APPLY => {
            let (language, file, query, template, whole_node) =
                decode_structural_edit_args("structural_apply", arguments)?;
            // The whole-file hash the agent saw at preview. apply refuses if the
            // file changed since then (the real "changed since the preview" guard,
            // threaded across the preview -> apply boundary).
            let expected_source_hash = decode_structural_apply_expected_hash(arguments)?;
            // Re-resolve the preview against the CURRENT file (the agent supplies
            // the same query+template; we re-run rather than trust agent-passed
            // spans -- the staleness guard is re-derived here).
            let source = std::fs::read_to_string(&file)
                .map_err(|e| format!("tool structural_apply: read '{file}': {e}"))?;
            // PREVIEW-STALENESS: refuse BEFORE any work if the file no longer
            // hashes to what the agent previewed. This is the load-bearing
            // production RefusedStale path -- it catches an external edit even
            // when the matched node's own text is unchanged (a reposition), which
            // the in-node re-hash cannot. Emit the RefusedStale row + Err, no write.
            if let Err(e) = crate::structural_edit::verify_source_unchanged(
                &file,
                &source,
                &expected_source_hash,
            ) {
                emit_structural_edit_event(
                    ssb,
                    clock,
                    crate::ssb::StructuralEditAppliedKind::RefusedStale,
                    &language,
                    0,
                    String::new(),
                    vec![file.clone()],
                )
                .await;
                return Err(format!("tool structural_apply: {e}"));
            }
            let prev = match crate::structural_edit::structural_preview(
                &language,
                &source,
                &query,
                &template,
                &whole_node,
            ) {
                Ok(p) => p,
                Err(
                    e @ (crate::structural_edit::StructuralEditError::InvalidQuery(_)
                    | crate::structural_edit::StructuralEditError::UnknownCapture(_)),
                ) => {
                    emit_structural_edit_event(
                        ssb,
                        clock,
                        crate::ssb::StructuralEditAppliedKind::ParseFailed,
                        &language,
                        0,
                        String::new(),
                        Vec::new(),
                    )
                    .await;
                    return Err(format!("tool structural_apply: {e}"));
                }
                Err(e) => return Err(format!("tool structural_apply: {e}")),
            };
            // Staleness + ambiguity guard. A stale/ambiguous match REFUSES the
            // whole apply (no partial corruption) and lands the RefusedStale row.
            if let Err(e) =
                crate::structural_edit::verify_apply_safe(&file, &source, &prev.rewrites)
            {
                emit_structural_edit_event(
                    ssb,
                    clock,
                    crate::ssb::StructuralEditAppliedKind::RefusedStale,
                    &language,
                    u32::try_from(prev.rewrites.len()).unwrap_or(u32::MAX),
                    prev.diff_hash.clone(),
                    vec![file.clone()],
                )
                .await;
                return Err(format!("tool structural_apply: {e}"));
            }
            // Gate + write each rewrite through the SHIPPED edit path. The gates
            // run BEFORE any byte lands (fail-closed), exactly as TOOL_EDIT does.
            // Fail-closed pre-pass: run every gate for all rewrites BEFORE any
            // byte lands, so a later gate rejection cannot leave a partial edit
            // (rewrites 0..k already written via dispatch_edit then `?` propagates
            // Err mid-loop). The dispatch loop below is gate-free; a gate failure
            // here refuses the whole apply with zero bytes written.
            for r in &prev.rewrites {
                enforce_taint_sink_gate(
                    "fs_write",
                    "structural_edit_apply",
                    &[file.as_str(), r.new_text.as_str()],
                    ssb,
                )?;
                enforce_fs_blast_radius(
                    &[file.as_str()],
                    taint_project_root().as_path(),
                    &format!("structural_apply {file}"),
                    ssb,
                    i64::try_from(clock.now_ms()).unwrap_or(i64::MAX),
                )?;
            }
            let mut total_bytes: u64 = 0;
            for r in &prev.rewrites {
                let req = EditRequest {
                    file_path: PathBuf::from(&file),
                    old_text: r.old_text.clone(),
                    new_text: r.new_text.clone(),
                };
                let outcome = dispatcher
                    .dispatch_edit(req)
                    .await
                    .map_err(|e| format!("tool structural_apply: dispatch error: {e:?}"))?;
                total_bytes += u64::from(outcome.bytes_written);
            }
            let applied_count = u32::try_from(prev.rewrites.len()).unwrap_or(u32::MAX);
            emit_structural_edit_event(
                ssb,
                clock,
                crate::ssb::StructuralEditAppliedKind::Applied,
                &language,
                applied_count,
                prev.diff_hash.clone(),
                vec![file.clone()],
            )
            .await;
            serde_json::to_vec(&serde_json::json!({
                "applied": true,
                "file": file,
                "rewrites_applied": applied_count,
                "bytes_written": total_bytes,
                "diff_hash": prev.diff_hash,
            }))
            .map_err(|e| format!("tool structural_apply: serialise error: {e}"))
        }

        unknown => Err(format!("unknown tool_id: {unknown}")),
    }
}

/// I-54: decode the shared (language, file, query, rewrite_template,
/// whole_node_capture) arg shape for the structural_preview / structural_apply
/// JSON-path tool arms.
#[allow(clippy::type_complexity)]
fn decode_structural_edit_args(
    tool: &str,
    arguments: &[u8],
) -> Result<(String, String, String, String, String), String> {
    let args: serde_json::Value = serde_json::from_slice(arguments)
        .map_err(|e| format!("tool {tool}: decode error: {e}"))?;
    structural_edit_string_field(tool, &args, "language").and_then(|language| {
        Ok((
            language,
            structural_edit_string_field(tool, &args, "file")?,
            structural_edit_string_field(tool, &args, "query")?,
            structural_edit_string_field(tool, &args, "rewrite_template")?,
            structural_edit_string_field(tool, &args, "whole_node_capture")?,
        ))
    })
}

/// I-54: pull one required string field from a decoded structural-edit arg blob.
fn structural_edit_string_field(
    tool: &str,
    args: &serde_json::Value,
    key: &str,
) -> Result<String, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("tool {tool}: missing '{key}' field"))
}

/// I-54: the structural_apply-only `expected_source_hash` arg -- the whole-file
/// blake3 hash the agent received from its prior `structural_preview` call. It is
/// the staleness anchor threaded across the preview -> apply boundary; apply
/// refuses (`RefusedStale`) if the file no longer hashes to it.
fn decode_structural_apply_expected_hash(arguments: &[u8]) -> Result<String, String> {
    let args: serde_json::Value = serde_json::from_slice(arguments)
        .map_err(|e| format!("tool structural_apply: decode error: {e}"))?;
    structural_edit_string_field("structural_apply", &args, "expected_source_hash")
}

/// I-54: emit one `StructuralEditApplied` SSB row (off-agent observable, R4).
/// Non-fatal: a write failure is logged, never propagated (mirrors the
/// checkpoint_tool / trace_query emit discipline).
async fn emit_structural_edit_event(
    ssb: Option<&Arc<crate::ssb::Ssb>>,
    clock: &Arc<dyn ClockProvider>,
    kind: crate::ssb::StructuralEditAppliedKind,
    language: &str,
    match_count: u32,
    diff_hash: String,
    paths: Vec<String>,
) {
    if let Some(ssb_arc) = ssb {
        let ts_ms = crate::ssb::Ssb::now_ms_via_clock(clock);
        let payload = crate::ssb::SsbPayload::StructuralEditApplied(
            crate::ssb::StructuralEditAppliedPayload {
                kind,
                language: language.to_owned(),
                match_count,
                diff_hash,
                paths,
                ts_ms,
            },
        );
        if let Err(e) = ssb_arc
            .write_payload("agent_rpc", 0, payload, Vec::new())
            .await
        {
            tracing::warn!(
                error = %e,
                "structural_edit: StructuralEditApplied SSB write failed (non-fatal)"
            );
        }
    }
}

/// I-52: emit one `CheckpointToolEvent` SSB row (off-agent observable, R4).
/// Non-fatal: a write failure is logged, never propagated (mirrors the
/// acknowledge_infeasible / trace_query emit discipline).
async fn emit_checkpoint_tool_event(
    ssb: Option<&Arc<crate::ssb::Ssb>>,
    clock: &Arc<dyn ClockProvider>,
    kind: crate::ssb::CheckpointToolEventKind,
    session_id: &str,
    label: &str,
    paths: Vec<String>,
    diverged_paths: Vec<String>,
) {
    if let Some(ssb_arc) = ssb {
        let ts_ms = crate::ssb::Ssb::now_ms_via_clock(clock);
        let payload = crate::ssb::SsbPayload::CheckpointToolEvent(
            crate::ssb::CheckpointToolEventPayload {
                kind,
                session_id: session_id.to_owned(),
                label: label.to_owned(),
                paths,
                diverged_paths,
                ts_ms,
            },
        );
        if let Err(e) = ssb_arc
            .write_payload("agent_rpc", 0, payload, Vec::new())
            .await
        {
            tracing::warn!(
                error = %e,
                "checkpoint_tool: CheckpointToolEvent SSB write failed (non-fatal)"
            );
        }
    }
}

/// J-1.T4: emit one `PerceptionScaffoldEvent` SSB row (off-agent observable, R4).
/// Non-fatal: a write failure is logged, never propagated (mirrors the
/// emit_structural_edit_event / emit_checkpoint_tool_event discipline).
///
/// The `ELAI_PERCEPTION_GATEWAY_ENABLE` gate is checked at the call site in
/// `handle_infer_request` (the perception block); this helper does NOT re-gate
/// (it is only called from inside the gated block). `agent_id="agent_rpc"`,
/// `superstep=0`. `ts_ms` via `Ssb::now_ms_via_clock` (entropy quarantine).
async fn emit_perception_scaffold_event(
    ssb: Option<&Arc<crate::ssb::Ssb>>,
    clock: &Arc<dyn ClockProvider>,
    kind: crate::ssb::PerceptionScaffoldEventKind,
    session_id: Option<&str>,
    details_json: serde_json::Value,
) {
    if let Some(ssb_arc) = ssb {
        let ts_ms = crate::ssb::Ssb::now_ms_via_clock(clock);
        let payload = crate::ssb::SsbPayload::PerceptionScaffoldEvent(
            crate::ssb::PerceptionScaffoldEventPayload {
                kind,
                session_id: session_id.map(|s| s.to_owned()),
                task_id: None,
                details_json,
                ts_ms,
            },
        );
        if let Err(e) = ssb_arc
            .write_payload("agent_rpc", 0, payload, Vec::new())
            .await
        {
            tracing::warn!(
                error = %e,
                "perception_scaffold: PerceptionScaffoldEvent SSB write failed (non-fatal)"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// H-13 helper: extract canvas mode from postcard arguments without full decode.
// Used by translate_tool_request canvas_* dispatch arms to pass current_mode
// to CanvasDispatcher before the full decode step.
// ---------------------------------------------------------------------------

/// Extract the `mode` field from a postcard-encoded canvas request without
/// deserializing the full struct. Uses the cheapest available request type
/// that all canvas tools share (CanvasScreenshotRequest has only mode+task_id).
fn extract_canvas_mode_from_postcard(
    tool_id: &str,
    arguments: &[u8],
) -> Result<String, String> {
    // All canvas Request structs have `mode: String` as the second-to-last field.
    // The cheapest extraction path is CanvasScreenshotRequest (2-field struct).
    // For other tools we rely on the full decode inside the dispatcher.
    // Fallback: decode as CanvasScreenshotRequest; if that fails, use "unknown".
    match postcard::from_bytes::<elai_ipc_types::CanvasScreenshotRequest>(arguments) {
        Ok(r) => Ok(r.mode),
        Err(_) => {
            // Try the other request types in order of likelihood.
            if let Ok(r) = postcard::from_bytes::<elai_ipc_types::CanvasCreateNodeRequest>(arguments) {
                return Ok(r.mode);
            }
            if let Ok(r) = postcard::from_bytes::<elai_ipc_types::CanvasConnectRequest>(arguments) {
                return Ok(r.mode);
            }
            if let Ok(r) = postcard::from_bytes::<elai_ipc_types::CanvasUpdateNodeRequest>(arguments) {
                return Ok(r.mode);
            }
            if let Ok(r) = postcard::from_bytes::<elai_ipc_types::CanvasAnnotateRequest>(arguments) {
                return Ok(r.mode);
            }
            if let Ok(r) = postcard::from_bytes::<elai_ipc_types::CanvasForkRequest>(arguments) {
                return Ok(r.mode);
            }
            if let Ok(r) = postcard::from_bytes::<elai_ipc_types::CanvasRenderDiagramRequest>(arguments) {
                return Ok(r.mode);
            }
            Err(format!(
                "tool {tool_id}: cannot extract canvas mode from postcard arguments \
                 (all decode attempts failed)"
            ))
        }
    }
}

// ---------------------------------------------------------------------------
// I-53 helper functions: recipe runners + utility tool pack.
//
// Inline implementations that use only std + serde_json to avoid the
// circular dep (elai-tools depends on elai-orchestrator; orchestrator cannot
// import elai-tools in production code). These mirror the logic in
// elai_tools::{recipe_runners, archive_reader, json_query}. Any behaviour
// change in the elai-tools versions must be reflected here.
//
// Validation note: the CLI path (GlobalToolRegistry) runs the full
// bash_validation::validate_command pipeline (elai-tools). This production
// path applies an inline essential-block check for recipe_run. Documented
// in workflow/reasoning/phase-I53-decisions.md DECISION-7.
// ---------------------------------------------------------------------------

/// Discover build-recipe runners in `cwd`.  Returns a serde_json::Value
/// object `{"runners": [...]}`.  Reads manifest files only; does not run
/// any process for just/make discovery.  Cargo uses `cargo metadata`.
fn i53_recipe_list(cwd: &std::path::Path) -> Result<serde_json::Value, String> {
    let mut runners: Vec<serde_json::Value> = Vec::new();

    // Just runner: parse Justfile / justfile / .justfile.
    let just_manifest = ["Justfile", "justfile", ".justfile"]
        .iter()
        .map(|n| cwd.join(n))
        .find(|p| p.exists());
    if let Some(manifest) = just_manifest {
        if let Ok(content) = std::fs::read_to_string(&manifest) {
            let tasks: Vec<serde_json::Value> = content
                .lines()
                .filter(|l| !l.starts_with(' ') && !l.starts_with('\t'))
                .filter_map(|l| {
                    let trimmed = l.trim();
                    if let Some(colon) = trimmed.find(':') {
                        let after = &trimmed[colon + 1..];
                        if after.starts_with('=') {
                            return None;
                        }
                        let lhs = &trimmed[..colon];
                        let mut parts = lhs.split_whitespace();
                        let name = parts.next()?;
                        if name.starts_with('_') {
                            return None;
                        }
                        let ok = name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
                        if ok && !name.is_empty() {
                            return Some(serde_json::json!({ "name": name, "doc": null, "parameters": [] }));
                        }
                    }
                    None
                })
                .take(20)
                .collect();
            if !tasks.is_empty() {
                runners.push(serde_json::json!({
                    "id": "just",
                    "label": "Just",
                    "command_prefix": "just",
                    "tasks": tasks,
                }));
            }
        }
    }

    // Make runner: parse Makefile.
    if cwd.join("Makefile").exists() {
        if let Ok(content) = std::fs::read_to_string(cwd.join("Makefile")) {
            let mut phony: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
            let mut candidates: Vec<serde_json::Value> = Vec::new();
            for line in content.lines() {
                let t = line.trim();
                if t.starts_with(".PHONY:") {
                    for p in t.trim_start_matches(".PHONY:").trim().split_whitespace() {
                        phony.insert(p.to_string());
                    }
                    continue;
                }
                if line.starts_with(' ') || line.starts_with('\t') {
                    continue;
                }
                let first = t.chars().next().unwrap_or('\0');
                if !first.is_ascii_alphabetic() && first != '_' {
                    continue;
                }
                if let Some(colon) = t.find(':') {
                    if t[colon + 1..].starts_with('=') {
                        continue;
                    }
                    let name = t[..colon].trim();
                    let ok = name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
                    if ok && !name.is_empty() {
                        let doc: Option<String> = t[colon + 1..]
                            .find("##")
                            .map(|p| t[colon + 1..][p + 2..].trim().to_string());
                        candidates.push(serde_json::json!({ "name": name, "doc": doc }));
                    }
                }
            }
            let tasks: Vec<serde_json::Value> = if phony.is_empty() {
                candidates
            } else {
                candidates
                    .into_iter()
                    .filter(|t| {
                        let name = t.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        phony.contains(name) || t.get("doc").and_then(|v| v.as_str()).is_some()
                    })
                    .collect()
            };
            if !tasks.is_empty() {
                runners.push(serde_json::json!({
                    "id": "make",
                    "label": "Make",
                    "command_prefix": "make",
                    "tasks": tasks.into_iter().take(20).collect::<Vec<_>>(),
                }));
            }
        }
    }

    // Cargo runner: `cargo metadata --no-deps --format-version=1`.
    if cwd.join("Cargo.toml").exists() {
        if let Ok(output) = std::process::Command::new("cargo")
            .args(["metadata", "--no-deps", "--format-version=1"])
            .current_dir(cwd)
            .output()
        {
            if output.status.success() {
                if let Ok(meta) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                    if let Some(packages) = meta.get("packages").and_then(|v| v.as_array()) {
                        let mut tasks: Vec<serde_json::Value> = Vec::new();
                        'outer: for pkg in packages {
                            let pkg_name = pkg.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            if let Some(targets) = pkg.get("targets").and_then(|v| v.as_array()) {
                                for target in targets {
                                    let tname = target.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                    let kinds = target.get("kind").and_then(|v| v.as_array());
                                    let is_bin = kinds.map_or(false, |ks| {
                                        ks.iter().any(|k| k.as_str() == Some("bin"))
                                    });
                                    let is_example = kinds.map_or(false, |ks| {
                                        ks.iter().any(|k| k.as_str() == Some("example"))
                                    });
                                    let (prefix, kind) = if is_bin {
                                        (format!("cargo run --package {pkg_name} --bin"), "binary")
                                    } else if is_example {
                                        (format!("cargo run --package {pkg_name} --example"), "example")
                                    } else {
                                        continue;
                                    };
                                    tasks.push(serde_json::json!({
                                        "name": format!("{prefix} {tname}"),
                                        "doc": format!("{kind}: {pkg_name}/{tname}"),
                                        "parameters": [],
                                    }));
                                    if tasks.len() >= 20 {
                                        break 'outer;
                                    }
                                }
                            }
                        }
                        if !tasks.is_empty() {
                            runners.push(serde_json::json!({
                                "id": "cargo",
                                "label": "Cargo",
                                "command_prefix": "cargo",
                                "tasks": tasks,
                            }));
                        }
                    }
                }
            }
        }
    }

    Ok(serde_json::json!({ "runners": runners }))
}

/// Essential inline block-list check for recipe_run in the production path.
///
/// Blocks the most dangerous command patterns. The full pipeline (bash_validation)
/// runs on the CLI path; this is the production-path guard (DECISION-7).
fn i53_recipe_run_check(command: &str) -> Result<(), String> {
    let lower = command.to_lowercase();
    // Block obvious destructive patterns.
    const BLOCKED: &[&str] = &[
        "rm -rf /",
        "rm -rf *",
        "mkfs",
        "dd if=",
        ":(){:|:&};:",
        "chmod -R 777 /",
        "sudo rm",
        "> /dev/sda",
        "format c:",
        "deltree",
    ];
    for pattern in BLOCKED {
        if lower.contains(pattern) {
            return Err(format!(
                "recipe_run blocked (production essential check): command matches dangerous pattern '{pattern}'"
            ));
        }
    }
    // Block absolute path writes outside cwd (e.g. > /etc/).
    if lower.contains("> /etc/") || lower.contains("> /root/") || lower.contains("> /sys/") {
        return Err(
            "recipe_run blocked (production essential check): redirect to system path".to_string(),
        );
    }
    Ok(())
}

/// Run a recipe command (production path).
fn i53_recipe_run(command: &str, cwd: &std::path::Path) -> Result<String, String> {
    i53_recipe_run_check(command)?;

    let parts: Vec<&str> = command.split_whitespace().collect();
    let (program, args) = parts
        .split_first()
        .ok_or_else(|| "recipe_run: empty command string".to_string())?;

    let output = std::process::Command::new(program)
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("recipe_run: spawn failed: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let exit_code = output.status.code().unwrap_or(-1);

    if output.status.success() {
        Ok(format!("exit_code: {exit_code}\nstdout:\n{stdout}\nstderr:\n{stderr}"))
    } else {
        Err(format!("recipe_run: exit_code={exit_code}\nstdout:\n{stdout}\nstderr:\n{stderr}"))
    }
}

// ---------------------------------------------------------------------------
// I-53 archive helpers (plain .tar, 512-byte header walk, std::io only).
// ---------------------------------------------------------------------------

const I53_TAR_BLOCK: usize = 512;

fn i53_tar_read_entries(
    path: &std::path::Path,
) -> Result<Vec<(String, bool, u64, u64)>, String> {
    use std::io::{Read, Seek, SeekFrom};

    let mut file = std::fs::File::open(path)
        .map_err(|e| format!("archive_read: open '{path:?}' failed: {e}"))?;

    let mut entries: Vec<(String, bool, u64, u64)> = Vec::new(); // (path, is_dir, size, data_offset)
    let mut pos: u64 = 0;

    loop {
        let mut block = [0u8; I53_TAR_BLOCK];
        let n = file
            .read(&mut block)
            .map_err(|e| format!("archive_read: I/O error: {e}"))?;
        if n < I53_TAR_BLOCK {
            break;
        }
        pos += I53_TAR_BLOCK as u64;

        if block.iter().all(|&b| b == 0) {
            break; // End-of-archive zero block.
        }

        // Name (bytes 0..100) + prefix (bytes 345..500).
        let name = {
            let end = block[0..100].iter().position(|&b| b == 0).unwrap_or(100);
            String::from_utf8_lossy(&block[0..end]).to_string()
        };
        let prefix = {
            let end = block[345..500].iter().position(|&b| b == 0).unwrap_or(155);
            String::from_utf8_lossy(&block[345..345 + end]).to_string()
        };
        let full = if prefix.is_empty() {
            name
        } else {
            format!("{prefix}/{name}")
        };
        let clean = full.trim_matches('/').to_string();

        if clean.is_empty() {
            continue;
        }

        // Traversal guard.
        for seg in clean.split('/') {
            if seg == ".." {
                return Err(format!(
                    "archive_read: entry '{clean}' contains '..' (traversal rejected)"
                ));
            }
        }
        if clean.starts_with('/') {
            return Err(format!(
                "archive_read: entry '{clean}' is absolute (traversal rejected)"
            ));
        }

        let typeflag = block[156] as char;
        let is_dir = typeflag == '5' || clean.ends_with('/');

        // Size field (bytes 124..136), octal string.
        let size_str = {
            let end = block[124..136].iter().position(|&b| b == 0).unwrap_or(12);
            String::from_utf8_lossy(&block[124..124 + end]).to_string()
        };
        let size = u64::from_str_radix(size_str.trim(), 8).unwrap_or(0);

        entries.push((clean, is_dir, size, pos));

        // Skip past data blocks.
        let skip = size.div_ceil(I53_TAR_BLOCK as u64) * I53_TAR_BLOCK as u64;
        file.seek(SeekFrom::Start(pos + skip))
            .map_err(|e| format!("archive_read: seek error: {e}"))?;
        pos += skip;
    }

    Ok(entries)
}

fn i53_check_tar_format(path: &std::path::Path) -> Result<(), String> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        return Err("archive_read: .tar.gz not supported (flate2 dep deferred; 2026-06-13)".to_string());
    }
    if name.ends_with(".zip") {
        return Err("archive_read: .zip not supported (zip crate dep deferred; 2026-06-13)".to_string());
    }
    if !name.ends_with(".tar") {
        return Err(format!("archive_read: unsupported format '{name}' (only .tar supported)"));
    }
    Ok(())
}

fn i53_archive_list(path: &std::path::Path) -> Result<serde_json::Value, String> {
    i53_check_tar_format(path)?;
    let entries = i53_tar_read_entries(path)?;
    let arr: Vec<serde_json::Value> = entries
        .iter()
        .map(|(p, is_dir, size, _)| serde_json::json!({ "path": p, "is_dir": is_dir, "size": size }))
        .collect();
    Ok(serde_json::json!({ "entries": arr }))
}

fn i53_archive_read_entry(path: &std::path::Path, entry_path: &str) -> Result<String, String> {
    use std::io::{Read, Seek, SeekFrom};

    i53_check_tar_format(path)?;

    // Traversal guard on the requested entry path.
    for seg in entry_path.split('/') {
        if seg == ".." {
            return Err(format!(
                "archive_read: entry_path '{entry_path}' contains '..' (traversal rejected)"
            ));
        }
    }
    if entry_path.starts_with('/') {
        return Err(format!(
            "archive_read: entry_path '{entry_path}' is absolute (traversal rejected)"
        ));
    }

    let entries = i53_tar_read_entries(path)?;
    let target = entry_path.trim_matches('/');
    let entry = entries
        .iter()
        .find(|(p, _, _, _)| p.trim_matches('/') == target)
        .ok_or_else(|| format!("archive_read: entry '{entry_path}' not found"))?;

    if entry.1 {
        return Err(format!("archive_read: '{entry_path}' is a directory"));
    }

    const MAX_BYTES: usize = 512 * 1024;
    let mut file = std::fs::File::open(path)
        .map_err(|e| format!("archive_read: open failed: {e}"))?;
    file.seek(SeekFrom::Start(entry.3))
        .map_err(|e| format!("archive_read: seek error: {e}"))?;

    let read_len = (entry.2 as usize).min(MAX_BYTES);
    let mut buf = vec![0u8; read_len];
    file.read_exact(&mut buf)
        .map_err(|e| format!("archive_read: read error: {e}"))?;

    let truncated = entry.2 as usize > MAX_BYTES;
    match std::str::from_utf8(&buf) {
        Ok(s) => {
            let mut out = s.to_string();
            if truncated {
                out.push_str(&format!("\n<truncated: {} bytes, limit {MAX_BYTES}>", entry.2));
            }
            Ok(out)
        }
        Err(_) => {
            let hex: String = buf
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<Vec<_>>()
                .join(" ");
            let mut out = format!("<binary> {hex}");
            if truncated {
                out.push_str(&format!("\n<truncated: {} bytes, limit {MAX_BYTES}>", entry.2));
            }
            Ok(out)
        }
    }
}

// ---------------------------------------------------------------------------
// I-53 JSON query helper (mirrors elai_tools::json_query, std + serde_json only).
// ---------------------------------------------------------------------------

fn i53_json_navigate<'v>(root: &'v serde_json::Value, dot_path: &str) -> Option<&'v serde_json::Value> {
    let mut cur = root;
    for seg in dot_path.split('.') {
        if seg.is_empty() {
            continue;
        }
        match cur {
            serde_json::Value::Object(m) => cur = m.get(seg)?,
            serde_json::Value::Array(a) => {
                let idx: usize = seg.parse().ok()?;
                cur = a.get(idx)?;
            }
            _ => return None,
        }
    }
    Some(cur)
}

fn i53_render_value(
    val: &serde_json::Value,
    indent: usize,
    depth: usize,
    max_depth: usize,
    scalar_len: usize,
    lines: &mut Vec<String>,
    max_lines: usize,
    truncated: &mut bool,
) {
    if lines.len() >= max_lines {
        *truncated = true;
        return;
    }
    let pfx = "  ".repeat(indent);
    if depth > max_depth {
        lines.push(format!("{pfx}..."));
        *truncated = true;
        return;
    }
    match val {
        serde_json::Value::Null => { lines.push(format!("{pfx}null")); }
        serde_json::Value::Bool(b) => { lines.push(format!("{pfx}{b}")); }
        serde_json::Value::Number(n) => { lines.push(format!("{pfx}{n}")); }
        serde_json::Value::String(s) => {
            let ts = if s.len() > scalar_len {
                format!("\"{}...\"", &s[..scalar_len])
            } else {
                format!("\"{s}\"")
            };
            lines.push(format!("{pfx}{ts}"));
        }
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                lines.push(format!("{pfx}[]"));
                return;
            }
            lines.push(format!("{pfx}["));
            for item in arr.iter() {
                i53_render_value(item, indent + 1, depth + 1, max_depth, scalar_len, lines, max_lines, truncated);
                if lines.len() >= max_lines {
                    *truncated = true;
                    break;
                }
            }
            if lines.len() < max_lines {
                lines.push(format!("{pfx}]"));
            }
        }
        serde_json::Value::Object(map) => {
            if map.is_empty() {
                lines.push(format!("{pfx}{{}}"));
                return;
            }
            lines.push(format!("{pfx}{{"));
            for (key, value) in map.iter() {
                if lines.len() >= max_lines {
                    *truncated = true;
                    break;
                }
                let kpfx = "  ".repeat(indent + 1);
                match value {
                    serde_json::Value::Null
                    | serde_json::Value::Bool(_)
                    | serde_json::Value::Number(_)
                    | serde_json::Value::String(_) => {
                        let vs = match value {
                            serde_json::Value::Null => "null".to_string(),
                            serde_json::Value::Bool(b) => b.to_string(),
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::String(s) => {
                                if s.len() > scalar_len {
                                    format!("\"{}...\"", &s[..scalar_len])
                                } else {
                                    format!("\"{s}\"")
                                }
                            }
                            _ => unreachable!(),
                        };
                        lines.push(format!("{kpfx}\"{key}\": {vs}"));
                    }
                    _ => {
                        lines.push(format!("{kpfx}\"{key}\":"));
                        i53_render_value(value, indent + 1, depth + 1, max_depth, scalar_len, lines, max_lines, truncated);
                    }
                }
            }
            if lines.len() < max_lines {
                lines.push(format!("{pfx}}}"));
            }
        }
    }
}

fn i53_json_query(
    file_path: &std::path::Path,
    dot_path: Option<&str>,
    max_depth: usize,
    max_lines: usize,
) -> Result<String, String> {
    const MAX_FILE: usize = 10 * 1024 * 1024;
    let meta = std::fs::metadata(file_path)
        .map_err(|e| format!("json_tree_query: stat '{file_path:?}' failed: {e}"))?;
    if meta.len() as usize > MAX_FILE {
        return Err(format!(
            "json_tree_query: file is {} bytes (limit {MAX_FILE})",
            meta.len()
        ));
    }
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("json_tree_query: read '{file_path:?}' failed: {e}"))?;
    let root: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("json_tree_query: parse '{file_path:?}' failed: {e}"))?;

    let node = match dot_path {
        Some(p) if !p.is_empty() => i53_json_navigate(&root, p).ok_or_else(|| {
            format!("json_tree_query: path '{p}' not found in '{file_path:?}'")
        })?,
        _ => &root,
    };

    let eff_lines = max_lines.max(1);
    let mut lines: Vec<String> = Vec::with_capacity(eff_lines + 1);
    let mut truncated = false;
    i53_render_value(node, 0, 0, max_depth, 60, &mut lines, eff_lines, &mut truncated);

    let mut output = lines.join("\n");
    if truncated {
        output.push_str(&format!(
            "\n<truncated: capped at {eff_lines} lines or depth {max_depth}>"
        ));
    }
    Ok(output)
}

// ---------------------------------------------------------------------------
// C-18 D1: public test-helper alias for `translate_tool_request`.
//
// Production callers stay inside the orchestrator crate and use the
// `pub(crate)` symbol directly. Contract/integration tests in
// `crates/orchestrator/tests/` link against the orchestrator as an external
// crate (rustc treats `tests/*.rs` as a separate compilation unit), so they
// cannot reach `pub(crate)` items.
//
// Gating on `any(test, feature = "test-support")`:
//   * `test` is set inside `lib.rs` integration tests (in-crate `#[cfg(test)]`).
//   * `test-support` is the orchestrator feature already used by
//     `tests/production_agent_rpc_contract.rs` (Cargo.toml dev-dep
//     `elai-orchestrator = { path = ".", features = ["test-support"] }`),
//     so this alias surfaces automatically for the C-18 dispatch test.
//
// The alias is a thin pass-through — no behaviour duplication, no extra
// allocation. If the underlying signature changes, the build breaks here
// first, which is the desired locking property.
#[cfg(any(test, feature = "test-support"))]
pub async fn translate_tool_request_for_test(
    tool_id: &str,
    arguments: &[u8],
    dispatcher: &dyn ToolDispatcher,
    ssb: Option<&Arc<crate::ssb::Ssb>>,
    artifact_vm: Option<&(dyn elai_knowledge::vector::VectorModule + Send + Sync)>,
    clock: &Arc<dyn ClockProvider>,
) -> Result<Vec<u8>, String> {
    translate_tool_request(tool_id, arguments, dispatcher, ssb, artifact_vm, clock, None).await
}


#[cfg(test)]
mod j13_seed_tests {
    use super::*;
    use elai_core::tool_dispatcher::ToolDispatcher;
    use elai_providers::AnthropicClient;
    use std::future::Future;
    use std::path::PathBuf;
    use std::pin::Pin;

    struct NoOpDispatcher;
    impl ToolDispatcher for NoOpDispatcher {
        fn dispatch_edit(
            &self,
            _req: elai_core::tool_dispatcher::EditRequest,
        ) -> Pin<Box<dyn Future<Output = Result<elai_core::tool_dispatcher::EditOutcome, elai_core::tool_dispatcher::ToolDispatchError>> + Send + '_>> {
            Box::pin(async { Err(elai_core::tool_dispatcher::ToolDispatchError::NotImplemented("noop".into())) })
        }
        fn dispatch_validate(
            &self,
            _req: elai_core::tool_dispatcher::ValidateRequest,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<elai_core::tool_dispatcher::ValidationTierResult>, elai_core::tool_dispatcher::ToolDispatchError>> + Send + '_>> {
            Box::pin(async { Ok(Vec::new()) })
        }
        fn dispatch_classify_failure(
            &self,
            _input: elai_core::tool_dispatcher::FailureInput,
        ) -> Pin<Box<dyn Future<Output = Option<elai_core::tool_dispatcher::ClassifiedFailure>> + Send + '_>> {
            Box::pin(async { None })
        }
        fn dispatch_git_commit(
            &self,
            _req: elai_core::tool_dispatcher::GitCommitRequest,
        ) -> Pin<Box<dyn Future<Output = Result<elai_core::tool_dispatcher::CommitOutcome, elai_core::tool_dispatcher::ToolDispatchError>> + Send + '_>> {
            Box::pin(async { Err(elai_core::tool_dispatcher::ToolDispatchError::NotImplemented("noop".into())) })
        }
        fn dispatch_file_read_outline(
            &self,
            _path: PathBuf,
            _line_threshold: usize,
        ) -> Pin<Box<dyn Future<Output = Result<elai_core::tool_dispatcher::FileView, elai_core::tool_dispatcher::ToolDispatchError>> + Send + '_>> {
            Box::pin(async { Err(elai_core::tool_dispatcher::ToolDispatchError::NotImplemented("noop".into())) })
        }
        fn dispatch_incremental_compile(
            &self,
            _req: elai_core::tool_dispatcher::IncrementalRequest,
        ) -> Pin<Box<dyn Future<Output = Result<elai_core::tool_dispatcher::IncrementalOutcome, elai_core::tool_dispatcher::ToolDispatchError>> + Send + '_>> {
            Box::pin(async { Err(elai_core::tool_dispatcher::ToolDispatchError::NotImplemented("noop".into())) })
        }
    }

    #[test]
    fn with_seed_populates_field() {
        let client = AnthropicClient::new("test-key");
        let dispatcher: Arc<dyn ToolDispatcher> = Arc::new(NoOpDispatcher);
        let server = ProductionAgentRpcServer::new(client, dispatcher, "test-model".into(), 100)
            .with_seed(42);
        assert_eq!(server.seed, Some(42));
    }

    #[test]
    fn new_defaults_seed_none() {
        let client = AnthropicClient::new("test-key");
        let dispatcher: Arc<dyn ToolDispatcher> = Arc::new(NoOpDispatcher);
        let server = ProductionAgentRpcServer::new(client, dispatcher, "test-model".into(), 100);
        assert_eq!(server.seed, None);
    }
}
