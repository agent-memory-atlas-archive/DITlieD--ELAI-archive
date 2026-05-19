// Dispatch context and backend selection for local/API inference routing.
//
// Provides DispatchContext (per-turn request metadata), BackendMode (routing
// enum), and build_dispatch_context() which assembles a context from FSM state,
// budget breakdown, and backend config.

use crate::context_budget::{BudgetError, ContextBudget, GenerationProfile, ModelCapabilityProfile};
use crate::fsm::FsmState;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;

// ---------------------------------------------------------------------------
// PhaseSlot — batch-fu (FU.1)
// ---------------------------------------------------------------------------

/// Per-dispatch phase-content payload threaded into [`build_dispatch_context`].
///
/// Carries four pieces of context a dispatch site may know about that
/// `build_dispatch_context` alone cannot synthesize:
///
/// - `goal`: the natural-language goal of the current phase.
/// - `constraints`: constitutional CONSTRAINTS (A-7). When non-empty,
///   [`build_dispatch_context`] prepends a `CONSTRAINTS:` block to the
///   assembled `PhaseSpec` slot before budgeting.
/// - `profile_hint`: the [`GenerationProfile`] the caller wants applied
///   (A-18). When set to [`GenerationProfile::RepairRetry`], downstream
///   callers may also invoke the context-telescope (A-46) on the repair
///   snippet via the separate `telescope_caller` surface — this struct is
///   the pure input carrier; side-effects (SSB emission, telescope calls)
///   belong to the call site, not `build_dispatch_context`.
/// - `repair_snippet`: optional prior failure context that the telescope
///   caller collapses into the Slot-3 budget (A-46 input side). `None`
///   when the dispatch is not a repair retry.
///
/// `Default` yields an empty slot — behaviour-equivalent to `None` on
/// [`build_dispatch_context`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PhaseSlot {
    /// Natural-language phase goal (may be empty on non-phase dispatches).
    pub goal: String,
    /// Batch-fu (A-7): constitutional CONSTRAINTS list sourced from
    /// [`crate::dispatcher`]'s caller's `ProjectProfile.constraints`.
    pub constraints: Vec<String>,
    /// Batch-fu (A-18): per-dispatch generation-profile hint. When set and
    /// non-default, downstream code emits a `ProfileSelected` SSB row.
    pub profile_hint: Option<GenerationProfile>,
    /// Batch-fu (A-46): optional snippet-of-prior-failure carried into the
    /// repair context assembly. Consumed by `telescope_caller::invoke_telescope`
    /// outside this pure assembly function.
    pub repair_snippet: Option<String>,
}

/// Header used to prefix the CONSTRAINTS block in the assembled PhaseSpec
/// slot. Exposed so tests / dashboard can scan assembled output without
/// duplicating the literal.
pub const CONSTRAINTS_BLOCK_HEADER: &str = "CONSTRAINTS:";

// ---------------------------------------------------------------------------
// BackendMode
// ---------------------------------------------------------------------------

/// C-5: provider-routing mode for the orchestrator's task runner.
///
/// `Api` and `Local` predate C-5 and remain the canonical pin states for
/// "always cloud" / "always local sidecar". The new `Hybrid` variant is
/// the production wiring for the cloud-mode escalation loop: the runner
/// first attempts the local sidecar and, on retry exhaustion or an
/// explicit escalation rule, promotes the baby step to the cloud via
/// [`crate::cloud_mode::CloudModeRouter`].
///
/// The CLI `--provider local|cloud|hybrid` flag is the source of truth
/// for selecting between these three at runtime. The dashboard's
/// `set_execution_mode` Tauri command accepts the matching upper-case
/// labels (`"LOCAL"`, `"CLOUD"`, `"HYBRID"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendMode {
    Api,
    Local,
    Hybrid,
}

impl Default for BackendMode {
    fn default() -> Self {
        Self::Api
    }
}

impl FromStr for BackendMode {
    type Err = DispatchError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "api" | "cloud" => Ok(Self::Api),
            "local" => Ok(Self::Local),
            "hybrid" | "auto" => Ok(Self::Hybrid),
            other => Err(DispatchError::Config(format!(
                "unknown backend mode: {other}"
            ))),
        }
    }
}

impl std::fmt::Display for BackendMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api => write!(f, "api"),
            Self::Local => write!(f, "local"),
            Self::Hybrid => write!(f, "hybrid"),
        }
    }
}

// ---------------------------------------------------------------------------
// BackendConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    #[serde(default)]
    pub mode: BackendMode,
    #[serde(default = "default_sidecar_url")]
    pub sidecar_url: String,
    #[serde(default = "default_local_model")]
    pub model: String,
    #[serde(default = "default_max_tools")]
    pub max_tools_local: usize,
    /// Ollama fallback URL when sidecar is unavailable (degraded mode).
    #[serde(default)]
    pub fallback_url: Option<String>,
}

fn default_sidecar_url() -> String {
    "http://localhost:8100".to_string()
}

fn default_local_model() -> String {
    "gemma4:e2b".to_string()
}

fn default_max_tools() -> usize {
    8
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            mode: BackendMode::default(),
            sidecar_url: default_sidecar_url(),
            model: default_local_model(),
            max_tools_local: default_max_tools(),
            fallback_url: None,
        }
    }
}

// ---------------------------------------------------------------------------
// DispatchError
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum DispatchError {
    #[error("fsm error: {0}")]
    Fsm(#[from] crate::fsm::FsmError),

    #[error("budget error: {0}")]
    Budget(#[from] BudgetError),

    #[error("config error: {0}")]
    Config(String),
}

// ---------------------------------------------------------------------------
// DispatchContext
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DispatchContext {
    pub fsm_state: FsmState,
    pub backend_mode: BackendMode,
    pub sidecar_url: String,
    pub model: String,
    pub filtered_tool_names: Vec<String>,
    pub max_tools: usize,
    pub budget: Option<ContextBudget>,
    /// Batch-fu (FU.1): assembled phase-content slot body. Carries the
    /// CONSTRAINTS block (A-7) when `phase_slot.constraints` is non-empty,
    /// followed by the phase goal (when set). `None` when no phase slot was
    /// provided to [`build_dispatch_context`] — preserves pre-batch-fu
    /// behaviour for every current caller that passes `None`.
    pub phase_content: Option<String>,
    /// Batch-fu (FU.1 / A-18): generation profile applied to this dispatch.
    /// Defaults to [`GenerationProfile::FirstAttempt`] when no hint is
    /// supplied. Carried into the context so the LLM-provider layer can
    /// apply the profile's temperature/top_p without a second lookup.
    pub profile: GenerationProfile,
}

// ---------------------------------------------------------------------------
// build_dispatch_context
// ---------------------------------------------------------------------------

/// Local model context ceiling: 131_072 tokens, 80% usable.
pub const LOCAL_MODEL_MAX_CONTEXT: usize = 131_072;
pub const LOCAL_BUDGET_FRACTION: usize = 80;

pub fn build_dispatch_context(
    fsm_state: FsmState,
    config: &BackendConfig,
    all_tool_names: &[String],
    priority_map: &BTreeMap<String, u8>,
    profile: Option<&ModelCapabilityProfile>,
    phase_slot: Option<&PhaseSlot>,
) -> Result<DispatchContext, DispatchError> {
    // Pre-filter tools by FSM state.
    let filtered: Vec<String> = all_tool_names
        .iter()
        .filter(|name| fsm_state.check_tool_allowed(name).is_ok())
        .cloned()
        .collect();

    // Sort by priority (lower = more important), then cap at max_tools_local.
    let max_tools = config.max_tools_local;
    let capped: Vec<String> = if filtered.len() > max_tools {
        let mut sorted = filtered;
        sorted.sort_by_key(|name| priority_map.get(name).copied().unwrap_or(u8::MAX));
        sorted.into_iter().take(max_tools).collect()
    } else {
        filtered
    };

    // Compute budget: use profile-aware allocation if profile is available,
    // otherwise fall back to fixed-percentage allocation.
    let raw_max = LOCAL_MODEL_MAX_CONTEXT * LOCAL_BUDGET_FRACTION / 100;
    let budget = match profile {
        Some(p) => ContextBudget::compute_with_profile(raw_max, p.fixed_slot_targets, p).ok(),
        None => ContextBudget::compute(raw_max, [0, 0, 0], 4096).ok(),
    };

    // Batch-fu (FU.1 + A-7): assemble the phase-content slot when a phase
    // slot was supplied. The order is fixed:
    //   1. `CONSTRAINTS:` block (one bullet per entry) when
    //      `phase_slot.constraints` is non-empty.
    //   2. A blank-line separator.
    //   3. The phase goal (when non-empty).
    //
    // Empty phase_slot OR `None` → `phase_content = None`, preserving
    // pre-batch-fu behaviour for every caller that passes `None`.
    let phase_content = phase_slot.and_then(assemble_phase_content);

    // Batch-fu (FU.1 / A-18): pick the applied profile. Caller-supplied hint
    // wins; absent hint falls through to `FirstAttempt` (the pre-batch-fu
    // implicit behaviour).
    let applied_profile = phase_slot
        .and_then(|s| s.profile_hint)
        .unwrap_or(GenerationProfile::FirstAttempt);

    Ok(DispatchContext {
        fsm_state,
        backend_mode: config.mode,
        sidecar_url: config.sidecar_url.clone(),
        model: config.model.clone(),
        filtered_tool_names: capped,
        max_tools,
        budget,
        phase_content,
        profile: applied_profile,
    })
}

/// Render a [`PhaseSlot`] into the assembled PhaseSpec body. Returns `None`
/// when every field is empty so a populated-but-useless slot does not
/// produce a stray empty string on the `DispatchContext`.
fn assemble_phase_content(slot: &PhaseSlot) -> Option<String> {
    let has_constraints = !slot.constraints.is_empty();
    let has_goal = !slot.goal.is_empty();
    if !has_constraints && !has_goal {
        return None;
    }
    let mut out = String::new();
    if has_constraints {
        out.push_str(CONSTRAINTS_BLOCK_HEADER);
        out.push('\n');
        for c in &slot.constraints {
            out.push_str("- ");
            out.push_str(c);
            out.push('\n');
        }
    }
    if has_goal {
        if has_constraints {
            out.push('\n');
        }
        out.push_str(&slot.goal);
    }
    Some(out)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_mode_from_str_roundtrip() {
        for (input, expected) in [
            ("api", BackendMode::Api),
            ("local", BackendMode::Local),
            ("LOCAL", BackendMode::Local),
            ("Api", BackendMode::Api),
        ] {
            let parsed: BackendMode = input.parse().expect(input);
            assert_eq!(parsed, expected);
            // Display roundtrip for canonical lowercase
            let displayed = parsed.to_string();
            let reparsed: BackendMode = displayed.parse().expect(&displayed);
            assert_eq!(reparsed, expected);
        }
    }

    #[test]
    fn backend_mode_from_str_rejects_unknown() {
        assert!("foo".parse::<BackendMode>().is_err());
        assert!("".parse::<BackendMode>().is_err());
    }

    #[test]
    fn dispatch_context_executing_state() {
        let config = BackendConfig::default();
        let tools: Vec<String> = vec![
            "bash".into(),
            "edit_file".into(),
            "read_file".into(),
            "write_file".into(),
            "WebSearch".into(),
        ];

        let prio = BTreeMap::new();
        let ctx =
            build_dispatch_context(FsmState::Executing, &config, &tools, &prio, None, None).expect("should succeed");

        assert_eq!(ctx.fsm_state, FsmState::Executing);
        assert_eq!(ctx.backend_mode, BackendMode::Api);
        // Executing allows bash, edit, read, write but blocks planning tools.
        // All 5 tools here should be allowed in Executing state.
        assert!(ctx.filtered_tool_names.contains(&"bash".to_string()));
        assert!(ctx.filtered_tool_names.contains(&"edit_file".to_string()));
        assert!(ctx.filtered_tool_names.contains(&"read_file".to_string()));
    }

    #[test]
    fn dispatch_context_planning_blocks_edit() {
        let config = BackendConfig::default();
        let tools: Vec<String> = vec![
            "bash".into(),
            "edit_file".into(),
            "read_file".into(),
            "write_file".into(),
            "WebSearch".into(),
            "EnterPlanMode".into(),
        ];

        let prio = BTreeMap::new();
        let ctx =
            build_dispatch_context(FsmState::Planning, &config, &tools, &prio, None, None).expect("should succeed");

        // Planning blocks: bash, edit_file, write_file
        assert!(!ctx.filtered_tool_names.contains(&"bash".to_string()));
        assert!(!ctx.filtered_tool_names.contains(&"edit_file".to_string()));
        assert!(!ctx.filtered_tool_names.contains(&"write_file".to_string()));
        // Planning allows: read_file, WebSearch, EnterPlanMode
        assert!(ctx.filtered_tool_names.contains(&"read_file".to_string()));
        assert!(ctx.filtered_tool_names.contains(&"WebSearch".to_string()));
        assert!(ctx
            .filtered_tool_names
            .contains(&"EnterPlanMode".to_string()));
    }

    #[test]
    fn dispatch_context_caps_tools_at_max() {
        let config = BackendConfig {
            max_tools_local: 3,
            ..Default::default()
        };
        // All these are allowed in Idle state.
        let tools: Vec<String> = (0..10).map(|i| format!("read_file_{i}")).collect();
        // Assign descending priorities so read_file_0 = highest priority (0).
        let prio: BTreeMap<String, u8> = (0..10u8).map(|i| (format!("read_file_{i}"), i)).collect();

        let ctx =
            build_dispatch_context(FsmState::Idle, &config, &tools, &prio, None, None).expect("should succeed");

        // Idle allows everything, but capped at 3. Priority sort keeps 0, 1, 2.
        assert!(ctx.filtered_tool_names.len() <= 3);
        assert!(ctx.filtered_tool_names.contains(&"read_file_0".to_string()));
        assert!(ctx.filtered_tool_names.contains(&"read_file_1".to_string()));
        assert!(ctx.filtered_tool_names.contains(&"read_file_2".to_string()));
    }

    #[test]
    fn default_backend_config_values() {
        let config = BackendConfig::default();
        assert_eq!(config.mode, BackendMode::Api);
        assert_eq!(config.sidecar_url, "http://localhost:8100");
        assert_eq!(config.model, "gemma4:e2b");
        assert_eq!(config.max_tools_local, 8);
    }

    #[test]
    fn backend_config_serde_roundtrip() {
        let config = BackendConfig {
            mode: BackendMode::Local,
            sidecar_url: "http://localhost:9000".to_string(),
            model: "gemma3:4b".to_string(),
            max_tools_local: 10,
            fallback_url: None,
        };
        let json = serde_json::to_string(&config).expect("serialize");
        let parsed: BackendConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.mode, BackendMode::Local);
        assert_eq!(parsed.sidecar_url, "http://localhost:9000");
        assert_eq!(parsed.model, "gemma3:4b");
        assert_eq!(parsed.max_tools_local, 10);
    }
}
