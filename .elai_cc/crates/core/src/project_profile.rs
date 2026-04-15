// Project Intelligence: Tier 1 structural discovery types.
// Spec: docs/specs/guide/project-intelligence-design.txt sections 1, 2A-2G, 5, 11.

use std::collections::HashMap;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Current schema version for project-profile.toml serialization.
pub const PROFILE_SCHEMA_VERSION: u32 = 1;

/// Default command timeout in seconds.
pub const DEFAULT_COMMAND_TIMEOUT_S: u32 = 120;

/// Minimum confidence threshold below which detection returns Unknown.
pub const MIN_DETECTION_CONFIDENCE: f32 = 0.6;

// ---------------------------------------------------------------------------
// Top-level profile
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

    // Tier 2 (placeholders, populated by A-5)
    pub naming: NamingConventions,
    pub error_handling: ErrorHandlingProfile,
    pub module_organization: ModuleOrganization,
    #[serde(default)]
    pub dep_profile: DependencyProfile,

    // Tier 3 (optional, populated by A-11)
    pub project_vision: Option<String>,
    pub architecture: Option<ArchitecturePattern>,
    pub domains: Vec<DomainClassification>,

    // Commands
    pub commands: BuildCommandRegistry,

    pub overrides_applied: bool,
}

// ---------------------------------------------------------------------------
// Build system
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BuildSystemProfile {
    pub system: BuildSystem,
    pub config_path: PathBuf,
    pub version: Option<String>,
    pub monorepo: bool,
    pub workspaces: Vec<WorkspaceMember>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BuildSystem {
    Cargo,
    Npm,
    Bun,
    Deno,
    PyProject,
    SetupPy,
    GoMod,
    CMake,
    Make,
    Gradle,
    Maven,
    Mix,
    Bundler,
    Composer,
    DotNet,
    Zig,
    Meson,
    Just,
    Task,
    Custom(String),
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceMember {
    pub name: String,
    pub path: PathBuf,
}

// ---------------------------------------------------------------------------
// Language
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LanguageProfile {
    pub primary: Language,
    pub secondary: Vec<Language>,
    pub distribution: HashMap<Language, usize>,
    pub tree_sitter_coverage: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Language {
    Rust,
    Python,
    TypeScript,
    JavaScript,
    Go,
    Java,
    Kotlin,
    CSharp,
    Cpp,
    C,
    Ruby,
    Elixir,
    PHP,
    Swift,
    Zig,
    Dart,
    Lua,
    Svelte,
    Vue,
    Haskell,
    Scala,
    Clojure,
    OCaml,
    Gleam,
    Other(String),
    Unknown,
}

impl Language {
    /// Lowercase string identifier for this language (used by convention extraction).
    pub fn id_str(&self) -> &str {
        match self {
            Self::Rust => "rust",
            Self::Python => "python",
            Self::TypeScript => "typescript",
            Self::JavaScript => "javascript",
            Self::Go => "go",
            Self::Java => "java",
            Self::Kotlin => "kotlin",
            Self::CSharp => "csharp",
            Self::Cpp => "cpp",
            Self::C => "c",
            Self::Ruby => "ruby",
            Self::Elixir => "elixir",
            Self::PHP => "php",
            Self::Swift => "swift",
            Self::Zig => "zig",
            Self::Dart => "dart",
            Self::Lua => "lua",
            Self::Svelte => "svelte",
            Self::Vue => "vue",
            Self::Haskell => "haskell",
            Self::Scala => "scala",
            Self::Clojure => "clojure",
            Self::OCaml => "ocaml",
            Self::Gleam => "gleam",
            Self::Other(s) => s.as_str(),
            Self::Unknown => "unknown",
        }
    }
}

// ---------------------------------------------------------------------------
// Framework
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FrameworkDetection {
    pub name: String,
    pub version: Option<String>,
    pub category: FrameworkCategory,
    pub confidence: f32,
    pub detected_via: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FrameworkCategory {
    Web,
    Game,
    ML,
    Desktop,
    Mobile,
    Fullstack,
    Server,
    Testing,
    Other(String),
}

// ---------------------------------------------------------------------------
// Test profile
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestProfile {
    pub runner_cmd: Option<String>,
    pub location_pattern: TestLocationPattern,
    pub coverage_tool: Option<String>,
    pub fixture_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TestLocationPattern {
    SameFile,
    Adjacent,
    SeparateDirectory,
    Mixed,
    Unknown,
}

// ---------------------------------------------------------------------------
// Lint / formatter
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LintProfile {
    pub tool: String,
    pub config_path: Option<PathBuf>,
    pub language: Language,
    pub auto_fix: bool,
}

// ---------------------------------------------------------------------------
// Entry points
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntryPoint {
    pub file: PathBuf,
    pub kind: EntryPointKind,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EntryPointKind {
    Binary,
    Library,
    Script,
    Server,
}

// ---------------------------------------------------------------------------
// CI / deployment
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CiProfile {
    pub system: CiSystem,
    pub config_path: PathBuf,
    pub detected_steps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CiSystem {
    GitHubActions,
    GitLabCi,
    Jenkins,
    CircleCi,
    BitbucketPipelines,
    TravisCi,
    Other(String),
}

// ---------------------------------------------------------------------------
// Tier 2 placeholders (populated by A-5)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NamingConventions {
    pub functions: Convention,
    pub types: Convention,
    pub constants: Convention,
    pub modules: Convention,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Convention {
    Strong(String),
    Moderate(String),
    Mixed,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ErrorHandlingProfile {
    pub pattern: ErrorPattern,
    pub propagation: PropagationStyle,
    pub error_crate: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ErrorPattern {
    ResultBased,
    ExceptionBased,
    ErrorCodeBased,
    Mixed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PropagationStyle {
    QuestionMark,
    TryCatch,
    IfErrNil,
    Mixed,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModuleOrganization {
    pub pattern: OrgPattern,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OrgPattern {
    Flat,
    ByType,
    ByFeature,
    Layered,
    Monorepo,
    Unknown,
}

// ---------------------------------------------------------------------------
// Dependency profile (spec section 3E)
// ---------------------------------------------------------------------------

/// How deeply dependencies were analyzed.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ClassificationDepth {
    /// Only manifest-declared deps (all languages).
    DeclarationOnly,
    /// DKG import-edge analysis (Rust-only, Tier B).
    UsageBased,
}

impl Default for ClassificationDepth {
    fn default() -> Self {
        Self::DeclarationOnly
    }
}

/// Pinning strategy detected from manifest lock files.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PinningStrategy {
    /// All deps pinned to exact versions.
    Pinned,
    /// Semver ranges (^, ~).
    Ranged,
    /// Wildcard or no version constraints.
    Wild,
    Unknown,
}

impl Default for PinningStrategy {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Dependency analysis results (Tier 2, spec section 3E).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DependencyProfile {
    pub total: usize,
    pub declared: Vec<String>,
    pub dev_only: Vec<String>,
    /// Tier B Rust-only: core deps (high import fan-in).
    pub core: Vec<String>,
    /// Tier B Rust-only: peripheral deps (low import fan-in).
    pub peripheral: Vec<String>,
    /// Tier B Rust-only: declared but unused deps.
    pub unused: Vec<String>,
    pub pinning_strategy: PinningStrategy,
    pub classification_depth: ClassificationDepth,
}

impl Default for DependencyProfile {
    fn default() -> Self {
        Self {
            total: 0,
            declared: Vec::new(),
            dev_only: Vec::new(),
            core: Vec::new(),
            peripheral: Vec::new(),
            unused: Vec::new(),
            pinning_strategy: PinningStrategy::Unknown,
            classification_depth: ClassificationDepth::DeclarationOnly,
        }
    }
}

// ---------------------------------------------------------------------------
// Tier 3 placeholders
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ArchitecturePattern {
    Layered,
    Hexagonal,
    Microservices,
    Monolith,
    EventDriven,
    Serverless,
    Unknown,
}

// ---------------------------------------------------------------------------
// Domain classification (spec section 4B: 12 domains from signal table)
// ---------------------------------------------------------------------------

/// Classified project domain with confidence and supporting signals.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DomainClassification {
    pub domain: ProjectDomain,
    pub confidence: f32,
    pub signals: Vec<String>,
}

/// The 12 project domains from the spec signal table.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ProjectDomain {
    WebBackend,
    WebFrontend,
    Fullstack,
    CliTool,
    Library,
    Game,
    MlPipeline,
    DataPipeline,
    Mobile,
    Desktop,
    Embedded,
    DevOps,
}

impl ProjectDomain {
    pub fn label(&self) -> &'static str {
        match self {
            Self::WebBackend => "WEB_BACKEND",
            Self::WebFrontend => "WEB_FRONTEND",
            Self::Fullstack => "FULLSTACK",
            Self::CliTool => "CLI_TOOL",
            Self::Library => "LIBRARY",
            Self::Game => "GAME",
            Self::MlPipeline => "ML_PIPELINE",
            Self::DataPipeline => "DATA_PIPELINE",
            Self::Mobile => "MOBILE",
            Self::Desktop => "DESKTOP",
            Self::Embedded => "EMBEDDED",
            Self::DevOps => "DEVOPS",
        }
    }
}

// ---------------------------------------------------------------------------
// Build command registry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BuildCommandRegistry {
    pub build: Vec<BuildCommand>,
    pub test: Vec<BuildCommand>,
    pub lint: Vec<BuildCommand>,
    pub format: Vec<BuildCommand>,
    pub run: Vec<BuildCommand>,
    pub clean: Vec<BuildCommand>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BuildCommand {
    pub cmd: String,
    pub working_dir: PathBuf,
    pub env: HashMap<String, String>,
    pub timeout_s: u32,
    pub scope: CommandScope,
    pub verified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CommandScope {
    Full,
    Crate(String),
    WorkspaceMember(String),
    Directory(PathBuf),
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ProjectProfileError {
    #[error("IO error scanning project: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("no build system detected in {0}")]
    NoBuildSystem(PathBuf),
    #[error("project root does not exist: {0}")]
    RootNotFound(PathBuf),
}

// ---------------------------------------------------------------------------
// Default impls
// ---------------------------------------------------------------------------

impl Default for BuildCommandRegistry {
    fn default() -> Self {
        Self {
            build: Vec::new(),
            test: Vec::new(),
            lint: Vec::new(),
            format: Vec::new(),
            run: Vec::new(),
            clean: Vec::new(),
        }
    }
}

impl Default for NamingConventions {
    fn default() -> Self {
        Self {
            functions: Convention::Unknown,
            types: Convention::Unknown,
            constants: Convention::Unknown,
            modules: Convention::Unknown,
        }
    }
}

impl Default for ErrorHandlingProfile {
    fn default() -> Self {
        Self {
            pattern: ErrorPattern::Unknown,
            propagation: PropagationStyle::Unknown,
            error_crate: None,
        }
    }
}

impl Default for ModuleOrganization {
    fn default() -> Self {
        Self {
            pattern: OrgPattern::Unknown,
        }
    }
}

impl Default for TestProfile {
    fn default() -> Self {
        Self {
            runner_cmd: None,
            location_pattern: TestLocationPattern::Unknown,
            coverage_tool: None,
            fixture_dir: None,
        }
    }
}

impl Default for LanguageProfile {
    fn default() -> Self {
        Self {
            primary: Language::Unknown,
            secondary: Vec::new(),
            distribution: HashMap::new(),
            tree_sitter_coverage: 0.0,
        }
    }
}

impl Default for BuildSystemProfile {
    fn default() -> Self {
        Self {
            system: BuildSystem::Unknown,
            config_path: PathBuf::new(),
            version: None,
            monorepo: false,
            workspaces: Vec::new(),
        }
    }
}
