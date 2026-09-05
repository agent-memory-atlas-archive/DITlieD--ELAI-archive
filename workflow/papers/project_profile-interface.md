Source: elai_core::project_profile
Reliability: HIGH (in-repo official source, reviewed)
Files examined:
  F:\.ELAI_workflow\.elai_cc\crates\core\src\project_profile.rs (614 lines)
  F:\.ELAI_workflow\.elai_cc\crates\core\src\lib.rs (re-export confirmation, lines 47-54)
  F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\dkg\mod.rs L1062 (extract_conventions producer)
  F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\conventions.rs (consumer + ConventionStats producer)

Purpose:
  Project Intelligence Tier 1 (structural discovery) + Tier 2 placeholders
  (naming/error-handling/module-org/dep-profile) + Tier 3 placeholders
  (vision/architecture/domains) + build command registry. Single canonical
  ProjectProfile struct serialised to project-profile.toml.
  Spec: docs/specs/guide/project-intelligence-design.txt sections 1, 2A-2G, 5, 11.

Module-level invariants (from doc comments + code):
  - Schema version: PROFILE_SCHEMA_VERSION = 1.
  - Detection confidence floor: MIN_DETECTION_CONFIDENCE = 0.6 (below -> Unknown).
  - Default command timeout: DEFAULT_COMMAND_TIMEOUT_S = 120 seconds.
  - All Tier 2/3 fields use #[serde(default)] OR Default impls -> backward-compatible
    deserialisation from pre-4A/4B profiles.
  - The 12 ProjectDomain variants are the canonical spec-section-4B set; do NOT extend
    without updating the signal table.

Constants (public, exported via lib.rs):
  pub const PROFILE_SCHEMA_VERSION: u32 = 1;
  pub const DEFAULT_COMMAND_TIMEOUT_S: u32 = 120;
  pub const MIN_DETECTION_CONFIDENCE: f32 = 0.6;

Public type list (with field types):

  pub struct ProjectProfile {
    pub schema_version: u32,
    pub generated_at: String,
    pub project_root: PathBuf,
    // Tier 1
    pub build_system: BuildSystemProfile,
    pub languages: LanguageProfile,
    pub frameworks: Vec<FrameworkDetection>,
    pub test_profile: TestProfile,
    pub lint_tools: Vec<LintProfile>,
    pub entry_points: Vec<EntryPoint>,
    pub ci: Option<CiProfile>,
    // Tier 2 (A-5)
    pub naming: NamingConventions,
    pub error_handling: ErrorHandlingProfile,
    pub module_organization: ModuleOrganization,
    pub dep_profile: DependencyProfile,            // #[serde(default)]
    // Tier 3 (A-11)
    pub project_vision: Option<String>,
    pub architecture: Option<ArchitecturePattern>,
    pub domains: Vec<DomainClassification>,
    // Batch 4A
    pub fix_template_keys: Vec<String>,            // #[serde(default)] — canonical
                                                   // {language}.{kind} keys for fix templates
    // Batch 4B
    pub constraints: Vec<String>,                  // #[serde(default)] — constitutional
                                                   // constraints (anti-staleness rules)
    pub commands: BuildCommandRegistry,
    pub overrides_applied: bool,
  }
  derives: Debug, Clone, serde::Serialize, serde::Deserialize

  pub struct BuildSystemProfile {
    pub system: BuildSystem,
    pub config_path: PathBuf,
    pub version: Option<String>,
    pub monorepo: bool,
    pub workspaces: Vec<WorkspaceMember>,
  }
  derives: Debug, Clone, serde::Serialize, serde::Deserialize, Default (manual)

  pub enum BuildSystem {
    Cargo, Npm, Bun, Deno, PyProject, SetupPy, GoMod, CMake, Make,
    Gradle, Maven, Mix, Bundler, Composer, DotNet, Zig, Meson, Just, Task,
    Custom(String), Unknown,
  }
  derives: Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize

  pub struct WorkspaceMember {
    pub name: String,
    pub path: PathBuf,
  }
  derives: Debug, Clone, serde::Serialize, serde::Deserialize

  pub struct LanguageProfile {
    pub primary: Language,
    pub secondary: Vec<Language>,
    pub distribution: BTreeMap<Language, usize>,
    pub tree_sitter_coverage: f32,
  }
  derives: Debug, Clone, serde::Serialize, serde::Deserialize, Default (manual)

  pub enum Language {
    Rust, Python, TypeScript, JavaScript, Go, Java, Kotlin, CSharp, Cpp, C,
    Ruby, Elixir, PHP, Swift, Zig, Dart, Lua, Svelte, Vue, Haskell, Scala,
    Clojure, OCaml, Gleam,
    Other(String), Unknown,
  }
  derives: Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize
  method: pub fn id_str(&self) -> &str
          Maps each variant to lowercase id ("rust"|"python"|..."unknown");
          Other(s) returns s.as_str(). Used by convention extraction to keep
          per-language keys consistent.

  pub struct FrameworkDetection {
    pub name: String,
    pub version: Option<String>,
    pub category: FrameworkCategory,
    pub confidence: f32,        // 0.0..=1.0; below MIN_DETECTION_CONFIDENCE -> Unknown
    pub detected_via: String,   // detector name (e.g., "cargo-toml-dep")
  }

  pub enum FrameworkCategory {
    Web, Game, ML, Desktop, Mobile, Fullstack, Server, Testing, Other(String),
  }
  derives: Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize

  pub struct TestProfile {
    pub runner_cmd: Option<String>,
    pub location_pattern: TestLocationPattern,
    pub coverage_tool: Option<String>,
    pub fixture_dir: Option<PathBuf>,
  }
  derives: Debug, Clone, serde::Serialize, serde::Deserialize, Default (manual)

  pub enum TestLocationPattern {
    SameFile, Adjacent, SeparateDirectory, Mixed, Unknown,
  }
  derives: Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize

  pub struct LintProfile {
    pub tool: String,
    pub config_path: Option<PathBuf>,
    pub language: Language,
    pub auto_fix: bool,
  }

  pub struct EntryPoint {
    pub file: PathBuf,
    pub kind: EntryPointKind,
  }

  pub enum EntryPointKind {
    Binary, Library, Script, Server,
  }
  derives: Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize

  pub struct CiProfile {
    pub system: CiSystem,
    pub config_path: PathBuf,
    pub detected_steps: Vec<String>,
  }

  pub enum CiSystem {
    GitHubActions, GitLabCi, Jenkins, CircleCi, BitbucketPipelines, TravisCi,
    Other(String),
  }

  pub struct NamingConventions {
    pub functions: Convention,
    pub types: Convention,
    pub constants: Convention,
    pub modules: Convention,
  }
  derives: Debug, Clone, serde::Serialize, serde::Deserialize, Default (manual)
  Default::default() -> all four fields = Convention::Unknown.

  pub enum Convention {
    Strong(String),   // e.g. "snake_case"; >= 0.9 conformance per spec
    Moderate(String), // 0.6..0.9 conformance
    Mixed,            // < 0.6 conformance, multiple styles
    Unknown,          // insufficient signal
  }
  derives: Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize

  pub struct ErrorHandlingProfile {
    pub pattern: ErrorPattern,
    pub propagation: PropagationStyle,
    pub error_crate: Option<String>,  // e.g. "thiserror", "anyhow"
  }
  derives: Debug, Clone, serde::Serialize, serde::Deserialize, Default (manual)

  pub enum ErrorPattern {
    ResultBased, ExceptionBased, ErrorCodeBased, Mixed, Unknown,
  }

  pub enum PropagationStyle {
    QuestionMark, TryCatch, IfErrNil, Mixed, Unknown,
  }

  pub struct ModuleOrganization {
    pub pattern: OrgPattern,
  }
  derives: Debug, Clone, serde::Serialize, serde::Deserialize, Default (manual)

  pub enum OrgPattern {
    Flat, ByType, ByFeature, Layered, Monorepo, Unknown,
  }

  pub enum ClassificationDepth {
    DeclarationOnly,  // manifest-only (all languages); Default
    UsageBased,       // DKG import-edge analysis (Rust-only, Tier B)
  }
  derives: Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default
  Default::default() -> DeclarationOnly.

  pub enum PinningStrategy {
    Pinned, Ranged, Wild, Unknown,
  }
  derives: Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default
  Default::default() -> Unknown.

  pub struct DependencyProfile {
    pub total: usize,
    pub declared: Vec<String>,
    pub dev_only: Vec<String>,
    pub core: Vec<String>,           // Tier B Rust-only
    pub peripheral: Vec<String>,     // Tier B Rust-only
    pub unused: Vec<String>,         // Tier B Rust-only
    pub pinning_strategy: PinningStrategy,
    pub classification_depth: ClassificationDepth,
  }
  derives: Debug, Clone, serde::Serialize, serde::Deserialize, Default (manual)

  pub enum ArchitecturePattern {
    Layered, Hexagonal, Microservices, Monolith, EventDriven, Serverless, Unknown,
  }
  derives: Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize

  pub struct DomainClassification {
    pub domain: ProjectDomain,
    pub confidence: f32,           // 0.0..=1.0
    pub signals: Vec<String>,      // human-readable witnesses
  }

  pub enum ProjectDomain {
    WebBackend, WebFrontend, Fullstack, CliTool, Library, Game, MlPipeline,
    DataPipeline, Mobile, Desktop, Embedded, DevOps,
  }
  derives: Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize
  method: pub fn label(&self) -> &'static str
          12-variant labels = UPPER_SNAKE strings ("WEB_BACKEND"|...|"DEVOPS").
          Use for dashboard rendering and rule predicate matching.

  pub struct BuildCommandRegistry {
    pub build:  Vec<BuildCommand>,
    pub test:   Vec<BuildCommand>,
    pub lint:   Vec<BuildCommand>,
    pub format: Vec<BuildCommand>,
    pub run:    Vec<BuildCommand>,
    pub clean:  Vec<BuildCommand>,
  }
  derives: Debug, Clone, serde::Serialize, serde::Deserialize, Default (manual)

  pub struct BuildCommand {
    pub cmd: String,
    pub working_dir: PathBuf,
    pub env: BTreeMap<String, String>,
    pub timeout_s: u32,             // defaults to DEFAULT_COMMAND_TIMEOUT_S
    pub scope: CommandScope,
    pub verified: bool,             // true if we have an actual successful run
  }
  derives: Debug, Clone, serde::Serialize, serde::Deserialize

  pub enum CommandScope {
    Full, Crate(String), WorkspaceMember(String), Directory(PathBuf),
  }

  pub enum ProjectProfileError {
    Io(#[from] std::io::Error),
    TomlParse(#[from] toml::de::Error),
    TomlSerialize(#[from] toml::ser::Error),
    JsonParse(#[from] serde_json::Error),
    NoBuildSystem(PathBuf),
    RootNotFound(PathBuf),
  }
  derives: Debug, thiserror::Error
  Display impls auto-generated via thiserror.

Public functions/methods on these types:
  - Language::id_str(&self) -> &str
  - ProjectDomain::label(&self) -> &'static str
  - All other types are pure data containers (no inherent fns on this file).

Default impls present (manual, used for #[serde(default)] / pre-existing TOML round-trip):
  - BuildCommandRegistry::default()      -> all six Vec fields empty
  - NamingConventions::default()         -> all four Convention::Unknown
  - ErrorHandlingProfile::default()      -> ErrorPattern::Unknown / PropagationStyle::Unknown / None crate
  - ModuleOrganization::default()        -> OrgPattern::Unknown
  - TestProfile::default()               -> Nones + TestLocationPattern::Unknown
  - LanguageProfile::default()           -> Language::Unknown primary, empty BTreeMap, 0.0 coverage
  - BuildSystemProfile::default()        -> BuildSystem::Unknown, empty path, no monorepo
  - DependencyProfile::default()         -> zeroed (PinningStrategy::Unknown, ClassificationDepth::DeclarationOnly)
  - ClassificationDepth::default()       -> DeclarationOnly
  - PinningStrategy::default()           -> Unknown

Re-exports (canonical, from crates/core/src/lib.rs L47-54 — what consumers actually import):
  pub use project_profile::{
    ArchitecturePattern, BuildCommand, BuildCommandRegistry, BuildSystem, BuildSystemProfile,
    CiProfile, CiSystem, CommandScope, Convention, EntryPoint, EntryPointKind, ErrorHandlingProfile,
    ErrorPattern, FrameworkCategory, FrameworkDetection, Language, LanguageProfile, LintProfile,
    ModuleOrganization, NamingConventions, OrgPattern, ProjectProfile, ProjectProfileError,
    PropagationStyle, TestLocationPattern, TestProfile, WorkspaceMember,
    DEFAULT_COMMAND_TIMEOUT_S, MIN_DETECTION_CONFIDENCE, PROFILE_SCHEMA_VERSION,
  };
  NOT re-exported from elai_core::*: DependencyProfile, ClassificationDepth, PinningStrategy,
  DomainClassification, ProjectDomain, fix_template_keys, constraints.
  Consumers needing these must import via elai_core::project_profile::* (path is pub mod).

Cross-file dependencies (consumers reading NamingConventions / ProjectProfile / related):
  Producers / writers:
    .elai_cc\crates\knowledge\src\project_profile.rs   (top-level builder/serialiser)
    .elai_cc\crates\knowledge\src\conventions.rs       (extract_from_dkg -> ConventionStats { naming: NamingConventions, .. })
    .elai_cc\crates\knowledge\src\incremental_profile.rs
    .elai_cc\crates\knowledge\src\telescope.rs
    .elai_cc\crates\knowledge\src\tier3.rs
    .elai_cc\crates\knowledge\src\fix_templates.rs
    .elai_cc\crates\knowledge\src\dkg\mod.rs  (DKG::extract_conventions -> ConventionStats, L1062)
    .elai_cc\crates\rules\src\project_rule_seeder.rs
    .elai_cc\crates\rules\src\rule_engine.rs
  Consumers (read at audit/orchestrate time):
    .elai_cc\crates\orchestrator\src\deterministic_auditor.rs
    .elai_cc\crates\orchestrator\src\deterministic_audit_gate.rs
  Tests:
    .elai_cc\crates\contracts\tests\pi_tier3_completions_contract.rs
    .elai_cc\crates\contracts\tests\pi_incremental_contract.rs
    .elai_cc\crates\contracts\tests\pi_telescope_contract.rs
    .elai_cc\crates\contracts\tests\pi_tier2_a33_contract.rs
    .elai_cc\crates\contracts\tests\dashboard_pi_tier3_on_open_contract.rs
    .elai_cc\crates\contracts\tests\post_execution_rule_eval_contract.rs
    .elai_cc\crates\contracts\tests\deterministic_auditor_a41_contract.rs
    .elai_cc\crates\contracts\tests\deterministic_auditor_completions_contract.rs
    .elai_cc\crates\contracts\tests\task_runner_deterministic_auditor_contract.rs
    .elai_cc\crates\contracts\tests\rules_web_contract.rs

extract_conventions producer interface (relevant to phase-3L consumer matching):
  Location: elai_knowledge::dkg::DkgIndex::extract_conventions
  Signature: pub fn extract_conventions(
               &self,
               project_root: &Path,
             ) -> Result<crate::conventions::ConventionStats, KnowledgeError>;
  Behaviour:
    1. Checks dkg_readiness(); if not Ready -> Err(KnowledgeError::Graph("DKG not ready ...")).
    2. Queries self.nodes()? (Vec<DkgNode>).
    3. Returns Ok(crate::conventions::extract_from_dkg(&nodes, project_root)).
  Output type:
    pub struct ConventionStats {
      pub naming: NamingConventions,   // <-- re-export of elai_core::NamingConventions
      // ...other aggregate fields per conventions.rs
    }
  Consumer rule: any new PI extractor that produces "convention output for consumers" MUST
  produce NamingConventions exactly as defined in this file (4 Convention fields). Do NOT
  invent a parallel struct.

Platform constraints:
  - All types are serde + (de)serialisable via TOML and JSON; PathBuf round-trips assume
    UTF-8 paths (Windows backslashes preserved, but knowledge-crate code normalises slashes
    in many places — be careful in new code).
  - schema_version must be bumped if ANY non-#[serde(default)] field is added/removed.
  - Adding new variants to public enums (BuildSystem, Language, FrameworkCategory, CiSystem,
    ProjectDomain, ArchitecturePattern, ErrorPattern, etc.) is a semver-MINOR change;
    matching on these enums elsewhere must be exhaustive or use _ catch-all.

Gotchas:
  - DependencyProfile.core / .peripheral / .unused are RUST-ONLY (classification_depth must be
    UsageBased to be meaningful). For non-Rust projects they remain empty Vec — do NOT
    interpret emptiness as "no core deps".
  - Convention::Strong/Moderate carry a String payload — the convention NAME (e.g. "snake_case",
    "camelCase"). Consumers MUST handle the payload, not just the variant.
  - ProjectProfile.fix_template_keys uses {language}.{kind} keys ("rust.unwrap", etc.) — full
    list lives in .elai_cc/data/fix_templates/; do NOT hardcode.
  - ProjectProfile.constraints (4B) is the list the auditor re-reads each baby step. New PI
    work that wants to inject constraints MUST go through that field, NOT a parallel channel.
  - ProjectDomain::label() returns UPPER_SNAKE strings — useful for rule predicates and
    dashboard rendering; do NOT use Debug format for stable identifiers.
  - MIN_DETECTION_CONFIDENCE = 0.6 is the floor — FrameworkDetection / DomainClassification
    with lower confidence should NOT be emitted (caller responsibility, not enforced here).
  - tree_sitter_coverage is f32 in [0.0, 1.0] but is NOT clamped at this layer.

Critical assertions (testable, Rust syntax):
  - assert!(PROFILE_SCHEMA_VERSION == 1);
  - assert!(DEFAULT_COMMAND_TIMEOUT_S == 120);
  - assert!((MIN_DETECTION_CONFIDENCE - 0.6).abs() < f32::EPSILON);
  - assert!(matches!(NamingConventions::default().functions, Convention::Unknown));
  - assert!(matches!(NamingConventions::default().types, Convention::Unknown));
  - assert!(matches!(NamingConventions::default().constants, Convention::Unknown));
  - assert!(matches!(NamingConventions::default().modules, Convention::Unknown));
  - assert!(matches!(DependencyProfile::default().classification_depth, ClassificationDepth::DeclarationOnly));
  - assert!(matches!(DependencyProfile::default().pinning_strategy, PinningStrategy::Unknown));
  - assert!(Language::Rust.id_str() == "rust");
  - assert!(Language::TypeScript.id_str() == "typescript");
  - assert!(Language::Other("nim".to_string()).id_str() == "nim");
  - assert!(Language::Unknown.id_str() == "unknown");
  - assert!(ProjectDomain::WebBackend.label() == "WEB_BACKEND");
  - assert!(ProjectDomain::DevOps.label() == "DEVOPS");
  - // round-trip via TOML
  - let p = ProjectProfile { /* defaults */ };
    let s = toml::to_string(&p)?;
    let r: ProjectProfile = toml::from_str(&s)?;
    assert!(r.schema_version == p.schema_version);

PI design hooks (relevant to phase-3L):
  - ProjectIntelligenceHook + CorpusView + Pi-prefixed types: place in the KNOWLEDGE crate
    (elai_knowledge), NOT this file. ProjectProfile owns the serialised snapshot;
    extractor traits belong in knowledge/. This avoids cyclic deps and keeps elai_core
    pure-data.
  - NamingConventions is the contract surface — any new convention extractor MUST emit
    this exact 4-field struct so consumers (NamingConventions match arms in auditor,
    dashboard renderers) need zero changes.
  - DO NOT introduce a parallel "PiNamingConventions" type. Reuse NamingConventions.
  - No name collisions today: ProjectIntelligenceHook / CorpusView / PiT1Hook /
    PiT2Hook / PiT3Hook do NOT appear anywhere in this file or the re-export list — safe
    to introduce them in elai_knowledge.
  - Tier 3 fields (project_vision, architecture, domains) are already Optional — extend
    via the existing fields, do NOT add a Tier 4 vector without spec sign-off.
  - schema_version bump: if Pi work adds non-#[serde(default)] fields to ProjectProfile,
    increment PROFILE_SCHEMA_VERSION and document migration.
