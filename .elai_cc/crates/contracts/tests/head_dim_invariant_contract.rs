//! E-31 — Head-dim invariant contract test.
//!
//! Locks the Qwen2.5-Coder model family attention head dimension at 128. The
//! sidecar (python/elai_inference/server.py) and SDPA / FA2 kernel paths assume
//! head_dim == 128; an upstream weight family with a different head_dim (e.g.
//! 64, 96, 256) silently degrades attention performance OR triggers fallback
//! code paths that have never been measured under load.
//!
//! Qwen2 config.json does NOT carry a `head_dim` field directly — it is computed
//! as `hidden_size / num_attention_heads` per Hugging Face transformers
//! Qwen2Attention. This test computes it and asserts == 128 against the
//! configured-on-disk model weights.
//!
//! CI-resilient: when model weights are not staged (the common CI / sandbox
//! case, since .elai_cc/models/ is gitignored), the test SKIPs with a clear
//! note rather than failing — the invariant is locked when the artefact is
//! present; absence of the artefact is not a contract violation.
//!
//! Producer (locked): .elai_cc/models/Qwen2.5-Coder-7B-Instruct-abliterated/config.json
//!                    .elai_cc/models/Qwen2.5-Coder-3B-Instruct-abliterated/config.json
//! Consumer (locked by): sidecar attention kernel selection (SDPA / FA2 / xformers).
//! Companion deferred to E-31.B: runtime assert in python/elai_inference/server.py
//! at model load, refusing to boot if head_dim != 128.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const EXPECTED_HEAD_DIM: u32 = 128;

/// Canonical model paths the sidecar may serve. Mirrors `CLAUDE.md` model
/// location entries; if the list grows, add here AND in CLAUDE.md so the
/// invariant covers every shippable model variant.
fn canonical_model_paths() -> &'static [&'static str] {
    static PATHS: OnceLock<Vec<&'static str>> = OnceLock::new();
    PATHS
        .get_or_init(|| {
            vec![
                "models/Qwen2.5-Coder-7B-Instruct-abliterated",
                "models/Qwen2.5-Coder-3B-Instruct-abliterated",
                "models/Qwen2.5-Coder-7B-Instruct",
                "models/Qwen2.5-Coder-3B-Instruct",
            ]
        })
        .as_slice()
}

/// Resolve the `.elai_cc/` workspace root from CARGO_MANIFEST_DIR. The
/// contracts crate sits at `.elai_cc/crates/contracts/`, so the workspace
/// root is two parents up from the manifest dir.
fn workspace_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| manifest.to_path_buf())
}

/// Returns `(hidden_size, num_attention_heads)` parsed from a model
/// `config.json`. Both fields are required by the Qwen2 HF spec and a
/// missing field is a real schema break (return `None`).
fn parse_qwen2_dims(config_path: &Path) -> Option<(u64, u64)> {
    let raw = std::fs::read_to_string(config_path).ok()?;
    // We deliberately avoid pulling in `serde_json` as a new dev-dependency
    // for one parse — string-slice a tiny extractor. The Qwen2 config.json
    // is canonical HF JSON with no nested structures for these fields.
    let hidden = extract_u64_field(&raw, "hidden_size")?;
    let heads = extract_u64_field(&raw, "num_attention_heads")?;
    Some((hidden, heads))
}

/// Minimal extractor: find `"field": <number>`. Sufficient for canonical HF
/// config.json which never wraps integer values in strings. Returns None if
/// not found or not a base-10 integer.
fn extract_u64_field(json: &str, field: &str) -> Option<u64> {
    let needle = format!("\"{field}\"");
    let after_key = json.find(&needle)? + needle.len();
    let after_colon = json[after_key..].find(':')? + after_key + 1;
    let slice = &json[after_colon..];
    let value: String = slice
        .chars()
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if value.is_empty() {
        return None;
    }
    value.parse::<u64>().ok()
}

#[test]
fn head_dim_invariant_locks_at_128_for_configured_models() {
    let workspace = workspace_root();
    let mut checked = 0_usize;
    let mut violations: Vec<String> = Vec::new();

    for rel in canonical_model_paths() {
        let config_path = workspace.join(rel).join("config.json");
        if !config_path.exists() {
            // CI / sandbox case — weights not present. Skip silently per the
            // contract's "absence is not a violation" rule.
            continue;
        }
        let Some((hidden, heads)) = parse_qwen2_dims(&config_path) else {
            violations.push(format!(
                "{}: config.json present but missing hidden_size or num_attention_heads",
                config_path.display()
            ));
            continue;
        };
        if heads == 0 {
            violations.push(format!(
                "{}: num_attention_heads == 0 (malformed config)",
                config_path.display()
            ));
            continue;
        }
        let head_dim = (hidden / heads) as u32;
        if head_dim != EXPECTED_HEAD_DIM {
            violations.push(format!(
                "{}: head_dim = {} (expected {}); hidden_size={} num_attention_heads={}",
                config_path.display(),
                head_dim,
                EXPECTED_HEAD_DIM,
                hidden,
                heads,
            ));
        }
        checked += 1;
    }

    if checked == 0 {
        // No weights staged anywhere — print a hint so a dev running locally
        // notices that the contract didn't actually exercise anything.
        eprintln!(
            "[head_dim_invariant_contract] no model config.json found under .elai_cc/models/; \
             skipping. The contract is dormant without staged weights — this is expected in CI."
        );
        return;
    }

    assert!(
        violations.is_empty(),
        "head_dim invariant violated for {} model(s):\n  {}",
        violations.len(),
        violations.join("\n  ")
    );
}

#[test]
fn parse_qwen2_dims_handles_real_qwen2_config_shape() {
    // Inline fixture matching the canonical Qwen2 7B abliterated config.json.
    // This test exercises the parser against a known-good shape so a CI run
    // without staged weights still verifies parser correctness.
    let json = r#"{
        "architectures": ["Qwen2ForCausalLM"],
        "hidden_size": 3584,
        "num_attention_heads": 28,
        "num_hidden_layers": 28,
        "max_position_embeddings": 32768
    }"#;
    let (hidden, heads) = extract_u64_field(json, "hidden_size")
        .zip(extract_u64_field(json, "num_attention_heads"))
        .expect("canonical config fixture must yield both dims");
    assert_eq!(hidden, 3584);
    assert_eq!(heads, 28);
    assert_eq!((hidden / heads) as u32, EXPECTED_HEAD_DIM);
}

#[test]
fn parse_qwen2_dims_returns_none_on_malformed_input() {
    assert!(extract_u64_field("{}", "hidden_size").is_none());
    assert!(extract_u64_field("{\"hidden_size\": \"not-a-number\"}", "hidden_size").is_none());
    assert!(extract_u64_field("not json at all", "hidden_size").is_none());
}
