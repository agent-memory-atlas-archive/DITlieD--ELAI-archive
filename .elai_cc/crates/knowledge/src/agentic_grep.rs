//! J-18 leg-3: narrow post-retrieval agentic-grep CONFIRM pass.
//!
//! When a static-index retrieval hit has low confidence, the agent can issue
//! a narrow `rg`/`grep` confirm over the real project tree to verify or
//! extend THAT hit (index-first, grep-to-repair, NOT index-OR-grep). The
//! broad axis (mid-task retrieval LOOP, SpIDER neighbor fusion,
//! sufficiency gating) is J-38, not this file.
//!
//! Additive, behind [`ELAI_AGENTIC_GREP_REPAIR_ENABLE_ENV`] (default OFF).
//! Degrades gracefully: `rg` missing -> `grep -rn` fallback -> `Ok(vec![])`
//! on no matches or tool failure. Never panics, never returns rank-k filler.

use std::path::Path;
use std::process::Command;

use crate::arag::RetrievalResult;
use crate::KnowledgeError;

/// Env-gate for the agentic-grep repair pass. Default-OFF so the
/// production pipeline returns the unrepaired index result until the
/// operator opts in. Mirrors the `ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE_ENV`
/// pattern in `pipeline.rs`.
pub const ELAI_AGENTIC_GREP_REPAIR_ENABLE_ENV: &str = "ELAI_AGENTIC_GREP_REPAIR_ENABLE";

/// Read [`ELAI_AGENTIC_GREP_REPAIR_ENABLE_ENV`] and return `true` when the
/// gate is on. Default (unset / invalid) = `false`.
// WIRE-DARK[J-18-leg3]: production caller lands in slice B (pipeline.rs),
// which gates the confirm pass behind this env flag. Unwired in slice A
// by design (this micro-slice ships the producer module only).
pub fn agentic_grep_repair_enabled_from_env() -> bool {
    match std::env::var(ELAI_AGENTIC_GREP_REPAIR_ENABLE_ENV) {
        Ok(v) => {
            let t = v.trim();
            t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("on")
        }
        Err(_) => false,
    }
}

/// Head-limit on the confirm pass: a NARROW confirm of ONE low-confidence
/// hit, not a broad re-search. Keeps the grep output bounded and the
/// repair additive (J-18 scope boundary vs J-38's loop).
const CONFIRM_MATCH_HEAD_LIMIT: usize = 8;

/// Build a safe grep pattern from the query and the hit snippet. Strips
/// shell metacharacters and whitespace, keeps identifier chars + dots +
/// slashes + hyphens + colons (path/symbol separators). Returns `None`
/// when nothing usable remains (caller returns `Ok(vec![])`).
fn safe_pattern(query: &str, snippet: &str) -> Option<String> {
    let candidates = [query.trim(), snippet.trim()];
    for cand in candidates {
        let filtered: String = cand
            .chars()
            .filter(|c| {
                c.is_alphanumeric()
                    || matches!(c, '.' | '/' | '-' | ':' | '_' | '+' | '#')
            })
            .collect();
        let trimmed = filtered.trim_matches(|c: char| !c.is_alphanumeric());
        if trimmed.len() >= 2 {
            return Some(trimmed.to_owned());
        }
    }
    None
}

/// Glob set for the confirm pass: code-ish files only, skip vendored,
/// generated, and binary noise. Mirrors the `--glob` discipline used by
/// the repo's other `rg` invocations.
const CONFIRM_GLOBS: &[&str] = &[
    "--glob=*.rs",
    "--glob=*.py",
    "--glob=*.ts",
    "--glob=*.tsx",
    "--glob=*.js",
    "--glob=*.jsx",
    "--glob=*.svelte",
    "--glob=*.go",
    "--glob=*.java",
    "--glob=*.c",
    "--glob=*.h",
    "--glob=*.cpp",
    "--glob=*.hpp",
    "--glob=*.toml",
    "--glob=*.md",
    "--glob=!target/**",
    "--glob=!node_modules/**",
    "--glob=!**/.git/**",
];

/// Run `rg` (or `grep -rn` fallback) over `project_root` with a narrow
/// pattern derived from `query` + `hit`, and return a list of
/// [`RetrievalResult`] confirming or extending `hit`. Returns
/// `Ok(vec![])` on no matches, missing tools, or tool failure (degrade,
/// never panic, never return rank-k filler).
///
/// `score` on returned results is `hit.score` plus a modest bump
/// (`0.01`), capped at `1.0`, because a live grep match is stronger
/// evidence than the static index alone. `chunk_id` is stable:
/// `agentic_grep_repair::<blake3-8-of-pattern>::<line>`.
// WIRE-DARK[J-18-leg3]: production caller lands in slice B (pipeline.rs),
// which calls this after a low-confidence hit. Unwired in slice A by
// design (this micro-slice ships the producer module only).
pub fn agentic_grep_confirm(
    query: &str,
    hit: &RetrievalResult,
    project_root: &Path,
) -> Result<Vec<RetrievalResult>, KnowledgeError> {
    let pattern = match safe_pattern(query, &hit.snippet) {
        Some(p) => p,
        None => return Ok(vec![]),
    };

    let bumped_score = (hit.score + 0.01).min(1.0);

    let matches = match rg_confirm(&pattern, project_root) {
        Some(lines) => lines,
        None => match grep_fallback_confirm(&pattern, project_root) {
            Some(lines) => lines,
            None => return Ok(vec![]),
        },
    };

    if matches.is_empty() {
        return Ok(vec![]);
    }

    let mut out: Vec<RetrievalResult> = Vec::with_capacity(CONFIRM_MATCH_HEAD_LIMIT);
    for (path_str, line_no, snippet) in matches.into_iter().take(CONFIRM_MATCH_HEAD_LIMIT) {
        let chunk_id = format!(
            "agentic_grep_repair::{path_str}::{line_no}::{}",
            blake3_prefix8(&pattern)
        );
        out.push(RetrievalResult {
            chunk_id,
            snippet,
            score: bumped_score,
            tool_used: "agentic_grep_repair".to_owned(),
        });
    }
    Ok(out)
}

/// blake3 8-hex-char prefix for stable chunk_id entropy. Mirrors the
/// arag exact-cache hashing idiom without pulling in the full 32-byte
/// digest (a prefix is enough for chunk-id stability).
fn blake3_prefix8(input: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(input.as_bytes());
    let digest = hasher.finalize();
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&digest.as_bytes()[..4]);
    format!("{:08x}", u32::from_be_bytes(buf))
}

/// Try `rg` over `project_root`. Returns `Some(lines)` on success (possibly
/// empty), `None` when `rg` is missing or the spawn fails (caller falls
/// back to grep). Lines: `(relative_path, line_no, snippet)`.
fn rg_confirm(pattern: &str, project_root: &Path) -> Option<Vec<(String, String, String)>> {
    let mut cmd = Command::new("rg");
    cmd.args(["-n", "--color", "never", "--no-heading", "-S"]);
    for g in CONFIRM_GLOBS {
        cmd.arg(g);
    }
    cmd.arg("--max-count").arg(CONFIRM_MATCH_HEAD_LIMIT.to_string());
    cmd.arg(pattern).arg(".");
    cmd.current_dir(project_root);
    let output = cmd.output().ok()?;
    if !output.status.success() {
        return Some(vec![]);
    }
    Some(parse_grep_lines(&String::from_utf8_lossy(&output.stdout)))
}

/// `grep -rn` fallback when `rg` is missing. Same glob discipline is
/// not available (grep has no `--glob`), so we accept the broader search
/// and rely on the head-limit + code-ish extension filter in
/// [`parse_grep_lines`].
fn grep_fallback_confirm(pattern: &str, project_root: &Path) -> Option<Vec<(String, String, String)>> {
    let mut cmd = Command::new("grep");
    cmd.args(["-rn", "--color=never", "-I"]);
    cmd.arg(pattern).arg(".");
    cmd.current_dir(project_root);
    let output = cmd.output().ok()?;
    if !output.status.success() {
        return Some(vec![]);
    }
    Some(parse_grep_lines(&String::from_utf8_lossy(&output.stdout)))
}

/// Parse `path:line:content` lines from grep/rg output. Skips lines
/// that don't match the 3-field shape and skips paths whose extension
/// is not code-ish (the grep fallback has no `--glob`).
fn parse_grep_lines(raw: &str) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for line in raw.lines() {
        let mut parts = line.splitn(3, ':');
        let (Some(path), Some(line_no), Some(content)) =
            (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        if line_no.is_empty() || !line_no.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        if !is_codeish_path(path) {
            continue;
        }
        out.push((path.to_owned(), line_no.to_owned(), content.to_owned()));
    }
    out
}

/// Conservative code-ish extension filter for the grep fallback leg.
fn is_codeish_path(path: &str) -> bool {
    const EXTS: &[&str] = &[
        "rs", "py", "ts", "tsx", "js", "jsx", "svelte", "go", "java", "c", "h", "cpp", "hpp",
        "toml", "md",
    ];
    let lower = path.to_ascii_lowercase();
    EXTS.iter().any(|ext| lower.ends_with(&format!(".{ext}")))
        && !lower.contains("/target/")
        && !lower.contains("/node_modules/")
        && !lower.contains("/.git/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arag::RetrievalResult;
    use std::fs;
    use tempfile::tempdir;

    fn make_hit(score: f64, snippet: &str) -> RetrievalResult {
        RetrievalResult {
            chunk_id: "idx::x".to_owned(),
            snippet: snippet.to_owned(),
            score,
            tool_used: "bm25".to_owned(),
        }
    }

    #[test]
    fn flag_env_parsing() {
        // Unset -> false.
        std::env::remove_var(ELAI_AGENTIC_GREP_REPAIR_ENABLE_ENV);
        assert!(!agentic_grep_repair_enabled_from_env());

        for val in ["1", "true", "TRUE", "on", "On"] {
            std::env::set_var(ELAI_AGENTIC_GREP_REPAIR_ENABLE_ENV, val);
            assert!(
                agentic_grep_repair_enabled_from_env(),
                "flag should be ON for {val:?}"
            );
        }
        for val in ["0", "false", "off", "", "yes", "2"] {
            std::env::set_var(ELAI_AGENTIC_GREP_REPAIR_ENABLE_ENV, val);
            assert!(
                !agentic_grep_repair_enabled_from_env(),
                "flag should be OFF for {val:?}"
            );
        }
        std::env::remove_var(ELAI_AGENTIC_GREP_REPAIR_ENABLE_ENV);
    }

    #[test]
    fn confirm_returns_empty_when_no_matches() {
        let dir = tempdir().expect("tempdir");
        let query = "zzz_nonexistent_symbol_xyz";
        let hit = make_hit(0.3, "zzz_nonexistent_symbol_xyz");
        let out = agentic_grep_confirm(query, &hit, dir.path()).expect("ok");
        assert!(out.is_empty(), "no matches -> empty vec, got {out:?}");
    }

    #[test]
    fn confirm_finds_planted_file_match() {
        let dir = tempdir().expect("tempdir");
        let code = "fn agentic_test_fixture_marker() -> u32 { 42 }\n";
        fs::write(dir.path().join("fixture.rs"), code).expect("write fixture");

        let query = "agentic_test_fixture_marker";
        let hit = make_hit(0.3, "agentic_test_fixture_marker");
        let out = agentic_grep_confirm(query, &hit, dir.path()).expect("ok");

        assert!(!out.is_empty(), "should find the planted match");
        let first = &out[0];
        assert_eq!(first.tool_used, "agentic_grep_repair");
        assert!(
            first.chunk_id.starts_with("agentic_grep_repair::"),
            "chunk_id should be namespaced, got {}",
            first.chunk_id
        );
        assert!(
            first.snippet.contains("agentic_test_fixture_marker"),
            "snippet should contain the match, got {}",
            first.snippet
        );
        assert!(
            first.score > hit.score,
            "score should bump above hit.score ({} > {})",
            first.score,
            hit.score
        );
        assert!(first.score <= 1.0, "score should cap at 1.0");
    }

    #[test]
    fn safe_pattern_strips_shell_metacharacters() {
        // Shell metacharacters (;, $, (), spaces) are dropped; identifier
        // chars + path/symbol separators (-, /) are kept.
        let p = safe_pattern("foo; rm -rf /", "");
        assert_eq!(p.as_deref(), Some("foorm-rf"));
        let p2 = safe_pattern("", "bar$(evil)");
        assert_eq!(p2.as_deref(), Some("barevil"));
        let p3 = safe_pattern("  ", "");
        assert_eq!(p3, None);
    }
}
