// J-29 SLICE-3: LiveUsageGauge -- process-global AtomicU64 counters for
// in-flight token usage, read by the dashboard `get_live_usage` Tauri
// command and the CLI progress reporter. This is a live gauge only; it
// never journals to the SSB. Reset on turn end.
//
// Producers:
//   - dashboard-v2/src-tauri/src/commands.rs cloud SSE loop (extract_text_delta)
//   - cli/src/runtime_client.rs TextDelta arm
// Consumer:
//   - dashboard-v2/src-tauri/src/commands.rs::get_live_usage
//
// Lock-free hot path: AtomicU64::fetch_add. Snapshot is a relaxed read
// of both counters; callers treat the snapshot as advisory.

use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, Default, serde::Serialize)]
pub struct LiveUsageSnapshot {
    pub output_tokens: u64,
    pub active_streams: u64,
}

static OUTPUT_TOKENS: AtomicU64 = AtomicU64::new(0);
static ACTIVE_STREAMS: AtomicU64 = AtomicU64::new(0);

pub fn bump_output(delta: u64) {
    if delta != 0 {
        OUTPUT_TOKENS.fetch_add(delta, Ordering::Relaxed);
    }
}

pub fn begin_stream() {
    ACTIVE_STREAMS.fetch_add(1, Ordering::Relaxed);
}

pub fn end_stream() {
    ACTIVE_STREAMS.fetch_sub(1, Ordering::Relaxed);
}

pub fn reset_turn() {
    OUTPUT_TOKENS.store(0, Ordering::Relaxed);
}

pub fn snapshot() -> LiveUsageSnapshot {
    LiveUsageSnapshot {
        output_tokens: OUTPUT_TOKENS.load(Ordering::Relaxed),
        active_streams: ACTIVE_STREAMS.load(Ordering::Relaxed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bump_and_snapshot() {
        reset_turn();
        bump_output(10);
        let s = snapshot();
        assert_eq!(s.output_tokens, 10);
        reset_turn();
        let s = snapshot();
        assert_eq!(s.output_tokens, 0);
    }

    #[test]
    fn active_streams_balance() {
        let before = snapshot().active_streams;
        begin_stream();
        begin_stream();
        assert_eq!(snapshot().active_streams, before + 2);
        end_stream();
        end_stream();
        assert_eq!(snapshot().active_streams, before);
    }
}
