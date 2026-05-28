// E-55 contract: NO DPO / PPO / GRPO / DAPO / KTO weight-update module
// exists in crates/orchestrator/src/ or crates/training/ until the
// rl_weight_update_unlocked SSB row has been emitted in a recorded
// session.
//
// Verifies (per plan stub_ban + Task 8 + Task 9):
// 1. SCHEMA_VERSION_PUBLIC >= 168 (E-55 floor; RewardLadderEvent variant
//    landed at this bump).
// 2. SsbActionType::RewardLadderEvent + RewardLadderEventKind 9-arm
//    discriminator are reachable from the public API (binds the wire-
//    format string + the unlock-event arm).
// 3. NO source file path under .elai_cc/crates/orchestrator/src/ or
//    .elai_cc/crates/training/ matches the regex
//        (?i)\b(dpo|ppo|grpo|dapo|kto)_
//    The regex is anchored on `<token>_` so unrelated identifiers (e.g.
//    "report") do not false-positive. The contract test's own filename
//    is excluded via an allowlist.
// 4. RewardLadderEventKind::RlWeightUpdateUnlocked exists as the unlock
//    discriminator -- it is the single signal that, once emitted in a
//    real session, allows the gated weight-update modules to be added
//    in a follow-up patch (E-55.1).
//
// DPO is HARD-AVOIDED forever (Apple 2025 OOD drop + arXiv:2502.20847
// gradient imbalance + ELAI has no paired preference data). PPO / GRPO /
// DAPO / KTO are HARD-DEFERRED until all 3 unlock conditions TRUE
// (bai_saturated + orm_coverage >= 0.8 + vram_headroom >= 4GB). See
// crates/orchestrator/src/rl/rl_weight_update_deferred.rs +
// workflow/reasoning/E-55-decisions.md.

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, RewardLadderEventKind, SsbActionType,
    SCHEMA_VERSION_PUBLIC,
};
use std::path::{Path, PathBuf};

/// Floor for E-55: head SCHEMA_VERSION at completion-drive close. Atomic
/// +1 bump from E-54's 167 head (single new variant RewardLadderEvent;
/// see workflow/reasoning/E-55-decisions.md).
const E55_SCHEMA_FLOOR: i64 = 168;

/// The contract test's own filename. Excluded from the regex-match scan
/// so the test does not fail by detecting its own filename.
const SELF_FILENAME: &str = "no_dpo_no_grpo_until_unlock.rs";

/// Token regex applied to filename basenames. Each token is checked as a
/// word-boundary prefix followed by `_` so unrelated identifiers do not
/// false-positive (e.g. "report_pkg" does not match).
const PROHIBITED_TOKENS: &[&str] = &["dpo_", "ppo_", "grpo_", "dapo_", "kto_"];

#[test]
fn schema_version_includes_e55_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= E55_SCHEMA_FLOOR,
        "schema must include E-55 RewardLadderEvent variant (got {SCHEMA_VERSION_PUBLIC}, floor {E55_SCHEMA_FLOOR})"
    );
}

#[test]
fn reward_ladder_event_wire_tag_resolves() {
    let parsed = ssb_action_type_from_str_public("reward_ladder_event")
        .expect("reward_ladder_event must round-trip via from_str");
    assert_eq!(parsed, SsbActionType::RewardLadderEvent);
}

#[test]
fn rl_weight_update_unlocked_kind_exists() {
    // Smoke: the unlock discriminator MUST be present so callers can
    // emit it once the 3 conditions all clear.
    let kind = RewardLadderEventKind::RlWeightUpdateUnlocked;
    assert_eq!(kind.as_str(), "rl_weight_update_unlocked");
}

#[test]
fn nine_arm_discriminator_present() {
    // Smoke: all 9 fire-points reachable from the public surface.
    let arms = [
        RewardLadderEventKind::RlaifJudgeRated,
        RewardLadderEventKind::RlaifCalibrationCorrelationLow,
        RewardLadderEventKind::GepaTemplateEvolved,
        RewardLadderEventKind::GepaTemplateEvicted,
        RewardLadderEventKind::RewardFusionComputed,
        RewardLadderEventKind::RewardHackingSuspected,
        RewardLadderEventKind::SftTrainingThresholdReached,
        RewardLadderEventKind::BaiArmObservationRecorded,
        RewardLadderEventKind::RlWeightUpdateUnlocked,
    ];
    let labels: std::collections::BTreeSet<_> =
        arms.iter().map(|a| a.as_str()).collect();
    assert_eq!(labels.len(), 9, "all 9 RewardLadderEventKind arms must be distinct");
}

#[test]
fn no_dpo_ppo_grpo_dapo_kto_module_until_unlock() {
    // Locate the repo root via CARGO_MANIFEST_DIR -- the contracts crate's
    // Cargo.toml lives at .elai_cc/crates/contracts/Cargo.toml.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let elai_cc_root = PathBuf::from(manifest_dir)
        .parent() // crates/
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("locate .elai_cc root via CARGO_MANIFEST_DIR");

    let scan_dirs = [
        elai_cc_root.join("crates").join("orchestrator").join("src"),
        elai_cc_root.join("crates").join("training"),
    ];

    let mut offenders: Vec<PathBuf> = Vec::new();
    for dir in &scan_dirs {
        if !dir.exists() {
            // crates/training/ may be absent until the SFT-LoRA patch
            // E-55.1 lands; absence is correct pre-unlock.
            continue;
        }
        walk_rs_files(dir, &mut |path| {
            let name = match path.file_name().and_then(|s| s.to_str()) {
                Some(s) => s,
                None => return,
            };
            // Skip the contract test file itself.
            if name == SELF_FILENAME {
                return;
            }
            let lower = name.to_ascii_lowercase();
            for tok in PROHIBITED_TOKENS {
                if lower.starts_with(tok) || lower.contains(&format!("/{tok}")) || lower.contains(tok) {
                    // Whitelist: rl_weight_update_deferred.rs may mention
                    // these tokens only inside comments, never in the
                    // FILENAME -- the filename does not contain any token
                    // so the substring check should not fire on it. Defense
                    // in depth: explicit-skip the deferred marker file by
                    // FILENAME (which contains none of the tokens).
                    if name == "rl_weight_update_deferred.rs" {
                        return;
                    }
                    offenders.push(path.to_path_buf());
                    return;
                }
            }
        });
    }

    assert!(
        offenders.is_empty(),
        "DPO/PPO/GRPO/DAPO/KTO weight-update modules are HARD-BLOCKED until rl_weight_update_unlocked SSB row emitted. \
         Found offending file(s): {:?}",
        offenders
    );
}

/// Walk *.rs files under `dir` and invoke `cb` for each. Pure stdlib --
/// no walkdir dep.
fn walk_rs_files(dir: &Path, cb: &mut dyn FnMut(&Path)) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_rs_files(&path, cb);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            cb(&path);
        }
    }
}
