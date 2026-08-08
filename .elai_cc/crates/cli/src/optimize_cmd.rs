//! E-8 — `elai optimize <template|status>` handler.
//!
//! Spawns the Python `elai_optimizer` service (GEPA primary per
//! arXiv:2507.19457; DSPy MIPROv2 / OPRO supporting), tails its
//! progress + emits the per-run report. Live LLM calls are
//! RUNPOD-PENDING -- the CLI surface compiles + dispatches; the
//! Python side reads the spec + emits an OptimizedTemplate JSON to
//! disk; the orchestrator admit path consumes the JSON.
//!
//! Reflection-LM refusal: arXiv:2405.10276 (Zhang et al.) -- ELAI
//! local Qwen2.5-Coder-7B (and other <30B local-class models) are
//! INEFFECTIVE as the OPRO optimizer. The args parser hard-refuses
//! before this handler runs.
//!
//! Per H-7 / CLAUDE.md hard rules: no `.unwrap()` / `.expect()`
//! outside test code.

use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::args::{CliOutputFormat, OptimizeAction};

use elai_orchestrator::baby_step_spec_lpo::{BabyStepSpec, BlockKind, SpecBlock};
use elai_orchestrator::optimizer_admit::{
    admit_optimized_template, AdmitError, CrossDomainCanaryResult, OptimizedTemplate,
};
use elai_orchestrator::reasoning_template::Blake3Hash;

/// Top-level dispatcher invoked by `main.rs`.
pub(crate) fn run(
    action: OptimizeAction,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    match action {
        OptimizeAction::Template {
            target,
            train,
            val,
            reflection_model,
            budget_tokens,
            include_cross_domain,
        } => run_template_dispatch(
            &target,
            &train,
            &val,
            &reflection_model,
            budget_tokens,
            include_cross_domain,
            output_format,
        ),
        OptimizeAction::Status => run_status(output_format),
    }
}

fn project_root_guess() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn python_optimizer_dir(project_root: &PathBuf) -> PathBuf {
    project_root.join(".elai_cc").join("python").join("elai_optimizer")
}

#[allow(clippy::too_many_arguments)]
fn run_template_dispatch(
    target: &str,
    train: &str,
    val: &str,
    reflection_model: &str,
    budget_tokens: u64,
    include_cross_domain: bool,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let project_root = project_root_guess();
    let py_dir = python_optimizer_dir(&project_root);
    // RUNPOD-PENDING: live cloud Sonnet + local Qwen calls happen
    // inside the Python optimizer. If the python directory is absent
    // (developer is on a fresh checkout without the optimizer
    // installed) we report a clean error rather than panicking.
    let py_present = py_dir.exists();
    if !py_present {
        let report = OptimizeRunReport {
            action: "template",
            target,
            train,
            val,
            reflection_model,
            budget_tokens,
            include_cross_domain,
            python_dir: py_dir.to_string_lossy().to_string(),
            python_present: py_present,
            runpod_pending: "live_gepa_reflection_loop",
            admitted: false,
            admission_error: Some("python optimizer dir absent".to_string()),
            content_blake3: String::new(),
            optimization_run_id: String::new(),
        };
        match output_format {
            CliOutputFormat::Json => print_json(&report)?,
            _ => print_template_text(&report),
        }
        return Err(format!(
            "python optimizer dir absent at {}; install via `pip install -e .` inside that dir first",
            py_dir.display()
        ));
    }
    // Unique per-run output dir keyed by SystemTime nanos (no uuid
    // crate needed). The Python side writes
    // <out_dir>/<optimization_run_id>.json.
    let run_dir_name = format!(
        "run-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let out_dir = project_root
        .join(".elai_cc")
        .join("optimization")
        .join("outputs")
        .join(&run_dir_name);
    // Attempt to invoke `python -m elai_optimizer.optimizer ...`. The
    // Python side parses --train/--val/--target itself + writes the
    // OptimizedTemplate JSON envelope to --out-dir. We pass --dry-run
    // when no --include-cross-domain is set so a smoke run does not
    // spend the cloud budget.
    let py_module = "elai_optimizer.optimizer";
    let mut cmd = Command::new("python");
    cmd.current_dir(&project_root)
        .arg("-m")
        .arg(py_module)
        .arg("--target")
        .arg(target)
        .arg("--train")
        .arg(train)
        .arg("--val")
        .arg(val)
        .arg("--reflection-model")
        .arg(reflection_model)
        .arg("--budget-tokens")
        .arg(budget_tokens.to_string())
        .arg("--out-dir")
        .arg(&out_dir);
    if include_cross_domain {
        cmd.arg("--include-cross-domain");
    } else {
        cmd.arg("--dry-run");
    }
    // Spawn + wait; surface exit code as a String error.
    let status = cmd
        .status()
        .map_err(|e| format!("python optimizer spawn: {e}"))?;
    if !status.success() {
        return Err(format!(
            "python optimizer exited with status {:?}",
            status.code()
        ));
    }
    // Read the OptimizedTemplate envelope JSON from the out-dir and
    // run the admit path. The Python side writes exactly one file
    // (<optimization_run_id>.json); we read the first .json file in
    // the out-dir.
    let envelope_path = match find_envelope_json(&out_dir) {
        Some(p) => p,
        None => {
            let report = OptimizeRunReport {
                action: "template",
                target,
                train,
                val,
                reflection_model,
                budget_tokens,
                include_cross_domain,
                python_dir: py_dir.to_string_lossy().to_string(),
                python_present: py_present,
                runpod_pending: "live_gepa_reflection_loop",
                admitted: false,
                admission_error: Some(format!(
                    "envelope JSON not found in {}",
                    out_dir.display()
                )),
                content_blake3: String::new(),
                optimization_run_id: String::new(),
            };
            match output_format {
                CliOutputFormat::Json => print_json(&report)?,
                _ => print_template_text(&report),
            }
            return Err(format!(
                "envelope JSON not found in {}",
                out_dir.display()
            ));
        }
    };
    let envelope = read_optimized_envelope(&envelope_path)
        .map_err(|e| format!("envelope read: {e}"))?;
    let content_blake3_hex = envelope.content_blake3.as_hex();
    let optimization_run_id = envelope.optimization_run_id.clone();
    // Build a BabyStepSpec with Task + RolePromptBody +
    // DeterministicCheck so the LPO guard can verify the optimizer
    // only rewrote editable blocks (RolePromptBody) and did not touch
    // the frozen DeterministicCheck.
    let spec = BabyStepSpec::from_blocks(vec![
        SpecBlock {
            kind: BlockKind::Task,
            body: target.to_string(),
        },
        SpecBlock {
            kind: BlockKind::RolePromptBody,
            body: "original role prompt body".to_string(),
        },
        SpecBlock {
            kind: BlockKind::DeterministicCheck,
            body: "shell command".to_string(),
        },
    ]);
    let admit_result = admit_optimized_template(&envelope, &spec);
    let (admitted, admission_error) = match &admit_result {
        Ok(a) => (a.admitted, None),
        Err(e) => (false, Some(format!("{e}"))),
    };
    let report = OptimizeRunReport {
        action: "template",
        target,
        train,
        val,
        reflection_model,
        budget_tokens,
        include_cross_domain,
        python_dir: py_dir.to_string_lossy().to_string(),
        python_present: py_present,
        runpod_pending: "live_gepa_reflection_loop",
        admitted,
        admission_error: admission_error.clone(),
        content_blake3: content_blake3_hex,
        optimization_run_id,
    };
    match output_format {
        CliOutputFormat::Json => print_json(&report)?,
        _ => print_template_text(&report),
    }
    if let Err(e) = admit_result {
        return Err(format!("{e}"));
    }
    Ok(())
}

fn run_status(output_format: CliOutputFormat) -> Result<(), String> {
    let project_root = project_root_guess();
    let py_dir = python_optimizer_dir(&project_root);
    let py_present = py_dir.exists();
    let report = OptimizeStatusReport {
        action: "status",
        python_dir: py_dir.to_string_lossy().to_string(),
        python_present: py_present,
        gepa_pypi_version_pinned: "0.1.x",
        cloud_reflection_only: "rejects qwen / local / 3B / 7B per arXiv:2405.10276",
        runpod_pending: "live_gepa_reflection_loop",
    };
    match output_format {
        CliOutputFormat::Json => print_json(&report)?,
        _ => print_status_text(&report),
    }
    Ok(())
}

fn print_json<T: Serialize>(report: &T) -> Result<(), String> {
    let json = serde_json::to_string_pretty(report)
        .map_err(|e| format!("optimize json encode: {e}"))?;
    println!("{json}");
    Ok(())
}

fn print_template_text(r: &OptimizeRunReport<'_>) {
    println!("optimize template ({})", r.target);
    println!("  train               : {}", r.train);
    println!("  val                 : {}", r.val);
    println!("  reflection_model    : {}", r.reflection_model);
    println!("  budget_tokens       : {}", r.budget_tokens);
    println!("  include_cross_domain: {}", r.include_cross_domain);
    println!("  python_dir          : {}", r.python_dir);
    println!("  python_present      : {}", r.python_present);
    println!("  runpod_pending      : {}", r.runpod_pending);
    println!("  admitted            : {}", r.admitted);
    if let Some(e) = &r.admission_error {
        println!("  admission_error     : {}", e);
    }
    println!("  content_blake3      : {}", r.content_blake3);
    println!("  optimization_run_id : {}", r.optimization_run_id);
}

fn print_status_text(s: &OptimizeStatusReport<'_>) {
    println!("optimize status (E-8 lean ship)");
    println!("  python_dir              : {}", s.python_dir);
    println!("  python_present          : {}", s.python_present);
    println!("  gepa_pypi_version_pinned: {}", s.gepa_pypi_version_pinned);
    println!("  cloud_reflection_only   : {}", s.cloud_reflection_only);
    println!("  runpod_pending          : {}", s.runpod_pending);
}

/// Find the first `.json` file in `out_dir`. The Python optimizer
/// writes exactly one envelope file (`<optimization_run_id>.json`).
fn find_envelope_json(out_dir: &PathBuf) -> Option<PathBuf> {
    let entries = std::fs::read_dir(out_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            return Some(path);
        }
    }
    None
}

/// Decode a 64-char lowercase hex string into a `Blake3Hash`. The
/// Python optimizer writes `content_blake3` / `parent_template_blake3`
/// as hex strings, but `Blake3Hash` uses `serde(transparent)` over
/// `[u8; 32]` (array-of-bytes wire format), so we cannot deserialize
/// the envelope directly into `OptimizedTemplate` — we parse the JSON
/// as `serde_json::Value` and convert the hex fields manually.
fn hex_to_blake3(hex: &str) -> Result<Blake3Hash, String> {
    if hex.len() != 64 {
        return Err(format!(
            "expected 64-char hex string, got {} chars",
            hex.len()
        ));
    }
    let mut bytes = [0u8; 32];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        if chunk.len() != 2 {
            return Err("odd-length hex string".to_string());
        }
        let hi = hex_nibble_value(chunk[0])?;
        let lo = hex_nibble_value(chunk[1])?;
        bytes[i] = (hi << 4) | lo;
    }
    Ok(Blake3Hash(bytes))
}

fn hex_nibble_value(c: u8) -> Result<u8, String> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(format!("invalid hex char: {c}")),
    }
}

/// Reverse of `BlockKind::as_str` — maps the snake-case wire tag
/// back to the enum variant. The Python optimizer writes
/// `target_blocks` as an array of these tags.
fn block_kind_from_str(s: &str) -> Option<BlockKind> {
    match s {
        "task" => Some(BlockKind::Task),
        "grounding" => Some(BlockKind::Grounding),
        "pre_gate" => Some(BlockKind::PreGate),
        "tdd" => Some(BlockKind::Tdd),
        "role_prompt_body" => Some(BlockKind::RolePromptBody),
        "deterministic_check" => Some(BlockKind::DeterministicCheck),
        "acceptance" => Some(BlockKind::Acceptance),
        "integration" => Some(BlockKind::Integration),
        "exit_criteria" => Some(BlockKind::ExitCriteria),
        _ => None,
    }
}

/// Read the OptimizedTemplate envelope JSON from `path` and convert
/// it into the Rust `OptimizedTemplate` struct. The Python side
/// writes `content_blake3` / `parent_template_blake3` as hex strings;
/// we convert them to `Blake3Hash` manually (see `hex_to_blake3`).
fn read_optimized_envelope(path: &PathBuf) -> Result<OptimizedTemplate, String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| format!("read {}: {e}", path.display()))?;
    let v: serde_json::Value =
        serde_json::from_str(&raw).map_err(|e| format!("parse json: {e}"))?;
    let content = v["content"]
        .as_str()
        .ok_or_else(|| "missing content".to_string())?
        .to_string();
    let content_blake3_hex = v["content_blake3"]
        .as_str()
        .ok_or_else(|| "missing content_blake3".to_string())?;
    let content_blake3 = hex_to_blake3(content_blake3_hex)?;
    let parent_hex = v["parent_template_blake3"]
        .as_str()
        .ok_or_else(|| "missing parent_template_blake3".to_string())?;
    let parent_template_blake3 = hex_to_blake3(parent_hex)?;
    let optimizer_version = v["optimizer_version"]
        .as_str()
        .ok_or_else(|| "missing optimizer_version".to_string())?
        .to_string();
    let optimization_run_id = v["optimization_run_id"]
        .as_str()
        .ok_or_else(|| "missing optimization_run_id".to_string())?
        .to_string();
    let val_lift_pct = v["val_lift_pct"]
        .as_f64()
        .ok_or_else(|| "missing val_lift_pct".to_string())?;
    let held_out_lift_pct = v["held_out_lift_pct"]
        .as_f64()
        .ok_or_else(|| "missing held_out_lift_pct".to_string())?;
    let mut cross_domain_canary_results = Vec::new();
    if let Some(arr) = v["cross_domain_canary_results"].as_array() {
        for item in arr {
            let family = item["family"]
                .as_str()
                .ok_or_else(|| "missing canary family".to_string())?
                .to_string();
            let delta_pct = item["delta_pct"]
                .as_f64()
                .ok_or_else(|| "missing canary delta_pct".to_string())?;
            cross_domain_canary_results.push(CrossDomainCanaryResult {
                family,
                delta_pct,
            });
        }
    }
    let mut target_blocks = Vec::new();
    if let Some(arr) = v["target_blocks"].as_array() {
        for item in arr {
            let tag = item
                .as_str()
                .ok_or_else(|| "target_blocks entries must be strings".to_string())?;
            let kind = block_kind_from_str(tag)
                .ok_or_else(|| format!("unknown block kind: {tag}"))?;
            target_blocks.push(kind);
        }
    }
    Ok(OptimizedTemplate {
        content,
        content_blake3,
        parent_template_blake3,
        optimizer_version,
        optimization_run_id,
        val_lift_pct,
        held_out_lift_pct,
        cross_domain_canary_results,
        target_blocks,
    })
}

#[derive(Serialize)]
struct OptimizeRunReport<'a> {
    action: &'a str,
    target: &'a str,
    train: &'a str,
    val: &'a str,
    reflection_model: &'a str,
    budget_tokens: u64,
    include_cross_domain: bool,
    python_dir: String,
    python_present: bool,
    runpod_pending: &'a str,
    admitted: bool,
    admission_error: Option<String>,
    content_blake3: String,
    optimization_run_id: String,
}

#[derive(Serialize)]
struct OptimizeStatusReport<'a> {
    action: &'a str,
    python_dir: String,
    python_present: bool,
    gepa_pypi_version_pinned: &'a str,
    cloud_reflection_only: &'a str,
    runpod_pending: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use elai_orchestrator::optimizer_admit::compute_optimized_content_hash;
    use std::io::Write;

    /// Build a valid envelope JSON string (matching the Python
    /// optimizer's `_write_envelope` format) for testing.
    fn envelope_json(content: &str, val_lift: f64, held_out: f64) -> String {
        let hash = compute_optimized_content_hash(content);
        let parent = compute_optimized_content_hash("parent body");
        format!(
            r#"{{"content":{content_repr},"content_blake3":"{content_hex}","parent_template_blake3":"{parent_hex}","optimizer_version":"gepa-0.1.1+elai-e8","optimization_run_id":"run-test-1","val_lift_pct":{val_lift},"held_out_lift_pct":{held_out},"cross_domain_canary_results":[{{"family":"math","delta_pct":1.5}}],"target_blocks":["role_prompt_body"]}}"#,
            content_repr = serde_json::to_string(content).unwrap_or_else(|_| "\"\"".into()),
            content_hex = hash.as_hex(),
            parent_hex = parent.as_hex(),
            val_lift = val_lift,
            held_out = held_out,
        )
    }

    fn write_envelope_to_temp(content: &str, val_lift: f64, held_out: f64) -> PathBuf {
        let mut tmp = std::env::temp_dir();
        tmp.push(format!(
            "elai_optimize_test_{}.json",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let json = envelope_json(content, val_lift, held_out);
        let mut f = std::fs::File::create(&tmp).expect("create temp");
        f.write_all(json.as_bytes()).expect("write temp");
        tmp
    }

    /// Test 1: a clean envelope with valid hash + adequate lift
    /// admits successfully.
    #[test]
    fn admit_clean_envelope_ok() {
        let content = "improved role prompt body";
        let path = write_envelope_to_temp(content, 8.0, 7.0);
        let envelope =
            read_optimized_envelope(&path).expect("envelope parse");
        let spec = BabyStepSpec::from_blocks(vec![
            SpecBlock {
                kind: BlockKind::Task,
                body: "executor".into(),
            },
            SpecBlock {
                kind: BlockKind::RolePromptBody,
                body: "original".into(),
            },
            SpecBlock {
                kind: BlockKind::DeterministicCheck,
                body: "shell command".into(),
            },
        ]);
        let result = admit_optimized_template(&envelope, &spec);
        assert!(result.is_ok(), "admit should succeed: {:?}", result);
        let admission = result.expect("admit ok");
        assert!(admission.admitted);
        assert_eq!(
            admission.content_blake3,
            compute_optimized_content_hash(content)
        );
        let _ = std::fs::remove_file(&path);
    }

    /// Test 2: an envelope with a hash mismatch (tampered content)
    /// is rejected by the admit path.
    #[test]
    fn admit_hash_mismatch_rejected() {
        let content = "improved role prompt body";
        let path = write_envelope_to_temp(content, 8.0, 7.0);
        // Tamper: rewrite the file with different content but the
        // same (now-stale) hash.
        let original = std::fs::read_to_string(&path).expect("read");
        let tampered = original.replace(
            "\"improved role prompt body\"",
            "\"tampered content\"",
        );
        std::fs::write(&path, &tampered).expect("write tampered");
        let envelope =
            read_optimized_envelope(&path).expect("envelope parse");
        let spec = BabyStepSpec::from_blocks(vec![
            SpecBlock {
                kind: BlockKind::Task,
                body: "executor".into(),
            },
            SpecBlock {
                kind: BlockKind::RolePromptBody,
                body: "original".into(),
            },
            SpecBlock {
                kind: BlockKind::DeterministicCheck,
                body: "shell command".into(),
            },
        ]);
        let result = admit_optimized_template(&envelope, &spec);
        assert!(matches!(result, Err(AdmitError::HashMismatch { .. })));
        let _ = std::fs::remove_file(&path);
    }

    /// Test 3: an envelope with val_lift below the minimum (5.0) is
    /// rejected.
    #[test]
    fn admit_low_val_lift_rejected() {
        let content = "improved role prompt body";
        let path = write_envelope_to_temp(content, 3.0, 2.5);
        let envelope =
            read_optimized_envelope(&path).expect("envelope parse");
        let spec = BabyStepSpec::from_blocks(vec![
            SpecBlock {
                kind: BlockKind::Task,
                body: "executor".into(),
            },
            SpecBlock {
                kind: BlockKind::RolePromptBody,
                body: "original".into(),
            },
            SpecBlock {
                kind: BlockKind::DeterministicCheck,
                body: "shell command".into(),
            },
        ]);
        let result = admit_optimized_template(&envelope, &spec);
        assert!(matches!(
            result,
            Err(AdmitError::ValLiftBelowMinimum { .. })
        ));
        let _ = std::fs::remove_file(&path);
    }

    /// Test 4: hex_to_blake3 round-trips a known blake3 hash.
    #[test]
    fn hex_to_blake3_roundtrip() {
        let content = "test content for hex roundtrip";
        let hash = compute_optimized_content_hash(content);
        let hex = hash.as_hex();
        let decoded = hex_to_blake3(&hex).expect("hex decode");
        assert_eq!(decoded, hash);
    }

    /// Test 5: hex_to_blake3 rejects invalid input lengths.
    #[test]
    fn hex_to_blake3_rejects_invalid_length() {
        assert!(hex_to_blake3("abc").is_err());
        assert!(hex_to_blake3("").is_err());
        // 63 chars (one short)
        let short = "0".repeat(63);
        assert!(hex_to_blake3(&short).is_err());
        // 64 chars but with invalid hex char
        let bad = "g".repeat(64);
        assert!(hex_to_blake3(&bad).is_err());
    }
}
