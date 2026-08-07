//! J-82 deterministic tool-output token emission benchmark.
use std::path::PathBuf;

fn builtin_fixture() -> String {
    let mut object = serde_json::Map::new();
    for i in 0..96 {
        object.insert(format!("record_{i:03}"), serde_json::json!({
            "id": i, "status": "ok", "payload": format!("deterministic payload {i} {}", "x".repeat(48))
        }));
    }
    let json = serde_json::Value::Object(object).to_string();
    format!("{json}\n--- tool log ---\n{}\n--- prose and code ---\nThe tool completed a deterministic inspection.\n```rust\nfn stable(value: u64) -> u64 {{ value.wrapping_mul(31) }}\n```\n", (0..180).map(|i| format!("2026-01-01T00:00:{i:02}Z INFO worker step={i} result=ok")).collect::<Vec<_>>().join("\n"))
}

fn load_corpus(path: PathBuf) -> anyhow::Result<String> {
    let raw = std::fs::read_to_string(&path)?;
    let mut outputs = Vec::new();
    for line in raw.lines().filter(|line| !line.trim().is_empty()) {
        match serde_json::from_str::<serde_json::Value>(line) {
            Ok(value) => outputs.push(value.get("output").and_then(|v| v.as_str()).unwrap_or(line).to_string()),
            Err(_) => outputs.push(line.to_string()),
        }
    }
    Ok(if outputs.is_empty() { raw } else { outputs.join("\n") })
}

pub(crate) fn run(corpus: Option<PathBuf>, seed: u64, json: bool, out: Option<PathBuf>) -> anyhow::Result<()> {
    let raw = match corpus { Some(path) => load_corpus(path)?, None => builtin_fixture() };
    let report = elai_orchestrator::token_emit::build_tool_output_bench_report(&raw, seed);
    let report_json = serde_json::to_string_pretty(&report)?;
    if let Some(path) = out { std::fs::write(path, &report_json)?; }
    if json { println!("{report_json}"); } else {
        println!("tool-output: seed={} baseline={} format={} format_delta={} protected={} protected_delta={}", report.seed, report.baseline_tokens, report.format_tokens, report.format_delta, report.protected_span_tokens, report.protected_span_delta);
        println!("  measure_first_guard={}", report.measure_first_guard);
        if let Some(tokens) = report.log_dict_tokens { println!("  log_dict_tokens={tokens}"); }
    }
    Ok(())
}
