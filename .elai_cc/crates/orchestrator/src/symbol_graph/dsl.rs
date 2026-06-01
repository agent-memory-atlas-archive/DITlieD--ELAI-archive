//! E-40 Tranche B-2 — Datalog-style DSL parser + executor over
//! `SymbolGraphCache`.
//!
//! Grammar (minimal v1):
//!
//! ```text
//! query    := <selector> ('|' <filter>)*
//! selector := DEF '(' <symbol> ')'
//!           | USE '(' <symbol> ')'
//!           | IMPL '(' <trait>  ')'
//!           | CALL '(' <fn>     ')'
//! filter   := 'lang' '(' <lang>    ')'
//!           | 'file' '(' <pattern> ')'
//!           | 'depth' '(' <n>       ')'
//! ```
//!
//! The executor compiles every selector down to one of the existing four
//! shard-backed query helpers in [`super::query`]:
//!
//! - `DEF(name)`  -> [`super::query::find_defs`]
//! - `USE(name)`  -> [`super::query::find_refs`]
//! - `CALL(name)` -> [`super::query::find_callees`]
//! - `IMPL(name)` -> [`super::query::find_defs`] (then `kind`-filtered)
//!
//! Filters re-trim the executor's intermediate result list. `depth` is
//! treated as advisory v1 (used by the dataflow side-channel, not by
//! the selector itself).
//!
//! Anti-stale-training rule compliance:
//! - `nom` 7.x parser (already in `Cargo.lock` as transitive). NO
//!   hand-rolled regex/state machine.
//! - `thiserror::Error` for the parse-error type, never
//!   `Box<dyn Error>`.
//! - Named constants for every magic threshold.
//!
//! See `workflow/reasoning/E-40-tranche-B2-decisions.md`.

use nom::branch::alt;
use nom::bytes::complete::{tag_no_case, take_while1};
use nom::character::complete::{char as nom_char, multispace0};
use nom::combinator::{map, map_res};
use nom::multi::many0;
use nom::sequence::{delimited, preceded};
use nom::{Finish, IResult};
use thiserror::Error;

use super::cache::SymbolGraphCache;
use super::query::{find_callees, find_defs, find_refs, SymbolMatch};

/// Errors surfaced by the DSL parser.
#[derive(Debug, Error)]
pub enum DslParseError {
    /// `nom` returned an error at parse time. Carries the residue tail
    /// so the caller can render a `pos` marker.
    #[error("symbol_graph dsl: parse error at \"{tail}\"")]
    Nom { tail: String },
    /// The DSL was empty after trimming whitespace.
    #[error("symbol_graph dsl: empty query")]
    Empty,
}

/// AST root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DslAst {
    pub selector: DslSelector,
    pub filters: Vec<DslFilter>,
}

/// Selector head.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DslSelector {
    /// `DEF(name)` — definitions of `name`.
    Def(String),
    /// `USE(name)` — callers / references to `name`.
    Use(String),
    /// `IMPL(trait_name)` — implementations of `trait_name`.
    Impl(String),
    /// `CALL(fn_name)` — direct callees of `fn_name`.
    Call(String),
}

/// Filter clause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DslFilter {
    /// `lang(rust|python|...)` — keep only matches whose file extension
    /// implies the named language.
    Lang(String),
    /// `file(substring)` — keep only matches whose file path contains
    /// `substring`.
    File(String),
    /// `depth(n)` — advisory hop limit for the dataflow side-channel
    /// (the selector itself is shard-bounded).
    Depth(u8),
}

// ---------------------------------------------------------------------------
// nom parser
// ---------------------------------------------------------------------------

/// Symbol/identifier body: alphanumerics + `_:.<>*` (covers SCIP-style
/// qualified ids and glob-ish file patterns).
fn ident_body(s: &str) -> IResult<&str, &str> {
    take_while1(|c: char| {
        c.is_alphanumeric()
            || c == '_'
            || c == ':'
            || c == '.'
            || c == '<'
            || c == '>'
            || c == '*'
            || c == '/'
            || c == '-'
            || c == '\\'
    })(s)
}

fn parens<'a>(s: &'a str) -> IResult<&'a str, &'a str> {
    delimited(
        preceded(multispace0, nom_char('(')),
        delimited(multispace0, ident_body, multispace0),
        nom_char(')'),
    )(s)
}

fn selector(s: &str) -> IResult<&str, DslSelector> {
    let (s, _) = multispace0(s)?;
    alt((
        map(preceded(tag_no_case("DEF"), parens), |id: &str| {
            DslSelector::Def(id.to_string())
        }),
        map(preceded(tag_no_case("USE"), parens), |id: &str| {
            DslSelector::Use(id.to_string())
        }),
        map(preceded(tag_no_case("IMPL"), parens), |id: &str| {
            DslSelector::Impl(id.to_string())
        }),
        map(preceded(tag_no_case("CALL"), parens), |id: &str| {
            DslSelector::Call(id.to_string())
        }),
    ))(s)
}

fn filter_clause(s: &str) -> IResult<&str, DslFilter> {
    let (s, _) = multispace0(s)?;
    alt((
        map(preceded(tag_no_case("lang"), parens), |id: &str| {
            DslFilter::Lang(id.to_string())
        }),
        map(preceded(tag_no_case("file"), parens), |id: &str| {
            DslFilter::File(id.to_string())
        }),
        map_res(preceded(tag_no_case("depth"), parens), |id: &str| {
            id.parse::<u8>().map(DslFilter::Depth)
        }),
    ))(s)
}

fn pipe_then_filter(s: &str) -> IResult<&str, DslFilter> {
    let (s, _) = multispace0(s)?;
    let (s, _) = nom_char('|')(s)?;
    filter_clause(s)
}

fn query(s: &str) -> IResult<&str, DslAst> {
    let (s, sel) = selector(s)?;
    let (s, filters) = many0(pipe_then_filter)(s)?;
    let (s, _) = multispace0(s)?;
    Ok((s, DslAst { selector: sel, filters }))
}

/// Parse a DSL string into a [`DslAst`].
pub fn parse_dsl(input: &str) -> Result<DslAst, DslParseError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(DslParseError::Empty);
    }
    match query(trimmed).finish() {
        Ok((_rest, ast)) => Ok(ast),
        Err(e) => Err(DslParseError::Nom {
            tail: e.input.chars().take(40).collect(),
        }),
    }
}

// ---------------------------------------------------------------------------
// Executor
// ---------------------------------------------------------------------------

/// Best-effort language-from-extension classifier. Returns the lowercase
/// language slug (matches the filter literal). Unknown extensions return
/// an empty string.
fn lang_from_file(path: &str) -> &'static str {
    if path.ends_with(".rs") {
        "rust"
    } else if path.ends_with(".py") {
        "python"
    } else if path.ends_with(".ts") || path.ends_with(".tsx") {
        "typescript"
    } else if path.ends_with(".js") || path.ends_with(".jsx") {
        "javascript"
    } else if path.ends_with(".svelte") {
        "svelte"
    } else if path.ends_with(".go") {
        "go"
    } else {
        ""
    }
}

/// Execute a parsed DSL AST against the cache.
///
/// Best-effort: any underlying [`super::query::QueryError`] is treated
/// as an empty result (the DSL surface is exploratory; callers see an
/// empty match list rather than a hard error if a symbol does not
/// resolve). Filters are applied in-order after the selector.
pub fn exec_dsl(ast: &DslAst, cache: &mut SymbolGraphCache) -> Vec<SymbolMatch> {
    let mut results = match &ast.selector {
        DslSelector::Def(name) => find_defs(cache, name).unwrap_or_default(),
        DslSelector::Use(name) => find_refs(cache, name).unwrap_or_default(),
        DslSelector::Call(name) => find_callees(cache, name).unwrap_or_default(),
        DslSelector::Impl(name) => {
            // IMPL is a best-effort overlay on find_defs: keep only nodes
            // whose `kind` looks like an impl block. Cache nodes carry
            // free-form `kind` strings from the SCIP-enriched JSON.
            let defs = find_defs(cache, name).unwrap_or_default();
            defs.into_iter()
                .filter(|m| {
                    let k = m.kind.to_ascii_lowercase();
                    k.contains("impl") || k.contains("trait_impl")
                })
                .collect()
        }
    };
    for f in &ast.filters {
        match f {
            DslFilter::Lang(lang) => {
                let want = lang.to_ascii_lowercase();
                results.retain(|m| lang_from_file(&m.file).eq_ignore_ascii_case(&want));
            }
            DslFilter::File(pattern) => {
                results.retain(|m| m.file.contains(pattern.as_str()));
            }
            DslFilter::Depth(_) => {
                // Advisory only — the selector list is already
                // shard-bounded. Dataflow side-channel consumes Depth.
            }
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol_graph::cache::SymbolNode;

    fn build_cache() -> SymbolGraphCache {
        let mut cache = SymbolGraphCache::empty_with_capacity(16).expect("empty");
        cache.index_node(SymbolNode {
            id: "src/a.rs:foo:10".into(),
            name: "foo".into(),
            file: "src/a.rs".into(),
            line: 10,
            kind: "function".into(),
            calls: vec!["src/b.rs:bar:20".into()],
        });
        cache.index_node(SymbolNode {
            id: "src/b.rs:bar:20".into(),
            name: "bar".into(),
            file: "src/b.rs".into(),
            line: 20,
            kind: "function".into(),
            calls: vec![],
        });
        cache.index_node(SymbolNode {
            id: "py/c.py:baz:30".into(),
            name: "baz".into(),
            file: "py/c.py".into(),
            line: 30,
            kind: "function".into(),
            calls: vec!["src/b.rs:bar:20".into()],
        });
        cache
    }

    #[test]
    fn parse_def_only() {
        let ast = parse_dsl("DEF(foo)").expect("parse");
        assert_eq!(ast.selector, DslSelector::Def("foo".into()));
        assert!(ast.filters.is_empty());
    }

    #[test]
    fn parse_use_with_filters() {
        let ast = parse_dsl("USE(bar) | lang(rust) | file(src/) | depth(3)").expect("parse");
        assert_eq!(ast.selector, DslSelector::Use("bar".into()));
        assert_eq!(ast.filters.len(), 3);
        assert!(matches!(ast.filters[2], DslFilter::Depth(3)));
    }

    #[test]
    fn parse_rejects_empty() {
        assert!(matches!(parse_dsl(""), Err(DslParseError::Empty)));
        assert!(matches!(parse_dsl("   "), Err(DslParseError::Empty)));
    }

    #[test]
    fn parse_rejects_garbage() {
        assert!(parse_dsl("NOT_A_SELECTOR(x)").is_err());
    }

    #[test]
    fn exec_def_returns_defs() {
        let mut cache = build_cache();
        let ast = parse_dsl("DEF(bar)").expect("parse");
        let out = exec_dsl(&ast, &mut cache);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].name, "bar");
    }

    #[test]
    fn exec_with_lang_filter_keeps_rust() {
        let mut cache = build_cache();
        let ast = parse_dsl("USE(bar) | lang(rust)").expect("parse");
        let out = exec_dsl(&ast, &mut cache);
        // foo (rust) calls bar; baz (python) also calls bar. Filter keeps foo.
        for m in &out {
            assert_eq!(super::lang_from_file(&m.file), "rust");
        }
    }
}
