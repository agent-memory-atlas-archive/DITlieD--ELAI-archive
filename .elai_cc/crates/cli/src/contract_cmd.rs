//! C-50 — `elai contract <scaffold|audit|list>` handler.
//!
//! Drives the orchestrator's [`elai_orchestrator::contract_coding`]
//! module. Three sub-actions:
//!   - `scaffold --fn-name NAME [--output PATH]`: emit a Rust
//!     contract test scaffolding for the canonical function name.
//!     The signature is synthesized as `fn <name>() -> ()` because
//!     the C-50 surface does not yet ingest user-supplied signatures
//!     (paper §4.4 describes the Discriminator pass, not the
//!     signature-parser); operators hand-edit the emitted file
//!     before locking it.
//!   - `audit --code-path PATH --spec-path PATH`: run the heuristic
//!     Discriminator against the code body at `--code-path` using
//!     the JSON-encoded `StructuralContract` at `--spec-path`.
//!     Prints the violation list (empty = consistent).
//!   - `list`: emit a deterministic sample of the kernel anchor
//!     names so operators can grep what the scaffolder produces
//!     without invoking it.
//!
//! Paper grounding caveats (arXiv:2604.13100, Lin, Zhao, Shi 2026):
//!   - This CLI surface ships the HEURISTIC structural Discriminator
//!     only. The RL-trained Generator -> Discriminator pipeline
//!     (Algorithm 1) is RUNPOD-PENDING.
//!   - Citation discipline: the headline "47%" is the Roguelike
//!     subset of Greenfield-5 (Table 1, Section 5.2) and TIES Trae.
//!     The differentiator is qualitative structural integrity per
//!     Section 5.2; failures are "strictly confined to Local Logic
//!     Errors, not architectural collapse". Do NOT cite the headline
//!     without the subset + structural-integrity qualifier.

use std::fs;
use std::path::PathBuf;

use elai_orchestrator::contract_coding::{
    audit_against_contract, scaffold_contract_test_from_signature, StructuralContract,
    TypeSignature,
};
use serde::Serialize;

use crate::args::{CliOutputFormat, ContractAction};

/// Top-level dispatcher invoked by main.rs.
pub(crate) fn run(action: ContractAction, output_format: CliOutputFormat) -> Result<(), String> {
    match action {
        ContractAction::Scaffold { fn_name, output } => {
            run_scaffold(&fn_name, output.as_deref(), output_format)
        }
        ContractAction::Audit { code_path, spec_path } => {
            run_audit(&PathBuf::from(code_path), &PathBuf::from(spec_path), output_format)
        }
        ContractAction::List => run_list(output_format),
    }
}

fn run_scaffold(
    fn_name: &str,
    output: Option<&str>,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    // Synthesise a default signature; the operator hand-edits before
    // locking the test. See module header for the signature-parser
    // RUNPOD-PENDING note.
    let synth_signature = format!("fn {fn_name}() -> ()");
    let sig = TypeSignature::new(fn_name, synth_signature, "rust");
    let body = scaffold_contract_test_from_signature(fn_name, &sig)
        .map_err(|e| format!("contract scaffold: {e}"))?;
    match output {
        Some(path) => {
            fs::write(path, &body)
                .map_err(|e| format!("contract scaffold write {path}: {e}"))?;
            if matches!(output_format, CliOutputFormat::Text) {
                println!("wrote {} bytes to {}", body.len(), path);
            } else {
                println!(
                    "{{\"action\":\"scaffold\",\"fn_name\":{:?},\"output\":{:?},\"bytes\":{}}}",
                    fn_name,
                    path,
                    body.len()
                );
            }
        }
        None => {
            // Stdout — text mode prints the body; JSON mode wraps it.
            match output_format {
                CliOutputFormat::Json => {
                    #[derive(Serialize)]
                    struct Out<'a> {
                        action: &'a str,
                        fn_name: &'a str,
                        body: &'a str,
                    }
                    let row = Out {
                        action: "scaffold",
                        fn_name,
                        body: &body,
                    };
                    let json = serde_json::to_string_pretty(&row)
                        .map_err(|e| format!("scaffold json encode: {e}"))?;
                    println!("{json}");
                }
                _ => {
                    println!("{body}");
                }
            }
        }
    }
    Ok(())
}

fn run_audit(
    code_path: &PathBuf,
    spec_path: &PathBuf,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let code = fs::read_to_string(code_path)
        .map_err(|e| format!("read code-path {}: {e}", code_path.display()))?;
    let spec_text = fs::read_to_string(spec_path)
        .map_err(|e| format!("read spec-path {}: {e}", spec_path.display()))?;
    let spec: StructuralContract = serde_json::from_str(&spec_text)
        .map_err(|e| format!("decode spec-path {}: {e}", spec_path.display()))?;
    spec.validate()
        .map_err(|e| format!("spec validation failed: {e}"))?;

    let verdict = audit_against_contract(&code, &spec);
    match output_format {
        CliOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&verdict)
                .map_err(|e| format!("audit json encode: {e}"))?;
            println!("{json}");
        }
        _ => {
            if verdict.is_consistent {
                println!("audit: is_consistent=true ({} violations)", verdict.violations.len());
            } else {
                println!(
                    "audit: is_consistent=false ({} violations)",
                    verdict.violations.len()
                );
                for v in &verdict.violations {
                    println!("  [{:?}] {}: {}", v.kind, v.element, v.message);
                }
            }
        }
    }
    Ok(())
}

fn run_list(output_format: CliOutputFormat) -> Result<(), String> {
    // Deterministic sample of the kernel anchors the scaffolder
    // emits for a hypothetical `add` target.
    let sample = ListSample {
        target_fn: "add".into(),
        namespace_section: "add_module".into(),
        constraint_name: "add_signature_pinned".into(),
        language_hint: "rust".into(),
    };
    match output_format {
        CliOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&sample)
                .map_err(|e| format!("list json encode: {e}"))?;
            println!("{json}");
        }
        _ => {
            println!("contract list (sample anchors for target_fn=\"add\"):");
            println!("  N section : {}", sample.namespace_section);
            println!("  Delta name: {}", sample.constraint_name);
            println!("  lang hint : {}", sample.language_hint);
        }
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct ListSample {
    target_fn: String,
    namespace_section: String,
    constraint_name: String,
    language_hint: String,
}
