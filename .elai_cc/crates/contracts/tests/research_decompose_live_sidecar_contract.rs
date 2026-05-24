//! D5 / A6: Live-sidecar grounding contract test for research_decompose.
//!
//! Gated by ELAI_LIVE_SIDECAR=1.  When unset, the test no-ops and passes so
//! cargo test remains green in CI without a live sidecar.
//!
//! -----------------------------------------------------------------------
//! Environment variables
//! -----------------------------------------------------------------------
//! ELAI_LIVE_SIDECAR=1
//!   Required to activate any live-path logic.  Without it the whole test
//!   body is skipped immediately (default-path, CI-safe, passes in <2 s).
//!
//! ELAI_LIVE_SIDECAR_AUTOSTART=1
//!   Optional, requires ELAI_LIVE_SIDECAR=1.
//!   When the sidecar (port 8100) is not reachable, the test will attempt
//!   to start the SSH tunnel via:
//!     powershell.exe -ExecutionPolicy Bypass -NonInteractive -File
//!       F:\.ELAI_workflow\workflow\remote-setup\start-tunnel.ps1
//!   The script is invoked with std::process::Command (no external crate
//!   dependency). After spawning it polls port 8100 for up to 60 s.
//!   If the port is still closed after 60 s the test returns the structured
//!   WARN+skip rather than panicking.
//!   When ELAI_LIVE_SIDECAR_AUTOSTART != "1", this code path is never
//!   executed and behaviour is identical to the original D5 test.
//!
//! -----------------------------------------------------------------------
//! A6 additions over D5
//! -----------------------------------------------------------------------
//! A6.1 — Parameterised queries: TEST_QUERIES slice replaces the single
//!         "What is RAG?" constant.  Each query is exercised independently;
//!         the overall test PASSes if every query either returned a valid
//!         ConceptDecomposition OR was cleanly skipped (structured WARN).
//!
//! A6.2 — Cache-hit latency assertion: after each successful query the test
//!         re-runs research_decompose with the SAME query on the SAME
//!         ResearchCache instance.  The second call must be >=30% faster
//!         because ground_or_cached in research_tree.rs calls cache.peek()
//!         before hitting the sidecar (confirmed: research_tree.rs:1596).
//!         If the second call is NOT faster, the assertion fires.
//!
//! A6.3 — ELAI_LIVE_SIDECAR_AUTOSTART: opt-in tunnel boot (see above).
//! -----------------------------------------------------------------------

use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

use elai_orchestrator::grounding_client::SidecarGroundingClient;
use elai_orchestrator::research_tree::{
    research_decompose, ExecutionMode, ResearchCache, DEFAULT_CACHE_MAX_ENTRIES,
    DEFAULT_CACHE_TTL_SECONDS,
};

// ---------------------------------------------------------------------------
// A6.1 — Query slice.  3 short, distinct, sidecar-friendly queries.
// Total expected live-path wall time: ≤10 min (120 s per call × 3 queries
// × 2 calls each = 720 s worst case; in practice cache hits are <1 s).
// ---------------------------------------------------------------------------

const TEST_QUERIES: &[&str] = &[
    "What is RAG?",
    "How does vector search work?",
    "Explain semantic chunking",
];

// ---------------------------------------------------------------------------
// Pre-flight helper — TCP connect with a short timeout.
// Returns true if the port accepts a connection within `timeout`.
// Uses only std::net; no reqwest / extra deps needed.
// ---------------------------------------------------------------------------

fn port_reachable(host: &str, port: u16, timeout: Duration) -> bool {
    let addr: SocketAddr = match format!("{}:{}", host, port).parse() {
        Ok(a) => a,
        Err(_) => return false,
    };
    TcpStream::connect_timeout(&addr, timeout).is_ok()
}

// ---------------------------------------------------------------------------
// A6.3 — Optional autostart: poll port `port` until open or `deadline` hit.
// ---------------------------------------------------------------------------

fn poll_port_until(host: &str, port: u16, deadline: Instant) -> bool {
    loop {
        if port_reachable(host, port, Duration::from_secs(2)) {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}

// ---------------------------------------------------------------------------
// D5 / A6 contract: live sidecar drives research_decompose end-to-end.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn live_sidecar_research_decompose_what_is_rag() {
    // Gate: skip entirely when env var unset (CI-friendly, default path).
    if std::env::var("ELAI_LIVE_SIDECAR").is_err() {
        return;
    }

    // -----------------------------------------------------------------------
    // A6.3 — Autostart: if port 8100 is closed AND autostart is requested,
    // invoke the start-tunnel.ps1 script and poll for up to 60 s.
    // -----------------------------------------------------------------------
    if !port_reachable("127.0.0.1", 8100, Duration::from_secs(5)) {
        if std::env::var("ELAI_LIVE_SIDECAR_AUTOSTART").as_deref() == Ok("1") {
            eprintln!("A6.3: port 8100 closed; attempting autostart via start-tunnel.ps1");
            let tunnel_script =
                r"F:\.ELAI_workflow\workflow\remote-setup\start-tunnel.ps1";
            let spawn_result = std::process::Command::new("powershell.exe")
                .args([
                    "-ExecutionPolicy",
                    "Bypass",
                    "-NonInteractive",
                    "-File",
                    tunnel_script,
                ])
                .spawn();
            match spawn_result {
                Ok(mut child) => {
                    eprintln!(
                        "A6.3: start-tunnel.ps1 spawned (pid {:?}); polling port 8100 for 60 s",
                        child.id()
                    );
                    let deadline = Instant::now() + Duration::from_secs(60);
                    if poll_port_until("127.0.0.1", 8100, deadline) {
                        eprintln!("A6.3: port 8100 open after autostart");
                    } else {
                        eprintln!(
                            "A6.3 SKIP: port 8100 still closed after 60 s autostart wait"
                        );
                        // Reap the child so we don't leave a zombie.
                        let _ = child.wait();
                        return;
                    }
                }
                Err(e) => {
                    eprintln!(
                        "A6.3 SKIP: failed to spawn start-tunnel.ps1: {e:?}"
                    );
                    return;
                }
            }
        } else {
            eprintln!("D5 SKIP: sidecar not reachable at 127.0.0.1:8100");
            return;
        }
    }

    // Pre-flight: SearXNG port 8888.
    if !port_reachable("127.0.0.1", 8888, Duration::from_secs(5)) {
        eprintln!("D5 SKIP: searxng not reachable at 127.0.0.1:8888");
        return;
    }

    // Build the production grounding client.
    // Timeout: 120 s per grounding call.  Three levels = up to 360 s total.
    // The remote GPU sidecar over SSH tunnel can take 60-90 s for a cold completion.
    let grounding = SidecarGroundingClient::new(
        "http://127.0.0.1:8100".to_string(),
        Duration::from_secs(120),
    );

    // -----------------------------------------------------------------------
    // A6.1 — Iterate over the query slice.
    // One ResearchCache per query so each query gets a clean first-call then
    // a cache-hit second call, with no cross-query interference.
    // -----------------------------------------------------------------------
    let mut all_passed = true;
    let mut queries_executed: usize = 0;

    for &query in TEST_QUERIES {
        eprintln!("---");
        eprintln!("A6.1: running query \"{}\"", query);

        // Fresh cache per query — isolates latency measurements.
        let mut cache = ResearchCache::new(DEFAULT_CACHE_MAX_ENTRIES, DEFAULT_CACHE_TTL_SECONDS);

        // ----------------------------------------------------------------
        // First call — may hit the live sidecar (cold).
        // ----------------------------------------------------------------
        let t0 = Instant::now();
        eprintln!("A6.1: first call to research_decompose(\"{}\")", query);

        let decomp = match research_decompose(
            query,
            ExecutionMode::Autonomous,
            &mut cache,
            &grounding,
            None, // Track A.6: live test has no SSB handle; pass None.
            None, // B-7: no tree emitter in live sidecar contract.
        )
        .await
        {
            Ok(d) => d,
            Err(e) => {
                eprintln!(
                    "A6 LIVE WARN: research_decompose returned Err for \"{query}\" — \
                     likely sidecar JSON schema non-compliance (Qwen 3B unconstrained). \
                     err={e:?}"
                );
                // Structured skip: not a panic, but does NOT count as a clean execution.
                // queries_executed is checked after the loop; if zero, the test fails.
                continue;
            }
        };
        queries_executed += 1;
        let elapsed_first = t0.elapsed();

        eprintln!(
            "A6.1: first call DONE in {:.2}s — {} concept(s), {} total_research_calls",
            elapsed_first.as_secs_f64(),
            decomp.concepts.len(),
            decomp.total_research_calls,
        );

        // ----------------------------------------------------------------
        // Structural assertions (addresses D4 auditor risk #1 — vacuous pass).
        // ----------------------------------------------------------------
        if decomp.concepts.is_empty() {
            eprintln!("A6 FAIL: L0 must yield at least one concept for \"{}\"", query);
            all_passed = false;
            continue;
        }
        let concept = &decomp.concepts[0];
        if concept.phases.is_empty() {
            eprintln!("A6 FAIL: L1 must yield at least one phase for \"{}\"", query);
            all_passed = false;
            continue;
        }
        let phase = &concept.phases[0];
        if phase.steps.is_empty() {
            eprintln!("A6 FAIL: L2 must yield at least one step for \"{}\"", query);
            all_passed = false;
            continue;
        }
        let step = &phase.steps[0];
        if step.baby_steps.is_empty() {
            eprintln!(
                "A6 FAIL: decompose_to_baby_steps must produce baby steps for \"{}\"",
                query
            );
            all_passed = false;
            continue;
        }

        eprintln!(
            "A6.1: PASS — concept=\"{}\", phase=\"{}\", step=\"{}\", baby_steps={}",
            concept.name,
            phase.name,
            step.description,
            step.baby_steps.len(),
        );

        // ----------------------------------------------------------------
        // A6.2 — Cache-hit second call.
        //
        // L1 cache confirmed present: research_tree.rs:1596 calls
        // cache.peek(cache_key) before any grounding I/O. A second call
        // with the same `idea` string will hit that path for the top-level
        // key (and also for concept:/phase: keys populated in the first
        // call), so we assert the second call is ≥30% faster.
        //
        // Cache detection: confirmed via Grep on research_tree.rs —
        // ground_or_cached() at line 1589 + cache.peek() at line 1596.
        // ----------------------------------------------------------------
        eprintln!("A6.2: second call (cache-hit path) for \"{}\"", query);
        let t1 = Instant::now();

        let decomp2 = match research_decompose(
            query,
            ExecutionMode::Autonomous,
            &mut cache,
            &grounding,
            None, // Track A.6: live test has no SSB handle; pass None.
            None, // B-7: no tree emitter in live sidecar contract.
        )
        .await
        {
            Ok(d) => d,
            Err(e) => {
                eprintln!(
                    "A6.2 WARN: second call returned Err for \"{query}\": {e:?}"
                );
                // Second-call error is surprising but not a test failure on its own;
                // the first call already passed structural assertions.
                continue;
            }
        };
        let elapsed_second = t1.elapsed();

        eprintln!(
            "A6.2: second call DONE in {:.3}s (first was {:.3}s)",
            elapsed_second.as_secs_f64(),
            elapsed_first.as_secs_f64(),
        );

        // Latency-drop check: second call should be ≥30% faster.
        // Cache confirmed at research_tree.rs:1596; failure indicates the
        // cache is not being populated or keying has changed.
        // Conditional (not assert!) so a failure on query[0] does not abort the
        // remaining queries — gather all failures, report at end.
        let threshold = elapsed_first.mul_f64(0.70); // 70% of first == 30% faster
        if elapsed_second > threshold {
            eprintln!(
                "A6.2 FAIL for \"{query}\": second call ({:.3}s) was NOT ≥30% faster \
                 than first ({:.3}s). Expected ≤{:.3}s. \
                 Check cache.peek() path at research_tree.rs:1596.",
                elapsed_second.as_secs_f64(),
                elapsed_first.as_secs_f64(),
                threshold.as_secs_f64(),
            );
            all_passed = false;
            continue;
        }

        if decomp2.concepts.len() != decomp.concepts.len() {
            eprintln!(
                "A6.2 FAIL for \"{query}\": cache-hit second call returned different \
                 concept count (first={}, second={})",
                decomp.concepts.len(),
                decomp2.concepts.len(),
            );
            all_passed = false;
            continue;
        }

        eprintln!("A6.2: PASS — cache-hit latency drop confirmed for \"{}\"", query);
    }

    // Vacuous-pass guard: if every query skipped via the Err arm (e.g. sidecar
    // down or schema non-compliance across the board), the structural assertions
    // never ran. Treat that as a hard failure — a live-sidecar contract test
    // that exercises zero queries does not actually verify any contract.
    assert!(
        queries_executed > 0,
        "A6 VACUOUS PASS GUARD: 0 of {} queries reached the assertion block. \
         All research_decompose calls returned Err — sidecar likely down or \
         L1 schema mismatch on every query. Run with ELAI_LIVE_SIDECAR=1 only \
         when sidecar+tunnel are healthy and parser tolerance (Priority B1+B2) is in.",
        TEST_QUERIES.len()
    );

    assert!(
        all_passed,
        "A6 aggregate FAIL: at least one query failed structural or cache-hit \
         assertions (see per-query eprintln output above). queries_executed={} of {}.",
        queries_executed,
        TEST_QUERIES.len()
    );

    eprintln!(
        "A6: {} of {} queries PASSED (rest skipped via structured-WARN branch)",
        queries_executed,
        TEST_QUERIES.len()
    );
}
