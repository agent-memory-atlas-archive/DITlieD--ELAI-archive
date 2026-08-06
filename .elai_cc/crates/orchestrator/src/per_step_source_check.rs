//! J-93 LEG-D — per-step deterministic SOURCE_CHECK on GRC grounding tags.

use std::path::{Path, PathBuf};

/// Env kill-switch. Default OFF.
pub const ELAI_PER_STEP_SOURCE_CHECK_ENABLE: &str = "ELAI_PER_STEP_SOURCE_CHECK_ENABLE";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepSourceVerdict {
    /// Citation verified against source span / file existence.
    Verified { evidence: String },
    /// External mode cited but check failed (missing file/symbol).
    Failed { evidence: String },
    /// No external receipt (training/reasoning-only/missing) OR flag-off path marker.
    Abstain { reason: String },
}

/// Truthy env check. Default OFF.
pub fn per_step_source_check_enabled() -> bool {
    std::env::var(ELAI_PER_STEP_SOURCE_CHECK_ENABLE)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn resolved_path(reference: &str, workspace_root: Option<&Path>) -> PathBuf {
    let path = Path::new(reference);
    if path.is_absolute() {
        path.to_path_buf()
    } else if let Some(root) = workspace_root {
        root.join(path)
    } else {
        path.to_path_buf()
    }
}

fn check_one(reference: &str, workspace_root: Option<&Path>) -> StepSourceVerdict {
    let trimmed = reference.trim();
    let Some((mode, value)) = trimmed.split_once(':') else {
        return StepSourceVerdict::Abstain {
            reason: "unknown_grounding_mode".into(),
        };
    };
    let mode = mode.trim().to_ascii_lowercase();
    let value = value.trim();
    match mode.as_str() {
        "file" | "grep" => {
            if value.is_empty() {
                StepSourceVerdict::Failed {
                    evidence: format!("{mode}:empty_reference"),
                }
            } else {
                let path = resolved_path(value, workspace_root);
                if path.is_file() {
                    StepSourceVerdict::Verified {
                        evidence: format!("{mode}:{} exists", path.display()),
                    }
                } else {
                    StepSourceVerdict::Failed {
                        evidence: format!("{mode}:{} missing", path.display()),
                    }
                }
            }
        }
        "tool" => {
            let parsed = value.rsplit_once("::").or_else(|| value.rsplit_once(':'));
            let Some((path_text, symbol)) = parsed else {
                return StepSourceVerdict::Abstain {
                    reason: "tool_without_path".into(),
                };
            };
            if path_text.trim().is_empty() || symbol.trim().is_empty() {
                return StepSourceVerdict::Abstain {
                    reason: "tool_without_path".into(),
                };
            }
            let path = resolved_path(path_text.trim(), workspace_root);
            let Ok(contents) = std::fs::read_to_string(&path) else {
                return StepSourceVerdict::Failed {
                    evidence: format!("tool:{} missing", path.display()),
                };
            };
            let symbol = symbol.trim();
            let pattern = format!(
                r"\b(fn|struct|enum|trait|impl|const|static|type|mod)\s+{}\b",
                regex::escape(symbol)
            );
            if regex::Regex::new(&pattern)
                .map(|r| r.is_match(&contents))
                .unwrap_or(false)
            {
                StepSourceVerdict::Verified {
                    evidence: format!("tool:{symbol} found in {}", path.display()),
                }
            } else {
                StepSourceVerdict::Failed {
                    evidence: format!("tool:{symbol} missing in {}", path.display()),
                }
            }
        }
        "test" | "api" => {
            if value.is_empty() {
                StepSourceVerdict::Failed {
                    evidence: format!("{mode}:empty_reference"),
                }
            } else {
                StepSourceVerdict::Verified {
                    evidence: format!("{mode}:{value} structurally present"),
                }
            }
        }
        "training" | "reasoning" | "" => StepSourceVerdict::Abstain {
            reason: "non_external_grounding".into(),
        },
        _ => StepSourceVerdict::Abstain {
            reason: "unknown_grounding_mode".into(),
        },
    }
}

/// Deterministically check GRC grounding citations without invoking an LLM.
pub fn per_step_source_check(
    grounding_tag: Option<&str>,
    claim_text: &str,
    workspace_root: Option<&Path>,
) -> StepSourceVerdict {
    let _ = claim_text;
    let Some(tag) = grounding_tag else {
        return StepSourceVerdict::Abstain {
            reason: "missing_grounding_tag".into(),
        };
    };
    let citations: Vec<&str> = tag
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if citations.is_empty() {
        return StepSourceVerdict::Abstain {
            reason: "missing_grounding_tag".into(),
        };
    }
    let mut verified = None;
    let mut abstain = None;
    for citation in citations {
        match check_one(citation, workspace_root) {
            StepSourceVerdict::Failed { evidence } => {
                return StepSourceVerdict::Failed { evidence };
            }
            StepSourceVerdict::Verified { evidence } => verified = Some(evidence),
            StepSourceVerdict::Abstain { reason } => abstain = Some(reason),
        }
    }
    verified
        .map(|evidence| StepSourceVerdict::Verified { evidence })
        .or_else(|| abstain.map(|reason| StepSourceVerdict::Abstain { reason }))
        .unwrap_or_else(|| StepSourceVerdict::Abstain {
            reason: "no_external_citation".into(),
        })
}

/// Production observe hook; enabled mode is warn/debug only and never aborts.
pub fn observe_step_grounding(
    grounding_tag: Option<&str>,
    claim_text: &str,
    workspace_root: Option<&Path>,
) {
    if !per_step_source_check_enabled() {
        return;
    }
    match per_step_source_check(grounding_tag, claim_text, workspace_root) {
        StepSourceVerdict::Verified { evidence } => {
            tracing::debug!(%evidence, "J-93 LEG-D per-step SOURCE_CHECK verified");
        }
        StepSourceVerdict::Failed { evidence } => {
            tracing::warn!(
                %evidence,
                claim = %claim_text,
                "J-93 LEG-D per-step SOURCE_CHECK failed (observe-only)"
            );
        }
        StepSourceVerdict::Abstain { reason } => {
            tracing::warn!(
                %reason,
                claim = %claim_text,
                "J-93 LEG-D per-step SOURCE_CHECK abstain (observe-only; RR-1)"
            );
        }
    }
}

#[cfg(test)]
mod j93_per_step_source_check_tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file(contents: &str) -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("j93-source-check-{id}.rs"));
        fs::write(&path, contents).expect("test temp file");
        path
    }

    #[test]
    fn flag_name_is_stable() {
        assert_eq!(
            ELAI_PER_STEP_SOURCE_CHECK_ENABLE,
            "ELAI_PER_STEP_SOURCE_CHECK_ENABLE"
        );
    }

    #[test]
    fn truthy_env_values_recognized() {
        fn truthy(v: &str) -> bool {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        }
        assert!(truthy("1"));
        assert!(truthy("TRUE"));
        assert!(truthy(" yes "));
        assert!(!truthy("0"));
        assert!(!truthy(""));
    }

    #[test]
    fn training_only_abstains() {
        assert!(matches!(
            per_step_source_check(Some("training:foo"), "", None),
            StepSourceVerdict::Abstain { .. }
        ));
    }

    #[test]
    fn missing_tag_abstains() {
        assert!(matches!(
            per_step_source_check(None, "", None),
            StepSourceVerdict::Abstain { .. }
        ));
    }

    #[test]
    fn file_exists_verifies() {
        let p = temp_file("x");
        assert!(matches!(
            per_step_source_check(Some(&format!("file:{}", p.display())), "", None),
            StepSourceVerdict::Verified { .. }
        ));
        let _ = fs::remove_file(p);
    }

    #[test]
    fn file_missing_fails() {
        assert!(matches!(
            per_step_source_check(Some("file:/no/such/path-xyz"), "", None),
            StepSourceVerdict::Failed { .. }
        ));
    }

    #[test]
    fn tool_with_path_finds_symbol() {
        let p = temp_file("fn hello_world() {}");
        assert!(matches!(
            per_step_source_check(
                Some(&format!("tool:{}::hello_world", p.display())),
                "",
                None
            ),
            StepSourceVerdict::Verified { .. }
        ));
        let _ = fs::remove_file(p);
    }

    #[test]
    fn tool_without_path_abstains() {
        let v = per_step_source_check(Some("tool:OnlySymbol"), "", None);
        assert!(matches!(
            v,
            StepSourceVerdict::Abstain { ref reason } if reason.contains("tool_without_path")
        ));
    }

    #[test]
    fn multi_one_fail_is_failed() {
        let p = temp_file("ok");
        let tag = format!("file:{},file:/no/such", p.display());
        assert!(matches!(
            per_step_source_check(Some(&tag), "", None),
            StepSourceVerdict::Failed { .. }
        ));
        let _ = fs::remove_file(p);
    }
}
