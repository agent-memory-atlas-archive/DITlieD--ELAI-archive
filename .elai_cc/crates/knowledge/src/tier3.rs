// Tier 3: Semantic understanding (LLM-assisted optional, deterministic fallback).
// Spec: docs/specs/guide/project-intelligence-design.txt sections 4A-4C.
//
// Domain classification: signal table with 12 domains (deterministic).
// Architecture inference: DKG dependency graph pattern detection (advisory only).
// Project vision: template-based generation (LLM path deferred to sidecar integration).

use tracing::debug;

use elai_core::project_profile::{
    ArchitecturePattern, DomainClassification, EntryPointKind, FrameworkCategory,
    Language, ProjectDomain, ProjectProfile,
};

use crate::DkgNodeInfo;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Minimum confidence to report a domain classification.
const DOMAIN_CONFIDENCE_THRESHOLD: f32 = 0.6;

/// Minimum confidence for architecture inference (spec 4C: < 0.6 => Unknown).
const ARCH_CONFIDENCE_THRESHOLD: f32 = 0.6;

/// Maximum token count for project vision (spec 4A: 200 tokens ~= 150 words).
const VISION_MAX_CHARS: usize = 800;

// ---------------------------------------------------------------------------
// Domain classification (spec section 4B: signal table, 12 domains)
// ---------------------------------------------------------------------------

/// Classify project into domains from Tier 1+2 signals. Returns all domains
/// that exceed the confidence threshold, sorted by confidence descending.
pub fn classify_domains(profile: &ProjectProfile) -> Vec<DomainClassification> {
    let mut results = Vec::new();

    // Collect signals per domain from Tier 1+2 data.
    check_web_backend(profile, &mut results);
    check_web_frontend(profile, &mut results);
    check_fullstack(profile, &mut results);
    check_cli_tool(profile, &mut results);
    check_library(profile, &mut results);
    check_game(profile, &mut results);
    check_ml_pipeline(profile, &mut results);
    check_data_pipeline(profile, &mut results);
    check_mobile(profile, &mut results);
    check_desktop(profile, &mut results);
    check_embedded(profile, &mut results);
    check_devops(profile, &mut results);

    // Filter by threshold and sort by confidence descending.
    results.retain(|d| d.confidence >= DOMAIN_CONFIDENCE_THRESHOLD);
    results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

    debug!(domain_count = results.len(), "domain classification complete");
    results
}

fn has_framework_category(profile: &ProjectProfile, category: FrameworkCategory) -> Option<String> {
    profile.frameworks.iter()
        .find(|f| f.category == category)
        .map(|f| f.name.clone())
}

fn has_dep_keyword(profile: &ProjectProfile, keyword: &str) -> bool {
    profile.frameworks.iter().any(|f| f.name.to_lowercase().contains(keyword))
}

fn has_entry_kind(profile: &ProjectProfile, kind: EntryPointKind) -> bool {
    profile.entry_points.iter().any(|e| e.kind == kind)
}

fn check_web_backend(profile: &ProjectProfile, results: &mut Vec<DomainClassification>) {
    let mut signals = Vec::new();
    let mut score: f32 = 0.0;

    if let Some(name) = has_framework_category(profile, FrameworkCategory::Web) {
        signals.push(format!("web framework: {name}"));
        score += 0.5;
    }
    if let Some(name) = has_framework_category(profile, FrameworkCategory::Server) {
        signals.push(format!("server framework: {name}"));
        score += 0.4;
    }
    if has_entry_kind(profile, EntryPointKind::Server) {
        signals.push("server entry point".to_owned());
        score += 0.3;
    }
    if !signals.is_empty() {
        results.push(DomainClassification { domain: ProjectDomain::WebBackend, confidence: score.min(1.0), signals });
    }
}

fn check_web_frontend(profile: &ProjectProfile, results: &mut Vec<DomainClassification>) {
    let mut signals = Vec::new();
    let mut score: f32 = 0.0;

    let frontend_langs = [Language::TypeScript, Language::JavaScript, Language::Svelte, Language::Vue];
    let has_frontend_lang = frontend_langs.iter().any(|l| {
        profile.languages.primary == *l || profile.languages.secondary.contains(l)
    });

    if has_frontend_lang {
        let fw_names: Vec<&str> = profile.frameworks.iter()
            .filter(|f| matches!(f.name.as_str(), "React" | "Vue" | "Svelte" | "Next.js"))
            .map(|f| f.name.as_str())
            .collect();
        if !fw_names.is_empty() {
            signals.push(format!("frontend framework: {}", fw_names.join(", ")));
            score += 0.6;
        }
    }

    // Check for component file extensions via framework detection.
    if profile.languages.secondary.contains(&Language::Svelte)
        || profile.languages.secondary.contains(&Language::Vue)
        || profile.languages.primary == Language::Svelte
        || profile.languages.primary == Language::Vue
    {
        signals.push("component file language detected".to_owned());
        score += 0.3;
    }

    if !signals.is_empty() {
        results.push(DomainClassification { domain: ProjectDomain::WebFrontend, confidence: score.min(1.0), signals });
    }
}

fn check_fullstack(profile: &ProjectProfile, results: &mut Vec<DomainClassification>) {
    // Fullstack = both backend AND frontend signals.
    let has_backend = has_framework_category(profile, FrameworkCategory::Web).is_some()
        || has_framework_category(profile, FrameworkCategory::Server).is_some();

    let frontend_fws = ["React", "Vue", "Svelte", "Next.js"];
    let has_frontend = profile.frameworks.iter()
        .any(|f| frontend_fws.contains(&f.name.as_str()));

    if has_backend && has_frontend {
        let mut signals = vec!["backend + frontend frameworks detected".to_owned()];
        if has_framework_category(profile, FrameworkCategory::Fullstack).is_some() {
            signals.push("fullstack framework detected".to_owned());
        }
        results.push(DomainClassification { domain: ProjectDomain::Fullstack, confidence: 0.8, signals });
    }
}

fn check_cli_tool(profile: &ProjectProfile, results: &mut Vec<DomainClassification>) {
    let mut signals = Vec::new();
    let mut score: f32 = 0.0;

    if has_entry_kind(profile, EntryPointKind::Binary) {
        signals.push("binary entry point".to_owned());
        score += 0.4;
    }

    // Arg parser dependencies.
    let arg_parsers = ["clap", "argparse", "commander", "cobra", "urfave/cli"];
    for ap in &arg_parsers {
        if has_dep_keyword(profile, ap) {
            signals.push(format!("arg parser dep: {ap}"));
            score += 0.4;
        }
    }

    if !signals.is_empty() {
        results.push(DomainClassification { domain: ProjectDomain::CliTool, confidence: score.min(1.0), signals });
    }
}

fn check_library(profile: &ProjectProfile, results: &mut Vec<DomainClassification>) {
    let mut signals = Vec::new();
    let mut score: f32 = 0.0;

    if has_entry_kind(profile, EntryPointKind::Library) {
        signals.push("library entry point".to_owned());
        score += 0.5;
    }
    if !has_entry_kind(profile, EntryPointKind::Binary) && !has_entry_kind(profile, EntryPointKind::Server) {
        signals.push("no binary/server entry".to_owned());
        score += 0.3;
    }

    if !signals.is_empty() {
        results.push(DomainClassification { domain: ProjectDomain::Library, confidence: score.min(1.0), signals });
    }
}

fn check_game(profile: &ProjectProfile, results: &mut Vec<DomainClassification>) {
    if let Some(name) = has_framework_category(profile, FrameworkCategory::Game) {
        results.push(DomainClassification {
            domain: ProjectDomain::Game,
            confidence: 0.9,
            signals: vec![format!("game engine: {name}")],
        });
    }
}

fn check_ml_pipeline(profile: &ProjectProfile, results: &mut Vec<DomainClassification>) {
    if let Some(name) = has_framework_category(profile, FrameworkCategory::ML) {
        results.push(DomainClassification {
            domain: ProjectDomain::MlPipeline,
            confidence: 0.8,
            signals: vec![format!("ML framework: {name}")],
        });
    }
    // Additional ML signals: torch, tensorflow, jax deps.
    let ml_deps = ["torch", "tensorflow", "jax", "scikit-learn"];
    let mut ml_signals: Vec<String> = Vec::new();
    for dep in &ml_deps {
        if has_dep_keyword(profile, dep) {
            ml_signals.push(format!("ML dep: {dep}"));
        }
    }
    if !ml_signals.is_empty() && !results.iter().any(|r| r.domain == ProjectDomain::MlPipeline) {
        let confidence = (ml_signals.len() as f32 * 0.3).min(1.0);
        results.push(DomainClassification { domain: ProjectDomain::MlPipeline, confidence, signals: ml_signals });
    }
}

fn check_data_pipeline(profile: &ProjectProfile, results: &mut Vec<DomainClassification>) {
    let etl_deps = ["pandas", "polars", "spark", "dbt", "airflow", "prefect", "dagster"];
    let mut signals: Vec<String> = Vec::new();
    for dep in &etl_deps {
        if has_dep_keyword(profile, dep) {
            signals.push(format!("ETL/data dep: {dep}"));
        }
    }
    if !signals.is_empty() {
        let confidence = (signals.len() as f32 * 0.3).min(1.0);
        results.push(DomainClassification { domain: ProjectDomain::DataPipeline, confidence, signals });
    }
}

fn check_mobile(profile: &ProjectProfile, results: &mut Vec<DomainClassification>) {
    if let Some(name) = has_framework_category(profile, FrameworkCategory::Mobile) {
        results.push(DomainClassification {
            domain: ProjectDomain::Mobile,
            confidence: 0.9,
            signals: vec![format!("mobile framework: {name}")],
        });
        return;
    }
    let mobile_langs = [Language::Swift, Language::Kotlin, Language::Dart];
    for lang in &mobile_langs {
        if profile.languages.primary == *lang {
            results.push(DomainClassification {
                domain: ProjectDomain::Mobile,
                confidence: 0.7,
                signals: vec![format!("mobile language: {}", lang.id_str())],
            });
            return;
        }
    }
}

fn check_desktop(profile: &ProjectProfile, results: &mut Vec<DomainClassification>) {
    if let Some(name) = has_framework_category(profile, FrameworkCategory::Desktop) {
        results.push(DomainClassification {
            domain: ProjectDomain::Desktop,
            confidence: 0.9,
            signals: vec![format!("desktop framework: {name}")],
        });
    }
}

fn check_embedded(profile: &ProjectProfile, results: &mut Vec<DomainClassification>) {
    let embedded_deps = ["cortex-m", "esp-hal", "no_std", "embedded-hal", "probe-rs"];
    let mut signals: Vec<String> = Vec::new();
    for dep in &embedded_deps {
        if has_dep_keyword(profile, dep) {
            signals.push(format!("embedded dep: {dep}"));
        }
    }
    if !signals.is_empty() {
        results.push(DomainClassification {
            domain: ProjectDomain::Embedded,
            confidence: 0.9,
            signals,
        });
    }
}

fn check_devops(profile: &ProjectProfile, results: &mut Vec<DomainClassification>) {
    let devops_deps = ["terraform", "ansible", "pulumi", "cdk"];
    let mut signals: Vec<String> = Vec::new();
    for dep in &devops_deps {
        if has_dep_keyword(profile, dep) {
            signals.push(format!("IaC dep: {dep}"));
        }
    }
    if !signals.is_empty() {
        results.push(DomainClassification {
            domain: ProjectDomain::DevOps,
            confidence: (signals.len() as f32 * 0.4).min(1.0),
            signals,
        });
    }
}

// ---------------------------------------------------------------------------
// Architecture inference (spec section 4C: advisory only)
// ---------------------------------------------------------------------------

/// Infer architecture pattern from DKG dependency graph + module organization.
/// Returns Unknown if confidence < threshold.
pub fn infer_architecture(
    profile: &ProjectProfile,
    dkg_nodes: &[DkgNodeInfo],
) -> ArchitecturePattern {
    // If no DKG data and no module org signal, return Unknown.
    if dkg_nodes.is_empty() && profile.module_organization.pattern == elai_core::project_profile::OrgPattern::Unknown {
        return ArchitecturePattern::Unknown;
    }

    let mut best = ArchitecturePattern::Unknown;
    let mut best_conf: f32 = 0.0;

    // Layered: clear dependency direction (handlers -> services -> repos).
    let layered_conf = detect_layered_pattern(profile, dkg_nodes);
    if layered_conf > best_conf { best_conf = layered_conf; best = ArchitecturePattern::Layered; }

    // Hexagonal: ports + adapters pattern.
    let hex_conf = detect_hexagonal_pattern(dkg_nodes);
    if hex_conf > best_conf { best_conf = hex_conf; best = ArchitecturePattern::Hexagonal; }

    // Monolith: single deployment, shared state (default for simple projects).
    let mono_conf = detect_monolith_pattern(profile);
    if mono_conf > best_conf { best_conf = mono_conf; best = ArchitecturePattern::Monolith; }

    // Microservices: multiple independent services.
    let micro_conf = detect_microservices_pattern(profile);
    if micro_conf > best_conf { best_conf = micro_conf; best = ArchitecturePattern::Microservices; }

    if best_conf < ARCH_CONFIDENCE_THRESHOLD {
        debug!(best_confidence = best_conf, "architecture confidence below threshold, returning Unknown");
        return ArchitecturePattern::Unknown;
    }

    debug!(?best, confidence = best_conf, "architecture inference complete");
    best
}

fn detect_layered_pattern(profile: &ProjectProfile, dkg_nodes: &[DkgNodeInfo]) -> f32 {
    let mut score: f32 = 0.0;

    if profile.module_organization.pattern == elai_core::project_profile::OrgPattern::Layered {
        score += 0.5;
    }

    // Look for handler/service/repo layer names in DKG file paths.
    let layer_keywords = ["handler", "service", "repo", "controller", "model", "middleware"];
    let mut layers_found = 0usize;
    for kw in &layer_keywords {
        if dkg_nodes.iter().any(|n| {
            n.file_path.to_str().map_or(false, |p| p.to_lowercase().contains(kw))
        }) {
            layers_found += 1;
        }
    }
    if layers_found >= 3 { score += 0.4; }
    else if layers_found >= 2 { score += 0.2; }

    score.min(1.0)
}

fn detect_hexagonal_pattern(dkg_nodes: &[DkgNodeInfo]) -> f32 {
    let mut score: f32 = 0.0;

    // Ports: look for trait definitions at core paths.
    let port_keywords = ["port", "adapter", "driven", "driving"];
    let mut ports_found = 0usize;
    for kw in &port_keywords {
        if dkg_nodes.iter().any(|n| {
            n.file_path.to_str().map_or(false, |p| p.to_lowercase().contains(kw))
        }) {
            ports_found += 1;
        }
    }
    if ports_found >= 2 { score += 0.7; }
    else if ports_found >= 1 { score += 0.3; }

    score.min(1.0)
}

fn detect_monolith_pattern(profile: &ProjectProfile) -> f32 {
    // Single deployment unit, not a monorepo.
    if !profile.build_system.monorepo && profile.entry_points.len() <= 2 {
        return 0.5;
    }
    0.0
}

fn detect_microservices_pattern(profile: &ProjectProfile) -> f32 {
    // Multiple workspace members with independent entry points.
    if profile.build_system.monorepo && profile.build_system.workspaces.len() >= 3 {
        let server_entries = profile.entry_points.iter()
            .filter(|e| e.kind == EntryPointKind::Server || e.kind == EntryPointKind::Binary)
            .count();
        if server_entries >= 3 {
            return 0.8;
        }
        return 0.4;
    }
    0.0
}

// ---------------------------------------------------------------------------
// Project vision generation (spec section 4A: template fallback)
// ---------------------------------------------------------------------------

/// Generate a project vision string from Tier 1+2 data.
/// This is the template fallback (~40 tokens). LLM-assisted path is deferred
/// to when sidecar integration for short prompts is available.
pub fn generate_project_vision(profile: &ProjectProfile) -> String {
    let lang = profile.languages.primary.id_str();

    let project_type = if !profile.domains.is_empty() {
        profile.domains[0].domain.label().to_owned()
    } else {
        entry_point_type_label(profile)
    };

    let frameworks: Vec<&str> = profile.frameworks.iter()
        .map(|f| f.name.as_str())
        .take(5)
        .collect();
    let fw_str = if frameworks.is_empty() {
        String::new()
    } else {
        format!(" using {}", frameworks.join(", "))
    };

    let module_count = if profile.build_system.monorepo {
        profile.build_system.workspaces.len()
    } else {
        0
    };

    let build_cmd = profile.commands.build.first()
        .map(|c| c.cmd.as_str())
        .unwrap_or("(none)");
    let test_cmd = profile.commands.test.first()
        .map(|c| c.cmd.as_str())
        .unwrap_or("(none)");

    let arch_str = profile.architecture.as_ref()
        .filter(|a| **a != ArchitecturePattern::Unknown)
        .map(|a| format!(" Architecture: {a:?}."))
        .unwrap_or_default();

    let mut vision = format!(
        "{lang} {project_type}{fw_str}.{arch_str}",
    );

    if module_count > 0 {
        vision.push_str(&format!(" {module_count} workspace members."));
    }

    vision.push_str(&format!(" Build: {build_cmd}. Test: {test_cmd}."));

    // Enforce max chars (proxy for ~200 tokens).
    if vision.len() > VISION_MAX_CHARS {
        vision.truncate(VISION_MAX_CHARS);
        if let Some(last_period) = vision.rfind('.') {
            vision.truncate(last_period + 1);
        }
    }

    vision
}

fn entry_point_type_label(profile: &ProjectProfile) -> String {
    if has_entry_kind(profile, EntryPointKind::Server) {
        "server".to_owned()
    } else if has_entry_kind(profile, EntryPointKind::Binary) {
        "binary".to_owned()
    } else if has_entry_kind(profile, EntryPointKind::Library) {
        "library".to_owned()
    } else if has_entry_kind(profile, EntryPointKind::Script) {
        "script".to_owned()
    } else {
        "project".to_owned()
    }
}

// ---------------------------------------------------------------------------
// Tier 3 orchestration: populate profile with domain, architecture, vision
// ---------------------------------------------------------------------------

/// Populate Tier 3 fields on a ProjectProfile: domains, architecture,
/// project_vision, and (batch 4A) the fix_template_keys discovered under
/// the project root. Call after Tier 1+2 are populated. `dkg_nodes` may be
/// empty (no DKG data).
pub fn populate_tier3(
    profile: &mut ProjectProfile,
    dkg_nodes: &[DkgNodeInfo],
) {
    debug!("populating Tier 3: domains, architecture, vision, fix_templates");

    profile.domains = classify_domains(profile);
    profile.architecture = Some(infer_architecture(profile, dkg_nodes));
    profile.project_vision = Some(generate_project_vision(profile));

    // Batch 4A (A-36): load curated fix templates from
    // `.elai_cc/data/fix_templates/` (relative to the project root). When the
    // directory is missing the registry is empty and `fix_template_keys`
    // stays the empty vec its `#[serde(default)]` guarantees — fully
    // backward compatible with pre-4A profiles.
    let registry = crate::fix_templates::FixTemplateRegistry::load_default(&profile.project_root);
    crate::fix_templates::attach_fix_template_keys(profile, &registry);
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

    fn base_profile() -> ProjectProfile {
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
            fix_template_keys: Vec::new(),
            overrides_applied: false,
        }
    }

    #[test]
    fn classify_web_backend_with_framework() {
        let mut p = base_profile();
        p.frameworks.push(FrameworkDetection {
            name: "Axum".to_owned(),
            version: None,
            category: FrameworkCategory::Web,
            confidence: 1.0,
            detected_via: "Cargo.toml".to_owned(),
        });
        p.entry_points.push(EntryPoint { file: PathBuf::from("src/main.rs"), kind: EntryPointKind::Server });

        let domains = classify_domains(&p);
        assert!(!domains.is_empty());
        assert_eq!(domains[0].domain, ProjectDomain::WebBackend);
        assert!(domains[0].confidence >= DOMAIN_CONFIDENCE_THRESHOLD);
    }

    #[test]
    fn classify_cli_tool_with_clap() {
        let mut p = base_profile();
        p.entry_points.push(EntryPoint { file: PathBuf::from("src/main.rs"), kind: EntryPointKind::Binary });
        p.frameworks.push(FrameworkDetection {
            name: "clap".to_owned(),
            version: None,
            category: FrameworkCategory::Other("arg_parser".to_owned()),
            confidence: 1.0,
            detected_via: "Cargo.toml".to_owned(),
        });

        let domains = classify_domains(&p);
        let cli = domains.iter().find(|d| d.domain == ProjectDomain::CliTool);
        assert!(cli.is_some(), "expected CLI_TOOL domain");
        assert!(cli.unwrap().confidence >= DOMAIN_CONFIDENCE_THRESHOLD);
    }

    #[test]
    fn classify_game_with_bevy() {
        let mut p = base_profile();
        p.frameworks.push(FrameworkDetection {
            name: "Bevy".to_owned(),
            version: None,
            category: FrameworkCategory::Game,
            confidence: 1.0,
            detected_via: "Cargo.toml".to_owned(),
        });

        let domains = classify_domains(&p);
        let game = domains.iter().find(|d| d.domain == ProjectDomain::Game);
        assert!(game.is_some());
        assert!(game.unwrap().confidence >= 0.9);
    }

    #[test]
    fn classify_empty_project_yields_no_domains() {
        let p = base_profile();
        let domains = classify_domains(&p);
        // Base profile has no frameworks, no entry points => no domains above threshold.
        assert!(domains.is_empty(), "expected no domains for empty profile");
    }

    #[test]
    fn architecture_unknown_without_data() {
        let p = base_profile();
        let arch = infer_architecture(&p, &[]);
        assert_eq!(arch, ArchitecturePattern::Unknown);
    }

    #[test]
    fn project_vision_template_generates_nonempty() {
        let mut p = base_profile();
        p.commands.build.push(BuildCommand {
            cmd: "cargo build".to_owned(),
            working_dir: PathBuf::from("."),
            env: HashMap::new(),
            timeout_s: 120,
            scope: CommandScope::Full,
            verified: false,
        });
        p.commands.test.push(BuildCommand {
            cmd: "cargo test".to_owned(),
            working_dir: PathBuf::from("."),
            env: HashMap::new(),
            timeout_s: 120,
            scope: CommandScope::Full,
            verified: false,
        });

        let vision = generate_project_vision(&p);
        assert!(!vision.is_empty());
        assert!(vision.contains("rust"));
        assert!(vision.contains("cargo build"));
        assert!(vision.len() <= VISION_MAX_CHARS);
    }

    #[test]
    fn populate_tier3_fills_all_fields() {
        let mut p = base_profile();
        p.frameworks.push(FrameworkDetection {
            name: "Axum".to_owned(),
            version: None,
            category: FrameworkCategory::Web,
            confidence: 1.0,
            detected_via: "Cargo.toml".to_owned(),
        });
        p.entry_points.push(EntryPoint { file: PathBuf::from("src/main.rs"), kind: EntryPointKind::Server });

        populate_tier3(&mut p, &[]);

        assert!(p.project_vision.is_some());
        assert!(p.architecture.is_some());
        assert!(!p.domains.is_empty());
    }

    #[test]
    fn ml_pipeline_detected_from_pytorch() {
        let mut p = base_profile();
        p.languages = LanguageProfile {
            primary: Language::Python,
            secondary: Vec::new(),
            distribution: HashMap::new(),
            tree_sitter_coverage: 0.0,
        };
        p.frameworks.push(FrameworkDetection {
            name: "PyTorch".to_owned(),
            version: None,
            category: FrameworkCategory::ML,
            confidence: 1.0,
            detected_via: "pyproject.toml".to_owned(),
        });

        let domains = classify_domains(&p);
        assert!(domains.iter().any(|d| d.domain == ProjectDomain::MlPipeline));
    }
}
