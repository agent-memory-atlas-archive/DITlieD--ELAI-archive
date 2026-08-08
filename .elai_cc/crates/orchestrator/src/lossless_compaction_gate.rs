//! Fail-closed lossless compaction gate.
//!
//! This module owns the policy wrapper around the archival precondition check;
//! the compaction mechanics and check result remain in [`crate::compact`].

use crate::compact::{check_lossless_compaction, CompactionConfig, LosslessCompactionCheck};
use crate::session::Session;

/// Environment variable that switches the gate to observe-only behavior.
pub const ELAI_LOSSLESS_GATE_DISABLE: &str = "ELAI_LOSSLESS_GATE_DISABLE";

/// Error returned when compaction would discard unrecoverable messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LosslessCompactionError {
    pub check: LosslessCompactionCheck,
}

/// Fail-closed gate over [`check_lossless_compaction`]. Default ON; DISABLE=1 observes.
pub fn enforce_lossless_compaction<R: crate::ssb::SsbReader + ?Sized>(
    session: &Session,
    config: CompactionConfig,
    snap_id: Option<&str>,
    ssb: Option<&R>,
) -> Result<(), LosslessCompactionError> {
    let check = check_lossless_compaction(session, config, snap_id, ssb);
    if std::env::var(ELAI_LOSSLESS_GATE_DISABLE).as_deref() == Ok("1") {
        if !check.is_safe {
            tracing::warn!(
                unrecoverable_count = check.unrecoverable_count,
                "lossless compaction gate disabled; proceeding with potentially lossy trim"
            );
        }
        return Ok(());
    }
    if check.is_safe {
        Ok(())
    } else {
        tracing::error!(
            unrecoverable_count = check.unrecoverable_count,
            to_be_dropped = check.to_be_dropped,
            "lossless compaction refused: dropped messages are not recoverable"
        );
        Err(LosslessCompactionError { check })
    }
}

#[cfg(test)]
mod tests {
    use super::enforce_lossless_compaction;
    use crate::compact::CompactionConfig;
    use crate::session::{ContentBlock, ConversationMessage, Session};
    use std::sync::Mutex;

    static LOSSLESS_GATE_ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn lossless_gate_refuses_unrecoverable_compaction() {
        let _lock = LOSSLESS_GATE_ENV_LOCK.lock().unwrap();
        std::env::remove_var("ELAI_LOSSLESS_GATE_DISABLE");
        let mut session = Session::new();
        session.messages = vec![
            ConversationMessage::user_text("old"),
            ConversationMessage::assistant(vec![ContentBlock::Text { text: "recent".into() }]),
        ];
        let config = CompactionConfig {
            preserve_recent_messages: 1,
            max_estimated_tokens: 0,
            preserve_intermediate_numerics: false,
        };
        assert!(enforce_lossless_compaction(
            &session,
            config,
            None,
            None::<&dyn crate::ssb::SsbReader>,
        )
        .is_err());
    }

    #[test]
    fn lossless_gate_allows_safe_noop_compaction() {
        let _lock = LOSSLESS_GATE_ENV_LOCK.lock().unwrap();
        std::env::remove_var("ELAI_LOSSLESS_GATE_DISABLE");
        let session = Session::new();
        assert!(enforce_lossless_compaction(
            &session,
            CompactionConfig::default(),
            None,
            None::<&dyn crate::ssb::SsbReader>,
        )
        .is_ok());
    }

    #[test]
    fn lossless_gate_kill_switch_observes() {
        let _lock = LOSSLESS_GATE_ENV_LOCK.lock().unwrap();
        let mut session = Session::new();
        session.messages = vec![
            ConversationMessage::user_text("old"),
            ConversationMessage::assistant(vec![ContentBlock::Text { text: "recent".into() }]),
        ];
        let config = CompactionConfig {
            preserve_recent_messages: 1,
            max_estimated_tokens: 0,
            preserve_intermediate_numerics: false,
        };
        std::env::set_var("ELAI_LOSSLESS_GATE_DISABLE", "1");
        let result = enforce_lossless_compaction(
            &session,
            config,
            None,
            None::<&dyn crate::ssb::SsbReader>,
        );
        std::env::remove_var("ELAI_LOSSLESS_GATE_DISABLE");
        assert!(result.is_ok());
    }
}
