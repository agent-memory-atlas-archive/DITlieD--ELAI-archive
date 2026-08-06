//! J-93 LEG-E — preference vs capability drift via dropped-constraint diff.
//! Deterministic. No LLM. OBSERVE-ONLY under kill-switch.

/// Env kill-switch. Default OFF.
pub const ELAI_DROPPED_CONSTRAINT_DIFF_ENABLE: &str = "ELAI_DROPPED_CONSTRAINT_DIFF_ENABLE";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintClass { Safety, Privacy, Security, Other }

impl ConstraintClass {
    pub fn as_str(self) -> &'static str {
        match self { Self::Safety => "safety", Self::Privacy => "privacy", Self::Security => "security", Self::Other => "other" }
    }
    pub fn is_preference_sensitive(self) -> bool { matches!(self, Self::Safety | Self::Privacy | Self::Security) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredConstraint { pub text: String, pub class: ConstraintClass }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriftKind {
    Disabled,
    UndeclaredSeal,
    CapabilityDriftNoDrop,
    CapabilityDrift { dropped: Vec<DeclaredConstraint> },
    PreferenceDrift { dropped: Vec<DeclaredConstraint> },
}
impl DriftKind { pub fn is_preference_drift(&self) -> bool { matches!(self, Self::PreferenceDrift { .. } | Self::UndeclaredSeal) } }

fn truthy_env_value(value: &str) -> bool {
    matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on")
}

pub fn dropped_constraint_diff_enabled() -> bool {
    std::env::var(ELAI_DROPPED_CONSTRAINT_DIFF_ENABLE).map(|v| truthy_env_value(&v)).unwrap_or(false)
}

fn prefixed(line: &str, prefix: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    if lower.starts_with(prefix) { Some(line[prefix.len()..].trim().to_owned()) } else { None }
}

pub fn classify_constraint_text(text: &str) -> ConstraintClass {
    let lower = text.to_ascii_lowercase();
    if ["security", "secret", "authz", "authentication"].iter().any(|w| lower.contains(w)) { ConstraintClass::Security }
    else if ["privacy", "pii", "gdpr", "confidential"].iter().any(|w| lower.contains(w)) { ConstraintClass::Privacy }
    else if ["safety", "harm", "weapon", "biohazard"].iter().any(|w| lower.contains(w)) { ConstraintClass::Safety }
    else { ConstraintClass::Other }
}

pub fn extract_sealed_constraints(sealed_objective: &str) -> Vec<DeclaredConstraint> {
    let mut result = Vec::new();
    for raw in sealed_objective.lines() {
        let line = raw.trim();
        if line.is_empty() { continue; }
        let parsed = prefixed(line, "safety:").map(|body| (ConstraintClass::Safety, body))
            .or_else(|| prefixed(line, "privacy:").map(|body| (ConstraintClass::Privacy, body)))
            .or_else(|| prefixed(line, "security:").map(|body| (ConstraintClass::Security, body)))
            .or_else(|| prefixed(line, "other:").map(|body| (ConstraintClass::Other, body)))
            .or_else(|| prefixed(line, "constraint:").map(|body| (classify_constraint_text(&body), body)));
        if let Some((class, text)) = parsed {
            result.push(DeclaredConstraint { text, class });
            continue;
        }
        let lower = line.to_ascii_lowercase();
        let bare = [("safety", ConstraintClass::Safety), ("privacy", ConstraintClass::Privacy), ("security", ConstraintClass::Security)]
            .iter().find(|(word, _)| lower.starts_with(word) && lower.as_bytes().get(word.len()).map(|b| !b.is_ascii_alphanumeric()).unwrap_or(true));
        if let Some((_, class)) = bare { result.push(DeclaredConstraint { text: line.to_owned(), class: *class }); }
    }
    result
}

fn significant_tokens(text: &str) -> std::collections::BTreeSet<String> {
    text.to_ascii_lowercase().split_whitespace().filter_map(|raw| {
        let token: String = raw.chars().filter(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '/' | '.')).collect();
        (token.len() >= 3).then_some(token)
    }).collect()
}

pub fn constraint_still_present(constraint: &DeclaredConstraint, current_summary: &str) -> bool {
    let tokens = significant_tokens(&constraint.text);
    if tokens.is_empty() { return true; }
    let current = significant_tokens(current_summary);
    tokens.iter().all(|token| current.contains(token))
}

pub fn classify_drift_assuming_enabled(sealed_objective: &str, current_summary: &str) -> DriftKind {
    let sealed = extract_sealed_constraints(sealed_objective);
    if sealed.is_empty() { return DriftKind::UndeclaredSeal; }
    let dropped: Vec<_> = sealed.into_iter().filter(|c| !constraint_still_present(c, current_summary)).collect();
    if dropped.is_empty() { DriftKind::CapabilityDriftNoDrop }
    else if dropped.iter().any(|c| c.class.is_preference_sensitive()) { DriftKind::PreferenceDrift { dropped } }
    else { DriftKind::CapabilityDrift { dropped } }
}

pub fn classify_drift(sealed_objective: &str, current_summary: &str) -> DriftKind {
    if !dropped_constraint_diff_enabled() { DriftKind::Disabled } else { classify_drift_assuming_enabled(sealed_objective, current_summary) }
}

pub fn observe_on_goal_drift(sealed_objective: &str, current_summary: &str) -> DriftKind {
    let kind = classify_drift(sealed_objective, current_summary);
    match &kind {
        DriftKind::Disabled => {}
        DriftKind::PreferenceDrift { dropped } => tracing::warn!(dropped = ?dropped.iter().map(|d| d.text.as_str()).collect::<Vec<_>>(), "J-93 LEG-E preference-drift (dropped safety/privacy/security; NO auto-retry; observe-only)"),
        DriftKind::UndeclaredSeal => tracing::warn!("J-93 LEG-E preference-drift fail-safe: sealed objective has no declared constraints (observe-only)"),
        DriftKind::CapabilityDrift { dropped } => tracing::warn!(dropped = ?dropped.iter().map(|d| d.text.as_str()).collect::<Vec<_>>(), "J-93 LEG-E capability-drift (non-sensitive constraint drop; observe-only)"),
        DriftKind::CapabilityDriftNoDrop => tracing::info!("J-93 LEG-E capability-drift with no constraint drop (observe-only)"),
    }
    kind
}

#[cfg(test)]
mod j93_dropped_constraint_diff_tests {
    use super::*;
    #[test] fn extract_prefixed_constraints() { let cs = extract_sealed_constraints("do the task
SAFETY: never exfiltrate secrets
PRIVACY: no PII in logs
ship it"); assert_eq!(cs.len(), 2); assert_eq!(cs[0].class, ConstraintClass::Safety); assert_eq!(cs[1].class, ConstraintClass::Privacy); }
    #[test] fn privacy_drop_is_preference_drift() { let kind = classify_drift_assuming_enabled("SAFETY: never exfiltrate secrets
OTHER: use rust", "use rust and ship the feature"); assert!(matches!(kind, DriftKind::PreferenceDrift { .. })); assert!(kind.is_preference_drift()); }
    #[test] fn other_drop_is_capability_drift() { let kind = classify_drift_assuming_enabled("OTHER: must use rustc nightly
OTHER: keep docs", "keep docs and use stable"); assert!(matches!(kind, DriftKind::CapabilityDrift { .. })); assert!(!kind.is_preference_drift()); }
    #[test] fn no_declared_constraints_is_undeclared_seal() { let kind = classify_drift_assuming_enabled("just a freeform goal with no tags", "totally different text"); assert_eq!(kind, DriftKind::UndeclaredSeal); assert!(kind.is_preference_drift()); }
    #[test] fn retained_constraints_no_drop() { let kind = classify_drift_assuming_enabled("PRIVACY: never export pii emails
OTHER: finish tests", "never export pii emails and finish tests today"); assert_eq!(kind, DriftKind::CapabilityDriftNoDrop); }
    #[test] fn flag_name_is_stable() { assert_eq!(ELAI_DROPPED_CONSTRAINT_DIFF_ENABLE, "ELAI_DROPPED_CONSTRAINT_DIFF_ENABLE"); }
    #[test] fn truthy_env_values_recognized() { assert!(truthy_env_value("1")); assert!(truthy_env_value("true")); assert!(truthy_env_value("YES")); assert!(truthy_env_value("on")); assert!(!truthy_env_value("0")); assert!(!truthy_env_value("")); assert!(!truthy_env_value("no")); }
}
