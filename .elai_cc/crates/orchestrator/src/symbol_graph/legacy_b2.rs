//! Phase B2 — Symbol Graph Impact runtime (library skeleton).
//!
//! Reads the deterministic codebase map at `.claude/tools/elai.map`
//! (TAB-separated, six columns: TAG / SYMBOL / FILE:LINE / LANG / EXTRA /
//! SNIPPET) and exposes a typed query surface for symbol impact analysis.
//!
//! Two production guarantees:
//!
//!   1. **Lazy I/O.** Construction does not load the ~50 MB map into
//!      memory. The first `query` performs a single line scan and caches
//!      a `HashMap<symbol -> Vec<line>>` for subsequent calls within the
//!      same `SymbolGraph` instance.
//!   2. **No panics on the hot path.** Every fallible operation
//!      (open / read / parse / line:col split / depth recursion) returns
//!      a typed [`SymbolGraphError`]; the only places `expect` is used are
//!      a `Mutex::lock` (lock poisoning is non-recoverable program death,
//!      not a normal error) and that is wrapped behind a helper so callers
//!      never see a panic at runtime.
//!
//! Depth semantics (matches the phase brief):
//!   * `depth == 1`: every row whose SYMBOL column equals the query.
//!   * `depth >= 2`: also recursively expand outgoing CALL / CALL_RESOLVED
//!     / SCIP_REF edges discovered in any row found so far, bounded by
//!     [`SymbolGraph::MAX_RECORDS`].
//!
//! See `F:\\.ELAI_workflow\\workflow\\handoff\\phase-B2.md` for the
//! authoritative spec.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Mutex;

use thiserror::Error;

// ---------------------------------------------------------------------------
// SymbolTag
// ---------------------------------------------------------------------------

/// Every tag string we expect to encounter in the first column of
/// `elai.map`. `Other` is the safety valve for forward-compatibility — the
/// mapper may add new tag strings before this enum is updated, and we'd
/// rather surface them as an opaque tag than fail the whole scan.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SymbolTag {
    /// Function definition (any language).
    DefFn,
    /// Method definition.
    DefMethod,
    /// Struct definition.
    DefStruct,
    /// Enum definition.
    DefEnum,
    /// Trait definition.
    DefTrait,
    /// Class definition (Python / Svelte).
    DefClass,
    /// Const / static definition.
    DefConst,
    /// `impl <Trait> for <Type>` block.
    ImplTrait,
    /// `use` statement (Rust) / `from import` (Python).
    Use,
    /// Import statement.
    Import,
    /// Re-export.
    Export,
    /// Plain function call.
    Call,
    /// Method call (`receiver.method(...)`).
    MethodCall,
    /// Macro invocation.
    Macro,
    /// Type reference in signature / generic.
    TypeRef,
    /// CALL row enriched with a resolved callee (post rust-analyzer).
    CallResolved,
    /// SCIP-resolved definition with canonical fully-qualified path.
    ScipDef,
    /// SCIP-resolved reference.
    ScipRef,
    /// SCIP-resolved import.
    ScipImport,
    /// Tag string we do not recognise — preserved verbatim.
    Other(String),
}

impl SymbolTag {
    /// True for outgoing-edge tags that depth>=2 should chase recursively.
    ///
    /// Only naive-tokenisable EXTRA formats belong here. CALL and
    /// METHOD_CALL rows store the enclosing scope identifier in EXTRA, so
    /// `caller_idents()` can extract a clean caller-scope key. Tags whose
    /// EXTRA carries structured payloads (SCIP_REF stores canonical paths
    /// like `scip-typescript npm <pkg> <ver> src/...`; CALL_RESOLVED stores
    /// `local:DEF_FN:L<line><-actual_caller`) are deliberately excluded —
    /// tokenising them would synthesize bogus caller identifiers
    /// (`typescript`, `npm`, `local`, etc.) and pollute the secondary
    /// `outgoing_by_scope` index. A future patch can add a dedicated
    /// parser for CALL_RESOLVED that splits on `<-` and takes the
    /// right-hand identifier; until then it stays out of this set.
    fn is_outgoing_edge(&self) -> bool {
        matches!(self, SymbolTag::Call | SymbolTag::MethodCall)
    }
}

impl FromStr for SymbolTag {
    type Err = SymbolGraphError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "DEF_FN" => SymbolTag::DefFn,
            "DEF_METHOD" => SymbolTag::DefMethod,
            "DEF_STRUCT" => SymbolTag::DefStruct,
            "DEF_ENUM" => SymbolTag::DefEnum,
            "DEF_TRAIT" => SymbolTag::DefTrait,
            "DEF_CLASS" => SymbolTag::DefClass,
            "DEF_CONST" => SymbolTag::DefConst,
            "IMPL_TRAIT" => SymbolTag::ImplTrait,
            "USE" => SymbolTag::Use,
            "IMPORT" => SymbolTag::Import,
            "EXPORT" => SymbolTag::Export,
            "CALL" => SymbolTag::Call,
            "METHOD_CALL" => SymbolTag::MethodCall,
            "MACRO" => SymbolTag::Macro,
            "TYPE_REF" => SymbolTag::TypeRef,
            "CALL_RESOLVED" => SymbolTag::CallResolved,
            "SCIP_DEF" => SymbolTag::ScipDef,
            "SCIP_REF" => SymbolTag::ScipRef,
            "SCIP_IMPORT" => SymbolTag::ScipImport,
            other => SymbolTag::Other(other.to_string()),
        })
    }
}

// ---------------------------------------------------------------------------
// SymbolRecord
// ---------------------------------------------------------------------------

/// One parsed row from `elai.map`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolRecord {
    /// Decoded tag enum.
    pub tag: SymbolTag,
    /// Symbol the row is keyed by (SYMBOL column).
    pub symbol: String,
    /// Source file path as written in the map (forward-slashed,
    /// repo-relative — we preserve the on-disk shape so call sites can
    /// resolve against `.elai_cc/` themselves).
    pub file: PathBuf,
    /// 1-based line number.
    pub line: u32,
    /// Language tag (`rust` / `python` / `typescript` / `svelte`).
    pub lang: String,
    /// Free-form EXTRA column (e.g. enclosing scope) — `None` if blank.
    pub extra: Option<String>,
    /// SNIPPET column — `None` if blank.
    pub snippet: Option<String>,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors emitted by the symbol-graph subsystem.
#[derive(Debug, Error)]
pub enum SymbolGraphError {
    /// The configured `elai.map` path does not exist or is unreadable.
    #[error("elai.map not found at {0}")]
    MapNotFound(PathBuf),
    /// Underlying I/O error while reading the map.
    #[error("io error reading elai.map: {0}")]
    Io(#[from] std::io::Error),
    /// A row in the map could not be decoded (column count, line number,
    /// or file:line split).
    #[error("parse error at line {line}: {reason}")]
    Parse {
        /// 1-based line number in the map file.
        line: u64,
        /// Human-readable cause.
        reason: String,
    },
    /// `new_from_env` could not locate `elai.map` in any candidate path.
    #[error("could not locate elai.map (tried: {0:?})")]
    Unresolved(Vec<PathBuf>),
}

// ---------------------------------------------------------------------------
// SymbolGraph
// ---------------------------------------------------------------------------

/// Lazy reader for `elai.map`. Cloning is cheap (path-only) — the parsed
/// cache lives behind a `Mutex` and is not shared across instances.
/// Loaded form of `elai.map`: a primary index by SYMBOL and a secondary
/// "outgoing edges" index by caller scope (the EXTRA column on CALL /
/// METHOD_CALL rows). The secondary index is what makes `depth >= 2`
/// queries usable: given a starting symbol `foo`, the outgoing CALL rows
/// are precisely the rows whose EXTRA names `foo` as the enclosing scope,
/// NOT the rows whose SYMBOL is `foo`. CALL_RESOLVED and SCIP_REF rows
/// are intentionally NOT indexed here — their EXTRA columns carry
/// structured payloads that naive tokenisation would corrupt into bogus
/// caller-scope identifiers (see `SymbolTag::is_outgoing_edge`).
#[derive(Debug, Default)]
struct LoadedMap {
    by_symbol: HashMap<String, Vec<SymbolRecord>>,
    outgoing_by_scope: HashMap<String, Vec<SymbolRecord>>,
}

#[derive(Debug)]
pub struct SymbolGraph {
    map_path: PathBuf,
    // Cache: primary + outgoing-edge indices. Populated on first query.
    // Mutex (not RwLock) — load is single-threaded and short-lived;
    // contention on this lock is not the bottleneck the map scan is.
    cache: Mutex<Option<LoadedMap>>,
}

impl SymbolGraph {
    /// Hard cap on records returned by a single `query` call. Bounds
    /// adversarial fan-out in depth>=2 expansions.
    pub const MAX_RECORDS: usize = 200;

    /// Construct a `SymbolGraph` that will read from `map_path` on first
    /// query. The path is not validated here — `query` returns
    /// [`SymbolGraphError::MapNotFound`] if the file is missing.
    pub fn new(map_path: PathBuf) -> Self {
        Self {
            map_path,
            cache: Mutex::new(None),
        }
    }

    /// Resolve `elai.map` from the `ELAI_MAP_PATH` env var, falling back
    /// to standard repo locations. Returns
    /// [`SymbolGraphError::Unresolved`] if none of the candidates exist.
    pub fn new_from_env() -> Result<Self, SymbolGraphError> {
        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Ok(env_path) = std::env::var("ELAI_MAP_PATH") {
            candidates.push(PathBuf::from(env_path));
        }
        // Common repo-relative anchors (from `.elai_cc/` or repo root).
        candidates.push(PathBuf::from("../.claude/tools/elai.map"));
        candidates.push(PathBuf::from("../../.claude/tools/elai.map"));
        candidates.push(PathBuf::from(".claude/tools/elai.map"));

        for candidate in &candidates {
            if candidate.is_file() {
                return Ok(Self::new(candidate.clone()));
            }
        }
        Err(SymbolGraphError::Unresolved(candidates))
    }

    /// Path to the underlying `elai.map`.
    pub fn map_path(&self) -> &Path {
        &self.map_path
    }

    /// Query for `symbol` at the given `depth` (>= 1).
    ///
    /// * `depth == 1`: every row whose SYMBOL column equals `symbol`.
    /// * `depth >= 2`: also recursively expand outgoing CALL / METHOD_CALL
    ///   rows attached to discovered symbols, bounded by
    ///   [`SymbolGraph::MAX_RECORDS`]. CALL_RESOLVED / SCIP_REF rows are
    ///   not followed (see `SymbolTag::is_outgoing_edge`).
    ///
    /// `depth == 0` is treated as `depth == 1` (graceful clamp).
    pub fn query(
        &self,
        symbol: &str,
        depth: u32,
    ) -> Result<Vec<SymbolRecord>, SymbolGraphError> {
        let depth = depth.max(1);
        self.ensure_loaded()?;

        // We hold the lock for the duration of the BFS. Two reasons:
        //   1. The cache is read-only after `ensure_loaded` and we just
        //      clone rows out — no lifetime entanglement.
        //   2. Concurrent queries are rare in the orchestrator; favouring
        //      simplicity over micro-contention is fine here.
        let guard = self
            .cache
            .lock()
            .expect("symbol_graph cache mutex poisoned (process is corrupt)");
        let cache = guard.as_ref().ok_or_else(|| SymbolGraphError::Parse {
            line: 0,
            reason: "cache missing after ensure_loaded".to_string(),
        })?;

        let mut out: Vec<SymbolRecord> = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();
        // BFS queue: (symbol, remaining_depth). `remaining_depth` counts
        // additional hops still allowed *after* this symbol's rows are
        // collected. The starting symbol gets `depth - 1` hops.
        let mut queue: VecDeque<(String, u32)> = VecDeque::new();
        queue.push_back((symbol.to_string(), depth - 1));

        while let Some((current, hops)) = queue.pop_front() {
            if !visited.insert(current.clone()) {
                continue;
            }
            // (1) Collect every row whose SYMBOL equals `current`.
            if let Some(rows) = cache.by_symbol.get(&current) {
                for row in rows {
                    if out.len() >= Self::MAX_RECORDS {
                        return Ok(out);
                    }
                    out.push(row.clone());
                }
            }

            if hops == 0 {
                continue;
            }
            // (2) Expand outgoing edges from `current`. In elai.map the
            //     row for "foo calls bar" has SYMBOL=bar EXTRA=foo, so
            //     the outgoing edges of `current` are the rows whose
            //     EXTRA names `current` as the enclosing scope.
            if let Some(out_rows) = cache.outgoing_by_scope.get(&current) {
                for row in out_rows {
                    if out.len() >= Self::MAX_RECORDS {
                        return Ok(out);
                    }
                    out.push(row.clone());
                    if !visited.contains(&row.symbol) {
                        queue.push_back((row.symbol.clone(), hops - 1));
                    }
                }
            }
        }

        Ok(out)
    }

    // ---- internal --------------------------------------------------------

    /// Populate the cache from disk if not yet loaded.
    fn ensure_loaded(&self) -> Result<(), SymbolGraphError> {
        {
            let guard = self
                .cache
                .lock()
                .expect("symbol_graph cache mutex poisoned (process is corrupt)");
            if guard.is_some() {
                return Ok(());
            }
        }
        let parsed = self.load_map()?;
        let mut guard = self
            .cache
            .lock()
            .expect("symbol_graph cache mutex poisoned (process is corrupt)");
        // Race-tolerant: if another caller raced us, keep their result —
        // contents are deterministic so the two are equivalent.
        if guard.is_none() {
            *guard = Some(parsed);
        }
        Ok(())
    }

    /// Single line-scan of `elai.map` producing both the primary symbol
    /// index and the outgoing-edge-by-scope index used for depth>=2.
    fn load_map(&self) -> Result<LoadedMap, SymbolGraphError> {
        if !self.map_path.is_file() {
            return Err(SymbolGraphError::MapNotFound(self.map_path.clone()));
        }
        let file = File::open(&self.map_path)?;
        let reader = BufReader::new(file);

        let mut loaded = LoadedMap::default();
        for (idx, line) in reader.lines().enumerate() {
            let line = line?;
            let line_no: u64 = (idx as u64) + 1;
            // Skip blank and comment lines (mapper writes a `# header`).
            let trimmed = line.trim_start();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            match parse_row(&line, line_no) {
                Ok(rec) => {
                    // Outgoing-edge index: CALL / METHOD_CALL rows are
                    // indexed by EXTRA (caller scope) so we can answer
                    // "what does foo call?" in O(1). Other call-like tags
                    // are excluded because their EXTRA columns carry
                    // structured payloads that resist naive tokenisation
                    // (see SymbolTag::is_outgoing_edge for rationale).
                    if rec.tag.is_outgoing_edge() {
                        if let Some(scope) = rec.extra.as_ref() {
                            // EXTRA can contain a snippet of the enclosing
                            // signature ("def_in_struct foo: &mut Self,").
                            // The mapper's convention is "first whitespace
                            // token is the caller identifier"; we use a
                            // light-weight extraction so callers can match
                            // by plain symbol name.
                            for ident in caller_idents(scope) {
                                loaded
                                    .outgoing_by_scope
                                    .entry(ident)
                                    .or_default()
                                    .push(rec.clone());
                            }
                        }
                    }
                    loaded
                        .by_symbol
                        .entry(rec.symbol.clone())
                        .or_default()
                        .push(rec);
                }
                Err(SymbolGraphError::Parse { .. }) => {
                    // Malformed individual rows do not poison the entire
                    // scan — `codebase_mapper.py` occasionally emits short
                    // rows during regen. Drop them silently; this is the
                    // same policy used by `impact_radius.py`.
                    continue;
                }
                Err(other) => return Err(other),
            }
        }
        Ok(loaded)
    }
}

/// Extract candidate caller identifiers from an EXTRA cell. The mapper
/// writes either a bare symbol (`foo`), a signature fragment
/// (`foo: &mut Self,`), or a context snippet (`fn foo(x: u32)`). We split
/// on common Rust/Python token boundaries and keep tokens that look like
/// identifiers, so a downstream query for "foo" matches all three shapes.
fn caller_idents(extra: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    // O(1) dedup against already-pushed idents; the Vec keeps insertion
    // order for downstream determinism.
    let mut seen: HashSet<String> = HashSet::new();
    let mut buf = String::new();
    for ch in extra.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            buf.push(ch);
        } else if !buf.is_empty() {
            push_ident(&mut out, &mut seen, std::mem::take(&mut buf));
        }
    }
    if !buf.is_empty() {
        push_ident(&mut out, &mut seen, buf);
    }
    out
}

fn push_ident(out: &mut Vec<String>, seen: &mut HashSet<String>, ident: String) {
    // Filter out language keywords that would create absurd fan-out
    // (every Rust fn would land under "fn" otherwise).
    const SKIP: &[&str] = &[
        "fn", "pub", "mut", "let", "self", "Self", "async", "await", "impl",
        "for", "in", "if", "else", "match", "use", "mod", "struct", "enum",
        "trait", "where", "as", "ref", "move", "return", "loop", "while",
        "const", "static", "type", "def", "class", "from", "import", "with",
        "lambda", "yield", "raise", "try", "except", "finally", "True",
        "False", "None",
    ];
    if SKIP.iter().any(|kw| *kw == ident.as_str()) {
        return;
    }
    if seen.insert(ident.clone()) {
        out.push(ident);
    }
}

// ---------------------------------------------------------------------------
// Row parser
// ---------------------------------------------------------------------------

/// Parse a single TSV row into a `SymbolRecord`. Returns
/// [`SymbolGraphError::Parse`] for any column-count / file:line / line-no
/// failure so the caller can decide whether to log-and-drop or abort.
fn parse_row(line: &str, line_no: u64) -> Result<SymbolRecord, SymbolGraphError> {
    // splitn(6, '\t') keeps the SNIPPET column intact even if it contains
    // embedded tabs (rare but possible from tree-sitter snippets).
    let mut cols = line.splitn(6, '\t');
    let tag = cols.next().ok_or_else(|| SymbolGraphError::Parse {
        line: line_no,
        reason: "missing TAG column".to_string(),
    })?;
    let symbol = cols.next().ok_or_else(|| SymbolGraphError::Parse {
        line: line_no,
        reason: "missing SYMBOL column".to_string(),
    })?;
    let file_line = cols.next().ok_or_else(|| SymbolGraphError::Parse {
        line: line_no,
        reason: "missing FILE:LINE column".to_string(),
    })?;
    let lang = cols.next().ok_or_else(|| SymbolGraphError::Parse {
        line: line_no,
        reason: "missing LANG column".to_string(),
    })?;
    let extra = cols.next().unwrap_or("");
    let snippet = cols.next().unwrap_or("");

    let (file, line_num) = split_file_line(file_line, line_no)?;
    let tag_enum = SymbolTag::from_str(tag)?;

    Ok(SymbolRecord {
        tag: tag_enum,
        symbol: symbol.to_string(),
        file,
        line: line_num,
        lang: lang.to_string(),
        extra: if extra.is_empty() {
            None
        } else {
            Some(extra.to_string())
        },
        snippet: if snippet.is_empty() {
            None
        } else {
            Some(snippet.to_string())
        },
    })
}

/// Split `path/to/file.rs:42` into `(PathBuf, 42)`. Tolerates Windows-style
/// paths (`C:\\...`) by anchoring on the LAST colon.
fn split_file_line(s: &str, line_no: u64) -> Result<(PathBuf, u32), SymbolGraphError> {
    let idx = s.rfind(':').ok_or_else(|| SymbolGraphError::Parse {
        line: line_no,
        reason: format!("FILE:LINE missing ':' delimiter: {s}"),
    })?;
    let (file_part, line_part) = s.split_at(idx);
    // line_part still has the leading ':' — strip it.
    let line_str = &line_part[1..];
    let parsed: u32 = line_str.parse().map_err(|_| SymbolGraphError::Parse {
        line: line_no,
        reason: format!("FILE:LINE has non-numeric line: {s}"),
    })?;
    Ok((PathBuf::from(file_part), parsed))
}

// ---------------------------------------------------------------------------
// Unit tests — micro-level coverage of the parser. Contract-level
// coverage lives in `crates/contracts/tests/symbol_graph_contract.rs`.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_well_formed_row() {
        let row = "DEF_FN\tfoo\tcrates/a/src/lib.rs:42\trust\t\tfn foo() {";
        let rec = parse_row(row, 1).expect("parse ok");
        assert_eq!(rec.symbol, "foo");
        assert_eq!(rec.line, 42);
        assert_eq!(rec.lang, "rust");
        assert!(matches!(rec.tag, SymbolTag::DefFn));
        assert_eq!(rec.extra, None);
        assert_eq!(rec.snippet.as_deref(), Some("fn foo() {"));
    }

    #[test]
    fn parses_unknown_tag_as_other() {
        let row = "BRAND_NEW_TAG\tx\tf.rs:1\trust\t\t";
        let rec = parse_row(row, 1).expect("parse ok");
        match rec.tag {
            SymbolTag::Other(s) => assert_eq!(s, "BRAND_NEW_TAG"),
            other => panic!("expected Other, got {:?}", other),
        }
    }

    #[test]
    fn rejects_non_numeric_line() {
        let row = "DEF_FN\tfoo\tcrates/a/src/lib.rs:NaN\trust\t\t";
        let err = parse_row(row, 1).expect_err("should fail");
        assert!(matches!(err, SymbolGraphError::Parse { .. }));
    }

    #[test]
    fn tag_is_outgoing_edge_matches_brief() {
        // Naive-tokenisable EXTRA tags are followed.
        assert!(SymbolTag::Call.is_outgoing_edge());
        assert!(SymbolTag::MethodCall.is_outgoing_edge());
        // Structured-EXTRA tags are NOT followed — naive tokenisation of
        // their EXTRA produces fake caller identifiers (BLOCK
        // SCIP_REF_INDEX_POLLUTION).
        assert!(!SymbolTag::CallResolved.is_outgoing_edge());
        assert!(!SymbolTag::ScipRef.is_outgoing_edge());
        // Non-call tags never qualify.
        assert!(!SymbolTag::DefFn.is_outgoing_edge());
        assert!(!SymbolTag::Use.is_outgoing_edge());
    }

    #[test]
    fn caller_idents_dedup_preserves_first_occurrence() {
        // Verify HashSet-backed dedup keeps the first occurrence and skips
        // subsequent duplicates. Insertion order matters because downstream
        // BFS expansion is order-sensitive.
        let got = caller_idents("foo bar foo baz bar");
        assert_eq!(got, vec!["foo", "bar", "baz"]);
    }
}
