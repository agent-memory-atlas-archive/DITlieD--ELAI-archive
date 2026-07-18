//! E-52 T2 -- per-language diagnostic-probe parser module (zero-new-dep).
//!
//! Houses the [`DiagnosticParser`] trait + a language-keyed dispatcher
//! ([`parser_for_language`]) + concrete parser impls. U0 ships the trait +
//! [`GenericLspFallback`]; U1 ships [`RustAnalyzerParser`]. Pyright/Tsc/
//! SvelteCheck (U2-U4) are RECON-gated and land in later slices.
//!
//! Parsing is a PURE FUNCTION of (stdout, stderr, req): the subprocess spawn
//! lives in [`crate::diagnostic_probe::dispatch_diagnostic_probe`] so the
//! parser stays unit-testable without spawning a tool. This separation is
//! load-bearing for the contract test at
//! `crates/contracts/tests/e52_rust_analyzer_parser_contract.rs`.
//!
//! No `regex` crate (U1 mandate: hand-rolled std::str parsing via
//! `std::str::Lines` + `starts_with` + `splitn`). If stdlib parsing proves
//! too brittle for a future parser, that is a material fork: STOP + handoff,
//! do NOT add the regex dep without owner approval.

use elai_knowledge::diagnostic::{
    Diagnostic, DiagnosticProbeOutcome, DiagnosticProbeRunStatus, DiagnosticSeverity, ProbeKind,
};

use crate::diagnostic_probe::DiagnosticProbeRequest;

// ---------------------------------------------------------------------------
// Trait + dispatcher (U0).
// ---------------------------------------------------------------------------

/// Per-language parser contract. `parse` is a pure function of the captured
/// stdout/stderr + the original request; it does NOT spawn the tool. The
/// subprocess invocation lives in
/// [`crate::diagnostic_probe::dispatch_diagnostic_probe`].
pub(crate) trait DiagnosticParser: Send + Sync {
    /// Stable probe-kind tag for SSB beacon hashing + outcome routing.
    fn kind(&self) -> ProbeKind;

    /// Parse captured stdout/stderr into a [`DiagnosticProbeOutcome`]. Pure:
    /// no IO, no subprocess. The caller chooses `run_status` (Completed vs
    /// TimedOut) BEFORE calling parse when the subprocess already exited; a
    /// successful subprocess exit maps to `Completed` here.
    fn parse(
        &self,
        stdout: &str,
        stderr: &str,
        req: &DiagnosticProbeRequest,
    ) -> DiagnosticProbeOutcome;
}

/// Pick the parser for a language slug. Unknown slugs fall through to
/// [`GenericLspFallback`] so the wire stays stable. The dispatcher is
/// exhaustive (the `_` arm is the fallback, not a panic).
pub(crate) fn parser_for_language(language: &str) -> Box<dyn DiagnosticParser> {
    match language.to_ascii_lowercase().as_str() {
        "rust" | "rs" => Box::new(RustAnalyzerParser),
        _ => Box::new(GenericLspFallback),
    }
}

// ---------------------------------------------------------------------------
// GenericLsp fallback (U0 skeleton; completed in U5). No subprocess; pure
// data constructor returning a zero-diagnostic Completed outcome.
// ---------------------------------------------------------------------------

pub(crate) struct GenericLspFallback;

impl DiagnosticParser for GenericLspFallback {
    fn kind(&self) -> ProbeKind {
        ProbeKind::GenericLsp
    }

    fn parse(
        &self,
        _stdout: &str,
        _stderr: &str,
        req: &DiagnosticProbeRequest,
    ) -> DiagnosticProbeOutcome {
        tracing::warn!(
            language = %req.language,
            file = %req.file_path.display(),
            "diagnostic_probe_parsers: language unmapped, returning zero-diagnostic Completed (GenericLspFallback)"
        );
        DiagnosticProbeOutcome {
            kind: ProbeKind::GenericLsp,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            hint_count: 0,
            duration_ms: 0,
            diagnostics: Vec::new(),
            run_status: DiagnosticProbeRunStatus::Completed,
        }
    }
}

// ---------------------------------------------------------------------------
// RustAnalyzer parser (U1). Hand-rolled std::str parsing of
// `rust-analyzer diagnostics <crate-root>` stdout.
// ---------------------------------------------------------------------------

// GROUNDED[rust-analyzer]: `rust-analyzer diagnostics --help` on
// rust-analyzer 1.97.1 (8bab26f 2026-07-14) lists the `diagnostics`
// subcommand with args `<path>` (Cargo.toml dir) + flags
// `--disable-build-scripts` / `--disable-proc-macros` / `--proc-macro-srv
// <path>` / `--severity <severity>`. Verified on a 2-error fixture
// (`let x: String = 5;` at /tmp/ra-fixture): stdout lines of the form
//   `at crate <CRATE>, file <ABS_PATH>: <Severity> <Code> from LineCol
//    { line: <N>, col: <N> } to LineCol { line: <N>, col: <N> }: <message>`
// interleaved with progress lines `<done>/<total> <pct>% processing <path>`
// and a final `diagnostic scan complete` line. Exit code is 0 even with
// Error lines present; the parser MUST NOT rely on exit code.
pub(crate) struct RustAnalyzerParser;

impl RustAnalyzerParser {
    /// Parse one `at crate ...` line into a [`Diagnostic`]. Returns None on
    /// any shape mismatch (the caller skips silently -- a partial parse is
    /// better than a panic on a version-drift line).
    fn parse_line(line: &str, req: &DiagnosticProbeRequest) -> Option<Diagnostic> {
        // Expected shape:
        //   at crate <CRATE>, file <ABS_PATH>: <Severity> <Code> from
        //   LineCol { line: <L>, col: <C> } to LineCol { line: <L>, col:
        //   <C> }: <message>
        // We anchor on the literal `at crate ` prefix, then split on the
        // literal `: ` separators to peel off the file path, then the
        // severity+code+LineCol segment, leaving the message as the tail.
        let rest = line.strip_prefix("at crate ")?;
        // Split off the crate name at the first `, file `.
        let (_, file_and_rest) = rest.split_once(", file ")?;
        // file_and_rest is "<ABS_PATH>: <Severity> <Code> from LineCol ... :
        // <message>". The file path ends at the first `: ` (the separator
        // before the severity). Use splitn(2, ": ") so any `: ` inside the
        // message tail is preserved.
        let (file_path, after_file) = file_and_rest.split_once(": ")?;
        // after_file is "<Severity> <Code> from LineCol { line: <L>, col:
        // <C> } to LineCol { line: <L>, col: <C> }: <message>". The message
        // is everything after the LAST `: ` that follows the closing `}` of
        // the second LineCol. We anchor on the literal `}: ` to split the
        // LineCol segment from the message.
        let (sev_code_loc, message) = after_file.rsplit_once("}: ")?;
        // sev_code_loc is "<Severity> <Code> from LineCol { line: <L>, col:
        // <C> } to LineCol { line: <L>, col: <C> }". Extract the first
        // LineCol's line/col via the literal `LineCol { line: ` prefix.
        let first_lc = sev_code_loc.split_once("LineCol { line: ")?;
        let (_, after_line_kw) = first_lc;
        // after_line_kw is "<L>, col: <C> } to LineCol ...". Split on `, col:
        // ` to get the line number.
        let (line_str, after_col_kw) = after_line_kw.split_once(", col: ")?;
        let line: u32 = line_str.trim().parse().ok()?;
        // after_col_kw is "<C> } to LineCol ...". Split on ` }` to get the
        // column.
        let (col_str, _) = after_col_kw.split_once(" }")?;
        let column: u32 = col_str.trim().parse().ok()?;

        // Now peel severity + code from the head of sev_code_loc. The head
        // is "<Severity> <Code> from LineCol ...". Split on ` from LineCol `
        // to separate the severity+code from the location.
        let (sev_code, _) = sev_code_loc.split_once(" from LineCol ")?;
        // sev_code is "<Severity> <Code>". The severity is the first token;
        // the code is the rest (may be a RustcHardError("E0308") form or a
        // bare code like `unused_imports`).
        let (sev_str, code_str) = sev_code.split_once(' ')?;
        let severity = match sev_str {
            "Error" => DiagnosticSeverity::Error,
            "Warning" => DiagnosticSeverity::Warning,
            "Info" => DiagnosticSeverity::Information,
            "Hint" => DiagnosticSeverity::Hint,
            _ => return None,
        };
        let code = parse_rust_analyzer_code(code_str);

        let path = normalize_rust_analyzer_path(file_path, req);
        Some(Diagnostic {
            path,
            line,
            column,
            severity,
            message: message.to_string(),
            code,
        })
    }
}

/// Strip the `RustcHardError("E0308")` wrapper to `Some("E0308")`, or pass a
/// bare code token through as `Some(<code>)`. Returns None when the token is
/// empty or unparseable.
fn parse_rust_analyzer_code(token: &str) -> Option<String> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(rest) = trimmed.strip_prefix("RustcHardError(") {
        // rest is `"E0308")` -- strip the trailing `)` then the surrounding
        // quotes.
        let inner = rest.strip_suffix(')')?;
        let unquoted = inner.trim().trim_matches('"');
        if unquoted.is_empty() {
            return None;
        }
        return Some(unquoted.to_string());
    }
    Some(trimmed.to_string())
}

/// Normalize an absolute rust-analyzer file path to a project-relative path
/// by deriving the project root as the prefix of `req.file_path` up to (not
/// including) the first `src` path segment, then stripping `<root>/` from the
/// emitted `file_path`. Falls back to the raw `file_path` when `req.file_path`
/// has no `src` segment or the derived root does not prefix-match the emitted
/// path. Pure: no filesystem access.
fn normalize_rust_analyzer_path(file_path: &str, req: &DiagnosticProbeRequest) -> String {
    let req_str = req.file_path.display().to_string();
    let components: Vec<&str> = req_str.split('/').collect();
    let src_idx = components.iter().position(|c| *c == "src");
    let Some(idx) = src_idx else {
        return file_path.to_string();
    };
    let root = components[..idx].join("/");
    let prefix = if root.is_empty() {
        "/".to_string()
    } else {
        format!("{root}/")
    };
    match file_path.strip_prefix(&prefix) {
        Some(rel) => rel.to_string(),
        None => file_path.to_string(),
    }
}

impl DiagnosticParser for RustAnalyzerParser {
    fn kind(&self) -> ProbeKind {
        ProbeKind::RustAnalyzer
    }

    fn parse(
        &self,
        stdout: &str,
        _stderr: &str,
        req: &DiagnosticProbeRequest,
    ) -> DiagnosticProbeOutcome {
        let kind = ProbeKind::RustAnalyzer;
        let mut diagnostics: Vec<Diagnostic> = Vec::new();
        let mut error_count = 0u32;
        let mut warning_count = 0u32;
        let mut info_count = 0u32;
        let mut hint_count = 0u32;

        for line in stdout.lines() {
            // Skip progress lines: `<done>/<total> <pct>% processing <path>`.
            // Anchor on the literal ` processing ` tail + the leading
            // `<n>/<n> <n>%` shape to avoid matching real diagnostic lines.
            if is_rust_analyzer_progress_line(line) {
                continue;
            }
            // Skip the final `diagnostic scan complete` line.
            if line.trim() == "diagnostic scan complete" {
                continue;
            }
            // Skip blank lines.
            if line.trim().is_empty() {
                continue;
            }
            if let Some(diag) = Self::parse_line(line, req) {
                match diag.severity {
                    DiagnosticSeverity::Error => error_count += 1,
                    DiagnosticSeverity::Warning => warning_count += 1,
                    DiagnosticSeverity::Information => info_count += 1,
                    DiagnosticSeverity::Hint => hint_count += 1,
                }
                diagnostics.push(diag);
            }
        }

        DiagnosticProbeOutcome {
            kind,
            error_count,
            warning_count,
            info_count,
            hint_count,
            duration_ms: 0,
            diagnostics,
            run_status: DiagnosticProbeRunStatus::Completed,
        }
    }
}

/// True when `line` matches the rust-analyzer progress shape
/// `<done>/<total> <pct>% processing <path>`. Hand-rolled to avoid the regex
/// crate (U1 mandate).
fn is_rust_analyzer_progress_line(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.contains(" processing ") {
        return false;
    }
    // The head before ` processing ` must be `<n>/<n> <n>%`.
    let head = match trimmed.split_once(" processing ") {
        Some((h, _)) => h,
        None => return false,
    };
    // head is `<done>/<total> <pct>%`. Split on the first space.
    let (frac, pct) = match head.split_once(' ') {
        Some((a, b)) => (a, b),
        None => return false,
    };
    // frac is `<done>/<total>`: both halves must be ascii digits.
    let (done, total) = match frac.split_once('/') {
        Some((a, b)) => (a, b),
        None => return false,
    };
    if done.is_empty() || total.is_empty() {
        return false;
    }
    if !done.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    if !total.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    // pct is `<n>%`: must end with `%` and the rest must be ascii digits.
    let pct_num = match pct.strip_suffix('%') {
        Some(n) => n,
        None => return false,
    };
    !pct_num.is_empty() && pct_num.chars().all(|c| c.is_ascii_digit())
}

// ---------------------------------------------------------------------------
// test_support -- pub wrappers for contract tests in the elai_cc_contracts
// crate. Gated on `#[cfg(any(test, feature = "test-support"))]` so the
// parser surface stays `pub(crate)` in production builds (matches the
// ipc_accept::test_support precedent at ipc_accept.rs:896, which defines
// pub items inside the gated module rather than re-exporting pub(crate)
// ones -- Rust forbids `pub use` of a `pub(crate)` item from a `pub mod`).
// ---------------------------------------------------------------------------

#[cfg(any(test, feature = "test-support"))]
pub mod test_support {
    use super::{DiagnosticParser, DiagnosticProbeRequest, GenericLspFallback, RustAnalyzerParser};
    use elai_knowledge::diagnostic::DiagnosticProbeOutcome;
    use std::path::PathBuf;

    /// Parse rust-analyzer `diagnostics` stdout into a
    /// [`DiagnosticProbeOutcome`]. Pure: no subprocess. Contract tests call
    /// this to exercise the parser without spawning rust-analyzer.
    #[must_use]
    pub fn parse_rust_analyzer_stdout(
        stdout: &str,
        stderr: &str,
        req: &DiagnosticProbeRequest,
    ) -> DiagnosticProbeOutcome {
        RustAnalyzerParser.parse(stdout, stderr, req)
    }

    /// Build a [`DiagnosticProbeRequest`] for the standard E-52 T2 fixture
    /// (`/tmp/ra-fixture/src/lib.rs`, language `rust`).
    #[must_use]
    pub fn fixture_req(file_path: &str) -> DiagnosticProbeRequest {
        DiagnosticProbeRequest::new(PathBuf::from(file_path), "rust")
    }

    /// Invoke the GenericLspFallback parser (contract tests for U5).
    #[must_use]
    pub fn parse_generic_lsp_fallback(
        stdout: &str,
        stderr: &str,
        req: &DiagnosticProbeRequest,
    ) -> DiagnosticProbeOutcome {
        GenericLspFallback.parse(stdout, stderr, req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn req() -> DiagnosticProbeRequest {
        DiagnosticProbeRequest::new(PathBuf::from("/tmp/ra-fixture/src/lib.rs"), "rust")
    }

    #[test]
    fn generic_lsp_fallback_returns_zero_diag_completed() {
        let parser = GenericLspFallback;
        let out = parser.parse("anything", "", &req());
        assert_eq!(out.run_status, DiagnosticProbeRunStatus::Completed);
        assert!(out.is_clean());
        assert_eq!(out.diagnostics.len(), 0);
        assert_eq!(out.kind, ProbeKind::GenericLsp);
    }

    #[test]
    fn parser_for_language_rust_returns_rust_analyzer() {
        let p = parser_for_language("rust");
        assert_eq!(p.kind(), ProbeKind::RustAnalyzer);
    }

    #[test]
    fn parser_for_language_unknown_returns_fallback() {
        let p = parser_for_language("brainfuck");
        assert_eq!(p.kind(), ProbeKind::GenericLsp);
    }

    #[test]
    fn rust_analyzer_empty_stdout_returns_clean() {
        let parser = RustAnalyzerParser;
        let out = parser.parse("", "", &req());
        assert_eq!(out.run_status, DiagnosticProbeRunStatus::Completed);
        assert!(out.is_clean());
        assert_eq!(out.diagnostics.len(), 0);
        assert_eq!(out.kind, ProbeKind::RustAnalyzer);
    }

    #[test]
    fn rust_analyzer_progress_line_detected() {
        assert!(is_rust_analyzer_progress_line("1/2 50% processing src/lib.rs"));
        assert!(is_rust_analyzer_progress_line("  1/2 50% processing src/lib.rs  "));
        assert!(!is_rust_analyzer_progress_line("at crate foo, file bar: Error E0308"));
        assert!(!is_rust_analyzer_progress_line("diagnostic scan complete"));
    }

    #[test]
    fn rust_analyzer_code_strips_rustc_hard_error_wrapper() {
        assert_eq!(
            parse_rust_analyzer_code("RustcHardError(\"E0308\")"),
            Some("E0308".to_string())
        );
        assert_eq!(
            parse_rust_analyzer_code("unused_imports"),
            Some("unused_imports".to_string())
        );
        assert_eq!(parse_rust_analyzer_code(""), None);
    }

    #[test]
    fn rust_analyzer_parses_grounded_fixture_line() {
        let line = "at crate ra_fixture, file /tmp/ra-fixture/src/lib.rs: Error RustcHardError(\"E0308\") from LineCol { line: 6, col: 12 } to LineCol { line: 6, col: 13 }: mismatched types: expected `String`, found integer";
        let parser = RustAnalyzerParser;
        let out = parser.parse(line, "", &req());
        assert_eq!(out.run_status, DiagnosticProbeRunStatus::Completed);
        assert_eq!(out.error_count, 1);
        assert!(!out.is_clean());
        assert_eq!(out.diagnostics.len(), 1);
        let d = &out.diagnostics[0];
        assert_eq!(d.severity, DiagnosticSeverity::Error);
        assert_eq!(d.code.as_deref(), Some("E0308"));
        assert_eq!(d.line, 6);
        assert_eq!(d.column, 12);
        assert!(d.message.starts_with("mismatched types"));
        assert_eq!(d.path, "src/lib.rs");
    }

    #[test]
    fn rust_analyzer_skips_progress_and_complete_lines() {
        let stdout = "1/2 50% processing src/lib.rs\n\
at crate ra_fixture, file /tmp/ra-fixture/src/lib.rs: Error RustcHardError(\"E0308\") from LineCol { line: 6, col: 12 } to LineCol { line: 6, col: 13 }: mismatched types\n\
2/2 100% processing src/lib.rs\n\
diagnostic scan complete\n";
        let parser = RustAnalyzerParser;
        let out = parser.parse(stdout, "", &req());
        assert_eq!(out.error_count, 1);
        assert_eq!(out.diagnostics.len(), 1);
    }

    #[test]
    fn rust_analyzer_parses_warning_severity() {
        let line = "at crate ra_fixture, file /tmp/ra-fixture/src/lib.rs: Warning unused_imports from LineCol { line: 1, col: 5 } to LineCol { line: 1, col: 10 }: unused import: `std::fs`";
        let parser = RustAnalyzerParser;
        let out = parser.parse(line, "", &req());
        assert_eq!(out.warning_count, 1);
        assert_eq!(out.error_count, 0);
        assert_eq!(out.diagnostics[0].severity, DiagnosticSeverity::Warning);
        assert_eq!(out.diagnostics[0].code.as_deref(), Some("unused_imports"));
    }
}
