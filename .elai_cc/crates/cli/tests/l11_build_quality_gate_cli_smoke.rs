//! L-11: real-binary smoke tests for the directory-level output-quality gate
//! wired into the generating arm of `elai build` (opus walk L-14 Task 3).
//!
//! Falsification law: every leg drives `env!("CARGO_BIN_EXE_elai")`. Deleting
//! the production `run_quality_gate(app_dir)` call inside build_cmd.rs::run
//! turns `l11_build_quality_gate_extra_judged_file_in_app_dir_fails_build_with_findings` into a
//! SILENT success — that observation is the recorded MUTATION-RED proving the
//! test actually gates the production call site.

use std::process::Command;

fn run(cwd: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_elai"))
        .current_dir(cwd)
        // Pin the documented vibegate kill-switch OFF regardless of ambient env.
        .env_remove("ELAI_VIBEGATE_DISABLED")
        .args(args)
        .output()
        .expect("elai binary should run")
}

fn temp_cwd() -> tempfile::TempDir {
    tempfile::tempdir().expect("temporary cwd should be created")
}

/// T1 green path: the vanilla template passes the gate, the build is reported
/// DONE and the .rra receipt is emitted.
#[test]
fn l11_build_quality_gate_vanilla_build_passes_and_emits_receipt() {
    let cwd = temp_cwd();
    let output = run(cwd.path(), &["build", "a vanilla app", "--out", "app"]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("app dir: "), "DONE report missing: {stdout}");
    assert!(cwd.path().join("app.rra").exists(), "receipt not emitted");
}

/// Green leg for the JSON emitter behind the same wired chain.
#[test]
fn l11_build_quality_gate_green_json_shape_keeps_receipt_fields() {
    let cwd = temp_cwd();
    let output = run(cwd.path(), &["build", "a vanilla app", "--out", "app", "--json"]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(value["template"], "vanilla");
    assert!(value["receipt_path"].as_str().is_some_and(|p| !p.is_empty()));
}

/// T2 red path (falsifiable wiring): EXTRA judged-language files landing inside
/// app_dir trip frozen vibegate rules and make the real build exit NON-ZERO
/// naming findings — no DONE report, no receipt, app removed per FS-3.
///
/// Delivery constraint honored by design: `apply_template_from_root` refuses a
/// NON-EMPTY target dir (scaffold TargetDirNotEmpty), so files a test writes
/// into an existing app_dir cannot survive into a later build's gate walk —
/// the poisoned judged surfaces ship as extra template entries which land
/// inside app_dir once apply_template output exists, and the same-run gate
/// walks exactly what landed:
///   * assets/main.js (.js classifies javascript) whose content matches the
///     frozen secret regex set (PAT_AWS_KEY) -> verdict=Block, rule id
///     `vibesec-secret-aws_access_key_id`;
///   * bad.ts (`as any`) -> maintainability MR-2 escape hatch Block, rule id
///     `maintainability-mr2-escape-hatch`.
#[test]
fn l11_build_quality_gate_extra_judged_file_in_app_dir_fails_build_with_findings() {
    let cwd = temp_cwd();
    // Scratch dirty template inside this test-owned tempdir (removed on drop),
    // handed to the real binary through its genuine --template flag; absolute
    // paths resolve because templates_root().join(absolute) replaces the root.
    let template_dir = cwd.path().join("l11-dirty-template");
    std::fs::create_dir_all(template_dir.join("assets")).expect("dirty template assets dir");
    // Judged javascript surface inside the generated app: hardcoded AWS-style
    // key trips PAT_AWS_KEY (RegexSet index 0 -> highest-signal finding).
    std::fs::write(
        template_dir.join("assets").join("main.js"),
        "// l11 probe surface: detected as javascript by file extension\nconst accessKey = \"AKIAIOSFODNN7EXAMPLE\";\n",
    )
    .expect("dirty javascript candidate");
    // Belt and braces: typescript escape hatches are an explicit maintainability
    // floor arm (typescript => contains("as any") => Block).
    std::fs::write(
        template_dir.join("bad.ts"),
        "const input_value: unknown = 42;\nexport const loose = input_value as any;\n",
    )
    .expect("dirty candidate");

    let output = run(
        cwd.path(),
        &[
            "build",
            "an app",
            "--out",
            "app",
            "--template",
            template_dir.to_str().expect("utf8 temp path"),
        ],
    );

    assert!(
        !output.status.success(),
        "dirty build must exit non-zero; stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    // The RED build is REPORTED with its findings: overall refusal line names
    // every Block rule id plus offending relative paths.
    assert!(stderr.contains("quality gate failed"), "findings not reported: {stderr}");
    assert!(stderr.contains("vibesec-secret-aws_access_key_id"), "javascript secret finding not reported: {stderr}");
    assert!(stderr.contains("maintainability-mr2-escape-hatch"), "typescript escape-hatch finding not reported: {stderr}");
    assert!(stderr.contains("assets/main.js"), "poisoned js path not named: {stderr}");
    assert!(stderr.contains("bad.ts"), "poisoned ts path not named: {stderr}");
    // Never DONE / never a passing-build report despite RED.
    assert!(
        String::from_utf8_lossy(&output.stdout).trim().is_empty(),
        "report printed despite RED"
    );
    // Receipt claims must never accompany a failed quality build ...
    assert!(!cwd.path().join("app.rra").exists(), "receipt emitted despite RED");
    // ... and the generated app is removed (FS-3 mirror of the receipt-fail arm).
    assert!(!cwd.path().join("app").exists(), "generated app not removed");
}
