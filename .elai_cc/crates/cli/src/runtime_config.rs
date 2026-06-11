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
use elai_orchestrator::ResearchPresets;
// C-28: re-use the orchestrator's MutationConfig (defined in
// `mutation_testing.rs`) so the [testing.mutation] elai_cc.toml block
// stays a single struct shared between the loader + the auditor's
// Step 6 gate.
use elai_orchestrator::MutationConfig;
// C-29: re-use the orchestrator's IntentConfig + MetamorphicConfig so the
// `[testing.intent]` + `[testing.metamorphic]` elai_cc.toml blocks stay
// single structs shared between the loader, the PLANNING -> TESTING_FIRST
// gate, and the auditor Step 7 hook.
use elai_orchestrator::{IntentConfig, MetamorphicConfig};
// E-46 D-8: planning-decomposition substrate config; defined in
// orchestrator::runtime_config and re-exported from orchestrator::lib.
use elai_orchestrator::PlanningConfig;
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
// C-28: TestingConfig — [testing.mutation] block in elai_cc.toml.
// ---------------------------------------------------------------------------

/// Aggregated config for the `[testing]` TOML section.
/// `mutation` is the only sub-block today; future tiers (T-2/T-3 in C-29)
/// land alongside.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TestingConfig {
    /// `[testing.mutation]` — T-1 mutation testing gate config.
    /// Absent section falls through to `MutationConfig::default()`.
    pub mutation: MutationConfig,
    /// `[testing.intent]` — C-29 T-2 IntentPair gate config (PLANNING ->
    /// TESTING_FIRST). Absent section falls through to
    /// `IntentConfig::default()` (`min_pairs = 3`).
    pub intent: IntentConfig,
    /// `[testing.metamorphic]` — C-29 T-3 metamorphic relations config
    /// consumed by `DeterministicAuditor` Step 7. Absent section falls
    /// through to `MetamorphicConfig::default()` (`enabled = true`,
    /// `relations = ["identity","inverse"]`, `sample_size = 100`).
    pub metamorphic: MetamorphicConfig,
}

// ---------------------------------------------------------------------------
// D-4 GAP-3: TripwireConfigToml — [tripwire] block in elai_cc.toml (L3-locked).
// ---------------------------------------------------------------------------

/// `[tripwire]` TOML section (D-4 GAP-3). Mirrors the tunable thresholds +
/// window sizes of [`elai_orchestrator::tripwire::TripwireConfig`] so the
/// L3 lock policy can enforce "values flow from this section into the
/// monitor; agent edits to the in-code `defaults()` never override the
/// configured values".
///
/// Field names mirror the TOML keys exactly (snake_case, no aliasing) so
/// the lock list in `scripts/locked_keys_registry.toml` can target
/// `[tripwire].<key>` literals without serde-rename indirection. All
/// fields default to the same paper-cited values as
/// `TripwireConfig::defaults()` so a missing section reproduces the
/// in-code defaults bit-for-bit.
///
/// Paper grounding (see also workflow/reasoning/D-4-decisions.md):
/// * `kl_spec_tau`, `iqr_multiplier`, `ppl_sigma_mult` — arxiv:2507.05619
/// * `corr_window`, `ddm_warning_threshold`, `ddm_drift_threshold`,
///   `ddm_min_instances` — Lu et al survey arxiv:2004.05785 §3.2 (Gama
///   2004 derivation).
/// * `delta_rho` — implementation choice not directly grounded in
///   arxiv:2210.10760; ELAI baseline calibration (DECISION-4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TripwireConfigToml {
    pub kl_spec_tau: f32,
    pub delta_rho: f32,
    pub corr_window: u32,
    pub iqr_multiplier: f32,
    pub ppl_sigma_mult: f32,
    pub ddm_min_instances: u32,
    pub ddm_warning_threshold: f32,
    pub ddm_drift_threshold: f32,
}

impl Default for TripwireConfigToml {
    fn default() -> Self {
        Self {
            kl_spec_tau: 0.3,
            delta_rho: 0.5,
            corr_window: 50,
            iqr_multiplier: 3.0,
            ppl_sigma_mult: 2.0,
            ddm_min_instances: 30,
            ddm_warning_threshold: 2.0,
            ddm_drift_threshold: 3.0,
        }
    }
}

impl TripwireConfigToml {
    /// Convert the TOML mirror into the orchestrator's runtime
    /// [`elai_orchestrator::tripwire::TripwireConfig`]. Field mapping is
    /// 1:1 except for naming-drift between the wire keys
    /// (`delta_rho` / `iqr_multiplier` / `ppl_sigma_mult` /
    /// `corr_window` / `ddm_min_instances`) and the in-code names
    /// (`rho_delta_threshold` / `iqr_outlier_multiplier` /
    /// `ppl_anomaly_sigma` / `window_size` / `min_sample_count`); the
    /// in-code names predate the TOML keys and the rename is intentional
    /// (the TOML keys are the user-facing surface).
    #[must_use]
    pub fn to_tripwire_config(&self) -> elai_orchestrator::tripwire::TripwireConfig {
        elai_orchestrator::tripwire::TripwireConfig {
            kl_spec_tau: self.kl_spec_tau,
            rho_delta_threshold: self.delta_rho,
            iqr_outlier_multiplier: self.iqr_multiplier,
            ppl_anomaly_sigma: self.ppl_sigma_mult,
            ddm_warning_threshold: self.ddm_warning_threshold,
            ddm_drift_threshold: self.ddm_drift_threshold,
            min_sample_count: self.ddm_min_instances,
            window_size: self.corr_window,
        }
    }
}

// ---------------------------------------------------------------------------
// C-34: EmpireConfig — [empire] block in elai_cc.toml.
// ---------------------------------------------------------------------------

/// `[empire]` TOML section (C-34). Host-side config carrier for the
/// Phase 4J Empire integration. All fields are optional; missing keys fall
/// back to the defaults below. The block is parsed unconditionally (no
/// `#[cfg(feature = "empire")]` on the field) so non-empire builds still
/// read + ignore the section without a parse warning — when the user
/// provides a section with the orchestrator NOT built with `--features
/// empire`, `validate_empire_section_when_enabled` emits a tracing::warn
/// once at startup and the bridge stays inert.
///
/// Phase 4J shipped the EmpireBridge state machine + the 5 SsbActionType
/// empire arms but NOT the construction site that hands a real
/// `Arc<EmpireBridge>` to AppState (the activation patch). This struct is
/// the typed carrier the activation patch will consume.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EmpireConfig {
    /// Master enable switch. When `false` (default) the empire feature path
    /// stays fully inert even when the `empire` cargo feature is compiled
    /// in. When `true` + the cargo feature is OFF the loader emits a
    /// `tracing::warn!` and continues (warn-not-panic).
    pub enabled: bool,
    /// Transport selector: `"unix"` (AF_UNIX, Linux/macOS) or `"tcp"`
    /// (cross-platform fallback). Unknown values fall back to `"unix"` with
    /// a tracing::warn.
    pub transport: String,
    /// AF_UNIX socket path. Default uses `$XDG_RUNTIME_DIR` when set,
    /// otherwise `/tmp/elai-empire.sock`. Ignored when `transport = "tcp"`.
    pub socket_path: String,
    /// TCP host. Ignored when `transport = "unix"`.
    pub tcp_host: String,
    /// TCP port. Ignored when `transport = "unix"`. `7430` is an IANA
    /// unassigned unprivileged port (default).
    pub tcp_port: u16,
    /// Default bridge state at construction. `"Disconnected"` is the only
    /// value the host-side scaffold currently honours; other values fall
    /// back to it with a tracing::warn.
    pub bridge_state_default: String,
}

impl Default for EmpireConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            transport: String::from("unix"),
            socket_path: String::from("$XDG_RUNTIME_DIR/elai-empire.sock"),
            tcp_host: String::from("127.0.0.1"),
            tcp_port: 7430,
            bridge_state_default: String::from("Disconnected"),
        }
    }
}

/// Warn-not-panic validation for the `[empire]` block. Called once from
/// `load_elai_config` after deserialization succeeds.
///
/// Behaviour:
///   - When `enabled = false`: silent (no-op). Block presence is purely
///     informational.
///   - When `enabled = true` AND the `empire` cargo feature is OFF on the
///     orchestrator crate (detected at runtime via a `cfg!` probe), emits
///     a `tracing::warn!` describing the mismatch. No panic, no abort.
///   - Validates `transport` is one of `{"unix", "tcp"}`; unknown values
///     warn and the loader leaves the field as-is for the activation patch
///     to fall back from.
///   - Validates `bridge_state_default` is `"Disconnected"`; other values
///     warn (reserved for future replay-from-snapshot wiring).
///   - `tcp_port == 0` is allowed (sentinel) but warns when transport is
///     tcp.
///
/// The runtime detection that the Empire codebase itself is reachable is
/// performed by `EmpireBridge::connect()` returning `NotConnected`; this
/// validator only catches static-config mismatches the user can fix in
/// elai_cc.toml without restarting.
pub fn validate_empire_section_when_enabled(cfg: &EmpireConfig) {
    if !cfg.enabled {
        return;
    }
    // Cargo feature probe — `elai-orchestrator` is the crate that defines
    // the `empire` feature. Detect via a transitive cfg-check: when the
    // orchestrator was built with `--features empire`, its `empire` module
    // is exported and the dashboard's state.rs field is a real
    // `Option<Arc<EmpireBridge>>`. We cannot reach into that crate from
    // here without taking a hard dep, so we use a build-time conditional
    // compile probe forwarded via the `elai-orchestrator/empire` feature
    // edge in this crate's Cargo.toml. Until the loader is wired with that
    // feature edge the probe defaults to "off" and the warn fires.
    let feature_compiled_in = cfg!(feature = "empire");
    if !feature_compiled_in {
        tracing::warn!(
            transport = %cfg.transport,
            "elai_cc.toml [empire] enabled=true but the orchestrator was not built with --features empire; bridge stays inert"
        );
    }
    match cfg.transport.as_str() {
        "unix" | "tcp" => {}
        other => {
            tracing::warn!(
                transport = other,
                "elai_cc.toml [empire].transport is not one of {{unix,tcp}}; activation patch will fall back to default"
            );
        }
    }
    if cfg.bridge_state_default != "Disconnected" {
        tracing::warn!(
            bridge_state_default = %cfg.bridge_state_default,
            "elai_cc.toml [empire].bridge_state_default is not 'Disconnected'; reserved values not honoured by current scaffold, falling back"
        );
    }
    if cfg.transport == "tcp" && cfg.tcp_port == 0 {
        tracing::warn!(
            "elai_cc.toml [empire].tcp_port=0 with transport=tcp; activation patch will reject and fall back to Disconnected"
        );
    }
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
    /// `[cloud]` section — B-27 cloud-mode routing + spend caps + privacy
    /// profile + air-gapped flag. Consumed by `cloud_mode::CloudModeRouter`
    /// via [`CloudConfigToml::to_cloud_config`].
    pub cloud: CloudConfigToml,
    /// `[research_presets]` section (B-40) — per-ProjectClass research stream
    /// presets consumed by `idea_framing::frame_idea`. Loaded via the same
    /// `load_elai_config` pass; accessible via [`ElaiRuntimeConfig::research_presets`].
    pub research_presets: ResearchPresets,
    /// `[testing]` section (C-28) — testing-trust tier configuration.
    /// Currently carries the T-1 mutation block under `testing.mutation`.
    pub testing: TestingConfig,
    /// `[empire]` section (C-34) — host-side config carrier for the
    /// Phase 4J Empire integration. See [`EmpireConfig`].
    pub empire: EmpireConfig,
    /// `[tripwire]` section (D-4 GAP-3) — L3-locked tripwire stack
    /// thresholds + windows. See [`TripwireConfigToml`]. Consumed via
    /// [`ElaiRuntimeConfig::tripwire_config`].
    pub tripwire: TripwireConfigToml,
    /// `[planning.*]` sections (E-46 D-8) — per-operator overrides for the
    /// planning-decomposition substrate constants. Absent stanzas fall back
    /// to the in-module defaults via `PlanningConfig::default()`. Passed to
    /// `TaskRunnerConfig::planning_config` at `run_subcommand` boot time so
    /// the orchestrator reads operator-supplied thresholds instead of
    /// compile-time constants.
    pub planning: PlanningConfig,
}

impl ElaiRuntimeConfig {
    /// B-40 BLOCK-1 audit fix: accessor for the `[research_presets]` block.
    /// Returns a reference to the parsed `ResearchPresets` so callers can
    /// pass `&presets` to `idea_framing::frame_idea` without cloning.
    #[must_use]
    pub fn research_presets(&self) -> &ResearchPresets {
        &self.research_presets
    }

    /// C-28: accessor for the `[testing.mutation]` block. Dashboards
    /// thread this into `DefaultDeterministicAuditGate::with_mutation_supplier`.
    #[must_use]
    pub fn mutation_config(&self) -> &MutationConfig {
        &self.testing.mutation
    }

    /// C-29: accessor for the `[testing.intent]` block. Threaded into
    /// `TransitionContext.min_intent_pairs` at the PLANNING -> TESTING_FIRST
    /// gate evaluation site.
    #[must_use]
    pub fn intent_config(&self) -> &IntentConfig {
        &self.testing.intent
    }

    /// C-29: accessor for the `[testing.metamorphic]` block. Threaded into
    /// `DeterministicAuditorConfig.metamorphic` so Step 7 sees the same
    /// relation list as the runtime loader.
    #[must_use]
    pub fn metamorphic_config(&self) -> &MetamorphicConfig {
        &self.testing.metamorphic
    }

    /// C-34: accessor for the `[empire]` block. The activation patch (the
    /// one that constructs `Arc<EmpireBridge>` and chains it onto AppState
    /// via `with_empire_bridge`) reads this through `&cfg.empire_config()`.
    #[must_use]
    pub fn empire_config(&self) -> &EmpireConfig {
        &self.empire
    }

    /// D-4 GAP-3: accessor for the `[tripwire]` block. The wiring layer
    /// that constructs a `TripwireMonitor` (currently invoked from
    /// `agent_pool::observe_post_consensus_tripwires` test fixtures; the
    /// production path will pull `cfg.tripwire_config().to_tripwire_config()`
    /// once the monitor is hung off AppState) reads this here.
    #[must_use]
    pub fn tripwire_config(&self) -> &TripwireConfigToml {
        &self.tripwire
    }
}

/// `[cloud]` TOML section. Serializable mirror of
/// [`elai_orchestrator::cloud_mode::CloudConfig`] — TOML cares about plain
/// types, so we keep the redaction profile + mode as strings here and
/// convert on the way out.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CloudConfigToml {
    pub default_provider: String,
    pub default_model: String,
    pub default_mode: String,
    pub air_gapped: bool,
    pub privacy: CloudPrivacyToml,
    pub spend: CloudSpendToml,
    /// C-5: BYOE Anthropic base-URL override. Maps onto
    /// [`elai_orchestrator::cloud_mode::CloudConfig::custom_endpoint`].
    /// None / unset / empty => DEFAULT_BASE_URL.
    #[serde(default)]
    pub custom_endpoint: Option<String>,
}

impl Default for CloudConfigToml {
    fn default() -> Self {
        Self {
            default_provider: "anthropic".to_string(),
            default_model: "claude-sonnet-4-6".to_string(),
            default_mode: "auto".to_string(),
            air_gapped: false,
            privacy: CloudPrivacyToml::default(),
            spend: CloudSpendToml::default(),
            custom_endpoint: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CloudPrivacyToml {
    pub profile: String,
}

impl Default for CloudPrivacyToml {
    fn default() -> Self {
        Self {
            profile: "standard".to_string(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CloudSpendToml {
    pub warn_at_usd: Option<f64>,
    pub hard_cap_daily_usd: Option<f64>,
    pub hard_cap_monthly_usd: Option<f64>,
    /// C-5: percent-of-cap WARN threshold. Defaults to `Some(80)` in
    /// [`elai_orchestrator::cloud_mode::CloudConfig::default`] when this
    /// field is unset in the toml.
    pub warn_at_percent: Option<u8>,
}

impl CloudConfigToml {
    /// Convert the TOML form into the orchestrator's typed
    /// [`elai_orchestrator::cloud_mode::CloudConfig`]. Unknown
    /// profile/mode values fall back to safe defaults via the
    /// `from_config_str` parsers (Standard / Auto).
    #[must_use]
    pub fn to_cloud_config(&self) -> elai_orchestrator::cloud_mode::CloudConfig {
        use elai_orchestrator::cloud_mode::{CloudConfig, ExecMode};
        use elai_orchestrator::privacy_redactor::RedactionProfile;
        // Lift the WARN percent: explicit toml value wins, otherwise the
        // CloudConfig::default() seed (Some(80) per spec
        // cloud-mode-design.txt:75) — never None when the cap is set so
        // existing deployments gain WARN rows without a toml edit.
        let warn_at_percent = self
            .spend
            .warn_at_percent
            .or(CloudConfig::default().warn_at_percent);
        CloudConfig {
            default_provider: self.default_provider.clone(),
            default_model: self.default_model.clone(),
            default_mode: ExecMode::from_config_str(&self.default_mode),
            air_gapped: self.air_gapped,
            privacy_profile: RedactionProfile::from_config_str(&self.privacy.profile),
            spend_cap_daily_usd: self.spend.hard_cap_daily_usd,
            spend_cap_monthly_usd: self.spend.hard_cap_monthly_usd,
            warn_at_percent,
            custom_endpoint: self
                .custom_endpoint
                .as_ref()
                .and_then(|s| {
                    if s.trim().is_empty() {
                        None
                    } else {
                        Some(s.clone())
                    }
                }),
        }
    }
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
    let cfg = match std::fs::read_to_string(&config_path) {
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
    };
    // C-34: warn-not-panic validation for the [empire] block. The bridge
    // construction site (activation patch) is the consumer; this validator
    // surfaces obvious misconfigurations (enabled=true with empire feature
    // OFF, unknown transport / state) so the user can correct them without
    // restarting the orchestrator after the activation patch lands.
    validate_empire_section_when_enabled(&cfg.empire);
    cfg
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

    // C-34: [empire] block parses with defaults when absent + validator is
    // silent. The block IS present in repo-shipped elai_cc.toml but the
    // loader must also work when the block is missing entirely.
    #[test]
    fn empire_section_defaults_when_absent() {
        with_toml_file("[output]\nstrip_emoji = true\n", |root| {
            let cfg = load_elai_config(root);
            // EmpireConfig::default() values per spec.
            assert!(!cfg.empire.enabled);
            assert_eq!(cfg.empire.transport, "unix");
            assert_eq!(cfg.empire.tcp_host, "127.0.0.1");
            assert_eq!(cfg.empire.tcp_port, 7430);
            assert_eq!(cfg.empire.bridge_state_default, "Disconnected");
            assert_eq!(cfg.empire_config(), &cfg.empire);
        });
    }

    #[test]
    fn empire_section_parses_explicit_values() {
        with_toml_file(
            r#"
[empire]
enabled = false
transport = "tcp"
tcp_host = "10.0.0.5"
tcp_port = 9100
socket_path = "/tmp/elai-empire-test.sock"
bridge_state_default = "Disconnected"
"#,
            |root| {
                let cfg = load_elai_config(root);
                assert!(!cfg.empire.enabled);
                assert_eq!(cfg.empire.transport, "tcp");
                assert_eq!(cfg.empire.tcp_host, "10.0.0.5");
                assert_eq!(cfg.empire.tcp_port, 9100);
                assert_eq!(cfg.empire.socket_path, "/tmp/elai-empire-test.sock");
            },
        );
    }

    #[test]
    fn empire_validator_is_silent_when_disabled() {
        // enabled=false MUST be a no-op (the validator early-returns before
        // any tracing call). We assert by constructing the config directly
        // and ensuring the call does not panic.
        let cfg = EmpireConfig {
            enabled: false,
            transport: "totally-wrong-value".to_string(),
            bridge_state_default: "Connected".to_string(),
            ..Default::default()
        };
        // No assertion on stderr; the contract is "no panic + early-return"
        // and the function returns ()..
        validate_empire_section_when_enabled(&cfg);
    }

    #[test]
    fn empire_validator_does_not_panic_on_unknown_transport() {
        let cfg = EmpireConfig {
            enabled: true,
            transport: "rfc-9999".to_string(),
            bridge_state_default: "Disconnected".to_string(),
            ..Default::default()
        };
        // Validator should warn (not panic) on unknown transport.
        validate_empire_section_when_enabled(&cfg);
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
