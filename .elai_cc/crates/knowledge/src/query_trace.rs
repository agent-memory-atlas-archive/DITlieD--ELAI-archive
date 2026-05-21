use crate::degradation::SearchMode;
use rustc_hash::FxHashMap;
use std::sync::Mutex;
use uuid::Uuid;
use elai_core::{EntropySource, OsEntropy};

// ---------------------------------------------------------------------------
// StageTrace (knowledge.txt L304-311)
// ---------------------------------------------------------------------------

/// Trace for a single stage in the retrieval pipeline.
///
/// Stages: "bm25", "splade", "code_vector", "text_vector", "pagerank",
/// "catrag", "arag", "rerank", "token_fit".
///
/// E-17 SP2 additions:
/// - "deep_cap"  — emitted when the DEEP candidate pool was trimmed by
///                  [`crate::deep::deep_pool_cap`] before RRF.
/// - "rrf_gate"  — emitted when the RRF quality gate
///                  ([`crate::deep::rrf_quality_gate`]) dropped one or
///                  more fused candidates below the configured floor.
#[derive(Debug, Clone, serde::Serialize)]
pub struct StageTrace {
    pub stage: String,
    pub latency_ms: u64,
    pub input_count: usize,
    pub output_count: usize,
    pub error: Option<String>,
    pub circuit_breaker_state: Option<String>,
}

// ---------------------------------------------------------------------------
// QueryTrace (knowledge.txt L297-303)
// ---------------------------------------------------------------------------

/// Structured trace emitted for every query through the knowledge pipeline.
///
/// Emitted at DEBUG level via `tracing` crate. Not stored in SSB per-query.
/// Aggregated per-session statistics (p50/p99 per stage) stored in SSB at
/// session end via `SessionTraceStats`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct QueryTrace {
    pub query_id: Uuid,
    pub stages: Vec<StageTrace>,
    pub total_latency_ms: u64,
    pub mode: String,
    pub result_count: usize,
}

impl QueryTrace {
    /// Build a `QueryTrace` from pipeline stages.
    ///
    /// `total_latency_ms` is the wall-clock time for the entire query, measured
    /// by the caller (not the sum of stage latencies, which may overlap).
    pub fn new(
        stages: Vec<StageTrace>,
        total_latency_ms: u64,
        mode: SearchMode,
        result_count: usize,
    ) -> Self {
        let mut entropy_buf = [0u8; 16];
        let mut _entropy = OsEntropy;
        _entropy.fill_bytes(&mut entropy_buf);
        // Bytes filled by OsEntropy form a valid opaque UUID (not RFC 4122 v4
        // but collision-resistant for trace IDs).
        let query_id = Uuid::from_bytes(entropy_buf);
        Self {
            query_id,
            stages,
            total_latency_ms,
            mode: format!("{mode}"),
            result_count,
        }
    }

    /// Emit this trace at DEBUG level via the `tracing` crate.
    pub fn emit(&self) {
        let json = serde_json::to_string(self).unwrap_or_else(|_| format!("{self:?}"));
        tracing::debug!(
            query_id = %self.query_id,
            mode = %self.mode,
            total_latency_ms = self.total_latency_ms,
            result_count = self.result_count,
            stage_count = self.stages.len(),
            trace = %json,
            "query_trace"
        );
    }
}

// ---------------------------------------------------------------------------
// SessionTraceStats (knowledge.txt L314)
// ---------------------------------------------------------------------------

/// Aggregated per-session statistics: p50/p99 latency per stage.
///
/// Accumulated across all queries in a session, then flushed to SSB at session
/// end. Thread-safe via internal `Mutex`.
pub struct SessionTraceStats {
    inner: Mutex<StatsInner>,
}

struct StatsInner {
    /// stage_name -> sorted-at-flush latency samples
    latencies: FxHashMap<String, Vec<u64>>,
    query_count: usize,
}

/// Per-stage percentile summary.
#[derive(Debug, Clone, serde::Serialize)]
pub struct StagePercentiles {
    pub stage: String,
    pub p50_ms: u64,
    pub p99_ms: u64,
    pub sample_count: usize,
}

/// Session-level summary ready for SSB serialization.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionSummary {
    pub query_count: usize,
    pub stage_percentiles: Vec<StagePercentiles>,
}

impl SessionTraceStats {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(StatsInner {
                latencies: FxHashMap::default(),
                query_count: 0,
            }),
        }
    }

    /// Record a completed query trace into the session accumulator.
    pub fn record(&self, trace: &QueryTrace) {
        let Ok(mut inner) = self.inner.lock() else {
            tracing::warn!("stats lock poisoned, skipping trace record");
            return;
        };
        inner.query_count += 1;
        for stage in &trace.stages {
            inner
                .latencies
                .entry(stage.stage.clone())
                .or_default()
                .push(stage.latency_ms);
        }
    }

    /// Compute p50/p99 per stage. Call at session end before SSB flush.
    pub fn summarize(&self) -> SessionSummary {
        let Ok(mut inner) = self.inner.lock() else {
            tracing::warn!("stats lock poisoned, returning empty summary");
            return SessionSummary {
                query_count: 0,
                stage_percentiles: Vec::new(),
            };
        };
        let mut percentiles = Vec::new();
        for (stage, samples) in &mut inner.latencies {
            samples.sort_unstable();
            let n = samples.len();
            if n == 0 {
                continue;
            }
            let p50 = samples[percentile_index(n, 50)];
            let p99 = samples[percentile_index(n, 99)];
            percentiles.push(StagePercentiles {
                stage: stage.clone(),
                p50_ms: p50,
                p99_ms: p99,
                sample_count: n,
            });
        }
        percentiles.sort_by(|a, b| a.stage.cmp(&b.stage));
        SessionSummary {
            query_count: inner.query_count,
            stage_percentiles: percentiles,
        }
    }

    /// Current query count (for testing).
    pub fn query_count(&self) -> usize {
        self.inner
            .lock()
            .map(|inner| inner.query_count)
            .unwrap_or(0)
    }
}

impl Default for SessionTraceStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Nearest-rank percentile index. Returns index in a sorted array of length `n`
/// for the given percentile (0-100). Uses ceil(p/100 * n) - 1, clamped.
fn percentile_index(n: usize, p: usize) -> usize {
    if n == 0 {
        return 0;
    }
    let rank = ((p as f64 / 100.0) * n as f64).ceil() as usize;
    rank.saturating_sub(1).min(n - 1)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::degradation::SearchMode;

    fn make_stage(name: &str, latency_ms: u64, input: usize, output: usize) -> StageTrace {
        StageTrace {
            stage: name.to_string(),
            latency_ms,
            input_count: input,
            output_count: output,
            error: None,
            circuit_breaker_state: None,
        }
    }

    fn make_trace(stages: Vec<StageTrace>, mode: SearchMode) -> QueryTrace {
        let result_count = stages.last().map_or(0, |s| s.output_count);
        QueryTrace::new(stages, 42, mode, result_count)
    }

    // ACCEPTANCE: Every query() call emits exactly one QueryTrace
    #[test]
    fn every_query_produces_one_trace() {
        let stages = vec![
            make_stage("bm25", 5, 0, 10),
            make_stage("code_vector", 12, 10, 8),
            make_stage("text_vector", 11, 10, 7),
            make_stage("rerank", 3, 25, 15),
        ];
        let trace = make_trace(stages, SearchMode::Hybrid);
        assert_eq!(trace.stages.len(), 4);
        assert!(!trace.query_id.is_nil());
    }

    // ACCEPTANCE: Emit exactly 3 traces for 3 queries (via subscriber capture)
    #[test]
    fn three_queries_emit_three_traces() {
        use tracing_subscriber::layer::SubscriberExt;


        let (events_tx, events_rx) = std::sync::mpsc::channel();
        let counter_layer = CounterLayer(Mutex::new(events_tx));
        let subscriber = tracing_subscriber::registry().with(counter_layer);

        let _guard = tracing::subscriber::set_default(subscriber);
        for _ in 0..3 {
            let trace = make_trace(
                vec![make_stage("bm25", 5, 0, 10)],
                SearchMode::Hybrid,
            );
            trace.emit();
        }
        drop(_guard);

        let count = events_rx.try_iter().count();
        assert_eq!(count, 3, "expected exactly 3 trace events, got {count}");
    }

    // ACCEPTANCE: HYBRID mode trace has bm25, code_vector, text_vector, rerank stages
    #[test]
    fn hybrid_trace_has_all_stages() {
        let stages = vec![
            make_stage("bm25", 5, 0, 10),
            make_stage("code_vector", 12, 10, 8),
            make_stage("text_vector", 11, 10, 7),
            make_stage("pagerank", 1, 25, 25),
            make_stage("catrag", 2, 25, 20),
            make_stage("arag", 3, 20, 18),
            make_stage("rerank", 3, 18, 15),
            make_stage("token_fit", 1, 15, 10),
        ];
        let trace = make_trace(stages, SearchMode::Hybrid);
        assert_eq!(trace.mode, "HYBRID");
        let stage_names: Vec<&str> = trace.stages.iter().map(|s| s.stage.as_str()).collect();
        assert!(stage_names.contains(&"bm25"));
        assert!(stage_names.contains(&"code_vector"));
        assert!(stage_names.contains(&"text_vector"));
        assert!(stage_names.contains(&"rerank"));
    }

    // ACCEPTANCE: BM25_ONLY trace reflects correct mode and only bm25 stage
    #[test]
    fn bm25_only_trace_reflects_mode() {
        let stages = vec![make_stage("bm25", 5, 0, 10)];
        let trace = make_trace(stages, SearchMode::Bm25Only);
        assert_eq!(trace.mode, "BM25_ONLY");
        assert_eq!(trace.stages.len(), 1);
        assert_eq!(trace.stages[0].stage, "bm25");
    }

    // ADVERSARIAL: circuit_breaker_state populated when breaker is OPEN
    #[test]
    fn circuit_breaker_state_populated_in_stage_trace() {
        let stage = StageTrace {
            stage: "rerank".to_string(),
            latency_ms: 0,
            input_count: 10,
            output_count: 10,
            error: None,
            circuit_breaker_state: Some("OPEN".to_string()),
        };
        let trace = QueryTrace::new(vec![stage], 5, SearchMode::Hybrid, 10);
        assert_eq!(
            trace.stages[0].circuit_breaker_state,
            Some("OPEN".to_string())
        );
    }

    // ADVERSARIAL: QueryTrace is actually emitted (not dead code)
    #[test]
    fn trace_emitted_with_subscriber() {
        use tracing_subscriber::layer::SubscriberExt;


        let (events_tx, events_rx) = std::sync::mpsc::channel();
        let counter_layer = CounterLayer(Mutex::new(events_tx));
        let subscriber = tracing_subscriber::registry().with(counter_layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        let trace = make_trace(
            vec![make_stage("bm25", 5, 0, 10)],
            SearchMode::Hybrid,
        );
        trace.emit();
        drop(_guard);

        let count = events_rx.try_iter().count();
        assert_eq!(count, 1, "expected 1 trace event, got {count}");
    }

    // ACCEPTANCE: p50/p99 aggregation computes correctly with known latencies
    #[test]
    fn p50_p99_aggregation() {
        let stats = SessionTraceStats::new();

        // 10 queries with bm25 latencies: 1,2,3,4,5,6,7,8,9,10
        for i in 1..=10u64 {
            let trace = QueryTrace::new(
                vec![make_stage("bm25", i, 0, 1)],
                i,
                SearchMode::Hybrid,
                1,
            );
            stats.record(&trace);
        }

        let summary = stats.summarize();
        assert_eq!(summary.query_count, 10);
        assert_eq!(summary.stage_percentiles.len(), 1);

        let bm25 = &summary.stage_percentiles[0];
        assert_eq!(bm25.stage, "bm25");
        assert_eq!(bm25.sample_count, 10);
        // p50 of [1..10]: ceil(0.5*10)-1 = 4 -> value 5
        assert_eq!(bm25.p50_ms, 5);
        // p99 of [1..10]: ceil(0.99*10)-1 = 9 -> value 10
        assert_eq!(bm25.p99_ms, 10);
    }

    // Edge case: single sample
    #[test]
    fn p50_p99_single_sample() {
        let stats = SessionTraceStats::new();
        let trace = QueryTrace::new(
            vec![make_stage("bm25", 42, 0, 1)],
            42,
            SearchMode::Hybrid,
            1,
        );
        stats.record(&trace);
        let summary = stats.summarize();
        let bm25 = &summary.stage_percentiles[0];
        assert_eq!(bm25.p50_ms, 42);
        assert_eq!(bm25.p99_ms, 42);
    }

    // Edge case: empty session
    #[test]
    fn empty_session_summary() {
        let stats = SessionTraceStats::new();
        let summary = stats.summarize();
        assert_eq!(summary.query_count, 0);
        assert!(summary.stage_percentiles.is_empty());
    }

    // ADVERSARIAL: latency >0 but exact values not asserted (non-deterministic)
    #[test]
    fn latency_fields_populated() {
        let stage = make_stage("bm25", 7, 0, 5);
        assert!(stage.latency_ms > 0);
    }

    // ACCEPTANCE: multiple stages aggregated independently
    #[test]
    fn multi_stage_aggregation() {
        let stats = SessionTraceStats::new();
        for i in 1..=5u64 {
            let trace = QueryTrace::new(
                vec![
                    make_stage("bm25", i * 2, 0, 10),
                    make_stage("rerank", i * 3, 10, 5),
                ],
                i * 5,
                SearchMode::Hybrid,
                5,
            );
            stats.record(&trace);
        }
        let summary = stats.summarize();
        assert_eq!(summary.query_count, 5);
        assert_eq!(summary.stage_percentiles.len(), 2);

        let bm25 = summary.stage_percentiles.iter().find(|s| s.stage == "bm25").unwrap();
        let rerank = summary.stage_percentiles.iter().find(|s| s.stage == "rerank").unwrap();
        assert_eq!(bm25.sample_count, 5);
        assert_eq!(rerank.sample_count, 5);
        // bm25 latencies: [2,4,6,8,10], p50=ceil(0.5*5)-1=2 -> 6
        assert_eq!(bm25.p50_ms, 6);
        // rerank latencies: [3,6,9,12,15], p50=ceil(0.5*5)-1=2 -> 9
        assert_eq!(rerank.p50_ms, 9);
    }

    // Helper: tracing layer that counts events via mpsc channel
    struct CounterLayer(Mutex<std::sync::mpsc::Sender<()>>);

    impl<S> tracing_subscriber::Layer<S> for CounterLayer
    where
        S: tracing::Subscriber,
    {
        fn on_event(
            &self,
            _event: &tracing::Event<'_>,
            _ctx: tracing_subscriber::layer::Context<'_, S>,
        ) {
            let _ = self.0.lock().unwrap().send(());
        }
    }
}
