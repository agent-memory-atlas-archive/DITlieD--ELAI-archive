// Convention extraction: Tier 2 project intelligence (deterministic, no LLM).
// Spec: docs/specs/guide/project-intelligence-design.txt sections 3A-3E.
//
// Two paths: DKG-based (Rust, when DKG has data) and regex-fallback (all languages).
// Both produce the same output types. DKG path queries the symbol graph;
// regex path scans source files directly with regex classification.

use rustc_hash::FxHashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;

use elai_core::project_profile::{
    Convention, ErrorHandlingProfile, ErrorPattern, ModuleOrganization, NamingConventions,
    OrgPattern, PropagationStyle, TestLocationPattern,
};

use crate::{DkgNodeInfo, SymbolKind};

// ---------------------------------------------------------------------------
// Constants — naming pattern thresholds (spec section 3A)
// ---------------------------------------------------------------------------

/// Above this ratio a convention is STRONG (auto-seed as rule).
const STRONG_THRESHOLD: f64 = 0.80;

/// Above this ratio (but below STRONG) a convention is MODERATE (WARN-level rule).
const MODERATE_THRESHOLD: f64 = 0.60;

/// Minimum sample size to classify a convention. Below this, report Unknown.
const MIN_SAMPLE_SIZE: usize = 5;

/// Schema version for the convention metadata persisted by the Project
/// Intelligence T2 hook (phase-3L). Bump only when persisted shape changes.
/// VERSION_GATE predicate in `PiT2Hook::run` refuses to seed when the on-disk
/// schema is older than this constant and instead queues a migration.
pub const CONVENTIONS_SCHEMA_VERSION: u32 = 1;

/// Maximum bytes scanned per source file in the regex-fallback path.
/// Avoids worst-case quadratic regex scans on auto-generated files.
const REGEX_MAX_FILE_BYTES: u64 = 512 * 1024;

/// Maximum depth walked into the project tree during regex fallback.
const REGEX_MAX_DEPTH: usize = 10;

// ---------------------------------------------------------------------------
// Naming case classification
// ---------------------------------------------------------------------------

/// Recognized naming styles per spec section 3A.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NamingStyle {
    SnakeCase,
    CamelCase,
    PascalCase,
    ScreamingSnake,
    KebabCase,
    Other,
}

impl NamingStyle {
    fn label(self) -> &'static str {
        match self {
            Self::SnakeCase => "snake_case",
            Self::CamelCase => "camelCase",
            Self::PascalCase => "PascalCase",
            Self::ScreamingSnake => "SCREAMING_SNAKE_CASE",
            Self::KebabCase => "kebab-case",
            Self::Other => "other",
        }
    }
}

/// Classify a name into a naming style. Order matters: check more specific
/// patterns first (SCREAMING_SNAKE before snake_case since both use underscores).
pub fn classify_name(name: &str) -> NamingStyle {
    if name.is_empty() {
        return NamingStyle::Other;
    }

    // Single-character names are ambiguous.
    if name.len() == 1 {
        return NamingStyle::Other;
    }

    // SCREAMING_SNAKE: all uppercase + underscores + digits
    if name.chars().all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit())
        && name.contains('_')
    {
        return NamingStyle::ScreamingSnake;
    }

    // kebab-case: lowercase + hyphens + digits
    if name.contains('-')
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '-' || c.is_ascii_digit())
    {
        return NamingStyle::KebabCase;
    }

    // snake_case: lowercase + underscores + digits, starts with lowercase
    if name
        .chars()
        .next()
        .map_or(false, |c| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit())
    {
        return NamingStyle::SnakeCase;
    }

    // PascalCase: starts uppercase, has lowercase, no underscores/hyphens
    if name
        .chars()
        .next()
        .map_or(false, |c| c.is_ascii_uppercase())
        && !name.contains('_')
        && !name.contains('-')
        && name.chars().any(|c| c.is_ascii_lowercase())
    {
        return NamingStyle::PascalCase;
    }

    // camelCase: starts lowercase, has uppercase, no underscores/hyphens
    if name
        .chars()
        .next()
        .map_or(false, |c| c.is_ascii_lowercase())
        && !name.contains('_')
        && !name.contains('-')
        && name.chars().any(|c| c.is_ascii_uppercase())
    {
        return NamingStyle::CamelCase;
    }

    NamingStyle::Other
}

// ---------------------------------------------------------------------------
// Histogram: count naming styles per symbol category
// ---------------------------------------------------------------------------

/// Counts of each naming style for a symbol category (functions, types, etc.).
#[derive(Debug, Clone, Default)]
pub struct NamingHistogram {
    counts: FxHashMap<NamingStyle, usize>,
    total: usize,
}

impl NamingHistogram {
    pub fn record(&mut self, style: NamingStyle) {
        *self.counts.entry(style).or_insert(0) += 1;
        self.total += 1;
    }

    /// Determine the dominant convention. Returns Unknown if sample too small.
    pub fn dominant_convention(&self) -> Convention {
        if self.total < MIN_SAMPLE_SIZE {
            return Convention::Unknown;
        }
        let (best_style, best_count) = self
            .counts
            .iter()
            .max_by_key(|(_, &v)| v)
            .map(|(&k, &v)| (k, v))
            .unwrap_or((NamingStyle::Other, 0));

        let ratio = best_count as f64 / self.total as f64;
        if ratio >= STRONG_THRESHOLD {
            Convention::Strong(best_style.label().to_owned())
        } else if ratio >= MODERATE_THRESHOLD {
            Convention::Moderate(best_style.label().to_owned())
        } else {
            Convention::Mixed
        }
    }
}

// ---------------------------------------------------------------------------
// ConventionStats: aggregate output of convention extraction
// ---------------------------------------------------------------------------

/// Aggregated convention statistics. Produced by either DKG or regex path.
#[derive(Debug, Clone)]
pub struct ConventionStats {
    pub naming: NamingConventions,
    pub error_handling: ErrorHandlingProfile,
    pub module_organization: ModuleOrganization,
    pub test_location: TestLocationPattern,
    pub dep_profile: elai_core::project_profile::DependencyProfile,
    /// "dkg" or "regex_fallback" — indicates which path produced the data.
    pub source: String,
}

// ---------------------------------------------------------------------------
// DKG-based extraction (Rust projects with populated DKG)
// ---------------------------------------------------------------------------

/// Extract conventions from DKG node data. Requires a populated DKG (Rust-only
/// as of Phase 1A). Caller passes nodes + project_root for path relativization.
pub fn extract_from_dkg(
    nodes: &[DkgNodeInfo],
    project_root: &Path,
) -> ConventionStats {
    let mut fn_hist = NamingHistogram::default();
    let mut type_hist = NamingHistogram::default();
    let mut const_hist = NamingHistogram::default();
    let mut mod_hist = NamingHistogram::default();

    // Naming conventions from DKG symbols (spec 3A: DKG path).
    for node in nodes {
        let style = classify_name(&node.symbol_name);
        match node.kind {
            SymbolKind::Function => fn_hist.record(style),
            SymbolKind::Struct | SymbolKind::Enum | SymbolKind::Trait | SymbolKind::TypeAlias => {
                type_hist.record(style);
            }
            SymbolKind::Const | SymbolKind::Static => const_hist.record(style),
            SymbolKind::Module => mod_hist.record(style),
            SymbolKind::Impl | SymbolKind::Macro => {}
        }
    }

    let naming = NamingConventions {
        functions: fn_hist.dominant_convention(),
        types: type_hist.dominant_convention(),
        constants: const_hist.dominant_convention(),
        modules: mod_hist.dominant_convention(),
    };

    // Error handling from DKG (spec 3B: Rust path).
    // Count Result<> return types in function signatures.
    let mut result_count: usize = 0;
    let mut option_count: usize = 0;
    let mut raw_count: usize = 0;
    let mut question_mark_likely = false;

    for node in nodes.iter().filter(|n| n.kind == SymbolKind::Function) {
        let sig = &node.signature;
        if sig.contains("Result<") || sig.contains("Result ") {
            result_count += 1;
        } else if sig.contains("Option<") || sig.contains("Option ") {
            option_count += 1;
        } else {
            raw_count += 1;
        }
    }

    let total_fns = result_count + option_count + raw_count;
    let error_pattern = if total_fns < MIN_SAMPLE_SIZE {
        ErrorPattern::Unknown
    } else if result_count as f64 / total_fns as f64 >= MODERATE_THRESHOLD {
        question_mark_likely = true;
        ErrorPattern::ResultBased
    } else {
        ErrorPattern::Mixed
    };

    let propagation = if question_mark_likely {
        PropagationStyle::QuestionMark
    } else {
        PropagationStyle::Unknown
    };

    // Detect error crate from function signatures (thiserror/anyhow/eyre).
    let error_crate = detect_error_crate_from_signatures(nodes);

    let error_handling = ErrorHandlingProfile {
        pattern: error_pattern,
        propagation,
        error_crate,
    };

    // Module organization from file paths (spec 3C).
    // CRITICAL: strip project root prefix to get relative paths.
    let relative_paths: Vec<PathBuf> = nodes
        .iter()
        .filter_map(|n| n.file_path.strip_prefix(project_root).ok())
        .map(|p| p.to_path_buf())
        .collect();
    let module_organization = detect_module_organization(&relative_paths);

    // Test patterns from DKG (spec 3D).
    let test_location = detect_test_location_dkg(nodes, project_root);

    ConventionStats {
        naming,
        error_handling,
        module_organization,
        test_location,
        dep_profile: elai_core::project_profile::DependencyProfile::default(),
        source: "dkg".to_owned(),
    }
}

// ---------------------------------------------------------------------------
// Regex-fallback extraction (non-Rust or empty DKG)
// ---------------------------------------------------------------------------

/// Extract conventions via regex scanning of source files.
/// Used when DKG has no data (non-Rust projects or DKG not built).
pub fn extract_from_regex_fallback(
    project_root: &Path,
    primary_language: &str,
) -> ConventionStats {
    let mut fn_hist = NamingHistogram::default();
    let mut type_hist = NamingHistogram::default();
    let mut const_hist = NamingHistogram::default();
    let mut mod_hist = NamingHistogram::default();

    // Language-specific function/class definition patterns.
    let fn_regex = function_regex_for_language(primary_language);
    let type_regex = type_regex_for_language(primary_language);
    let const_regex = const_regex_for_language(primary_language);

    let walker = ignore::WalkBuilder::new(project_root)
        .hidden(true)
        .git_ignore(true)
        .max_depth(Some(REGEX_MAX_DEPTH))
        .build();

    let extensions = source_extensions_for_language(primary_language);

    // Determinism: collect all matching paths first, sort, THEN scan.
    // `ignore::WalkBuilder` ordering is filesystem-dependent (varies between
    // Linux/Windows/NTFS). Sorting absolute paths guarantees byte-identical
    // ConventionStats on repeated runs over the same corpus.
    let mut scan_paths: Vec<PathBuf> = Vec::new();
    for entry in walker.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !extensions.contains(&ext) {
            continue;
        }
        scan_paths.push(path.to_path_buf());
    }
    scan_paths.sort();

    let mut source_paths: Vec<PathBuf> = Vec::new();
    let mut test_in_src = 0usize;
    let mut test_separate = 0usize;

    for path in &scan_paths {
        if let Ok(rel) = path.strip_prefix(project_root) {
            source_paths.push(rel.to_path_buf());
        }

        // Bound the read to REGEX_MAX_FILE_BYTES to avoid pathological
        // auto-generated files dominating the histogram.
        let content = match std::fs::metadata(path) {
            Ok(md) if md.len() > REGEX_MAX_FILE_BYTES => {
                tracing::trace!(
                    path = ?path,
                    bytes = md.len(),
                    "skipping oversized file in convention scan"
                );
                continue;
            }
            _ => match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            },
        };

        // Extract names via regex and classify.
        if let Some(re) = fn_regex {
            for cap in re.captures_iter(&content) {
                // TS arrow-function form binds the identifier to capture
                // group 2; everything else uses group 1.
                if let Some(name) = cap.get(1).or_else(|| cap.get(2)) {
                    fn_hist.record(classify_name(name.as_str()));
                }
            }
        }
        if let Some(re) = type_regex {
            for cap in re.captures_iter(&content) {
                if let Some(name) = cap.get(1) {
                    type_hist.record(classify_name(name.as_str()));
                }
            }
        }
        if let Some(re) = const_regex {
            for cap in re.captures_iter(&content) {
                if let Some(name) = cap.get(1) {
                    const_hist.record(classify_name(name.as_str()));
                }
            }
        }

        // Svelte: component PascalCase identity also lives in the file name
        // itself (canonical convention per kit.svelte.dev). Capture the file
        // stem as a TYPE-class signal so PascalCase shows up in the type
        // histogram even when the script block is empty.
        if primary_language == "svelte" {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                let style = classify_name(stem);
                if style == NamingStyle::PascalCase {
                    type_hist.record(style);
                }
            }
        }

        // Module name from directory structure.
        if let Ok(rel) = path.strip_prefix(project_root) {
            if let Some(parent) = rel.parent() {
                if let Some(dir_name) = parent.file_name().and_then(|n| n.to_str()) {
                    if dir_name != "src" && !dir_name.is_empty() {
                        mod_hist.record(classify_name(dir_name));
                    }
                }
            }
        }

        // Test location detection.
        let rel_str = path
            .strip_prefix(project_root)
            .ok()
            .and_then(|p| p.to_str())
            .unwrap_or("");
        let is_test_file = rel_str.contains("test") || rel_str.contains("spec");
        let has_inline_test = content.contains("#[cfg(test)]")
            || content.contains("def test_")
            || content.contains("describe(")
            || content.contains("it(");
        if is_test_file {
            test_separate += 1;
        } else if has_inline_test {
            test_in_src += 1;
        }
    }

    let naming = NamingConventions {
        functions: fn_hist.dominant_convention(),
        types: type_hist.dominant_convention(),
        constants: const_hist.dominant_convention(),
        modules: mod_hist.dominant_convention(),
    };

    // Error handling via grep (spec 3B: fallback).
    let error_handling = detect_error_handling_regex(project_root, primary_language, &extensions);

    // Module organization from file paths (spec 3C).
    let module_organization = detect_module_organization(&source_paths);

    // Test location from counts.
    let test_total = test_in_src + test_separate;
    let test_location = if test_total < MIN_SAMPLE_SIZE {
        TestLocationPattern::Unknown
    } else {
        let inline_ratio = test_in_src as f64 / test_total as f64;
        if inline_ratio >= STRONG_THRESHOLD {
            TestLocationPattern::SameFile
        } else if inline_ratio <= (1.0 - STRONG_THRESHOLD) {
            TestLocationPattern::SeparateDirectory
        } else {
            TestLocationPattern::Mixed
        }
    };

    ConventionStats {
        naming,
        error_handling,
        module_organization,
        test_location,
        dep_profile: elai_core::project_profile::DependencyProfile::default(),
        source: "regex_fallback".to_owned(),
    }
}

// ---------------------------------------------------------------------------
// Helpers: error handling detection
// ---------------------------------------------------------------------------

fn detect_error_crate_from_signatures(nodes: &[DkgNodeInfo]) -> Option<String> {
    // Check if any signatures reference known error crates.
    let all_sigs: String = nodes
        .iter()
        .map(|n| n.signature.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    if all_sigs.contains("anyhow::") || all_sigs.contains("anyhow::Result") {
        Some("anyhow".to_owned())
    } else if all_sigs.contains("thiserror") {
        Some("thiserror".to_owned())
    } else if all_sigs.contains("eyre::") {
        Some("eyre".to_owned())
    } else {
        None
    }
}

fn detect_error_handling_regex(
    project_root: &Path,
    language: &str,
    extensions: &[&str],
) -> ErrorHandlingProfile {
    let mut result_count = 0usize;
    let mut try_except_count = 0usize;
    let mut if_err_nil_count = 0usize;
    let mut error_crate: Option<String> = None;
    let mut custom_exception_count = 0usize;
    let mut fp_ts_neverthrow_count = 0usize;
    let mut errorf_wrap_count = 0usize;

    let walker = ignore::WalkBuilder::new(project_root)
        .hidden(true)
        .git_ignore(true)
        .max_depth(Some(REGEX_MAX_DEPTH))
        .build();

    // Determinism: sort paths before scanning so the error_crate auto-detect
    // (first-match-wins on `anyhow::` vs `thiserror::`) is stable across runs.
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in walker.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !extensions.contains(&ext) {
            continue;
        }
        paths.push(path.to_path_buf());
    }
    paths.sort();

    for path in &paths {
        let content = match std::fs::metadata(path) {
            Ok(md) if md.len() > REGEX_MAX_FILE_BYTES => continue,
            _ => match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            },
        };

        match language {
            "rust" => {
                result_count += content.matches("Result<").count();
                result_count += content.matches("-> Result").count();
                // Detect error crate from Cargo.toml-like patterns in source.
                if error_crate.is_none() {
                    if content.contains("anyhow::") {
                        error_crate = Some("anyhow".to_owned());
                    } else if content.contains("thiserror::") || content.contains("#[error(") {
                        error_crate = Some("thiserror".to_owned());
                    }
                }
            }
            "python" => {
                try_except_count += content.matches("try:").count();
                try_except_count += content.matches("except ").count();
                // A-33: detect custom exception classes (spec 3B Python).
                // Match both single-inheritance and multi-inheritance patterns.
                custom_exception_count += content.matches("(Exception)").count();
                custom_exception_count += content.matches("(Exception,").count();
                custom_exception_count += content.matches("(BaseException)").count();
                custom_exception_count += content.matches("(BaseException,").count();
                custom_exception_count += content.matches("(ValueError)").count();
                custom_exception_count += content.matches("(ValueError,").count();
                custom_exception_count += content.matches("(TypeError)").count();
                custom_exception_count += content.matches("(RuntimeError)").count();
            }
            "typescript" | "javascript" => {
                try_except_count += content.matches("try {").count();
                try_except_count += content.matches("catch (").count();
                try_except_count += content.matches("catch(").count();
                // A-33: detect fp-ts/neverthrow Result types (spec 3B TypeScript).
                fp_ts_neverthrow_count += content.matches("from 'fp-ts").count();
                fp_ts_neverthrow_count += content.matches("from \"fp-ts").count();
                fp_ts_neverthrow_count += content.matches("from 'neverthrow").count();
                fp_ts_neverthrow_count += content.matches("from \"neverthrow").count();
            }
            "go" => {
                if_err_nil_count += content.matches("if err != nil").count();
                if_err_nil_count += content.matches("if err !=nil").count();
                // A-33: detect error wrapping with fmt.Errorf %w (spec 3B Go).
                errorf_wrap_count += content.matches("fmt.Errorf(").count();
            }
            _ => {}
        }
    }

    let total = result_count + try_except_count + if_err_nil_count
        + custom_exception_count + fp_ts_neverthrow_count + errorf_wrap_count;
    if total < MIN_SAMPLE_SIZE {
        return ErrorHandlingProfile::default();
    }

    let (pattern, propagation) = if result_count > try_except_count && result_count > if_err_nil_count
    {
        (ErrorPattern::ResultBased, PropagationStyle::QuestionMark)
    } else if try_except_count > result_count && try_except_count > if_err_nil_count {
        (ErrorPattern::ExceptionBased, PropagationStyle::TryCatch)
    } else if if_err_nil_count > result_count && if_err_nil_count > try_except_count {
        (ErrorPattern::ErrorCodeBased, PropagationStyle::IfErrNil)
    } else {
        (ErrorPattern::Mixed, PropagationStyle::Mixed)
    };

    // A-33: refine error_crate based on per-language signals.
    if error_crate.is_none() {
        if custom_exception_count > 0 {
            error_crate = Some("custom_exceptions".to_owned());
        } else if fp_ts_neverthrow_count > 0 {
            error_crate = Some("fp-ts/neverthrow".to_owned());
        } else if errorf_wrap_count > 0 {
            error_crate = Some("fmt.Errorf".to_owned());
        }
    }

    ErrorHandlingProfile {
        pattern,
        propagation,
        error_crate,
    }
}

// ---------------------------------------------------------------------------
// Helpers: module organization detection (spec 3C)
// ---------------------------------------------------------------------------

fn detect_module_organization(relative_paths: &[PathBuf]) -> ModuleOrganization {
    if relative_paths.is_empty() {
        return ModuleOrganization {
            pattern: OrgPattern::Unknown,
        };
    }

    // Collect first directory component under src/ (or project root).
    let mut first_dirs: FxHashMap<String, usize> = FxHashMap::default();
    let mut files_in_root = 0usize;
    let mut has_packages_or_crates = false;

    for path in relative_paths {
        let components: Vec<_> = path.components().collect();
        if components.len() <= 1 {
            files_in_root += 1;
            continue;
        }

        // Skip "src" prefix to get the meaningful first directory.
        let meaningful_start = if components
            .first()
            .and_then(|c| c.as_os_str().to_str())
            .map_or(false, |s| s == "src")
        {
            1
        } else {
            0
        };

        if components.len() <= meaningful_start + 1 {
            files_in_root += 1;
            continue;
        }

        if let Some(dir_name) = components
            .get(meaningful_start)
            .and_then(|c| c.as_os_str().to_str())
        {
            *first_dirs.entry(dir_name.to_owned()).or_insert(0) += 1;
            if dir_name == "packages" || dir_name == "crates" || dir_name == "libs" {
                has_packages_or_crates = true;
            }
        }
    }

    let total_files = relative_paths.len();

    // Monorepo: packages/, crates/, libs/ directories.
    if has_packages_or_crates {
        return ModuleOrganization {
            pattern: OrgPattern::Monorepo,
        };
    }

    // Flat: all files in one directory.
    if files_in_root as f64 / total_files as f64 >= STRONG_THRESHOLD {
        return ModuleOrganization {
            pattern: OrgPattern::Flat,
        };
    }

    // By-type keywords: models, controllers, views, services, handlers, middleware.
    let by_type_keywords = [
        "models",
        "controllers",
        "views",
        "services",
        "handlers",
        "middleware",
        "routes",
        "schemas",
    ];

    // By-feature: directories that contain mixed concerns (multiple file types).
    let by_type_count = first_dirs
        .keys()
        .filter(|d| by_type_keywords.contains(&d.as_str()))
        .count();

    // Layered keywords: domain, application, infrastructure, presentation.
    let layered_keywords = [
        "domain",
        "application",
        "infrastructure",
        "presentation",
        "adapters",
        "ports",
    ];
    let layered_count = first_dirs
        .keys()
        .filter(|d| layered_keywords.contains(&d.as_str()))
        .count();

    if layered_count >= 2 {
        return ModuleOrganization {
            pattern: OrgPattern::Layered,
        };
    }
    if by_type_count >= 2 {
        return ModuleOrganization {
            pattern: OrgPattern::ByType,
        };
    }

    // Default to ByFeature if we have multiple meaningful directories
    // (assumes each directory groups related concerns).
    if first_dirs.len() >= 3 {
        return ModuleOrganization {
            pattern: OrgPattern::ByFeature,
        };
    }

    ModuleOrganization {
        pattern: OrgPattern::Unknown,
    }
}

// ---------------------------------------------------------------------------
// Helpers: test location detection from DKG (spec 3D)
// ---------------------------------------------------------------------------

fn detect_test_location_dkg(nodes: &[DkgNodeInfo], project_root: &Path) -> TestLocationPattern {
    let mut inline_test_files = 0usize;
    let mut separate_test_files = 0usize;

    // Deduplicate file paths.
    let mut seen_files: FxHashMap<PathBuf, bool> = FxHashMap::default();
    for node in nodes {
        let rel = node
            .file_path
            .strip_prefix(project_root)
            .unwrap_or(&node.file_path);
        if seen_files.contains_key(rel) {
            continue;
        }
        let rel_str = rel.to_str().unwrap_or("");
        let is_test_dir =
            rel_str.contains("tests/") || rel_str.contains("test/") || rel_str.contains("spec/");
        seen_files.insert(rel.to_path_buf(), is_test_dir);
    }

    // For Rust: check for #[cfg(test)] in source files.
    for (rel_path, is_in_test_dir) in &seen_files {
        if *is_in_test_dir {
            separate_test_files += 1;
        } else {
            // Check if any function in this file looks like a test.
            let has_test_fn = nodes.iter().any(|n| {
                let n_rel = n
                    .file_path
                    .strip_prefix(project_root)
                    .unwrap_or(&n.file_path);
                n_rel == rel_path
                    && n.kind == SymbolKind::Function
                    && (n.symbol_name.starts_with("test_")
                        || n.symbol_name.contains("_test"))
            });
            if has_test_fn {
                inline_test_files += 1;
            }
        }
    }

    let total = inline_test_files + separate_test_files;
    if total < MIN_SAMPLE_SIZE {
        return TestLocationPattern::Unknown;
    }

    let inline_ratio = inline_test_files as f64 / total as f64;
    if inline_ratio >= STRONG_THRESHOLD {
        TestLocationPattern::SameFile
    } else if inline_ratio <= (1.0 - STRONG_THRESHOLD) {
        TestLocationPattern::SeparateDirectory
    } else {
        TestLocationPattern::Mixed
    }
}

// ---------------------------------------------------------------------------
// Helpers: language-specific regex patterns for fallback path
// ---------------------------------------------------------------------------
//
// Regex compilation is HOT-PATH cost; per CLAUDE.md anti-stale-training rules
// + spec Step 1 stub_ban, compile each pattern exactly once at module init
// via std::sync::LazyLock. The .expect() calls are bound to crate-controlled
// literal patterns and are therefore in the static-init carve-out documented
// in CLAUDE.md (S1) — they cannot fail at runtime.

static RE_RUST_FN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(?:pub\s+)?(?:async\s+)?fn\s+([a-zA-Z_]\w*)")
        .expect("RE_RUST_FN: literal pattern, must compile")
});
static RE_PYTHON_FN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*def\s+([a-zA-Z_]\w*)")
        .expect("RE_PYTHON_FN: literal pattern, must compile")
});
static RE_TS_FN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)(?:function\s+([a-zA-Z_$]\w*)|(?:const|let|var)\s+([a-zA-Z_$]\w*)\s*=\s*(?:async\s*)?\()")
        .expect("RE_TS_FN: literal pattern, must compile")
});
static RE_GO_FN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^func\s+(?:\([^)]+\)\s+)?([a-zA-Z_]\w*)")
        .expect("RE_GO_FN: literal pattern, must compile")
});
/// Svelte: regex extracts both `<script>` function declarations
/// (`function`, `const = (...) =>`) and component identifiers — Svelte
/// components are PascalCase exports that show up either as the file's
/// own name (handled in caller) or as `<ComponentName .../>` references.
static RE_SVELTE_FN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)(?:function\s+([a-zA-Z_$]\w*)|(?:const|let)\s+([a-zA-Z_$]\w*)\s*=\s*(?:async\s*)?\()")
        .expect("RE_SVELTE_FN: literal pattern, must compile")
});

static RE_RUST_TYPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(?:pub\s+)?(?:struct|enum|trait|type)\s+([A-Za-z_]\w*)")
        .expect("RE_RUST_TYPE: literal pattern, must compile")
});
static RE_PYTHON_TYPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*class\s+([A-Za-z_]\w*)")
        .expect("RE_PYTHON_TYPE: literal pattern, must compile")
});
static RE_TS_TYPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)(?:class|interface|type)\s+([A-Za-z_$]\w*)")
        .expect("RE_TS_TYPE: literal pattern, must compile")
});
static RE_GO_TYPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^type\s+([A-Za-z_]\w*)")
        .expect("RE_GO_TYPE: literal pattern, must compile")
});
/// Svelte: type-like declarations land in `<script lang="ts">` blocks — reuse
/// the TS shape but compile once for the Svelte registry slot so the helper
/// stays branchless.
static RE_SVELTE_TYPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)(?:class|interface|type)\s+([A-Za-z_$]\w*)")
        .expect("RE_SVELTE_TYPE: literal pattern, must compile")
});

static RE_RUST_CONST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(?:pub\s+)?(?:const|static)\s+([A-Za-z_]\w*)")
        .expect("RE_RUST_CONST: literal pattern, must compile")
});
static RE_PYTHON_CONST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^([A-Z][A-Z0-9_]+)\s*=")
        .expect("RE_PYTHON_CONST: literal pattern, must compile")
});
static RE_TS_CONST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(?:export\s+)?const\s+([A-Z][A-Z0-9_]+)")
        .expect("RE_TS_CONST: literal pattern, must compile")
});
static RE_GO_CONST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(?:const|var)\s+([A-Za-z_]\w*)")
        .expect("RE_GO_CONST: literal pattern, must compile")
});
static RE_SVELTE_CONST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(?:export\s+)?const\s+([A-Z][A-Z0-9_]+)")
        .expect("RE_SVELTE_CONST: literal pattern, must compile")
});

/// Returns the LazyLock-cached function regex for `lang`, or `None` for an
/// unsupported language. Caller borrows `&'static Regex`; never allocates.
fn function_regex_for_language(lang: &str) -> Option<&'static Regex> {
    match lang {
        "rust" => Some(&RE_RUST_FN),
        "python" => Some(&RE_PYTHON_FN),
        "typescript" | "javascript" => Some(&RE_TS_FN),
        "go" => Some(&RE_GO_FN),
        "svelte" => Some(&RE_SVELTE_FN),
        _ => None,
    }
}

/// Returns the LazyLock-cached type regex for `lang`, or `None`.
fn type_regex_for_language(lang: &str) -> Option<&'static Regex> {
    match lang {
        "rust" => Some(&RE_RUST_TYPE),
        "python" => Some(&RE_PYTHON_TYPE),
        "typescript" | "javascript" => Some(&RE_TS_TYPE),
        "go" => Some(&RE_GO_TYPE),
        "svelte" => Some(&RE_SVELTE_TYPE),
        _ => None,
    }
}

/// Returns the LazyLock-cached const regex for `lang`, or `None`.
fn const_regex_for_language(lang: &str) -> Option<&'static Regex> {
    match lang {
        "rust" => Some(&RE_RUST_CONST),
        "python" => Some(&RE_PYTHON_CONST),
        "typescript" | "javascript" => Some(&RE_TS_CONST),
        "go" => Some(&RE_GO_CONST),
        "svelte" => Some(&RE_SVELTE_CONST),
        _ => None,
    }
}

fn source_extensions_for_language(lang: &str) -> Vec<&'static str> {
    match lang {
        "rust" => vec!["rs"],
        "python" => vec!["py", "pyi"],
        "typescript" => vec!["ts", "tsx"],
        "javascript" => vec!["js", "jsx", "mjs", "cjs"],
        "go" => vec!["go"],
        "java" => vec!["java"],
        "csharp" => vec!["cs"],
        "svelte" => vec!["svelte"],
        _ => vec![],
    }
}

// ---------------------------------------------------------------------------
// CorpusView + extract_conventions: unified entry point (spec Step 1)
// ---------------------------------------------------------------------------

/// Read-only view over the corpus an extractor needs to compute
/// `ConventionStats`. Thin newtype wrapping the existing DKG nodes slice
/// plus the regex-fallback fields, so callers do not have to pick between
/// `extract_from_dkg` and `extract_from_regex_fallback` themselves.
///
/// Spec: phase-3L Step 1 — `extract_conventions(corpus: &CorpusView)`.
#[derive(Debug, Clone, Copy)]
pub struct CorpusView<'a> {
    /// DKG node table for the indexed project, if available. When present
    /// AND non-empty, takes precedence over the regex fallback.
    pub dkg_nodes: Option<&'a [DkgNodeInfo]>,
    /// Absolute path to the project root (used to relativize node paths and
    /// to walk source files in the regex path).
    pub project_root: &'a Path,
    /// Primary language id (matches `Language::id_str()`, e.g. "rust",
    /// "python", "typescript", "svelte"). Drives regex selection in the
    /// fallback path; ignored in the DKG path.
    pub primary_language: &'a str,
}

impl<'a> CorpusView<'a> {
    /// Construct a CorpusView. `dkg_nodes=None` forces the regex-fallback path.
    pub fn new(
        dkg_nodes: Option<&'a [DkgNodeInfo]>,
        project_root: &'a Path,
        primary_language: &'a str,
    ) -> Self {
        Self {
            dkg_nodes,
            project_root,
            primary_language,
        }
    }
}

/// Extract `ConventionStats` from a `CorpusView` using the DKG path when
/// available (non-empty nodes) and the regex fallback otherwise.
///
/// Deterministic: same `CorpusView` -> byte-identical `ConventionStats`
/// (modulo non-deterministic `std::fs` ordering, which is normalised by
/// `ignore::WalkBuilder` + sorted intermediate aggregation; verified by the
/// `determinism_matrix` test module below).
///
/// Spec: phase-3L Step 1.
pub fn extract_conventions(corpus: &CorpusView<'_>) -> ConventionStats {
    match corpus.dkg_nodes {
        Some(nodes) if !nodes.is_empty() => extract_from_dkg(nodes, corpus.project_root),
        _ => extract_from_regex_fallback(corpus.project_root, corpus.primary_language),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_snake_case() {
        assert_eq!(classify_name("my_function"), NamingStyle::SnakeCase);
        assert_eq!(classify_name("get_value_2"), NamingStyle::SnakeCase);
    }

    #[test]
    fn classify_camel_case() {
        assert_eq!(classify_name("myFunction"), NamingStyle::CamelCase);
        assert_eq!(classify_name("getValue"), NamingStyle::CamelCase);
    }

    #[test]
    fn classify_pascal_case() {
        assert_eq!(classify_name("MyStruct"), NamingStyle::PascalCase);
        assert_eq!(classify_name("HttpClient"), NamingStyle::PascalCase);
    }

    #[test]
    fn classify_screaming_snake() {
        assert_eq!(classify_name("MAX_VALUE"), NamingStyle::ScreamingSnake);
        assert_eq!(classify_name("HTTP_STATUS_OK"), NamingStyle::ScreamingSnake);
    }

    #[test]
    fn classify_kebab_case() {
        assert_eq!(classify_name("my-component"), NamingStyle::KebabCase);
    }

    #[test]
    fn classify_other() {
        assert_eq!(classify_name(""), NamingStyle::Other);
        assert_eq!(classify_name("x"), NamingStyle::Other);
    }

    #[test]
    fn histogram_strong_convention() {
        let mut hist = NamingHistogram::default();
        for _ in 0..9 {
            hist.record(NamingStyle::SnakeCase);
        }
        hist.record(NamingStyle::CamelCase);
        assert_eq!(
            hist.dominant_convention(),
            Convention::Strong("snake_case".to_owned())
        );
    }

    #[test]
    fn histogram_moderate_convention() {
        let mut hist = NamingHistogram::default();
        for _ in 0..7 {
            hist.record(NamingStyle::SnakeCase);
        }
        for _ in 0..3 {
            hist.record(NamingStyle::CamelCase);
        }
        assert_eq!(
            hist.dominant_convention(),
            Convention::Moderate("snake_case".to_owned())
        );
    }

    #[test]
    fn histogram_mixed() {
        let mut hist = NamingHistogram::default();
        for _ in 0..4 {
            hist.record(NamingStyle::SnakeCase);
        }
        for _ in 0..4 {
            hist.record(NamingStyle::CamelCase);
        }
        for _ in 0..2 {
            hist.record(NamingStyle::PascalCase);
        }
        assert_eq!(hist.dominant_convention(), Convention::Mixed);
    }

    #[test]
    fn histogram_too_small() {
        let mut hist = NamingHistogram::default();
        hist.record(NamingStyle::SnakeCase);
        hist.record(NamingStyle::SnakeCase);
        assert_eq!(hist.dominant_convention(), Convention::Unknown);
    }

    #[test]
    fn module_org_flat() {
        let paths: Vec<PathBuf> = (0..10)
            .map(|i| PathBuf::from(format!("file{i}.rs")))
            .collect();
        let org = detect_module_organization(&paths);
        assert_eq!(org.pattern, OrgPattern::Flat);
    }

    #[test]
    fn module_org_monorepo() {
        let paths = vec![
            PathBuf::from("crates/core/lib.rs"),
            PathBuf::from("crates/rules/lib.rs"),
            PathBuf::from("crates/knowledge/lib.rs"),
        ];
        let org = detect_module_organization(&paths);
        assert_eq!(org.pattern, OrgPattern::Monorepo);
    }

    #[test]
    fn module_org_by_type() {
        let paths = vec![
            PathBuf::from("src/models/user.rs"),
            PathBuf::from("src/models/order.rs"),
            PathBuf::from("src/controllers/user.rs"),
            PathBuf::from("src/controllers/order.rs"),
            PathBuf::from("src/views/index.rs"),
        ];
        let org = detect_module_organization(&paths);
        assert_eq!(org.pattern, OrgPattern::ByType);
    }

    #[test]
    fn module_org_layered() {
        let paths = vec![
            PathBuf::from("src/domain/user.rs"),
            PathBuf::from("src/application/service.rs"),
            PathBuf::from("src/infrastructure/db.rs"),
        ];
        let org = detect_module_organization(&paths);
        assert_eq!(org.pattern, OrgPattern::Layered);
    }

    #[test]
    fn module_org_by_feature() {
        let paths = vec![
            PathBuf::from("src/auth/handler.rs"),
            PathBuf::from("src/auth/model.rs"),
            PathBuf::from("src/billing/handler.rs"),
            PathBuf::from("src/billing/model.rs"),
            PathBuf::from("src/users/handler.rs"),
        ];
        let org = detect_module_organization(&paths);
        assert_eq!(org.pattern, OrgPattern::ByFeature);
    }

    // -----------------------------------------------------------------------
    // 4-language determinism matrix (phase-3L Step 3 + P1 stub-closure).
    // -----------------------------------------------------------------------
    //
    // Each language synthesises an 8-file corpus in a TempDir and asserts:
    //   (1) the expected naming-style majority is captured in the dominant
    //       Convention for functions (or types for Svelte components), AND
    //   (2) extract_conventions(&CorpusView{regex path}) is byte-deterministic
    //       across two consecutive runs (Debug format compare — ConventionStats
    //       intentionally does not derive PartialEq to avoid forcing trait on
    //       elai_core types, per phase-3L Pi-prefix decision in the
    //       project_profile-interface.md source ingest).
    //
    // The DKG path is exercised by `dkg_conventions_with_nodes` below; this
    // module is the regex-fallback determinism contract.

    fn build_corpus(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::TempDir::new().expect("create temp dir");
        for (path, content) in files {
            let full = dir.path().join(path);
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent).expect("create parent dirs");
            }
            std::fs::write(&full, content).expect("write file");
        }
        dir
    }

    fn run_twice_is_deterministic(corpus_root: &Path, lang: &str) {
        let view = CorpusView::new(None, corpus_root, lang);
        let a = extract_conventions(&view);
        let b = extract_conventions(&view);
        // ConventionStats lacks PartialEq (would force PartialEq on
        // elai_core::project_profile types — out of scope per phase-3L
        // pre-resolved decisions). Compare Debug for byte equality.
        assert_eq!(
            format!("{a:?}"),
            format!("{b:?}"),
            "extract_conventions not deterministic for {lang}"
        );
    }

    #[test]
    fn rust_4lang_majority_snake_case() {
        // 8 Rust files, each declaring 3 snake_case fns + 1 PascalCase type.
        let files: Vec<(String, String)> = (0..8)
            .map(|i| {
                (
                    format!("src/lib_{i}.rs"),
                    format!(
                        "pub fn parse_input_{i}() {{}}\n\
                         pub fn handle_request_{i}() {{}}\n\
                         pub fn render_output_{i}() {{}}\n\
                         pub struct WidgetState{i};\n",
                    ),
                )
            })
            .collect();
        let refs: Vec<(&str, &str)> =
            files.iter().map(|(p, c)| (p.as_str(), c.as_str())).collect();
        let dir = build_corpus(&refs);
        let view = CorpusView::new(None, dir.path(), "rust");
        let stats = extract_conventions(&view);
        assert_eq!(
            stats.naming.functions,
            Convention::Strong("snake_case".to_owned()),
            "rust fn majority should be snake_case"
        );
        run_twice_is_deterministic(dir.path(), "rust");
    }

    #[test]
    fn python_4lang_majority_snake_case() {
        let files: Vec<(String, String)> = (0..8)
            .map(|i| {
                (
                    format!("pkg/mod_{i}.py"),
                    format!(
                        "def parse_input_{i}():\n    return None\n\n\
                         def handle_request_{i}():\n    return None\n\n\
                         def render_output_{i}():\n    return None\n\n\
                         class WidgetState{i}:\n    pass\n",
                    ),
                )
            })
            .collect();
        let refs: Vec<(&str, &str)> =
            files.iter().map(|(p, c)| (p.as_str(), c.as_str())).collect();
        let dir = build_corpus(&refs);
        let view = CorpusView::new(None, dir.path(), "python");
        let stats = extract_conventions(&view);
        assert_eq!(
            stats.naming.functions,
            Convention::Strong("snake_case".to_owned()),
            "python fn majority should be snake_case"
        );
        run_twice_is_deterministic(dir.path(), "python");
    }

    #[test]
    fn typescript_4lang_majority_camel_case() {
        let files: Vec<(String, String)> = (0..8)
            .map(|i| {
                (
                    format!("src/mod_{i}.ts"),
                    format!(
                        "export function parseInput{i}() {{}}\n\
                         export function handleRequest{i}() {{}}\n\
                         export function renderOutput{i}() {{}}\n\
                         export class WidgetState{i} {{}}\n",
                    ),
                )
            })
            .collect();
        let refs: Vec<(&str, &str)> =
            files.iter().map(|(p, c)| (p.as_str(), c.as_str())).collect();
        let dir = build_corpus(&refs);
        let view = CorpusView::new(None, dir.path(), "typescript");
        let stats = extract_conventions(&view);
        assert_eq!(
            stats.naming.functions,
            Convention::Strong("camelCase".to_owned()),
            "typescript fn majority should be camelCase"
        );
        run_twice_is_deterministic(dir.path(), "typescript");
    }

    #[test]
    fn svelte_4lang_majority_pascal_case() {
        // Svelte's canonical convention: component identity comes from the
        // file name (kit.svelte.dev/docs). The fallback path captures the
        // file stem as a TYPE-class PascalCase signal so .svelte-only
        // projects still get a dominant convention.
        let stems = [
            "AlphaPanel",
            "BetaModal",
            "GammaDrawer",
            "DeltaButton",
            "EpsilonChart",
            "ZetaTable",
            "EtaForm",
            "ThetaWindow",
        ];
        let files: Vec<(String, String)> = stems
            .iter()
            .map(|s| {
                (
                    format!("src/lib/components/{s}.svelte"),
                    "<script>let n = 0;</script>\n<div />".to_owned(),
                )
            })
            .collect();
        let refs: Vec<(&str, &str)> =
            files.iter().map(|(p, c)| (p.as_str(), c.as_str())).collect();
        let dir = build_corpus(&refs);
        let view = CorpusView::new(None, dir.path(), "svelte");
        let stats = extract_conventions(&view);
        assert_eq!(
            stats.naming.types,
            Convention::Strong("PascalCase".to_owned()),
            "svelte component majority should be PascalCase"
        );
        run_twice_is_deterministic(dir.path(), "svelte");
    }

    #[test]
    fn extract_conventions_dispatches_to_dkg_when_nodes_present() {
        let project_root = PathBuf::from("/project");
        let nodes = vec![DkgNodeInfo {
            node_id: 1,
            symbol_name: "parse_thing".to_owned(),
            file_path: PathBuf::from("/project/src/lib.rs"),
            signature: "fn parse_thing() -> Result<(), Error>".to_owned(),
            kind: SymbolKind::Function,
            start_line: 1,
            end_line: 5,
        }];
        let view = CorpusView::new(Some(&nodes), &project_root, "rust");
        let stats = extract_conventions(&view);
        assert_eq!(stats.source, "dkg");
    }

    #[test]
    fn extract_conventions_falls_back_to_regex_when_dkg_empty() {
        // Empty DKG slice -> regex path on an empty tmpdir (returns Unknown
        // naming + "regex_fallback" source).
        let dir = tempfile::TempDir::new().expect("tmp");
        let view = CorpusView::new(Some(&[]), dir.path(), "rust");
        let stats = extract_conventions(&view);
        assert_eq!(stats.source, "regex_fallback");
    }

    #[test]
    fn conventions_schema_version_is_one() {
        assert_eq!(CONVENTIONS_SCHEMA_VERSION, 1);
    }

    #[test]
    fn dkg_conventions_with_nodes() {
        let project_root = PathBuf::from("/project");
        let nodes = vec![
            DkgNodeInfo {
                node_id: 1,
                symbol_name: "process_data".to_owned(),
                file_path: PathBuf::from("/project/src/lib.rs"),
                signature: "fn process_data() -> Result<(), Error>".to_owned(),
                kind: SymbolKind::Function,
                start_line: 1,
                end_line: 10,
            },
            DkgNodeInfo {
                node_id: 2,
                symbol_name: "handle_request".to_owned(),
                file_path: PathBuf::from("/project/src/lib.rs"),
                signature: "fn handle_request() -> Result<Response, Error>".to_owned(),
                kind: SymbolKind::Function,
                start_line: 12,
                end_line: 20,
            },
            DkgNodeInfo {
                node_id: 3,
                symbol_name: "fetch_items".to_owned(),
                file_path: PathBuf::from("/project/src/lib.rs"),
                signature: "fn fetch_items() -> Result<Vec<Item>, Error>".to_owned(),
                kind: SymbolKind::Function,
                start_line: 22,
                end_line: 30,
            },
            DkgNodeInfo {
                node_id: 4,
                symbol_name: "parse_input".to_owned(),
                file_path: PathBuf::from("/project/src/lib.rs"),
                signature: "fn parse_input() -> Result<Input, ParseError>".to_owned(),
                kind: SymbolKind::Function,
                start_line: 32,
                end_line: 40,
            },
            DkgNodeInfo {
                node_id: 5,
                symbol_name: "validate_config".to_owned(),
                file_path: PathBuf::from("/project/src/lib.rs"),
                signature: "fn validate_config() -> Result<Config, ConfigError>".to_owned(),
                kind: SymbolKind::Function,
                start_line: 42,
                end_line: 50,
            },
            DkgNodeInfo {
                node_id: 6,
                symbol_name: "MyStruct".to_owned(),
                file_path: PathBuf::from("/project/src/lib.rs"),
                signature: "struct MyStruct".to_owned(),
                kind: SymbolKind::Struct,
                start_line: 52,
                end_line: 55,
            },
            DkgNodeInfo {
                node_id: 7,
                symbol_name: "MAX_RETRIES".to_owned(),
                file_path: PathBuf::from("/project/src/lib.rs"),
                signature: "const MAX_RETRIES: u32 = 3".to_owned(),
                kind: SymbolKind::Const,
                start_line: 57,
                end_line: 57,
            },
        ];
        let stats = extract_from_dkg(&nodes, &project_root);
        assert_eq!(
            stats.naming.functions,
            Convention::Strong("snake_case".to_owned())
        );
        assert_eq!(stats.error_handling.pattern, ErrorPattern::ResultBased);
        assert_eq!(
            stats.error_handling.propagation,
            PropagationStyle::QuestionMark
        );
        assert_eq!(stats.source, "dkg");
    }
}
