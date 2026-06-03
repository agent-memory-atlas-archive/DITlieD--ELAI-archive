// C-39: MCP bridge dispatch implementation.
//
// Routes tool calls through DefaultMcpBridge (plugins crate, feature=mcp).
// This is the --via-mcp path. Uses the same HTTP JSON-RPC 2.0 client that
// the dashboard uses (plugins/src/mcp_bridge.rs).
//
// STUB_BAN compliance (C-39): on network error, surfaces to SSB and fails
// closed -- no silent retry.
//
// Distinct-action-type contract: this impl emits SsbPayload::CliMcpDispatch
// rows; the legacy in-process / dashboard path emits McpToolCalled. The
// CI parity checker unions both action types (see scripts/ci_mcp_parity_check.py).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use elai_core::clock::{ClockProvider, OsClock};

use super::{Dispatch, DispatchRequest, DispatchResult};
use crate::roadmap_signals::roadmap_signals_enabled;
use crate::ssb::{
    AiRoadmapSignalEventPayload, CliMcpDispatchPayload, RoadmapSignalKind, Ssb, SsbPayload,
};

/// E-38 (SCHEMA_VERSION 175): idempotency latch for the
/// `McpTransportVersionLogged` SSB row. The row is emitted exactly once
/// per process boot regardless of how many `McpClientDispatch::new`
/// instances are constructed.
static MCP_TRANSPORT_VERSION_LOGGED: AtomicBool = AtomicBool::new(false);

/// E-38 -- canonical roadmap target string emitted alongside the current
/// transport label. Sourced from
/// modelcontextprotocol.io/development/roadmap (verified May 2026):
/// stateless transport + Tasks primitive + MCP Registry Q4 2026 +
/// SAML/OIDC/PKCE Q2 2026.
pub const MCP_TRANSPORT_ROADMAP_TARGET: &str =
    "stateless+tasks+registry-q4-2026+saml-oidc-pkce-q2-2026";

/// E-38 -- current default MCP transport identifier ELAI ships against.
/// Aligned with `elai_plugins::mcp_bridge::McpTransportMode::default()` =
/// StreamableHttp (MCP spec 2025-11-25). Updates here MUST stay in sync
/// with the bridge's Default impl.
pub const MCP_TRANSPORT_CURRENT_DEFAULT: &str = "streamable_http";

/// MCP bridge dispatch. Routes tool calls via DefaultMcpBridge HTTP client.
/// Emits CliMcpDispatch SSB rows on each call (when an SSB writer is wired).
/// Port is the bridge server port (default 8200 from DEFAULT_MCP_SERVER_URL).
pub struct McpClientDispatch {
    pub port: u16,
    /// Optional SSB writer. When `Some`, every dispatch emits one
    /// CliMcpDispatch row carrying tool_name + port. When `None` (test
    /// constructor), dispatch is observable only via the return value.
    ssb: Option<Arc<Ssb>>,
    /// C-47b CLI-arm: long-lived shared calibration registry. When `Some`
    /// (env-gate `ELAI_C47B_CASE_BASE_PERSIST_ENABLE` on at the
    /// `run_subcommand` wiring seam), [`Self::dispatch_tool`] routes the
    /// per-call bridge through `dispatch_tool_call_with_gate` so the
    /// post-dispatch `record_call` fires and observations persist to the
    /// shared `case_base.sqlite3` (same file the dashboard arm reads).
    /// When `None` (the default, gate off), `dispatch_tool` keeps the
    /// exact `http_call_tool` path -- byte-identical default behaviour.
    /// Cloned into each transient `DefaultMcpBridge` so accumulation lands
    /// in the one shared registry, not in throwaway per-call state.
    #[cfg(feature = "mcp")]
    calibration_registry: Option<Arc<elai_plugins::ToolCalibrationRegistry>>,
    /// F-3p1 / Slice C3 (ENFORCE B): optional `CapabilityFlowAdapter` wired
    /// into the `ToolGate` of each per-call `DefaultMcpBridge`. When `Some`,
    /// every tool dispatch is routed through `dispatch_tool_call_with_gate`
    /// so `ToolGate::check_tool_allowed` consults the adapter's
    /// `CommittedSetChecker::check_committed` before the actual bridge call.
    /// DEFAULT-ON once wired; no env gate on the enforcement path.
    #[cfg(feature = "mcp")]
    capability_flow_adapter: Option<Arc<crate::capability_flow::CapabilityFlowAdapter>>,
}

impl McpClientDispatch {
    /// Construct with the given bridge server port and SSB writer for
    /// observable telemetry. This is the production constructor.
    ///
    /// E-38 (SCHEMA_VERSION 175): the constructor opportunistically
    /// emits one `AiRoadmapSignalEvent::McpTransportVersionLogged` SSB
    /// row PER PROCESS BOOT (gated by the idempotency latch
    /// [`MCP_TRANSPORT_VERSION_LOGGED`] + the
    /// [`crate::ssb::ELAI_AI_ROADMAP_SIGNALS_ENABLE_ENV`] env-gate). The
    /// row records the current MCP transport label vs the roadmap
    /// target so dashboards can render drift over time.
    pub fn new(port: u16, ssb: Arc<Ssb>) -> Self {
        let this = Self {
            port,
            ssb: Some(ssb),
            // C-47b: registry is attached post-construction via
            // `with_calibration_registry` at the wiring seam; default None.
            #[cfg(feature = "mcp")]
            calibration_registry: None,
            // F-3p1 / Slice C3: adapter attached post-construction via
            // `with_capability_flow_adapter`; default None (pass-through).
            #[cfg(feature = "mcp")]
            capability_flow_adapter: None,
        };
        this.emit_transport_version_once();
        this
    }

    /// Construct with just a port; no SSB emission. Used by tests + the
    /// contract suite where the runtime is too lightweight to host an
    /// Ssb instance.
    pub fn new_noop() -> Self {
        // Default port matches DEFAULT_MCP_SERVER_URL in plugins/src/mcp_bridge.rs.
        Self {
            port: 8200,
            ssb: None,
            #[cfg(feature = "mcp")]
            calibration_registry: None,
            #[cfg(feature = "mcp")]
            capability_flow_adapter: None,
        }
    }

    /// Builder: attach an SSB writer post-construction. Used by the
    /// runtime wiring path when the writer becomes available after the
    /// dispatcher is constructed.
    pub fn with_ssb(mut self, ssb: Arc<Ssb>) -> Self {
        self.ssb = Some(ssb);
        self
    }

    /// F-3p1 / Slice C3 (ENFORCE B): wire a `CapabilityFlowAdapter` into
    /// each per-call `DefaultMcpBridge` so `ToolGate::check_tool_allowed`
    /// consults the adapter's committed-set enforcement layer on every
    /// `dispatch_tool` call.
    ///
    /// Call this once at session-start (e.g. from `run_subcommand` or the
    /// dashboard bootstrap) after constructing a `CapabilityFlowAdapter` and
    /// sharing the same `Arc` with the `WebResearchOrchestrator` producer.
    ///
    /// When not called, the dispatch path is byte-identical to the legacy
    /// pre-C3 behaviour (zero behaviour change on the default leg).
    #[cfg(feature = "mcp")]
    pub fn with_capability_flow_adapter(
        mut self,
        adapter: Arc<crate::capability_flow::CapabilityFlowAdapter>,
    ) -> Self {
        self.capability_flow_adapter = Some(adapter);
        self
    }

    /// C-47b CLI-arm builder: attach the long-lived shared calibration
    /// registry. Called once at the `run_subcommand` wiring seam when the
    /// `ELAI_C47B_CASE_BASE_PERSIST_ENABLE` env-gate is on. Mirrors the
    /// dashboard arm's `AppState.calibration_registry` field: the registry
    /// (with its `CasePersistStore`) is constructed once and the `Arc` is
    /// cloned into every per-call `DefaultMcpBridge` inside
    /// [`Self::dispatch_tool`], so observations accumulate in the one
    /// shared case base rather than in throwaway per-call bridge state.
    /// When this is never called, `dispatch_tool` keeps the exact
    /// `http_call_tool` path (zero behaviour change on the default leg).
    #[cfg(feature = "mcp")]
    pub fn with_calibration_registry(
        mut self,
        registry: Arc<elai_plugins::ToolCalibrationRegistry>,
    ) -> Self {
        self.calibration_registry = Some(registry);
        self
    }

    /// E-38 -- emit the one-shot transport-version log. Idempotent
    /// across multiple `new` calls via [`MCP_TRANSPORT_VERSION_LOGGED`].
    /// No-op when [`roadmap_signals_enabled`] returns false or no SSB
    /// writer is attached. The row carries the current transport label
    /// + the canonical roadmap target so consumers can compute drift
    /// without re-issuing a config query.
    fn emit_transport_version_once(&self) {
        if !roadmap_signals_enabled() {
            return;
        }
        let Some(ssb) = self.ssb.as_ref() else { return };
        // compare_exchange so concurrent constructors land exactly one
        // emit per process boot.
        if MCP_TRANSPORT_VERSION_LOGGED
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return;
        }
        let ts_ms = i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX);
        let payload = SsbPayload::AiRoadmapSignalEvent(AiRoadmapSignalEventPayload {
            kind: RoadmapSignalKind::McpTransportVersionLogged,
            plan_session_id: String::new(),
            assertion_label: String::new(),
            hint: String::new(),
            assertions_checked: 0,
            mcp_transport_current: MCP_TRANSPORT_CURRENT_DEFAULT.to_string(),
            mcp_transport_target: MCP_TRANSPORT_ROADMAP_TARGET.to_string(),
            backend_name: String::new(),
            declared_context_tokens: 0,
            mrcr_score_label: String::new(),
            posture_doc_path: String::new(),
            posture_doc_mtime_ms: 0,
            age_days: 0,
            details_json: serde_json::json!({
                "caller_site": "mcp_client_dispatch_new",
                "port": self.port,
                "roadmap_source": "modelcontextprotocol.io/development/roadmap"
            }),
            ts_ms,
        });
        let ssb = ssb.clone();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let rt = match rt {
            Ok(rt) => rt,
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "mcp_transport_version_logged: ssb runtime build failed; skipping emit"
                );
                return;
            }
        };
        let result = rt.block_on(async move {
            ssb.write_payload("mcp-bridge", 0, payload, Vec::new()).await
        });
        if let Err(e) = result {
            tracing::warn!(
                error = %e,
                "mcp_transport_version_logged: ssb emit failed (non-fatal)"
            );
        }
    }

    fn emit_ssb(&self, tool_name: &str) {
        let Some(ssb) = self.ssb.as_ref() else { return };
        let payload = SsbPayload::CliMcpDispatch(CliMcpDispatchPayload {
            tool_name: tool_name.to_owned(),
            port: self.port,
        });
        // Fire-and-forget emit on a current-thread runtime. Errors are
        // logged but never propagated to the dispatch caller -- a missing
        // SSB row is a telemetry gap, not a dispatch failure.
        let ssb = ssb.clone();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let rt = match rt {
            Ok(rt) => rt,
            Err(e) => {
                tracing::warn!(error = %e, "cli_mcp_dispatch: ssb runtime build failed; skipping emit");
                return;
            }
        };
        let result = rt.block_on(async move {
            ssb.write_payload("cli-mcp-dispatch", 0, payload, Vec::new()).await
        });
        if let Err(e) = result {
            tracing::warn!(error = %e, "cli_mcp_dispatch: ssb emit failed");
        }
    }
}

impl Dispatch for McpClientDispatch {
    fn dispatch_tool(&self, req: &DispatchRequest) -> DispatchResult {
        // C-39: always emit one CliMcpDispatch row per call, even on
        // network failure -- the row records the ATTEMPT so dashboards
        // and CI parity scripts see the symmetric event stream.
        self.emit_ssb(&req.tool_name);

        // Build a current-thread runtime and round-trip the call through
        // DefaultMcpBridge::http_call_tool. The bridge handles JSON-RPC 2.0
        // framing + result sanitization. stub_ban: no silent retry; the
        // first error surfaces to the caller.
        #[cfg(feature = "mcp")]
        {
            use std::sync::Arc as StdArc;
            use elai_plugins::mcp_bridge::DefaultMcpBridge;
            use elai_plugins::ToolGate;
            use elai_plugins::LazySchemaCache;
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("cli_mcp_dispatch: runtime build failed: {}", e))?;
            let url = format!("http://localhost:{}", self.port);
            let tool_name = req.tool_name.clone();
            let args = req.args.clone();
            // F-3p1 / Slice C3 (ENFORCE B): when a CapabilityFlowAdapter is
            // wired, build a ToolGate with the adapter as its committed checker
            // so ToolGate::check_tool_allowed consults the committed set via
            // the CommittedSetChecker trait. This is a strict SUPERSET of the
            // existing paths: when the adapter's inner slot is None (no active
            // commitment), check_committed returns Ok(()) -- zero regression on
            // sessions that do not use WebResearchOrchestrator. The gate source
            // is SafeDefault so no per-state tool rules change; ONLY the
            // committed-set layer is added.
            //
            // C-47b CLI-arm: calibration registry path layered on top of the
            // capability adapter path -- both can coexist.
            let result = match (
                self.capability_flow_adapter.as_ref(),
                self.calibration_registry.as_ref(),
            ) {
                (Some(adapter), Some(registry)) => {
                    let gate = StdArc::new(
                        ToolGate::safe_default()
                            .with_committed_checker(adapter.clone() as StdArc<dyn elai_plugins::CommittedSetChecker + Send + Sync>),
                    );
                    let cache = StdArc::new(LazySchemaCache::new(1));
                    let bridge = DefaultMcpBridge::default()
                        .with_tool_gate(gate, cache)
                        .with_calibration_registry(registry.clone());
                    rt.block_on(async move {
                        bridge
                            .dispatch_tool_call_with_gate(&url, &tool_name, "", args)
                            .await
                    })
                }
                (Some(adapter), None) => {
                    let gate = StdArc::new(
                        ToolGate::safe_default()
                            .with_committed_checker(adapter.clone() as StdArc<dyn elai_plugins::CommittedSetChecker + Send + Sync>),
                    );
                    let cache = StdArc::new(LazySchemaCache::new(1));
                    let bridge = DefaultMcpBridge::default().with_tool_gate(gate, cache);
                    rt.block_on(async move {
                        bridge
                            .dispatch_tool_call_with_gate(&url, &tool_name, "", args)
                            .await
                    })
                }
                (None, Some(registry)) => {
                    // Legacy C-47b path (no capability adapter).
                    let bridge =
                        DefaultMcpBridge::default().with_calibration_registry(registry.clone());
                    rt.block_on(async move {
                        bridge
                            .dispatch_tool_call_with_gate(&url, &tool_name, "", args)
                            .await
                    })
                }
                (None, None) => {
                    let bridge = DefaultMcpBridge::default();
                    rt.block_on(async move { bridge.http_call_tool(&url, &tool_name, args).await })
                }
            };
            result
                .map(|r| r.content)
                .map_err(|e| format!("cli_mcp_dispatch: bridge call failed: {}", e))
        }
        #[cfg(not(feature = "mcp"))]
        {
            // Without the mcp feature there is no DefaultMcpBridge in the
            // dependency graph. Surface a structured error -- callers
            // should activate the feature before selecting --via-mcp.
            Err(format!(
                "cli_mcp_dispatch: plugins/mcp feature not compiled in (port {})",
                self.port
            ))
        }
    }
}
