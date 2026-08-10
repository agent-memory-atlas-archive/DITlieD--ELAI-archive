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
fn build_without_plan_only_fails_closed() {
    // Falsifies the production generating-arm refusal.
    let cwd = temp_cwd();
    let output = run(cwd.path(), &["build", "an app"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("receipt"));
}

#[test]
fn build_requires_an_idea() {
    // Falsifies the production build parser required-argument validation.
    let cwd = temp_cwd();
    let output = run(cwd.path(), &["build"]);
    assert!(!output.status.success());
}
