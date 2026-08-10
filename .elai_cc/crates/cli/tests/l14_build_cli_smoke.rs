use std::process::Command;

fn run(cwd: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_elai"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("elai binary should run")
}

fn temp_cwd() -> tempfile::TempDir {
    tempfile::tempdir().expect("temporary cwd should be created")
}

#[test]
fn build_plan_only_text_reports_resolved_profile() {
    // Falsifies the production `CliAction::Build` dispatch arm.
    let cwd = temp_cwd();
    let output = run(cwd.path(), &["build", "a TODO web app with a REST API", "--plan-only"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("class:"));
    assert!(stdout.contains("profile:"));
    assert!(stdout.contains("not built: --plan-only"));
}

#[test]
fn build_plan_only_json_is_handler_output() {
    // Falsifies the production `build_cmd::run` JSON output call site.
    let cwd = temp_cwd();
    let output = run(cwd.path(), &["build", "a TODO web app with a REST API", "--plan-only", "--json"]);
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(value["plan_only"], true);
    assert!(value["class"].as_str().is_some_and(|class| !class.is_empty()));
}

#[test]
fn build_plan_only_writes_nothing() {
    // Falsifies the production plan-only read-only path.
    let cwd = temp_cwd();
    let output = run(cwd.path(), &["build", "a TODO web app with a REST API", "--plan-only"]);
    assert!(output.status.success());
    assert_eq!(cwd.path().read_dir().expect("read temp cwd").count(), 0);
}

#[test]
fn build_without_out_names_out() {
    let cwd = temp_cwd();
    let output = run(cwd.path(), &["build", "an app"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--out"));
}

#[test]
fn build_generates_app_and_receipt() {
    let cwd = temp_cwd();
    let output = run(cwd.path(), &["build", "a vanilla app", "--out", "app"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(cwd.path().join("app").read_dir().expect("app dir").next().is_some());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let receipt = stdout.lines().find_map(|line| line.strip_prefix("receipt: ")).expect("receipt line");
    assert!(cwd.path().join(receipt).exists(), "receipt path: {receipt}");
}

#[test]
fn build_refuses_non_empty_out_dir() {
    let cwd = temp_cwd(); let app = cwd.path().join("app"); std::fs::create_dir(&app).expect("app dir");
    let file = app.join("keep.txt"); std::fs::write(&file, b"keep").expect("file");
    let output = run(cwd.path(), &["build", "an app", "--out", "app"]);
    assert!(!output.status.success()); assert_eq!(std::fs::read(&file).expect("keep file"), b"keep");
}

#[test]
fn build_unknown_template_errors() {
    let cwd = temp_cwd(); let output = run(cwd.path(), &["build", "an app", "--out", "app", "--template", "nope"]);
    assert!(!output.status.success()); assert!(String::from_utf8_lossy(&output.stderr).contains("nope"));
}

#[test]
fn build_receipt_disabled_generates_nothing() {
    let cwd = temp_cwd();
    let output = Command::new(env!("CARGO_BIN_EXE_elai")).current_dir(cwd.path()).args(["build", "an app", "--out", "app"]).env("ELAI_WRAP_RRA_DISABLE", "1").output().expect("elai binary");
    assert!(!output.status.success()); assert!(!cwd.path().join("app").exists());
}

#[test]
fn build_requires_an_idea() {
    // Falsifies the production build parser required-argument validation.
    let cwd = temp_cwd();
    let output = run(cwd.path(), &["build"]);
    assert!(!output.status.success());
}
