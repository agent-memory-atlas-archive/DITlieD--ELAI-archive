//! J-112 -- deterministic spoken notification rendering for the V-1 seam.
//!
//! V-7 entropy quarantine: variant selection is a seeded hash of (session_id, ts_ms),
//! no ambient randomness. V-9: no model invocation on the notification event path.

use std::hash::{Hash, Hasher};

use crate::ssb::AgentNotificationKind;

/// Render concise spoken text for a notification without I/O or ambient state.
pub fn render_spoken(kind: AgentNotificationKind, session_id: &str, ts_ms: u64) -> String {
    let seed = row_seed(session_id, ts_ms);
    match kind {
        AgentNotificationKind::Completion => {
            const VARIANTS: [&str; 3] = [
                "Task complete.",
                "The task is complete.",
                "Task finished successfully.",
            ];
            VARIANTS[variant_index(seed, VARIANTS.len())].to_owned()
        }
        AgentNotificationKind::NeedsInput => {
            const VARIANTS: [&str; 3] = [
                "Human input is needed.",
                "The task needs your input.",
                "Run blocked; human decision required.",
            ];
            VARIANTS[variant_index(seed, VARIANTS.len())].to_owned()
        }
    }
}

fn variant_index(seed: u64, n_variants: usize) -> usize {
    if n_variants == 0 { 0 } else { (seed as usize) % n_variants }
}

fn row_seed(session_id: &str, ts_ms: u64) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    session_id.hash(&mut hasher);
    ts_ms.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_is_deterministic() {
        assert_eq!(
            render_spoken(AgentNotificationKind::Completion, "s", 7),
            render_spoken(AgentNotificationKind::Completion, "s", 7)
        );
    }

    #[test]
    fn kinds_have_distinct_non_empty_text() {
        let completion = render_spoken(AgentNotificationKind::Completion, "s", 7);
        let needs_input = render_spoken(AgentNotificationKind::NeedsInput, "s", 7);
        assert!(!completion.is_empty());
        assert!(!needs_input.is_empty());
        assert_ne!(completion, needs_input);
    }
}
