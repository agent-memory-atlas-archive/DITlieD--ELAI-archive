// Project Intelligence: Tier 1 structural discovery (pure filesystem analysis).
// Spec: docs/specs/guide/project-intelligence-design.txt sections 2A-2G, 5.
//
// No LLM, no DKG, no network. Pattern matching on known config files.
// If the signal is not present, the field is Unknown.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use elai_core::project_profile::{
    BuildCommand, BuildCommandRegistry, BuildSystem, BuildSystemProfile, CiProfile, CiSystem,
    CommandScope, EntryPoint, EntryPointKind, FrameworkCategory, FrameworkDetection, Language,
    LanguageProfile, LintProfile, ProjectProfile, ProjectProfileError, TestLocationPattern,
    TestProfile, WorkspaceMember, DEFAULT_COMMAND_TIMEOUT_S, PROFILE_SCHEMA_VERSION,
};

use ignore::WalkBuilder;
use tracing::debug;

// ---------------------------------------------------------------------------
// Build system detection table (spec section 2A)
// ---------------------------------------------------------------------------

/// Config file -> (BuildSystem, build_cmd, test_cmd, lint_cmd, format_cmd, clean_cmd).
/// Priority is encoded in order: language-native first, then package managers,
/// task runners, generic last.
const BUILD_SYSTEM_TABLE: &[(&str, BuildSystem, &str, &str, &str, &str, &str)] = &[
    // Language-native (priority 1)
    ("Cargo.toml",           BuildSystem::Cargo,     "cargo build",            "cargo test",                  "cargo clippy",           "cargo fmt",       "cargo clean"),
    ("go.mod",               BuildSystem::GoMod,     "go build ./...",         "go test ./...",               "golangci-lint run",      "gofmt -w .",      "go clean"),
    ("mix.exs",              BuildSystem::Mix,       "mix compile",            "mix test",                    "mix credo",              "mix format",      ""),
    ("build.zig",            BuildSystem::Zig,       "zig build",              "zig build test",              "",                       "",                ""),
    // Package managers (priority 2)
    ("pyproject.toml",       BuildSystem::PyProject, "",                       "pytest",                      "ruff check",             "ruff format",     ""),
    ("setup.py",             BuildSystem::SetupPy,   "python setup.py build",  "pytest",                     "",                       "",                ""),
    ("composer.json",        BuildSystem::Composer,  "composer install",       "",                            "",                       "",                ""),
    ("Gemfile",              BuildSystem::Bundler,   "bundle exec rake build", "bundle exec rspec",           "bundle exec rubocop",    "",                ""),
    ("deno.json",            BuildSystem::Deno,      "deno compile",           "deno test",                   "deno lint",              "deno fmt",        ""),
    ("deno.jsonc",           BuildSystem::Deno,      "deno compile",           "deno test",                   "deno lint",              "deno fmt",        ""),
    // Task runners (priority 3)
    ("justfile",             BuildSystem::Just,      "just build",             "just test",                   "just lint",              "just format",     "just clean"),
    ("Taskfile.yml",         BuildSystem::Task,      "task build",             "task test",                   "task lint",              "task format",     "task clean"),
    // Generic (priority 4)
    ("Makefile",             BuildSystem::Make,      "make",                   "make test",                   "make lint",              "make format",     "make clean"),
    ("GNUmakefile",          BuildSystem::Make,      "make",                   "make test",                   "make lint",              "make format",     "make clean"),
    ("CMakeLists.txt",       BuildSystem::CMake,     "cmake --build build",    "ctest --test-dir build",      "",                       "",                "cmake --build build --target clean"),
    ("meson.build",          BuildSystem::Meson,     "meson compile -C build", "meson test -C build",         "",                       "",                ""),
];

/// Extra entries that need content reading (package.json, build.gradle, pom.xml, .csproj, bun).
/// These are checked separately because commands depend on file contents.
const CONTENT_BASED_SYSTEMS: &[(&str, BuildSystem)] = &[
    ("package.json",      BuildSystem::Npm),
    ("build.gradle",      BuildSystem::Gradle),
    ("build.gradle.kts",  BuildSystem::Gradle),
    ("pom.xml",           BuildSystem::Maven),
];

// ---------------------------------------------------------------------------
// Language extension map (spec section 2B)
// ---------------------------------------------------------------------------

fn extension_to_language(ext: &str) -> Option<Language> {
    match ext {
        "rs"          => Some(Language::Rust),
        "py" | "pyi"  => Some(Language::Python),
        "ts" | "tsx"  => Some(Language::TypeScript),
        "js" | "jsx" | "mjs" | "cjs" => Some(Language::JavaScript),
        "go"          => Some(Language::Go),
        "java"        => Some(Language::Java),
        "kt" | "kts"  => Some(Language::Kotlin),
        "cs"          => Some(Language::CSharp),
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Some(Language::Cpp),
        "c" | "h"     => Some(Language::C),
        "rb"          => Some(Language::Ruby),
        "ex" | "exs"  => Some(Language::Elixir),
        "php"         => Some(Language::PHP),
        "swift"       => Some(Language::Swift),
        "zig"         => Some(Language::Zig),
        "dart"        => Some(Language::Dart),
        "lua"         => Some(Language::Lua),
        "svelte"      => Some(Language::Svelte),
        "vue"         => Some(Language::Vue),
        "hs"          => Some(Language::Haskell),
        "scala" | "sc" => Some(Language::Scala),
        "clj" | "cljs" | "cljc" => Some(Language::Clojure),
        "ml" | "mli"  => Some(Language::OCaml),
        "gleam"       => Some(Language::Gleam),
        _             => None,
    }
}

// ---------------------------------------------------------------------------
// Framework detection table (spec section 2C)
// ---------------------------------------------------------------------------

struct FrameworkRule {
    #[allow(dead_code)]
    language: Language,
    dep_file: &'static str,
    dep_key: &'static str,
    name: &'static str,
    category: FrameworkCategory,
}

const FRAMEWORK_RULES: &[FrameworkRule] = &[
    FrameworkRule { language: Language::Rust,       dep_file: "Cargo.toml",      dep_key: "axum",              name: "Axum",        category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Rust,       dep_file: "Cargo.toml",      dep_key: "actix-web",         name: "Actix",       category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Rust,       dep_file: "Cargo.toml",      dep_key: "bevy",              name: "Bevy",        category: FrameworkCategory::Game },
    FrameworkRule { language: Language::Rust,       dep_file: "Cargo.toml",      dep_key: "leptos",            name: "Leptos",      category: FrameworkCategory::Fullstack },
    FrameworkRule { language: Language::Rust,       dep_file: "Cargo.toml",      dep_key: "tauri",             name: "Tauri",       category: FrameworkCategory::Desktop },
    FrameworkRule { language: Language::Python,     dep_file: "pyproject.toml",  dep_key: "fastapi",           name: "FastAPI",     category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Python,     dep_file: "pyproject.toml",  dep_key: "django",            name: "Django",      category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Python,     dep_file: "pyproject.toml",  dep_key: "flask",             name: "Flask",       category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Python,     dep_file: "pyproject.toml",  dep_key: "torch",             name: "PyTorch",     category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python,     dep_file: "pyproject.toml",  dep_key: "tensorflow",        name: "TensorFlow",  category: FrameworkCategory::ML },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json",    dep_key: "next",              name: "Next.js",     category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json",    dep_key: "svelte",            name: "Svelte",      category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json",    dep_key: "react",             name: "React",       category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json",    dep_key: "vue",               name: "Vue",         category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json",    dep_key: "express",           name: "Express",     category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Go,         dep_file: "go.mod",          dep_key: "gin-gonic/gin",     name: "Gin",         category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Go,         dep_file: "go.mod",          dep_key: "gofiber/fiber",     name: "Fiber",       category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Ruby,       dep_file: "Gemfile",         dep_key: "rails",             name: "Rails",       category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Elixir,     dep_file: "mix.exs",         dep_key: "phoenix",           name: "Phoenix",     category: FrameworkCategory::Web },
];

// ---------------------------------------------------------------------------
// Lint/formatter config detection (spec section 2E)
// ---------------------------------------------------------------------------

struct LintRule {
    config_file: &'static str,
    tool: &'static str,
    language: Language,
    auto_fix: bool,
}

const LINT_RULES: &[LintRule] = &[
    LintRule { config_file: "rustfmt.toml",            tool: "rustfmt",        language: Language::Rust,       auto_fix: true },
    LintRule { config_file: ".rustfmt.toml",           tool: "rustfmt",        language: Language::Rust,       auto_fix: true },
    LintRule { config_file: "clippy.toml",             tool: "clippy",         language: Language::Rust,       auto_fix: false },
    LintRule { config_file: ".eslintrc.json",          tool: "ESLint",         language: Language::TypeScript, auto_fix: true },
    LintRule { config_file: ".eslintrc.js",            tool: "ESLint",         language: Language::TypeScript, auto_fix: true },
    LintRule { config_file: ".eslintrc.yml",           tool: "ESLint",         language: Language::TypeScript, auto_fix: true },
    LintRule { config_file: "eslint.config.js",        tool: "ESLint",         language: Language::TypeScript, auto_fix: true },
    LintRule { config_file: "eslint.config.mjs",       tool: "ESLint",         language: Language::TypeScript, auto_fix: true },
    LintRule { config_file: ".prettierrc",             tool: "Prettier",       language: Language::TypeScript, auto_fix: true },
    LintRule { config_file: ".prettierrc.json",        tool: "Prettier",       language: Language::TypeScript, auto_fix: true },
    LintRule { config_file: "prettier.config.js",      tool: "Prettier",       language: Language::TypeScript, auto_fix: true },
    LintRule { config_file: "biome.json",              tool: "Biome",          language: Language::TypeScript, auto_fix: true },
    LintRule { config_file: "ruff.toml",               tool: "Ruff",           language: Language::Python,     auto_fix: true },
    LintRule { config_file: ".ruff.toml",              tool: "Ruff",           language: Language::Python,     auto_fix: true },
    LintRule { config_file: ".flake8",                 tool: "Flake8",         language: Language::Python,     auto_fix: false },
    LintRule { config_file: ".golangci.yml",           tool: "golangci-lint",  language: Language::Go,         auto_fix: false },
    LintRule { config_file: ".clang-format",           tool: "clang-format",   language: Language::Cpp,        auto_fix: true },
    LintRule { config_file: ".editorconfig",           tool: "EditorConfig",   language: Language::Unknown,    auto_fix: false },
];

// ---------------------------------------------------------------------------
// CI detection (spec section 2G)
// ---------------------------------------------------------------------------

struct CiRule {
    path: &'static str,
    system: CiSystem,
    is_glob: bool,
}

const CI_RULES: &[CiRule] = &[
    CiRule { path: ".github/workflows", system: CiSystem::GitHubActions,       is_glob: true },
    CiRule { path: ".gitlab-ci.yml",    system: CiSystem::GitLabCi,            is_glob: false },
    CiRule { path: "Jenkinsfile",       system: CiSystem::Jenkins,             is_glob: false },
    CiRule { path: ".circleci",         system: CiSystem::CircleCi,            is_glob: false },
    CiRule { path: "bitbucket-pipelines.yml", system: CiSystem::BitbucketPipelines, is_glob: false },
    CiRule { path: ".travis.yml",       system: CiSystem::TravisCi,            is_glob: false },
];

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Detect project profile from filesystem. Tier 1 only (no LLM, no DKG).
pub fn detect_project_profile(project_root: &Path) -> Result<ProjectProfile, ProjectProfileError> {
    if !project_root.exists() {
        return Err(ProjectProfileError::RootNotFound(project_root.to_path_buf()));
    }

    debug!(root = %project_root.display(), "starting Tier 1 project detection");

    let build_system = detect_build_system(project_root)?;
    let languages = detect_languages(project_root);
    let frameworks = detect_frameworks(project_root);
    let test_profile = detect_test_profile(project_root, &build_system);
    let lint_tools = detect_lint_tools(project_root);
    let entry_points = detect_entry_points(project_root, &languages);
    let ci = detect_ci(project_root);
    let commands = build_command_registry(project_root, &build_system);

    let generated_at = {
        let now = std::time::SystemTime::now();
        let duration = now
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        // ISO 8601 without external dep: use seconds since epoch as fallback
        format!("{}", duration.as_secs())
    };

    Ok(ProjectProfile {
        schema_version: PROFILE_SCHEMA_VERSION,
        generated_at,
        project_root: project_root.to_path_buf(),
        build_system,
        languages,
        frameworks,
        test_profile,
        lint_tools,
        entry_points,
        ci,
        naming: Default::default(),
        error_handling: Default::default(),
        module_organization: Default::default(),
        project_vision: None,
        architecture: None,
        domains: Vec::new(),
        commands,
        overrides_applied: false,
    })
}

// ---------------------------------------------------------------------------
// Build system detection (spec section 2A)
// ---------------------------------------------------------------------------

fn detect_build_system(root: &Path) -> Result<BuildSystemProfile, ProjectProfileError> {
    // Check language-native first (priority order encoded in table)
    for &(config_file, ref system, _, _, _, _, _) in BUILD_SYSTEM_TABLE {
        let config_path = root.join(config_file);
        if config_path.exists() {
            debug!(system = ?system, config = %config_path.display(), "detected build system");
            let (monorepo, workspaces) = detect_workspaces(root, system, &config_path)?;
            let version = detect_build_version(root, system);
            return Ok(BuildSystemProfile {
                system: system.clone(),
                config_path: PathBuf::from(config_file),
                version,
                monorepo,
                workspaces,
            });
        }
    }

    // Check content-based systems
    for &(config_file, ref system) in CONTENT_BASED_SYSTEMS {
        let config_path = root.join(config_file);
        if config_path.exists() {
            debug!(system = ?system, config = %config_path.display(), "detected content-based build system");
            let (monorepo, workspaces) = detect_workspaces(root, system, &config_path)?;
            return Ok(BuildSystemProfile {
                system: system.clone(),
                config_path: PathBuf::from(config_file),
                version: None,
                monorepo,
                workspaces,
            });
        }
    }

    // Check for .csproj / .sln files
    let has_csproj = std::fs::read_dir(root)
        .ok()
        .map(|entries| {
            entries.filter_map(|e| e.ok()).any(|e| {
                let name = e.file_name();
                let s = name.to_string_lossy();
                s.ends_with(".csproj") || s.ends_with(".sln")
            })
        })
        .unwrap_or(false);

    if has_csproj {
        return Ok(BuildSystemProfile {
            system: BuildSystem::DotNet,
            config_path: PathBuf::from("."),
            version: None,
            monorepo: false,
            workspaces: Vec::new(),
        });
    }

    // Check for bun (bunfig.toml or bun.lockb)
    if root.join("bunfig.toml").exists() || root.join("bun.lockb").exists() {
        return Ok(BuildSystemProfile {
            system: BuildSystem::Bun,
            config_path: if root.join("bunfig.toml").exists() {
                PathBuf::from("bunfig.toml")
            } else {
                PathBuf::from("bun.lockb")
            },
            version: None,
            monorepo: false,
            workspaces: Vec::new(),
        });
    }

    Ok(BuildSystemProfile::default())
}

fn detect_workspaces(
    _root: &Path,
    system: &BuildSystem,
    config_path: &Path,
) -> Result<(bool, Vec<WorkspaceMember>), ProjectProfileError> {
    match system {
        BuildSystem::Cargo => {
            let content = std::fs::read_to_string(config_path)?;
            if let Ok(parsed) = content.parse::<toml::Table>() {
                if let Some(ws) = parsed.get("workspace") {
                    let members = ws
                        .get("members")
                        .and_then(|m| m.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str())
                                .map(|s| WorkspaceMember {
                                    name: s.to_string(),
                                    path: PathBuf::from(s),
                                })
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    return Ok((true, members));
                }
            }
            Ok((false, Vec::new()))
        }
        BuildSystem::Npm => {
            let content = std::fs::read_to_string(config_path)?;
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(workspaces) = parsed.get("workspaces").and_then(|w| w.as_array()) {
                    let members = workspaces
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| WorkspaceMember {
                            name: s.to_string(),
                            path: PathBuf::from(s),
                        })
                        .collect();
                    return Ok((true, members));
                }
            }
            Ok((false, Vec::new()))
        }
        _ => Ok((false, Vec::new())),
    }
}

fn detect_build_version(root: &Path, system: &BuildSystem) -> Option<String> {
    match system {
        BuildSystem::Cargo => {
            let content = std::fs::read_to_string(root.join("Cargo.toml")).ok()?;
            let parsed: toml::Table = content.parse().ok()?;
            // Check [package].edition or [workspace.package].edition
            parsed
                .get("package")
                .or_else(|| {
                    parsed
                        .get("workspace")
                        .and_then(|ws| ws.get("package"))
                })
                .and_then(|pkg| pkg.get("edition"))
                .and_then(|e| e.as_str())
                .map(|s| format!("edition {s}"))
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Language detection (spec section 2B)
// ---------------------------------------------------------------------------

fn detect_languages(root: &Path) -> LanguageProfile {
    let mut distribution: HashMap<Language, usize> = HashMap::new();

    let walker = WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .git_global(false)
        .git_exclude(true)
        .build();

    for entry in walker.flatten() {
        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
            continue;
        }
        if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
            if let Some(lang) = extension_to_language(ext) {
                *distribution.entry(lang).or_insert(0) += 1;
            }
        }
    }

    if distribution.is_empty() {
        return LanguageProfile::default();
    }

    // Primary = language with most files
    let mut sorted: Vec<_> = distribution.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));

    let primary = sorted[0].0.clone();
    let secondary: Vec<Language> = sorted.iter().skip(1).map(|(l, _)| (*l).clone()).collect();

    LanguageProfile {
        primary,
        secondary,
        distribution,
        tree_sitter_coverage: 0.0, // Tier 1 does not run tree-sitter
    }
}

// ---------------------------------------------------------------------------
// Framework detection (spec section 2C)
// ---------------------------------------------------------------------------

fn detect_frameworks(root: &Path) -> Vec<FrameworkDetection> {
    let mut results = Vec::new();

    for rule in FRAMEWORK_RULES {
        let dep_path = root.join(rule.dep_file);
        if !dep_path.exists() {
            continue;
        }
        let content = match std::fs::read_to_string(&dep_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let found = match rule.dep_file {
            "Cargo.toml" => {
                // Check [dependencies] and [workspace.dependencies]
                content.contains(&format!("\"{}\"", rule.dep_key))
                    || content.contains(&format!("{} =", rule.dep_key))
                    || content.contains(&format!("{} {{", rule.dep_key))
            }
            "package.json" => {
                // Check dependencies and devDependencies
                content.contains(&format!("\"{}\"", rule.dep_key))
            }
            "go.mod" => content.contains(rule.dep_key),
            "Gemfile" => content.contains(rule.dep_key),
            "mix.exs" => content.contains(rule.dep_key),
            "pyproject.toml" => content.contains(rule.dep_key),
            _ => false,
        };

        if found {
            results.push(FrameworkDetection {
                name: rule.name.to_string(),
                version: None,
                category: rule.category.clone(),
                confidence: 0.9,
                detected_via: format!("{} dependency", rule.dep_file),
            });
        }
    }

    results
}

// ---------------------------------------------------------------------------
// Test profile detection (spec section 2D)
// ---------------------------------------------------------------------------

fn detect_test_profile(root: &Path, build: &BuildSystemProfile) -> TestProfile {
    let runner_cmd = match build.system {
        BuildSystem::Cargo => Some("cargo test".to_string()),
        BuildSystem::GoMod => Some("go test ./...".to_string()),
        BuildSystem::PyProject | BuildSystem::SetupPy => Some("pytest".to_string()),
        BuildSystem::Mix => Some("mix test".to_string()),
        BuildSystem::Deno => Some("deno test".to_string()),
        BuildSystem::Npm => read_npm_script(root, "test"),
        _ => None,
    };

    // Detect test location pattern
    let has_tests_dir = root.join("tests").is_dir()
        || root.join("test").is_dir()
        || root.join("__tests__").is_dir()
        || root.join("spec").is_dir();

    let location_pattern = if has_tests_dir {
        TestLocationPattern::SeparateDirectory
    } else {
        match build.system {
            BuildSystem::Cargo => TestLocationPattern::SameFile, // Rust uses #[cfg(test)]
            _ => TestLocationPattern::Unknown,
        }
    };

    // Detect fixture directory
    let fixture_dir = ["tests/fixtures", "test/fixtures", "fixtures", "testdata"]
        .iter()
        .map(|p| root.join(p))
        .find(|p| p.is_dir())
        .map(|p| p.strip_prefix(root).unwrap_or(&p).to_path_buf());

    TestProfile {
        runner_cmd,
        location_pattern,
        coverage_tool: None,
        fixture_dir,
    }
}

fn read_npm_script(root: &Path, script_name: &str) -> Option<String> {
    let pkg_path = root.join("package.json");
    let content = std::fs::read_to_string(&pkg_path).ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&content).ok()?;
    parsed
        .get("scripts")
        .and_then(|s| s.get(script_name))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

// ---------------------------------------------------------------------------
// Lint/formatter detection (spec section 2E)
// ---------------------------------------------------------------------------

fn detect_lint_tools(root: &Path) -> Vec<LintProfile> {
    let mut results = Vec::new();

    for rule in LINT_RULES {
        let config_path = root.join(rule.config_file);
        if config_path.exists() {
            results.push(LintProfile {
                tool: rule.tool.to_string(),
                config_path: Some(PathBuf::from(rule.config_file)),
                language: rule.language.clone(),
                auto_fix: rule.auto_fix,
            });
        }
    }

    // Check pyproject.toml for [tool.black] or [tool.ruff]
    let pyproject = root.join("pyproject.toml");
    if pyproject.exists() {
        if let Ok(content) = std::fs::read_to_string(&pyproject) {
            if content.contains("[tool.black]") {
                results.push(LintProfile {
                    tool: "Black".to_string(),
                    config_path: Some(PathBuf::from("pyproject.toml")),
                    language: Language::Python,
                    auto_fix: true,
                });
            }
            if content.contains("[tool.ruff]") && !results.iter().any(|l| l.tool == "Ruff") {
                results.push(LintProfile {
                    tool: "Ruff".to_string(),
                    config_path: Some(PathBuf::from("pyproject.toml")),
                    language: Language::Python,
                    auto_fix: true,
                });
            }
        }
    }

    results
}

// ---------------------------------------------------------------------------
// Entry point detection (spec section 2F)
// ---------------------------------------------------------------------------

fn detect_entry_points(root: &Path, languages: &LanguageProfile) -> Vec<EntryPoint> {
    let mut results = Vec::new();

    // Rust: src/main.rs or [[bin]] in Cargo.toml
    if root.join("src/main.rs").exists() {
        results.push(EntryPoint {
            file: PathBuf::from("src/main.rs"),
            kind: EntryPointKind::Binary,
        });
    }
    if root.join("src/lib.rs").exists() {
        results.push(EntryPoint {
            file: PathBuf::from("src/lib.rs"),
            kind: EntryPointKind::Library,
        });
    }

    // Python
    if languages.primary == Language::Python || languages.distribution.contains_key(&Language::Python) {
        // Check pyproject.toml [project.scripts]
        let pyproject = root.join("pyproject.toml");
        if pyproject.exists() {
            if let Ok(content) = std::fs::read_to_string(&pyproject) {
                if content.contains("[project.scripts]") {
                    results.push(EntryPoint {
                        file: PathBuf::from("pyproject.toml"),
                        kind: EntryPointKind::Script,
                    });
                }
            }
        }
    }

    // Node.js: check package.json "main" or "bin"
    let pkg_json = root.join("package.json");
    if pkg_json.exists() {
        if let Ok(content) = std::fs::read_to_string(&pkg_json) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                if parsed.get("bin").is_some() {
                    results.push(EntryPoint {
                        file: PathBuf::from("package.json"),
                        kind: EntryPointKind::Binary,
                    });
                } else if parsed.get("main").is_some() {
                    let main_val = parsed["main"].as_str().unwrap_or("index.js");
                    results.push(EntryPoint {
                        file: PathBuf::from(main_val),
                        kind: EntryPointKind::Library,
                    });
                }
            }
        }
    }

    // Go: cmd/ directory or main.go
    if root.join("cmd").is_dir() {
        results.push(EntryPoint {
            file: PathBuf::from("cmd/"),
            kind: EntryPointKind::Binary,
        });
    } else if root.join("main.go").exists() {
        results.push(EntryPoint {
            file: PathBuf::from("main.go"),
            kind: EntryPointKind::Binary,
        });
    }

    results
}

// ---------------------------------------------------------------------------
// CI detection (spec section 2G)
// ---------------------------------------------------------------------------

fn detect_ci(root: &Path) -> Option<CiProfile> {
    for rule in CI_RULES {
        let path = root.join(rule.path);
        if rule.is_glob {
            // Check directory exists (e.g., .github/workflows/)
            if path.is_dir() {
                let yml_files: Vec<String> = std::fs::read_dir(&path)
                    .ok()?
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path()
                            .extension()
                            .map_or(false, |ext| ext == "yml" || ext == "yaml")
                    })
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect();

                if !yml_files.is_empty() {
                    return Some(CiProfile {
                        system: rule.system.clone(),
                        config_path: PathBuf::from(rule.path),
                        detected_steps: yml_files,
                    });
                }
            }
        } else if path.exists() {
            return Some(CiProfile {
                system: rule.system.clone(),
                config_path: PathBuf::from(rule.path),
                detected_steps: Vec::new(),
            });
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Build command registry (spec section 5)
// ---------------------------------------------------------------------------

fn build_command_registry(root: &Path, build: &BuildSystemProfile) -> BuildCommandRegistry {
    let working_dir = PathBuf::from(".");

    // Find matching entry in static table
    for &(_, ref system, build_cmd, test_cmd, lint_cmd, fmt_cmd, clean_cmd) in BUILD_SYSTEM_TABLE {
        if *system == build.system {
            return BuildCommandRegistry {
                build: make_cmd_vec(build_cmd, &working_dir),
                test: make_cmd_vec(test_cmd, &working_dir),
                lint: make_cmd_vec(lint_cmd, &working_dir),
                format: make_cmd_vec(fmt_cmd, &working_dir),
                run: Vec::new(),
                clean: make_cmd_vec(clean_cmd, &working_dir),
            };
        }
    }

    // Content-based systems
    match build.system {
        BuildSystem::Npm => {
            let build_cmd = read_npm_script(root, "build").unwrap_or_default();
            let test_cmd = read_npm_script(root, "test").unwrap_or_default();
            let lint_cmd = read_npm_script(root, "lint").unwrap_or_default();
            BuildCommandRegistry {
                build: make_cmd_vec_if_nonempty(&build_cmd, &working_dir),
                test: make_cmd_vec_if_nonempty(&test_cmd, &working_dir),
                lint: make_cmd_vec_if_nonempty(&lint_cmd, &working_dir),
                format: Vec::new(),
                run: Vec::new(),
                clean: Vec::new(),
            }
        }
        BuildSystem::Gradle => BuildCommandRegistry {
            build: make_cmd_vec("./gradlew build", &working_dir),
            test: make_cmd_vec("./gradlew test", &working_dir),
            lint: make_cmd_vec("./gradlew lint", &working_dir),
            format: Vec::new(),
            run: Vec::new(),
            clean: make_cmd_vec("./gradlew clean", &working_dir),
        },
        BuildSystem::Maven => BuildCommandRegistry {
            build: make_cmd_vec("mvn compile", &working_dir),
            test: make_cmd_vec("mvn test", &working_dir),
            lint: make_cmd_vec("mvn checkstyle:check", &working_dir),
            format: Vec::new(),
            run: Vec::new(),
            clean: make_cmd_vec("mvn clean", &working_dir),
        },
        BuildSystem::DotNet => BuildCommandRegistry {
            build: make_cmd_vec("dotnet build", &working_dir),
            test: make_cmd_vec("dotnet test", &working_dir),
            lint: make_cmd_vec("dotnet format --verify-no-changes", &working_dir),
            format: make_cmd_vec("dotnet format", &working_dir),
            run: make_cmd_vec("dotnet run", &working_dir),
            clean: make_cmd_vec("dotnet clean", &working_dir),
        },
        BuildSystem::Bun => {
            let test_cmd = read_npm_script(root, "test").unwrap_or_else(|| "bun test".to_string());
            BuildCommandRegistry {
                build: make_cmd_vec("bun build", &working_dir),
                test: make_cmd_vec(&test_cmd, &working_dir),
                lint: Vec::new(),
                format: Vec::new(),
                run: Vec::new(),
                clean: Vec::new(),
            }
        }
        _ => BuildCommandRegistry::default(),
    }
}

fn make_cmd_vec(cmd: &str, working_dir: &Path) -> Vec<BuildCommand> {
    if cmd.is_empty() {
        return Vec::new();
    }
    vec![BuildCommand {
        cmd: cmd.to_string(),
        working_dir: working_dir.to_path_buf(),
        env: HashMap::new(),
        timeout_s: DEFAULT_COMMAND_TIMEOUT_S,
        scope: CommandScope::Full,
        verified: false,
    }]
}

fn make_cmd_vec_if_nonempty(cmd: &str, working_dir: &Path) -> Vec<BuildCommand> {
    if cmd.is_empty() {
        Vec::new()
    } else {
        make_cmd_vec(cmd, working_dir)
    }
}

// ---------------------------------------------------------------------------
// Tier 2 convention population
// ---------------------------------------------------------------------------

/// Populate Tier 2 convention fields on an existing profile.
///
/// Uses DKG-based extraction if the engine has data (Rust projects),
/// otherwise falls back to regex-based scanning. Updates naming,
/// error_handling, module_organization, and test_profile.location_pattern.
///
/// Spec: project-intelligence-design.txt sections 3A-3E, 10.
pub fn populate_tier2_conventions(
    profile: &mut ProjectProfile,
    dkg_engine: Option<&crate::dkg::DkgEngine>,
) {
    let primary_lang = profile.languages.primary.id_str();
    let project_root = profile.project_root.clone();

    // Try DKG path first (Rust projects with populated graph).
    let stats = if let Some(engine) = dkg_engine {
        match engine.extract_conventions(&project_root) {
            Ok(s) => {
                debug!(source = "dkg", "Tier 2 conventions extracted from DKG");
                s
            }
            Err(e) => {
                debug!(error = %e, "DKG extraction failed, falling back to regex");
                crate::conventions::extract_from_regex_fallback(&project_root, primary_lang)
            }
        }
    } else {
        debug!("no DKG engine provided, using regex fallback for conventions");
        crate::conventions::extract_from_regex_fallback(&project_root, primary_lang)
    };

    profile.naming = stats.naming;
    profile.error_handling = stats.error_handling;
    profile.module_organization = stats.module_organization;
    // Only update test location if detection found something meaningful.
    if stats.test_location != elai_core::project_profile::TestLocationPattern::Unknown {
        profile.test_profile.location_pattern = stats.test_location;
    }
}

// ---------------------------------------------------------------------------
// TOML serialization
// ---------------------------------------------------------------------------

/// Serialize profile to TOML for storage at .elai_cc/project-profile.toml.
pub fn serialize_profile(profile: &ProjectProfile) -> Result<String, ProjectProfileError> {
    toml::to_string_pretty(profile).map_err(ProjectProfileError::TomlSerialize)
}

/// Deserialize profile from TOML.
pub fn deserialize_profile(toml_str: &str) -> Result<ProjectProfile, ProjectProfileError> {
    toml::from_str(toml_str).map_err(ProjectProfileError::TomlParse)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_project(files: &[(&str, &str)]) -> TempDir {
        let dir = TempDir::new().expect("create temp dir");
        for (path, content) in files {
            let full = dir.path().join(path);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent).expect("create parent dirs");
            }
            fs::write(&full, content).expect("write file");
        }
        dir
    }

    // --- Build system detection (22 per spec section 2A) ---

    #[test]
    fn detect_cargo() {
        let dir = setup_project(&[("Cargo.toml", "[package]\nname = \"test\"\nedition = \"2021\"")]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::Cargo);
        assert_eq!(result.version, Some("edition 2021".to_string()));
    }

    #[test]
    fn detect_cargo_workspace() {
        let dir = setup_project(&[("Cargo.toml", "[workspace]\nmembers = [\"crate-a\", \"crate-b\"]")]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::Cargo);
        assert!(result.monorepo);
        assert_eq!(result.workspaces.len(), 2);
        assert_eq!(result.workspaces[0].name, "crate-a");
    }

    #[test]
    fn detect_npm() {
        let dir = setup_project(&[("package.json", r#"{"name":"test","scripts":{"test":"jest"}}"#)]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::Npm);
    }

    #[test]
    fn detect_npm_workspaces() {
        let dir = setup_project(&[("package.json", r#"{"name":"mono","workspaces":["packages/*"]}"#)]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::Npm);
        assert!(result.monorepo);
    }

    #[test]
    fn detect_pyproject() {
        let dir = setup_project(&[("pyproject.toml", "[project]\nname = \"myapp\"")]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::PyProject);
    }

    #[test]
    fn detect_setup_py() {
        let dir = setup_project(&[("setup.py", "from setuptools import setup\nsetup(name='x')")]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::SetupPy);
    }

    #[test]
    fn detect_go_mod() {
        let dir = setup_project(&[("go.mod", "module example.com/foo\n\ngo 1.21")]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::GoMod);
    }

    #[test]
    fn detect_cmake() {
        let dir = setup_project(&[("CMakeLists.txt", "cmake_minimum_required(VERSION 3.20)")]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::CMake);
    }

    #[test]
    fn detect_makefile() {
        let dir = setup_project(&[("Makefile", "all:\n\techo hello")]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::Make);
    }

    #[test]
    fn detect_gradle() {
        let dir = setup_project(&[("build.gradle", "plugins { id 'java' }")]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::Gradle);
    }

    #[test]
    fn detect_gradle_kts() {
        let dir = setup_project(&[("build.gradle.kts", "plugins { java }")]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::Gradle);
    }

    #[test]
    fn detect_maven() {
        let dir = setup_project(&[("pom.xml", "<project><modelVersion>4.0.0</modelVersion></project>")]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::Maven);
    }

    #[test]
    fn detect_mix() {
        let dir = setup_project(&[("mix.exs", "defmodule MyApp.MixProject do\nend")]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::Mix);
    }

    #[test]
    fn detect_bundler() {
        let dir = setup_project(&[("Gemfile", "source 'https://rubygems.org'\ngem 'rails'")]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::Bundler);
    }

    #[test]
    fn detect_composer() {
        let dir = setup_project(&[("composer.json", r#"{"name":"vendor/pkg"}"#)]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::Composer);
    }

    #[test]
    fn detect_deno() {
        let dir = setup_project(&[("deno.json", r#"{"tasks":{"test":"deno test"}}"#)]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::Deno);
    }

    #[test]
    fn detect_dotnet() {
        let dir = setup_project(&[("MyApp.csproj", "<Project Sdk=\"Microsoft.NET.Sdk\"></Project>")]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::DotNet);
    }

    #[test]
    fn detect_bun() {
        let dir = setup_project(&[("bunfig.toml", "[install]\npeer = false")]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::Bun);
    }

    #[test]
    fn detect_zig() {
        let dir = setup_project(&[("build.zig", "const std = @import(\"std\");")]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::Zig);
    }

    #[test]
    fn detect_just() {
        let dir = setup_project(&[("justfile", "build:\n  cargo build")]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::Just);
    }

    #[test]
    fn detect_task() {
        let dir = setup_project(&[("Taskfile.yml", "version: '3'\ntasks:\n  build:")]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::Task);
    }

    #[test]
    fn detect_meson() {
        let dir = setup_project(&[("meson.build", "project('mylib', 'c')")]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::Meson);
    }

    // --- Priority: Cargo wins over Makefile (spec: language-native > generic) ---

    #[test]
    fn cargo_beats_makefile() {
        let dir = setup_project(&[
            ("Cargo.toml", "[package]\nname = \"test\""),
            ("Makefile", "all:\n\tcargo build"),
        ]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::Cargo);
    }

    // --- Empty project ---

    #[test]
    fn empty_project_returns_unknown() {
        let dir = setup_project(&[]);
        let result = detect_build_system(dir.path()).expect("detect");
        assert_eq!(result.system, BuildSystem::Unknown);
    }

    // --- Language detection ---

    #[test]
    fn detect_rust_language() {
        let dir = setup_project(&[
            ("src/main.rs", "fn main() {}"),
            ("src/lib.rs", "pub fn hello() {}"),
        ]);
        let langs = detect_languages(dir.path());
        assert_eq!(langs.primary, Language::Rust);
        assert_eq!(*langs.distribution.get(&Language::Rust).unwrap_or(&0), 2);
    }

    #[test]
    fn detect_polyglot() {
        let dir = setup_project(&[
            ("src/main.rs", "fn main() {}"),
            ("scripts/run.py", "print('hello')"),
            ("scripts/util.py", "def f(): pass"),
        ]);
        let langs = detect_languages(dir.path());
        // Python has more files
        assert_eq!(langs.primary, Language::Python);
        assert!(langs.secondary.contains(&Language::Rust));
    }

    // --- Framework detection ---

    #[test]
    fn detect_axum_framework() {
        let dir = setup_project(&[("Cargo.toml", "[dependencies]\naxum = \"0.7\"")]);
        let fws = detect_frameworks(dir.path());
        assert_eq!(fws.len(), 1);
        assert_eq!(fws[0].name, "Axum");
        assert_eq!(fws[0].category, FrameworkCategory::Web);
    }

    #[test]
    fn detect_fastapi_framework() {
        let dir = setup_project(&[("pyproject.toml", "[project]\ndependencies = [\"fastapi\"]")]);
        let fws = detect_frameworks(dir.path());
        assert_eq!(fws.len(), 1);
        assert_eq!(fws[0].name, "FastAPI");
    }

    // --- CI detection ---

    #[test]
    fn detect_github_actions() {
        let dir = setup_project(&[(".github/workflows/ci.yml", "name: CI\non: push")]);
        let ci = detect_ci(dir.path());
        assert!(ci.is_some());
        let ci = ci.expect("ci");
        assert_eq!(ci.system, CiSystem::GitHubActions);
        assert!(ci.detected_steps.contains(&"ci.yml".to_string()));
    }

    #[test]
    fn detect_gitlab_ci() {
        let dir = setup_project(&[(".gitlab-ci.yml", "stages:\n  - build")]);
        let ci = detect_ci(dir.path());
        assert!(ci.is_some());
        assert_eq!(ci.as_ref().map(|c| &c.system), Some(&CiSystem::GitLabCi));
    }

    // --- Lint detection ---

    #[test]
    fn detect_rustfmt_and_clippy() {
        let dir = setup_project(&[("rustfmt.toml", "max_width = 100"), ("clippy.toml", "")]);
        let lints = detect_lint_tools(dir.path());
        assert!(lints.iter().any(|l| l.tool == "rustfmt"));
        assert!(lints.iter().any(|l| l.tool == "clippy"));
    }

    #[test]
    fn detect_ruff_from_pyproject() {
        let dir = setup_project(&[("pyproject.toml", "[tool.ruff]\nline-length = 100")]);
        let lints = detect_lint_tools(dir.path());
        assert!(lints.iter().any(|l| l.tool == "Ruff"));
    }

    // --- Entry point detection ---

    #[test]
    fn detect_rust_entry_points() {
        let dir = setup_project(&[("src/main.rs", "fn main() {}"), ("src/lib.rs", "")]);
        let eps = detect_entry_points(dir.path(), &LanguageProfile::default());
        assert!(eps.iter().any(|e| e.kind == EntryPointKind::Binary && e.file == PathBuf::from("src/main.rs")));
        assert!(eps.iter().any(|e| e.kind == EntryPointKind::Library && e.file == PathBuf::from("src/lib.rs")));
    }

    // --- Build command registry ---

    #[test]
    fn cargo_command_registry() {
        let build = BuildSystemProfile {
            system: BuildSystem::Cargo,
            config_path: PathBuf::from("Cargo.toml"),
            version: None,
            monorepo: false,
            workspaces: Vec::new(),
        };
        let dir = setup_project(&[]);
        let reg = build_command_registry(dir.path(), &build);
        assert_eq!(reg.build.len(), 1);
        assert_eq!(reg.build[0].cmd, "cargo build");
        assert_eq!(reg.test[0].cmd, "cargo test");
        assert_eq!(reg.lint[0].cmd, "cargo clippy");
        assert_eq!(reg.format[0].cmd, "cargo fmt");
    }

    // --- Full profile integration ---

    #[test]
    fn full_rust_project_profile() {
        let dir = setup_project(&[
            ("Cargo.toml", "[package]\nname = \"myapp\"\nedition = \"2021\"\n\n[dependencies]\naxum = \"0.7\""),
            ("src/main.rs", "fn main() {}"),
            ("src/lib.rs", "pub fn hello() {}"),
            ("tests/integration.rs", ""),
            ("rustfmt.toml", "max_width = 100"),
            (".github/workflows/ci.yml", "name: CI"),
        ]);
        let profile = detect_project_profile(dir.path()).expect("profile");
        assert_eq!(profile.build_system.system, BuildSystem::Cargo);
        assert_eq!(profile.languages.primary, Language::Rust);
        assert!(profile.frameworks.iter().any(|f| f.name == "Axum"));
        assert_eq!(profile.test_profile.runner_cmd, Some("cargo test".to_string()));
        assert!(profile.lint_tools.iter().any(|l| l.tool == "rustfmt"));
        assert!(profile.ci.is_some());
        assert!(!profile.commands.build.is_empty());
    }

    #[test]
    fn full_python_project_profile() {
        let dir = setup_project(&[
            ("pyproject.toml", "[project]\nname = \"myapp\"\ndependencies = [\"fastapi\"]\n\n[tool.ruff]\nline-length = 88"),
            ("src/app.py", "from fastapi import FastAPI"),
            ("tests/test_app.py", "def test_hello(): pass"),
        ]);
        let profile = detect_project_profile(dir.path()).expect("profile");
        assert_eq!(profile.build_system.system, BuildSystem::PyProject);
        assert_eq!(profile.languages.primary, Language::Python);
        assert!(profile.frameworks.iter().any(|f| f.name == "FastAPI"));
        assert_eq!(profile.test_profile.runner_cmd, Some("pytest".to_string()));
        assert!(profile.lint_tools.iter().any(|l| l.tool == "Ruff"));
    }

    // --- Adversarial tests ---

    #[test]
    fn nonexistent_root() {
        let result = detect_project_profile(Path::new("/nonexistent/path/that/does/not/exist"));
        assert!(result.is_err());
    }

    #[test]
    fn corrupted_cargo_toml() {
        let dir = setup_project(&[("Cargo.toml", "{{{{ invalid toml !!!!")]);
        // Corrupted TOML causes a parse error in workspace detection
        let result = detect_build_system(dir.path());
        // Either error or detected as Cargo (parse error happens during workspace detection)
        match &result {
            Ok(bs) => assert_eq!(bs.system, BuildSystem::Cargo),
            Err(_) => {} // parse error is acceptable
        }
    }

    #[test]
    fn competing_build_systems() {
        let dir = setup_project(&[
            ("Cargo.toml", "[package]\nname = \"test\""),
            ("Makefile", "build:\n\tcargo build"),
            ("justfile", "build:\n  cargo build"),
        ]);
        let result = detect_build_system(dir.path()).expect("detect");
        // Cargo (language-native) should win
        assert_eq!(result.system, BuildSystem::Cargo);
    }

    #[test]
    fn empty_project_full_profile() {
        let dir = setup_project(&[]);
        let profile = detect_project_profile(dir.path()).expect("profile");
        assert_eq!(profile.build_system.system, BuildSystem::Unknown);
        assert_eq!(profile.languages.primary, Language::Unknown);
        assert!(profile.frameworks.is_empty());
    }

    // --- Serialization round-trip ---

    #[test]
    fn profile_toml_roundtrip() {
        let dir = setup_project(&[
            ("Cargo.toml", "[package]\nname = \"test\"\nedition = \"2021\""),
            ("src/main.rs", "fn main() {}"),
        ]);
        let profile = detect_project_profile(dir.path()).expect("profile");
        let toml_str = serialize_profile(&profile).expect("serialize");
        let restored = deserialize_profile(&toml_str).expect("deserialize");
        assert_eq!(restored.build_system.system, profile.build_system.system);
        assert_eq!(restored.schema_version, PROFILE_SCHEMA_VERSION);
    }

    // --- Test profile details ---

    #[test]
    fn test_profile_separate_dir() {
        let dir = setup_project(&[
            ("Cargo.toml", "[package]\nname = \"test\""),
            ("tests/integration.rs", ""),
            ("tests/fixtures/data.json", "{}"),
        ]);
        let build = detect_build_system(dir.path()).expect("build");
        let tp = detect_test_profile(dir.path(), &build);
        assert_eq!(tp.location_pattern, TestLocationPattern::SeparateDirectory);
        assert!(tp.fixture_dir.is_some());
    }

    #[test]
    fn npm_test_script_detection() {
        let dir = setup_project(&[("package.json", r#"{"scripts":{"test":"vitest run","build":"vite build"}}"#)]);
        let build = detect_build_system(dir.path()).expect("build");
        let tp = detect_test_profile(dir.path(), &build);
        assert_eq!(tp.runner_cmd, Some("vitest run".to_string()));
    }
}
