D5 Live-Sidecar Grounding Contract Test — Decision Record

WHAT
New file: crates/contracts/tests/research_decompose_live_sidecar_contract.rs
Single #[tokio::test] gated by ELAI_LIVE_SIDECAR env var.
Fixed query "What is RAG?". Asserts ConceptDecomposition structural shape.

KEY DECISIONS

1. Function called: elai_orchestrator::research_tree::research_decompose
   Reason: task spec says "D5: live-sidecar grounding contract test for research_decompose".
   research_decompose is the direct target. run_research_pipeline (research_pipeline.rs) is a
   separate higher-level wrapper; the reference test (research_decompose_real_prompts_contract.rs)
   also calls research_decompose directly. Using the same entry point keeps D5 a faithful
   live counterpart to the existing canned-mock contract.

2. SidecarGroundingClient::new signature: new(base_url: String, timeout: Duration).
   Verified from grounding_client.rs line 129. Takes owned String, not &str.
   Test passes "http://127.0.0.1:8100".to_string().

3. Pre-flight approach: std::net::TcpStream::connect_timeout (pure std, zero new deps).
   reqwest is NOT in contracts Cargo.toml dev-dependencies.
   SidecarGroundingClient::ground() hits /v1/completions and expects valid JSON in return;
   using it for pre-flight would fail if the sidecar returns non-JSON, giving false skip.
   TcpStream::connect_timeout to port 8100 and 8888 is reliable, dependency-free, and
   distinguishes "port open" from "grounding path works".

4. Structural assertions (addresses D4 risk #1 — vacuous pass):
   - decomp.concepts non-empty (L0 produced extractable concepts)
   - concept.phases non-empty (L1 produced extractable phases)
   - phase.steps non-empty (L2 produced extractable steps)
   - step.baby_steps non-empty (decompose_to_baby_steps is deterministic — always fires)
   These four checks together mean the full L0->L1->L2 grounding cascade ran and produced
   parseable output. A fetch-error that silently returns empty sub_results would fail here.

5. Test timeout: #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
   Addresses D4 risk #2. Wall time budget: 3 levels x up to 60s each = 180s max.
   But the sidecar is fast in practice; expected ~30-60s.

6. No new Cargo.toml deps added. std::net::TcpStream::connect_timeout covers pre-flight.

EXPECTED TEST OUTCOME
- ELAI_LIVE_SIDECAR unset: 1 test passes immediately (early return, no network I/O).
- ELAI_LIVE_SIDECAR=1, sidecar+searxng up: test runs ~30-90s, 1 test passes.
- ELAI_LIVE_SIDECAR=1, sidecar down: pre-flight fails, eprintln D5 SKIP, 1 test passes.
- ELAI_LIVE_SIDECAR=1, searxng down: pre-flight fails, eprintln D5 SKIP, 1 test passes.

UNKNOWNS
- Whether the live sidecar's /v1/completions JSON shape exactly matches CompletionsResponse
  {choices:[{text:...}]}. If the sidecar returns Anthropic-protocol JSON instead, parse_research_report_json
  will return InvalidJson and the test will FAIL with that error. This would be a sidecar
  protocol mismatch, not a test bug. The auditor should note this if D5 fails live.
- SearXNG fetcher path: ELAI_FETCHER routing. If no obscura binary, fetch silently degrades.
  The structural assertions still catch empty sub_results from that path.
