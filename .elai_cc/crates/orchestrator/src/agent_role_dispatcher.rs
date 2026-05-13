//! Phase A5 — `AgentRoleDispatcher`.
//!
//! Library-tier router that maps a role hint (e.g. `"executor"` or
//! `"prompt_engineer"`) onto the `provider/model` pair configured in the
//! dashboard's `AppState.model_routing` slot.
//!
//! Inputs and outputs:
//!   - Input: a `serde_json::Value` matching the JSON-array shape produced
//!     at `crates/dashboard-v2/src-tauri/src/state.rs:402-411`:
//!     `[ { "role": "<id>", "target": "<kind:model>", "fallback": "<…>"? }, … ]`.
//!   - Output: a [`RoleDispatch`] data struct describing the provider variant
//!     and concrete model identifier the caller should instantiate.
//!
//! Scope (Phase A5):
//!   - Pure routing data structure + parser. **No** `ConversationRuntime`
//!     instantiation; **no** `LlmProvider` construction; **no** network I/O.
//!     The placeholder at `task_runner.rs:2470-2503` remains untouched and
//!     is wired in a follow-up patch.
//!
//! Drift handling:
//!   - `state.rs` writes the routing array with kebab-case role IDs
//!     (`"prompt-engineer"`) but the canonical `AgentRole` enum at
//!     `ipc-types/src/lib.rs:14` serializes as snake_case
//!     (`"prompt_engineer"`). [`AgentRoleDispatcher::pick_role`] normalises
//!     both forms by replacing `-` with `_` before comparison so callers
//!     can hand it either shape without surprises.

use std::sync::{Arc, Mutex};

use elai_providers::ProviderKind;
use serde::Deserialize;
use thiserror::Error;

/// One row of the dashboard `model_routing` array.
///
/// Mirrors the JSON shape committed at `state.rs:402-411`; the `fallback`
/// field is optional in the source schema so we keep it `Option<String>`
/// here as well.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RoleConfig {
    /// Role identifier as written in `model_routing` (kebab-case in the
    /// current default, but consumers MUST treat the value as opaque and
    /// rely on [`AgentRoleDispatcher::pick_role`] for lookups so the
    /// kebab/snake normalisation runs).
    pub role: String,
    /// Provider + model selector in the form `"<kind>:<model>"` — e.g.
    /// `"anthropic:claude-sonnet-4-6"`, `"local:qwen-2.5-coder-7b"`,
    /// `"xai:grok-3"`. Parsed into [`RoleDispatch`] by `build_dispatch`.
    pub target: String,
    /// Optional secondary target used by future failover logic.
    /// Phase A5 stores and forwards it untouched; no consumer reads it yet.
    #[serde(default)]
    pub fallback: Option<String>,
}

/// Resolved dispatch handle: the provider variant + the concrete model id
/// the caller should hand to a `ConversationRuntime` or `LlmProvider`.
///
/// This is intentionally a data struct rather than a live runtime — Phase
/// A5 stops at routing; the production wiring (instantiating the
/// provider impl, attaching tools, fetching the system prompt) is a
/// follow-up patch driven by the placeholder at `task_runner.rs:2470`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleDispatch {
    /// Which provider family services this role.
    pub provider_kind: ProviderKind,
    /// Concrete model identifier passed to the provider (e.g.
    /// `"claude-sonnet-4-6"` for `ProviderKind::Anthropic`, or the local
    /// sidecar slug for `ProviderKind::Ollama`).
    pub model: String,
    /// Tool names the dispatched runtime is permitted to call. Empty in
    /// the library-tier path; production wiring populates it from the
    /// per-role policy table.
    pub tools: Vec<String>,
    /// Optional system prompt override for the role. `None` means "use
    /// the runtime's default" — populated by the production wiring layer.
    pub system_prompt: Option<String>,
}

/// Errors surfaced by the dispatcher. All variants are recoverable from
/// the caller's point of view — none represent corrupted state.
#[derive(Debug, Error)]
pub enum RoleDispatchError {
    /// The routing payload was not a JSON array. Includes the actual JSON
    /// type for diagnostics.
    #[error("model_routing must be a JSON array, got {0}")]
    InvalidRoutingShape(&'static str),
    /// A single entry in the array failed to deserialize into
    /// [`RoleConfig`] (missing required `role`/`target`, wrong types …).
    #[error("invalid role entry: {0}")]
    InvalidRoleEntry(String),
    /// `pick_role` did not find any entry whose normalised role id
    /// matched the caller's hint.
    #[error("role not found: {0}")]
    RoleNotFound(String),
    /// `target` string did not contain a recognised `<kind>:<model>`
    /// prefix and could not be classified from the model name alone.
    #[error("could not determine provider for target '{0}'")]
    UnknownProvider(String),
    /// `target` parsed to a prefix but the trailing model identifier was
    /// empty (e.g. `"anthropic:"`).
    #[error("missing model identifier in target '{0}'")]
    MissingModel(String),
    /// Held when the dispatcher is constructed from a shared
    /// `Arc<Mutex<Value>>` and the lock has been poisoned by a panicking
    /// writer.
    #[error("model_routing mutex poisoned")]
    RoutingMutexPoisoned,
}

/// Role-aware router over the dashboard `model_routing` slot.
#[derive(Debug, Clone)]
pub struct AgentRoleDispatcher {
    roles: Vec<RoleConfig>,
}

impl AgentRoleDispatcher {
    /// Parse a `model_routing` JSON value into a dispatcher.
    ///
    /// Accepts the canonical array-of-objects shape from `state.rs`.
    /// Any other shape (object, scalar, null) is rejected with
    /// [`RoleDispatchError::InvalidRoutingShape`].
    pub fn from_value(value: &serde_json::Value) -> Result<Self, RoleDispatchError> {
        let array = value.as_array().ok_or_else(|| {
            RoleDispatchError::InvalidRoutingShape(json_type_name(value))
        })?;
        let mut roles = Vec::with_capacity(array.len());
        for (idx, entry) in array.iter().enumerate() {
            let role: RoleConfig = serde_json::from_value(entry.clone())
                .map_err(|err| {
                    RoleDispatchError::InvalidRoleEntry(format!(
                        "entry {idx}: {err}"
                    ))
                })?;
            roles.push(role);
        }
        Ok(Self { roles })
    }

    /// Construct from the dashboard's shared `Arc<Mutex<Value>>` slot.
    ///
    /// Takes a snapshot under the lock and immediately releases it; the
    /// dispatcher does **not** retain a reference to the mutex. Callers
    /// that need to react to mutations re-run `from_arc_mutex` after the
    /// `set_model_routing` Tauri command fires.
    pub fn from_arc_mutex(
        routing: &Arc<Mutex<serde_json::Value>>,
    ) -> Result<Self, RoleDispatchError> {
        let guard = routing
            .lock()
            .map_err(|_| RoleDispatchError::RoutingMutexPoisoned)?;
        Self::from_value(&guard)
    }

    /// Number of roles loaded — primarily useful in tests and audit
    /// logging.
    #[must_use]
    pub fn role_count(&self) -> usize {
        self.roles.len()
    }

    /// Find the role config matching `hint`. Both kebab-case
    /// (`"prompt-engineer"`) and snake_case (`"prompt_engineer"`) inputs
    /// resolve identically because both sides are normalised to
    /// snake_case before comparison.
    pub fn pick_role(&self, hint: &str) -> Result<&RoleConfig, RoleDispatchError> {
        let normalised = normalise_role(hint);
        self.roles
            .iter()
            .find(|cfg| normalise_role(&cfg.role) == normalised)
            .ok_or_else(|| RoleDispatchError::RoleNotFound(hint.to_owned()))
    }

    /// Resolve the role's `target` string into a [`RoleDispatch`].
    ///
    /// `tools` and `system_prompt` are left empty/None in this tier —
    /// the production wiring layer fills them in from the per-role
    /// policy tables once it lands.
    pub fn build_dispatch(
        &self,
        role: &RoleConfig,
    ) -> Result<RoleDispatch, RoleDispatchError> {
        let (provider_kind, model) = parse_target(&role.target)?;
        Ok(RoleDispatch {
            provider_kind,
            model,
            tools: Vec::new(),
            system_prompt: None,
        })
    }
}

/// Normalise a role identifier so kebab and snake forms compare equal.
///
/// Lowercases the input and replaces `-` with `_`. Whitespace is trimmed
/// so callers can hand the raw value through without sanitising.
fn normalise_role(raw: &str) -> String {
    raw.trim().to_ascii_lowercase().replace('-', "_")
}

/// Parse a `<kind>:<model>` target into its [`ProviderKind`] and model
/// string.
///
/// Recognised prefixes (case-insensitive):
///   - `anthropic:` -> [`ProviderKind::Anthropic`]
///   - `local:` / `ollama:` -> [`ProviderKind::Ollama`]
///   - `xai:` -> [`ProviderKind::Xai`]
///   - `openai:` -> [`ProviderKind::OpenAi`]
///
/// Targets without a `<kind>:` prefix or with an unrecognised prefix are
/// rejected with [`RoleDispatchError::UnknownProvider`]. We deliberately
/// do **not** fall back to model-name heuristics: the contract's
/// `stub_ban` clause forbids silent provider defaulting, and the upstream
/// `elai_providers::detect_provider_kind` returns `Anthropic` for
/// anything it cannot identify which would be exactly such a silent
/// fallback. Callers (the dashboard `set_model_routing` Tauri command in
/// particular) are responsible for writing the prefix form already shown
/// in the default `model_routing` value at
/// `state.rs:402-411`.
fn parse_target(target: &str) -> Result<(ProviderKind, String), RoleDispatchError> {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return Err(RoleDispatchError::UnknownProvider(target.to_owned()));
    }
    let (prefix, rest) = trimmed
        .split_once(':')
        .ok_or_else(|| RoleDispatchError::UnknownProvider(target.to_owned()))?;
    let model = rest.trim().to_owned();
    if model.is_empty() {
        return Err(RoleDispatchError::MissingModel(target.to_owned()));
    }
    let kind = match prefix.trim().to_ascii_lowercase().as_str() {
        "anthropic" => ProviderKind::Anthropic,
        "local" | "ollama" => ProviderKind::Ollama,
        "xai" => ProviderKind::Xai,
        "openai" => ProviderKind::OpenAi,
        _ => return Err(RoleDispatchError::UnknownProvider(target.to_owned())),
    };
    Ok((kind, model))
}

/// Stringify the JSON type tag for error messages.
fn json_type_name(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalise_handles_kebab_and_snake() {
        assert_eq!(normalise_role("prompt-engineer"), "prompt_engineer");
        assert_eq!(normalise_role("PROMPT_ENGINEER"), "prompt_engineer");
        assert_eq!(normalise_role("  Memory-Writer "), "memory_writer");
    }

    #[test]
    fn parse_target_handles_known_prefixes() {
        let (kind, model) = parse_target("anthropic:claude-sonnet-4-6").unwrap();
        assert_eq!(kind, ProviderKind::Anthropic);
        assert_eq!(model, "claude-sonnet-4-6");

        let (kind, model) = parse_target("local:qwen-2.5-coder-7b").unwrap();
        assert_eq!(kind, ProviderKind::Ollama);
        assert_eq!(model, "qwen-2.5-coder-7b");
    }

    #[test]
    fn parse_target_rejects_empty_model() {
        let err = parse_target("anthropic:").unwrap_err();
        assert!(matches!(err, RoleDispatchError::MissingModel(_)));
    }

    #[test]
    fn parse_target_rejects_bare_alias_no_silent_fallback() {
        // stub_ban: no silent fallback to a single hardcoded provider —
        // bare model names without an explicit `<kind>:` prefix must
        // surface UnknownProvider rather than be classified by guess.
        let err = parse_target("sonnet").unwrap_err();
        assert!(matches!(err, RoleDispatchError::UnknownProvider(_)));
    }

    #[test]
    fn parse_target_rejects_unknown_prefix() {
        let err = parse_target("madeup:model-x").unwrap_err();
        assert!(matches!(err, RoleDispatchError::UnknownProvider(_)));
    }
}
