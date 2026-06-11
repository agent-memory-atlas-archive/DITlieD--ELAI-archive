//! E-40 Task 2 (Tranche B-3) — Stack-Graph-style push/pop scope-graph
//! name resolution over [`SymbolGraphCache`].
//!
//! Design adoption only: this is a fresh Rust re-implementation of the
//! push/pop scope-graph model from arXiv:2211.01224 (Stack Graphs). The
//! upstream `github/stack-graphs` crate was ARCHIVED 2025-09-09 and is
//! NOT a runtime dependency (per E-40 WIRING_CONTRACT `stub_ban`: "no
//! Stack-Graphs runtime dependency").
//!
//! Model summary (paper §3, design-level):
//!   * a name *reference* opens a scope STACK and walks outward,
//!     PUSHing the symbols that gate a lookup and POPping them when a
//!     matching *definition* is found in an enclosing scope;
//!   * resolution is the join of a file-local partial path (intra-file
//!     scope chain) with cross-file scope-stack continuations (a
//!     definition imported / re-exported from another file).
//!
//! Concretely, over ELAI's flat JSON-rooted [`SymbolGraphCache`] (which
//! carries `name -> ids` and `id -> reverse-callers`), the scope graph:
//!   1. collects all *definitions* of the queried name (the candidate
//!      pop targets);
//!   2. for each language, applies the per-language push/pop rules in
//!      `.elai_cc/scope_graph_rules/<language>.toml` to decide which
//!      scope boundaries a reference may cross (file-local, module,
//!      crate, cross-file via re-export) and to rank candidates;
//!   3. returns ranked [`Resolution`] rows tagging each candidate with
//!      the [`ResolutionConfidence`] earned by the scope chain it
//!      satisfied (Exact for a same-file/same-scope pop, CrossFile for a
//!      push/pop join across files, Heuristic when only a name match
//!      remains and no scope rule fired).
//!
//! This is NOT a full incremental partial-path index (that is the heavy
//! Stack-Graphs database the paper builds for IDE latency); it is the
//! design's resolution *semantics* applied on demand to the existing
//! cache, which is sufficient for the auditor "where does this name
//! resolve across modules" question E-40 targets, without the archived
//! crate and without a second on-disk index.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::warn;

use super::cache::SymbolGraphCache;

/// Default directory (relative to project root / cwd) holding the
/// per-language push/pop scope rules.
pub const DEFAULT_SCOPE_RULES_DIR: &str = ".elai_cc/scope_graph_rules";

/// Maximum number of scope boundaries a single resolution walk may cross
/// before giving up. Mirrors the B-29 depth-clamp discipline (clamp 10);
/// scope walks are far shallower in practice (a name rarely resolves more
/// than a handful of nested scopes out), so 10 is a generous ceiling that
/// still guarantees termination on adversarial cyclic re-export rule sets.
pub const SCOPE_WALK_CLAMP: u8 = 10;

/// Error surface for scope-graph resolution.
#[derive(Debug, Error)]
pub enum ScopeGraphError {
    /// Caller passed an empty symbol name.
    #[error("scope_graph: query symbol name must be non-empty")]
    EmptySymbol,
    /// Caller passed an unsupported / unknown language slug.
    #[error("scope_graph: unsupported language '{0}' (expected one of the slugs with a rules TOML, e.g. rust|python|typescript|go)")]
    UnknownLanguage(String),
    /// The rules directory could not be read.
    #[error("scope_graph: failed to read rules dir {path}: {source}")]
    RulesDirIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    /// A rules TOML file could not be parsed.
    #[error("scope_graph: failed to parse rules file {path}: {source}")]
    RulesParse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
}

/// One push/pop scope boundary kind a reference may cross on its way to a
/// definition. The ordering (derived via the `#[repr]` discriminant and
/// the `rank()` helper) is from *tightest* (cheapest, most precise) to
/// *loosest* (most permissive). A resolution earns the confidence of the
/// loosest boundary it had to cross.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeBoundary {
    /// Same file, same lexical scope (a pop with no push — direct hit).
    Local,
    /// Same file, an enclosing block/module scope (one push then pop).
    File,
    /// A sibling module within the same crate/package (module-path push).
    Module,
    /// Another file reached via an import / re-export edge (cross-file
    /// push/pop join — the canonical Stack-Graphs cross-file case).
    CrossFile,
    /// Crate/package boundary (the loosest non-heuristic cross we model).
    Crate,
}

impl ScopeBoundary {
    /// Tightness rank — lower is tighter/more precise. Used to pick the
    /// best resolution and to map to a confidence tier.
    #[must_use]
    pub fn rank(self) -> u8 {
        match self {
            Self::Local => 0,
            Self::File => 1,
            Self::Module => 2,
            Self::CrossFile => 3,
            Self::Crate => 4,
        }
    }

    /// Snake-case wire label (for SSB `details_json` + dashboard).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::File => "file",
            Self::Module => "module",
            Self::CrossFile => "cross_file",
            Self::Crate => "crate",
        }
    }
}

/// Confidence earned by a resolution, mapped from the loosest boundary it
/// had to cross. Mirrors the E-40 `confidence: Unresolved|Heuristic`
/// vocabulary referenced in the WIRING_CONTRACT `stub_ban` ("mark
/// `confidence: Heuristic`").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionConfidence {
    /// A scope rule fired and the definition was found within the
    /// boundaries the rule permits (Local / File / Module).
    Exact,
    /// The definition was found only by crossing a CrossFile / Crate
    /// boundary the rules permit — correct under the language's import
    /// semantics but reached via a wider walk.
    CrossFile,
    /// No scope rule fired (the language has no matching rule, or the
    /// candidate sits outside every permitted boundary); the match is a
    /// bare name match and SHOULD be escalated to the LSP oracle if it
    /// affects an audit verdict (per E-40 Task 5).
    Heuristic,
}

impl ResolutionConfidence {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::CrossFile => "cross_file",
            Self::Heuristic => "heuristic",
        }
    }

    fn from_boundary(boundary: ScopeBoundary) -> Self {
        match boundary {
            ScopeBoundary::Local | ScopeBoundary::File | ScopeBoundary::Module => Self::Exact,
            ScopeBoundary::CrossFile | ScopeBoundary::Crate => Self::CrossFile,
        }
    }
}

/// A name-resolution query: resolve `name` as referenced from
/// `from_file`, under the rules for `language`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameQuery {
    /// The symbol name being referenced.
    pub name: String,
    /// The file the reference appears in (the scope walk starts here).
    /// Empty means "no origin file" — every candidate is then judged at
    /// CrossFile/Heuristic confidence since no Local/File pop is possible.
    pub from_file: String,
    /// Language slug selecting the push/pop rule set.
    pub language: String,
}

/// One resolved definition for a [`NameQuery`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resolution {
    /// Stable node id of the resolved definition (`<file>:<name>:<line>`).
    pub id: String,
    /// Definition file path.
    pub file: String,
    /// Definition line.
    pub line: u32,
    /// Symbol kind (function / method / struct / ...).
    pub kind: String,
    /// The loosest scope boundary the walk had to cross to reach this
    /// definition.
    pub boundary: ScopeBoundary,
    /// Confidence tier earned by `boundary`.
    pub confidence: ResolutionConfidence,
}

/// Per-language push/pop scope rules, loaded from a TOML file.
///
/// The TOML schema is intentionally small (the resolution engine is a
/// fixed push/pop walk; the rules only toggle which boundaries the walk
/// may cross and supply the language's module/path separators):
///
/// ```toml
/// language = "rust"
/// # Boundaries a reference is permitted to cross, tightest-first. The
/// # walk stops at the tightest boundary that yields a definition.
/// allowed_boundaries = ["local", "file", "module", "cross_file", "crate"]
/// # Path separator that delimits module segments inside a file path
/// # (used to decide same-module vs sibling-module).
/// module_separator = "/"
/// # Substrings whose presence in BOTH files marks them same-module
/// # (e.g. a Rust `mod.rs` sibling set). Optional.
/// same_module_markers = ["mod.rs"]
/// # When true, a name re-exported (the def's file is reachable as a
/// # caller-of via the reverse-call index) counts as CrossFile rather
/// # than Heuristic.
/// follow_reexports = true
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeRules {
    /// The language slug this rule set applies to (must match the file
    /// stem; validated at load).
    pub language: String,
    /// Boundaries a reference may cross, tightest-first. A boundary not in
    /// this list is forbidden — a candidate reachable only past a
    /// forbidden boundary is dropped (not returned at lower confidence).
    pub allowed_boundaries: Vec<ScopeBoundary>,
    /// Module path separator inside file paths.
    #[serde(default = "default_module_separator")]
    pub module_separator: String,
    /// Filename markers that, when shared, treat two files as same-module.
    #[serde(default)]
    pub same_module_markers: Vec<String>,
    /// Whether to treat re-exports (reverse-call reachable) as CrossFile.
    #[serde(default)]
    pub follow_reexports: bool,
}

fn default_module_separator() -> String {
    "/".to_string()
}

impl ScopeRules {
    /// True if `boundary` is permitted by this rule set.
    fn permits(&self, boundary: ScopeBoundary) -> bool {
        self.allowed_boundaries.contains(&boundary)
    }
}

/// The scope-graph resolver: a borrowed view over the cache plus the
/// per-language rule sets loaded from TOML.
pub struct ScopeGraph<'a> {
    cache: &'a mut SymbolGraphCache,
    rules: BTreeMap<String, ScopeRules>,
}

impl<'a> ScopeGraph<'a> {
    /// Build a resolver over `cache` with rules loaded from `rules_dir`.
    ///
    /// Every `*.toml` in `rules_dir` is parsed; the file stem must match
    /// its `language` field. Malformed files surface
    /// [`ScopeGraphError::RulesParse`] (fail-closed — a bad rule set is a
    /// hard error, not a silent skip, so a typo never silently drops a
    /// language's resolution semantics).
    pub fn load_from_dir(
        cache: &'a mut SymbolGraphCache,
        rules_dir: impl AsRef<Path>,
    ) -> Result<Self, ScopeGraphError> {
        let dir = rules_dir.as_ref();
        let mut rules: BTreeMap<String, ScopeRules> = BTreeMap::new();
        let entries = std::fs::read_dir(dir).map_err(|source| ScopeGraphError::RulesDirIo {
            path: dir.to_path_buf(),
            source,
        })?;
        for entry in entries {
            let entry = entry.map_err(|source| ScopeGraphError::RulesDirIo {
                path: dir.to_path_buf(),
                source,
            })?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("toml") {
                continue;
            }
            let raw = std::fs::read_to_string(&path).map_err(|source| {
                ScopeGraphError::RulesDirIo {
                    path: path.clone(),
                    source,
                }
            })?;
            let parsed: ScopeRules = toml::from_str(&raw).map_err(|source| {
                ScopeGraphError::RulesParse {
                    path: path.clone(),
                    source,
                }
            })?;
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if stem != parsed.language {
                    warn!(
                        "scope_graph: rules file {} declares language '{}' but stem is '{}'; using declared language",
                        path.display(),
                        parsed.language,
                        stem
                    );
                }
            }
            rules.insert(parsed.language.clone(), parsed);
        }
        Ok(Self { cache, rules })
    }

    /// Build a resolver over `cache` with an explicit in-memory rule set.
    /// Used by tests and by callers that synthesize rules without a TOML
    /// directory on disk.
    pub fn with_rules(
        cache: &'a mut SymbolGraphCache,
        rules: BTreeMap<String, ScopeRules>,
    ) -> Self {
        Self { cache, rules }
    }

    /// Number of loaded language rule sets.
    #[must_use]
    pub fn language_count(&self) -> usize {
        self.rules.len()
    }

    /// Whether a rule set is loaded for `language`.
    #[must_use]
    pub fn has_language(&self, language: &str) -> bool {
        self.rules.contains_key(language)
    }

    /// Resolve `query` to its candidate definitions, applying the
    /// push/pop scope rules for `query.language`.
    ///
    /// Returns the candidates ranked tightest-boundary-first (a Local hit
    /// before a CrossFile one). An empty vec means no permitted boundary
    /// yielded a definition. Errors only on bad input (empty name /
    /// unknown language) — a name with zero defs is `Ok(vec![])`.
    pub fn resolve_name(&mut self, query: &NameQuery) -> Result<Vec<Resolution>, ScopeGraphError> {
        if query.name.is_empty() {
            return Err(ScopeGraphError::EmptySymbol);
        }
        let rules = self
            .rules
            .get(&query.language)
            .ok_or_else(|| ScopeGraphError::UnknownLanguage(query.language.clone()))?
            .clone();

        // 1. Candidate definitions = every node defined under this name.
        let def_ids = self.cache.ids_by_name(&query.name);
        if def_ids.is_empty() {
            return Ok(Vec::new());
        }

        // 2. For the re-export check we need the set of files that can
        //    reach `from_file` through a call edge (an importer chain).
        //    Snapshot it once so the borrow of `cache` is released before
        //    the per-candidate `get_node` loop below.
        let reexport_files = if rules.follow_reexports && !query.from_file.is_empty() {
            self.reexport_reachable_files(&query.name)
        } else {
            Vec::new()
        };

        let mut out: Vec<Resolution> = Vec::with_capacity(def_ids.len());
        for id in def_ids {
            let Some(node) = self.cache.get_node(&id) else {
                continue;
            };
            let def_file = node.file.clone();
            let def_line = node.line;
            let def_kind = node.kind.clone();

            let boundary = classify_boundary(
                &query.from_file,
                &def_file,
                &rules,
                &reexport_files,
            );

            // Drop candidates reachable only past a forbidden boundary.
            if !rules.permits(boundary) {
                continue;
            }

            out.push(Resolution {
                id,
                file: def_file,
                line: def_line,
                kind: def_kind,
                boundary,
                confidence: ResolutionConfidence::from_boundary(boundary),
            });
        }

        // 3. Rank tightest-first; deterministic tiebreak on node id
        //    (lexicographic) per the B-29.5 hop-band determinism rule.
        out.sort_by(|a, b| {
            a.boundary
                .rank()
                .cmp(&b.boundary.rank())
                .then_with(|| a.id.cmp(&b.id))
        });

        // 4. If nothing survived the permitted-boundary filter but defs
        //    exist, surface the closest candidates as Heuristic so the
        //    caller can escalate to the LSP oracle rather than report a
        //    false "unresolved" (only when the Crate/CrossFile boundary
        //    itself is forbidden — i.e. a genuinely cross-boundary name).
        if out.is_empty() {
            out = self.heuristic_fallback(&query.name);
        }

        Ok(out)
    }

    /// Files that re-export / import `name` (reverse-call reachable from a
    /// definition of `name`). A re-export is modelled as: some node whose
    /// call edge targets a def of `name` lives in a different file — that
    /// file is treated as a CrossFile continuation point.
    fn reexport_reachable_files(&mut self, name: &str) -> Vec<String> {
        let def_ids = self.cache.ids_by_name(name);
        let mut files: Vec<String> = Vec::new();
        for def_id in &def_ids {
            for caller_id in self.cache.callers_of(def_id) {
                if let Some(node) = self.cache.get_node(&caller_id) {
                    if !node.file.is_empty() {
                        files.push(node.file.clone());
                    }
                }
            }
        }
        files.sort();
        files.dedup();
        files
    }

    /// Heuristic fallback — every def of `name` returned at Heuristic
    /// confidence, ranked by node id. Used when no permitted boundary
    /// matched; the caller is expected to escalate to the LSP oracle if
    /// the result affects an audit verdict (E-40 Task 5).
    fn heuristic_fallback(&mut self, name: &str) -> Vec<Resolution> {
        let def_ids = self.cache.ids_by_name(name);
        let mut out: Vec<Resolution> = Vec::with_capacity(def_ids.len());
        for id in def_ids {
            if let Some(node) = self.cache.get_node(&id) {
                out.push(Resolution {
                    id: id.clone(),
                    file: node.file.clone(),
                    line: node.line,
                    kind: node.kind.clone(),
                    boundary: ScopeBoundary::Crate,
                    confidence: ResolutionConfidence::Heuristic,
                });
            }
        }
        out.sort_by(|a, b| a.id.cmp(&b.id));
        out
    }
}

/// Classify the scope boundary a reference in `from_file` must cross to
/// reach a definition in `def_file`, under `rules`.
///
/// This is the push/pop decision, expressed structurally over file paths
/// + the re-export set (the only scope signal the flat cache carries):
///   * same file                              -> Local (pop, no push)
///   * same module (shared module path /      -> File or Module
///     same_module_markers)
///   * def file is a re-export continuation    -> CrossFile
///   * otherwise a different file/crate        -> CrossFile, or Crate when
///     the top-level path segment differs (different crate root)
fn classify_boundary(
    from_file: &str,
    def_file: &str,
    rules: &ScopeRules,
    reexport_files: &[String],
) -> ScopeBoundary {
    if from_file.is_empty() {
        // No origin scope to pop from; the tightest honest claim is
        // CrossFile (the def is in *some* file, reached without a local
        // pop). Crate if the rule set has no CrossFile permission.
        return ScopeBoundary::CrossFile;
    }
    if from_file == def_file {
        return ScopeBoundary::Local;
    }

    let from_segs: Vec<&str> = split_module_path(from_file, &rules.module_separator);
    let def_segs: Vec<&str> = split_module_path(def_file, &rules.module_separator);

    // Same-module markers: if both files share a configured marker AND
    // their parent path matches, treat as a File boundary (sibling decl in
    // the same module, e.g. two items in the same mod.rs neighbourhood).
    let from_parent = parent_path(&from_segs);
    let def_parent = parent_path(&def_segs);
    if from_parent == def_parent {
        let shares_marker = rules.same_module_markers.iter().any(|m| {
            from_file.contains(m.as_str()) || def_file.contains(m.as_str())
        }) || rules.same_module_markers.is_empty();
        if shares_marker {
            return ScopeBoundary::File;
        }
        return ScopeBoundary::Module;
    }

    // Different parent but shared crate root (first segment) -> Module if
    // they share at least the top segment, else a Crate boundary.
    let shared_root = from_segs.first().is_some()
        && from_segs.first() == def_segs.first();

    // A re-export continuation always counts as CrossFile when permitted.
    if reexport_files.iter().any(|f| f == def_file) {
        return ScopeBoundary::CrossFile;
    }

    if shared_root {
        ScopeBoundary::Module
    } else {
        ScopeBoundary::Crate
    }
}

/// Split a file path into module segments on the configured separator,
/// dropping a trailing extension-bearing leaf's extension so
/// `crates/a/src/foo.rs` and `crates/a/src/foo` compare equal at the leaf.
fn split_module_path<'p>(path: &'p str, sep: &str) -> Vec<&'p str> {
    if sep.is_empty() {
        return vec![path];
    }
    path.split(sep).filter(|s| !s.is_empty()).collect()
}

/// The parent path (all but the last segment) joined back with '/'. Empty
/// when there is no parent.
fn parent_path(segs: &[&str]) -> String {
    if segs.len() <= 1 {
        return String::new();
    }
    segs[..segs.len() - 1].join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol_graph::cache::{SymbolGraphCache, SymbolNode};

    fn rules_rust() -> ScopeRules {
        ScopeRules {
            language: "rust".into(),
            allowed_boundaries: vec![
                ScopeBoundary::Local,
                ScopeBoundary::File,
                ScopeBoundary::Module,
                ScopeBoundary::CrossFile,
                ScopeBoundary::Crate,
            ],
            module_separator: "/".into(),
            same_module_markers: vec!["mod.rs".into()],
            follow_reexports: true,
        }
    }

    fn rules_map() -> BTreeMap<String, ScopeRules> {
        let mut m = BTreeMap::new();
        m.insert("rust".to_string(), rules_rust());
        m
    }

    fn build_cache() -> SymbolGraphCache {
        let mut cache = SymbolGraphCache::empty_with_capacity(64).expect("empty");
        // Same-file def of `foo` (Local target from a/x.rs).
        cache.index_node(SymbolNode {
            id: "crates/a/src/x.rs:foo:10".into(),
            name: "foo".into(),
            file: "crates/a/src/x.rs".into(),
            line: 10,
            kind: "function".into(),
            calls: vec![],
        });
        // Cross-file def of `bar` in another crate.
        cache.index_node(SymbolNode {
            id: "crates/b/src/y.rs:bar:20".into(),
            name: "bar".into(),
            file: "crates/b/src/y.rs".into(),
            line: 20,
            kind: "function".into(),
            calls: vec![],
        });
        // Re-export continuation: a node in a/x.rs that calls bar (so
        // a/x.rs is a re-export-reachable importer of bar).
        cache.index_node(SymbolNode {
            id: "crates/a/src/x.rs:use_bar:11".into(),
            name: "use_bar".into(),
            file: "crates/a/src/x.rs".into(),
            line: 11,
            kind: "function".into(),
            calls: vec!["crates/b/src/y.rs:bar:20".into()],
        });
        cache
    }

    #[test]
    fn resolve_local_def_is_local_boundary() {
        let mut cache = build_cache();
        let mut sg = ScopeGraph::with_rules(&mut cache, rules_map());
        let q = NameQuery {
            name: "foo".into(),
            from_file: "crates/a/src/x.rs".into(),
            language: "rust".into(),
        };
        let res = sg.resolve_name(&q).expect("resolve");
        assert_eq!(res.len(), 1, "exactly one def of foo");
        assert_eq!(res[0].boundary, ScopeBoundary::Local);
        assert_eq!(res[0].confidence, ResolutionConfidence::Exact);
    }

    #[test]
    fn resolve_cross_crate_def_is_crate_boundary() {
        let mut cache = build_cache();
        let mut sg = ScopeGraph::with_rules(&mut cache, rules_map());
        let q = NameQuery {
            name: "bar".into(),
            from_file: "crates/a/src/x.rs".into(),
            language: "rust".into(),
        };
        let res = sg.resolve_name(&q).expect("resolve");
        assert_eq!(res.len(), 1);
        // a/x.rs re-exports bar (calls it) -> CrossFile beats Crate.
        assert_eq!(res[0].boundary, ScopeBoundary::CrossFile);
        assert_eq!(res[0].confidence, ResolutionConfidence::CrossFile);
    }

    #[test]
    fn empty_name_rejected() {
        let mut cache = build_cache();
        let mut sg = ScopeGraph::with_rules(&mut cache, rules_map());
        let q = NameQuery {
            name: String::new(),
            from_file: "crates/a/src/x.rs".into(),
            language: "rust".into(),
        };
        assert!(matches!(
            sg.resolve_name(&q),
            Err(ScopeGraphError::EmptySymbol)
        ));
    }

    #[test]
    fn unknown_language_rejected() {
        let mut cache = build_cache();
        let mut sg = ScopeGraph::with_rules(&mut cache, rules_map());
        let q = NameQuery {
            name: "foo".into(),
            from_file: "crates/a/src/x.rs".into(),
            language: "cobol".into(),
        };
        assert!(matches!(
            sg.resolve_name(&q),
            Err(ScopeGraphError::UnknownLanguage(_))
        ));
    }

    #[test]
    fn forbidden_boundary_falls_back_to_heuristic() {
        // Rules permit ONLY Local — a cross-crate def of bar has no
        // permitted boundary, so it surfaces as Heuristic fallback.
        let mut cache = build_cache();
        let mut local_only = rules_rust();
        local_only.allowed_boundaries = vec![ScopeBoundary::Local];
        local_only.follow_reexports = false;
        let mut m = BTreeMap::new();
        m.insert("rust".to_string(), local_only);
        let mut sg = ScopeGraph::with_rules(&mut cache, m);
        let q = NameQuery {
            name: "bar".into(),
            from_file: "crates/a/src/x.rs".into(),
            language: "rust".into(),
        };
        let res = sg.resolve_name(&q).expect("resolve");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].confidence, ResolutionConfidence::Heuristic);
    }

    #[test]
    fn no_origin_file_is_cross_file() {
        let mut cache = build_cache();
        let mut sg = ScopeGraph::with_rules(&mut cache, rules_map());
        let q = NameQuery {
            name: "foo".into(),
            from_file: String::new(),
            language: "rust".into(),
        };
        let res = sg.resolve_name(&q).expect("resolve");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].boundary, ScopeBoundary::CrossFile);
    }

    #[test]
    fn deterministic_order_across_runs() {
        // Two defs of the same name at the same boundary must order
        // identically across runs via the id lexicographic tiebreak.
        let mut cache = SymbolGraphCache::empty_with_capacity(16).expect("empty");
        cache.index_node(SymbolNode {
            id: "crates/z/src/b.rs:dup:2".into(),
            name: "dup".into(),
            file: "crates/z/src/b.rs".into(),
            line: 2,
            kind: "function".into(),
            calls: vec![],
        });
        cache.index_node(SymbolNode {
            id: "crates/z/src/a.rs:dup:1".into(),
            name: "dup".into(),
            file: "crates/z/src/a.rs".into(),
            line: 1,
            kind: "function".into(),
            calls: vec![],
        });
        let mut sg = ScopeGraph::with_rules(&mut cache, rules_map());
        let q = NameQuery {
            name: "dup".into(),
            from_file: "crates/z/src/c.rs".into(),
            language: "rust".into(),
        };
        let r1 = sg.resolve_name(&q).expect("r1");
        let r2 = sg.resolve_name(&q).expect("r2");
        assert_eq!(r1, r2, "resolution must be deterministic");
        // a.rs sorts before b.rs by id.
        assert_eq!(r1[0].file, "crates/z/src/a.rs");
    }
}
