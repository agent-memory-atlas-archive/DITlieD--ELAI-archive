// Session 4D.11: contract tests for SearxSearchBackend — production
// SearchBackend impl wrapping SearxClient against the sidecar `/search` proxy.
//
// We hand-roll a local TCP stub (no new HTTP-mock dependency per CLAUDE.md
// "No new dependencies without confirmation") that speaks the minimum of
// HTTP/1.1 needed to satisfy reqwest's GET /search?q=...&format=json.

use elai_orchestrator::{SearchBackend, SearxSearchBackend};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

// HTTP response served to the first accepted connection. SearXNG contract:
// {"results": [{"title": ..., "url": ..., "content": ..., "engine": ..., "score": ...}]}.
const SEARX_OK_BODY: &str = r#"{"query":"rust","number_of_results":2,"results":[{"title":"Rust Book","url":"https://doc.rust-lang.org/book/","content":"<b>Rust</b> is a systems language","engine":"duckduckgo","score":1.0},{"title":"What is Rust?","url":"https://stackoverflow.com/q/1","content":"Rust safety","engine":"bing","score":0.8}]}"#;

/// Spawn a one-shot HTTP stub on 127.0.0.1:0 that replies with `body` once,
/// then closes. Returns the bound base URL (e.g., "http://127.0.0.1:54321").
async fn spawn_one_shot_stub(body: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    let base_url = format!("http://{}", addr);

    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            // Drain request (best-effort) until we see end of headers.
            let mut buf = [0u8; 4096];
            let _ = socket.read(&mut buf).await;

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body,
            );
            let _ = socket.write_all(response.as_bytes()).await;
            let _ = socket.shutdown().await;
        }
    });

    base_url
}

#[test]
fn searx_backend_constructs_with_default_url() {
    let backend = SearxSearchBackend::new(elai_orchestrator::DEFAULT_SIDECAR_SEARCH_URL);
    assert!(backend.is_ok(), "default URL must construct cleanly");
}

#[test]
fn searx_backend_is_dyn_safe() {
    // Confirms SearchBackend trait can be used via trait objects (main.rs relies
    // on Box<dyn SearchBackend> for the stub/searx switch at the call site).
    let backend: Box<dyn SearchBackend> = Box::new(
        SearxSearchBackend::new("http://127.0.0.1:1").expect("constructs"),
    );
    // We do NOT call .search() here (would block on a real connect attempt).
    // Existence of the Box<dyn SearchBackend> is the contract we care about.
    drop(backend);
}

#[test]
fn searx_backend_returns_results_from_local_http_stub() {
    // Dedicated runtime for the stub server; SearxSearchBackend owns a
    // separate current-thread runtime internally for its blocking search call.
    let stub_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(1)
        .build()
        .expect("stub runtime");

    let base_url = stub_rt.block_on(spawn_one_shot_stub(SEARX_OK_BODY));

    let backend = SearxSearchBackend::new(base_url).expect("construct backend");
    let results = backend.search("rust", 10).expect("search");

    assert_eq!(results.len(), 2, "stub returns 2 results");
    assert_eq!(results[0].url, "https://doc.rust-lang.org/book/");
    assert_eq!(results[0].title, "Rust Book");
    // snippet must have been HTML-stripped by web_search::strip_html.
    assert!(!results[0].snippet.contains('<'));
    assert!(results[0].snippet.contains("Rust"));
    assert_eq!(results[1].url, "https://stackoverflow.com/q/1");
}

#[test]
fn searx_backend_truncates_to_max_results() {
    let stub_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(1)
        .build()
        .expect("stub runtime");
    let base_url = stub_rt.block_on(spawn_one_shot_stub(SEARX_OK_BODY));

    let backend = SearxSearchBackend::new(base_url).expect("construct");
    // Stub returns 2; request max 1 -> expect 1.
    let results = backend.search("rust", 1).expect("search");
    assert_eq!(results.len(), 1);
}

#[test]
fn searx_backend_surfaces_error_on_unreachable_host() {
    // Reserved discard port 1 — reliably refuses connections on localhost.
    let backend = SearxSearchBackend::new("http://127.0.0.1:1").expect("constructs");
    let err = backend.search("rust", 5).expect_err("must fail when unreachable");

    match err {
        elai_core::research_types::ResearchPipelineError::SearchBackend(msg) => {
            assert!(msg.to_lowercase().contains("searx"), "error text: {msg}");
        }
        other => panic!("expected SearchBackend error, got {other:?}"),
    }
}
