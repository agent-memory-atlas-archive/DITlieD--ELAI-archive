// Phase B (s-44680): contract tests for the four work streams of the
// website-creation pipeline continuation.
//
// Streams covered:
//   B1 — Tier-2 AST: TreeSitterAstEvaluator must support html/css/svelte
//        and the 11 Tier-2 rules in data/rules/web.toml must be ast_pattern.
//   B2 — Real WebLint runner: lint::{LintFinding, LintSeverity,
//        LintRunnerConfig, NodeMissing variant of WebValidatorError,
//        run_web_lint signature}.
//   B3 — Tier-3 contrast: contrast::{srgb_to_linear, relative_luminance,
//        contrast_ratio, wcag_aa_pass, evaluate_contrast_in_css,
//        ContrastFinding}.
//   B4 — WebPerformance tier: WebValidationTier::WebPerformance variant +
//        pipeline_tree_navigator::web_validation_tiers_for_level adds
//        WebPerformance at the Concept decomposition level.
//        cwv::{LighthouseReport, run_lighthouse signature}.
//
// All tests are pure unit tests (no spawned processes, no network).

use std::path::{Path, PathBuf};

use elai_knowledge::TreeSitterAstEvaluator;
use elai_orchestrator::pipeline_tree_navigator::{
    web_validation_tiers_for_level, DecompositionLevel,
};
use elai_rules::predicate::AstEvaluator;
use elai_web_validator::{
    contrast::{
        contrast_ratio, evaluate_contrast_in_css, relative_luminance, srgb_to_linear,
        wcag_aa_pass, ContrastFinding, TextSize,
    },
    cwv::{LighthouseReport, LIGHTHOUSE_DEFAULT_TIMEOUT_SECS},
    lint::{LintFinding, LintRunnerConfig, LintSeverity},
    WebValidationTier, WebValidatorError,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn workspace_rules_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("data")
        .join("rules")
}

fn load_web_rules() -> toml::Value {
    let path = workspace_rules_dir().join("web.toml");
    let body = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    toml::from_str(&body).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

// ---------------------------------------------------------------------------
// B1 — TreeSitterAstEvaluator language registration
// ---------------------------------------------------------------------------

#[test]
fn b1_tree_sitter_evaluator_supports_html() {
    let eval = TreeSitterAstEvaluator::new();
    let src = "<html lang=\"en\"><body><h1>Title</h1></body></html>";
    let res = eval.evaluate_query("html", src, "(element) @e");
    assert!(res.is_ok(), "html should parse: {res:?}");
}

#[test]
fn b1_tree_sitter_evaluator_supports_css() {
    let eval = TreeSitterAstEvaluator::new();
    let src = ":root { --primary-hsl: 162 68% 51%; }";
    let res = eval.evaluate_query("css", src, "(declaration) @d");
    assert!(res.is_ok(), "css should parse: {res:?}");
}

#[test]
fn b1_tree_sitter_evaluator_supports_svelte() {
    let eval = TreeSitterAstEvaluator::new();
    // tree-sitter-svelte-ng accepts mixed markup + script.
    let src = "<script>let x = 1;</script>\n<h1>{x}</h1>";
    let res = eval.evaluate_query("svelte", src, "(element) @el");
    assert!(res.is_ok(), "svelte should parse: {res:?}");
}

#[test]
fn b1_tree_sitter_evaluator_rejects_unknown_language() {
    let eval = TreeSitterAstEvaluator::new();
    let res = eval.evaluate_query("klingon", "<html></html>", "(element) @e");
    assert!(res.is_err(), "unknown language must error");
}

// ---------------------------------------------------------------------------
// B1 — All 11 Tier-2 rules in web.toml are ast_pattern (no grep_pattern)
// ---------------------------------------------------------------------------

#[test]
fn b1_all_tier2_rules_use_ast_pattern() {
    let doc = load_web_rules();
    let rules = doc
        .get("rules")
        .and_then(|v| v.as_array())
        .expect("rules array present");
    let mut tier2_count = 0usize;
    let mut grep_in_tier2: Vec<String> = Vec::new();
    for rule in rules {
        let tier = rule.get("tier").and_then(|v| v.as_integer()).unwrap_or(0);
        if tier != 2 {
            continue;
        }
        tier2_count += 1;
        let id = rule
            .get("rule_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_owned();
        let pred = rule
            .get("predicate_json")
            .and_then(|v| v.as_str())
            .expect("predicate_json present");
        let parsed: serde_json::Value =
            serde_json::from_str(pred).expect("predicate_json is JSON");
        let ty = parsed
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        // WEB-022 (deep relative imports) is allowed to remain grep_pattern —
        // module-graph traversal is not what tree-sitter buys us. Everything
        // else in Tier 2 must be ast_pattern after Phase B.
        if id == "WEB-022" {
            continue;
        }
        if ty != "ast_pattern" {
            grep_in_tier2.push(format!("{id} -> type={ty}"));
        }
    }
    assert_eq!(tier2_count, 11, "tier 2 must remain 11 rules total");
    assert!(
        grep_in_tier2.is_empty(),
        "Tier-2 rules still using non-ast_pattern (Phase B must upgrade): {grep_in_tier2:?}"
    );
}

#[test]
fn b1_all_predicates_remain_valid_json() {
    let doc = load_web_rules();
    let rules = doc.get("rules").and_then(|v| v.as_array()).unwrap();
    for r in rules {
        let pred = r
            .get("predicate_json")
            .and_then(|v| v.as_str())
            .expect("predicate_json present");
        let _: serde_json::Value =
            serde_json::from_str(pred).expect("every predicate_json must be valid JSON");
    }
}

#[test]
fn b1_tier2_languages_are_registered_in_evaluator() {
    // Cross-check: every `language` declared in a Tier-2 ast_pattern predicate
    // must be supported by TreeSitterAstEvaluator. Otherwise rules silently
    // never fire in production.
    let eval = TreeSitterAstEvaluator::new();
    let doc = load_web_rules();
    let rules = doc.get("rules").and_then(|v| v.as_array()).unwrap();
    for r in rules {
        let tier = r.get("tier").and_then(|v| v.as_integer()).unwrap_or(0);
        if tier != 2 {
            continue;
        }
        let pred_str = r.get("predicate_json").and_then(|v| v.as_str()).unwrap();
        let pred: serde_json::Value = serde_json::from_str(pred_str).unwrap();
        if pred.get("type").and_then(|v| v.as_str()) != Some("ast_pattern") {
            continue;
        }
        let lang = pred.get("language").and_then(|v| v.as_str()).expect("language field");
        // Probe with a trivially-valid query and minimal source. We only care
        // that the language id resolves to a registered grammar.
        let probe = eval.evaluate_query(lang, "", "(_) @x");
        // A registered grammar yields Ok or InvalidQuery (latter from ts_query
        // parsing); an unregistered grammar yields UnsupportedLanguage.
        match probe {
            Ok(_) => {}
            Err(e) => {
                let msg = format!("{e:?}");
                assert!(
                    !msg.contains("UnsupportedLanguage"),
                    "Tier-2 rule references language '{lang}' that TreeSitterAstEvaluator does not register"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// B2 — WebLint public surface
// ---------------------------------------------------------------------------

#[test]
fn b2_lint_finding_serializes_with_prefixed_rule_id() {
    let f = LintFinding {
        rule_id: "eslint:no-undef".to_owned(),
        severity: LintSeverity::Warning,
        file: PathBuf::from("src/App.svelte"),
        line: Some(12),
        column: Some(4),
        message: "x is not defined".to_owned(),
        fixable: false,
    };
    let json = serde_json::to_string(&f).expect("serialize LintFinding");
    assert!(json.contains("eslint:no-undef"));
    assert!(json.contains("warning") || json.contains("Warning"));
}

#[test]
fn b2_lint_runner_config_default_is_safe() {
    let cfg = LintRunnerConfig::default();
    // Default config must not enable any tool — opting in is explicit so that
    // toolchains lacking node don't auto-fail.
    assert!(!cfg.eslint, "eslint must default to disabled");
    assert!(!cfg.stylelint, "stylelint must default to disabled");
    assert!(!cfg.axe, "axe must default to disabled");
}

#[test]
fn b2_node_missing_error_variant_exists() {
    // We assert the variant is present by constructing it. If the variant is
    // renamed or removed, this test fails to compile (which is the contract
    // signal we want).
    let e = WebValidatorError::NodeMissing {
        probe_command: "node --version".to_owned(),
    };
    let s = format!("{e}");
    assert!(s.to_lowercase().contains("node"));
}

#[tokio::test(flavor = "current_thread")]
async fn b2_run_web_lint_with_no_tools_enabled_returns_empty() {
    use std::path::PathBuf;
    let cfg = LintRunnerConfig::default();
    let dir = std::env::temp_dir();
    let res = elai_web_validator::lint::run_web_lint(&PathBuf::from(&dir), &cfg).await;
    // With no tools enabled: no subprocess spawned, returns empty Vec.
    let findings = res.expect("disabled-config call must succeed");
    assert!(findings.is_empty(), "no tools enabled => no findings");
}

// ---------------------------------------------------------------------------
// B3 — Contrast (pure Rust WCAG)
// ---------------------------------------------------------------------------

#[test]
fn b3_srgb_to_linear_at_endpoints() {
    let lin0 = srgb_to_linear(0.0);
    let lin1 = srgb_to_linear(1.0);
    assert!((lin0 - 0.0).abs() < 1e-9);
    assert!((lin1 - 1.0).abs() < 1e-9);
}

#[test]
fn b3_relative_luminance_white_is_one() {
    let l = relative_luminance(1.0, 1.0, 1.0);
    assert!((l - 1.0).abs() < 1e-9, "white luminance should be 1.0, got {l}");
}

#[test]
fn b3_relative_luminance_black_is_zero() {
    let l = relative_luminance(0.0, 0.0, 0.0);
    assert!((l - 0.0).abs() < 1e-9, "black luminance should be 0.0, got {l}");
}

#[test]
fn b3_contrast_ratio_white_on_black_is_21_to_1() {
    let r = contrast_ratio((1.0, 1.0, 1.0), (0.0, 0.0, 0.0));
    assert!(
        (r - 21.0).abs() < 1e-6,
        "white-on-black WCAG ratio is exactly 21:1, got {r}"
    );
}

#[test]
fn b3_contrast_ratio_identical_colors_is_one() {
    let r = contrast_ratio((0.5, 0.5, 0.5), (0.5, 0.5, 0.5));
    assert!((r - 1.0).abs() < 1e-9, "identical colors must yield 1.0:1, got {r}");
}

#[test]
fn b3_contrast_ratio_is_symmetric() {
    let a = contrast_ratio((0.1, 0.2, 0.3), (0.9, 0.8, 0.7));
    let b = contrast_ratio((0.9, 0.8, 0.7), (0.1, 0.2, 0.3));
    assert!((a - b).abs() < 1e-9, "contrast must be symmetric, got {a} vs {b}");
}

#[test]
fn b3_wcag_aa_thresholds() {
    // AA: 4.5:1 normal, 3:1 large.
    assert!(wcag_aa_pass(4.5, TextSize::Normal));
    assert!(!wcag_aa_pass(4.49, TextSize::Normal));
    assert!(wcag_aa_pass(3.0, TextSize::Large));
    assert!(!wcag_aa_pass(2.99, TextSize::Large));
}

#[test]
fn b3_evaluate_contrast_in_css_finds_low_contrast() {
    // Intentionally low-contrast pair: light gray text on white.
    let css = r#"
.body { color: #eeeeee; background: #ffffff; }
.ok { color: #000000; background: #ffffff; }
"#;
    let findings = evaluate_contrast_in_css(css);
    // We expect at least one ContrastFinding for the low-contrast block, and
    // no finding for the black-on-white block.
    assert!(
        findings.iter().any(|f: &ContrastFinding| f.ratio < 4.5),
        "should flag the low-contrast block; got {findings:?}"
    );
    assert!(
        findings.iter().all(|f: &ContrastFinding| f.ratio < 4.5),
        "should not flag passing pairs; got {findings:?}"
    );
}

// ---------------------------------------------------------------------------
// B4 — WebPerformance tier (variant + pipeline policy + cwv module)
// ---------------------------------------------------------------------------

#[test]
fn b4_web_validation_tier_has_web_performance_variant() {
    // Constructing the variant proves it exists at compile time.
    let t = WebValidationTier::WebPerformance;
    let s = serde_json::to_string(&t).expect("serialize");
    let back: WebValidationTier = serde_json::from_str(&s).expect("deserialize");
    assert_eq!(t, back);
}

#[test]
fn b4_concept_level_includes_web_performance() {
    let tiers = web_validation_tiers_for_level(DecompositionLevel::Concept);
    assert!(
        tiers.contains(&WebValidationTier::WebPerformance),
        "WebPerformance must run at Concept level (heaviest gate); got {tiers:?}"
    );
}

#[test]
fn b4_baby_step_does_not_include_web_performance() {
    let tiers = web_validation_tiers_for_level(DecompositionLevel::BabyStep);
    assert!(
        !tiers.contains(&WebValidationTier::WebPerformance),
        "WebPerformance must NOT run at BabyStep level (too expensive); got {tiers:?}"
    );
}

#[test]
fn b4_lighthouse_report_round_trips_serde() {
    let r = LighthouseReport {
        lcp_ms: 1234,
        cls: 0.05,
        inp_ms: 200,
        perf_score: 0.93,
    };
    let json = serde_json::to_string(&r).expect("serialize");
    let back: LighthouseReport = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(r.lcp_ms, back.lcp_ms);
    assert!((r.cls - back.cls).abs() < 1e-9);
    assert_eq!(r.inp_ms, back.inp_ms);
    assert!((r.perf_score - back.perf_score).abs() < 1e-9);
}

#[test]
fn b4_lighthouse_default_timeout_is_named() {
    // The constant must exist and be a sane value (>= 30s, <= 600s).
    assert!(LIGHTHOUSE_DEFAULT_TIMEOUT_SECS >= 30);
    assert!(LIGHTHOUSE_DEFAULT_TIMEOUT_SECS <= 600);
}

#[test]
fn b4_chromium_missing_error_variant_exists() {
    let e = WebValidatorError::ChromiumMissing {
        probe_command: "google-chrome --version".to_owned(),
    };
    let s = format!("{e}");
    assert!(s.to_lowercase().contains("chromium") || s.to_lowercase().contains("chrome"));
}
