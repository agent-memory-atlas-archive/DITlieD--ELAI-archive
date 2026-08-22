// Shared IPC wire protocol types for agent <-> orchestrator communication.
//
// This crate exists to break the circular dependency between elai_cc_agent
// (which must NOT depend on elai-orchestrator) and elai-orchestrator (which
// defines the message types). Both crates depend on this shared crate.
//
// Wire format: [1-byte version][4-byte LE length][postcard payload]

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use uuid::Uuid;

// E-6: TypedAgentEnvelope — typed inter-agent payload wrapper.
// Lives in ipc-types so both elai_cc_agent and elai-orchestrator can
// construct + verify envelopes without taking a dep on the orchestrator
// crate. See workflow/reasoning/E-6-decisions.md.
pub mod envelope;
pub use envelope::{
    EnvelopeError, EnvelopePayload, MastTag, TypedAgentEnvelope, ENVELOPE_MAX_BYTES,
    ENVELOPE_WIRE_VERSION,
};

// ---------------------------------------------------------------------------
// AgentRole — the 10 agent roles
// ---------------------------------------------------------------------------
//
// B-40 (2026-05-16): added IdeaFramer + IdeaCritic for the pre-decomposition
// FRAMING substate. These roles cross the IPC wire as part of
// AgentDispatchConfig::role but the wire FRAME structure (IpcMessageType tag
// bijection, IPC_MESSAGE_TYPE_VARIANT_COUNT) is unchanged — adding string
// variants to a #[serde(rename_all="snake_case")] enum is a soft-compat
// change. At that time, IPC_PROTOCOL_VERSION stayed at 2 because:
//   (1) the IpcMessageType variant table was unchanged (still 16 tags)
//   (2) MAX_FRAME_SIZE and FRAME_HEADER_SIZE are unchanged
//   (3) orchestrator + agent binaries ship together — older agents won't be
//       handed an `idea_framer` dispatch config unless paired with this code
// Documented in workflow/reasoning/B-40-decisions.md DECISION-IPC.
//
// B-41 (2026-05-16): added 5 per-level Auditor* variants
// (AuditorConcepts/AuditorPhases/AuditorSteps/AuditorBabySteps/
// AuditorBabyStepDetail). The pre-existing `Auditor` variant is RETAINED as
// the Plan-mode (single-cycle pass, back-compat). 5 unit variants chosen over
// parametric `Auditor(AuditorMode)` to preserve `#[derive(Copy)]` on
// AgentRole — payload-carrying variants would require AuditorMode to be Copy
// too, plus widen every existing pattern arm. Mapping from
// orchestrator::auditor_mode::AuditorMode to AgentRole is in
// AuditorMode::map_to_agent_role. Documented in
// workflow/reasoning/B-41-decisions.md DECISION-AGENTROLE.
//
// IPC_PROTOCOL_VERSION stayed at 2 at that time: same soft-compat rationale as B-40 (snake_case
// string variants, no IpcMessageType tag change).

// B-37C: added PartialOrd + Ord so AgentRole can key a deterministic
// BTreeMap (PerRoleRuntime). Soft-compat: ordering is by declaration
// position and does not affect wire format (Serialize/Deserialize go
// via the `rename_all = "snake_case"` string tags, not the discriminant).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    Executor,
    /// Plan-mode auditor — single-cycle integration pass at end of plan.
    /// Pre-B-41 behaviour is preserved on this variant; B-41 adds 5
    /// per-level siblings (AuditorConcepts/Phases/Steps/BabySteps/BabyStepDetail).
    Auditor,
    Researcher,
    Designer,
    PromptEngineer,
    ContextCompiler,
    MemoryWriter,
    MediaValidator,
    /// B-40: pre-decomposition idea-framer agent. Designer-shape GRC
    /// (Observe->PlanAction->Act->Verify, no Hypothesize) with restricted
    /// budget 5/800. Runs BEFORE Concept decomposition; produces
    /// FramedIdea + TradeoffCard from research-preset streams.
    IdeaFramer,
    /// B-40: pre-decomposition idea-critic agent. Auditor-shape GRC
    /// (Observe->Verify->Verify) with restricted budget 5/800. Runs per
    /// critic round (max 3); emits CriticFinding rows that BLOCK
    /// decomposition if unresolved at round 3.
    IdeaCritic,
    /// B-41: tier-1 auditor — reviews Concept decomposition (market fit, IP
    /// boundaries, contradictions vs FramedIdea, grounding against cosmos).
    /// GRC budget 6/1000, Observe->Verify->Verify, grounding=High.
    AuditorConcepts,
    /// B-41: tier-2 auditor — reviews Phase decomposition (ordering, contract
    /// sufficiency, dependency cycles). GRC budget 5/800.
    AuditorPhases,
    /// B-41: tier-3 auditor — reviews Step decomposition (granularity, test
    /// coverage of step boundaries). GRC budget 4/700.
    AuditorSteps,
    /// B-41: tier-4 auditor — reviews BabyStep batch (BabyStepSpec 8-block
    /// completeness). GRC budget 4/700.
    AuditorBabySteps,
    /// B-41: tier-4.5 auditor — line-level pass over BabyStep batch outputs
    /// (lazy-code, unwrap usage, hardcoded secrets, stale APIs). GRC budget
    /// 3/600. Cascades to BabyStep tier on second BLOCK (no separate
    /// DecompositionLevel::BabyStepDetail in this slice — see
    /// B-41-decisions.md DECISION-CASCADE).
    AuditorBabyStepDetail,
    /// E-12 (E-11 prereq fulfilled inside E-12): plan-scope auditor that
    /// runs the 15-item production-grade checklist (gitleaks / trufflehog /
    /// semgrep OWASP / RLS probe / dep audit / lighthouse / biome / ruff /
    /// clippy / etc) against the emitted FilePlan + FrameworkProfile. GRC
    /// budget 6/1000 (parity with Plan-mode auditor). The matching
    /// `AuditorMode::ProductionGrade` variant is the dispatch entry; this
    /// AgentRole is the spawned-agent label that the orchestrator hands to
    /// `agent_pool::spawn` for the dedicated production-gate cycle.
    AuditorProductionGrade,
    /// E-37: off-model prompt-injection classifier (OWASP LLM01 mitigation).
    /// PromptArmor-style separate-runtime classifier (paper:
    /// arXiv:2507.15219) that inspects post-Spotlight envelope content
    /// before the orchestrator forwards untrusted text to the main agent
    /// loop. Verdicts (Clean / Suspect / Malicious) drive
    /// SsbActionType::PromptArmorVerdict + InjectionAttemptBlocked /
    /// InjectionAttemptDetected emissions. Default backend mapping:
    /// local_qwen_3b primary + claude_haiku_4_5_cloud fallback. The
    /// matching `elai_safety::InjectionClassifier` trait + `HeuristicClassifier`
    /// pure-regex fallback live in the safety crate so wire-dark deployments
    /// retain protection even when the sidecar is offline.
    InjectionClassifier,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Wire protocol version. Mismatch between orchestrator and agent -> kill.
///
/// Bumped to 3 because the breaking removal of a wire message variant requires
/// older binaries to be rebuilt.
pub const IPC_PROTOCOL_VERSION: u8 = 3;

/// Hard cap on any single IPC frame payload (10 MiB).
pub const MAX_FRAME_SIZE: usize = 10 * 1024 * 1024;

/// Per-type size limits (enforced at frame read time).
pub const MAX_AGENT_HEARTBEAT_SIZE: usize = 1024;
pub const MAX_THINK_STEP_SIZE: usize = 64 * 1024;
pub const MAX_TOOL_RESULT_SIZE: usize = MAX_FRAME_SIZE;

/// Agent heartbeat interval in seconds.
pub const AGENT_HEARTBEAT_INTERVAL_SECS: u64 = 5;

/// Timeout after which a missing heartbeat is treated as agent death.
pub const AGENT_HEARTBEAT_TIMEOUT_SECS: u64 = 15;

/// Length-prefix field size (4 bytes, little-endian u32).
pub const LENGTH_PREFIX_SIZE: usize = 4;

/// Total frame header size: 1 (version) + 4 (length).
pub const FRAME_HEADER_SIZE: usize = 1 + LENGTH_PREFIX_SIZE;

// ---------------------------------------------------------------------------
// ToolId — opaque tool identifier
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToolId(pub String);

impl ToolId {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ---------------------------------------------------------------------------
// SeccompProfile
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeccompProfile {
    IpcBaseline,
    NetworkExtra,
}

// ---------------------------------------------------------------------------
// ResourceLimits
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_bytes: u64,
    pub max_cpu_secs: u64,
    pub max_open_files: u32,
}

// ---------------------------------------------------------------------------
// ProcessConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessConfig {
    pub allowed_paths: Vec<PathBuf>,
    pub seccomp_profile: SeccompProfile,
    pub resource_limits: ResourceLimits,
}

// ---------------------------------------------------------------------------
// ToolFilter
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolFilter {
    pub allowed_tools: Vec<ToolId>,
}

impl ToolFilter {
    pub fn allows(&self, tool_id: &ToolId) -> bool {
        self.allowed_tools.contains(tool_id)
    }
}

// ---------------------------------------------------------------------------
// AgentDispatchConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentDispatchConfig {
    pub role: AgentRole,
    pub model_preference: String,
    pub tool_filter: ToolFilter,
    pub rule_domains: Vec<String>,
    pub grc_budget_max_calls: u32,
    pub grc_budget_max_chars: u32,
    pub sandbox_config: ProcessConfig,
}

// ---------------------------------------------------------------------------
// IpcMessageType — 15 message variants (LOCKED — see note below)
// ---------------------------------------------------------------------------
//
// Subsequent additions still LOCKED behind explicit decision.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IpcMessageType {
    AgentRequest {
        task: String,
        dispatch_config: AgentDispatchConfig,
    },
    AgentResponse {
        content: String,
        artifacts: Vec<String>,
    },
    InferRequest {
        prompt: String,
        max_tokens: u32,
    },
    InferResponse {
        text: String,
        tokens_used: u32,
    },
    ToolRequest {
        tool_id: ToolId,
        arguments: Vec<u8>,
    },
    ToolResult {
        tool_id: ToolId,
        output: Vec<u8>,
        success: bool,
    },
    BashRequest {
        command: String,
        timeout_ms: u64,
    },
    BashResult {
        stdout: Vec<u8>,
        stderr: Vec<u8>,
        exit_code: i32,
    },
    GroundingRequest {
        topic: String,
        query: String,
    },
    GroundingResult {
        topic: String,
        evidence: Vec<String>,
        confidence: f32,
    },
    ThinkStep {
        topic: String,
        step_type: String,
        content: String,
    },
    CheckpointSync {
        agent_id: Uuid,
        sequence: u64,
        payload: Vec<u8>,
    },
    AgentHeartbeat {
        agent_id: Uuid,
        uptime_secs: u64,
    },
    AgentKill {
        reason: String,
    },
    ReplanRequest {
        reason: String,
        severity: String,
    },
}

// Compile-time assertion: exactly 15 variants.
#[doc(hidden)]
pub const fn _ipc_variant_tag(msg: &IpcMessageType) -> u8 {
    match msg {
        IpcMessageType::AgentRequest { .. } => 0,
        IpcMessageType::AgentResponse { .. } => 1,
        IpcMessageType::InferRequest { .. } => 2,
        IpcMessageType::InferResponse { .. } => 3,
        IpcMessageType::ToolRequest { .. } => 4,
        IpcMessageType::ToolResult { .. } => 5,
        IpcMessageType::BashRequest { .. } => 6,
        IpcMessageType::BashResult { .. } => 7,
        IpcMessageType::GroundingRequest { .. } => 8,
        IpcMessageType::GroundingResult { .. } => 9,
        IpcMessageType::ThinkStep { .. } => 10,
        IpcMessageType::CheckpointSync { .. } => 11,
        IpcMessageType::AgentHeartbeat { .. } => 12,
        IpcMessageType::AgentKill { .. } => 13,
        IpcMessageType::ReplanRequest { .. } => 14,
    }
}

/// Number of IpcMessageType variants.
pub const IPC_MESSAGE_TYPE_VARIANT_COUNT: usize = 15;

// ---------------------------------------------------------------------------
// IpcFrame — wire-level frame
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct IpcFrame {
    pub version: u8,
    pub payload: Vec<u8>,
}

impl IpcFrame {
    pub fn encode(msg: &IpcMessageType) -> Result<Self, IpcFrameError> {
        let payload = postcard::to_allocvec(msg)
            .map_err(|e| IpcFrameError::SerializationFailed(e.to_string()))?;
        if payload.len() > MAX_FRAME_SIZE {
            return Err(IpcFrameError::PayloadTooLarge {
                size: payload.len(),
                limit: MAX_FRAME_SIZE,
            });
        }
        Ok(Self {
            version: IPC_PROTOCOL_VERSION,
            payload,
        })
    }

    pub fn decode(payload: &[u8]) -> Result<IpcMessageType, IpcFrameError> {
        postcard::from_bytes(payload)
            .map_err(|e| IpcFrameError::DeserializationFailed(e.to_string()))
    }

    pub fn from_wire_bytes(data: &[u8]) -> Result<Self, IpcFrameError> {
        if data.len() < FRAME_HEADER_SIZE {
            return Err(IpcFrameError::FrameTooShort);
        }
        let version = data[0];
        if version != IPC_PROTOCOL_VERSION {
            return Err(IpcFrameError::VersionMismatch {
                expected: IPC_PROTOCOL_VERSION,
                actual: version,
            });
        }
        let len = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as usize;
        if len > MAX_FRAME_SIZE {
            return Err(IpcFrameError::PayloadTooLarge {
                size: len,
                limit: MAX_FRAME_SIZE,
            });
        }
        let total = FRAME_HEADER_SIZE + len;
        if data.len() < total {
            return Err(IpcFrameError::FrameTooShort);
        }
        Ok(Self {
            version,
            payload: data[FRAME_HEADER_SIZE..total].to_vec(),
        })
    }

    pub fn to_wire_bytes(&self) -> Result<Vec<u8>, IpcFrameError> {
        let len: u32 =
            self.payload
                .len()
                .try_into()
                .map_err(|_| IpcFrameError::PayloadTooLarge {
                    size: self.payload.len(),
                    limit: u32::MAX as usize,
                })?;
        let mut buf = Vec::with_capacity(FRAME_HEADER_SIZE + self.payload.len());
        buf.push(self.version);
        buf.extend_from_slice(&len.to_le_bytes());
        buf.extend_from_slice(&self.payload);
        Ok(buf)
    }
}

// ---------------------------------------------------------------------------
// IpcFrameError
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IpcFrameError {
    #[error("serialization failed: {0}")]
    SerializationFailed(String),
    #[error("deserialization failed: {0}")]
    DeserializationFailed(String),
    #[error("payload too large: {size} bytes exceeds limit of {limit} bytes")]
    PayloadTooLarge { size: usize, limit: usize },
    #[error("protocol version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u8, actual: u8 },
    #[error("frame too short: need at least {FRAME_HEADER_SIZE} bytes")]
    FrameTooShort,
}

// ---------------------------------------------------------------------------
// FsmState — mirrors orchestrator FSM states for tool filtering
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FsmState {
    Idle,
    Planning,
    TestingFirst,
    Executing,
    Auditing,
    Memorizing,
    Complete,
    Configuring,
}

// ---------------------------------------------------------------------------
// Typed tool argument structs (postcard-serialized in ToolRequest.arguments)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitCommitRequest {
    pub message: String,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitUndoRequest {
    pub commit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitBranchRequest {
    pub branch_name: String,
    pub from_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidateRequest {
    pub file_path: String,
    pub language: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceRequest {
    pub target_function: String,
    pub file_path: String,
    pub depth: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditRequest {
    pub file_path: String,
    pub old_text: String,
    pub new_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimplifyRequest {
    pub file_path: String,
    pub target_function: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassifyRequest {
    pub error_output: String,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileReadRequest {
    pub file_path: String,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompileCheckRequest {
    pub file_path: String,
    pub language: String,
    pub retry_round: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DkgCallersRequest {
    pub symbol: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DkgDependentsRequest {
    pub symbol: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DkgDataFlowRequest {
    pub variable: String,
    pub direction: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DkgSubgraphRequest {
    pub entry_point: String,
    pub depth: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationCheckRequest {
    pub target_symbol: String,
    pub project_root: String,
}

// ---------------------------------------------------------------------------
// Tool result payload (generic success/error envelope)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolResultPayload {
    pub output: String,
    pub diagnostics: Vec<String>,
}

// ---------------------------------------------------------------------------
// Well-known tool IDs (constants to avoid string typos)
// ---------------------------------------------------------------------------

pub const TOOL_GIT_COMMIT: &str = "git_commit";
pub const TOOL_GIT_UNDO: &str = "git_undo";
pub const TOOL_GIT_BRANCH: &str = "git_branch";
pub const TOOL_VALIDATE: &str = "validate";
pub const TOOL_TRACE: &str = "trace";
pub const TOOL_EDIT: &str = "edit";
pub const TOOL_SIMPLIFY: &str = "simplify";
pub const TOOL_CLASSIFY: &str = "classify";
pub const TOOL_FILE_READ: &str = "file_read";
pub const TOOL_COMPILE_CHECK: &str = "compile_check";
pub const TOOL_DKG_CALLERS: &str = "dkg_callers";
pub const TOOL_DKG_DEPENDENTS: &str = "dkg_dependents";
pub const TOOL_DKG_DATA_FLOW: &str = "dkg_data_flow";
pub const TOOL_DKG_SUBGRAPH: &str = "dkg_subgraph";
pub const TOOL_INTEGRATION_CHECK: &str = "integration_check";

// Web research tool IDs (4D.17 — for future Tier C agent IPC proxy).
pub const TOOL_SEARCH_REQUEST: &str = "search_request";
pub const TOOL_FETCH_REQUEST: &str = "fetch_request";
pub const TOOL_EMBED_REQUEST: &str = "embed_request";

// B-29 Phase 5 — Symbol-graph tool IDs.
pub const TOOL_SYMBOL_IMPACT: &str = "symbol_impact";
pub const TOOL_SYMBOL_FLOW: &str = "symbol_flow";
pub const TOOL_SYMBOL_CONTEXT: &str = "symbol_context";
pub const TOOL_SYMBOL_LIST: &str = "symbol_list";

// E-40 Tranche B-2 — SymbolGraphDSL + Dataflow tool IDs (orchestrator-rooted
// JSON cache surface, distinct from B-29 SQLite-rooted tools above).
// `symbol_dsl_query` runs a Datalog-style query over the JSON-rooted symbol
// graph; `symbol_dataflow_impact` walks tree-sitter heuristic def/use edges
// to compute hop-bounded impact for an agent's pre-edit claim. Wire-dark
// 5th-list RRF integration in elai_knowledge::pipeline is gated by
// `ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE`. See
// `workflow/reasoning/E-40-tranche-B2-decisions.md`.
pub const TOOL_SYMBOL_DSL_QUERY: &str = "symbol_dsl_query";
pub const TOOL_SYMBOL_DATAFLOW_IMPACT: &str = "symbol_dataflow_impact";

// C-24 (DRIFT-5) — Mermaid graph-export tool ID. Surfaces the same code
// path the dashboard Tauri `export_graph_mermaid` command uses (and the
// shared `export_graph_mermaid_impl` C-18 extracted) over the IPC wire
// so REPL / agent callers can request a Mermaid string without going
// through the Tauri layer.
pub const TOOL_EXPORT_GRAPH_MERMAID: &str = "export_graph_mermaid";

// B-34 — Entry-point detection tool ID.
pub const TOOL_ENTRY_POINTS: &str = "entry_points";

// B-35 Phase B — Context artifact tool IDs.
pub const TOOL_LIST_ARTIFACTS: &str = "list_artifacts";
pub const TOOL_INDEX_ARTIFACTS: &str = "index_artifacts";
pub const TOOL_SEARCH_ARTIFACTS: &str = "search_artifacts";
pub const TOOL_REMOVE_ARTIFACTS: &str = "remove_artifacts";

// ---------------------------------------------------------------------------
// Web research typed request structs (postcard-serialized in ToolRequest.arguments)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebSearchRequest {
    pub query: String,
    pub max_results: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebFetchRequest {
    pub url: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbedRequest {
    pub texts: Vec<String>,
    pub model: String,
}

// ---------------------------------------------------------------------------
// B-29 Phase 5 — Symbol-graph typed request structs
// (postcard-serialized in ToolRequest.arguments)
// ---------------------------------------------------------------------------

/// Request for `symbol_impact` tool — BFS reverse-dependency traversal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolImpactRequest {
    pub project_root: String,
    pub target: String,
    pub depth: u32,
}

/// Request for `symbol_flow` tool — DFS call-flow traversal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolFlowRequest {
    pub project_root: String,
    pub entrypoint_id: String,
    pub depth: u32,
}

/// Request for `symbol_context` tool — 360° view of a symbol definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolContextRequest {
    pub project_root: String,
    pub name: String,
    pub file_hint: Option<String>,
}

/// Request for `symbol_list` tool — list symbols with optional filters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolListRequest {
    pub project_root: String,
    pub file: Option<String>,
    pub query: Option<String>,
    pub limit: u32,
}

// ---------------------------------------------------------------------------
// E-40 Tranche B-2 — SymbolGraphDSL + Dataflow typed request/response structs
// (postcard-serialized in ToolRequest.arguments / ToolResult.payload).
// Orchestrator-rooted JSON cache surface, see DECISION-1 in
// workflow/reasoning/E-40-decisions.md. Field names mirror the JSON schema
// declared in production_agent_rpc::build_tool_catalog so the agent's JSON
// input round-trips through serde_json -> postcard -> serde_json without
// renames.
// ---------------------------------------------------------------------------

/// Request for `symbol_dsl_query` tool — Datalog-style DSL over the
/// orchestrator-rooted JSON symbol graph. Grammar:
/// `query := <selector> ('|' <filter>)*`,
/// `selector := DEF(<symbol>) | USE(<symbol>) | IMPL(<trait>) | CALL(<fn>)`,
/// `filter := lang(<lang>) | file(<pattern>) | depth(<n>)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolDslQueryRequest {
    pub project_root: String,
    pub dsl: String,
    pub depth_cap: u8,
}

/// Single match row returned by `symbol_dsl_query`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolMatch {
    pub symbol: String,
    pub file_path: String,
    pub line: u32,
    pub kind: String,
}

/// Dataflow edge kind discriminator. Mirrors the orchestrator-side
/// `DataflowEdgeKind` enum (snake_case serde).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataflowEdgeKind {
    DefUse,
    UseDef,
    AssignFlow,
    CallChain,
}

/// Single dataflow edge returned by `symbol_dsl_query` (when the DSL
/// selector implies dataflow inspection) or surfaced as side context by
/// `symbol_dataflow_impact`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataflowEdge {
    pub source: String,
    pub target: String,
    pub edge_kind: DataflowEdgeKind,
}

/// Response for `symbol_dsl_query` tool — DSL match results plus any
/// dataflow side-context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolDslQueryResponse {
    pub matches: Vec<SymbolMatch>,
    pub dataflow_edges: Vec<DataflowEdge>,
    pub total_scanned: u32,
}

/// Request for `symbol_dataflow_impact` tool — BFS impact analysis over the
/// JSON symbol graph's reverse-call index, bounded by `depth_cap`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolDataflowImpactRequest {
    pub project_root: String,
    pub symbol: String,
    pub depth_cap: u8,
}

/// Response for `symbol_dataflow_impact` tool — transitively-impacted
/// symbol names plus the achieved depth + edge count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolDataflowImpactResponse {
    pub impacted_symbols: Vec<String>,
    pub depth_actual: u8,
    pub total_edges: u32,
}

/// Request for the `export_graph_mermaid` tool — render the same
/// VizData payload as a Mermaid flowchart string. Mirrors the Tauri
/// command `export_graph_mermaid` (dashboard-v2/src-tauri/src/commands.rs)
/// and the shared `export_graph_mermaid_impl` free function it delegates
/// to (C-18). `mode` ∈ {"file", "symbol", "decomposition"}. `max_nodes`
/// caps the rendered subset; when `None`, the impl uses the B-33
/// default (200).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportGraphMermaidRequest {
    pub project_root: String,
    pub mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_nodes: Option<u32>,
}

// ---------------------------------------------------------------------------
// B-35 Phase B — Context artifact typed request structs
// (postcard-serialized in ToolRequest.arguments)
// ---------------------------------------------------------------------------

/// Request for `list_artifacts` tool — list all indexed artifact entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListArtifactsRequest {
    /// Absolute path to the project root (used to locate `context_*.db`).
    pub db_path: String,
}

/// Request for `index_artifacts` tool — trigger (re)indexing of one artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexArtifactsRequest {
    /// Absolute path to the `context_*.db` store file.
    pub db_path: String,
    /// Name of the artifact to (re)index. Must match a manifest entry.
    pub artifact_name: String,
}

/// Request for `search_artifacts` tool — semantic search over artifact chunks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchArtifactsRequest {
    /// Absolute path to the `context_*.db` store file.
    pub db_path: String,
    /// Natural-language search query.
    pub query: String,
    /// Optional artifact name filter; `None` searches all artifacts.
    pub artifact_name: Option<String>,
    /// Maximum number of results to return. Defaults to `DEFAULT_ARTIFACT_SEARCH_LIMIT`.
    pub limit: Option<usize>,
}

/// Default limit when `SearchArtifactsRequest::limit` is `None`.
pub const DEFAULT_ARTIFACT_SEARCH_LIMIT: usize = 10;

/// Request for `remove_artifacts` tool — delete all chunks for one artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoveArtifactsRequest {
    /// Absolute path to the `context_*.db` store file.
    pub db_path: String,
    /// Name of the artifact whose chunks should be removed.
    pub artifact_name: String,
}

// ---------------------------------------------------------------------------
// B-34 — Entry-point detection typed request/response structs
// (postcard-serialized in ToolRequest.arguments for the request;
//  JSON-serialized in ToolResponse for the response)
// ---------------------------------------------------------------------------

/// Request for `entry_points` tool — detect project entry points.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryPointsRequest {
    /// Absolute or repo-relative path to the project root.
    /// Used to locate the symbol-graph DB (same as other symbol-graph tools).
    pub project_root: String,
    /// When false, filter out `Test` reason entries. Defaults to true so the
    /// researcher agent sees all entry points including test harnesses.
    #[serde(default = "default_include_tests")]
    pub include_tests: bool,
}

fn default_include_tests() -> bool {
    true
}

/// Wire DTO for a single detected entry point (JSON surface).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryPointDto {
    /// Stable id: symbol id (heuristics 2–4) or file path (orphan heuristic 1).
    pub id: String,
    /// Human-readable name (qualified symbol name or file path).
    pub name: String,
    /// Source file path.
    pub file: String,
    /// 1-based start line, if available.
    pub line: Option<u32>,
    /// Reason discriminant string: "orphan", "well_known", "framework", "test".
    pub reason: String,
    /// Additional detail for the reason (e.g. "main", "framework:http-get").
    pub reason_detail: Option<String>,
}

/// Summary counts across all detected entry points.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryPointSummaryDto {
    /// Total detected entry-point count.
    pub total: u32,
    /// Count per reason key (e.g. "orphan" -> 3, "well_known:main" -> 2).
    pub by_reason: BTreeMap<String, u32>,
}

/// Response for `entry_points` tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryPointsResponse {
    pub entries: Vec<EntryPointDto>,
    pub summary: EntryPointSummaryDto,
}

// ---------------------------------------------------------------------------
// E-35 — Structured-output substrate (XGrammar + INSTRUCTOR mode + CodeAct).
// ---------------------------------------------------------------------------
//
// These types are the Rust-side wire contract for the sidecar's
// XGrammarLogitsProcessor (Python side — to be wired in a separate session;
// see crates/providers/src/runtime/sidecar_xgrammar_hook.rs for the
// authoritative interface description and `followups_for_other_sessions`).
//
// All variants are additive and DO NOT change the IpcMessageType discriminant
// table — they ride inside ToolRequest.arguments / ToolResult.output via
// postcard the same way EditRequest does. IPC_PROTOCOL_VERSION is unchanged.

/// Structured-output mode selector. Maps to the CRANE
/// (arxiv:2502.09061) split between unconstrained reasoning and
/// constrained answer phases.
///
/// * `Instructor` — emit `<think>...</think>` unconstrained, then `<answer>`
///   with the XGrammar mask applied. Default for reasoning roles (auditor,
///   planner, decomposer, idea-framer).
/// * `Grammar` — apply XGrammar mask directly, no `<think>` phase. Used for
///   low-reasoning enum/header blocks (FailureTaxonomy enum, BabyStepSpec
///   status enum, ReasoningTemplate type-tag).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructuredMode {
    Instructor,
    Grammar,
}

impl StructuredMode {
    /// Stable snake_case label used in SSB beacons and log lines.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Instructor => "instructor",
            Self::Grammar => "grammar",
        }
    }
}

/// XGrammar tool-call envelope variant. Qwen2.5-Coder family uses
/// `<tools>...</tools>` (hanXen/vllm-qwen2.5-coder-tool-parser);
/// Qwen2.5-7B-Instruct (non-Coder) uses native hermes `<tool_call>`.
/// The variant is resolved at sidecar boot from `model.config.name_or_path`
/// or the explicit `ELAI_TOOL_PARSER_VARIANT` env override.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolParserVariant {
    /// `<tools>...</tools>` envelope. Qwen2.5-Coder-{3B,7B,32B}.
    QwenTools,
    /// Native hermes `<tool_call>...</tool_call>` envelope. Llama-3, Qwen2.5-Instruct
    /// (non-Coder), DeepSeek-V2.
    Hermes,
}

impl ToolParserVariant {
    /// Stable snake_case label for SSB beacons and log lines.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::QwenTools => "qwen_tools",
            Self::Hermes => "hermes",
        }
    }
}

/// Request payload sent from the Rust orchestrator to the sidecar's
/// XGrammarLogitsProcessor. The payload is postcard-serialized into
/// ToolRequest.arguments (tool_id = TOOL_XGRAMMAR_INFER) and consumed
/// by `python/elai_inference/server.py::XGrammarLogitsProcessor` (wired
/// in a separate orchestration session — see
/// `crates/providers/src/runtime/sidecar_xgrammar_hook.rs`).
///
/// The schema is identified by `schema_id` (the canonical key the sidecar
/// pre-compiled at boot, format `"{role}.{block}"` e.g. `"auditor.findings"`)
/// AND by `schema_hash` (`blake3(schema_canonical_json)`) so the sidecar
/// can verify cache validity before applying the mask. If the hash does NOT
/// match the cached entry, the sidecar MUST surface a CacheMiss error rather
/// than silently recompile (per WIRING_CONTRACT::stub_ban).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XGrammarRequest {
    /// Canonical "{role}.{block}" identifier of the pre-compiled grammar.
    pub schema_id: String,
    /// `blake3(schema_canonical_json)` of the schema the orchestrator used to
    /// construct the request. The sidecar verifies this matches the cached
    /// entry before applying the mask. CLAUDE.md anti-stale: blake3,
    /// NEVER blake2/sha2.
    pub schema_hash: String,
    /// `StructuredMode::Instructor` triggers the `<think>...</think>`
    /// pre-phase; `StructuredMode::Grammar` applies the mask immediately.
    pub mode: StructuredMode,
    /// Tool-call envelope variant when this request involves tool calls.
    /// `None` for non-tool-call structured outputs.
    pub tool_parser: Option<ToolParserVariant>,
    /// The model prompt. Already rendered (template merged); the sidecar
    /// does not template.
    pub prompt: String,
    /// Hard upper bound on emitted tokens. Caller (orchestrator) sizes this
    /// against the role's GRC budget + `max_context`.
    pub max_tokens: u32,
}

/// Response payload returned from the sidecar's XGrammarLogitsProcessor.
/// Carries both the constrained `<answer>` body (always non-empty on success)
/// and the optional unconstrained `<think>` body (None when mode == Grammar).
///
/// The sidecar guarantees:
/// 1. `answer_text` parses as valid JSON against the schema identified by
///    `(schema_id, schema_hash)` from the matching `XGrammarRequest`, OR the
///    sidecar returns an error frame instead of an XGrammarResponse.
/// 2. `mask_applied_count` > 0 — if the mask never fired the sidecar emits
///    `LegacyJsonObjectRefused` SSB instead of returning success.
/// 3. `think_text.is_some()` iff `mode == Instructor`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XGrammarResponse {
    /// Echoed from the request for cross-check at the consumer.
    pub schema_id: String,
    pub schema_hash: String,
    /// Constrained body (parses against the schema).
    pub answer_text: String,
    /// Unconstrained reasoning body. Some iff `request.mode == Instructor`.
    pub think_text: Option<String>,
    /// Number of times the XGrammar mask was applied during decoding.
    /// Used by `xgrammar_mask_applied` SSB beacon and contract tests.
    pub mask_applied_count: u32,
    /// Wall-clock generation time in milliseconds. Includes both `<think>`
    /// and `<answer>` phases under Instructor mode.
    pub generation_ms: u32,
    /// `true` iff the schema_hash matched a pre-compiled cache entry at
    /// the sidecar (no on-demand recompile). Per WIRING_CONTRACT::stub_ban
    /// this MUST be true; on-demand compile is a misconfiguration and the
    /// sidecar surfaces an error frame instead of returning success.
    pub cache_hit: bool,
}

/// CodeAct (arxiv:2402.01030 ICML 2024) action payload — composes multiple
/// tools in one Python action. Issued by `AgentRole::Runner` (registered in
/// orchestrator AgentRole table — out-of-scope for the E-35 typed
/// substrate; see `followups_for_other_sessions`).
///
/// Sandbox enforces:
/// * `python_code.lines().count() <= 200` (anti-runaway cap)
/// * every tool invocation in the script resolves to a `ToolId` ∈
///   `tool_allowlist` (otherwise the sandbox rejects pre-execution)
/// * fixed wall-clock timeout enforced by the dispatching crate (out-of-scope
///   for the typed surface here)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeActAction {
    /// Python source emitted by the model (already extracted from the
    /// fenced code block).
    pub python_code: String,
    /// Tools the agent is permitted to invoke from `python_code`. Empty
    /// allowlist == no tool calls allowed (script may still compute over
    /// passed-in arguments).
    pub tool_allowlist: Vec<ToolId>,
    /// `blake3(python_code)`, recorded in the SSB row for audit. CLAUDE.md
    /// anti-stale: blake3, NEVER blake2/sha2.
    pub action_hash: String,
}

/// Hard line cap for a single `CodeActAction.python_code`. The sandbox
/// rejects scripts exceeding this limit pre-execution.
pub const CODEACT_MAX_LINES: usize = 200;

/// Well-known tool ID for the sidecar-bound XGrammar inference call.
pub const TOOL_XGRAMMAR_INFER: &str = "xgrammar_infer";

/// Well-known tool ID for the CodeAct Python execution channel.
pub const TOOL_CODEACT_EXEC: &str = "codeact_exec";

// ---------------------------------------------------------------------------
// H-13 canvas op tools (model drives the canvas)
// ---------------------------------------------------------------------------

/// Place (create) a new node on the canvas at the given coordinates.
pub const TOOL_CANVAS_CREATE_NODE: &str = "canvas_create_node";

/// Connect two existing nodes with a directed edge.
pub const TOOL_CANVAS_CONNECT: &str = "canvas_connect";

/// Update the label, metadata, or position of an existing canvas node.
pub const TOOL_CANVAS_UPDATE_NODE: &str = "canvas_update_node";

/// Add a text annotation anchored to a node or a free canvas position.
pub const TOOL_CANVAS_ANNOTATE: &str = "canvas_annotate";

/// Fork an existing node into a new sibling node sharing the same parent edges.
pub const TOOL_CANVAS_FORK: &str = "canvas_fork";

/// Render the current canvas graph as a Mermaid/architecture diagram
/// (ARCHITECTURE mode); emits CanvasOpKind::LayoutChanged rows.
pub const TOOL_CANVAS_RENDER_DIAGRAM: &str = "canvas_render_diagram";

/// Capture the canvas as a PNG and route to the shipped media_validator Tier2.
pub const TOOL_CANVAS_SCREENSHOT: &str = "canvas_screenshot";

// ---------------------------------------------------------------------------
// H-15 canvas spawn_view tool
// ---------------------------------------------------------------------------

/// Spawn a new canvas view with the given view_type and seed ops.
/// view_type selects a registered renderer (e.g. "comparison_board").
/// seed_ops are canvas_op_applied op_ids that bootstrap the view's initial graph.
/// Emits a CanvasViewEvent(ViewSpawned) SSB row. Gated behind
/// ELAI_CANVAS_SPAWN_ENABLE (default: text-unless-confidence-clears per GD-4).
pub const TOOL_CANVAS_SPAWN_VIEW: &str = "canvas_spawn_view";

pub const TOOL_CANVAS_BIND_NODE: &str = "canvas_bind_node";
pub const TOOL_CANVAS_DECLARE_EDGE: &str = "canvas_declare_edge";
pub const TOOL_CANVAS_SET_DISPLAY: &str = "canvas_set_display";

// ---------------------------------------------------------------------------
// I-8: acknowledge_infeasible tool (aletheia voluntary abort)
// ---------------------------------------------------------------------------

/// Well-known tool ID for the voluntary abort / infeasibility acknowledgement verb.
///
/// When a model calls this tool it explicitly declares the current trajectory
/// infeasible.  The dispatcher emits a `TrajectoryAborted(Voluntary)` SSB row
/// and returns `Err(reason)` so the task runner surfaces the abort as a
/// `ToolOutcome::Failed` message.  The constant must be defined HERE (not as a
/// literal in the match arm) so the 3-arm IS-14 pattern never silently routes to
/// the unknown fallback.
pub const TOOL_ACKNOWLEDGE_INFEASIBLE: &str = "acknowledge_infeasible";

/// Wire struct for `acknowledge_infeasible` tool calls.
///
/// Postcard-serialised in `ToolRequest.arguments` and consumed by
/// `convert_json_input_to_postcard` / `translate_tool_request`.
/// Field names MUST match the JSON `input_schema` in `build_tool_catalog`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcknowledgeInfeasibleRequest {
    /// Human-readable reason the model considers this trajectory infeasible.
    /// Forwarded as the `Err(reason)` text that surfaces to the task runner.
    pub reason: String,
    /// Optional trajectory id for correlation with earlier SSB rows.
    #[serde(default)]
    pub trajectory_id: Option<String>,
}

/// I-13: wire constant for the `trace_query` tool (IS-14 3-arm pattern).
/// Defined here so the convert/dispatch match arms never route to the unknown fallback.
pub const TOOL_TRACE_QUERY: &str = "trace_query";

/// I-13: wire struct for `trace_query` tool calls.
///
/// Postcard-serialised in `ToolRequest.arguments` by `convert_json_input_to_postcard`
/// and consumed by `translate_tool_request`.
/// Field names MUST match the JSON `input_schema` in `build_tool_catalog`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceQueryRequest {
    /// Absolute or repo-relative path to the file to expand.
    pub file_path: String,
    /// First line to return (1-indexed, inclusive). Must be >= 1.
    pub start_line: u32,
    /// Last line to return (1-indexed, inclusive). Must be >= start_line.
    pub end_line: u32,
}

/// I-52: wire constants for the agent-callable checkpoint tool (3-arm
/// save/rewind/diff over the H-49 time-travel CAS substrate). Defined here so
/// the JSON-path dispatch match arms never route to the unknown fallback.
/// These tools use the JSON dispatch path (no postcard wire struct), mirroring
/// the I-53 recipe_* / archive_read / json_tree_query tools.
pub const TOOL_CHECKPOINT_SAVE: &str = "checkpoint_save";
/// I-52: rewind to a saved checkpoint label (refuses on RV-1 divergence).
pub const TOOL_CHECKPOINT_REWIND: &str = "checkpoint_rewind";
/// I-52: diff the working tree against a saved checkpoint label.
pub const TOOL_CHECKPOINT_DIFF: &str = "checkpoint_diff";

/// I-54: wire constants for the agent-callable structural (AST-aware) edit tool
/// (3-arm search/preview/apply over the shipped tree-sitter grammars; one version
/// source via elai_knowledge::TreeSitterAstEvaluator; ast-grep dep BANNED, SLC-3).
/// Defined here so the JSON-path dispatch match arms never route to the unknown
/// fallback. These tools use the JSON dispatch path (no postcard wire struct),
/// mirroring the I-52 checkpoint_* / I-53 recipe_* tools.
pub const TOOL_STRUCTURAL_SEARCH: &str = "structural_search";
/// I-54: preview the would-be diff of a query + rewrite template (ZERO bytes).
pub const TOOL_STRUCTURAL_PREVIEW: &str = "structural_preview";
/// I-54: apply a previewed structural edit (rides the SHIPPED edit gates;
/// refuses on a stale match -- the file changed since preview).
pub const TOOL_STRUCTURAL_APPLY: &str = "structural_apply";

/// Request for `canvas_spawn_view` — spawn a new canvas view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasSpawnViewRequest {
    /// Renderer key selecting the view component (e.g. "comparison_board").
    /// Must be a registered view_type in the renderer registry.
    pub view_type: String,
    /// Op ids (canvas_op_applied) to seed the view's initial graph state.
    /// May be empty (spawns a blank view).
    pub seed_ops: Vec<String>,
    /// Canvas mode at spawn time.
    pub mode: String,
    /// Task id for SSB tracing.
    pub task_id: String,
}

/// Response from `canvas_spawn_view`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasSpawnViewResponse {
    /// Stable view instance identifier (used for per-tab SSB polling).
    pub view_id: String,
    /// Renderer view_type echo.
    pub view_type: String,
    /// Number of seed ops applied.
    pub seed_op_count: usize,
    /// Echo of the triggering task_id.
    pub task_id: String,
}

// ---------------------------------------------------------------------------
// H-13 canvas op request/response wire structs.
// Field names MUST match the JSON input_schema in production_agent_rpc.rs
// build_tool_catalog() for the 3-arm pattern to round-trip correctly.
// All structs postcard-serializable for the convert->dispatch arms.
// ---------------------------------------------------------------------------

/// Request for `canvas_bind_node`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasBindNodeRequest {
    pub node_id: String,
    pub binding_kind: String,
    pub binding_ref: String,
    pub display: String,
    pub col: i64,
    pub row: i64,
    pub mode: String,
}

/// Request for `canvas_declare_edge`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasDeclareEdgeRequest {
    pub producer_node: String,
    pub consumer_node: String,
    pub produces: String,
    pub consumes: String,
    pub reachable_from: String,
    pub observable_effect: String,
    pub mode: String,
}

/// Request for `canvas_set_display`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasSetDisplayRequest {
    pub node_id: String,
    pub binding_kind: String,
    pub binding_ref: String,
    pub display: String,
    pub col: i64,
    pub row: i64,
    pub mode: String,
}

/// Request for `canvas_create_node` — create a new node on the active canvas.
/// NOTE: Optional fields use `#[serde(default)]` only (no skip_serializing_if)
/// to ensure postcard round-trip compatibility (postcard requires fixed-width
/// Option discriminants; skip_serializing_if would omit None bytes).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasCreateNodeRequest {
    /// Unique node id chosen by the model (must not already exist in the graph).
    pub node_id: String,
    /// Human-readable label shown on the canvas node.
    pub label: String,
    /// Canvas mode namespace; must match the active mode (e.g. "arch", "code").
    pub mode: String,
    /// Optional x-coordinate hint (0.0-1.0 normalised canvas space).
    #[serde(default)]
    pub x: Option<f64>,
    /// Optional y-coordinate hint (0.0-1.0 normalised canvas space).
    #[serde(default)]
    pub y: Option<f64>,
    /// Arbitrary metadata (node type, colour, etc). JSON-encoded object string or null.
    #[serde(default)]
    pub meta: Option<String>,
    /// Task id that triggered this op (for SSB tracing).
    pub task_id: String,
}

/// Request for `canvas_connect` — add a directed edge between two existing nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasConnectRequest {
    /// Source node id (must exist in the graph).
    pub from_id: String,
    /// Target node id (must exist in the graph).
    pub to_id: String,
    /// Optional edge label.
    #[serde(default)]
    pub label: Option<String>,
    /// Canvas mode namespace.
    pub mode: String,
    /// Task id for SSB tracing.
    pub task_id: String,
}

/// Request for `canvas_update_node` — update the label or metadata of an existing node.
/// NOTE: Optional fields use `#[serde(default)]` only for postcard compatibility.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasUpdateNodeRequest {
    /// Id of the node to update (must exist in the graph).
    pub node_id: String,
    /// New label; if omitted the label is unchanged.
    #[serde(default)]
    pub label: Option<String>,
    /// Metadata patch (JSON-encoded string); merged into existing meta.
    #[serde(default)]
    pub meta: Option<String>,
    /// Canvas mode namespace.
    pub mode: String,
    /// Task id for SSB tracing.
    pub task_id: String,
}

/// Request for `canvas_annotate` — add a text annotation to the canvas.
/// NOTE: Optional fields use `#[serde(default)]` only for postcard compatibility.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasAnnotateRequest {
    /// Annotation text.
    pub text: String,
    /// Node id to anchor the annotation to; null for free-floating.
    #[serde(default)]
    pub anchor_node_id: Option<String>,
    /// Canvas mode namespace.
    pub mode: String,
    /// Task id for SSB tracing.
    pub task_id: String,
}

/// Request for `canvas_fork` — fork an existing node into a sibling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasForkRequest {
    /// Id of the node to fork (must exist in the graph).
    pub source_node_id: String,
    /// Id for the new forked node (must not already exist).
    pub new_node_id: String,
    /// Label for the new forked node.
    pub label: String,
    /// Canvas mode namespace.
    pub mode: String,
    /// Task id for SSB tracing.
    pub task_id: String,
}

/// Request for `canvas_render_diagram` — emit structured nodes/edges for an
/// architecture diagram via the ARCHITECTURE mode canvas.
/// Reuses the server-side `export_graph_mermaid` path for layout data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasRenderDiagramRequest {
    /// Graph render mode passed to `export_graph_mermaid` (e.g. "symbol", "file").
    pub graph_mode: String,
    /// Maximum nodes to render; server-side default applied when None.
    #[serde(default)]
    pub max_nodes: Option<u32>,
    /// Canvas mode (must be "arch").
    pub mode: String,
    /// Task id for SSB tracing.
    pub task_id: String,
}

/// Request for `canvas_screenshot` — capture the canvas WebGL surface as PNG
/// and route to the shipped media_validator Tier2 for VLM scoring.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasScreenshotRequest {
    /// Canvas mode namespace at capture time.
    pub mode: String,
    /// Task id for SSB tracing and media_validator correlation.
    pub task_id: String,
}

/// Uniform response returned by all canvas op tool arms.
/// The `op_id` is a stable identifier for the emitted SSB row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasOpResponse {
    /// Unique id for this canvas op SSB row.
    pub op_id: String,
    /// Snake-case wire name of the op kind (e.g. "node_created").
    pub kind: String,
    /// Canvas mode at the time the op was applied.
    pub mode: String,
    /// Echo of the triggering task_id.
    pub task_id: String,
}

// ---------------------------------------------------------------------------
// E-35 / SearchReplace tier-1 typed action shapes.
// The full matcher + DelimiterCollisionGuard implementation lives in
// crates/tools/src/search_replace.rs (this crate stays IPC-only).
// ---------------------------------------------------------------------------

/// SEARCH/REPLACE tier-1 edit request. Uses custom non-git fence tokens
/// (`<<< ELAI_SEARCH` / `=== ELAI_REPLACE` / `>>> ELAI_END`) deliberately
/// chosen to NOT collide with git conflict markers, Rust/Python/Svelte/TS
/// comment syntax. Source: arXiv:2510.12487 Diff-XYZ + aider polyglot
/// leaderboard (Nov 2025).
///
/// The matching itself is implemented in `elai_tools::search_replace`; this
/// type is the wire shape so the orchestrator can hand a structured patch
/// to the tools crate without re-parsing fenced text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchReplaceTier1 {
    /// File the patch targets. Resolved against the project root by the
    /// dispatching crate.
    pub file_path: String,
    /// One or more (search, replace) blocks. Order matters — applied
    /// top-to-bottom. Each block must match exactly once or the dispatcher
    /// falls back to whole-file rewrite (see DelimiterCollisionGuard).
    pub edits: Vec<SearchReplaceEdit>,
}

/// One SEARCH/REPLACE pair inside a `SearchReplaceTier1`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchReplaceEdit {
    pub search: String,
    pub replace: String,
}

// ---------------------------------------------------------------------------
// Tests — E-35 typed surface round-trips
// ---------------------------------------------------------------------------

#[cfg(test)]
mod e35_tests {
    use super::*;

    #[test]
    fn structured_mode_round_trip() {
        let m = StructuredMode::Instructor;
        let bytes = postcard::to_allocvec(&m).expect("encode");
        let back: StructuredMode = postcard::from_bytes(&bytes).expect("decode");
        assert_eq!(m, back);
        assert_eq!(StructuredMode::Instructor.as_str(), "instructor");
        assert_eq!(StructuredMode::Grammar.as_str(), "grammar");
    }

    #[test]
    fn tool_parser_variant_labels_stable() {
        assert_eq!(ToolParserVariant::QwenTools.as_str(), "qwen_tools");
        assert_eq!(ToolParserVariant::Hermes.as_str(), "hermes");
    }

    #[test]
    fn xgrammar_request_round_trips_postcard() {
        let req = XGrammarRequest {
            schema_id: "auditor.findings".to_string(),
            schema_hash: "blake3:deadbeef".to_string(),
            mode: StructuredMode::Instructor,
            tool_parser: Some(ToolParserVariant::QwenTools),
            prompt: "audit phase B-35".to_string(),
            max_tokens: 1024,
        };
        let bytes = postcard::to_allocvec(&req).expect("encode");
        let back: XGrammarRequest = postcard::from_bytes(&bytes).expect("decode");
        assert_eq!(req, back);
    }

    #[test]
    fn xgrammar_response_round_trips_postcard() {
        let resp = XGrammarResponse {
            schema_id: "auditor.findings".to_string(),
            schema_hash: "blake3:deadbeef".to_string(),
            answer_text: "{\"findings\":[]}".to_string(),
            think_text: Some("reasoning trail".to_string()),
            mask_applied_count: 17,
            generation_ms: 412,
            cache_hit: true,
        };
        let bytes = postcard::to_allocvec(&resp).expect("encode");
        let back: XGrammarResponse = postcard::from_bytes(&bytes).expect("decode");
        assert_eq!(resp, back);
    }

    #[test]
    fn codeact_action_respects_max_lines_constant() {
        // The cap itself is enforced in the sandbox crate; here we just lock
        // the constant against accidental change.
        assert_eq!(CODEACT_MAX_LINES, 200);
        let act = CodeActAction {
            python_code: "for i in range(3): print(i)".to_string(),
            tool_allowlist: vec![ToolId::new(TOOL_FILE_READ)],
            action_hash: "blake3:cafebabe".to_string(),
        };
        let bytes = postcard::to_allocvec(&act).expect("encode");
        let back: CodeActAction = postcard::from_bytes(&bytes).expect("decode");
        assert_eq!(act, back);
    }

    #[test]
    fn search_replace_tier1_round_trips_postcard() {
        let patch = SearchReplaceTier1 {
            file_path: "src/lib.rs".to_string(),
            edits: vec![
                SearchReplaceEdit {
                    search: "old".to_string(),
                    replace: "new".to_string(),
                },
                SearchReplaceEdit {
                    search: "abc".to_string(),
                    replace: "xyz".to_string(),
                },
            ],
        };
        let bytes = postcard::to_allocvec(&patch).expect("encode");
        let back: SearchReplaceTier1 = postcard::from_bytes(&bytes).expect("decode");
        assert_eq!(patch, back);
    }

    #[test]
    fn well_known_tool_ids_stable() {
        assert_eq!(TOOL_XGRAMMAR_INFER, "xgrammar_infer");
        assert_eq!(TOOL_CODEACT_EXEC, "codeact_exec");
    }

    // H-13 canvas tool id stability.
    #[test]
    fn h13_canvas_tool_ids_stable() {
        assert_eq!(TOOL_CANVAS_CREATE_NODE, "canvas_create_node");
        assert_eq!(TOOL_CANVAS_CONNECT, "canvas_connect");
        assert_eq!(TOOL_CANVAS_UPDATE_NODE, "canvas_update_node");
        assert_eq!(TOOL_CANVAS_ANNOTATE, "canvas_annotate");
        assert_eq!(TOOL_CANVAS_FORK, "canvas_fork");
        assert_eq!(TOOL_CANVAS_RENDER_DIAGRAM, "canvas_render_diagram");
        assert_eq!(TOOL_CANVAS_SCREENSHOT, "canvas_screenshot");
        assert_eq!(TOOL_CANVAS_BIND_NODE, "canvas_bind_node");
        assert_eq!(TOOL_CANVAS_DECLARE_EDGE, "canvas_declare_edge");
        assert_eq!(TOOL_CANVAS_SET_DISPLAY, "canvas_set_display");
    }
}
