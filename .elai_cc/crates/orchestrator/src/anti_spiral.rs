// Anti-spiral / hot-loop detection for GRC agents.
// Monitors repeated (step_type, topic, content_hash) tuples per agent.
// Triggers escalation after SPIRAL_THRESHOLD identical steps, then HardStop
// if the agent ignores the escalation and repeats again.
//
// Production wiring scheduled in Session wire-antispiral:
// task_runner owns an `AntiSpiralDetector` per task, feeds every GRC step
// into `check_step`, injects `ESCALATION_MESSAGE` on EscalationSignalNeeded,
// and emits `SsbPayload::SpiralDetected` + a `spiral_hardstop` Finding on
// HardStop. Depends on Session wire-4A (real agent execution; anti-spiral
// on synthetic baby steps is meaningless). Claude-Code-layer mcp_think
// anti-spiral is a separate concern — this module is defense-in-depth at
// the orchestrator layer.

use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;

use elai_core::ClockProvider;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Ring buffer capacity: stores N steps to detect N consecutive identical ones.
pub const RING_SIZE: usize = 4;

/// Number of consecutive identical steps before escalation fires.
pub const SPIRAL_THRESHOLD: u32 = 3;

/// Message injected into the agent's context on escalation.
pub const ESCALATION_MESSAGE: &str =
    "Anti-spiral: 3x repeated step detected. Converge or request help.";

/// spiral_depth value for the HardStop signal (escalation was ignored).
pub const HARD_STOP_DEPTH: u32 = 2;

/// spiral_depth value for the initial EscalationSignalNeeded.
pub const ESCALATION_DEPTH: u32 = 1;

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct StepRecord {
    step_type: String,
    topic: String,
    content_hash: [u8; 32],
}

#[derive(Debug)]
struct AgentSpiralState {
    step_ring: VecDeque<StepRecord>,
    same_step_count: u32,
    escalation_sent: bool,
    recovery_target_step_index: Option<i64>,
}

impl AgentSpiralState {
    fn new() -> Self {
        Self {
            step_ring: VecDeque::with_capacity(RING_SIZE),
            same_step_count: 0,
            escalation_sent: false,
            recovery_target_step_index: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Public result type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpiralResult {
    Clean,
    EscalationSignalNeeded {
        message: String,
        spiral_depth: u32,
        /// Blake3 hex of the repeated step content (for AntiSpiralHit SSB row).
        step_hash: String,
        /// Number of consecutive repetitions at escalation time.
        count: u32,
    },
    HardStop {
        spiral_depth: u32,
    },
}

// ---------------------------------------------------------------------------
// Detector
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct AntiSpiralDetector<C: ClockProvider> {
    states: HashMap<Uuid, AgentSpiralState>,
    _clock: PhantomData<C>,
}

impl<C: ClockProvider> Default for AntiSpiralDetector<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: ClockProvider> AntiSpiralDetector<C> {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            _clock: PhantomData,
        }
    }

    /// Record a GRC step for an agent.
    ///
    /// `superstep` is the current SSB superstep index, used to compute the
    /// recovery target (the step just before the spiral started).
    pub fn record_step(
        &mut self,
        agent_id: Uuid,
        step_type: &str,
        topic: &str,
        content: &str,
        superstep: i64,
    ) -> SpiralResult {
        let normalized = normalize_content(content);
        let hash = hash_content(&normalized);
        let record = StepRecord {
            step_type: step_type.to_owned(),
            topic: topic.to_owned(),
            content_hash: hash,
        };

        let state = self
            .states
            .entry(agent_id)
            .or_insert_with(AgentSpiralState::new);

        let is_repeat = state
            .step_ring
            .back()
            .is_some_and(|prev| *prev == record);

        if is_repeat {
            state.same_step_count += 1;
        } else {
            // Different step breaks the streak.
            state.same_step_count = 1;
            state.escalation_sent = false;
            state.recovery_target_step_index = None;
        }

        // Maintain ring buffer capacity.
        if state.step_ring.len() >= RING_SIZE {
            state.step_ring.pop_front();
        }
        state.step_ring.push_back(record);

        if state.escalation_sent && is_repeat {
            // Agent ignored the escalation and repeated again.
            return SpiralResult::HardStop { spiral_depth: HARD_STOP_DEPTH };
        }

        if state.same_step_count > SPIRAL_THRESHOLD {
            state.escalation_sent = true;
            // Recovery target is the step just BEFORE the spiral started.
            // The spiral started at (superstep - same_step_count + 1), so
            // the safe checkpoint is one before that.
            let spiral_start = superstep - (state.same_step_count as i64) + 1;
            state.recovery_target_step_index = Some(spiral_start - 1);
            let hash_hex = hash.iter().map(|b| format!("{b:02x}")).collect::<String>();
            return SpiralResult::EscalationSignalNeeded {
                message: ESCALATION_MESSAGE.to_owned(),
                spiral_depth: ESCALATION_DEPTH,
                step_hash: hash_hex,
                count: state.same_step_count,
            };
        }

        SpiralResult::Clean
    }

    /// Returns the recovery target step index for an agent in spiral state.
    /// This is the superstep just before the spiral started (NOT the latest).
    pub fn get_recovery_target(&self, agent_id: &Uuid) -> Option<i64> {
        self.states
            .get(agent_id)
            .and_then(|s| s.recovery_target_step_index)
    }

    /// Clear all spiral tracking state for an agent (phase boundary / clean exit).
    pub fn reset_agent(&mut self, agent_id: &Uuid) {
        self.states.remove(agent_id);
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Normalize content for semantic comparison:
/// lowercase, collapse all whitespace runs to single space, trim.
fn normalize_content(s: &str) -> String {
    let lower = s.to_lowercase();
    let mut result = String::with_capacity(lower.len());
    let mut prev_was_ws = true; // treat leading whitespace as collapsible
    for ch in lower.chars() {
        if ch.is_whitespace() {
            if !prev_was_ws {
                result.push(' ');
            }
            prev_was_ws = true;
        } else {
            result.push(ch);
            prev_was_ws = false;
        }
    }
    // Trim trailing space left by the collapse logic.
    if result.ends_with(' ') {
        result.pop();
    }
    result
}

/// Hash normalized content using blake3 (32-byte output).
fn hash_content(normalized: &str) -> [u8; 32] {
    *blake3::hash(normalized.as_bytes()).as_bytes()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::SeededClock;
    use proptest::prelude::*;

    fn detector() -> AntiSpiralDetector<SeededClock> {
        AntiSpiralDetector::new()
    }

    // TEST-1: 3 identical steps -> still Clean (threshold is >3), 4th triggers escalation
    #[test]
    fn test_spiral_detected_after_threshold() {
        let mut det = detector();
        let id = Uuid::nil();

        // Steps 1-3: same content, same_step_count goes 1->2->3
        for i in 0..3 {
            let result = det.record_step(id, "observe", "debug", "checking file X", i);
            assert_eq!(result, SpiralResult::Clean, "step {} should be clean", i);
        }
        // Step 4: same_step_count becomes 4 which is > SPIRAL_THRESHOLD (3)
        let result = det.record_step(id, "observe", "debug", "checking file X", 3);
        match &result {
            SpiralResult::EscalationSignalNeeded {
                message,
                spiral_depth,
                step_hash,
                count,
            } => {
                assert_eq!(message.as_str(), ESCALATION_MESSAGE);
                assert_eq!(*spiral_depth, 1);
                assert_eq!(*count, 4);
                // step_hash is a 64-char blake3 hex string
                assert_eq!(step_hash.len(), 64, "expected 64-char hex step_hash");
            }
            other => panic!("expected EscalationSignalNeeded, got {:?}", other),
        }
    }

    // TEST-2: 3 different steps -> Clean (no spiral)
    #[test]
    fn test_different_steps_no_spiral() {
        let mut det = detector();
        let id = Uuid::nil();

        let r1 = det.record_step(id, "observe", "debug", "step A", 0);
        let r2 = det.record_step(id, "hypothesize", "debug", "step B", 1);
        let r3 = det.record_step(id, "plan_action", "debug", "step C", 2);
        assert_eq!(r1, SpiralResult::Clean);
        assert_eq!(r2, SpiralResult::Clean);
        assert_eq!(r3, SpiralResult::Clean);
    }

    // TEST-3: spiral + escalation sent + 1 more identical -> HardStop
    #[test]
    fn test_hard_stop_after_escalation_ignored() {
        let mut det = detector();
        let id = Uuid::nil();

        // Trigger escalation (4 identical steps)
        for i in 0..4 {
            det.record_step(id, "observe", "debug", "stuck", i);
        }
        // 5th identical step: agent ignored escalation
        let result = det.record_step(id, "observe", "debug", "stuck", 4);
        assert_eq!(result, SpiralResult::HardStop { spiral_depth: 2 });
    }

    // TEST-4: spiral + escalation + different step -> Clean (recovery)
    #[test]
    fn test_recovery_after_escalation() {
        let mut det = detector();
        let id = Uuid::nil();

        // Trigger escalation
        for i in 0..4 {
            det.record_step(id, "observe", "debug", "stuck", i);
        }
        // Different step = agent recovered
        let result = det.record_step(id, "plan_action", "debug", "new approach", 4);
        assert_eq!(result, SpiralResult::Clean);

        // Verify escalation_sent was reset (repeat of the new step should not HardStop)
        let r2 = det.record_step(id, "plan_action", "debug", "new approach", 5);
        assert_eq!(r2, SpiralResult::Clean);
    }

    // TEST-5: trailing space normalization
    #[test]
    fn test_trailing_space_normalization() {
        let a = normalize_content("check file X");
        let b = normalize_content("check file X ");
        assert_eq!(a, b);
        assert_eq!(hash_content(&a), hash_content(&b));
    }

    // TEST-6: internal whitespace collapse
    #[test]
    fn test_internal_whitespace_collapse() {
        let a = normalize_content("check file X");
        let b = normalize_content("CHECK  FILE  X");
        assert_eq!(a, b);
        assert_eq!(hash_content(&a), hash_content(&b));
    }

    // TEST-7: recovery target is spiral_start - 1
    #[test]
    fn test_recovery_target_before_spiral() {
        let mut det = detector();
        let id = Uuid::nil();

        // supersteps 10, 11, 12, 13 — spiral starts at 10, so recovery = 9
        for superstep in 10..14 {
            det.record_step(id, "observe", "debug", "same", superstep);
        }
        let target = det.get_recovery_target(&id);
        assert_eq!(target, Some(9));
    }

    // TEST-8: proptest whitespace normalization
    proptest! {
        #[test]
        fn test_proptest_whitespace_normalization(
            base in "[a-z]{1,20}",
            leading_ws in "\\s{0,5}",
            trailing_ws in "\\s{0,5}",
            internal_ws in "\\s{1,5}",
        ) {
            let plain = format!("word1 {base}");
            let decorated = format!("{leading_ws}word1{internal_ws}{base}{trailing_ws}");
            prop_assert_eq!(normalize_content(&plain), normalize_content(&decorated));
        }
    }

    // TEST-9: independent agent tracking
    #[test]
    fn test_independent_agent_tracking() {
        let mut det = detector();
        let a = Uuid::from_u128(1);
        let b = Uuid::from_u128(2);

        // Agent A spirals
        for i in 0..4 {
            det.record_step(a, "observe", "debug", "stuck", i);
        }
        // Agent B is fine
        let result = det.record_step(b, "observe", "debug", "stuck", 0);
        assert_eq!(result, SpiralResult::Clean);
    }

    // TEST-10: reset_agent clears state
    #[test]
    fn test_reset_agent_clears_state() {
        let mut det = detector();
        let id = Uuid::nil();

        // Build up to threshold
        for i in 0..3 {
            det.record_step(id, "observe", "debug", "stuck", i);
        }
        det.reset_agent(&id);

        // Same steps after reset should start fresh
        for i in 0..3 {
            let result = det.record_step(id, "observe", "debug", "stuck", i + 10);
            assert_eq!(result, SpiralResult::Clean);
        }
    }

    // TEST-11: get_recovery_target returns None for unknown agent
    #[test]
    fn test_recovery_target_none_for_unknown() {
        let det = detector();
        assert_eq!(det.get_recovery_target(&Uuid::nil()), None);
    }

    // TEST-12: escalation does not fire on step_type change alone
    #[test]
    fn test_step_type_change_breaks_spiral() {
        let mut det = detector();
        let id = Uuid::nil();

        for i in 0..3 {
            det.record_step(id, "observe", "debug", "same content", i);
        }
        // Same content + topic, but different step_type
        let result = det.record_step(id, "hypothesize", "debug", "same content", 3);
        assert_eq!(result, SpiralResult::Clean);
    }

    // TEST-13: topic change breaks spiral
    #[test]
    fn test_topic_change_breaks_spiral() {
        let mut det = detector();
        let id = Uuid::nil();

        for i in 0..3 {
            det.record_step(id, "observe", "debug-a", "same content", i);
        }
        let result = det.record_step(id, "observe", "debug-b", "same content", 3);
        assert_eq!(result, SpiralResult::Clean);
    }
}
