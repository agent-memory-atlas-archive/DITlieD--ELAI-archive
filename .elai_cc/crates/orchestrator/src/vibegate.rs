//! F-1 VibeGate — compound evaluator on the PreToolUse path.
//!
//! Provides [`evaluate_vibegate`], an infallible synchronous function that
//! runs two security axes against candidate source:
//!
//!   1. **Secret axis** — [`crate::vibegate_secrets::SECRET_PATTERNS`] regex set.
//!      On match: [`HookFinding`] with `verdict=Block`, `category="vibesec_secret"`.
//!   2. **Code-vuln axis** — F-2a security rules (GrepPattern, SEC-SQLI-* / SEC-XSS-* /
//!      SEC-WCRYPTO-* / SEC-AUTHZ-*) evaluated via [`elai_rules::rule_engine::DefaultRuleEngine`].
//!      On Block verdict: [`HookFinding`] with `verdict=Block`, `category="vibesec_code_vuln"`.
//!
//! The function is fail-closed: any internal error (rule compile failure, etc.)
//! is logged via `tracing::error` and treated as a Block finding so the gate
//! can never be bypassed by an unexpected engine fault.
//!
//! SSB emission of [`crate::ssb::SsbPayload::VibeGateBlocked`] rows uses
//! [`emit_vibegate_blocked_to_ssb`] — called from the async
//! [`crate::hook_dispatcher::InProcessHookDispatcher::dispatch_pre_tool_use`]
//! after `evaluate_vibegate` returns.
//!
//! Seed integrity: [`check_vibegate_seed_integrity`] wraps
//! [`elai_rules::vibesec_seed::verify_vibesec_seed_integrity`]. Call once at
//! startup. Failure: `tracing::error` + returns `false`, and the dispatcher
//! sets a flag to return a fail-closed Block on every subsequent call.

use std::sync::{Arc, LazyLock};

use elai_rules::predicate::{
    AstError, AstEvaluator, AstMatch, DkgError, DkgQuerier, ReachabilityMatch,
    ReachabilityParams,
};
use elai_rules::rule_engine::{
    CompiledPredicate, DefaultRuleEngine, EvaluationContext, FsmPhaseBit, Rule, RuleEngine,
};
use rustc_hash::FxHashMap;
use tracing::{error, warn};

use crate::hook_dispatcher::{HookFinding, HookTier, HookVerdictKind};
use crate::ssb::{
    SecurityRuleBlockedPayload, Ssb, SsbPayload, VibeGateBlockedKind, VibeGateBlockedPayload,
};
use crate::vibegate_secrets::{SECRET_LABELS, SECRET_PATTERNS};

// ---------------------------------------------------------------------------
// Category tags emitted on HookFinding — match the VibeGateBlockedKind labels.
// ---------------------------------------------------------------------------

/// `category` field on [`HookFinding`] for a secret-scan block.
pub const VIBESEC_SECRET_CATEGORY: &str = "vibesec_secret";

/// `category` field on [`HookFinding`] for a code-vuln block.
pub const VIBESEC_CODE_VULN_CATEGORY: &str = "vibesec_code_vuln";

/// F-1 Task 6 (W-2) — the ONLY VibeGate env var: a kill-switch, default OFF
/// (unset). VibeGate enforcement AND `VibeGateBlocked` / `SecurityRuleBlocked`
/// SSB emission are ON by default in production; setting this to a truthy value
/// (`"1"` / `"true"` / `"yes"` / `"on"`) DISABLES them. There is deliberately
/// NO default-off `*_ENABLE`-style flag — that is the wire-dark antipattern
/// the F-plan W-2 doctrine forbids.
pub const ELAI_VIBEGATE_DISABLED_ENV: &str = "ELAI_VIBEGATE_DISABLED";

// ---------------------------------------------------------------------------
// NoOp AST / DKG impls (F-2a security rules are all GrepPattern)
// ---------------------------------------------------------------------------

struct NoOpAst;
impl AstEvaluator for NoOpAst {
    fn evaluate_query(
        &self,
        _language: &str,
        _source: &str,
        _query: &str,
    ) -> Result<Vec<AstMatch>, AstError> {
        Ok(vec![])
    }
    fn validate_query(&self, _language: &str, _query: &str) -> Result<(), AstError> {
        Ok(())
    }
}

struct NoOpDkg;
impl DkgQuerier for NoOpDkg {
    fn query_reachability(
        &self,
        _params: &ReachabilityParams,
    ) -> Result<Vec<ReachabilityMatch>, DkgError> {
        Ok(vec![])
    }
    fn validate_params(&self, _params: &ReachabilityParams) -> Result<(), DkgError> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Compiled security rules (once-initialized via LazyLock)
// ---------------------------------------------------------------------------

/// Compiled F-2a security seed rules. Initialised once on first access.
/// The loader is infallible at the `LazyLock` level (failure returns empty vec,
/// logged via `tracing::warn` inside `load_security_seed_rules_builtin`).
// SAFETY: LazyLock infallible init (CLAUDE.md LazyLock exception — the
// function handles its own errors and never panics).
static SECURITY_RULES: LazyLock<Vec<(Rule, CompiledPredicate)>> = LazyLock::new(|| {
    let engine = DefaultRuleEngine::new(Box::new(NoOpAst), Box::new(NoOpDkg));
    crate::rule_loader::load_security_seed_rules_builtin(&engine)
});

// Re-use a single engine for evaluation. The engine is stateless after
// construction.
// SAFETY: LazyLock infallible (DefaultRuleEngine::new never fails).
static RULE_ENGINE: LazyLock<DefaultRuleEngine> =
    LazyLock::new(|| DefaultRuleEngine::new(Box::new(NoOpAst), Box::new(NoOpDkg)));

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Evaluate the VibeGate compound security check against `source`.
///
/// # Arguments
///
/// * `tool_name`  — name of the tool being pre-gated (logged only).
/// * `source`     — candidate source text (file content / edit delta).
/// * `file_path`  — project-relative path (used by language-filter in rules).
///
/// # Returns
///
/// A `Vec<HookFinding>` (may be empty on clean pass). Multiple findings are
/// possible if both axes fire. The function is **infallible** — no `Result`.
/// Internal engine errors are logged and converted to fail-closed Block
/// findings so the gate can never be bypassed by an unexpected fault.
#[must_use]
pub fn evaluate_vibegate(tool_name: &str, source: &str, file_path: &str) -> Vec<HookFinding> {
    let mut findings = Vec::new();

    // --- Axis 1: secret scan ---
    run_secret_axis(source, file_path, &mut findings);

    // --- Axis 2: code-vuln rules ---
    run_code_vuln_axis(tool_name, source, file_path, &mut findings);

    findings
}

/// Compute the blake3 hex prefix (first 16 chars) of a source string.
/// Used in SSB payloads — never stores the full source.
#[must_use]
pub fn source_blake3_prefix(source: &str) -> String {
    let hex = blake3::hash(source.as_bytes()).to_hex();
    hex[..16].to_string()
}

// ---------------------------------------------------------------------------
// Secret axis
// ---------------------------------------------------------------------------

fn run_secret_axis(source: &str, file_path: &str, out: &mut Vec<HookFinding>) {
    let matches = SECRET_PATTERNS.matches(source);
    if !matches.matched_any() {
        return;
    }
    // Emit one finding for the first matched pattern (highest signal).
    let first_label = matches
        .iter()
        .next()
        .and_then(|idx| SECRET_LABELS.get(idx).copied())
        .unwrap_or("secret_pattern");

    let message = format!(
        "vibesec_secret: secret pattern '{}' detected in {}",
        first_label, file_path
    );
    out.push(HookFinding {
        tier: HookTier::T1,
        verdict: HookVerdictKind::Block,
        rule_id: Some(format!("vibesec-secret-{}", first_label)),
        category: VIBESEC_SECRET_CATEGORY.to_string(),
        message,
        fix_template: None,
    });
}

// ---------------------------------------------------------------------------
// Code-vuln axis
// ---------------------------------------------------------------------------

fn run_code_vuln_axis(
    tool_name: &str,
    source: &str,
    file_path: &str,
    out: &mut Vec<HookFinding>,
) {
    let rules = &*SECURITY_RULES;
    if rules.is_empty() {
        // Seed rules failed to load — fail-closed.
        error!(
            tool_name,
            file_path,
            "vibegate: security seed rules are empty; fail-closed block emitted"
        );
        out.push(HookFinding {
            tier: HookTier::T1,
            verdict: HookVerdictKind::Block,
            rule_id: Some("vibesec-seed-load-failed".to_string()),
            category: VIBESEC_CODE_VULN_CATEGORY.to_string(),
            message: format!(
                "vibesec_code_vuln: security rules failed to load; tool '{}' blocked fail-closed",
                tool_name
            ),
            fix_template: None,
        });
        return;
    }

    // B3: the live production caller (task_runner baby-step pre-gate) passes the
    // task-summary text with file_path="" (HookInvocation::for_pre_tool_use sets
    // file_path=None). With no extension the rule engine's per-language GrepPattern
    // filter would skip every language-scoped security rule, silently bypassing the
    // gate. When the path is empty OR its language is unknown, evaluate in
    // language-agnostic mode so the F-2a security patterns match the text regardless
    // of extension. (The per-Edit/Write byte-level path that carries a real file_path
    // is a Tier-1.5 follow-up — see workflow/reasoning/F-1-decisions.md.)
    let language_agnostic = file_path.is_empty()
        || detect_language_label(file_path) == "unknown";

    let context = EvaluationContext {
        action: tool_name,
        source,
        file_path,
        fsm_phase: FsmPhaseBit::EXECUTING,
        metric_values: FxHashMap::default(),
        language_agnostic,
    };

    match RULE_ENGINE.evaluate(rules, &context) {
        Ok(verdicts) => {
            for v in &verdicts {
                if matches!(v.result, elai_rules::predicate::VerdictResult::Block) {
                    let message = format!(
                        "vibesec_code_vuln: rule '{}' blocked in {}",
                        v.rule_id, file_path
                    );
                    out.push(HookFinding {
                        tier: HookTier::T1,
                        verdict: HookVerdictKind::Block,
                        rule_id: Some(v.rule_id.clone()),
                        category: VIBESEC_CODE_VULN_CATEGORY.to_string(),
                        message,
                        fix_template: None,
                    });
                    // Short-circuit: one block finding is enough for the hot path.
                    return;
                }
            }
        }
        Err(e) => {
            error!(
                tool_name,
                file_path,
                error = %e,
                "vibegate: rule engine evaluation error; fail-closed block emitted"
            );
            out.push(HookFinding {
                tier: HookTier::T1,
                verdict: HookVerdictKind::Block,
                rule_id: Some("vibesec-engine-error".to_string()),
                category: VIBESEC_CODE_VULN_CATEGORY.to_string(),
                message: format!(
                    "vibesec_code_vuln: rule engine error for tool '{}'; blocked fail-closed",
                    tool_name
                ),
                fix_template: None,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// SSB emission
// ---------------------------------------------------------------------------

/// Emit [`SsbPayload::VibeGateBlocked`] rows for each vibe-gate finding.
///
/// Iterates `findings` and emits one SSB row per `vibesec_secret` or
/// `vibesec_code_vuln` Block finding. Non-Block and non-vibesec findings are
/// skipped (they travel through `emit_hook_events_to_ssb` as HookEvent rows).
///
/// Default-ON (W-2): emits with NO env override. The ONLY env var is the
/// kill-switch [`ELAI_VIBEGATE_DISABLED_ENV`] — when set truthy this no-ops.
/// A `SecurityRuleBlocked` flat beacon is also emitted for every CodeVuln
/// finding so the dashboard security panel can trend rule firings.
///
/// # Parameters
///
/// * `writer`    — shared SSB writer.
/// * `agent_id`  — agent id string (e.g. `"task_runner"`).
/// * `superstep` — superstep counter.
/// * `source`    — full source text (used to compute blake3 prefix only).
/// * `file_path` — project-relative file path.
/// * `fsm_state` — optional current FSM state label.
/// * `findings`  — findings returned by [`evaluate_vibegate`].
///
/// Returns the number of `VibeGateBlocked` rows written.
pub async fn emit_vibegate_blocked_to_ssb(
    writer: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    source: &str,
    file_path: &str,
    fsm_state: Option<&str>,
    findings: &[HookFinding],
) -> usize {
    // B1 (W-2) — default-ON: VibeGateBlocked + SecurityRuleBlocked rows emit in
    // production with NO env override. The ONLY env var is the kill-switch
    // ELAI_VIBEGATE_DISABLED (default unset = enabled). The previous default-off
    // ELAI_VIBE_GATE_ENABLE / ELAI_SECURITY_RULE_BLOCKED_ENABLE pair was the exact
    // wire-dark antipattern the F-plan W-2 doctrine forbids.
    if is_env_truthy(ELAI_VIBEGATE_DISABLED_ENV) {
        return 0;
    }

    let blake3_prefix = source_blake3_prefix(source);
    let mut emitted = 0usize;

    for finding in findings {
        if finding.verdict != HookVerdictKind::Block {
            continue;
        }

        match finding.category.as_str() {
            VIBESEC_SECRET_CATEGORY => {
                let payload = SsbPayload::VibeGateBlocked(VibeGateBlockedPayload {
                    kind: VibeGateBlockedKind::Secret,
                    rule_id: finding.rule_id.clone(),
                    cwe: None,
                    file_path: file_path.to_string(),
                    source_blake3: blake3_prefix.clone(),
                    citation: None,
                    fsm_state: fsm_state.map(ToOwned::to_owned),
                });
                match writer
                    .write_payload(agent_id, superstep, payload, Vec::new())
                    .await
                {
                    Ok(_) => emitted += 1,
                    Err(e) => warn!(
                        error = %e,
                        category = VIBESEC_SECRET_CATEGORY,
                        "vibegate: failed to emit VibeGateBlocked Secret row"
                    ),
                }
            }
            VIBESEC_CODE_VULN_CATEGORY => {
                // Extract CWE from rule_id if possible (e.g. SEC-SQLI-001 → CWE-89).
                let cwe = finding.rule_id.as_deref().and_then(cwe_from_rule_id);
                let payload = SsbPayload::VibeGateBlocked(VibeGateBlockedPayload {
                    kind: VibeGateBlockedKind::CodeVuln,
                    rule_id: finding.rule_id.clone(),
                    cwe: cwe.map(ToOwned::to_owned),
                    file_path: file_path.to_string(),
                    source_blake3: blake3_prefix.clone(),
                    citation: None,
                    fsm_state: fsm_state.map(ToOwned::to_owned),
                });
                match writer
                    .write_payload(agent_id, superstep, payload, Vec::new())
                    .await
                {
                    Ok(_) => emitted += 1,
                    Err(e) => warn!(
                        error = %e,
                        category = VIBESEC_CODE_VULN_CATEGORY,
                        "vibegate: failed to emit VibeGateBlocked CodeVuln row"
                    ),
                }

                // SecurityRuleBlocked flat beacon — emitted alongside every
                // CodeVuln block (default-ON, no separate env gate per B1/W-2)
                // so the dashboard security panel can trend rule firings.
                if let Some(rule_id) = finding.rule_id.as_deref() {
                    let cwe_str = cwe.unwrap_or("CWE-unknown").to_string();
                    let language = detect_language_label(file_path);
                    let sec_payload =
                        SsbPayload::SecurityRuleBlocked(SecurityRuleBlockedPayload {
                            rule_id: rule_id.to_string(),
                            cwe: cwe_str,
                            file_path: file_path.to_string(),
                            language: language.to_string(),
                            candidate_blake3: blake3_prefix.clone(),
                        });
                    match writer
                        .write_payload(agent_id, superstep, sec_payload, Vec::new())
                        .await
                    {
                        Ok(_) => {}
                        Err(e) => warn!(
                            error = %e,
                            rule_id,
                            "vibegate: failed to emit SecurityRuleBlocked row"
                        ),
                    }
                }
            }
            _ => {}
        }
    }

    emitted
}

// ---------------------------------------------------------------------------
// Seed integrity check
// ---------------------------------------------------------------------------

/// Verify vibesec seed integrity against the live DB.
///
/// Returns `true` when the digest matches; `false` on any failure.
/// Failure is logged via `tracing::error`.
///
/// Call once at startup (e.g. from [`crate::hook_dispatcher::InProcessHookDispatcher`]
/// construction or the task_runner setup path).
pub fn check_vibegate_seed_integrity(conn: &rusqlite::Connection) -> bool {
    match elai_rules::vibesec_seed::verify_vibesec_seed_integrity(conn) {
        Ok(()) => true,
        Err(e) => {
            error!(
                error = %e,
                "vibegate: vibesec seed integrity check failed; \
                 VibeGate will block all code-vuln checks fail-closed"
            );
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Test whether an env var is set to a truthy value (`"1"`, `"true"`, `"yes"`, `"on"`).
fn is_env_truthy(var: &str) -> bool {
    std::env::var(var)
        .map(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on" | "TRUE" | "YES" | "ON"))
        .unwrap_or(false)
}

/// Map a rule_id to its associated CWE identifier string.
///
/// Derived from the F-2a seed rule taxonomy:
///   SEC-SQLI-*   → CWE-89
///   SEC-XSS-*    → CWE-79
///   SEC-WCRYPTO-* → CWE-327
///   SEC-AUTHZ-*  → CWE-862
///
/// Made `pub` for F-10: the [`crate::cwe_classifier`] reuses this single
/// source-of-truth bucketing so the F-1 scanner and the monotonicity gate
/// never drift apart on CWE class assignment.
#[must_use]
pub fn cwe_from_rule_id(rule_id: &str) -> Option<&'static str> {
    if rule_id.starts_with("SEC-SQLI-") {
        Some("CWE-89")
    } else if rule_id.starts_with("SEC-XSS-") {
        Some("CWE-79")
    } else if rule_id.starts_with("SEC-WCRYPTO-") {
        Some("CWE-327")
    } else if rule_id.starts_with("SEC-AUTHZ-") {
        Some("CWE-862")
    } else {
        None
    }
}

/// Derive a language label from a file path extension.
fn detect_language_label(file_path: &str) -> &'static str {
    let lower = file_path.to_lowercase();
    if lower.ends_with(".rs") {
        "rust"
    } else if lower.ends_with(".py") || lower.ends_with(".pyi") {
        "python"
    } else if lower.ends_with(".ts") || lower.ends_with(".tsx") {
        "typescript"
    } else if lower.ends_with(".js") || lower.ends_with(".jsx") {
        "javascript"
    } else {
        "unknown"
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_source_produces_no_findings() {
        let findings = evaluate_vibegate(
            "edit_tool",
            "fn add(a: i32, b: i32) -> i32 { a + b }",
            "src/math.rs",
        );
        let blocks: Vec<_> = findings
            .iter()
            .filter(|f| f.verdict == HookVerdictKind::Block)
            .collect();
        assert!(blocks.is_empty(), "clean Rust source must produce no blocks");
    }

    #[test]
    fn sqli_in_task_text_with_empty_file_path_produces_code_vuln_block() {
        // B3 regression: the live production caller (task_runner baby-step
        // pre-gate) passes the task-summary text with file_path="". Before the
        // fix, the per-language GrepPattern filter skipped every language-scoped
        // security rule (unknown extension), so the code-vuln axis never fired
        // on the one live path. With language-agnostic evaluation, a python-
        // shaped SQLi string must now produce a code-vuln block even with no
        // file extension.
        let source = r#"cursor.execute(f"SELECT * FROM users WHERE id = {uid}")"#;
        let findings = evaluate_vibegate("edit_tool", source, "");
        assert!(
            findings.iter().any(|f| f.verdict == HookVerdictKind::Block
                && f.category == VIBESEC_CODE_VULN_CATEGORY),
            "SQLi in task text with empty file_path must produce a vibesec_code_vuln block (B3)"
        );
    }

    #[test]
    fn clean_task_text_with_empty_file_path_produces_no_block() {
        // B3 false-positive guard: a benign string with no file extension must
        // NOT block even under language-agnostic evaluation.
        let findings = evaluate_vibegate(
            "edit_tool",
            "implement a function that adds two integers and returns the sum",
            "",
        );
        let blocks: Vec<_> = findings
            .iter()
            .filter(|f| f.verdict == HookVerdictKind::Block)
            .collect();
        assert!(
            blocks.is_empty(),
            "benign task text with empty file_path must produce no blocks (B3 FP guard)"
        );
    }

    #[test]
    fn aws_key_produces_secret_block() {
        let source = "const KEY: &str = \"AKIAIOSFODNN7EXAMPLEKEY1\";";
        let findings = evaluate_vibegate("edit_tool", source, "src/config.rs");
        assert!(
            findings
                .iter()
                .any(|f| f.verdict == HookVerdictKind::Block
                    && f.category == VIBESEC_SECRET_CATEGORY),
            "AWS key must produce a vibesec_secret block"
        );
    }

    #[test]
    fn pem_key_produces_secret_block() {
        let source = "# -----BEGIN RSA PRIVATE KEY-----\n# base64data\n";
        let findings = evaluate_vibegate("edit_tool", source, "scripts/deploy.py");
        assert!(
            findings
                .iter()
                .any(|f| f.category == VIBESEC_SECRET_CATEGORY),
            "PEM key must produce a vibesec_secret block"
        );
    }

    #[test]
    fn source_blake3_prefix_is_16_chars() {
        let prefix = source_blake3_prefix("hello world");
        assert_eq!(prefix.len(), 16, "blake3 prefix must be 16 hex chars");
    }

    #[test]
    fn cwe_from_rule_id_maps_correctly() {
        assert_eq!(cwe_from_rule_id("SEC-SQLI-001"), Some("CWE-89"));
        assert_eq!(cwe_from_rule_id("SEC-XSS-002"), Some("CWE-79"));
        assert_eq!(cwe_from_rule_id("SEC-WCRYPTO-003"), Some("CWE-327"));
        assert_eq!(cwe_from_rule_id("SEC-AUTHZ-001"), Some("CWE-862"));
        assert_eq!(cwe_from_rule_id("UNKNOWN-001"), None);
    }

    #[test]
    fn detect_language_label_works() {
        assert_eq!(detect_language_label("src/main.rs"), "rust");
        assert_eq!(detect_language_label("app.py"), "python");
        assert_eq!(detect_language_label("index.ts"), "typescript");
        assert_eq!(detect_language_label("page.tsx"), "typescript");
        assert_eq!(detect_language_label("README.md"), "unknown");
    }
}
