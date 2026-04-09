// Dispatch context and backend selection for local/API inference routing.
//
// Provides DispatchContext (per-turn request metadata), BackendMode (routing
// enum), and build_dispatch_context() which assembles a context from FSM state,
// budget breakdown, and backend config.

use crate::context_budget::{BudgetError, ContextBudget};
use crate::fsm::FsmState;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

// ---------------------------------------------------------------------------
// BackendMode
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendMode {
    Api,
    Local,
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
            "api" => Ok(Self::Api),
            "local" => Ok(Self::Local),
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
}

fn default_sidecar_url() -> String {
    "http://localhost:8100".to_string()
}

fn default_local_model() -> String {
    "gemma4:e2b".to_string()
}

fn default_max_tools() -> usize {
    15
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            mode: BackendMode::default(),
            sidecar_url: default_sidecar_url(),
            model: default_local_model(),
            max_tools_local: default_max_tools(),
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
) -> Result<DispatchContext, DispatchError> {
    // Pre-filter tools by FSM state.
    let filtered: Vec<String> = all_tool_names
        .iter()
        .filter(|name| fsm_state.check_tool_allowed(name).is_ok())
        .cloned()
        .collect();

    // Cap at max_tools_local for local backend.
    let max_tools = config.max_tools_local;
    let capped: Vec<String> = if filtered.len() > max_tools {
        filtered.into_iter().take(max_tools).collect()
    } else {
        filtered
    };

    // Compute budget for local model (best-effort).
    let budget = ContextBudget::compute(
        LOCAL_MODEL_MAX_CONTEXT * LOCAL_BUDGET_FRACTION / 100,
        [0, 0, 0],
        4096,
    )
    .ok();

    Ok(DispatchContext {
        fsm_state,
        backend_mode: config.mode,
        sidecar_url: config.sidecar_url.clone(),
        model: config.model.clone(),
        filtered_tool_names: capped,
        max_tools,
        budget,
    })
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

        let ctx =
            build_dispatch_context(FsmState::Executing, &config, &tools).expect("should succeed");

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

        let ctx =
            build_dispatch_context(FsmState::Planning, &config, &tools).expect("should succeed");

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

        let ctx =
            build_dispatch_context(FsmState::Idle, &config, &tools).expect("should succeed");

        // Idle allows everything, but capped at 3.
        // Note: read_file_0..9 are unknown tools — ToolCategory::classify falls through
        // to a default. We're testing the cap logic, not classification.
        assert!(ctx.filtered_tool_names.len() <= 3);
    }

    #[test]
    fn default_backend_config_values() {
        let config = BackendConfig::default();
        assert_eq!(config.mode, BackendMode::Api);
        assert_eq!(config.sidecar_url, "http://localhost:8100");
        assert_eq!(config.model, "gemma4:e2b");
        assert_eq!(config.max_tools_local, 15);
    }

    #[test]
    fn backend_config_serde_roundtrip() {
        let config = BackendConfig {
            mode: BackendMode::Local,
            sidecar_url: "http://localhost:9000".to_string(),
            model: "gemma3:4b".to_string(),
            max_tools_local: 10,
        };
        let json = serde_json::to_string(&config).expect("serialize");
        let parsed: BackendConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.mode, BackendMode::Local);
        assert_eq!(parsed.sidecar_url, "http://localhost:9000");
        assert_eq!(parsed.model, "gemma3:4b");
        assert_eq!(parsed.max_tools_local, 10);
    }
}
