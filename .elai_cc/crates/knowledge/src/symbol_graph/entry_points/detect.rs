// B-34 — Entry-point detection algorithm.
//
// Pure function — no I/O after payloads are supplied. Mirrors the algorithm
// in SocratiCode graph-entrypoints.ts:26-139 adapted for ELAI Rust types.
//
// Heuristics applied in order:
//   1. Orphan files: file has no callers in the reverse_file_index
//      AND has at least one outgoing call (graph leaf that calls out = entry).
//   2. Conventional names: symbol name matches the language's conventional set
//      (from names.rs). Also includes Python `__main__` as an additive check.
//   3. Framework routes: outgoing callee_name matches HTTP verb or "route".
//      Enclosing symbol is attributed via find_symbol_at.
//   4. Test functions: name matches /^test[_A-Z]/ or /^Test[A-Z_]/ or is
//      exactly "it" or "describe". Implemented char-by-char (no regex crate).
//
// Deduplication: BTreeSet<(id, reason_key)> — same symbol can appear for
// multiple heuristics, but never twice for the same heuristic.

use std::collections::{BTreeMap, BTreeSet};

use super::names::entry_point_names_per_language;
use super::types::{DetectedEntryPoint, EntryPointReason};
use crate::symbol_graph::types::{SymbolGraphFilePayload, SymbolNode};

// ---------------------------------------------------------------------------
// Framework HTTP verb set (Heuristic 3)
// ---------------------------------------------------------------------------

/// HTTP router method names (lower-cased) that signal a framework route handler.
const HTTP_VERBS: &[&str] = &[
    "get", "post", "put", "delete", "patch", "head", "options", "all", "use",
];

/// Returns the framework reason string for a callee name, or None.
fn framework_reason(callee_name: &str) -> Option<&'static str> {
    let lower = callee_name.to_ascii_lowercase();
    // Compare lower-cased name against the static verb list.
    for verb in HTTP_VERBS {
        if lower == *verb {
            // Safety: all HTTP_VERBS are &'static str embedded in the binary.
            // We cannot return the dynamic `lower` (String), so we return the
            // static slice element directly.
            return Some(match *verb {
                "get"     => "framework:http-get",
                "post"    => "framework:http-post",
                "put"     => "framework:http-put",
                "delete"  => "framework:http-delete",
                "patch"   => "framework:http-patch",
                "head"    => "framework:http-head",
                "options" => "framework:http-options",
                "all"     => "framework:http-all",
                "use"     => "framework:http-use",
                _         => unreachable!("matched HTTP_VERBS member"),
            });
        }
    }
    if lower == "route" {
        return Some("framework:route");
    }
    None
}

// ---------------------------------------------------------------------------
// Test-function name matching (Heuristic 4) — no regex crate
// ---------------------------------------------------------------------------

/// Returns true if `name` matches any test-function naming convention:
///   - starts with "test" followed by '_' or an uppercase letter
///   - starts with "Test" followed by an uppercase letter or '_'
///   - is exactly "it"
///   - is exactly "describe"
fn is_test_name(name: &str) -> bool {
    if name == "it" || name == "describe" {
        return true;
    }
    let chars: Vec<char> = name.chars().collect();
    // Pattern: ^test[_A-Z]
    if chars.len() >= 5
        && chars[0] == 't' && chars[1] == 'e' && chars[2] == 's' && chars[3] == 't'
        && (chars[4] == '_' || chars[4].is_ascii_uppercase())
    {
        return true;
    }
    // Pattern: ^Test[A-Z_]
    if chars.len() >= 5
        && chars[0] == 'T' && chars[1] == 'e' && chars[2] == 's' && chars[3] == 't'
        && (chars[4].is_ascii_uppercase() || chars[4] == '_')
    {
        return true;
    }
    false
}

// ---------------------------------------------------------------------------
// find_symbol_at
// ---------------------------------------------------------------------------

/// Find the deepest (highest start_line) symbol that contains `line`.
///
/// Port of SocratiCode graph-entrypoints.ts:96-104. Symbols with
/// `name == "<module>"` are excluded (they cover the whole file).
///
/// Returns `None` if no symbol spans `line` (after excluding `<module>`).
pub fn find_symbol_at(symbols: &[SymbolNode], line: u32) -> Option<&SymbolNode> {
    let mut best: Option<&SymbolNode> = None;
    for sym in symbols {
        if sym.name == "<module>" {
            continue;
        }
        if sym.line <= line && line <= sym.end_line {
            match best {
                None => best = Some(sym),
                Some(prev) => {
                    // Deepest = highest start_line (more specific scope).
                    // Strict > so first-match wins on tie (SocratiCode parity).
                    if sym.line > prev.line {
                        best = Some(sym);
                    }
                }
            }
        }
    }
    best
}

// ---------------------------------------------------------------------------
// detect_entry_points (main public API)
// ---------------------------------------------------------------------------

/// Detect entry points across all supplied per-file payloads.
///
/// `reverse_index`: callee_file -> Set<caller_file>, produced by
/// `SymbolGraphCache::ensure_reverse_index()`. Files absent as keys have no
/// callers.
///
/// `payloads`: the per-file payload slice to inspect. For selectivity, callers
/// should pre-filter to orphan-candidate files + filename-pattern matches before
/// loading payloads (see the dispatcher arm in production_agent_rpc.rs).
///
/// Returns one `DetectedEntryPoint` per (id, reason_key) pair — the same
/// symbol may appear multiple times with different reasons.
pub fn detect_entry_points(
    reverse_index: &BTreeMap<String, BTreeSet<String>>,
    payloads: &[SymbolGraphFilePayload],
) -> Vec<DetectedEntryPoint> {
    let mut out: Vec<DetectedEntryPoint> = Vec::new();
    // Dedup key: (symbol_or_file_id, reason_key)
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();

    let push = |out: &mut Vec<DetectedEntryPoint>,
                seen: &mut BTreeSet<(String, String)>,
                entry: DetectedEntryPoint| {
        let key = (entry.id.clone(), entry.reason.key());
        if seen.insert(key) {
            out.push(entry);
        }
    };

    // Lazy name table (computed once).
    let names_table = entry_point_names_per_language();

    for payload in payloads {
        let file = &payload.file;

        // Heuristic 1: Orphan files — no callers AND has outgoing calls.
        // An orphan is a file that no other file imports/calls into.
        let caller_count = reverse_index
            .get(file)
            .map_or(0, |callers| callers.len());
        if caller_count == 0 && !payload.outgoing_calls.is_empty() {
            push(
                &mut out,
                &mut seen,
                DetectedEntryPoint {
                    id: file.clone(),
                    name: file.clone(),
                    file: file.clone(),
                    line: None,
                    reason: EntryPointReason::Orphan,
                },
            );
        }

        // Heuristic 2: Conventional names.
        let conventional: Option<&[&str]> = names_table.get(payload.language.as_str()).copied();
        for sym in &payload.symbols {
            if sym.name == "<module>" {
                continue;
            }
            // __main__ special-case (Python and any language where it appears).
            if sym.name == "__main__" {
                push(
                    &mut out,
                    &mut seen,
                    DetectedEntryPoint {
                        id: sym.id.clone(),
                        name: sym.qualified_name.clone(),
                        file: sym.file.clone(),
                        line: Some(sym.line),
                        reason: EntryPointReason::WellKnownName("__main__".to_string()),
                    },
                );
                continue;
            }
            if let Some(names) = conventional {
                if names.contains(&sym.name.as_str()) {
                    push(
                        &mut out,
                        &mut seen,
                        DetectedEntryPoint {
                            id: sym.id.clone(),
                            name: sym.qualified_name.clone(),
                            file: sym.file.clone(),
                            line: Some(sym.line),
                            reason: EntryPointReason::WellKnownName(sym.name.clone()),
                        },
                    );
                }
            }
        }

        // Heuristic 3: Framework routes — inspect outgoing calls.
        for edge in &payload.outgoing_calls {
            let Some(reason_str) = framework_reason(&edge.callee_name) else {
                continue;
            };
            // Find the enclosing symbol at the call site line.
            let Some(enclosing) = find_symbol_at(&payload.symbols, edge.call_site.line) else {
                continue;
            };
            push(
                &mut out,
                &mut seen,
                DetectedEntryPoint {
                    id: enclosing.id.clone(),
                    name: enclosing.qualified_name.clone(),
                    file: enclosing.file.clone(),
                    line: Some(enclosing.line),
                    reason: EntryPointReason::FrameworkRoute(reason_str.to_string()),
                },
            );
        }

        // Heuristic 4: Test functions.
        for sym in &payload.symbols {
            if sym.name == "<module>" {
                continue;
            }
            if is_test_name(&sym.name) {
                push(
                    &mut out,
                    &mut seen,
                    DetectedEntryPoint {
                        id: sym.id.clone(),
                        name: sym.qualified_name.clone(),
                        file: sym.file.clone(),
                        line: Some(sym.line),
                        reason: EntryPointReason::Test,
                    },
                );
            }
        }
    }

    out
}

// ---------------------------------------------------------------------------
// Unit tests for helpers (no I/O)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{find_symbol_at, framework_reason, is_test_name};
    use crate::symbol_graph::types::{SymbolKind, SymbolNode};

    fn make_sym(name: &str, line: u32, end_line: u32) -> SymbolNode {
        SymbolNode {
            id: format!("test::{name}"),
            name: name.to_string(),
            qualified_name: name.to_string(),
            kind: SymbolKind::Function,
            file: "test.rs".to_string(),
            line,
            end_line,
            language: "rust".to_string(),
        }
    }

    #[test]
    fn test_is_test_name_patterns() {
        assert!(is_test_name("test_foo"));
        assert!(is_test_name("testBar"));  // test + uppercase B
        assert!(is_test_name("TestBar"));
        assert!(is_test_name("Test_foo"));
        assert!(is_test_name("it"));
        assert!(is_test_name("describe"));
        assert!(!is_test_name("main"));
        assert!(!is_test_name("testx")); // test + lowercase x = false
        assert!(!is_test_name("Test"));  // no 5th char
        assert!(!is_test_name("regular_fn"));
    }

    #[test]
    fn test_framework_reason_verbs() {
        assert_eq!(framework_reason("get"), Some("framework:http-get"));
        assert_eq!(framework_reason("GET"), Some("framework:http-get"));
        assert_eq!(framework_reason("Post"), Some("framework:http-post"));
        assert_eq!(framework_reason("route"), Some("framework:route"));
        assert_eq!(framework_reason("Route"), Some("framework:route"));
        assert_eq!(framework_reason("emit"), None);
    }

    #[test]
    fn test_find_symbol_at_deepest() {
        let outer = make_sym("outer", 1, 20);
        let inner = make_sym("inner", 5, 15);
        let symbols = vec![outer.clone(), inner.clone()];
        // Line 7 is within both; inner has higher start_line -> deepest.
        let result = find_symbol_at(&symbols, 7);
        assert_eq!(result.map(|s| &s.name), Some(&inner.name));
    }

    #[test]
    fn test_find_symbol_at_excludes_module() {
        let module = make_sym("<module>", 1, 100);
        let func = make_sym("main", 5, 20);
        let symbols = vec![module, func.clone()];
        let result = find_symbol_at(&symbols, 10);
        assert_eq!(result.map(|s| &s.name), Some(&func.name));
    }

    #[test]
    fn test_find_symbol_at_none_when_no_match() {
        let sym = make_sym("helper", 50, 60);
        let symbols = vec![sym];
        assert!(find_symbol_at(&symbols, 10).is_none());
    }
}
