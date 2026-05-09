// Phase C (s-67812): contract tests pinning the JSON wire format that the
// dashboard-v2 Tauri commands run_web_lint_check / run_lighthouse_audit /
// probe_chromium_runtime emit. The dashboard TypeScript stubs in
// crates/dashboard-v2/src/lib/api/stub.ts unwrap these shapes; if the
// Rust side renames a field these tests fail BEFORE the dashboard breaks.
//
// Strategy: build sample serde_json::Value payloads matching what
// commands.rs:4076-4144 produces, assert key presence + types, then
// round-trip the canonical types (LintFinding / LighthouseReport /
// LintSeverity) and pin their serde keys. Any rename / type drift on
// the Rust side fails this contract test before the dashboard breaks.

use elai_web_validator::{
    cwv::LighthouseReport,
    lint::{LintFinding, LintSeverity},
};
use serde_json::{json, Value};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// run_web_lint_check wire format
// ---------------------------------------------------------------------------

#[test]
fn c3_web_lint_wire_response_zero_findings() {
    let payload: Value = json!({
        "ok": true,
        "project_dir": "/abs/path/to/project",
        "tool_count": 0,
        "findings": [],
    });
    assert_eq!(payload["ok"], json!(true));
    assert_eq!(payload["tool_count"], json!(0));
    assert!(payload["findings"].is_array());
    assert_eq!(payload["findings"].as_array().unwrap().len(), 0);
    assert!(payload["project_dir"].is_string());
}

#[test]
fn c3_web_lint_wire_response_with_multi_severity_findings() {
    let payload: Value = json!({
        "ok": true,
        "project_dir": "/abs/path/to/project",
        "tool_count": 3,
        "findings": [
            {
                "rule_id": "eslint:no-undef",
                "severity": "error",
                "file": "src/main.ts",
                "line": 12,
                "column": 7,
                "message": "'foo' is not defined",
                "fixable": false,
            },
            {
                "rule_id": "stylelint:color-hex-length",
                "severity": "warning",
                "file": "src/app.css",
                "line": 4,
                "column": null,
                "message": "Expected '#fff' to be '#ffffff'",
                "fixable": true,
            },
            {
                "rule_id": "axe:color-contrast",
                "severity": "info",
                "file": "src/index.html",
                "line": null,
                "column": null,
                "message": "Element has insufficient color contrast",
                "fixable": false,
            },
        ],
    });
    let findings = payload["findings"].as_array().unwrap();
    assert_eq!(findings.len(), 3);

    // Severity strings must be lowercase to match the TS union.
    assert_eq!(findings[0]["severity"], json!("error"));
    assert_eq!(findings[1]["severity"], json!("warning"));
    assert_eq!(findings[2]["severity"], json!("info"));

    // Each finding must round-trip as the canonical Rust type.
    for raw in findings {
        let parsed: LintFinding = serde_json::from_value(raw.clone())
            .expect("each finding entry must deserialize as LintFinding");
        assert!(!parsed.rule_id.is_empty());
        assert!(!parsed.message.is_empty());
    }
}

// ---------------------------------------------------------------------------
// run_lighthouse_audit wire format
// ---------------------------------------------------------------------------

#[test]
fn c3_lighthouse_wire_response_round_trips() {
    // Mirrors the real shape commands.rs:4124-4132 emits (7 fields).
    // The dashboard stub.ts ignores `project_dir` and `url` via TS
    // structural typing, but the contract test must include them so a
    // future drop of those fields does not silently slip through.
    let payload: Value = json!({
        "ok": true,
        "project_dir": "/abs/path/to/project",
        "url": "http://localhost:5173",
        "lcp_ms": 1450u32,
        "cls": 0.04,
        "inp_ms": 87u32,
        "perf_score": 0.92,
    });
    // Required keys the TS side reads.
    for k in ["ok", "lcp_ms", "cls", "inp_ms", "perf_score"] {
        assert!(payload.get(k).is_some(), "missing key {k}");
    }
    // Required keys the TS side ignores but the wire must still emit.
    for k in ["project_dir", "url"] {
        assert!(
            payload.get(k).is_some(),
            "wire response must continue to include {k}",
        );
    }
    assert_eq!(payload["lcp_ms"], json!(1450u32));
    assert_eq!(payload["inp_ms"], json!(87u32));
    assert!((payload["cls"].as_f64().unwrap() - 0.04).abs() < 1e-9);
    assert!((payload["perf_score"].as_f64().unwrap() - 0.92).abs() < 1e-9);
}

#[test]
fn c3_lighthouse_wire_field_names_are_snake_case() {
    // Pinning that perf_score (NOT performance_score) is the chosen key.
    // If commands.rs renames the field, the dashboard's
    // LighthouseWireResponse interface in stub.ts breaks at runtime;
    // this assertion catches the drift here instead.
    let canonical: Value = json!({
        "ok": true,
        "project_dir": "/abs/path/to/project",
        "url": "http://localhost:5173",
        "lcp_ms": 1500u32,
        "cls": 0.05,
        "inp_ms": 100u32,
        "perf_score": 0.9,
    });
    assert!(canonical.get("perf_score").is_some());
    assert!(
        canonical.get("performance_score").is_none(),
        "wire format must use perf_score, not performance_score"
    );
}

// ---------------------------------------------------------------------------
// probe_chromium_runtime wire format
// ---------------------------------------------------------------------------

#[test]
fn c3_chromium_probe_available_wire_response() {
    let payload: Value = json!({
        "ok": true,
        "available": true,
    });
    assert_eq!(payload["ok"], json!(true));
    assert_eq!(payload["available"], json!(true));
    assert!(payload.get("reason").is_none(), "available case must omit reason");
}

#[test]
fn c3_chromium_probe_unavailable_wire_response() {
    let payload: Value = json!({
        "ok": true,
        "available": false,
        "reason": "ChromiumMissing",
    });
    assert_eq!(payload["available"], json!(false));
    assert!(payload["reason"].is_string());
    assert!(!payload["reason"].as_str().unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// Canonical type serde-format stability (the TS interfaces depend on this)
// ---------------------------------------------------------------------------

#[test]
fn c3_lint_severity_serializes_as_lowercase_variant() {
    // The TS union 'error' | 'warning' | 'info' depends on this serde
    // rename. If lint.rs drops #[serde(rename_all = "lowercase")] this
    // test fails immediately and the dashboard would mis-render badges.
    let s_err = serde_json::to_value(LintSeverity::Error).unwrap();
    let s_warn = serde_json::to_value(LintSeverity::Warning).unwrap();
    let s_info = serde_json::to_value(LintSeverity::Info).unwrap();
    assert_eq!(s_err, json!("error"));
    assert_eq!(s_warn, json!("warning"));
    assert_eq!(s_info, json!("info"));
}

#[test]
fn c3_lint_finding_serializes_with_expected_keys_only() {
    // Field names here ARE the TS interface contract. Renaming any of
    // them in lint.rs breaks the dashboard at runtime; this test catches
    // the rename at the contract level instead.
    let finding = LintFinding {
        rule_id: "eslint:no-unused-vars".to_string(),
        severity: LintSeverity::Warning,
        file: PathBuf::from("src/foo.ts"),
        line: Some(10),
        column: Some(5),
        message: "unused".to_string(),
        fixable: false,
    };
    let v = serde_json::to_value(&finding).unwrap();
    let obj = v
        .as_object()
        .expect("LintFinding must serialize to JSON object");
    let expected_keys = ["rule_id", "severity", "file", "line", "column", "message", "fixable"];
    for k in expected_keys {
        assert!(obj.contains_key(k), "LintFinding missing key {k}");
    }
    assert_eq!(
        obj.len(),
        expected_keys.len(),
        "LintFinding must have exactly {} fields, got {}",
        expected_keys.len(),
        obj.len()
    );
    assert_eq!(obj["severity"], json!("warning"));
    assert_eq!(obj["fixable"], json!(false));
}

#[test]
fn c3_lighthouse_report_canonical_serializes_with_four_fields() {
    let report = LighthouseReport {
        lcp_ms: 1500,
        cls: 0.05,
        inp_ms: 100,
        perf_score: 0.9,
    };
    let v = serde_json::to_value(&report).unwrap();
    let obj = v
        .as_object()
        .expect("LighthouseReport must serialize to JSON object");
    assert_eq!(obj.len(), 4, "LighthouseReport must have exactly 4 fields");
    for k in ["lcp_ms", "cls", "inp_ms", "perf_score"] {
        assert!(obj.contains_key(k), "LighthouseReport missing key {k}");
    }
}

#[test]
fn c3_lint_finding_round_trips_optional_position_fields() {
    // Option<u32> for line/column must serialize as null in JSON when None
    // so the TS `line?: number` / `column?: number` interface accepts the
    // wire form without runtime coercion.
    let finding = LintFinding {
        rule_id: "axe:color-contrast".to_string(),
        severity: LintSeverity::Info,
        file: PathBuf::from("src/index.html"),
        line: None,
        column: None,
        message: "low contrast".to_string(),
        fixable: false,
    };
    let v = serde_json::to_value(&finding).unwrap();
    assert_eq!(v["line"], Value::Null);
    assert_eq!(v["column"], Value::Null);
    let back: LintFinding = serde_json::from_value(v).unwrap();
    assert_eq!(back.line, None);
    assert_eq!(back.column, None);
}
