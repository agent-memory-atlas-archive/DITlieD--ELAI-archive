// Context Telescope: wire ProjectProfile data into prompt assembly across 5 layers.
// Spec: docs/specs/guide/project-intelligence-design.txt section 8.
//
// Layer 1: Project vision (from Tier 3 or template fallback).
// Layer 2-4: Decomposition summaries receive profile (handled by caller).
// Prompt assembly: build/test commands in baby steps.
// Few-shot examples: convention-matched (handled by DKG caller).
// Template generation: convention-aware scaffolds.
//
// This module produces the text blocks that get injected into PhaseSpec (Slot 3).

use std::fmt::Write;

use elai_core::project_profile::{
    ArchitecturePattern, Convention, Language, ProjectProfile,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum tokens for the telescope injection into PhaseSpec (rough char estimate).
/// Keeps the telescope compact to avoid blowing Slot 3 budget.
const TELESCOPE_MAX_CHARS: usize = 1200;

// ---------------------------------------------------------------------------
// TelescopeContext: assembled context data for prompt injection
// ---------------------------------------------------------------------------

/// Assembled telescope context ready for injection into PhaseSpec content.
#[derive(Debug, Clone)]
pub struct TelescopeContext {
    /// Layer 1: project vision string.
    pub project_vision: String,
    /// Domains detected (for rule activation and prompt templates).
    pub domain_labels: Vec<String>,
    /// Architecture (advisory, for decomposition hints).
    pub architecture: Option<String>,
    /// Build command for verification instructions.
    pub build_cmd: Option<String>,
    /// Test command for verification instructions.
    pub test_cmd: Option<String>,
    /// Convention summary for template generation.
    pub convention_summary: Option<String>,
    /// Primary language identifier.
    pub language: String,
}

impl TelescopeContext {
    /// Format as a compact text block for PhaseSpec injection.
    /// This goes into Slot 3 (PhaseSpec) as a PROJECT CONTEXT header.
    pub fn format_for_phase_spec(&self) -> String {
        let mut out = String::with_capacity(TELESCOPE_MAX_CHARS);

        out.push_str("PROJECT CONTEXT\n");
        let _ = writeln!(out, "Vision: {}", self.project_vision);

        if !self.domain_labels.is_empty() {
            let _ = writeln!(out, "Domains: {}", self.domain_labels.join(", "));
        }

        if let Some(ref arch) = self.architecture {
            let _ = writeln!(out, "Architecture: {arch}");
        }

        let _ = writeln!(out, "Language: {}", self.language);

        if let Some(ref conv) = self.convention_summary {
            let _ = writeln!(out, "Conventions: {conv}");
        }

        if let Some(ref build) = self.build_cmd {
            let _ = writeln!(out, "Build: {build}");
        }
        if let Some(ref test) = self.test_cmd {
            let _ = writeln!(out, "Test: {test}");
        }

        // Enforce size limit.
        if out.len() > TELESCOPE_MAX_CHARS {
            out.truncate(TELESCOPE_MAX_CHARS);
            if let Some(last_nl) = out.rfind('\n') {
                out.truncate(last_nl + 1);
            }
        }

        out
    }

    /// Format a compact verification instruction for baby step prompts.
    /// "After writing this code, verify: {test_cmd}"
    pub fn format_verification_instruction(&self) -> Option<String> {
        self.test_cmd.as_ref().map(|cmd| {
            format!("After writing this code, verify: {cmd}")
        })
    }

    /// Estimated token count (chars / 4 is a rough English approximation).
    pub fn estimated_tokens(&self) -> usize {
        self.format_for_phase_spec().len() / 4
    }
}

// ---------------------------------------------------------------------------
// Assembly: build TelescopeContext from ProjectProfile
// ---------------------------------------------------------------------------

/// Assemble telescope context from a ProjectProfile.
/// This is the main entry point: given profile data, produce the context
/// that gets injected into prompt assembly.
pub fn assemble_telescope(profile: &ProjectProfile) -> TelescopeContext {
    // Layer 1: project vision.
    let project_vision = profile.project_vision.clone()
        .unwrap_or_else(|| crate::tier3::generate_project_vision(profile));

    // Domain labels.
    let domain_labels: Vec<String> = profile.domains.iter()
        .map(|d| d.domain.label().to_owned())
        .collect();

    // Architecture (advisory).
    let architecture = profile.architecture.as_ref()
        .filter(|a| **a != ArchitecturePattern::Unknown)
        .map(|a| format!("{a:?}"));

    // Build/test commands.
    let build_cmd = profile.commands.build.first().map(|c| c.cmd.clone());
    let test_cmd = profile.commands.test.first().map(|c| c.cmd.clone());

    // Convention summary.
    let convention_summary = format_convention_summary(profile);

    let language = profile.languages.primary.id_str().to_owned();

    TelescopeContext {
        project_vision,
        domain_labels,
        architecture,
        build_cmd,
        test_cmd,
        convention_summary,
        language,
    }
}

/// Build a compact convention summary from Tier 2 naming + error handling data.
fn format_convention_summary(profile: &ProjectProfile) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();

    if let Convention::Strong(ref style) = profile.naming.functions {
        parts.push(format!("fn:{style}"));
    }
    if let Convention::Strong(ref style) = profile.naming.types {
        parts.push(format!("types:{style}"));
    }
    if let Convention::Strong(ref style) = profile.naming.constants {
        parts.push(format!("const:{style}"));
    }
    if let Some(ref crate_name) = profile.error_handling.error_crate {
        parts.push(format!("errors:{crate_name}"));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
}

// ---------------------------------------------------------------------------
// Convention-aware template generation (spec section 8: TEMPLATE GENERATION)
// ---------------------------------------------------------------------------

/// Generate a convention-aware function template for the project's primary language.
/// Used by decomposer and baby step generation (future Phase 4D integration).
pub fn convention_function_template(profile: &ProjectProfile) -> String {
    let fn_style = match &profile.naming.functions {
        Convention::Strong(s) => s.as_str(),
        _ => "snake_case",
    };

    let error_type = profile.error_handling.error_crate.as_deref()
        .unwrap_or("Error");

    match profile.languages.primary {
        Language::Rust => {
            format!("fn {{{}}}(...) -> Result<T, {}> {{ ... }}", fn_style, error_type)
        }
        Language::Python => {
            format!("def {{{}}}(self, ...) -> ...:", fn_style)
        }
        Language::TypeScript | Language::JavaScript => {
            let style = if fn_style == "snake_case" { "camelCase" } else { fn_style };
            format!("function {{{}}}(...): ... {{ ... }}", style)
        }
        Language::Go => {
            format!("func {{{}}}(...) (..., error) {{ ... }}", fn_style)
        }
        _ => {
            format!("function {{{fn_style}}}(...)")
        }
    }
}

/// Generate a language-specific import template from profile.
/// Used by Layer 1 fix templates (reasoning-architecture-design.txt).
pub fn import_template(profile: &ProjectProfile) -> &'static str {
    match profile.languages.primary {
        Language::Rust => "use {crate}::{symbol};",
        Language::Python => "from {module} import {symbol}",
        Language::Go => "import \"{package}\"",
        Language::TypeScript | Language::JavaScript => "import {{ {symbol} }} from '{module}';",
        Language::Ruby => "require '{module}'",
        Language::Java | Language::Kotlin => "import {package}.{symbol};",
        Language::CSharp => "using {namespace};",
        _ => "import {module}",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::project_profile::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn test_profile() -> ProjectProfile {
        ProjectProfile {
            schema_version: PROFILE_SCHEMA_VERSION,
            generated_at: "2026-04-14T00:00:00Z".to_owned(),
            project_root: PathBuf::from("/test"),
            build_system: BuildSystemProfile {
                system: BuildSystem::Cargo,
                config_path: PathBuf::from("Cargo.toml"),
                version: None,
                monorepo: false,
                workspaces: Vec::new(),
            },
            languages: LanguageProfile {
                primary: Language::Rust,
                secondary: Vec::new(),
                distribution: HashMap::new(),
                tree_sitter_coverage: 0.0,
            },
            frameworks: vec![FrameworkDetection {
                name: "Axum".to_owned(),
                version: None,
                category: FrameworkCategory::Web,
                confidence: 1.0,
                detected_via: "Cargo.toml".to_owned(),
            }],
            test_profile: TestProfile::default(),
            lint_tools: Vec::new(),
            entry_points: vec![EntryPoint {
                file: PathBuf::from("src/main.rs"),
                kind: EntryPointKind::Server,
            }],
            ci: None,
            naming: NamingConventions {
                functions: Convention::Strong("snake_case".to_owned()),
                types: Convention::Strong("PascalCase".to_owned()),
                constants: Convention::Strong("SCREAMING_SNAKE_CASE".to_owned()),
                modules: Convention::Unknown,
            },
            error_handling: ErrorHandlingProfile {
                pattern: ErrorPattern::ResultBased,
                propagation: PropagationStyle::QuestionMark,
                error_crate: Some("thiserror".to_owned()),
            },
            module_organization: ModuleOrganization { pattern: OrgPattern::Layered },
            dep_profile: DependencyProfile::default(),
            project_vision: Some("Rust WEB_BACKEND using Axum. Build: cargo build. Test: cargo test.".to_owned()),
            architecture: Some(ArchitecturePattern::Layered),
            domains: vec![DomainClassification {
                domain: ProjectDomain::WebBackend,
                confidence: 0.8,
                signals: vec!["web framework: Axum".to_owned()],
            }],
            commands: BuildCommandRegistry {
                build: vec![BuildCommand {
                    cmd: "cargo build".to_owned(),
                    working_dir: PathBuf::from("."),
                    env: HashMap::new(),
                    timeout_s: 120,
                    scope: CommandScope::Full,
                    verified: false,
                }],
                test: vec![BuildCommand {
                    cmd: "cargo test".to_owned(),
                    working_dir: PathBuf::from("."),
                    env: HashMap::new(),
                    timeout_s: 120,
                    scope: CommandScope::Full,
                    verified: false,
                }],
                lint: Vec::new(),
                format: Vec::new(),
                run: Vec::new(),
                clean: Vec::new(),
            },
            overrides_applied: false,
        }
    }

    #[test]
    fn telescope_assembles_from_profile() {
        let profile = test_profile();
        let ctx = assemble_telescope(&profile);

        assert!(!ctx.project_vision.is_empty());
        assert_eq!(ctx.language, "rust");
        assert!(!ctx.domain_labels.is_empty());
        assert_eq!(ctx.domain_labels[0], "WEB_BACKEND");
        assert!(ctx.build_cmd.is_some());
        assert!(ctx.test_cmd.is_some());
        assert!(ctx.convention_summary.is_some());
    }

    #[test]
    fn telescope_format_contains_all_sections() {
        let profile = test_profile();
        let ctx = assemble_telescope(&profile);
        let formatted = ctx.format_for_phase_spec();

        assert!(formatted.contains("PROJECT CONTEXT"));
        assert!(formatted.contains("Vision:"));
        assert!(formatted.contains("Domains:"));
        assert!(formatted.contains("Language: rust"));
        assert!(formatted.contains("Build: cargo build"));
        assert!(formatted.contains("Test: cargo test"));
        assert!(formatted.contains("Conventions:"));
    }

    #[test]
    fn telescope_format_respects_size_limit() {
        let profile = test_profile();
        let ctx = assemble_telescope(&profile);
        let formatted = ctx.format_for_phase_spec();

        assert!(formatted.len() <= TELESCOPE_MAX_CHARS);
    }

    #[test]
    fn verification_instruction_generated() {
        let profile = test_profile();
        let ctx = assemble_telescope(&profile);
        let instr = ctx.format_verification_instruction();

        assert!(instr.is_some());
        assert!(instr.unwrap().contains("cargo test"));
    }

    #[test]
    fn convention_template_rust() {
        let profile = test_profile();
        let tmpl = convention_function_template(&profile);
        assert!(tmpl.contains("Result<T"));
        assert!(tmpl.contains("thiserror"));
    }

    #[test]
    fn import_template_rust() {
        let profile = test_profile();
        let tmpl = import_template(&profile);
        assert!(tmpl.contains("use"));
    }

    #[test]
    fn empty_profile_telescope_still_works() {
        let profile = ProjectProfile {
            schema_version: PROFILE_SCHEMA_VERSION,
            generated_at: "2026-04-14T00:00:00Z".to_owned(),
            project_root: PathBuf::from("/empty"),
            build_system: BuildSystemProfile::default(),
            languages: LanguageProfile::default(),
            frameworks: Vec::new(),
            test_profile: TestProfile::default(),
            lint_tools: Vec::new(),
            entry_points: Vec::new(),
            ci: None,
            naming: NamingConventions::default(),
            error_handling: ErrorHandlingProfile::default(),
            module_organization: ModuleOrganization::default(),
            dep_profile: DependencyProfile::default(),
            project_vision: None,
            architecture: None,
            domains: Vec::new(),
            commands: BuildCommandRegistry::default(),
            overrides_applied: false,
        };

        let ctx = assemble_telescope(&profile);
        let formatted = ctx.format_for_phase_spec();
        assert!(formatted.contains("PROJECT CONTEXT"));
        assert!(formatted.contains("Language: unknown"));
    }
}
