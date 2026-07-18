//! PS-READINESS-E2ERUNNER Slice B: the ONLY live consumer of the 4B sidecar
//! at http://127.0.0.1:8181. Drives a REAL model: health-checks the sidecar,
//! asks the 4B to generate a small Rust function, splices that output into a
//! fixed single-file test harness, compiles + runs it via `rustc --test`, and
//! emits ONE signed off-agent artifact carrying the REAL compile exit code +
//! test exit code + a lane tag (model + host).
//!
//! This is NOT a fixture runner. A missing/unreachable sidecar or any fixture
//! fallback is a hard FAIL (wiring-enforcement R1), never a synthesized pass.
//!
//! # Wiring discipline
//!
//! producer  : `run_local_lane_e2e` (this file).
//! consumer  : `cli::bench_local_lane_e2e::run` (CLI sub-verb
//!             `elai bench local-lane-e2e`), a non-test caller in a different
//!             file -> WIRED, not dark.
//! consumes  : `SidecarRuntime::complete` against http://127.0.0.1:8181 with
//!             `chat_template_kwargs.enable_thinking = false`.
//!
//! # No `.unwrap()` / `.expect()` in non-test paths.
//! # blake3 only for any content hashing (anti-stale rule).

use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use elai_providers::runtime::{
    CompletionConfig, Message, MessageRole, RuntimeAdapter as _, RuntimeBackend,
    RuntimeFactoryConfig, RuntimeHealth, create_runtime,
};
use thiserror::Error;
use tokio::time::timeout;

/// Health-check timeout: mirrors cli/src/runtime_cmd.rs (5s).
const HEALTH_CHECK_TIMEOUT_SECS: u64 = 5;

/// Max tokens for the single completion. The target is a tiny `fn add`, so
/// 256 is generous; keeping it small avoids a runaway model burning VRAM.
const COMPLETION_MAX_TOKENS: u32 = 256;

/// Temperature for the generation: 0.0 for deterministic code emission.
const COMPLETION_TEMPERATURE: f32 = 0.0;

/// The exact prompt instruction sent to the 4B. enable_thinking=false plus
/// this instruction is why the output is code, not reasoning.
const GENERATION_PROMPT: &str =
    "Write exactly this Rust function and nothing else. No markdown fences, \
     no prose, no explanation, no comments. Just the function body:\n\n\
     pub fn add(a: i32, b: i32) -> i32";

/// The fixed test harness spliced after the extracted function. `HARNESS_ASSERT`
/// is the canonical assertion; the test binary exit code is the off-agent
/// artifact the runner reports.
const HARNESS_TEMPLATE: &str = "\
#[cfg(test)]
mod t {{
    use super::*;
    #[test]
    fn a() {{
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
        assert_eq!(add(0, 0), 0);
    }}
}}
";

/// The function signature the extracted code must contain.
const REQUIRED_FN_SIGNATURE: &str = "fn add";

/// The rustc edition used to compile the generated test harness.
const RUSTC_EDITION: &str = "2021";

/// The off-agent report: carries the REAL compile exit code + test exit code +
/// a lane tag (model + host). `functional_pass` is `compile_exit_code == 0 &&
/// test_exit_code == 0`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LocalLaneE2eReport {
    pub model: String,
    pub host: String,
    pub health_check_passed: bool,
    pub complete_latency_ms: u64,
    pub generated_chars: usize,
    pub compile_exit_code: i32,
    pub test_exit_code: i32,
    pub functional_pass: bool,
}

/// Errors raised by `run_local_lane_e2e`. Each variant maps to a distinct
/// failure mode; `SidecarUnavailable` is a hard FAIL (R1), never a green
/// beacon.
#[derive(Debug, Error)]
pub enum LocalLaneE2eError {
    /// The sidecar health check failed (unreachable or Unavailable).
    #[error("local-lane e2e: sidecar unavailable: {0}")]
    SidecarUnavailable(String),
    /// The `complete()` call failed after a successful health check.
    #[error("local-lane e2e: completion failed: {0}")]
    CompleteFailed(String),
    /// `rustc` could not be spawned (toolchain missing on the runner host).
    #[error("local-lane e2e: toolchain error: {0}")]
    Toolchain(String),
    /// The model output contained no `fn add` code block to extract.
    #[error("local-lane e2e: code extraction failed: {0}")]
    CodeExtraction(String),
}

/// Run the local-lane e2e benchmark against the 4B sidecar at `endpoint`.
///
/// This is the runner core. It drives a REAL model; no fixture fallback.
/// The production caller is `cli::bench_local_lane_e2e::run` (non-test,
/// different file) -> WIRED.
pub async fn run_local_lane_e2e(
    endpoint: &str,
    model: &str,
) -> Result<LocalLaneE2eReport, LocalLaneE2eError> {
    let host = hostname_string();

    // a. Build RuntimeFactoryConfig for the Sidecar backend, pointing at the
    //    live 4B endpoint. Mirrors cli/src/runtime_cmd.rs:342-366.
    let cfg = RuntimeFactoryConfig {
        backend: RuntimeBackend::Sidecar,
        base_url: Some(endpoint.to_string()),
        model: Some(model.to_string()),
        ..RuntimeFactoryConfig::for_backend(RuntimeBackend::Sidecar)
    };
    let adapter = create_runtime(cfg).map_err(|e| {
        LocalLaneE2eError::SidecarUnavailable(format!("create_runtime: {e}"))
    })?;

    // b. Health check with a 5s timeout. NO fallback: Unavailable or timeout
    //    -> Err(SidecarUnavailable). health_check_passed=true only on Ready.
    let health = timeout(
        Duration::from_secs(HEALTH_CHECK_TIMEOUT_SECS),
        adapter.health_check(),
    )
    .await
    .unwrap_or(RuntimeHealth::Unavailable {
        reason: format!("health probe timeout (>{HEALTH_CHECK_TIMEOUT_SECS}s)"),
    });
    match health {
        RuntimeHealth::Ready => {}
        RuntimeHealth::Degraded { reason } => {
            return Err(LocalLaneE2eError::SidecarUnavailable(
                format!("degraded: {reason}"),
            ));
        }
        RuntimeHealth::Unavailable { reason } => {
            return Err(LocalLaneE2eError::SidecarUnavailable(reason));
        }
    }

    // c. Build messages: a single user Message asking for exactly the Rust
    //    function, code only, no prose/fences/explanation.
    let messages = vec![Message {
        role: MessageRole::User,
        content: GENERATION_PROMPT.to_string(),
    }];

    // d. CompletionConfig: Default + model + chat_template_kwargs with
    //    enable_thinking=false so the 4B emits code, not reasoning. sidecar.rs
    //    forwards chat_template_kwargs into the outbound MessageRequest.
    let mut chat_kwargs = serde_json::Map::new();
    chat_kwargs.insert(
        "enable_thinking".to_string(),
        serde_json::Value::Bool(false),
    );
    let config = CompletionConfig {
        model: Some(model.to_string()),
        max_tokens: COMPLETION_MAX_TOKENS,
        temperature: COMPLETION_TEMPERATURE,
        chat_template_kwargs: Some(chat_kwargs),
        ..CompletionConfig::default()
    };

    let start = Instant::now();
    let response = adapter
        .complete(messages, config)
        .await
        .map_err(|e| LocalLaneE2eError::CompleteFailed(e.to_string()))?;
    let complete_latency_ms = start.elapsed().as_millis() as u64;

    // e. Extract the code: strip any ```rust / ``` fences and surrounding
    //    prose. If no `fn add` is present -> Err(CodeExtraction).
    let extracted = extract_code(&response.text)
        .ok_or_else(|| LocalLaneE2eError::CodeExtraction(
            format!("no `{REQUIRED_FN_SIGNATURE}` in model output ({} chars)", response.text.len()),
        ))?;
    let generated_chars = extracted.len();

    // f. Splice into the fixed test harness + write to a fresh tempdir.
    let harness = format!("{extracted}\n{HARNESS_TEMPLATE}");
    let temp_dir = unique_tempdir();
    std::fs::create_dir_all(&temp_dir).map_err(|e| LocalLaneE2eError::Toolchain(
        format!("could not create tempdir {}: {e}", temp_dir.display()),
    ))?;
    let source_path = temp_dir.join("lane_e2e_test.rs");
    let bin_path = temp_dir.join("lane_e2e_test_bin");
    std::fs::write(&source_path, &harness).map_err(|e| {
        LocalLaneE2eError::Toolchain(format!(
            "could not write {}: {e}",
            source_path.display(),
        ))
    })?;

    // g. Compile + run via `rustc --edition 2021 --test <file> -o <bin>`,
    //    then run <bin>. Capture REAL exit codes. If rustc cannot be
    //    spawned at all -> Err(Toolchain).
    let compile_exit_code = match Command::new("rustc")
        .args([
            "--edition",
            RUSTC_EDITION,
            "--test",
            source_path.to_str().unwrap_or("lane_e2e_test.rs"),
            "-o",
            bin_path.to_str().unwrap_or("lane_e2e_test_bin"),
        ])
        .output()
    {
        Ok(out) => out.status.code().unwrap_or(-1),
        Err(e) => {
            best_effort_cleanup(&temp_dir);
            return Err(LocalLaneE2eError::Toolchain(format!(
                "could not spawn rustc: {e}",
            )));
        }
    };

    let test_exit_code = if compile_exit_code == 0 {
        match Command::new(&bin_path).output() {
            Ok(out) => out.status.code().unwrap_or(-1),
            Err(e) => {
                best_effort_cleanup(&temp_dir);
                return Err(LocalLaneE2eError::Toolchain(format!(
                    "could not spawn test binary {}: {e}",
                    bin_path.display(),
                )));
            }
        }
    } else {
        // Compile failed; skip the test run. test_exit_code stays -1 so
        // functional_pass is false.
        -1
    };

    // h. functional_pass = compile_exit_code==0 && test_exit_code==0.
    let functional_pass = compile_exit_code == 0 && test_exit_code == 0;

    best_effort_cleanup(&temp_dir);

    Ok(LocalLaneE2eReport {
        model: model.to_string(),
        host,
        health_check_passed: true,
        complete_latency_ms,
        generated_chars,
        compile_exit_code,
        test_exit_code,
        functional_pass,
    })
}

/// Strip any ```rust / ``` fences and surrounding prose from `raw`, returning
/// the code block containing `fn add` if present. Handles:
///   - raw code with no fences (returned as-is if it contains `fn add`)
///   - a single fenced block (```rust\n...\n``` or ```\n...\n```)
///   - prose with a fenced block (extracts the first fenced block)
/// Returns None if no `fn add` is present in the candidate.
fn extract_code(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    // Case 1: no fences at all. If it contains `fn add`, use it directly.
    if !trimmed.contains("```") {
        if trimmed.contains(REQUIRED_FN_SIGNATURE) {
            return Some(trimmed.to_string());
        }
        return None;
    }
    // Case 2: extract the first fenced block. The fence may be ```rust or ```.
    let after_open = trimmed.split("```").nth(1)?;
    // Strip an optional language tag on the opening fence line (e.g. "rust\n").
    let body = after_open
        .strip_prefix("rust\n")
        .or_else(|| after_open.strip_prefix("rust\r\n"))
        .unwrap_or(after_open);
    // The body ends at the next ``` (already split off). Trim trailing fence.
    let code = body
        .rsplit("```")
        .last()
        .unwrap_or(body)
        .trim();
    if code.contains(REQUIRED_FN_SIGNATURE) {
        Some(code.to_string())
    } else {
        None
    }
}

/// Best-effort hostname for the lane tag. Falls back to "unknown" if the
/// hostname cannot be determined (never panics).
fn hostname_string() -> String {
    #[cfg(unix)]
    {
        if let Ok(name) = std::env::var("HOSTNAME") {
            if !name.is_empty() {
                return name;
            }
        }
        if let Ok(out) = Command::new("hostname").output() {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !s.is_empty() {
                    return s;
                }
            }
        }
    }
    "unknown".to_string()
}

/// Build a unique tempdir path under std::env::temp_dir() using a nanosecond
/// clock. Does NOT create the dir (caller creates it).
fn unique_tempdir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir().join(format!("elai-local-lane-e2e-{nanos}"))
}

/// Best-effort cleanup of the tempdir. A failure is not an error.
fn best_effort_cleanup(dir: &std::path::Path) {
    let _ = std::fs::remove_dir_all(dir);
}

// ---------------------------------------------------------------------------
// Unit tests (code-extraction helper ONLY; never hits the network)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_code_plain_no_fences() {
        let raw = "pub fn add(a: i32, b: i32) -> i32 { a + b }";
        let out = extract_code(raw).expect("plain code with fn add");
        assert!(out.contains("fn add"));
        assert!(!out.contains("```"));
    }

    #[test]
    fn extract_code_rust_fenced() {
        let raw = "Here is the function:\n```rust\npub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n```\nDone.";
        let out = extract_code(raw).expect("fenced rust block with fn add");
        assert!(out.contains("fn add"));
        assert!(!out.contains("```"));
        assert!(out.contains("a + b"));
    }

    #[test]
    fn extract_code_bare_fenced() {
        let raw = "```\npub fn add(a: i32, b: i32) -> i32 { a + b }\n```";
        let out = extract_code(raw).expect("bare fenced block with fn add");
        assert!(out.contains("fn add"));
        assert!(!out.contains("```"));
    }

    #[test]
    fn extract_code_no_fn_add_returns_none() {
        let raw = "pub fn sub(a: i32, b: i32) -> i32 { a - b }";
        assert!(extract_code(raw).is_none());
    }

    #[test]
    fn extract_code_empty_returns_none() {
        assert!(extract_code("").is_none());
    }

    #[test]
    fn extract_code_fenced_no_fn_add_returns_none() {
        let raw = "```rust\npub fn sub(a: i32, b: i32) -> i32 { a - b }\n```";
        assert!(extract_code(raw).is_none());
    }
}
