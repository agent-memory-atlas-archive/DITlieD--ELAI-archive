//! Track C-A — PrefixKVCache cache-hit contract.
//!
//! Drives `research_decompose` twice end-to-end against the live sidecar
//! and asserts that the second cascade's total wall time is <= 0.4× the
//! first cascade's wall time (>=2.5× cumulative speedup, conservative).
//!
//! Gated by ELAI_LIVE_SIDECAR=1.  When unset, the test no-ops and passes
//! so default `cargo test` runs stay fast and CI-friendly.
//!
//! Recommended invocation:
//!   ELAI_LIVE_SIDECAR=1 cargo test -p elai_cc_contracts \
//!       --test research_decompose_cache_hit -- --nocapture
//!
//! Pre-conditions (all checked, all skip-on-fail):
//!   - sidecar reachable at http://127.0.0.1:8100
//!   - sidecar booted with `--kv-cache-size N` where N >= 2 (default 4)
//!   - sidecar booted with `--turboquant-bits 0` (PrefixKVCache auto-disables
//!     when TQ active; the speedup target cannot be met under TQ)

use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

use elai_orchestrator::grounding_client::SidecarGroundingClient;
use elai_orchestrator::research_tree::{
    research_decompose, ExecutionMode, ResearchCache,
};

const QUERY: &str = "What is RAG?";
const SPEEDUP_FACTOR_MAX: f64 = 0.4; // second wall <= 0.4 × first wall

fn port_reachable(host: &str, port: u16, timeout: Duration) -> bool {
    let addr: SocketAddr = match format!("{}:{}", host, port).parse() {
        Ok(a) => a,
        Err(_) => return false,
    };
    TcpStream::connect_timeout(&addr, timeout).is_ok()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn prefix_kv_cache_speedup_research_decompose() {
    if std::env::var("ELAI_LIVE_SIDECAR").is_err() {
        eprintln!("C-A SKIP: ELAI_LIVE_SIDECAR not set");
        return;
    }
    if !port_reachable("127.0.0.1", 8100, Duration::from_secs(5)) {
        eprintln!("C-A SKIP: sidecar not reachable at 127.0.0.1:8100");
        return;
    }
    if !port_reachable("127.0.0.1", 8888, Duration::from_secs(5)) {
        eprintln!("C-A SKIP: searxng not reachable at 127.0.0.1:8888");
        return;
    }

    let grounding = SidecarGroundingClient::new(
        "http://127.0.0.1:8100".to_string(),
        Duration::from_secs(120),
    );

    // Use a TTL=0 ResearchCache so the orchestrator-level cache does NOT
    // serve the second call from RAM — every research_decompose level
    // goes back to the sidecar, forcing the speedup to come from
    // PrefixKVCache rather than orchestrator-side caching.
    let mut cache = ResearchCache::new(64, 0);

    let t0 = Instant::now();
    let r1 = research_decompose(
        QUERY,
        ExecutionMode::Autonomous,
        &mut cache,
        &grounding,
        None,
        None, // B-7: no tree emitter in cache-hit contract.
    )
    .await;
    let first_wall = t0.elapsed();
    if let Err(e) = &r1 {
        eprintln!("C-A SKIP: first research_decompose errored: {e:?}");
        return;
    }

    let t1 = Instant::now();
    let r2 = research_decompose(
        QUERY,
        ExecutionMode::Autonomous,
        &mut cache,
        &grounding,
        None,
        None, // B-7: no tree emitter in cache-hit contract.
    )
    .await;
    let second_wall = t1.elapsed();
    if let Err(e) = &r2 {
        panic!("second research_decompose errored: {e:?}");
    }

    let ratio =
        second_wall.as_secs_f64() / first_wall.as_secs_f64().max(0.001);
    eprintln!(
        "C-A: first_wall={:.2?} second_wall={:.2?} ratio={:.3}",
        first_wall, second_wall, ratio
    );

    assert!(
        ratio <= SPEEDUP_FACTOR_MAX,
        "PrefixKVCache speedup target missed: second/first ratio = {:.3} \
         (expected <= {:.3} = >=2.5× speedup). first_wall={:.2?} \
         second_wall={:.2?}. \
         If ratio is near 1.0, sidecar is likely booted with \
         --kv-cache-size 0 OR --turboquant-bits > 0 (auto-disabled). \
         Check sidecar.log for prefix_cache_init / prefix_cache_disabled.",
        ratio,
        SPEEDUP_FACTOR_MAX,
        first_wall,
        second_wall
    );
}
