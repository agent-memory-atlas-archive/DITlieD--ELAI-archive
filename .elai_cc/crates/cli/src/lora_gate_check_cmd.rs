//! E-47 -- `elai lora-gate-check [--config PATH] [--json]` dispatcher.
//!
//! Reads the `[finetune.lora]` block from `.elai_cc/elai_cc.toml`
//! (or the path supplied via `--config`), builds a
//! [`LoraShipGateConfig`], evaluates the triple-AND ship-gate, and
//! prints the outcome. When `ELAI_LORA_SHIP_GATE_ENABLE=1`, also
//! emits a `lora_ship_gate_event` SSB row via the orchestrator's
//! single-writer SSB handle.
//!
//! Producer-side LoRA training stack (Unsloth + QLoRA-NF4 + PiSSA +
//! DoRA on RTX 3090) is RUNPOD-PENDING; this dispatcher only
//! evaluates the GATE LAYER and does not load any adapter.

use std::fs;
use std::path::{Path, PathBuf};

use elai_orchestrator::lora_ship_gate::{
    check_lora_ship_gate, LoraShipGateConfig, LoraShipGateOutcome,
    LORA_SHIP_GATE_DEFERRAL_TOKEN,
};

const DEFAULT_CONFIG_PATH: &str = ".elai_cc/elai_cc.toml";

/// Parsed view of the `[finetune.lora]` block.
#[derive(Debug, Clone)]
struct FinetuneLoraBlock {
    ship_gate: String,
    confirm_user_understanding: bool,
    enabled: bool,
}

impl Default for FinetuneLoraBlock {
    fn default() -> Self {
        Self {
            ship_gate: LORA_SHIP_GATE_DEFERRAL_TOKEN.to_owned(),
            confirm_user_understanding: false,
            enabled: false,
        }
    }
}

impl FinetuneLoraBlock {
    fn into_gate_cfg(self) -> LoraShipGateConfig {
        LoraShipGateConfig {
            ship_gate_token: self.ship_gate,
            confirm_user_understanding: self.confirm_user_understanding,
            enabled: self.enabled,
        }
    }
}

fn parse_finetune_lora(toml_text: &str) -> Result<FinetuneLoraBlock, anyhow::Error> {
    let root: toml::Value = toml::from_str(toml_text)?;
    let Some(finetune) = root.get("finetune") else {
        return Ok(FinetuneLoraBlock::default());
    };
    let Some(lora) = finetune.get("lora") else {
        return Ok(FinetuneLoraBlock::default());
    };
    let ship_gate = lora
        .get("ship_gate")
        .and_then(|v| v.as_str())
        .unwrap_or(LORA_SHIP_GATE_DEFERRAL_TOKEN)
        .to_owned();
    let confirm_user_understanding = lora
        .get("confirm_user_understanding")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let enabled = lora
        .get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    Ok(FinetuneLoraBlock {
        ship_gate,
        confirm_user_understanding,
        enabled,
    })
}

fn resolve_config_path(supplied: Option<String>) -> PathBuf {
    match supplied {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from(DEFAULT_CONFIG_PATH),
    }
}

fn load_block(path: &Path) -> Result<FinetuneLoraBlock, anyhow::Error> {
    if !path.exists() {
        // Default-deferred posture even when the toml is absent.
        return Ok(FinetuneLoraBlock::default());
    }
    let text = fs::read_to_string(path)?;
    parse_finetune_lora(&text)
}

/// Synchronous entrypoint. Mirrors `archive_verify::run` / `ppm_health::run`
/// owning their own current-thread tokio runtime so the parent `main()`
/// stays sync.
pub fn run(config: Option<String>, json: bool) -> Result<(), anyhow::Error> {
    let path = resolve_config_path(config);
    let block = load_block(&path)?;
    let cfg = block.into_gate_cfg();
    // Producer-side LoRA training is RUNPOD-PENDING; calibration is
    // never available at the GATE LAYER ship. Pass None so the gate
    // surfaces CalibrationMissing if the operator overrode the
    // deferral token without supplying baseline data.
    let outcome = check_lora_ship_gate(&cfg, None);

    if json {
        let payload = serde_json::json!({
            "config_path": path.display().to_string(),
            "enabled": cfg.enabled,
            "ship_gate_token": cfg.ship_gate_token,
            "confirm_user_understanding": cfg.confirm_user_understanding,
            "outcome": outcome.as_str(),
        });
        println!("{}", payload);
    } else {
        println!("lora-gate-check");
        println!("  config_path                = {}", path.display());
        println!("  enabled                    = {}", cfg.enabled);
        println!("  ship_gate_token            = {}", cfg.ship_gate_token);
        println!(
            "  confirm_user_understanding = {}",
            cfg.confirm_user_understanding
        );
        println!("  outcome                    = {}", outcome.as_str());
        if !std::env::var("ELAI_LORA_SHIP_GATE_ENABLE")
            .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or(false)
        {
            println!(
                "  ssb_emit                   = skipped (ELAI_LORA_SHIP_GATE_ENABLE unset; wire-dark default)"
            );
        }
        if matches!(outcome, LoraShipGateOutcome::Refused) {
            eprintln!(
                "lora-gate-check: REFUSED -- adapter load blocked. Cite arXiv:2405.09673 (Biderman et al. 5-19pp catastrophic forgetting). Flip per workflow/decisions/E-47-flip-runbook.md."
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_missing_block_defaults_to_deferral() {
        let text = "# no finetune block\n[other]\nfoo = 1\n";
        let block = parse_finetune_lora(text).expect("parse");
        assert_eq!(block.ship_gate, LORA_SHIP_GATE_DEFERRAL_TOKEN);
        assert!(!block.confirm_user_understanding);
        assert!(!block.enabled);
    }

    #[test]
    fn parse_default_finetune_block() {
        let text = r#"
[finetune.lora]
ship_gate = "DEFERRED_PER_E47_REPORT"
confirm_user_understanding = true
"#;
        let block = parse_finetune_lora(text).expect("parse");
        assert_eq!(block.ship_gate, LORA_SHIP_GATE_DEFERRAL_TOKEN);
        assert!(block.confirm_user_understanding);
        assert!(!block.enabled);
    }
}
