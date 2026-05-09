//! Unified runtime configuration loader for elai_cc.
//!
//! Reads `elai_cc.toml` from the project root and deserializes it into
//! [`ElaiRuntimeConfig`]. All sections are optional — missing sections fall
//! back to their respective `Default` implementations so the application
//! works without the file being present.
//!
//! # Wiring status (A-26 / A-27 / A-30 / A-31)
//!
//! | Section              | Status | Notes                                              |
//! |----------------------|--------|----------------------------------------------------|
//! | [output]             | WIRED  | Passed to ConversationRuntime::with_output_config  |
//! | [actions]            | WIRED  | ActionPolicy stored on GlobalToolRegistry;         |
//! |                      |        | passed to execute_tool_with_enforcer() as param    |
//! | [tools.routing_rules]| WIRED  | init_routing_config() called at startup;           |
//! |                      |        | check_routing() reads OnceLock — enabled +         |
//! |                      |        | allow_override_prefix respected at runtime          |
//! | [tools.safety_filter]| WIRED  | init_safety_config() called at startup;            |
//! |                      |        | check_bash_safety() short-circuits when disabled   |
//! | [inference]          | PARSED | A-39: safety defaults deserialized; sidecar launch  |
//! |                      |        | wiring pending (BLOCK-3, future phase)             |

use std::path::Path;

use elai_orchestrator::OutputConfig;
use elai_tools::action_classifier::ActionPolicy;
use elai_tools::routing_rules::RoutingConfig;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// SafetyFilterConfig
// ---------------------------------------------------------------------------

/// Runtime toggle for the safety filter (A-27).
///
/// Currently the safety filter has no config struct of its own; this type
/// provides one so it can be loaded from `[tools.safety_filter]` and
/// passed to a future `SafetyFilter::with_config()` API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SafetyFilterConfig {
    /// When `false`, all Bash commands skip safety checking. Default: `true`.
    pub enabled: bool,
}

impl Default for SafetyFilterConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

// ---------------------------------------------------------------------------
// ToolsConfig
// ---------------------------------------------------------------------------

/// Aggregated config for the [tools] TOML section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ToolsConfig {
    /// `[tools.routing_rules]` — runtime config for the A-26 routing layer.
    pub routing_rules: RoutingConfig,
    /// `[tools.safety_filter]` — runtime toggle for the A-27 safety filter.
    pub safety_filter: SafetyFilterConfig,
}

// ---------------------------------------------------------------------------
// InferenceConfig (A-39: during-generation safety defaults)
// ---------------------------------------------------------------------------

/// Runtime configuration for during-generation features.
///
/// All features ship DISABLED by default — calibration/testing must confirm
/// they help before enabling. See anti-staleness-enforcement.txt sections 2.1/2.2.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InferenceConfig {
    /// Steering vector (ActAdd) toggle. Measured NEUTRAL on Qwen2.5-Coder-7B.
    pub steering_enabled: bool,
    /// DoLa + ETC incompatibility guard. When true, DoLa auto-disables on <32B models.
    pub dola_on_32b_only: bool,
    /// Model that calibration results are valid for.
    pub calibration_target_model: String,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            steering_enabled: false,
            dola_on_32b_only: true,
            calibration_target_model: String::from("Qwen2.5-Coder-7B"),
        }
    }
}

// ---------------------------------------------------------------------------
// AuditorConfig (A-41: deterministic-first auditor toggle)
// ---------------------------------------------------------------------------

/// Serde helper: returns `true` for `deterministic_first` default.
fn default_true() -> bool {
    true
}

/// Configuration for the auditor agent behavior.
///
/// Controls whether deterministic checks (build, version gate, file size,
/// lazy code, completeness) run before the LLM auditor.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AuditorConfig {
    /// When `true`, run all 5 deterministic checks before the LLM auditor.
    /// When `false`, skip deterministic checks and go straight to LLM audit.
    /// Default: `true`.
    #[serde(default = "default_true")]
    pub deterministic_first: bool,
}

impl Default for AuditorConfig {
    fn default() -> Self {
        Self {
            deterministic_first: true,
        }
    }
}

// ---------------------------------------------------------------------------
// AgentsConfig (A-41: agent-level configuration)
// ---------------------------------------------------------------------------

/// Aggregated config for the `[agents]` TOML section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentsConfig {
    /// `[agents.auditor]` — deterministic-first auditor toggle.
    pub auditor: AuditorConfig,
}

// ---------------------------------------------------------------------------
// ElaiRuntimeConfig
// ---------------------------------------------------------------------------

/// Unified runtime configuration read from `elai_cc.toml`.
///
/// The struct mirrors the top-level TOML sections. All fields implement
/// `Default` so a missing file or missing sections degrade gracefully.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ElaiRuntimeConfig {
    /// `[tools.*]` section.
    pub tools: ToolsConfig,
    /// `[output]` section — passed to `ConversationRuntime::with_output_config`.
    pub output: OutputConfig,
    /// `[actions]` section — action classifier policy.
    pub actions: ActionPolicy,
    /// `[inference]` section — during-generation feature safety defaults (A-39).
    pub inference: InferenceConfig,
    /// `[agents]` section — agent-level configuration (A-41).
    pub agents: AgentsConfig,
}

// ---------------------------------------------------------------------------
// Loader
// ---------------------------------------------------------------------------

/// Load `elai_cc.toml` from `project_root`.
///
/// Returns `ElaiRuntimeConfig::default()` if the file is absent or
/// unparseable (with a tracing warning on parse failure).
pub fn load_elai_config(project_root: &Path) -> ElaiRuntimeConfig {
    let config_path = project_root.join("elai_cc.toml");
    match std::fs::read_to_string(&config_path) {
        Ok(content) => toml::from_str::<ElaiRuntimeConfig>(&content).unwrap_or_else(|e| {
            tracing::warn!(
                path = %config_path.display(),
                error = %e,
                "failed to parse elai_cc.toml — using defaults"
            );
            ElaiRuntimeConfig::default()
        }),
        Err(_) => {
            tracing::debug!(
                path = %config_path.display(),
                "elai_cc.toml not found — using defaults"
            );
            ElaiRuntimeConfig::default()
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Create a temp directory with a unique name and write `content` as `elai_cc.toml`.
    /// Returns the temp dir path (caller owns the `TempDir`-equivalent via drop of the
    /// returned `std::path::PathBuf`; we keep the dir alive via a guard file).
    fn with_toml_file(content: &str, test_fn: impl FnOnce(&Path)) {
        let mut dir = std::env::temp_dir();
        // Use a per-test unique subdirectory to avoid races.
        let id = std::time::SystemTime::now() // TEST_KEEP: test-only temp-dir uniquifier
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        dir.push(format!("elai_runtime_config_test_{id}"));
        std::fs::create_dir_all(&dir).expect("create test dir");
        let path = dir.join("elai_cc.toml");
        let mut f = std::fs::File::create(&path).expect("create toml");
        f.write_all(content.as_bytes()).expect("write toml");
        test_fn(&dir);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_file_returns_defaults() {
        let mut dir = std::env::temp_dir();
        dir.push("elai_runtime_config_no_file_test");
        std::fs::create_dir_all(&dir).expect("create test dir");
        // No elai_cc.toml written — should get defaults.
        let cfg = load_elai_config(&dir);
        let _ = std::fs::remove_dir_all(&dir);
        assert!(cfg.output.strip_emoji);
        assert!(cfg.output.strip_sycophancy);
        assert_eq!(cfg.output.max_response_tokens, 0);
        assert!(cfg.tools.routing_rules.enabled);
        assert!(cfg.tools.safety_filter.enabled);
    }

    #[test]
    fn output_section_parsed() {
        with_toml_file(
            r#"
[output]
strip_emoji = false
max_response_tokens = 512
"#,
            |root| {
                let cfg = load_elai_config(root);
                assert!(!cfg.output.strip_emoji);
                assert!(cfg.output.strip_sycophancy); // still default
                assert_eq!(cfg.output.max_response_tokens, 512);
            },
        );
    }

    #[test]
    fn routing_rules_section_parsed() {
        with_toml_file(
            "[tools.routing_rules]\nenabled = false\nallow_override_prefix = \"#\"\n",
            |root| {
                let cfg = load_elai_config(root);
                assert!(!cfg.tools.routing_rules.enabled);
                assert_eq!(cfg.tools.routing_rules.allow_override_prefix, "#");
            },
        );
    }

    #[test]
    fn actions_section_parsed() {
        with_toml_file(
            "[actions]\nmode = \"Autonomous\"\n",
            |root| {
                let cfg = load_elai_config(root);
                assert_eq!(
                    cfg.actions.mode,
                    elai_tools::action_classifier::ExecutionMode::Autonomous
                );
            },
        );
    }

    #[test]
    fn malformed_toml_returns_defaults() {
        with_toml_file("this is not valid [ toml @@@@", |root| {
            let cfg = load_elai_config(root);
            // Should not panic; returns defaults silently.
            assert!(cfg.output.strip_emoji);
        });
    }

    #[test]
    fn empty_file_returns_defaults() {
        with_toml_file("", |root| {
            let cfg = load_elai_config(root);
            assert!(cfg.output.strip_emoji);
            assert_eq!(cfg.output.max_response_tokens, 0);
        });
    }

    #[test]
    fn inference_section_parsed() {
        with_toml_file(
            r#"
[inference]
steering_enabled = true
dola_on_32b_only = false
calibration_target_model = "Qwen2.5-Coder-14B"
"#,
            |root| {
                let cfg = load_elai_config(root);
                assert!(cfg.inference.steering_enabled);
                assert!(!cfg.inference.dola_on_32b_only);
                assert_eq!(cfg.inference.calibration_target_model, "Qwen2.5-Coder-14B");
            },
        );
    }

    #[test]
    fn inference_defaults_when_absent() {
        with_toml_file("[output]\nstrip_emoji = false\n", |root| {
            let cfg = load_elai_config(root);
            // Inference should have safe defaults even when section is absent.
            assert!(!cfg.inference.steering_enabled);
            assert!(cfg.inference.dola_on_32b_only);
            assert_eq!(cfg.inference.calibration_target_model, "Qwen2.5-Coder-7B");
        });
    }

    #[test]
    fn auditor_config_defaults_deterministic_first_true() {
        // When [agents.auditor] is absent, deterministic_first defaults to true.
        with_toml_file("[output]\nstrip_emoji = false\n", |root| {
            let cfg = load_elai_config(root);
            assert!(
                cfg.agents.auditor.deterministic_first,
                "deterministic_first must default to true when [agents.auditor] is absent"
            );
        });
    }

    #[test]
    fn auditor_config_explicit_false() {
        with_toml_file(
            "[agents.auditor]\ndeterministic_first = false\n",
            |root| {
                let cfg = load_elai_config(root);
                assert!(
                    !cfg.agents.auditor.deterministic_first,
                    "deterministic_first = false must be respected"
                );
            },
        );
    }
}
