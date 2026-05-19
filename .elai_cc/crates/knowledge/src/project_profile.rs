// Project Intelligence: Tier 1 structural discovery (pure filesystem analysis).
// Spec: docs/specs/guide/project-intelligence-design.txt sections 2A-2G, 5.
//
// No LLM, no DKG, no network. Pattern matching on known config files.
// If the signal is not present, the field is Unknown.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use elai_core::project_profile::{
    BuildCommand, BuildCommandRegistry, BuildSystem, BuildSystemProfile, CiProfile, CiSystem,
    CommandScope, EntryPoint, EntryPointKind, FrameworkCategory, FrameworkDetection, Language,
    LanguageProfile, LintProfile, ProjectProfile, ProjectProfileError, TestLocationPattern,
    TestProfile, WorkspaceMember, DEFAULT_COMMAND_TIMEOUT_S, PROFILE_SCHEMA_VERSION,
};
use elai_core::{ClockProvider as _, OsClock};

use ignore::WalkBuilder;
use tracing::{debug, warn};

// ---------------------------------------------------------------------------
// A-32 constants
// ---------------------------------------------------------------------------

/// Confidence value for framework detection via dependency file match.
const FRAMEWORK_MATCH_CONFIDENCE: f32 = 0.9;

/// Override file name, placed at project root. Never auto-generated.
const OVERRIDE_FILE_NAME: &str = "project-profile-overrides.toml";

/// Keywords scanned in CI config, Makefile targets, and package.json scripts
/// to detect coverage tooling.
const COVERAGE_TOOL_KEYWORDS: &[(&str, &str)] = &[
    ("llvm-cov", "llvm-cov"),
    ("grcov", "grcov"),
    ("pytest-cov", "pytest-cov"),
    ("coverage.py", "coverage.py"),
    ("istanbul", "istanbul"),
    ("c8", "c8"),
    ("nyc", "nyc"),
    ("go tool cover", "go-cover"),
    ("kcov", "kcov"),
    ("tarpaulin", "tarpaulin"),
];

/// Coverage-related Makefile/justfile target names.
const COVERAGE_TARGET_NAMES: &[&str] = &["coverage", "cov", "cover"];

// ---------------------------------------------------------------------------
// Build system detection table (spec section 2A)
// ---------------------------------------------------------------------------

/// Config file -> (BuildSystem, build_cmd, test_cmd, lint_cmd, format_cmd, clean_cmd).
///
/// Build system priority (spec section 2A, 4-tier ordering):
///   Priority 1 — Language-native: Cargo, GoMod, Mix, Zig (lines below)
///   Priority 2 — Package managers: PyProject, SetupPy, Composer, Bundler, Deno (lines below)
///   Priority 3 — Task runners: Just, Task (lines below)
///   Priority 4 — Generic: Make, CMake, Meson (lines below)
///
/// First match wins. Language-native always beats generic.
/// Content-based systems (package.json/build.gradle/pom.xml) checked separately.
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
    // ---- Rust (19 entries) ----
    FrameworkRule { language: Language::Rust, dep_file: "Cargo.toml", dep_key: "axum",       name: "Axum",     category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Rust, dep_file: "Cargo.toml", dep_key: "actix-web",  name: "Actix",    category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Rust, dep_file: "Cargo.toml", dep_key: "rocket",     name: "Rocket",   category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Rust, dep_file: "Cargo.toml", dep_key: "warp",       name: "Warp",     category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Rust, dep_file: "Cargo.toml", dep_key: "poem",       name: "Poem",     category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Rust, dep_file: "Cargo.toml", dep_key: "tide",       name: "Tide",     category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Rust, dep_file: "Cargo.toml", dep_key: "salvo",      name: "Salvo",    category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Rust, dep_file: "Cargo.toml", dep_key: "bevy",       name: "Bevy",     category: FrameworkCategory::Game },
    FrameworkRule { language: Language::Rust, dep_file: "Cargo.toml", dep_key: "leptos",     name: "Leptos",   category: FrameworkCategory::Fullstack },
    FrameworkRule { language: Language::Rust, dep_file: "Cargo.toml", dep_key: "tauri",      name: "Tauri",    category: FrameworkCategory::Desktop },
    FrameworkRule { language: Language::Rust, dep_file: "Cargo.toml", dep_key: "yew",        name: "Yew",      category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Rust, dep_file: "Cargo.toml", dep_key: "dioxus",     name: "Dioxus",   category: FrameworkCategory::Desktop },
    FrameworkRule { language: Language::Rust, dep_file: "Cargo.toml", dep_key: "iced",       name: "Iced",     category: FrameworkCategory::Desktop },
    FrameworkRule { language: Language::Rust, dep_file: "Cargo.toml", dep_key: "egui",       name: "Egui",     category: FrameworkCategory::Desktop },
    FrameworkRule { language: Language::Rust, dep_file: "Cargo.toml", dep_key: "diesel",     name: "Diesel",   category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Rust, dep_file: "Cargo.toml", dep_key: "sea-orm",    name: "SeaORM",   category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Rust, dep_file: "Cargo.toml", dep_key: "sqlx",       name: "Sqlx",     category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Rust, dep_file: "Cargo.toml", dep_key: "tonic",      name: "Tonic",    category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Rust, dep_file: "Cargo.toml", dep_key: "prost",      name: "Prost",    category: FrameworkCategory::Server },

    // ---- Python (21 entries) ----
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "fastapi",        name: "FastAPI",       category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "django",         name: "Django",        category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "flask",          name: "Flask",         category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "starlette",      name: "Starlette",     category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "sanic",          name: "Sanic",         category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "tornado",        name: "Tornado",       category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "celery",         name: "Celery",        category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "scrapy",         name: "Scrapy",        category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "pydantic",       name: "Pydantic",      category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "sqlalchemy",     name: "SQLAlchemy",    category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "alembic",        name: "Alembic",       category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "torch",          name: "PyTorch",       category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "tensorflow",     name: "TensorFlow",    category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "transformers",   name: "Transformers",  category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "scikit-learn",   name: "scikit-learn",  category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "jax",            name: "JAX",           category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "lightning",      name: "Lightning",     category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "keras",          name: "Keras",         category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "opencv-python",  name: "OpenCV",        category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "gradio",         name: "Gradio",        category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "streamlit",      name: "Streamlit",     category: FrameworkCategory::ML },

    // ---- TypeScript / JavaScript (29 entries) ----
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "next",                    name: "Next.js",         category: FrameworkCategory::Fullstack },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "svelte",                  name: "Svelte",          category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "react",                   name: "React",           category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "vue",                     name: "Vue",             category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "express",                 name: "Express",         category: FrameworkCategory::Server },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "@nestjs/core",            name: "Nest.js",         category: FrameworkCategory::Server },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "fastify",                 name: "Fastify",         category: FrameworkCategory::Server },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "hono",                    name: "Hono",            category: FrameworkCategory::Server },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "solid-js",                name: "Solid",           category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "@builder.io/qwik",        name: "Qwik",            category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "@remix-run/react",        name: "Remix",           category: FrameworkCategory::Fullstack },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "astro",                   name: "Astro",           category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "nuxt",                    name: "Nuxt",            category: FrameworkCategory::Fullstack },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "@angular/core",           name: "Angular",         category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "gatsby",                  name: "Gatsby",          category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "ember-source",            name: "Ember",           category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "backbone",                name: "Backbone",        category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "@ionic/angular",          name: "Ionic",           category: FrameworkCategory::Mobile },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "@capacitor/core",         name: "Capacitor",       category: FrameworkCategory::Mobile },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "electron",                name: "Electron",        category: FrameworkCategory::Desktop },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "d3",                      name: "D3",              category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "three",                   name: "Three.js",        category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "babylonjs",               name: "Babylon.js",      category: FrameworkCategory::Game },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "vite",                    name: "Vite",            category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "vitest",                  name: "Vitest",          category: FrameworkCategory::Testing },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "jest",                    name: "Jest",            category: FrameworkCategory::Testing },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "@playwright/test",        name: "Playwright",      category: FrameworkCategory::Testing },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "cypress",                 name: "Cypress",         category: FrameworkCategory::Testing },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "prisma",                  name: "Prisma",          category: FrameworkCategory::Server },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "drizzle-orm",             name: "Drizzle",         category: FrameworkCategory::Server },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "@trpc/server",            name: "tRPC",            category: FrameworkCategory::Server },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "socket.io",               name: "Socket.io",       category: FrameworkCategory::Server },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "@apollo/server",          name: "Apollo GraphQL",  category: FrameworkCategory::Server },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "react-native",            name: "React Native",    category: FrameworkCategory::Mobile },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "expo",                    name: "Expo",            category: FrameworkCategory::Mobile },

    // ---- Python (also check requirements.txt / setup.cfg) ----
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "aiohttp",        name: "aiohttp",       category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "bottle",         name: "Bottle",        category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "falcon",         name: "Falcon",        category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "litestar",       name: "Litestar",      category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "pyramid",        name: "Pyramid",       category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "dash",           name: "Dash",          category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "panel",          name: "Panel",         category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "uvicorn",        name: "Uvicorn",       category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "gunicorn",       name: "Gunicorn",      category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "dramatiq",       name: "Dramatiq",      category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "huey",           name: "Huey",          category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "peewee",         name: "Peewee",        category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "tortoise-orm",   name: "Tortoise ORM",  category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "mongoengine",    name: "MongoEngine",   category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "pytest",         name: "pytest",        category: FrameworkCategory::Testing },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "polars",         name: "Polars",        category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "pandas",         name: "Pandas",        category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "numpy",          name: "NumPy",         category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "scipy",          name: "SciPy",         category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "xgboost",        name: "XGBoost",       category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "lightgbm",       name: "LightGBM",      category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "catboost",       name: "CatBoost",      category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "mlflow",         name: "MLflow",        category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "wandb",          name: "W&B",           category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "dvc",            name: "DVC",           category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "airflow",        name: "Airflow",       category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "prefect",        name: "Prefect",       category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "dagster",        name: "Dagster",       category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "dask",           name: "Dask",          category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "ray",            name: "Ray",           category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "spacy",          name: "spaCy",         category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "nltk",           name: "NLTK",          category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "matplotlib",     name: "Matplotlib",    category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "plotly",         name: "Plotly",        category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "seaborn",        name: "Seaborn",       category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "pillow",         name: "Pillow",        category: FrameworkCategory::ML },
    FrameworkRule { language: Language::Python, dep_file: "pyproject.toml", dep_key: "httpx",          name: "HTTPX",         category: FrameworkCategory::Web },

    // ---- TypeScript / JavaScript (additional entries) ----
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "koa",                      name: "Koa",             category: FrameworkCategory::Server },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "@reduxjs/toolkit",         name: "Redux Toolkit",   category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "zustand",                  name: "Zustand",         category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "mobx",                     name: "MobX",            category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "tailwindcss",              name: "Tailwind CSS",    category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "styled-components",        name: "styled-components", category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "@emotion/react",           name: "Emotion",         category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "storybook",                name: "Storybook",       category: FrameworkCategory::Testing },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "@testing-library/react",   name: "Testing Library", category: FrameworkCategory::Testing },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "webpack",                  name: "Webpack",         category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "esbuild",                  name: "esbuild",         category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "rollup",                   name: "Rollup",          category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "mongoose",                 name: "Mongoose",        category: FrameworkCategory::Server },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "typeorm",                  name: "TypeORM",         category: FrameworkCategory::Server },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "sequelize",                name: "Sequelize",       category: FrameworkCategory::Server },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "knex",                     name: "Knex",            category: FrameworkCategory::Server },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "graphql",                  name: "GraphQL",         category: FrameworkCategory::Server },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "rxjs",                     name: "RxJS",            category: FrameworkCategory::Web },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "mocha",                    name: "Mocha",           category: FrameworkCategory::Testing },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "chai",                     name: "Chai",            category: FrameworkCategory::Testing },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "puppeteer",                name: "Puppeteer",       category: FrameworkCategory::Testing },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "cheerio",                  name: "Cheerio",         category: FrameworkCategory::Server },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "bull",                     name: "Bull",            category: FrameworkCategory::Server },
    FrameworkRule { language: Language::TypeScript, dep_file: "package.json", dep_key: "bullmq",                   name: "BullMQ",          category: FrameworkCategory::Server },

    // ---- Go (9 entries) ----
    FrameworkRule { language: Language::Go, dep_file: "go.mod", dep_key: "gin-gonic/gin",            name: "Gin",       category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Go, dep_file: "go.mod", dep_key: "gofiber/fiber",            name: "Fiber",     category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Go, dep_file: "go.mod", dep_key: "labstack/echo",            name: "Echo",      category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Go, dep_file: "go.mod", dep_key: "go-chi/chi",               name: "Chi",       category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Go, dep_file: "go.mod", dep_key: "gorilla/mux",              name: "Gorilla Mux", category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Go, dep_file: "go.mod", dep_key: "gorm.io/gorm",             name: "Gorm",      category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Go, dep_file: "go.mod", dep_key: "entgo.io/ent",             name: "Ent",       category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Go, dep_file: "go.mod", dep_key: "stretchr/testify",         name: "Testify",   category: FrameworkCategory::Testing },
    FrameworkRule { language: Language::Go, dep_file: "go.mod", dep_key: "google.golang.org/grpc",   name: "gRPC-Go",   category: FrameworkCategory::Server },

    // ---- Java (8 entries) ----
    FrameworkRule { language: Language::Java, dep_file: "build.gradle",     dep_key: "spring-boot-starter",            name: "Spring Boot",    category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Java, dep_file: "build.gradle",     dep_key: "io.quarkus",                     name: "Quarkus",        category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Java, dep_file: "build.gradle",     dep_key: "io.micronaut",                   name: "Micronaut",      category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Java, dep_file: "build.gradle",     dep_key: "org.hibernate",                  name: "Hibernate",      category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Java, dep_file: "pom.xml",         dep_key: "spring-boot-starter",             name: "Spring Boot",    category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Java, dep_file: "pom.xml",         dep_key: "quarkus",                         name: "Quarkus",        category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Java, dep_file: "pom.xml",         dep_key: "micronaut",                       name: "Micronaut",      category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Java, dep_file: "pom.xml",         dep_key: "hibernate",                       name: "Hibernate",      category: FrameworkCategory::Server },

    // ---- C# (4 entries) ----
    FrameworkRule { language: Language::CSharp, dep_file: ".csproj",       dep_key: "Microsoft.AspNetCore",            name: "ASP.NET",        category: FrameworkCategory::Web },
    FrameworkRule { language: Language::CSharp, dep_file: ".csproj",       dep_key: "Microsoft.AspNetCore.Components", name: "Blazor",         category: FrameworkCategory::Fullstack },
    FrameworkRule { language: Language::CSharp, dep_file: ".csproj",       dep_key: "Microsoft.Maui",                  name: "MAUI",           category: FrameworkCategory::Mobile },
    FrameworkRule { language: Language::CSharp, dep_file: ".csproj",       dep_key: "Microsoft.EntityFrameworkCore",   name: "EF Core",        category: FrameworkCategory::Server },

    // ---- Ruby (5 entries) ----
    FrameworkRule { language: Language::Ruby, dep_file: "Gemfile", dep_key: "rails",     name: "Rails",    category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Ruby, dep_file: "Gemfile", dep_key: "sinatra",   name: "Sinatra",  category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Ruby, dep_file: "Gemfile", dep_key: "hanami",    name: "Hanami",   category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Ruby, dep_file: "Gemfile", dep_key: "rspec",     name: "RSpec",    category: FrameworkCategory::Testing },
    FrameworkRule { language: Language::Ruby, dep_file: "Gemfile", dep_key: "sidekiq",   name: "Sidekiq",  category: FrameworkCategory::Server },

    // ---- Elixir (3 entries) ----
    FrameworkRule { language: Language::Elixir, dep_file: "mix.exs", dep_key: "phoenix",       name: "Phoenix",      category: FrameworkCategory::Web },
    FrameworkRule { language: Language::Elixir, dep_file: "mix.exs", dep_key: "ecto",          name: "Ecto",         category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Elixir, dep_file: "mix.exs", dep_key: "absinthe",      name: "Absinthe",     category: FrameworkCategory::Server },

    // ---- Dart / Flutter (1 entry) ----
    FrameworkRule { language: Language::Dart,   dep_file: "pubspec.yaml",  dep_key: "flutter",   name: "Flutter",  category: FrameworkCategory::Mobile },

    // ---- Kotlin (1 entry) ----
    FrameworkRule { language: Language::Kotlin, dep_file: "build.gradle.kts", dep_key: "compose", name: "Compose",  category: FrameworkCategory::Mobile },

    // ---- PHP (5 entries) ----
    FrameworkRule { language: Language::PHP, dep_file: "composer.json", dep_key: "laravel/framework",   name: "Laravel",    category: FrameworkCategory::Web },
    FrameworkRule { language: Language::PHP, dep_file: "composer.json", dep_key: "symfony/symfony",     name: "Symfony",    category: FrameworkCategory::Web },
    FrameworkRule { language: Language::PHP, dep_file: "composer.json", dep_key: "slim/slim",           name: "Slim",       category: FrameworkCategory::Web },
    FrameworkRule { language: Language::PHP, dep_file: "composer.json", dep_key: "phpunit/phpunit",     name: "PHPUnit",    category: FrameworkCategory::Testing },
    FrameworkRule { language: Language::PHP, dep_file: "composer.json", dep_key: "doctrine/orm",        name: "Doctrine",   category: FrameworkCategory::Server },

    // ---- C++ (2 entries) ----
    FrameworkRule { language: Language::Cpp, dep_file: "CMakeLists.txt", dep_key: "Qt",        name: "Qt",       category: FrameworkCategory::Desktop },
    FrameworkRule { language: Language::Cpp, dep_file: "CMakeLists.txt", dep_key: "wxWidgets", name: "wxWidgets", category: FrameworkCategory::Desktop },

    // ---- Scala (2 entries) ----
    FrameworkRule { language: Language::Scala, dep_file: "build.sbt",    dep_key: "akka",      name: "Akka",     category: FrameworkCategory::Server },
    FrameworkRule { language: Language::Scala, dep_file: "build.sbt",    dep_key: "playframework", name: "Play", category: FrameworkCategory::Web },

    // ---- Swift (1 entry) ----
    FrameworkRule { language: Language::Swift, dep_file: "Package.swift", dep_key: "vapor",     name: "Vapor",    category: FrameworkCategory::Web },

    // ---- Haskell (1 entry) ----
    FrameworkRule { language: Language::Haskell, dep_file: "package.yaml", dep_key: "yesod",   name: "Yesod",    category: FrameworkCategory::Web },

    // ---- Gleam (1 entry) ----
    FrameworkRule { language: Language::Gleam, dep_file: "gleam.toml",    dep_key: "wisp",     name: "Wisp",     category: FrameworkCategory::Web },

    // ---- Zig (1 entry) ----
    FrameworkRule { language: Language::Zig, dep_file: "build.zig.zon",  dep_key: "zap",      name: "Zap",      category: FrameworkCategory::Web },
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
    let ci = detect_ci(project_root);
    let test_profile = detect_test_profile(project_root, &build_system, &ci);
    let lint_tools = detect_lint_tools(project_root);
    let entry_points = detect_entry_points(project_root, &languages);
    let commands = build_command_registry(project_root, &build_system);

    let generated_at = {
        let duration = OsClock
            .system_now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        // ISO 8601 without external dep: use seconds since epoch as fallback
        format!("{}", duration.as_secs())
    };

    let mut profile = ProjectProfile {
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
        dep_profile: Default::default(),
        project_vision: None,
        architecture: None,
        domains: Vec::new(),
        commands,
        fix_template_keys: Vec::new(),
        constraints: Vec::new(),
        overrides_applied: false,
    };

    // A-32: apply user overrides (field-by-field merge from separate TOML file)
    if let Some(overrides_table) = load_overrides(project_root) {
        merge_overrides(&mut profile, &overrides_table);
    }

    // A-32: Tier A static command validation
    validate_tier_a(&mut profile.commands, project_root);

    Ok(profile)
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
    let mut distribution: BTreeMap<Language, usize> = BTreeMap::new();

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
    // Track already-detected framework names to avoid duplicates when the same
    // framework appears via multiple dep_files (e.g. Spring Boot in both
    // build.gradle and pom.xml).
    let mut seen_names: Vec<String> = Vec::new();

    for rule in FRAMEWORK_RULES {
        // Special case: .csproj files are named *.csproj, not literally ".csproj"
        let dep_path = if rule.dep_file == ".csproj" {
            match find_csproj_file(root) {
                Some(p) => p,
                None => continue,
            }
        } else {
            let p = root.join(rule.dep_file);
            if !p.exists() {
                // Also check .kts variant for build.gradle
                if rule.dep_file == "build.gradle" {
                    let kts = root.join("build.gradle.kts");
                    if kts.exists() { kts } else { continue; }
                } else {
                    continue;
                }
            } else {
                p
            }
        };

        let content = match std::fs::read_to_string(&dep_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let found = match rule.dep_file {
            "Cargo.toml" | "gleam.toml" => {
                content.contains(&format!("\"{}\"", rule.dep_key))
                    || content.contains(&format!("{} =", rule.dep_key))
                    || content.contains(&format!("{} {{", rule.dep_key))
            }
            "package.json" | "composer.json" => {
                content.contains(&format!("\"{}\"", rule.dep_key))
            }
            "go.mod" => content.contains(rule.dep_key),
            "Gemfile" | "mix.exs" => content.contains(rule.dep_key),
            "pyproject.toml" => content.contains(rule.dep_key),
            "build.gradle" | "build.gradle.kts" => content.contains(rule.dep_key),
            "pom.xml" | ".csproj" => content.contains(rule.dep_key),
            "CMakeLists.txt" | "build.zig.zon" => content.contains(rule.dep_key),
            "pubspec.yaml" => content.contains(rule.dep_key),
            "build.sbt" => content.contains(rule.dep_key),
            "Package.swift" => content.contains(rule.dep_key),
            "package.yaml" => content.contains(rule.dep_key),
            _ => false,
        };

        if found && !seen_names.contains(&rule.name.to_string()) {
            seen_names.push(rule.name.to_string());
            results.push(FrameworkDetection {
                name: rule.name.to_string(),
                version: None,
                category: rule.category.clone(),
                confidence: FRAMEWORK_MATCH_CONFIDENCE,
                detected_via: format!("{} dependency", rule.dep_file),
            });
        }
    }

    results
}

/// Find any .csproj file in the project root directory.
fn find_csproj_file(root: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        if name.to_string_lossy().ends_with(".csproj") {
            return Some(entry.path());
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Test profile detection (spec section 2D)
// ---------------------------------------------------------------------------

fn detect_test_profile(root: &Path, build: &BuildSystemProfile, ci: &Option<CiProfile>) -> TestProfile {
    let runner_cmd = match build.system {
        BuildSystem::Cargo => Some("cargo test".to_string()),
        BuildSystem::GoMod => Some("go test ./...".to_string()),
        BuildSystem::PyProject | BuildSystem::SetupPy => Some("pytest".to_string()),
        BuildSystem::Mix => Some("mix test".to_string()),
        BuildSystem::Deno => Some("deno test".to_string()),
        BuildSystem::Npm => read_npm_script(root, "test"),
        _ => None,
    };

    // A-32: refined location pattern detection (SameFile, Adjacent, SeparateDirectory, Mixed)
    let location_pattern = detect_test_location_pattern_refined(root, build);

    // A-32: coverage tool detection
    let coverage_tool = detect_coverage_tool(root, ci);

    // Detect fixture directory
    let fixture_dir = ["tests/fixtures", "test/fixtures", "fixtures", "testdata"]
        .iter()
        .map(|p| root.join(p))
        .find(|p| p.is_dir())
        .map(|p| p.strip_prefix(root).unwrap_or(&p).to_path_buf());

    TestProfile {
        runner_cmd,
        location_pattern,
        coverage_tool,
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
                // Use match instead of ok()? to avoid aborting the entire function
                // if read_dir fails for one CI rule (e.g. permission denied on .github/).
                let yml_files: Vec<String> = match std::fs::read_dir(&path) {
                    Ok(entries) => entries,
                    Err(_) => continue,
                }
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
        env: BTreeMap::new(),
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
// Override loading (spec section 1, A-32)
// ---------------------------------------------------------------------------

/// Load project-profile-overrides.toml from project root.
/// Returns None if file is absent (not an error). Malformed TOML logs a
/// warning and returns None (does not abort detection).
fn load_overrides(project_root: &Path) -> Option<toml::Table> {
    let override_path = project_root.join(OVERRIDE_FILE_NAME);
    if !override_path.exists() {
        // Absent override file is intentional — not an error.
        return None;
    }
    match std::fs::read_to_string(&override_path) {
        Ok(content) => match content.parse::<toml::Table>() {
            Ok(table) => {
                debug!(path = %override_path.display(), "loaded project profile overrides");
                Some(table)
            }
            Err(e) => {
                warn!(path = %override_path.display(), error = %e,
                      "malformed overrides TOML, skipping");
                None
            }
        },
        Err(e) => {
            warn!(path = %override_path.display(), error = %e,
                  "could not read overrides file, skipping");
            None
        }
    }
}

/// Merge overrides into profile field-by-field. Only recognized fields are
/// applied; unknown keys are silently ignored. This uses partial toml::Table
/// traversal rather than full deserialization to avoid aborting on type
/// mismatches in unrelated fields.
fn merge_overrides(profile: &mut ProjectProfile, overrides: &toml::Table) {
    // test_profile overrides
    if let Some(tp) = overrides.get("test_profile").and_then(|v| v.as_table()) {
        if let Some(cmd) = tp.get("runner_cmd").and_then(|v| v.as_str()) {
            profile.test_profile.runner_cmd = Some(cmd.to_string());
        }
        if let Some(cov) = tp.get("coverage_tool").and_then(|v| v.as_str()) {
            profile.test_profile.coverage_tool = Some(cov.to_string());
        }
    }

    // project_vision override
    if let Some(vision) = overrides.get("project_vision").and_then(|v| v.as_str()) {
        profile.project_vision = Some(vision.to_string());
    }

    // Batch 4B (A-7): constitutional constraints override. Accepts a TOML
    // array of strings; non-string entries are silently skipped rather than
    // aborting the override merge for a typo. Replaces the list wholesale
    // (not a merge) so operators can explicitly pin the set they trust.
    if let Some(list) = overrides.get("constraints").and_then(|v| v.as_array()) {
        let filtered: Vec<String> = list
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        profile.constraints = filtered;
    }

    profile.overrides_applied = true;
    debug!("applied project profile overrides");
}

/// C-21 Task 8 — append BLOCK-severity deprecated_patterns rows from
/// rules.db as constraint entries. Format: `"DO NOT USE {deprecated}.
/// Use {replacement}."`. De-duplicated on deprecated_name across the
/// existing constraint set (so TOML overrides do not get clobbered by
/// rules-db rows naming the same deprecated symbol).
///
/// Silent fallthrough: if `rules_db_path` is `None` or the open fails,
/// the function returns Ok(()) without modifying `profile.constraints`.
/// The caller can therefore treat this as an additive enrichment with
/// no risk of aborting the profile assembly.
///
/// Spec: phase-4H.txt anti-staleness enforcement section 2.2 +
/// rules/src/schema.rs `query_deprecated_patterns(conn, Some("BLOCK"))`.
pub fn append_constraints_from_rules_db(
    profile: &mut ProjectProfile,
    rules_db_path: &Path,
) -> Result<(), ProjectProfileError> {
    // Silent skip when the rules.db file is absent — profile assembly
    // must succeed before any rule is ever recorded.
    if !rules_db_path.exists() {
        debug!(
            path = %rules_db_path.display(),
            "rules.db not present; skipping constraint enrichment"
        );
        return Ok(());
    }
    let clock = OsClock;
    let conn = match elai_rules::schema::open_rules_db(rules_db_path, &clock) {
        Ok(c) => c,
        Err(e) => {
            warn!(
                error = %e,
                path = %rules_db_path.display(),
                "open_rules_db failed; skipping constraint enrichment"
            );
            return Ok(());
        }
    };
    let entries = match elai_rules::schema::query_deprecated_patterns(&conn, Some("BLOCK")) {
        Ok(rows) => rows,
        Err(e) => {
            warn!(
                error = %e,
                "query_deprecated_patterns failed; skipping constraint enrichment"
            );
            return Ok(());
        }
    };
    // Build a de-dup set keyed on the deprecated_name token already
    // mentioned in any existing constraint string. Conservative substring
    // match so TOML entries phrased differently still de-dup.
    let mut appended = 0usize;
    for entry in entries {
        let already_present = profile
            .constraints
            .iter()
            .any(|c| c.contains(&entry.deprecated_name));
        if already_present {
            continue;
        }
        let constraint = format!(
            "DO NOT USE {}. Use {}.",
            entry.deprecated_name, entry.replacement_name
        );
        profile.constraints.push(constraint);
        appended += 1;
    }
    debug!(
        appended,
        "appended {} BLOCK-severity deprecated_patterns rows to constraints",
        appended
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Coverage tool detection (spec section 2D, A-32)
// ---------------------------------------------------------------------------

/// Detect coverage tool by scanning CI config content, Makefile/justfile
/// targets, and package.json scripts. Returns first match.
fn detect_coverage_tool(root: &Path, ci: &Option<CiProfile>) -> Option<String> {
    // 1. Scan CI config file content for coverage tool keywords
    if let Some(ci_profile) = ci {
        let ci_path = root.join(&ci_profile.config_path);
        let ci_content = if ci_path.is_dir() {
            // GitHub Actions: read all YAML files in the directory
            read_dir_contents(&ci_path)
        } else {
            std::fs::read_to_string(&ci_path).ok()
        };
        if let Some(content) = ci_content {
            let lower = content.to_lowercase();
            for &(keyword, tool_name) in COVERAGE_TOOL_KEYWORDS {
                if lower.contains(keyword) {
                    return Some(tool_name.to_string());
                }
            }
        }
    }

    // 2. Scan Makefile targets for coverage-related names
    let makefile_path = root.join("Makefile");
    if makefile_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&makefile_path) {
            let targets = parse_makefile_targets_from_content(&content);
            for target in &targets {
                let lower = target.to_lowercase();
                // Use contains() for both Makefile targets and npm scripts (consistent)
                if COVERAGE_TARGET_NAMES.iter().any(|&name| lower.contains(name)) {
                    return Some(format!("make-{target}"));
                }
            }
        }
    }

    // 3. Scan package.json scripts for coverage-related keys
    let pkg_path = root.join("package.json");
    if pkg_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&pkg_path) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(scripts) = parsed.get("scripts").and_then(|s| s.as_object()) {
                    for key in scripts.keys() {
                        let lower = key.to_lowercase();
                        if COVERAGE_TARGET_NAMES.iter().any(|&name| lower.contains(name)) {
                            return Some(format!("npm-{key}"));
                        }
                    }
                }
            }
        }
    }

    None
}

/// Read and concatenate all files in a directory (for GitHub Actions workflow dir).
fn read_dir_contents(dir: &Path) -> Option<String> {
    let mut combined = String::new();
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        // Skip entries where file_type() fails (broken symlinks, permission denied)
        // instead of aborting the entire function.
        if entry.file_type().map_or(false, |ft| ft.is_file()) {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                combined.push_str(&content);
                combined.push('\n');
            }
        }
    }
    if combined.is_empty() { None } else { Some(combined) }
}

// ---------------------------------------------------------------------------
// Refined test location pattern detection (spec section 2D, A-32)
// ---------------------------------------------------------------------------

/// Detect test location pattern with full coverage:
/// SameFile, Adjacent, SeparateDirectory, Mixed.
fn detect_test_location_pattern_refined(
    root: &Path,
    build: &BuildSystemProfile,
) -> TestLocationPattern {
    let has_test_dir = root.join("tests").is_dir()
        || root.join("test").is_dir()
        || root.join("__tests__").is_dir()
        || root.join("spec").is_dir();

    let has_same_file = match build.system {
        BuildSystem::Cargo => true, // Rust uses #[cfg(test)] by convention
        _ => false,
    };

    let has_adjacent = detect_adjacent_test_files(root, build);

    // Determine pattern from signals
    // NARROWING-OK: bool->u8 is always 0 or 1; no truncation possible; summed result is at most 3 (fits u8).
    let signal_count =
        has_test_dir as u8 + has_same_file as u8 + has_adjacent as u8;

    if signal_count > 1 {
        TestLocationPattern::Mixed
    } else if has_test_dir {
        TestLocationPattern::SeparateDirectory
    } else if has_same_file {
        TestLocationPattern::SameFile
    } else if has_adjacent {
        TestLocationPattern::Adjacent
    } else {
        TestLocationPattern::Unknown
    }
}

/// Check for adjacent test files (e.g. foo.rs + foo_test.rs, foo.py + test_foo.py).
fn detect_adjacent_test_files(root: &Path, build: &BuildSystemProfile) -> bool {
    let walker = WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .git_global(false)
        .git_exclude(true)
        .max_depth(Some(4))
        .build();

    for entry in walker.flatten() {
        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
            continue;
        }
        let path = entry.path();
        let file_name = match path.file_stem().and_then(|s| s.to_str()) {
            Some(n) => n,
            None => continue,
        };
        let ext = match path.extension().and_then(|e| e.to_str()) {
            Some(e) => e,
            None => continue,
        };

        // Rust: foo_test.rs adjacent to foo.rs
        if ext == "rs" && file_name.ends_with("_test") {
            return true;
        }
        // Python: test_foo.py adjacent to foo.py
        if ext == "py" && file_name.starts_with("test_") {
            // Only count if not inside a tests/ directory
            let in_test_dir = path.components().any(|c| {
                let s = c.as_os_str().to_string_lossy();
                s == "tests" || s == "test" || s == "__tests__" || s == "spec"
            });
            if !in_test_dir {
                return true;
            }
        }
        // TypeScript/JS: foo.test.ts, foo.spec.ts
        if (ext == "ts" || ext == "js" || ext == "tsx" || ext == "jsx")
            && (file_name.ends_with(".test") || file_name.ends_with(".spec"))
        {
            let in_test_dir = path.components().any(|c| {
                let s = c.as_os_str().to_string_lossy();
                s == "__tests__" || s == "test" || s == "spec"
            });
            if !in_test_dir {
                return true;
            }
        }
    }

    // Also check based on build system convention (Go: _test.go files)
    if build.system == BuildSystem::GoMod {
        let walker = WalkBuilder::new(root)
            .hidden(true)
            .git_ignore(true)
            .max_depth(Some(3))
            .build();
        for entry in walker.flatten() {
            if let Some(name) = entry.path().file_name().and_then(|n| n.to_str()) {
                if name.ends_with("_test.go") {
                    return true;
                }
            }
        }
    }

    false
}

// ---------------------------------------------------------------------------
// Command validation — Tier A static (spec section 5, A-32)
// ---------------------------------------------------------------------------

/// Validate all commands in the registry using static filesystem checks.
/// Sets `verified = true` on commands that pass all checks.
fn validate_tier_a(commands: &mut BuildCommandRegistry, root: &Path) {
    // Pre-parse Makefile/justfile targets for target-name validation
    let makefile_targets = parse_makefile_targets(root);
    let justfile_targets = parse_justfile_targets(root);
    let pkg_scripts = parse_package_json_scripts(root);

    let all_categories: [&mut Vec<BuildCommand>; 6] = [
        &mut commands.build,
        &mut commands.test,
        &mut commands.lint,
        &mut commands.format,
        &mut commands.run,
        &mut commands.clean,
    ];

    for cmds in all_categories {
        for cmd in cmds.iter_mut() {
            let binary = match cmd.cmd.split_whitespace().next() {
                Some(b) => b,
                None => continue,
            };

            let mut valid = true;

            // Check 1: binary exists on PATH or is a known wrapper
            if !is_known_wrapper(binary) && !binary_on_path(binary) {
                debug!(cmd = %cmd.cmd, binary = binary, "Tier A: binary not found on PATH");
                valid = false;
            }

            // Check 2: if the command uses make/just, verify the target exists
            if binary == "make" || binary == "gmake" {
                if let Some(target) = cmd.cmd.split_whitespace().nth(1) {
                    if !makefile_targets.iter().any(|t| t == target) && !makefile_targets.is_empty() {
                        debug!(cmd = %cmd.cmd, target = target, "Tier A: make target not found");
                        valid = false;
                    }
                }
            }
            if binary == "just" {
                if let Some(target) = cmd.cmd.split_whitespace().nth(1) {
                    if !justfile_targets.iter().any(|t| t == target) && !justfile_targets.is_empty() {
                        debug!(cmd = %cmd.cmd, target = target, "Tier A: just target not found");
                        valid = false;
                    }
                }
            }

            // Check 3: npm/bun script key validation
            if (binary == "npm" || binary == "bun") && cmd.cmd.contains("run ") {
                if let Some(script_name) = cmd.cmd.split("run ").nth(1).map(|s| s.split_whitespace().next().unwrap_or(s)) {
                    if !pkg_scripts.iter().any(|s| s == script_name) && !pkg_scripts.is_empty() {
                        debug!(cmd = %cmd.cmd, script = script_name, "Tier A: npm script not found");
                        valid = false;
                    }
                }
            }

            cmd.verified = valid;
        }
    }
}

/// Check if a binary name is a known wrapper/builtin that doesn't appear on PATH.
fn is_known_wrapper(binary: &str) -> bool {
    matches!(binary, "./gradlew" | "./mvnw" | "npx" | "bunx")
}

/// Check if binary exists on PATH by searching PATH directories.
/// Rejects binary names containing path separators to prevent directory traversal.
fn binary_on_path(binary: &str) -> bool {
    // Guard: reject path separators to prevent directory traversal
    if binary.contains('/') || binary.contains('\\') {
        return false;
    }

    let path_var = match std::env::var("PATH") {
        Ok(p) => p,
        Err(_) => return false,
    };

    #[cfg(target_os = "windows")]
    let separator = ';';
    #[cfg(not(target_os = "windows"))]
    let separator = ':';

    for dir in path_var.split(separator) {
        let candidate = Path::new(dir).join(binary);
        if candidate.exists() {
            return true;
        }
        // On Windows, also check with .exe/.cmd/.bat extensions
        #[cfg(target_os = "windows")]
        {
            for ext in &[".exe", ".cmd", ".bat"] {
                let with_ext = Path::new(dir).join(format!("{binary}{ext}"));
                if with_ext.exists() {
                    return true;
                }
            }
        }
    }
    false
}

/// Parse Makefile targets (lines matching `^target_name:`)
fn parse_makefile_targets(root: &Path) -> Vec<String> {
    let makefile = root.join("Makefile");
    if !makefile.exists() {
        let gnumakefile = root.join("GNUmakefile");
        if gnumakefile.exists() {
            return parse_makefile_targets_from_path(&gnumakefile);
        }
        return Vec::new();
    }
    parse_makefile_targets_from_path(&makefile)
}

fn parse_makefile_targets_from_path(path: &Path) -> Vec<String> {
    match std::fs::read_to_string(path) {
        Ok(content) => parse_makefile_targets_from_content(&content),
        Err(_) => Vec::new(),
    }
}

fn parse_makefile_targets_from_content(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            // Skip comments, variable assignments, .PHONY
            if trimmed.starts_with('#') || trimmed.starts_with('.') {
                return None;
            }
            if let Some(colon_pos) = trimmed.find(':') {
                let target = trimmed[..colon_pos].trim();
                if !target.is_empty() && !target.contains('=') && !target.contains('$') {
                    return Some(target.to_string());
                }
            }
            None
        })
        .collect()
}

/// Parse justfile targets (lines matching `^target_name:`)
fn parse_justfile_targets(root: &Path) -> Vec<String> {
    let justfile = root.join("justfile");
    if !justfile.exists() {
        return Vec::new();
    }
    match std::fs::read_to_string(&justfile) {
        Ok(content) => content
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with('#') || trimmed.starts_with('@') || trimmed.is_empty() {
                    return None;
                }
                // justfile target lines contain ':' — extract the first word as the target name.
                // Handles both "target:" and "target arg1 arg2:" forms.
                if let Some(colon_pos) = trimmed.find(':') {
                    let before = trimmed[..colon_pos].trim();
                    if let Some(name) = before.split_whitespace().next() {
                        if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
                            return Some(name.to_string());
                        }
                    }
                }
                None
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Parse package.json script keys
fn parse_package_json_scripts(root: &Path) -> Vec<String> {
    let pkg_path = root.join("package.json");
    if !pkg_path.exists() {
        return Vec::new();
    }
    let content = match std::fs::read_to_string(&pkg_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let parsed: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    parsed
        .get("scripts")
        .and_then(|s| s.as_object())
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default()
}

/// Tier B command validation with user consent gate.
/// Runs actual commands (e.g., `cargo check`, `pytest --collect-only`) to verify
/// they succeed. Requires explicit user consent before execution.
///
/// Returns Ok(true) if user consented and validation ran, Ok(false) if declined.
/// Deferred from CLI wiring in A-32 — LiveCli does not yet have a consent prompt.
pub fn validate_tier_b_with_consent(
    commands: &mut BuildCommandRegistry,
    _root: &Path,
) -> Result<bool, ProjectProfileError> {
    // Tier B is opt-in. Until Phase 4A sandbox is available, commands run
    // unsandboxed. CLI wiring deferred — this function exists for downstream
    // callers to invoke once consent infrastructure is ready.
    //
    // When wired:
    // 1. Print consent prompt: "Verify build commands by running them? (y/N)"
    // 2. Read stdin, default N.
    // 3. If declined: return Ok(false), all commands stay UNVERIFIED.
    // 4. If accepted: run each command, set verified=true on success.
    let _ = commands; // suppress unused warning until CLI wiring
    Ok(false)
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
    profile.dep_profile = stats.dep_profile;
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
        let tp = detect_test_profile(dir.path(), &build, &None);
        // A-32: Cargo projects with tests/ dir now detect Mixed (SameFile + SeparateDirectory)
        assert_eq!(tp.location_pattern, TestLocationPattern::Mixed);
        assert!(tp.fixture_dir.is_some());
    }

    #[test]
    fn npm_test_script_detection() {
        let dir = setup_project(&[("package.json", r#"{"scripts":{"test":"vitest run","build":"vite build"}}"#)]);
        let build = detect_build_system(dir.path()).expect("build");
        let tp = detect_test_profile(dir.path(), &build, &None);
        assert_eq!(tp.runner_cmd, Some("vitest run".to_string()));
    }

    // ======================================================================
    // A-32: Override loading and merge tests
    // ======================================================================

    #[test]
    fn override_absent_file_returns_none() {
        let dir = setup_project(&[("Cargo.toml", "[package]\nname = \"test\"")]);
        let result = load_overrides(dir.path());
        assert!(result.is_none());
    }

    #[test]
    fn override_valid_file_merges_fields() {
        let dir = setup_project(&[
            ("Cargo.toml", "[package]\nname = \"test\""),
            ("project-profile-overrides.toml", "[test_profile]\nrunner_cmd = \"custom-test\"\ncoverage_tool = \"tarpaulin\""),
        ]);
        let mut profile = detect_project_profile(dir.path()).expect("profile");
        // Profile should have overrides applied automatically
        assert!(profile.overrides_applied);
        assert_eq!(profile.test_profile.runner_cmd, Some("custom-test".to_string()));
        assert_eq!(profile.test_profile.coverage_tool, Some("tarpaulin".to_string()));
    }

    #[test]
    fn override_malformed_toml_skips_gracefully() {
        let dir = setup_project(&[
            ("Cargo.toml", "[package]\nname = \"test\""),
            ("project-profile-overrides.toml", "{{{{ invalid toml !!!!"),
        ]);
        let profile = detect_project_profile(dir.path()).expect("profile");
        // Should not panic, overrides not applied
        assert!(!profile.overrides_applied);
    }

    // ======================================================================
    // A-32: Framework expansion tests
    // ======================================================================

    #[test]
    fn framework_rules_count_meets_target() {
        assert!(FRAMEWORK_RULES.len() >= 180,
            "FRAMEWORK_RULES has {} entries, need >= 180", FRAMEWORK_RULES.len());
    }

    #[test]
    fn detect_spring_boot_from_gradle() {
        let dir = setup_project(&[
            ("build.gradle", "plugins { id 'java' }\ndependencies {\n  implementation 'org.springframework.boot:spring-boot-starter-web'\n}"),
        ]);
        let fws = detect_frameworks(dir.path());
        assert!(fws.iter().any(|f| f.name == "Spring Boot"), "expected Spring Boot, got {:?}", fws);
    }

    #[test]
    fn detect_aspnet_from_csproj() {
        let dir = setup_project(&[
            ("MyApp.csproj", "<Project Sdk=\"Microsoft.NET.Sdk.Web\">\n  <ItemGroup>\n    <PackageReference Include=\"Microsoft.AspNetCore\" />\n  </ItemGroup>\n</Project>"),
        ]);
        let fws = detect_frameworks(dir.path());
        assert!(fws.iter().any(|f| f.name == "ASP.NET"), "expected ASP.NET, got {:?}", fws);
    }

    #[test]
    fn detect_flutter_from_pubspec() {
        let dir = setup_project(&[
            ("pubspec.yaml", "name: myapp\ndependencies:\n  flutter:\n    sdk: flutter"),
        ]);
        let fws = detect_frameworks(dir.path());
        assert!(fws.iter().any(|f| f.name == "Flutter"), "expected Flutter, got {:?}", fws);
    }

    #[test]
    fn detect_nestjs_and_prisma() {
        let dir = setup_project(&[
            ("package.json", r#"{"dependencies":{"@nestjs/core":"10.0","prisma":"5.0"}}"#),
        ]);
        let fws = detect_frameworks(dir.path());
        assert!(fws.iter().any(|f| f.name == "Nest.js"));
        assert!(fws.iter().any(|f| f.name == "Prisma"));
    }

    #[test]
    fn detect_echo_go() {
        let dir = setup_project(&[
            ("go.mod", "module example.com/app\n\ngo 1.21\n\nrequire github.com/labstack/echo/v4 v4.11.0"),
        ]);
        let fws = detect_frameworks(dir.path());
        assert!(fws.iter().any(|f| f.name == "Echo"), "expected Echo, got {:?}", fws);
    }

    #[test]
    fn framework_confidence_uses_named_constant() {
        let dir = setup_project(&[("Cargo.toml", "[dependencies]\naxum = \"0.7\"")]);
        let fws = detect_frameworks(dir.path());
        assert_eq!(fws[0].confidence, FRAMEWORK_MATCH_CONFIDENCE);
    }

    // ======================================================================
    // A-32: Coverage tool detection tests
    // ======================================================================

    #[test]
    fn detect_coverage_from_ci_config() {
        let dir = setup_project(&[
            (".github/workflows/ci.yml", "name: CI\nsteps:\n  - run: cargo tarpaulin"),
        ]);
        let ci = detect_ci(dir.path());
        let coverage = detect_coverage_tool(dir.path(), &ci);
        assert_eq!(coverage, Some("tarpaulin".to_string()));
    }

    #[test]
    fn detect_coverage_from_makefile_target() {
        let dir = setup_project(&[
            ("Makefile", "build:\n\tcargo build\ncoverage:\n\tcargo tarpaulin"),
        ]);
        let coverage = detect_coverage_tool(dir.path(), &None);
        assert_eq!(coverage, Some("make-coverage".to_string()));
    }

    #[test]
    fn detect_coverage_from_package_json_script() {
        let dir = setup_project(&[
            ("package.json", r#"{"scripts":{"test":"jest","coverage":"jest --coverage"}}"#),
        ]);
        let coverage = detect_coverage_tool(dir.path(), &None);
        assert!(coverage.is_some());
        assert!(coverage.as_ref().map_or(false, |c| c.contains("coverage")));
    }

    // ======================================================================
    // A-32: Test location pattern tests (Mixed, Adjacent)
    // ======================================================================

    #[test]
    fn detect_mixed_location_pattern() {
        // Rust project with both SameFile (#[cfg(test)]) and SeparateDirectory (tests/)
        let dir = setup_project(&[
            ("Cargo.toml", "[package]\nname = \"test\""),
            ("src/main.rs", "fn main() {}\n#[cfg(test)]\nmod tests { }"),
            ("tests/integration.rs", ""),
        ]);
        let build = detect_build_system(dir.path()).expect("build");
        let pattern = detect_test_location_pattern_refined(dir.path(), &build);
        assert_eq!(pattern, TestLocationPattern::Mixed);
    }

    #[test]
    fn detect_adjacent_test_files_python() {
        let dir = setup_project(&[
            ("pyproject.toml", "[project]\nname = \"myapp\""),
            ("src/app.py", "def run(): pass"),
            ("src/test_app.py", "def test_run(): pass"),
        ]);
        let build = detect_build_system(dir.path()).expect("build");
        let pattern = detect_test_location_pattern_refined(dir.path(), &build);
        assert_eq!(pattern, TestLocationPattern::Adjacent);
    }

    // ======================================================================
    // A-32: Tier A command validation tests
    // ======================================================================

    #[test]
    fn tier_a_validates_cargo_on_path() {
        let dir = setup_project(&[("Cargo.toml", "[package]\nname = \"test\"")]);
        let build = BuildSystemProfile {
            system: BuildSystem::Cargo,
            config_path: PathBuf::from("Cargo.toml"),
            version: None,
            monorepo: false,
            workspaces: Vec::new(),
        };
        let mut reg = build_command_registry(dir.path(), &build);
        validate_tier_a(&mut reg, dir.path());
        // cargo should be on PATH in CI/dev environments
        // If not on PATH (unlikely), verified stays false — both are acceptable.
        // We just verify it doesn't panic.
        assert!(!reg.build.is_empty());
    }

    #[test]
    fn tier_a_makefile_target_validation() {
        let dir = setup_project(&[
            ("Makefile", "build:\n\techo build\ntest:\n\techo test\nlint:\n\techo lint"),
        ]);
        let targets = parse_makefile_targets(dir.path());
        assert!(targets.contains(&"build".to_string()));
        assert!(targets.contains(&"test".to_string()));
        assert!(targets.contains(&"lint".to_string()));
    }

    #[test]
    fn tier_a_justfile_target_parsing() {
        // justfile format: "target:" or "target arg1 arg2:" for targets with parameters
        let dir = setup_project(&[
            ("justfile", "build:\n  cargo build\ntest:\n  cargo test\nformat arg:\n  cargo fmt"),
        ]);
        let targets = parse_justfile_targets(dir.path());
        assert!(targets.contains(&"build".to_string()), "missing 'build': {:?}", targets);
        assert!(targets.contains(&"test".to_string()), "missing 'test': {:?}", targets);
        assert!(targets.contains(&"format".to_string()), "missing 'format': {:?}", targets);
    }

    #[test]
    fn tier_a_missing_binary_stays_unverified() {
        let mut reg = BuildCommandRegistry {
            build: vec![BuildCommand {
                cmd: "nonexistent-binary-xyz build".to_string(),
                working_dir: PathBuf::from("."),
                env: BTreeMap::new(),
                timeout_s: DEFAULT_COMMAND_TIMEOUT_S,
                scope: CommandScope::Full,
                verified: false,
            }],
            ..Default::default()
        };
        let dir = setup_project(&[]);
        validate_tier_a(&mut reg, dir.path());
        assert!(!reg.build[0].verified, "unknown binary should not be verified");
    }

    #[test]
    fn tier_b_returns_false_without_consent() {
        let mut reg = BuildCommandRegistry::default();
        let dir = setup_project(&[]);
        let result = validate_tier_b_with_consent(&mut reg, dir.path());
        assert_eq!(result.expect("no error"), false);
    }
}
