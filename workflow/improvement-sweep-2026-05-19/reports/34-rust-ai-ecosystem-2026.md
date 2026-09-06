# Rust AI ecosystem (May 2026)

## TL;DR
- ort is STILL on 2.0.0-rc.12 (Mar 5 2026, ~1.24M dl/mo). The 2.0 train has had release candidates since 2024 with no stable yet. CLAUDE.md anti-stale rule "ort stable not -rc" cannot be honored in 2026; either accept rc.12 (it is what every production caller uses, including SurrealDB / Magika / Bloop / wasmtime WASI-NN) or pin 1.13.1 (Nov 2022, abandoned). Update the rule.
- rmcp 1.7.0 (May 13 2026, ~2.1M dl/mo) is now the canonical Rust MCP SDK and the #1 ML crate on lib.rs. ELAI is currently consuming MCP only as a client into local-rag — wire `rmcp` for any future ELAI-exposed MCP surfaces instead of hand-rolling JSON-RPC.
- For LLM-client-as-library: replace any hand-rolled provider client with `genai` 0.6 (Jeremy Chone, 24k dl/mo, 20+ providers including Anthropic/Gemini/Groq/xAI/Ollama, one trait). `rig-core` 0.37 (275k dl/mo) is heavier but gives the agent framework + 10 vector stores in the same crate.
- Drop any `bincode` / hand-rolled binary format in the IPC fastpath: `rkyv` wins djkoloski's benchmark for nearly every payload (zero-copy, ~rustc 1.97 nightly 2026-04-23). Keep `postcard` for size-bounded embedded/socket cases (it beats bincode on size + zlib).
- `swiftide` is stale (0.32.1 last release Nov 15 2025, ~57 dl/mo on lib.rs — effectively dormant). Do NOT adopt for new code in ELAI. Use rig or hand-roll.

## Key sources
| URL | crate/repo | takeaway |
|---|---|---|
| https://lib.rs/crates/rmcp | rmcp 1.7.0 | official MCP SDK, 2.1M dl/mo, Rust 2024, Apache-2.0 |
| https://github.com/modelcontextprotocol/rust-sdk | rust-sdk | official repo, tokio async, MCP spec 2024-11-05 |
| https://lib.rs/crates/rig-core | rig-core 0.37.0 | 275k dl/mo, 20+ providers, 10+ vector stores, OTel GenAI |
| https://github.com/0xPlaygrounds/rig | rig | agent + RAG + structured extraction + streaming in one |
| https://lib.rs/crates/mistralrs | mistralrs 0.8.1 | 9.7k dl/mo, candle-based, ISQ + PagedAttention + FA |
| https://github.com/EricLBuehler/mistral.rs | mistral.rs | multimodal text/vision/audio, agentic loop, replaces archived `llm` crate |
| https://lib.rs/crates/candle-core | candle-core 0.10.2 | HF, 565k dl/mo, 864 dependent crates, NCCL distributed |
| https://lib.rs/crates/ort | ort 2.0.0-rc.12 | ONNX Runtime 1.24 wrapper, 1.24M dl/mo, STILL rc |
| https://lib.rs/crates/fastembed | fastembed 5.13.4 | 258k dl/mo, sync API (no tokio), uses pykeio/ort |
| https://lib.rs/crates/tantivy | tantivy 0.26.1 | 946k dl/mo, BM25, ~2x faster than Lucene |
| https://lib.rs/crates/async-openai | async-openai 0.38.2 | 593k dl/mo, retry+backoff, WASM, Tower middleware |
| https://lib.rs/crates/genai | genai 0.6.0-beta.20 | 24k dl/mo, one trait for 20+ providers (Anthropic, Gemini, Groq, xAI, Ollama, DeepSeek, Copilot) |
| https://github.com/jeremychone/rust-genai | rust-genai | the multiprovider crate, 48 contributors |
| https://lib.rs/crates/swiftide | swiftide 0.32.1 | 57 dl/mo, last release Nov 2025 — dormant, skip |
| https://github.com/bosun-ai/swiftide | swiftide repo | streaming agents 0.26 |
| https://david.kolo.ski/blog/rkyv-is-faster-than/ | rkyv | wins almost every category vs bincode/capnp/postcard/prost |
| https://oxc.rs/docs/guide/benchmarks | oxc | 3x faster than swc, 5x faster than Biome on JS/TS |
| https://medium.com/@hchan_nvim/benchmark-typescript-parsers-demystify-rust-tooling-performance-025ebfd391a3 | oxc/swc/tree-sitter | serialization cost dominates JS-side AST cost |
| https://v2.tauri.app/security/ | Tauri v2 | Isolation pattern + Capabilities ACL for IPC trust boundary |
| https://github.com/pepperoni21/ollama-rs | ollama-rs 0.3.4 | Feb 12 2026, simple HTTP wrapper, well-maintained |
| https://corrode.dev/blog/async/ | tokio vs smol | ecosystem has consolidated on tokio; smol for library/embedded |
| https://crates.io/crates/anthropic-sdk-rust | anthropic-sdk-rust | unofficial, full TS-SDK parity |
| https://crates.io/crates/adk-anthropic | adk-anthropic | Mar 29 2026, adaptive thinking + effort + structured outputs |

## Crate landscape table
| domain | crate | last-publish | downloads/month | maturity | anti-stale OK? | citation |
|---|---|---|---|---|---|---|
| MCP SDK | rmcp | 2026-05-13 | ~2.1M | production, official | YES | lib.rs/crates/rmcp |
| Agent FW | rig-core | 2026-05-13 | 275k | production, opinionated | YES | lib.rs/crates/rig-core |
| Agent FW | swiftide | 2025-11-15 | 57 | DORMANT | NO (stale) | lib.rs/crates/swiftide |
| Local LLM | mistralrs | 2026-04-02 | 9.7k | active, advanced | YES | lib.rs/crates/mistralrs |
| Local LLM | candle-core | 2026-04-01 | 565k | mature, baseline | YES | lib.rs/crates/candle-core |
| Local LLM | kalosm-language | — | mid-tier | active, high-level wrapper over candle | YES | lib.rs/crates/kalosm-language |
| ONNX | ort | 2026-03-05 | 1.24M | rc.12 ONLY, no stable 2.x | NO per current CLAUDE.md rule (see TL;DR) | lib.rs/crates/ort |
| Embeddings | fastembed | 2026-04-27 | 258k | production, sync, no-tokio | YES | lib.rs/crates/fastembed |
| BM25 | tantivy | 2026-04-21 | 946k | mature | YES | lib.rs/crates/tantivy |
| LLM client | async-openai | 2026-05-14 | 594k | mature unofficial | YES | lib.rs/crates/async-openai |
| LLM multi | genai | 2026-05-16 | 24k | beta but active | YES (semver beta) | lib.rs/crates/genai |
| LLM client | anthropic-sdk-rust | active | mid | unofficial, parity | YES | crates.io/crates/anthropic-sdk-rust |
| LLM client | adk-anthropic | 2026-03-29 | low | parity w/ March 2026 API | YES | crates.io/crates/adk-anthropic |
| Local LLM | ollama-rs | 2026-02-12 | mid | simple HTTP wrapper | YES | github.com/pepperoni21/ollama-rs |
| Serialization | rkyv | active | high | production, zero-copy | YES (already complementing postcard) | david.kolo.ski/blog |
| Serialization | postcard | active | high | embedded-friendly | YES (already in ELAI) | djkoloski/rust_serialization_benchmark |
| Parser | tree-sitter | active | very high | de facto | YES (already in ELAI) | github.com/tree-sitter/tree-sitter |
| Parser TS/JS | oxc | active | growing | 3x swc | YES | oxc.rs/docs/guide/benchmarks |
| Parser TS/JS | swc | mature | very high | de facto for transpile | YES | medium TS parser benchmark |
| FSM | smlang | active | mid | DSL, no_std, async | YES | crates.io/crates/smlang |
| FSM | rust-fsm | active | mid | minimal | YES | github.com/eugene-babichenko/rust-fsm |
| Web/sidecar | axum | active | very high | tokio-native, Tower | YES | aarambhdevhub medium 2026 |
| Web/sidecar | actix-web | active | very high | 10-15% faster on heavy load | YES (rule allows either) | fastbuilder.ai 2026 |
| Async runtime | tokio | active | dominant | de facto | YES | corrode.dev async |
| Test | proptest | active | high | property-based + shrinking | YES | github.com/proptest-rs/proptest |
| Test | criterion | active | high | statistical micro-bench | YES | dasroot 2026 |
| Test | insta | active | high | snapshot, -30% flakiness vs raw asserts | YES | dasroot 2026 |

## Where Rust wins for AI dev
- Inference deployment binary size + cold start. Candle's stated motivation is "reduce binary bloat to enable serverless"; ort delivers 13x smaller serving image and 3-5x faster inference vs Python with 60-80% less memory in production reports (Bloop, SurrealDB, Google Magika, wasmtime WASI-NN). [calmops.com/programming/rust/real-time-ml-model-development-with-rust-and-onnx-runtime/, github.com/pykeio/ort]
- IPC and zero-copy paths: rkyv + postcard are not matched by anything in the Python/TS stacks; rkyv wins nearly every payload type vs bincode/capnp/cbor/flatbuffers/postcard/prost on the canonical benchmark. [david.kolo.ski/blog/rkyv-is-faster-than/]
- Agent framework concurrency: 2026 ecosystem reports note 5x memory reduction and 25-44% latency improvement over Python agent frameworks plus orders-of-magnitude cold-start advantage. [zylos.ai/research/2026-04-01-rust-native-ai-agent-frameworks-ecosystem-2026, ossinsight.io/blog/rust-ai-agent-infrastructure-2026]
- BM25 retrieval: tantivy ~2x Lucene latency, <10 ms startup, used by Etsy/ParadeDB/Nuclia. [lib.rs/crates/tantivy]
- Tauri v2 IPC isolation: Capabilities ACL + Isolation pattern provide a real trust-boundary story that Electron/JS-only stacks do not. [v2.tauri.app/security/]

## Where Rust loses to Python/TS
- Model R&D: PyPI 530k packages vs crates.io ~155k; no Rust equivalent of HF Transformers training stack, PyTorch nn, NumPy/Pandas, peft, accelerate. Training stays in Python. [tech-insider.org/python-vs-rust-2026, dev.to/thedailyagent/python-vs-go-vs-rust-for-ai-agents-in-2026]
- Quantization toolchains (bnb 4-bit, AWQ, GPTQ-int4, EXL2, MLX): all Python-first. Anti-stale rule already concedes "bnb 4-bit OK (Python sidecar only)". Same applies to FA2 — Rust has no first-class FA2 path, mistral.rs leans on its own kernel set; for production-grade FA2 Python sidecar remains canonical. [github.com/EricLBuehler/mistral.rs]
- Rapid prototyping velocity: Rust build time + borrow-checker cost is real. AI-assisted codegen quality is measurably higher in TS/Python than Rust per 2026 dev-tooling surveys. [predict.codes/blog/programming-predictions-2026, medium "Rust and TypeScript are best languages for AI-Assisted development"]
- Notebook + viz: no Jupyter/Polars-notebook equivalent at the maturity tier (evcxr exists but is hobby-grade).
- ONNX 2.x stability: ort 2.0.0-rc.12 since 2024, no stable. Python's onnxruntime is GA.

## Best new adoptions for ELAI
1. rmcp 1.7.0 | github.com/modelcontextprotocol/rust-sdk + lib.rs metrics | replaces any hand-rolled MCP server/client surface ELAI exposes; ELAI already speaks MCP through local-rag client, but if ELAI's CLI ever exposes itself as an MCP server (so other agents can call it), rmcp is the only sane choice. Apache-2.0, official.
2. genai 0.6 (beta) | lib.rs/crates/genai | replaces hand-rolled provider client code in CloudGroundingClient (per recent commits "B3 cloud-mode variant"). One trait covers Anthropic + Gemini + Groq + xAI + Ollama + DeepSeek + Copilot. Caveat: beta semver, pin exact tag.
3. mistralrs 0.8 (when local-Rust path needed) | github.com/EricLBuehler/mistral.rs | replaces archived `llm` crate, sits on candle, adds ISQ + PagedAttention + agentic loop. ELAI's local path is Python sidecar today; mistralrs is the in-process Rust alternative for a future "embedded model" mode.
4. fastembed 5.13 | lib.rs/crates/fastembed | replaces any custom embedding shim around ort + tokenizers. Sync API, no tokio dependency — fits ELAI's tantivy indexing path cleanly. Uses pykeio/ort under the hood (already in ELAI's dependency tree transitively).
5. ollama-rs 0.3.4 | github.com/pepperoni21/ollama-rs | optional fallback transport for local Ollama users; simpler than full genai if Ollama is the only target.
6. oxc | oxc.rs | additional TypeScript/JavaScript parser path beyond tree-sitter for higher fidelity when ELAI's symbol_graph indexes Tauri Svelte/TS sources. 3x swc, 5x Biome on parse speed.
7. rkyv expansion | david.kolo.ski/blog | promote rkyv from "we know about it" to the chosen format for symbol_graph snapshots + tantivy segment metadata caches. Postcard stays for size-bounded IPC frames.
8. insta + proptest | dasroot.net 2026 testing posts | adopt insta snapshots for SSB event-payload regression tests and proptest for the deterministic-replay invariants in the FSM. -30% test flakiness reported.

## What to drop
- "ort stable not -rc" rule as a hard rule. Ground truth as of 2026-05-13: there IS no stable ort 2.x and rc.12 is what every production user runs. Re-word to "ort 2.x rc track is acceptable; pin exact rc; do NOT use ort 1.13.1 (abandoned 2022)". [lib.rs/crates/ort]
- swiftide (if it was ever on the candidate list). 57 dl/mo, no release in 6 months. The agent-framework alternative is rig. [lib.rs/crates/swiftide]
- Any reference to the archived `llm` crate (last version 0.1.1, archived). Replace with mistralrs or candle direct. [search result: "mistral.rs has replaced the old llm crate (now archived)"]
- Plain bincode in the hot path. Replace with rkyv (zero-copy) or keep postcard for size-critical paths. [david.kolo.ski/blog]
- Any consideration of async-std. The ecosystem consolidated on tokio; async-std is deprecated upstream. [corrode.dev/blog/async/, weeklyrust.substack.com Goodbye Async-Std]

## Anti-patterns
- Hand-rolling provider clients per provider. The cost is real (auth refresh, retry semantics, streaming SSE parsing, tool-call schema variance across Anthropic/OpenAI/Gemini). genai or rig solves it. [jeremychone/rust-genai README]
- Using async-trait macro for new code. Native async fn in traits is stable; the macro adds Box<dyn> overhead and is in the CLAUDE.md anti-stale list. [Rust 1.75+ docs]
- Mixing typestate FSM with runtime FSM in the same module: the practical pattern is typestate for core logic + enum wrapper for persistence/serialization. ELAI's mcp_fsm_get_state is already enum-wrapper; do not retrofit typestate into existing transitions. [medium.com Generic Finite State Machines]
- Treating Candle and ort as interchangeable. Candle = research/training + serverless WASM; ort = production deployed-model inference. ELAI's symbol_graph embedding pass should use ort (via fastembed); a future training/fine-tune helper should use candle. [dev.to mayu2008 sentence-transformers]
- Running Tauri IPC without Capability ACL. Tauri v2 requires explicit capabilities per window; default-allow is a real attack surface. [v2.tauri.app/security/capabilities/]

## Open questions
1. Does ELAI's CloudGroundingClient (per patch-R2 commit fd5c2e5) currently retry/backoff/refresh OAuth the way async-openai / genai already do? If not, migration cost = how many lines?
2. Measured ELAI symbol_graph indexing latency on a 50k-file repo with current ort/candle path vs fastembed 5.13 — is the sync-no-tokio API a win or a regression vs the existing async pipeline?
3. Should ELAI expose itself as an MCP server (rmcp) so the user can drive ELAI from Claude Desktop / Cursor / other MCP clients? If yes, what subset of CLI verbs maps cleanly to MCP tools?
4. rkyv adoption in the SSB / dashboard IPC channel: measured bytes-on-wire and decode-latency delta vs current postcard. If <2x improvement, skip — postcard is already shipped.
5. swiftide is dormant but the underlying bosun-ai org is the same one publishing async-anthropic. Is the agent piece migrating to a successor crate or is the whole project abandoned? (Affects whether any swiftide-derived patterns ELAI inherited need replacement.)
