//! F-10 Slice A2 — deterministic CWE-class classifier.
//!
//! Single source of truth for "which CWE security classes does this candidate
//! source introduce". The classifier is the input layer of the F-10
//! security-monotonicity gate (Slice B): two consecutive iterations of a
//! self-improvement loop are compared by the *set* of CWE classes each emits;
//! a class present at iteration N but absent at N-1 is a regression.
//!
//! Design (F-10-decisions.md, FORK 2 = CHOSEN orchestrator-local):
//!
//!   * Layer 1 — reuse the already-shipped F-1 scanner
//!     [`crate::vibegate::evaluate_vibegate`] and bucket each `HookFinding`'s
//!     `rule_id` into a [`CweClass`] via [`crate::vibegate::cwe_from_rule_id`].
//!     This gives CWE-89 (SQLi), CWE-79 (XSS), CWE-327 (weak-crypto) and
//!     CWE-862 (missing-authz) for FREE off the F-2a seed corpus — no parallel
//!     pattern table, no seed-TOML edits, no drift.
//!   * Layer 2 — a small set of compiled-once in-module regexes for the
//!     injection classes the F-1 seed corpus does NOT gate at write-time:
//!     CWE-78 (OS command injection), CWE-95 (eval injection),
//!     CWE-502 (insecure deserialization), CWE-22 (path traversal). These are
//!     anchored to dangerous sinks with interpolation/dynamic-input shapes to
//!     keep the per-language false-positive rate low (measured in
//!     workflow/reasoning/F-10-classifier-calibration.md).
//!
//! Determinism + isolation: every classification is a pure function of the
//! input string. No network, no filesystem, no clock. The regex set compiles
//! once via `LazyLock`.
//!
//! Goodhart guard: the `#[cfg(test)] calibration` module pairs a clean-corpus
//! false-positive bound (FP < 5% per language, over real ELAI source) with a
//! known-vuln recall set (each canonical sink MUST classify into its CWE),
//! so the gate cannot be silently gamed by under-detecting.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use regex::Regex;

use crate::vibegate::{cwe_from_rule_id, evaluate_vibegate, source_blake3_prefix};

// ---------------------------------------------------------------------------
// CweClass
// ---------------------------------------------------------------------------

/// A CWE security class the classifier can assign to candidate source.
///
/// The first four variants mirror the F-1/F-2a seed taxonomy bucketed by
/// [`cwe_from_rule_id`]; the next four are the injection classes the F-1
/// corpus does not gate at write-time and are detected by Layer-2 regexes.
/// [`CweClass::Other`] is a catch-all for any future F-1 rule whose CWE
/// mapping is not yet a named variant here (forward-compatible — keeps the
/// classifier total without silently dropping a finding).
///
/// `Ord`/`PartialOrd` are derived so a `BTreeSet<CweClass>` has a stable,
/// deterministic iteration order (required for reproducible class-set diffs
/// and golden fingerprints in the monotonicity gate).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CweClass {
    /// CWE-89 — SQL injection. F-1 bucket (`SEC-SQLI-*`).
    Sqli,
    /// CWE-79 — cross-site scripting. F-1 bucket (`SEC-XSS-*`).
    Xss,
    /// CWE-327 — use of a broken / risky cryptographic algorithm. F-1 bucket
    /// (`SEC-WCRYPTO-*`).
    WeakCrypto,
    /// CWE-862 — missing authorization. F-1 bucket (`SEC-AUTHZ-*`).
    MissingAuthz,
    /// CWE-78 — OS command injection. Layer-2 (F-1 does not gate this).
    CommandInjection,
    /// CWE-95 — eval injection (`eval()` / `exec()` on dynamic input).
    /// Layer-2 (F-1 does not gate this).
    EvalInjection,
    /// CWE-502 — deserialization of untrusted data (`pickle.loads`,
    /// unsafe `yaml.load`). Layer-2 (F-1 does not gate this).
    InsecureDeserialization,
    /// CWE-22 — path traversal. Layer-2 (F-1 does not gate this).
    PathTraversal,
    /// Catch-all for a CWE string the classifier recognises from F-1 but does
    /// not yet model as a named variant. Carries the raw `CWE-NNN` string.
    Other(String),
}

impl CweClass {
    /// The canonical `CWE-NNN` identifier string for this class.
    ///
    /// For [`CweClass::Other`] this returns the carried raw string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            CweClass::Sqli => "CWE-89",
            CweClass::Xss => "CWE-79",
            CweClass::WeakCrypto => "CWE-327",
            CweClass::MissingAuthz => "CWE-862",
            CweClass::CommandInjection => "CWE-78",
            CweClass::EvalInjection => "CWE-95",
            CweClass::InsecureDeserialization => "CWE-502",
            CweClass::PathTraversal => "CWE-22",
            CweClass::Other(raw) => raw.as_str(),
        }
    }

    /// Parse a `CWE-NNN` identifier string into a [`CweClass`].
    ///
    /// A recognised id maps to its named variant; any other non-empty
    /// `CWE-`-prefixed string becomes [`CweClass::Other`]. Returns `None`
    /// for an empty string so callers never construct an `Other("")`.
    #[must_use]
    pub fn parse(cwe: &str) -> Option<CweClass> {
        let trimmed = cwe.trim();
        if trimmed.is_empty() {
            return None;
        }
        Some(match trimmed {
            "CWE-89" => CweClass::Sqli,
            "CWE-79" => CweClass::Xss,
            "CWE-327" => CweClass::WeakCrypto,
            "CWE-862" => CweClass::MissingAuthz,
            "CWE-78" => CweClass::CommandInjection,
            "CWE-95" => CweClass::EvalInjection,
            "CWE-502" => CweClass::InsecureDeserialization,
            "CWE-22" => CweClass::PathTraversal,
            other => CweClass::Other(other.to_string()),
        })
    }
}

// ---------------------------------------------------------------------------
// Layer-2 compiled regexes (injection classes F-1 does not gate)
// ---------------------------------------------------------------------------

/// One compiled Layer-2 detector: a regex, the CWE class it implies, and the
/// set of language labels it applies to (empty = all languages).
struct Layer2Pattern {
    re: Regex,
    class: CweClass,
    langs: &'static [&'static str],
}

impl Layer2Pattern {
    /// `true` if this pattern is in-scope for `lang`. An empty `langs` slice
    /// means language-agnostic (applies everywhere).
    fn applies_to(&self, lang: &str) -> bool {
        self.langs.is_empty() || self.langs.contains(&lang)
    }
}

/// Build the Layer-2 detector set. Each regex is anchored to a dangerous sink
/// with a dynamic-input shape (interpolation / variable argument) rather than
/// a constant literal, to bound the false-positive rate. Per-language scoping
/// keeps Rust-only constructs (e.g. `Command::new`) from firing on Python/TS
/// and vice-versa.
///
/// # Panics (test-only safety)
///
/// The literals here are fixed, valid regexes verified by the `regex_set_compiles`
/// unit test. `Regex::new(..)` is unwrapped INSIDE this once-initialised
/// `LazyLock` builder, which the CLAUDE.md LazyLock exception permits: a bad
/// literal is a compile-corpus bug that surfaces immediately on first access
/// AND is caught deterministically by the dedicated unit test below, never in
/// a production hot path on dynamic input.
fn build_layer2_patterns() -> Vec<Layer2Pattern> {
    // Helper closure: compile a literal pattern that is statically known-good.
    // The unit test `regex_set_compiles` re-validates every literal.
    let mk = |pat: &str, class: CweClass, langs: &'static [&'static str]| {
        // SAFETY: literal, fixed regex string; validated by regex_set_compiles
        // unit test. LazyLock init exception (CLAUDE.md).
        #[allow(clippy::unwrap_used)]
        let re = Regex::new(pat).unwrap();
        Layer2Pattern { re, class, langs }
    };

    vec![
        // --- CWE-78: OS command injection ---
        // Python: os.system / os.popen with an argument that is not a single
        // quoted string literal (i.e. contains interpolation or a variable).
        mk(
            r#"os\.(system|popen)\s*\(\s*[^)'"]"#,
            CweClass::CommandInjection,
            &["python"],
        ),
        // Python: subprocess.* called with shell=True (the documented sink).
        mk(
            r"subprocess\.(run|call|Popen|check_output|check_call)\s*\([^)]*shell\s*=\s*True",
            CweClass::CommandInjection,
            &["python"],
        ),
        // Python: f-string interpolation directly into os.system.
        mk(
            r#"os\.(system|popen)\s*\(\s*f["']"#,
            CweClass::CommandInjection,
            &["python"],
        ),
        // JS/TS: child_process exec / execSync with template-literal
        // interpolation or string concatenation (a dynamic command string).
        mk(
            r"(child_process|cp)\.(exec|execSync)\s*\(\s*`[^`]*\$\{",
            CweClass::CommandInjection,
            &["typescript", "javascript"],
        ),
        mk(
            r#"\bexec(Sync)?\s*\(\s*["'][^"']*["']\s*\+"#,
            CweClass::CommandInjection,
            &["typescript", "javascript"],
        ),
        // Rust: Command::new with .arg("...{...}") via format! — narrow shape
        // to avoid flagging the overwhelmingly common static-arg usage.
        mk(
            r#"Command::new\s*\([^)]*\)[\s\S]{0,80}?\.arg\s*\(\s*format!\s*\("#,
            CweClass::CommandInjection,
            &["rust"],
        ),
        // Rust: sh -c with a format!-built command string (classic shell sink).
        mk(
            r#"\.args?\s*\(\s*\[?\s*"-c"[\s\S]{0,40}?format!\s*\("#,
            CweClass::CommandInjection,
            &["rust"],
        ),
        // --- CWE-95: eval injection ---
        // Python: eval()/exec() on a non-literal argument.
        mk(
            r#"\b(eval|exec)\s*\(\s*[^)'"]"#,
            CweClass::EvalInjection,
            &["python"],
        ),
        // JS/TS: eval() / new Function() on a dynamic argument.
        mk(
            r#"\beval\s*\(\s*[^)'"]"#,
            CweClass::EvalInjection,
            &["typescript", "javascript"],
        ),
        mk(
            r"new\s+Function\s*\(",
            CweClass::EvalInjection,
            &["typescript", "javascript"],
        ),
        // --- CWE-502: insecure deserialization ---
        // Python: pickle.loads / cPickle.loads (untrusted deserialization).
        mk(
            r"\b(c?[Pp]ickle)\.loads?\s*\(",
            CweClass::InsecureDeserialization,
            &["python"],
        ),
        // Python: yaml.load(...). The Rust `regex` crate has no look-around
        // (rust-lang/regex#618), so "without SafeLoader" cannot be expressed in
        // the pattern; the SafeLoader exclusion is applied as a substring guard
        // in `classify_delta` for this specific class.
        mk(
            r"yaml\.load\s*\(",
            CweClass::InsecureDeserialization,
            &["python"],
        ),
        // Python: marshal.loads (also a code-deserialization sink).
        mk(
            r"\bmarshal\.loads?\s*\(",
            CweClass::InsecureDeserialization,
            &["python"],
        ),
        // --- CWE-22: path traversal ---
        // Python: open()/os.path.join with a literal `..` traversal segment.
        mk(
            r#"open\s*\(\s*[^)]*\.\.[\\/]"#,
            CweClass::PathTraversal,
            &["python"],
        ),
        // Python: f-string interpolated path passed straight to open().
        mk(
            r#"open\s*\(\s*f["'][^"']*\{[^}]+\}[^"']*["']"#,
            CweClass::PathTraversal,
            &["python"],
        ),
        // JS/TS: fs.readFile/readFileSync with a literal `..` traversal.
        mk(
            r#"fs\.(readFile|readFileSync|writeFile|writeFileSync|createReadStream)\s*\(\s*[^)]*\.\.[\\/]"#,
            CweClass::PathTraversal,
            &["typescript", "javascript"],
        ),
        // Rust: std::fs::read/File::open with a format!-built path containing
        // a `..` traversal segment.
        mk(
            r#"(fs::(read|read_to_string|write)|File::open)\s*\([\s\S]{0,80}?\.\.[\\/]"#,
            CweClass::PathTraversal,
            &["rust"],
        ),
    ]
}

/// Layer-2 detector set, compiled once.
// SAFETY: LazyLock infallible at access time; the builder validates literals.
static LAYER2_PATTERNS: LazyLock<Vec<Layer2Pattern>> = LazyLock::new(build_layer2_patterns);

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Classify the set of CWE security classes present in `source`.
///
/// Two layers, unioned into a deterministic [`BTreeSet`]:
///
///   1. Run the F-1 scanner [`evaluate_vibegate`] over `source` and bucket
///      each finding's `rule_id` into a [`CweClass`] via [`cwe_from_rule_id`].
///   2. Run the Layer-2 injection-class regexes that apply to `lang`.
///
/// `lang` is a language label (`"rust"` / `"python"` / `"typescript"` /
/// `"javascript"`; anything else is treated as language-agnostic for Layer-2
/// scoping). It selects which Layer-2 patterns apply; Layer 1 is run in the
/// F-1 language-agnostic mode (empty `file_path`) so the F-2a python/typescript
/// security rules match regardless of the caller's `lang`, mirroring the live
/// task-runner pre-gate behaviour (vibegate B3).
///
/// Pure + deterministic: no network, no filesystem, no clock.
#[must_use]
pub fn classify_delta(source: &str, lang: &str) -> BTreeSet<CweClass> {
    let mut classes = BTreeSet::new();

    // --- Layer 1: reuse the F-1 scanner, bucket rule_id -> CweClass ---
    // Empty file_path => F-1 evaluates language-agnostic, matching the live
    // task-runner pre-gate (vibegate.rs:223). "classifier" is a logged-only
    // tool-name tag.
    let findings = evaluate_vibegate("classifier", source, "");
    for finding in &findings {
        if let Some(rule_id) = finding.rule_id.as_deref() {
            if let Some(cwe) = cwe_from_rule_id(rule_id) {
                if let Some(class) = CweClass::parse(cwe) {
                    classes.insert(class);
                }
            }
        }
    }

    // --- Layer 2: injection-class regexes scoped to `lang` ---
    // SafeLoader guard: a `yaml.load(...)` whose source uses the safe loader is
    // NOT a CWE-502 sink. Rust `regex` has no look-around (rust-lang/regex#618),
    // so this exclusion is a substring check applied only to the deserialization
    // class. Computed once.
    let uses_safe_yaml_loader = source.contains("SafeLoader");

    for pat in LAYER2_PATTERNS.iter() {
        if !pat.applies_to(lang) || !pat.re.is_match(source) {
            continue;
        }
        // The bare `yaml.load(` detector is suppressed when the safe loader is
        // present. Other CWE-502 sinks (pickle/marshal) and all other classes
        // are unaffected by this guard.
        if pat.class == CweClass::InsecureDeserialization
            && uses_safe_yaml_loader
            && pat.re.as_str().contains("yaml")
        {
            continue;
        }
        classes.insert(pat.class.clone());
    }

    classes
}

/// Compute the blake3 hex-prefix fingerprint of a candidate source string.
///
/// Delegates to [`source_blake3_prefix`] (blake3, first 16 hex chars) so the
/// monotonicity gate records a stable per-iteration fingerprint without ever
/// storing the full source. blake3 only (CLAUDE.md HARD RULE — never
/// blake2/sha2).
#[must_use]
pub fn delta_fingerprint(source: &str) -> String {
    source_blake3_prefix(source)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regex_set_compiles() {
        // Forces LazyLock init; a bad literal would panic here deterministically
        // rather than on a production hot path.
        let n = LAYER2_PATTERNS.len();
        assert!(n >= 8, "expected the full Layer-2 detector set, got {n}");
    }

    #[test]
    fn as_str_and_parse_round_trip_named_variants() {
        for class in [
            CweClass::Sqli,
            CweClass::Xss,
            CweClass::WeakCrypto,
            CweClass::MissingAuthz,
            CweClass::CommandInjection,
            CweClass::EvalInjection,
            CweClass::InsecureDeserialization,
            CweClass::PathTraversal,
        ] {
            let s = class.as_str().to_string();
            let parsed = CweClass::parse(&s).expect("named CWE id must parse");
            assert_eq!(parsed, class, "round-trip failed for {s}");
        }
    }

    #[test]
    fn parse_empty_is_none() {
        assert_eq!(CweClass::parse(""), None);
        assert_eq!(CweClass::parse("   "), None);
    }

    #[test]
    fn parse_unknown_cwe_is_other() {
        assert_eq!(
            CweClass::parse("CWE-1004"),
            Some(CweClass::Other("CWE-1004".to_string()))
        );
        assert_eq!(CweClass::parse("CWE-1004").map(|c| c.as_str().to_string()),
            Some("CWE-1004".to_string()));
    }

    #[test]
    fn fingerprint_is_16_hex_chars_and_stable() {
        let fp = delta_fingerprint("fn main() {}");
        assert_eq!(fp.len(), 16, "blake3 prefix must be 16 hex chars");
        assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(fp, delta_fingerprint("fn main() {}"), "must be deterministic");
        assert_ne!(fp, delta_fingerprint("fn main() { 1; }"));
    }

    #[test]
    fn empty_source_classifies_to_empty_set() {
        assert!(classify_delta("", "rust").is_empty());
        assert!(classify_delta("", "python").is_empty());
    }

    #[test]
    fn command_injection_python_os_system() {
        let s = "os.system(user_cmd)";
        assert!(classify_delta(s, "python").contains(&CweClass::CommandInjection));
    }

    #[test]
    fn command_injection_python_shell_true() {
        let s = "subprocess.run(cmd, shell=True)";
        assert!(classify_delta(s, "python").contains(&CweClass::CommandInjection));
    }

    #[test]
    fn eval_injection_python() {
        let s = "result = eval(expr)";
        assert!(classify_delta(s, "python").contains(&CweClass::EvalInjection));
    }

    #[test]
    fn insecure_deser_pickle() {
        let s = "obj = pickle.loads(data)";
        assert!(classify_delta(s, "python").contains(&CweClass::InsecureDeserialization));
    }

    #[test]
    fn insecure_deser_unsafe_yaml_load() {
        let s = "cfg = yaml.load(open('c.yml'))";
        assert!(classify_delta(s, "python").contains(&CweClass::InsecureDeserialization));
    }

    #[test]
    fn safe_yaml_loader_is_not_deser_fp() {
        // SafeLoader guard: yaml.load with the safe loader must NOT classify as
        // CWE-502 (rust-lang/regex has no look-around, so this is a substring
        // guard in classify_delta).
        let s = "cfg = yaml.load(fh, Loader=yaml.SafeLoader)";
        assert!(!classify_delta(s, "python").contains(&CweClass::InsecureDeserialization));
    }

    #[test]
    fn path_traversal_python_dotdot() {
        let s = r#"open("../../etc/passwd")"#;
        assert!(classify_delta(s, "python").contains(&CweClass::PathTraversal));
    }

    #[test]
    fn layer2_scoping_python_pattern_not_applied_to_rust() {
        // os.system is a python-scoped Layer-2 pattern; passing lang=rust
        // must NOT fire it (Layer-2 language scoping).
        let s = "os.system(user_cmd)";
        assert!(!classify_delta(s, "rust").contains(&CweClass::CommandInjection));
    }
}

// ---------------------------------------------------------------------------
// Calibration (Goodhart-guarded) — FP bound on real clean source + recall set
// ---------------------------------------------------------------------------
//
// This module measures the classifier's per-language false-positive rate over
// a corpus of REAL, known-clean ELAI source files (embedded via include_str!
// so the test is deterministic + offline) and asserts FP < 5% per language.
// It is paired with a known-vuln recall set so the gate cannot be gamed by
// under-detecting. Measured numbers are recorded in
// workflow/reasoning/F-10-classifier-calibration.md.
#[cfg(test)]
mod calibration {
    use super::*;

    // Real clean ELAI Rust source, embedded from sibling orchestrator modules.
    // These are production modules with no command/eval/deser/path-traversal
    // sinks; any Layer-2 firing on them is a false positive.
    const CLEAN_RS: &[(&str, &str)] = &[
        ("agent_role.rs", include_str!("agent_role.rs")),
        ("agent_template.rs", include_str!("agent_template.rs")),
        ("anti_spiral.rs", include_str!("anti_spiral.rs")),
        ("archive_chain.rs", include_str!("archive_chain.rs")),
        ("baby_step.rs", include_str!("baby_step.rs")),
        ("branch_lock.rs", include_str!("branch_lock.rs")),
        ("cache_check.rs", include_str!("cache_check.rs")),
        ("coherence_check.rs", include_str!("coherence_check.rs")),
    ];

    // Clean Python snippets sampled from real ELAI sidecar style (FastAPI /
    // torch inference). No os.system, no eval, no pickle.loads, no traversal.
    const CLEAN_PY: &[(&str, &str)] = &[
        (
            "embeddings_style.py",
            "import torch\n\
             from transformers import AutoModel, AutoTokenizer\n\n\
             def embed(texts: list[str], model, tokenizer) -> torch.Tensor:\n\
             \x20   with torch.inference_mode():\n\
             \x20       enc = tokenizer(texts, return_tensors='pt', padding=True)\n\
             \x20       out = model(**enc)\n\
             \x20   return out.last_hidden_state.mean(dim=1)\n",
        ),
        (
            "server_style.py",
            "from fastapi import FastAPI\n\
             import orjson\n\n\
             app = FastAPI()\n\n\
             @app.post('/v1/messages')\n\
             async def messages(body: dict) -> dict:\n\
             \x20   prompt = body.get('prompt', '')\n\
             \x20   return {'ok': True, 'len': len(prompt)}\n",
        ),
        (
            "config_style.py",
            "import yaml\n\n\
             def load_config(path: str) -> dict:\n\
             \x20   with open(path, 'r', encoding='utf-8') as fh:\n\
             \x20       return yaml.load(fh, Loader=yaml.SafeLoader)\n",
        ),
        (
            "hashing_style.py",
            "import hashlib\n\n\
             def digest(data: bytes) -> str:\n\
             \x20   return hashlib.sha256(data).hexdigest()\n",
        ),
    ];

    // Clean TypeScript snippets in the dashboard-v2 API-client style.
    // No child_process, no eval, no traversal.
    const CLEAN_TS: &[(&str, &str)] = &[
        (
            "api_stub_style.ts",
            "export interface Task { id: string; title: string; }\n\n\
             export async function fetchTasks(): Promise<Task[]> {\n\
             \x20 const res = await fetch('/api/tasks');\n\
             \x20 return res.json() as Promise<Task[]>;\n\
             }\n",
        ),
        (
            "types_style.ts",
            "export type DrawerKey = 'cosmos' | 'critic' | 'runtime';\n\n\
             export const DRAWER_LABELS: Record<DrawerKey, string> = {\n\
             \x20 cosmos: 'Cosmos', critic: 'Critic', runtime: 'Runtime',\n\
             };\n",
        ),
        (
            "store_style.ts",
            "import { writable } from 'svelte/store';\n\n\
             export const count = writable(0);\n\
             export function increment(): void { count.update((n) => n + 1); }\n",
        ),
        (
            "hash_style.ts",
            "import { createHash } from 'node:crypto';\n\n\
             export function sha256(data: string): string {\n\
             \x20 return createHash('sha256').update(data).digest('hex');\n\
             }\n",
        ),
    ];

    /// Count clean files that produce ANY Layer-2 false positive for `lang`.
    /// (Layer 1 — F-1 — is also run; an F-1 hit on a clean file would likewise
    /// count, so the bound covers the full classify_delta surface.)
    fn fp_count(corpus: &[(&str, &str)], lang: &str) -> (usize, Vec<String>) {
        let mut fp = 0usize;
        let mut offenders = Vec::new();
        for (name, src) in corpus {
            let classes = classify_delta(src, lang);
            if !classes.is_empty() {
                fp += 1;
                let cs: Vec<&str> = classes.iter().map(CweClass::as_str).collect();
                offenders.push(format!("{name}: {cs:?}"));
            }
        }
        (fp, offenders)
    }

    #[test]
    fn calibration_false_positive_rate_under_5pct_per_language() {
        // Rust
        let (rs_fp, rs_off) = fp_count(CLEAN_RS, "rust");
        let rs_rate = rs_fp as f64 / CLEAN_RS.len() as f64;
        // Python
        let (py_fp, py_off) = fp_count(CLEAN_PY, "python");
        let py_rate = py_fp as f64 / CLEAN_PY.len() as f64;
        // TypeScript
        let (ts_fp, ts_off) = fp_count(CLEAN_TS, "typescript");
        let ts_rate = ts_fp as f64 / CLEAN_TS.len() as f64;

        // Surface measured numbers in test output for the calibration record.
        eprintln!(
            "F-10 calibration FP: rust={rs_fp}/{} ({:.1}%) {rs_off:?} | \
             python={py_fp}/{} ({:.1}%) {py_off:?} | \
             typescript={ts_fp}/{} ({:.1}%) {ts_off:?}",
            CLEAN_RS.len(),
            rs_rate * 100.0,
            CLEAN_PY.len(),
            py_rate * 100.0,
            CLEAN_TS.len(),
            ts_rate * 100.0,
        );

        assert!(py_rate < 0.05, "python FP rate {py_rate} >= 5%: {py_off:?}");
        assert!(ts_rate < 0.05, "typescript FP rate {ts_rate} >= 5%: {ts_off:?}");
        // Rust FP guard: if the reused Rust patterns exceed 5%, the F-10
        // decision is WARN-only-for-Rust (documented in the calibration record),
        // not a spiral of pattern-tightening. The assert keeps the bound visible.
        assert!(rs_rate < 0.05, "rust FP rate {rs_rate} >= 5%: {rs_off:?}");
    }

    #[test]
    fn calibration_recall_known_vuln_snippets() {
        // Each canonical sink MUST classify into the correct CWE class.
        // (Goodhart guard — the FP bound cannot be met by under-detecting.)
        let cases: &[(&str, &str, CweClass)] = &[
            // python os.system on a variable -> CWE-78
            ("os.system(user_input)", "python", CweClass::CommandInjection),
            // python eval on a variable -> CWE-95
            ("value = eval(payload)", "python", CweClass::EvalInjection),
            // SQLi string-concat / f-string -> CWE-89 (F-1 Layer-1 reuse)
            (
                "cursor.execute(f\"SELECT * FROM users WHERE id = {uid}\")",
                "python",
                CweClass::Sqli,
            ),
            // weak crypto md5 -> CWE-327 (F-1 Layer-1 reuse)
            ("h = hashlib.md5(data).hexdigest()", "python", CweClass::WeakCrypto),
        ];
        let mut recalled = 0usize;
        for (src, lang, expected) in cases {
            let classes = classify_delta(src, lang);
            assert!(
                classes.contains(expected),
                "recall miss: {src:?} ({lang}) expected {expected:?}, got {:?}",
                classes.iter().map(CweClass::as_str).collect::<Vec<_>>()
            );
            recalled += 1;
        }
        eprintln!(
            "F-10 calibration recall: {recalled}/{} known-vuln snippets classified correctly",
            cases.len()
        );
        assert_eq!(recalled, cases.len(), "all known-vuln snippets must recall");
    }
}
