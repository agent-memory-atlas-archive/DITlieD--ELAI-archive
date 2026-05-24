//! C-35 (SCHEMA_VERSION 88): Tool Attention Dynamic Gating.
//! WIRING_CONTRACT produces: ToolGate, GatedCatalog, SummaryRow.
//! stub_ban: filter_for_state MUST NOT silently fall through to full catalog
//! when gate manifest missing. Uses safe default (eager=[], gated=ALL, denied=[]).

use std::collections::{HashMap, HashSet};
use thiserror::Error;
use crate::mcp_bridge::McpToolSpec;

pub type FullSchema = McpToolSpec;

/// C-35 M4: Summarize trait. Maps a full tool schema to the 1-line summary that
/// goes into the gated_summaries section of the assembled prompt. Plan-promised
/// "schema.rs Summarize trait" lands here alongside the gate types to keep
/// related logic in one module (mcp_bridge.rs already owns McpToolSpec, splitting
/// across three modules buys nothing).
pub trait Summarize {
    /// Single-line tool summary suitable for the gated section of the prompt.
    fn summarize(&self) -> String;
}

impl Summarize for McpToolSpec {
    fn summarize(&self) -> String {
        self.description.clone()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SummaryRow {
    pub name: String,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct GatedCatalog {
    pub eager_schemas: Vec<FullSchema>,
    pub gated_summaries: Vec<SummaryRow>,
    pub denied_names: HashSet<String>,
}

impl GatedCatalog {
    pub fn prompt_tokens_saved(&self, full_size_bytes: usize, chars_per_token: usize) -> usize {
        let eager: usize = self.eager_schemas.iter()
            .map(|s| serde_json::to_string(s).unwrap_or_default().len()).sum();
        let summary: usize = self.gated_summaries.iter()
            .map(|s| format!("{}: {}", s.name, s.summary).len()).sum();
        full_size_bytes.saturating_sub(eager + summary) / chars_per_token.max(1)
    }
    pub fn gated_tool_count_for_prompt(&self) -> usize { self.gated_summaries.len() }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolGateStateConfig {
    pub eager: Vec<String>,
    pub gated: Vec<String>,
    pub denied: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolGateConfig {
    pub states: HashMap<String, ToolGateStateConfig>,
    pub max_eager_schemas: usize,
}

#[derive(Debug, Error)]
pub enum ToolGateError {
    #[error("tool {tool_name:?} is denied in state {fsm_state:?}")]
    ToolDenied { tool_name: String, fsm_state: String },
    #[error("gate manifest IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("gate manifest parse error: {0}")]
    Parse(#[from] toml::de::Error),
}

#[derive(Debug, Clone)]
enum GateSource { Config(ToolGateConfig), SafeDefault }

#[derive(Debug, Clone)]
pub struct ToolGate { source: GateSource }

impl ToolGate {
    pub fn from_config(config: ToolGateConfig) -> Self { Self { source: GateSource::Config(config) } }
    pub fn safe_default() -> Self { Self { source: GateSource::SafeDefault } }
    pub fn reason(&self) -> &str {
        match &self.source { GateSource::SafeDefault => "manifest_absent", GateSource::Config(_) => "config_loaded" }
    }
    pub fn load(manifest_path: &std::path::Path) -> Result<Self, ToolGateError> {
        let content = std::fs::read_to_string(manifest_path)?;
        let raw: TomlManifest = toml::from_str(&content)?;
        let states = raw.gate.into_iter().map(|(k, v)| (k.to_lowercase(), ToolGateStateConfig {
            eager: v.eager.unwrap_or_default(),
            gated: v.gated.unwrap_or_default(),
            denied: v.denied.unwrap_or_default(),
        })).collect();
        Ok(Self::from_config(ToolGateConfig { states, max_eager_schemas: 4 }))
    }
    pub fn filter_for_state(&self, fsm_state: &str, full_catalog: &[McpToolSpec]) -> GatedCatalog {
        match &self.source {
            GateSource::SafeDefault => self.all_gated(full_catalog),
            GateSource::Config(cfg) => match cfg.states.get(&fsm_state.to_lowercase()) {
                None => self.all_gated(full_catalog),
                Some(sc) => {
                    let eager_set: HashSet<&str> = sc.eager.iter().map(|s| s.as_str()).collect();
                    let denied_set: HashSet<&str> = sc.denied.iter().map(|s| s.as_str()).collect();
                    let mut eager_schemas = Vec::new();
                    let mut gated_summaries = Vec::new();
                    for tool in full_catalog {
                        if denied_set.contains(tool.name.as_str()) { continue; }
                        if eager_set.contains(tool.name.as_str()) && eager_schemas.len() < cfg.max_eager_schemas {
                            eager_schemas.push(tool.clone());
                        } else {
                            gated_summaries.push(SummaryRow { name: tool.name.clone(), summary: tool.description.clone() });
                        }
                    }
                    GatedCatalog { eager_schemas, gated_summaries, denied_names: sc.denied.iter().cloned().collect() }
                }
            }
        }
    }
    fn all_gated(&self, full_catalog: &[McpToolSpec]) -> GatedCatalog {
        GatedCatalog {
            eager_schemas: vec![],
            gated_summaries: full_catalog.iter().map(|s| SummaryRow { name: s.name.clone(), summary: s.description.clone() }).collect(),
            denied_names: HashSet::new(),
        }
    }
    pub fn check_tool_allowed(&self, fsm_state: &str, tool_name: &str) -> Result<(), ToolGateError> {
        match &self.source {
            GateSource::SafeDefault => Ok(()),
            GateSource::Config(cfg) => {
                if cfg.states.get(&fsm_state.to_lowercase()).map(|s| s.denied.iter().any(|d| d == tool_name)).unwrap_or(false) {
                    Err(ToolGateError::ToolDenied { tool_name: tool_name.to_owned(), fsm_state: fsm_state.to_owned() })
                } else { Ok(()) }
            }
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct TomlManifest { gate: HashMap<String, TomlStateEntry> }

#[derive(Debug, serde::Deserialize)]
struct TomlStateEntry {
    eager: Option<Vec<String>>,
    gated: Option<Vec<String>>,
    denied: Option<Vec<String>>,
}

pub fn load_gate(manifest_path: &std::path::Path) -> Result<ToolGate, ToolGateError> {
    if !manifest_path.exists() {
        tracing::warn!(path = ?manifest_path, "tool gate manifest missing, using safe default (eager=[], gated=ALL, denied=[])");
        return Ok(ToolGate::safe_default());
    }
    ToolGate::load(manifest_path)
}
