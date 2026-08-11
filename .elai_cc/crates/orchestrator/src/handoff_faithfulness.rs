//! Fail-closed handoff-faithfulness gate for critical-atom recall.

use crate::belief_state::{BeliefState, CriticalAtom};
use crate::lossless_compaction_gate::ELAI_LOSSLESS_GATE_DISABLE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoffFaithfulnessError {
    pub missing: Vec<CriticalAtom>,
}

fn keep_part(belief: &BeliefState, part: &str) -> bool {
    // The SSB row carries an id, not claim text, so re-derive ids in hash space.
    let quarantined = crate::claim::extract::extract_claims(part)
        .claims()
        .iter()
        .any(|claim| {
            let fact_id = crate::claim::prune::claim_fact_id(
                &crate::claim::prune::content_key(claim),
            );
            belief.quarantined_claims.iter().any(|id| id == &fact_id)
        });
    if !quarantined {
        return true;
    }
    let preserve_collision = belief.critical_atoms.iter().any(|atom| {
        atom.must_preserve && part.contains(atom.value.as_str())
    });
    if preserve_collision {
        tracing::warn!("quarantine projection collided with must-preserve atom; retaining part");
        return true;
    }
    false
}

/// Build the context haystack exposed to the receiving model.
pub fn projected_context_from_belief(belief: &BeliefState) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for t in &belief.tried {
        if keep_part(belief, &t.label) {
            parts.push(&t.label);
        }
    }
    for value in &belief.failed_because {
        if keep_part(belief, value) {
            parts.push(value);
        }
    }
    for value in &belief.constraints {
        if keep_part(belief, value) {
            parts.push(value);
        }
    }
    let ast = String::from_utf8_lossy(&belief.current_ast_state);
    if !ast.is_empty() && keep_part(belief, ast.as_ref()) {
        parts.push(ast.as_ref());
    }
    parts.join("\n")
}

/// Refuse projections missing any required atom, unless observation mode is enabled.
pub fn enforce_handoff_faithfulness(
    belief: &BeliefState,
    projected_context: &str,
) -> Result<(), HandoffFaithfulnessError> {
    let missing: Vec<CriticalAtom> = belief
        .critical_atoms
        .iter()
        .filter(|atom| atom.must_preserve && !projected_context.contains(atom.value.as_str()))
        .cloned()
        .collect();
    if missing.is_empty() {
        return Ok(());
    }
    if std::env::var(ELAI_LOSSLESS_GATE_DISABLE).as_deref() == Ok("1") {
        tracing::warn!(missing_count = missing.len(), "handoff gate disabled; proceeding");
        return Ok(());
    }
    tracing::error!(missing_count = missing.len(), "handoff gate refused projection");
    Err(HandoffFaithfulnessError { missing })
}

/// Build the projection and enforce the handoff gate.
pub fn enforce_belief_handoff_faithfulness(
    belief: &BeliefState,
) -> Result<(), HandoffFaithfulnessError> {
    let projection = projected_context_from_belief(belief);
    enforce_handoff_faithfulness(belief, &projection)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::belief_state::{AtomKind, TriedApproach};
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn belief(value: &str, required: bool) -> BeliefState {
        BeliefState::new(
            vec![TriedApproach { label: "attempt".into(), token_cost: 0 }],
            Vec::new(), Vec::new(), Vec::new(), Vec::new(), 0,
        ).with_critical_atoms(vec![CriticalAtom {
            kind: AtomKind::BugLocation, value: value.into(), must_preserve: required,
        }])
    }

    #[test]
    fn present_atom_passes() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::remove_var(ELAI_LOSSLESS_GATE_DISABLE);
        assert!(enforce_handoff_faithfulness(&belief("foo.rs:42", true), "foo.rs:42").is_ok());
    }

    #[test]
    fn absent_atom_fails() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::remove_var(ELAI_LOSSLESS_GATE_DISABLE);
        assert!(enforce_handoff_faithfulness(&belief("foo.rs:42", true), "context").is_err());
    }

    #[test]
    fn paraphrase_fails() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::remove_var(ELAI_LOSSLESS_GATE_DISABLE);
        assert!(enforce_handoff_faithfulness(&belief("foo.rs:42", true), "bug near foo").is_err());
    }

    #[test]
    fn optional_atom_may_be_missing() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::remove_var(ELAI_LOSSLESS_GATE_DISABLE);
        assert!(enforce_handoff_faithfulness(&belief("foo.rs:42", false), "context").is_ok());
    }

    #[test]
    fn kill_switch_allows_missing_atom() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::set_var(ELAI_LOSSLESS_GATE_DISABLE, "1");
        let result = enforce_handoff_faithfulness(&belief("foo.rs:42", true), "context");
        std::env::remove_var(ELAI_LOSSLESS_GATE_DISABLE);
        assert!(result.is_ok());
    }
}
