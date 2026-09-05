# E-34: Rust AI ecosystem 2026 — anti-stale rule amendment + rmcp adoption + genai consolidation
Objective: amend CLAUDE.md's anti-stale rule that says "ort stable not -rc" since ground truth on 2026-05-13 is that ort 2.x has NO stable release after 18+ months of release candidates and rc.12 (Mar 5 2026, ~1.24M dl/mo) is what every production caller runs (SurrealDB, Magika, Bloop, wasmtime WASI-NN); ELAI already pins `ort = "=2.0.0-rc.12"` at `crates/knowledge/Cargo.toml:50` — the rule contradicts the code. Then ship four targeted ecosystem adoptions grounded in lib.rs May 2026 download metrics: (1) rmcp 1.7.0 as the canonical MCP SDK for any ELAI-as-MCP-server surface; (2) genai 0.6 to consolidate hand-rolled provider clients in CloudGroundingClient; (3) drop the archived `llm` crate references entirely; (4) promote rkyv expansion + insta + proptest. EXPLICITLY FLAG the rule-vs-code contradiction.
Estimated effort: 1-2 sessions, ~6-10 h
Pre-requisites: B-16 PLANNED (anti-stale rule enforcement) — sequence E-34 BEFORE B-16 since E-34 is the rule amendment B-16 will enforce; B-37 SHIPPED (RuntimeAdapter — genai consolidation lands here); CLAUDE.md "Anti-Stale-Training" section.
Parallel-safe with: E-31, E-32, E-33, E-35.
Source report: workflow/improvement-sweep-2026-05-19/reports/34-rust-ai-ecosystem-2026.md

## Context

**FLAG (critical)**: source report directly contradicts a CLAUDE.md HARD RULE. The "Anti-Stale-Training" section in CLAUDE.md states: `ort stable (never -rc)`. Ground truth verified by the report on 2026-05-13: lib.rs/crates/ort shows ort 2.0.0-rc.12 published 2026-03-05 with ~1.24M downloads/month, and the 2.x train has had release candidates since 2024 with NO stable yet. The previous "stable" was 1.13.1, last published Nov 2022 — effectively abandoned. ELAI itself already pins `ort = "=2.0.0-rc.12"` at `crates/knowledge/Cargo.toml:50` (verified by grep). The rule as written is unenforceable AND contradicted by ELAI's own code; B-16 (PLANNED, "ort stable (drop "=2.0.0-rc.12" at knowledge\Cargo.toml:27)") would, if executed, break the only production-grade ONNX path. The fix is to AMEND the rule, not the dep policy. E-34's recommended rewording: "ort 2.x rc track is acceptable; pin exact rc version; do NOT use ort 1.13.1 (abandoned 2022)".

The rest of the report is a 2026 lib.rs crate landscape audit. The canonical adoption candidates with grounded download metrics: rmcp 1.7.0 (2026-05-13, ~2.1M dl/mo) is the official Rust MCP SDK and now the #1 ML crate on lib.rs; rig-core 0.37 (2026-05-13, 275k dl/mo) is the heavier agent framework with 20+ providers + 10 vector stores; genai 0.6 (Jeremy Chone, 24k dl/mo, May 2026) is the lighter one-trait multi-provider client; mistralrs 0.8.1 (9.7k dl/mo, April 2026) replaces the archived `llm` crate; fastembed 5.13.4 (258k dl/mo) wraps ort + tokenizers cleanly. The losers: swiftide 0.32.1 is dormant (57 dl/mo, no release in 6 months); the `llm` crate is archived (last v0.1.1); async-std is upstream-deprecated (ecosystem on tokio). rkyv wins djkoloski's benchmark for nearly every payload (zero-copy); postcard is correct for size-bounded IPC.

Cited grounding: lib.rs/crates/rmcp + github.com/modelcontextprotocol/rust-sdk, lib.rs/crates/rig-core + github.com/0xPlaygrounds/rig, lib.rs/crates/mistralrs + github.com/EricLBuehler/mistral.rs ("mistral.rs has replaced the old llm crate (now archived)"), lib.rs/crates/genai + github.com/jeremychone/rust-genai, lib.rs/crates/ort (the rc.12 evidence), david.kolo.ski/blog/rkyv-is-faster-than/, v2.tauri.app/security/ (Capability ACL), corrode.dev/blog/async/ (tokio consolidation), oxc.rs/docs/guide/benchmarks (3x swc), dasroot.net 2026 testing posts (insta + proptest -30% flakiness).

## Existing state (B/C/D coverage)

- B-16 (PLANNED, "Anti-Stale Rule Enforcement"): says "ort stable (drop "=2.0.0-rc.12" at knowledge\Cargo.toml:27)" — DIRECTLY CONFLICTS with the report's finding. E-34 amends the rule + amends B-16's spec so B-16 enforces the corrected rule, not the broken one. E-34 must ship BEFORE B-16.
- B-37 (SHIPPED commit fd5c2e5): RuntimeAdapter + CloudGroundingClient handle Anthropic + cloud routing. E-34 EXTENDS by consolidating provider-client logic to genai 0.6; per patch-R2 commits, CloudGroundingClient already has OAuth auto-refresh — verify if it ALREADY implements retry/backoff/streaming-SSE-parse the way genai/async-openai do, or if migration to genai saves real code (source report Open Question 1).
- ELAI's `crates/knowledge/Cargo.toml:50` (verified by grep): `ort = "=2.0.0-rc.12"` — the contradiction.
- ELAI's MCP surface today: ELAI consumes local-rag MCP server tools (mcp_search_context, mcp_memorize, etc per CLAUDE.md). If ELAI ever EXPOSES itself as MCP server, rmcp is the canonical SDK. Source report Open Question 3 explores this — currently DEFERRED; E-34 ships rmcp as the adoption candidate but does not wire an MCP server surface in this E-N.
- CLAUDE.md anti-stale list: includes blake3, rand>=0.9, getrandom>=0.3, tracing not log, thiserror/anyhow, tokio::sync::Mutex, native async fn in traits, ort stable, deadpool/bb8, globset/ignore, named constants, .try_into(). E-34 amends the "ort stable" entry; the rest stay.
- swiftide is NOT in ELAI's dependency tree (verified absence via Cargo.toml grep is appropriate before adoption discussion). E-34's role: explicitly mark swiftide as "do not adopt" so it doesn't get suggested in future plan drafts.

## Reads (mandatory before coding)

- workflow/improvement-sweep-2026-05-19/reports/34-rust-ai-ecosystem-2026.md (full)
- F:\.ELAI_workflow\CLAUDE.md "Anti-Stale-Training" + "Rust:" lines (the ort + async-trait + log entries)
- F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md "B-16: Anti-Stale Rule Enforcement" section
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\Cargo.toml line 50 (`ort = "=2.0.0-rc.12"`)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\grounding_client.rs (CloudGroundingClient: assess retry/backoff/streaming-SSE-parse currently in tree)
- F:\.ELAI_workflow\.elai_cc\crates\providers\src\runtime\sidecar.rs (RuntimeAdapter integration site for genai)
- F:\.ELAI_workflow\.elai_cc\crates\providers\src\decoding.rs (provider routing entry point)
- F:\.ELAI_workflow\.elai_cc\Cargo.lock (verify rkyv + postcard versions present today; check transitive ort)
- F:\.ELAI_workflow\.elai_cc\outdated-tech-audit.md (the canonical anti-stale-evidence doc referenced by CLAUDE.md)
- workflow/papers/ (search for any existing notes on rmcp / genai / rig / mistralrs — ingest source-grounding via paper-ingestor mode=source if not present)
- lib.rs/crates/rmcp + lib.rs/crates/genai + lib.rs/crates/ort (online — verify download metrics + latest version BEFORE final adoption decision per anti-slopsquatting rule)

## Tasks

1. Amend `F:\.ELAI_workflow\CLAUDE.md` "Anti-Stale-Training" Rust line (~3 LOC delta). Change `ort stable (never -rc)` to `ort 2.x rc track acceptable; pin exact rc version (=2.0.0-rc.12 today); NEVER use ort 1.13.1 (abandoned Nov 2022). 2.x has had no stable since 2024 — lib.rs ground truth.` Cite lib.rs/crates/ort + the existing `crates/knowledge/Cargo.toml:50` pin. This is the SINGLE most important deliverable in E-34 — the rest of E-34 depends on this amendment because B-16's spec inherits it.

2. Amend `outdated-tech-audit.md` (CLAUDE.md's canonical anti-stale evidence doc — verified path: `F:\.ELAI_workflow\.elai_cc\outdated-tech-audit.md`) to reflect the ort 2.x rc-track reality (~15 LOC). Append section "2026-05 ort 2.x rc track audit" citing the May 13 2026 ground truth: ~1.24M dl/mo, all production callers on rc.X, no stable scheduled.

3. Amend B-16's spec entry in `F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md` (~3 LOC delta — the line "ort stable (drop "=2.0.0-rc.12" at knowledge\Cargo.toml:27)"). New text: "ort 2.x rc-track pin maintained (`=2.0.0-rc.12` today at knowledge\Cargo.toml:50). B-16 enforces the rc-track rule, not a stable-only rule. Re-evaluate when ort 2.0.0 stable releases."

4. Wire `genai = "0.6.0-beta.20"` adoption assessment as a new module `crates/providers/src/runtime/genai_adapter.rs` (~250 LOC). Implements RuntimeAdapter for the genai-backed provider trait — covers Anthropic + Gemini + Groq + xAI + Ollama + DeepSeek + Copilot under one trait. This is a NEW RuntimeAdapter variant alongside existing AnthropicApi; do NOT delete the existing CloudGroundingClient yet — the migration happens AFTER source report Open Question 1 (does ELAI's current CloudGroundingClient already implement retry/backoff/refresh the way genai does?) is answered. Pin exact tag per beta-semver discipline. Cite lib.rs/crates/genai + github.com/jeremychone/rust-genai.

5. Add `crates/providers/Cargo.toml` dep entry for `genai` (~2 LOC). Workspace-level pin in `.elai_cc/Cargo.toml` if the project uses workspace deps (verify via grep first). Anti-slopsquatting verification: confirm `npm view`-equivalent for crates is `cargo search genai` → must show version 0.6.0-beta.X with publish date in last 6 months. CLAUDE.md anti-slopsquatting HARD RULE.

6. Add `rmcp = "1.7.0"` to `crates/orchestrator/Cargo.toml` (or a new `crates/mcp/` crate if cleaner) — adoption-ready BUT no consumer yet (~3 LOC). Source report's MCP-as-server surface is DEFERRED (Open Question 3); this E-N ships the dep + a stub `crates/mcp/src/lib.rs` (~30 LOC) re-exporting rmcp + a `compile_test` ensuring it links. Concrete server surface is a follow-up E-N.

7. Author `crates/providers/src/runtime/genai_adapter.rs` stub with placeholder + test (~100 LOC implementation + ~50 LOC test). The stub MUST NOT be a Lazy Code stub per CLAUDE.md zero-lazy rule — it's a real RuntimeAdapter impl that delegates one method (e.g. `complete()`) to genai for Anthropic, leaves other methods as `unimplemented!("E-34.B: full migration")` registered in the stub registry per .claude/rules/stub-tracking.md. Track in `workflow/sessions/{id}/stubs.md` with `will_resolve_in: E-34.B`.

8. Drop any references to the archived `llm` crate (~5 LOC delta grep + replace). Grep `.elai_cc/**/Cargo.toml` for `^llm = ` or `llm = {` — if any found, remove + replace with `mistralrs = "0.8.1"` or `candle-core = "0.10.2"`. If none found, document as "llm crate not present in tree" in decision doc.

9. Run a `cargo deny` / `cargo audit` baseline + record current rkyv + postcard versions (~10 LOC scripting). Verify rkyv is present in the dependency tree (likely via tantivy or similar); document the actual version. Source report says rkyv wins djkoloski's benchmark for nearly every payload — promote rkyv from "we know about it" to the chosen format for symbol_graph snapshots + tantivy segment metadata caches in a FUTURE E-N, not this one. Keep postcard for size-bounded IPC.

10. Add `insta` + `proptest` adoption assessment: ~30 LOC test scaffold in `crates/orchestrator/tests/insta_snapshot_demo.rs` and `crates/orchestrator/tests/proptest_demo.rs`. Demonstrates insta SSB event-payload snapshot + proptest FSM-replay invariant. Source report cites dasroot 2026 testing posts: -30% flakiness reported. Anti-slopsquatting check: `cargo search insta proptest` → confirm currency.

11. EXPLICITLY ban swiftide in CLAUDE.md "Anti-Stale-Training" section (~2 LOC delta). Add line: "swiftide REJECTED (dormant since Nov 2025, 57 dl/mo per lib.rs); use rig-core 0.37 for agent framework OR hand-roll." Cite lib.rs/crates/swiftide.

12. Author `workflow/reasoning/E-34-decisions.md` (~150 LOC): (a) full record of the ort rc-track contradiction + amendment rationale; (b) why rmcp dep but no server surface yet (Open Question 3); (c) why genai adoption but not full CloudGroundingClient deletion (Open Question 1 unanswered); (d) explicit DEFERRED list: rkyv expansion (separate E-N), MCP server surface (E-34.B), full CloudGroundingClient->genai migration (E-34.C), rig-core agent framework adoption (open question — does ELAI need a heavier framework or is hand-rolled + genai sufficient).

13. Optional: add `oxc = "..."` (~3 LOC dep + ~150 LOC use site) for higher-fidelity TS/JS parsing in `crates/knowledge/src/ast_adapter.rs` (which currently uses tree-sitter per CLAUDE.md anti-stale). Source report says oxc 3x swc, 5x Biome. KEEP tree-sitter as canonical (CLAUDE.md HARD RULE — no swap); add oxc as an OPTIONAL second-pass refinement only. If integration cost > 0.5 days, defer entirely.

## WIRING_CONTRACT

produces_symbols:
  - crates/providers/src/runtime/genai_adapter.rs::GenAiAdapter (new struct, RuntimeAdapter impl)
  - crates/mcp/src/lib.rs::re-export rmcp (new minimal crate)
  - crates/orchestrator/tests/insta_snapshot_demo.rs (insta demo)
  - crates/orchestrator/tests/proptest_demo.rs (proptest demo)
  - workflow/reasoning/E-34-decisions.md (rule amendment rationale)
  - CLAUDE.md Anti-Stale-Training line amendments (ort + swiftide-rejection)
  - outdated-tech-audit.md 2026-05 audit appendix
  - B-PLAN-SUMMARY.md B-16 spec correction
consumes_symbols:
  - CLAUDE.md "Anti-Stale-Training" section (existing HARD RULE block)
  - .elai_cc/outdated-tech-audit.md (CLAUDE.md's canonical evidence doc)
  - crates/knowledge/Cargo.toml line 50 (existing ort = "=2.0.0-rc.12" — UNCHANGED)
  - crates/providers/src/runtime/mod.rs (existing RuntimeAdapter trait — adds GenAiAdapter variant)
  - crates/orchestrator/src/grounding_client.rs (existing CloudGroundingClient — kept; coexists with new adapter)
reachable_from:
  - any role dispatch -> RuntimeOrchestrator::pick_runtime -> GenAiAdapter (when policy selects)
  - any future ELAI-as-MCP-server surface -> crates/mcp::rmcp (re-export)
  - cargo test --workspace -> insta_snapshot_demo + proptest_demo passing
stub_ban:
  - no archived `llm` crate reference remains in any Cargo.toml after task 8
  - swiftide MUST be absent from any Cargo.toml after task 11 (rejected)
  - async-std MUST be absent from any Cargo.toml (ecosystem on tokio per CLAUDE.md HARD RULE)
  - any unimplemented!() in genai_adapter.rs MUST have a stub-registry row pointing to E-34.B (.claude/rules/stub-tracking.md)
  - CLAUDE.md anti-slopsquatting: any new crate dep gets `cargo search <name>` verification + license + last-published date check before commit
  - .unwrap() banned in genai_adapter.rs production paths (test code OK)
  - blake3 for any cache key (NEVER blake2/sha2)
frontend_pair: null (this is a Rust-ecosystem + rule-amendment E-N; no Svelte changes)
observable_effect:
  trigger: "cargo check --workspace + cargo test -p providers --test genai_adapter"
  expect_ssb: null (rule + dependency change, no runtime SSB)
  expect_artifact: "cargo check green; CLAUDE.md anti-stale ort line reflects rc-track reality; B-16 spec corrected; outdated-tech-audit.md carries 2026-05 audit appendix; insta + proptest demos pass"

## DETERMINISTIC_CHECK

```bash
# 1. CLAUDE.md ort rule amended (rc track acceptable)
grep "ort 2.x rc track acceptable\|rc.12 today" F:/.ELAI_workflow/CLAUDE.md
# Expected: amended text present

# 2. CLAUDE.md ort old wording GONE
! grep "ort stable (never -rc)" F:/.ELAI_workflow/CLAUDE.md
# Expected: exit code 0 (string not found)

# 3. swiftide explicitly rejected in CLAUDE.md
grep "swiftide REJECTED\|swiftide.*dormant" F:/.ELAI_workflow/CLAUDE.md
# Expected: rejection line present

# 4. outdated-tech-audit.md has 2026-05 ort audit appendix
grep "2026-05.*ort\|ort 2.x rc track" F:/.ELAI_workflow/.elai_cc/outdated-tech-audit.md

# 5. B-PLAN-SUMMARY.md B-16 spec corrected
grep "ort 2.x rc-track pin maintained\|=2.0.0-rc.12.*knowledge.Cargo.toml:50" F:/.ELAI_workflow/docs/thefinalglance/B-plan/B-PLAN-SUMMARY.md

# 6. ort pin in code UNCHANGED
grep '^ort = "=2.0.0-rc.12"$' F:/.ELAI_workflow/.elai_cc/crates/knowledge/Cargo.toml
# Expected: unchanged line at line 50

# 7. genai dep present + GenAiAdapter module exists
grep "^genai = " F:/.ELAI_workflow/.elai_cc/crates/providers/Cargo.toml
test -f F:/.ELAI_workflow/.elai_cc/crates/providers/src/runtime/genai_adapter.rs
rg "struct GenAiAdapter" F:/.ELAI_workflow/.elai_cc/crates/providers/src/runtime/genai_adapter.rs

# 8. rmcp dep + minimal mcp crate exist
grep "^rmcp = " F:/.ELAI_workflow/.elai_cc/crates/orchestrator/Cargo.toml F:/.ELAI_workflow/.elai_cc/crates/mcp/Cargo.toml 2>/dev/null
test -f F:/.ELAI_workflow/.elai_cc/crates/mcp/src/lib.rs

# 9. archived llm crate absent
! grep -r "^llm = " F:/.ELAI_workflow/.elai_cc/crates/*/Cargo.toml

# 10. insta + proptest demos present and pass
test -f F:/.ELAI_workflow/.elai_cc/crates/orchestrator/tests/insta_snapshot_demo.rs
test -f F:/.ELAI_workflow/.elai_cc/crates/orchestrator/tests/proptest_demo.rs
cd F:/.ELAI_workflow/.elai_cc && cargo test -p orchestrator --test insta_snapshot_demo --test proptest_demo

# 11. cargo check --workspace green
cd F:/.ELAI_workflow/.elai_cc && cargo check --workspace
```

## ACCEPTANCE

- [ ] CLAUDE.md "Anti-Stale-Training" ort line amended to allow rc-track with exact-version pin
- [ ] CLAUDE.md OLD wording "ort stable (never -rc)" removed
- [ ] CLAUDE.md swiftide-rejection line added
- [ ] outdated-tech-audit.md carries 2026-05 ort + rmcp + genai audit appendix
- [ ] B-16 spec in B-PLAN-SUMMARY.md corrected to "rc-track pin maintained" not "drop rc.12"
- [ ] `crates/knowledge/Cargo.toml:50` `ort = "=2.0.0-rc.12"` UNCHANGED (no premature stable migration)
- [ ] `crates/providers/Cargo.toml` has `genai = "0.6.0-beta.20"` (pinned beta tag)
- [ ] `crates/providers/src/runtime/genai_adapter.rs` exists; one method (complete) wired to genai for Anthropic; other methods are registered stubs (per .claude/rules/stub-tracking.md) pointing to E-34.B
- [ ] `crates/mcp/` crate exists with rmcp re-export and links cleanly (cargo check green)
- [ ] insta + proptest demo tests pass
- [ ] archived `llm` crate references absent from all Cargo.toml files
- [ ] async-std absent from all Cargo.toml files
- [ ] cargo check --workspace green
- [ ] Decision doc at `workflow/reasoning/E-34-decisions.md` captures rule-amendment rationale + open questions Q1-Q5 from source report
- [ ] Anti-slopsquatting verified: every new dep has `cargo search` evidence + license check recorded in decision doc
- [ ] CLAUDE.md anti-disturbance: no `.unwrap()` introduced; blake3 for any new cache key

## Tests to add

- `.elai_cc/crates/providers/tests/genai_adapter_smoke.rs`: smoke test asserting `GenAiAdapter::complete` returns a non-empty completion against a mock provider (offline; no network).
- `.elai_cc/crates/mcp/tests/rmcp_compile.rs`: compile-only test that `use rmcp::*;` resolves (link sanity).
- `.elai_cc/crates/orchestrator/tests/insta_snapshot_demo.rs`: snapshot a synthetic SSB event-payload set; first run creates baseline, subsequent runs verify no drift.
- `.elai_cc/crates/orchestrator/tests/proptest_demo.rs`: property-based test of FSM replay determinism over 100 random transition sequences.
- `.elai_cc/crates/contracts/tests/anti_stale_ort_rule_contract.rs`: lint-style test asserting CLAUDE.md contains the new rc-track wording (and NOT the old "ort stable (never -rc)" wording).

## Files touched

- MODIFY: `F:\.ELAI_workflow\CLAUDE.md` (~5 LOC: ort rule amendment + swiftide rejection)
- MODIFY: `F:\.ELAI_workflow\.elai_cc\outdated-tech-audit.md` (~30 LOC: 2026-05 audit appendix)
- MODIFY: `F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md` (~3 LOC: B-16 spec correction)
- MODIFY: `.elai_cc/crates/providers/Cargo.toml` (~2 LOC: genai dep)
- MODIFY: `.elai_cc/crates/orchestrator/Cargo.toml` OR CREATE `.elai_cc/crates/mcp/Cargo.toml` (~5 LOC: rmcp dep)
- CREATE: `.elai_cc/crates/providers/src/runtime/genai_adapter.rs` (~250 LOC)
- CREATE: `.elai_cc/crates/mcp/src/lib.rs` (~30 LOC re-export)
- CREATE: `.elai_cc/crates/mcp/Cargo.toml` (~15 LOC)
- CREATE: `.elai_cc/crates/providers/tests/genai_adapter_smoke.rs` (~120 LOC)
- CREATE: `.elai_cc/crates/mcp/tests/rmcp_compile.rs` (~20 LOC)
- CREATE: `.elai_cc/crates/orchestrator/tests/insta_snapshot_demo.rs` (~80 LOC)
- CREATE: `.elai_cc/crates/orchestrator/tests/proptest_demo.rs` (~120 LOC)
- CREATE: `.elai_cc/crates/contracts/tests/anti_stale_ort_rule_contract.rs` (~50 LOC)
- CREATE: `workflow/reasoning/E-34-decisions.md` (~150 LOC)

## Citations / Grounding

- lib.rs/crates/ort (cited in source) — 2.0.0-rc.12 published 2026-03-05, ~1.24M dl/mo, no stable since 2024
- lib.rs/crates/rmcp + github.com/modelcontextprotocol/rust-sdk (cited in source) — 2.1M dl/mo, official MCP SDK, Rust 2024, Apache-2.0
- lib.rs/crates/rig-core + github.com/0xPlaygrounds/rig (cited in source) — 275k dl/mo, agent + RAG + structured extraction
- lib.rs/crates/genai + github.com/jeremychone/rust-genai (cited in source) — 24k dl/mo, one trait for 20+ providers
- lib.rs/crates/mistralrs + github.com/EricLBuehler/mistral.rs (cited in source) — replaces archived llm crate
- lib.rs/crates/swiftide (cited in source) — 57 dl/mo, last release Nov 2025, REJECTED
- david.kolo.ski/blog/rkyv-is-faster-than/ (cited in source) — rkyv wins nearly every payload vs bincode/capnp/postcard/prost
- corrode.dev/blog/async/ (cited in source) — tokio consolidation; async-std deprecated
- v2.tauri.app/security/ (cited in source) — Capability ACL + Isolation pattern
- oxc.rs/docs/guide/benchmarks (cited in source) — oxc 3x swc, 5x Biome (OPTIONAL adoption)
- dasroot.net 2026 testing posts (cited in source) — insta + proptest -30% flakiness
- crates.io/crates/anthropic-sdk-rust + crates.io/crates/adk-anthropic (cited in source) — alternative Anthropic clients
- lib.rs/crates/fastembed (cited in source) — 258k dl/mo, sync, wraps ort
- lib.rs/crates/tantivy (cited in source) — 946k dl/mo, ~2x Lucene

## Open questions / risks

- **CRITICAL**: source report directly contradicts a CLAUDE.md HARD RULE. The rule amendment in task 1 is the highest-priority deliverable; do NOT proceed with the genai/rmcp adoptions until the rule is amended, or B-16 (when it runs) will try to "fix" the rc.12 pin and break ort entirely.
- CloudGroundingClient currently has OAuth auto-refresh per patch-R2 commits — must AUDIT whether it ALSO implements retry/backoff/streaming-SSE-parse the way genai/async-openai do. If yes, the genai migration is a code-cleanup play, not a feature add. Source report Open Question 1.
- Migrating provider-client to genai introduces a NEW dependency with beta-semver; pin the exact tag (`0.6.0-beta.20` or whatever current at adoption time). Plan migration in two phases: E-34 ships dep + adapter stub; E-34.C migrates CloudGroundingClient call sites and removes old code.
- ELAI-as-MCP-server is DEFERRED (source report Open Question 3): rmcp adoption ships the dep + minimal crate but no server surface. Concrete surface (which CLI verbs map to MCP tools) is a separate design problem.
- rkyv expansion to symbol_graph snapshots + tantivy segment metadata: source report Open Question 4 — measure decode-latency delta vs postcard; if <2x improvement, skip. E-34 ships the testing demo only.
- B-16 anti-stale rule enforcement (PLANNED) will lint the CLAUDE.md anti-stale list; E-34's amended rule must be the version B-16 enforces. Coordinate timing: E-34 ships BEFORE B-16 starts.
- Beta-semver discipline: genai 0.6.0-beta.X means breaking changes possible between betas; lock to a specific beta tag and reassess at every minor bump. Add a comment in Cargo.toml citing the beta pin reason.
- swiftide rejection is preemptive (not in tree). Source report Open Question 5: "is the whole project abandoned?" — if bosun-ai org migrates to a successor, revisit; otherwise it stays rejected.
- oxc adoption is OPTIONAL — only adopt if integration cost in `crates/knowledge/src/ast_adapter.rs` is < 0.5 days. tree-sitter stays canonical per CLAUDE.md "Tree-sitter retained" rule.
- CLAUDE.md edits must NOT regress other anti-stale lines (rand>=0.9, blake3, native async fn, etc). Diff carefully.

## Next

- E-34.B (deferred): full genai adoption — replace CloudGroundingClient call sites with GenAiAdapter; remove old code; close stub-registry rows.
- E-34.C (deferred): ELAI-as-MCP-server surface — design which CLI verbs map to MCP tools; wire rmcp.
- E-34.D (deferred): rkyv expansion to symbol_graph snapshots + tantivy segment metadata caches — only if decode-latency benchmark shows >2x win over postcard.
- E-34.E (deferred): rig-core 0.37 evaluation — does ELAI need a heavier agent framework or is current hand-rolled + genai sufficient?
- E-34.F (deferred): oxc adoption decision in `crates/knowledge/src/ast_adapter.rs` (if integration cost < 0.5 days).
- B-16 (PLANNED): anti-stale rule enforcement — inherits E-34's amended rule. SEQUENCE: E-34 ships before B-16 starts.
- B-37 (SHIPPED): RuntimeOrchestrator hosts the new GenAiAdapter variant.
- C-13 (PLANNED IPC dispatcher production swap): unaffected by E-34; sequence independent.
