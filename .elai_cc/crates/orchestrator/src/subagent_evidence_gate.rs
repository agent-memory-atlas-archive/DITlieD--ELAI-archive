//! J-52 T1 evidence gate: validates a subagent's recorded receipt before acceptance.
//! The production call site is `crates/tools/src/lib.rs` `fn run_agent_job`; the gate can
//! be disabled with `ELAI_SUBAGENT_EVIDENCE_GATE_DISABLE`.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum EvidenceVerdict {
    Present { receipt_path: PathBuf },
    Blocked { reason: String },
    Disabled,
}

const ELAI_SUBAGENT_EVIDENCE_GATE_DISABLE_VAR: &str = "ELAI_SUBAGENT_EVIDENCE_GATE_DISABLE";

#[cfg(test)]
thread_local! {
    static EVIDENCE_GATE_DISABLED_OVERRIDE: std::cell::Cell<Option<bool>> =
        std::cell::Cell::new(None);
}

#[cfg(test)]
fn set_evidence_gate_disabled_for_test(value: Option<bool>) {
    EVIDENCE_GATE_DISABLED_OVERRIDE.with(|override_value| override_value.set(value));
}

pub fn subagent_evidence_gate_disabled() -> bool {
    #[cfg(test)]
    if let Some(value) = EVIDENCE_GATE_DISABLED_OVERRIDE.with(std::cell::Cell::get) {
        return value;
    }

    std::env::var(ELAI_SUBAGENT_EVIDENCE_GATE_DISABLE_VAR)
        .map(|value| value == "1")
        .unwrap_or(false)
}

pub fn evaluate_subagent_return(final_text: &str, evidence_root: &Path) -> EvidenceVerdict {
    if subagent_evidence_gate_disabled() {
        tracing::warn!(
            "subagent_evidence_gate: disabled via {}",
            ELAI_SUBAGENT_EVIDENCE_GATE_DISABLE_VAR
        );
        return EvidenceVerdict::Disabled;
    }

    let candidate = match final_text
        .lines()
        .map(str::trim)
        .rev()
        .find(|line| !line.is_empty())
        .and_then(|line| line.strip_prefix("EVIDENCE_RECORDED: "))
        .map(str::trim)
        .filter(|path| !path.is_empty())
    {
        Some(path) => PathBuf::from(path),
        None => {
            return EvidenceVerdict::Blocked {
                reason: "subagent trust gate: no EVIDENCE_RECORDED receipt line in the final message"
                    .to_owned(),
            };
        }
    };

    if candidate.components().any(|component| {
        matches!(component, std::path::Component::ParentDir)
    }) {
        return EvidenceVerdict::Blocked {
            reason: "subagent trust gate: receipt path escapes the agent evidence root".to_owned(),
        };
    }

    if candidate.is_absolute() && !candidate.starts_with(evidence_root) {
        return EvidenceVerdict::Blocked {
            reason: "subagent trust gate: receipt path escapes the agent evidence root".to_owned(),
        };
    }

    let resolved = if candidate.is_absolute() {
        candidate
    } else {
        evidence_root.join(candidate)
    };

    let file_type = match std::fs::symlink_metadata(&resolved) {
        Ok(metadata) => metadata.file_type(),
        Err(_) => {
            return EvidenceVerdict::Blocked {
                reason: "subagent trust gate: receipt file is missing or unreadable".to_owned(),
            };
        }
    };
    if file_type.is_symlink() {
        return EvidenceVerdict::Blocked {
            reason: "subagent trust gate: receipt path is a symlink".to_owned(),
        };
    }

    let canonical = match resolved.canonicalize() {
        Ok(path) => path,
        Err(_) => {
            return EvidenceVerdict::Blocked {
                reason: "subagent trust gate: receipt file is missing or unreadable".to_owned(),
            };
        }
    };
    let canonical_root = match evidence_root.canonicalize() {
        Ok(path) => path,
        Err(_) => {
            return EvidenceVerdict::Blocked {
                reason: "subagent trust gate: receipt file is missing or unreadable".to_owned(),
            };
        }
    };
    if !canonical.starts_with(&canonical_root) {
        return EvidenceVerdict::Blocked {
            reason: "subagent trust gate: receipt path escapes the agent evidence root".to_owned(),
        };
    }

    let metadata = match std::fs::metadata(&canonical) {
        Ok(metadata) => metadata,
        Err(_) => {
            return EvidenceVerdict::Blocked {
                reason: "subagent trust gate: receipt file is missing or unreadable".to_owned(),
            };
        }
    };
    if !metadata.is_file() {
        return EvidenceVerdict::Blocked {
            reason: "subagent trust gate: receipt file is missing or unreadable".to_owned(),
        };
    }
    if metadata.len() == 0 {
        return EvidenceVerdict::Blocked {
            reason: "subagent trust gate: receipt file is empty".to_owned(),
        };
    }

    EvidenceVerdict::Present {
        receipt_path: canonical,
    }
}

#[cfg(test)]
mod tests {
    use super::{evaluate_subagent_return, set_evidence_gate_disabled_for_test, EvidenceVerdict};
    use std::fs;
    use tempfile::TempDir;

    fn receipt(root: &TempDir, name: &str, contents: &[u8]) -> std::path::PathBuf {
        let path = root.path().join(name);
        fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn present_and_non_empty_receipt() {
        let root = TempDir::new().unwrap();
        let path = receipt(&root, "receipt.json", b"evidence");
        let result = evaluate_subagent_return(
            &format!("done\nEVIDENCE_RECORDED: {}", path.display()),
            root.path(),
        );
        assert!(matches!(result, EvidenceVerdict::Present { .. }));
    }

    #[test]
    fn no_receipt_line_is_blocked_with_trust_reason() {
        let root = TempDir::new().unwrap();
        let result = evaluate_subagent_return("done\n", root.path());
        match result {
            EvidenceVerdict::Blocked { reason } => assert!(reason.contains("trust")),
            other => panic!("unexpected verdict: {other:?}"),
        }
    }

    #[test]
    fn zero_byte_receipt_is_blocked() {
        let root = TempDir::new().unwrap();
        let path = receipt(&root, "empty", b"");
        let result = evaluate_subagent_return(
            &format!("EVIDENCE_RECORDED: {}", path.display()),
            root.path(),
        );
        assert!(matches!(result, EvidenceVerdict::Blocked { .. }));
    }

    #[cfg(unix)]
    #[test]
    fn symlink_receipt_is_blocked() {
        use std::os::unix::fs::symlink;
        let root = TempDir::new().unwrap();
        let target = receipt(&root, "target", b"evidence");
        let link = root.path().join("link");
        symlink(&target, &link).unwrap();
        let result = evaluate_subagent_return(
            &format!("EVIDENCE_RECORDED: {}", link.display()),
            root.path(),
        );
        assert!(matches!(result, EvidenceVerdict::Blocked { .. }));
    }

    #[test]
    fn parent_traversal_is_blocked() {
        let root = TempDir::new().unwrap();
        let result = evaluate_subagent_return("EVIDENCE_RECORDED: ../receipt", root.path());
        assert!(matches!(result, EvidenceVerdict::Blocked { .. }));
    }

    #[test]
    fn absolute_path_outside_root_is_blocked() {
        let root = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        let path = receipt(&outside, "receipt", b"evidence");
        let result = evaluate_subagent_return(
            &format!("EVIDENCE_RECORDED: {}", path.display()),
            root.path(),
        );
        assert!(matches!(result, EvidenceVerdict::Blocked { .. }));
    }

    #[test]
    fn disable_override_returns_disabled() {
        let root = TempDir::new().unwrap();
        set_evidence_gate_disabled_for_test(Some(true));
        let result = evaluate_subagent_return("not a receipt", root.path());
        set_evidence_gate_disabled_for_test(None);
        assert!(matches!(result, EvidenceVerdict::Disabled));
    }
}
