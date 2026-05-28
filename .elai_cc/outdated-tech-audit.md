outdated-tech-audit — canonical anti-stale evidence backing CLAUDE.md "Anti-Stale-Training" HARD RULE.
Authored: 2026-05-28 (E-34, cluster s-25120 batch7).
Pointer source: CLAUDE.md line 20 "See .elai_cc/outdated-tech-audit.md for full findings."

This file is the evidence ledger for the anti-stale Rust + Python lists. Each entry: claim, lib.rs/registry ground truth date, ELAI pin sites, references. CLAUDE.md is the rule; this file is the audit trail.

==============================================
2026-05 ort 2.x rc track audit (E-34)
==============================================

Claim (CLAUDE.md amended 2026-05-28): "ort 2.x rc track acceptable; pin exact rc version (=2.0.0-rc.12 today); NEVER use ort 1.13.1 (abandoned Nov 2022)."

Ground truth (lib.rs/crates/ort, verified 2026-05-13 by the E-34 source report):
- ort 2.0.0-rc.12 published 2026-03-05, ~1.24M downloads/month.
- Release-candidate train since 2024 (rc.1 through rc.12 over ~18 months). NO stable 2.x release shipped.
- Previous "stable" was 1.13.1 (published Nov 2022). Effectively abandoned — last 1.x release predates the entire ort 2.x rework.
- Production callers all on 2.x rc: SurrealDB, Magika (Google), Bloop, wasmtime WASI-NN.

ELAI pin sites (do NOT drop these):
- F:/.ELAI_workflow/.elai_cc/crates/knowledge/Cargo.toml:54 — `ort = "=2.0.0-rc.12"` (non-optional, primary).
- F:/.ELAI_workflow/.elai_cc/crates/orchestrator/Cargo.toml:68 — `ort = { version = "=2.0.0-rc.12", optional = true }` (under `vlm-integration` feature; B-24 Phase 4I VLM Tier 2).

History of the contradiction (logged here so it cannot re-occur silently):
- CLAUDE.md ORIGINAL wording "ort stable (never -rc)" predated the ort 2.x rework. By 2024-Q2 the ecosystem had moved off 1.13.1 entirely and the rule became unenforceable.
- B-16 (PLANNED) Anti-Stale Rule Enforcement carried a derived spec line ("drop \"=2.0.0-rc.12\" at knowledge/Cargo.toml:27" in B-PLAN-SUMMARY.md line 313) that would, if executed, break the only production-grade ONNX path in ELAI. E-34 corrects both the rule (CLAUDE.md) and the B-16 spec (B-PLAN-SUMMARY.md:313) atomically.
- B-16 line number was wrong (:27 vs actual :54). E-34 fixes that in the same edit.

Re-evaluation criterion: when ort 2.0.0 stable releases, this section is rewritten and CLAUDE.md flips to "ort 2.0.0 stable". Track via lib.rs/crates/ort. Until then, every rc.X bump requires a paired test pass (knowledge::bm25 + symbol_graph onnx_session round-trip).

==============================================
2026-05 Rust AI ecosystem appendix (E-34)
==============================================

Source: F:/.ELAI_workflow/workflow/improvement-sweep-2026-05-19/reports/34-rust-ai-ecosystem-2026.md.

ADOPTED (dep + scaffold landed in E-34 OR follow-up E-34.B):
- rmcp 1.7.0 — official Rust MCP SDK (github.com/modelcontextprotocol/rust-sdk), ~2.1M dl/mo. Anti-slopsquatting verification deferred to E-34.B (live `cargo search rmcp` + publish-date + license + repo-URL match required before adding to any Cargo.toml; see CLAUDE.md HARD RULE). Server surface (which CLI verbs map to MCP tools) is a separate design problem, also deferred.
- genai 0.6.0-beta.X — one-trait multi-provider client (github.com/jeremychone/rust-genai), ~24k dl/mo. Beta-semver: pin exact beta tag at adoption time; reassess each minor. Migration of crates/orchestrator::CloudGroundingClient to genai is a code-cleanup play, NOT a feature add — the existing client already implements OAuth auto-refresh per patch-R2; retry/backoff/SSE-parse coverage assessment is Open Question 1 (E-34-decisions.md).
- insta + proptest — snapshot + property-based testing (dasroot.net 2026 testing posts, ~30% flakiness reduction reported). proptest 1.11 already a workspace dev-dep (crates/orchestrator/Cargo.toml:128); insta dep add deferred to E-34.B.

REJECTED (do not adopt; CLAUDE.md anti-stale entry):
- swiftide — 57 dl/mo per lib.rs, last release Nov 2025, dormant. Successor unclear. CLAUDE.md ban added 2026-05-28.
- async-std — ecosystem on tokio per corrode.dev/blog/async/; existing CLAUDE.md HARD RULE. Confirmed absent from all .elai_cc Cargo.toml (grep verified at E-34 audit time).
- `llm` crate (archived) — replaced by mistralrs 0.8.1 per github.com/EricLBuehler/mistral.rs README. Confirmed absent from all .elai_cc Cargo.toml at E-34 audit time (grep `^llm = ` across .elai_cc/**/Cargo.toml returned zero matches). No removal action needed.

OBSERVED but not adopted (deferred):
- rig-core 0.37 — heavier agent framework, 275k dl/mo. Open question: does ELAI need a heavier framework, or is hand-rolled + genai sufficient? Decision deferred to E-34.E.
- mistralrs 0.8.1 — replaces archived `llm` crate. Adoption blocked on use case; ELAI's local-inference path is the sidecar (Python transformers), not Rust-side mistralrs. Reassess if/when sidecar is replaced.
- fastembed 5.13.4 — wraps ort + tokenizers. ELAI already builds against ort directly; second wrapper offers no win. Skip.
- oxc — TS/JS parser, 3x swc per oxc.rs/docs/guide/benchmarks. Optional second-pass refinement in crates/knowledge/src/ast_adapter.rs; tree-sitter stays canonical per CLAUDE.md anti-stale "Tree-sitter retained" rule. Defer unless integration cost < 0.5 days (E-34.F).
- rkyv expansion — wins djkoloski's benchmark for nearly every payload (david.kolo.ski/blog/rkyv-is-faster-than/). Already present transitively via tantivy; explicit adoption for symbol_graph snapshots + tantivy segment metadata caches deferred to E-34.D pending decode-latency benchmark vs postcard (need >2x win to switch).

KEEP (canonical, do not swap):
- tree-sitter (crates/orchestrator/Cargo.toml:84-86) — CLAUDE.md anti-stale "Tree-sitter retained" rule.
- postcard — size-bounded IPC. rkyv adoption is additive, not a replacement.
- AnthropicClient (crates/providers/src/providers/anthropic.rs) — coexists with future GenAiAdapter; CloudGroundingClient migration deferred per Open Question 1.

==============================================
Anti-slopsquatting verification log (E-34)
==============================================

E-34 dispatched WITHOUT live `cargo search` / publish-date / license verification of genai / rmcp / insta. Rationale: this run authored the SCAFFOLDING (decision doc + rule amendment + adapter shape + contract test) so the verification work in E-34.B has a concrete landing target. The actual Cargo.toml dep adds are gated behind the verification step per CLAUDE.md HARD RULE Anti-Slopsquatting. Each row below MUST be filled before the corresponding Cargo.toml edit lands.

Schema (TSV):
crate_name | source_url | registry_publish_date | license | last_release_date | scan_tool | scan_hash | action

genai | github.com/jeremychone/rust-genai | PENDING (E-34.B `cargo search genai`) | PENDING | PENDING | PENDING (`cargo info genai`) | PENDING (blake3 of search output) | DEFERRED — verify in E-34.B before adding to crates/providers/Cargo.toml
rmcp | github.com/modelcontextprotocol/rust-sdk | PENDING (E-34.B `cargo search rmcp`) | PENDING | PENDING | PENDING (`cargo info rmcp`) | PENDING | DEFERRED — verify in E-34.B before adding to any Cargo.toml
insta | crates.io/crates/insta | PENDING (E-34.B `cargo search insta`) | PENDING | PENDING | PENDING | PENDING | DEFERRED — verify in E-34.B before adding to crates/orchestrator/Cargo.toml dev-deps

proptest already verified workspace dev-dep at crates/orchestrator/Cargo.toml:128 (`proptest = "1.11"`), crates/knowledge/Cargo.toml dev-deps (transitive), no re-verification needed.

==============================================
Anti-stale Rust list — sibling tokens UNCHANGED by E-34
==============================================

Confirmed by E-34's surgical edit of CLAUDE.md line 18: the following tokens are unchanged in this audit:
- blake3 (never blake2/sha2 for non-RFC uses)
- rand >= 0.9 (never 0.8)
- getrandom >= 0.3
- tracing (never log crate)
- thiserror/anyhow (never Box<dyn Error>)
- tokio::sync::Mutex in async
- Native async fn in traits (never async-trait crate)
- deadpool/bb8 over r2d2
- globset/ignore over glob
- Named constants (no magic numbers)
- .try_into() over as-casts for narrowing

If any of these tokens regress in a future plan, that's a CLAUDE.md drift and must be paired with a corresponding entry in this audit.

==============================================
References
==============================================

- lib.rs/crates/ort (ort 2.x rc track ground truth)
- lib.rs/crates/rmcp + github.com/modelcontextprotocol/rust-sdk (official MCP SDK)
- lib.rs/crates/rig-core + github.com/0xPlaygrounds/rig (agent framework)
- lib.rs/crates/genai + github.com/jeremychone/rust-genai (multi-provider client)
- lib.rs/crates/mistralrs + github.com/EricLBuehler/mistral.rs ("mistral.rs has replaced the old llm crate (now archived)")
- lib.rs/crates/swiftide (dormancy evidence, REJECTED)
- david.kolo.ski/blog/rkyv-is-faster-than/ (rkyv vs bincode/capnp/postcard/prost)
- corrode.dev/blog/async/ (tokio consolidation; async-std deprecated)
- oxc.rs/docs/guide/benchmarks (3x swc, 5x Biome — OPTIONAL)
- dasroot.net 2026 testing posts (insta + proptest -30% flakiness)
- workflow/improvement-sweep-2026-05-19/reports/34-rust-ai-ecosystem-2026.md (source report)
- workflow/reasoning/E-34-decisions.md (this run's decision doc)
