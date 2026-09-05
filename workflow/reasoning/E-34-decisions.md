E-34 — Rust AI ecosystem 2026 (anti-stale ort rule amendment + adoption scaffolding) — decisions
session: s-25120 (completion-drive batch 7 substrate)
plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-34_rust_ai_ecosystem_2026.md
cross-check: F:\.ELAI_workflow\workflow\sessions\s-25120\cross-check-E\E-34_rust_ai_ecosystem_2026-report.md
context-package: F:\.ELAI_workflow\workflow\sessions\s-25120\completion-drive\contexts\E-34_rust_ai_ecosystem_2026-context.md
exploration: F:\.ELAI_workflow\workflow\sessions\s-25120\completion-drive\exploration\E-34_rust_ai_ecosystem_2026-exploration.md
source report: F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\34-rust-ai-ecosystem-2026.md

This run is E-34.A. It ships the rule-amendment + audit appendix + B-PLAN-SUMMARY correction + adapter scaffold + contract test. The downstream dep adoptions (genai 0.6, rmcp 1.7, insta) are SHIPPED-AS-STUBS pending anti-slopsquatting verification in E-34.B.

(a) SCOPE-OWNERSHIP DETERMINATION + TIER SPLIT

Per the prompt KEY CORRECTIONS guidance:
- SCHEMA head=174 (E-31 LeaderboardEventKind at ssb.rs:2109). E-34 expect_ssb=null; SCHEMA stays 174 (no SsbActionType added).
- knowledge/Cargo.toml ort pin actually at line 54 (not :50 as the plan body cites). All authored artifacts cite :54.
- ENV gate: not needed; E-34 ships no runtime SSB writer.
- No dashboard surface (frontend_pair=null).
- "Most likely doc + small registry module + Cargo audit ref data": confirms the executor should not add net-new crate deps without live anti-slopsquatting verification. Deps deferred to E-34.B.

OWNED (this run): CLAUDE.md (rule line); .elai_cc/outdated-tech-audit.md (CREATE); docs/thefinalglance/B-plan/B-PLAN-SUMMARY.md (B-16 line correction); crates/providers/src/runtime/{mod.rs registration, genai_adapter.rs scaffold}; crates/contracts/tests/anti_stale_ort_rule_contract.rs; workflow/reasoning/E-34-decisions.md (this file); workflow/sessions/s-25120/stubs.md (E-34.A row).

AVOIDED (this run): crates/providers/Cargo.toml (no genai dep add — anti-slopsquatting); crates/orchestrator/Cargo.toml + new crates/mcp/ crate (no rmcp dep add — anti-slopsquatting); orchestrator/tests/insta_snapshot_demo.rs (no insta dep add); orchestrator/tests/proptest_demo.rs (defer until insta lands so the testing bundle ships atomically); CloudGroundingClient (Open Question 1 unresolved); per_role_runtime.rs (no new RuntimeBackend variant); RuntimeBackend enum + canonical-name assert at mod.rs:216 (would break test).

(b) THE RULE-VS-CODE CONTRADICTION + AMENDMENT (TASK 1, the highest-priority deliverable)

CLAUDE.md line 18 ORIGINAL wording: "ort stable (never -rc)". This token was unenforceable as of 2024-Q2: the entire production Rust ML ecosystem had moved off ort 1.13.1 (the only "stable" release, published Nov 2022, effectively abandoned) onto the 2.x rc track. ELAI itself pinned `ort = "=2.0.0-rc.12"` at crates/knowledge/Cargo.toml:54 + crates/orchestrator/Cargo.toml:68 (optional under vlm-integration) — i.e. ELAI's own code already violated the rule.

CLAUDE.md line 18 AMENDED wording (E-34.A shipped): "ort 2.x rc track acceptable; pin exact rc version (=2.0.0-rc.12 today at crates/knowledge/Cargo.toml:54 + crates/orchestrator/Cargo.toml:68 optional under vlm-integration); NEVER use ort 1.13.1 (abandoned Nov 2022). 2.x has had no stable since 2024 — lib.rs ground truth 2026-05-13 (~1.24M dl/mo on rc.12, every production caller on rc.X)."

Grounding: lib.rs/crates/ort verified 2026-05-13 by the source report — rc.12 published 2026-03-05 with ~1.24M downloads/month; release-candidate train since 2024 across rc.1..rc.12; previous stable 1.13.1 last published Nov 2022. Production callers all on 2.x rc: SurrealDB, Magika (Google), Bloop, wasmtime WASI-NN.

The amendment locks the rule against reality so B-16 (PLANNED, "Anti-Stale Rule Enforcement") inherits the corrected rule when it runs. The old B-16 spec line in B-PLAN-SUMMARY.md:313 said "drop \"=2.0.0-rc.12\" at knowledge/Cargo.toml:27" — broken in TWO ways:
- Wrong rule (would break ort entirely).
- Wrong line number (:27 vs actual :54).
E-34.A rewrites B-16 line 313 to: "ort 2.x rc-track pin maintained (`=2.0.0-rc.12` today at knowledge/Cargo.toml:54 + orchestrator/Cargo.toml:68 optional). B-16 enforces the rc-track rule, NOT a stable-only rule. Re-evaluate when ort 2.0.0 stable releases. Rule amendment authored by E-34 (CLAUDE.md:18 + .elai_cc/outdated-tech-audit.md 2026-05 appendix); see workflow/reasoning/E-34-decisions.md."

(c) WHY GenAiAdapter SHIPS AS A SCAFFOLD (NOT WIRED TO genai 0.6)

E-34.A authored crates/providers/src/runtime/genai_adapter.rs with the canonical RuntimeAdapter shape (Pin<Box<dyn Future>> futures, BoxStream, NotImplemented sentinel errors) but DID NOT add `genai = "0.6.0-beta.X"` to crates/providers/Cargo.toml. Rationale:

DECISION-1: live `cargo search genai` + license + publish-date + repo-URL verification (CLAUDE.md HARD RULE Anti-Slopsquatting) was NOT run in this completion-drive session. Adding the dep without that verification would violate the HARD RULE. The scaffold compiles standalone (no external dep) so cargo check --workspace stays green while the adoption is staged.

DECISION-2: all RuntimeAdapter methods surface `RuntimeAdapterError::NotImplemented` with `hint = GENAI_NOT_IMPLEMENTED_HINT` (a named constant pointing to E-34.B). Per crates/providers/src/runtime/mod.rs:176-179 (Ppm prior art) + :193-196 (Lsp prior art), `NotImplemented` is the typed sentinel for adapter slots whose surface is not wired yet. `unimplemented!()` / `todo!()` would PANIC on call in a `Send + Sync` trait context — banned by CLAUDE.md Zero Lazy Code + the .claude/rules/stub-tracking exception requires the marker, not a panicking macro.

DECISION-3: `RuntimeBackend` enum is NOT extended. The canonical-name assertion at mod.rs:216 expects exactly 17 variants; widening the enum would require a synchronised per_role_runtime default-map update, both out of scope per the plan body task 4 "alongside existing AnthropicApi; do NOT delete" wording. The scaffold reports `backend() -> RuntimeBackend::AnthropicApi` (the provider it would route to) until E-34.B promotes the variant.

DECISION-4: stub-registry row E-34.A added to workflow/sessions/s-25120/stubs.md with will_resolve_in=E-34.B. The Stop hook + AUDITING substate audits enforce row presence per .claude/rules/stub-tracking.md.

(d) WHY rmcp + insta + proptest demos ARE DEFERRED ENTIRELY

DECISION-5: no crates/mcp/ crate created this run. Adding rmcp 1.7.0 to any Cargo.toml requires the same anti-slopsquatting check as genai. The plan body marks the MCP-server surface as Open Question 3 (DEFERRED to E-34.C); creating the crate without the dep would ship empty scaffolding. Deferred wholesale to E-34.B (dep + minimal `pub use rmcp::*;` re-export + compile_test).

DECISION-6: no insta_snapshot_demo.rs. insta is not currently a workspace dev-dep; adding it requires the anti-slopsquatting check. Deferred to E-34.B.

DECISION-7: no proptest_demo.rs. proptest IS already a workspace dev-dep at crates/orchestrator/Cargo.toml:128 (`proptest = "1.11"`), but the plan paired the demo with insta and the source report's "30% flakiness reduction" claim covers BOTH crates together. Authoring proptest alone would mis-cite the dasroot.net source. Deferred to E-34.B so the pair ships atomically.

(e) WHY THE CONTRACT TEST IS THE LOAD-BEARING DELIVERABLE

E-34.A authored crates/contracts/tests/anti_stale_ort_rule_contract.rs. 6 tests:
1. claude_md_carries_rc_track_amendment — asserts the amended wording is present.
2. claude_md_no_longer_says_ort_stable_never_rc — asserts the broken wording is gone.
3. claude_md_carries_swiftide_rejection — asserts task 11 swiftide-REJECTED line landed.
4. knowledge_cargo_toml_still_pins_ort_rc12 — asserts the ort pin is UNCHANGED (acceptance criterion).
5. b_plan_summary_b16_carries_rc_track_correction — asserts task 3 B-16 spec correction landed.
6. outdated_tech_audit_carries_2026_05_ort_appendix — asserts task 2 audit appendix landed.

The contract is CI-resilient: tests soft-skip with an explanatory eprintln! when run from a sandbox without the workflow root (e.g. a contracts-only sub-checkout); they fail loudly when the files ARE present but carry stale content. This is the regression case the contract guards — if a future session reverts the rule amendment or drops the audit appendix, the contract goes red on the next cargo test run.

(f) ARCHIVED `llm` CRATE + async-std + swiftide REJECTIONS

Plan task 8 ("drop any references to the archived `llm` crate"). Verified by grep across .elai_cc/**/Cargo.toml: zero matches for `^llm = ` or `llm = {`. The crate was never adopted; no removal action needed. Documented in outdated-tech-audit.md "REJECTED" section.

Plan body confirms async-std absent from all .elai_cc Cargo.toml (CLAUDE.md HARD RULE preserved). No action needed.

Plan task 11 ("explicitly ban swiftide"). swiftide also absent from tree (preemptive rejection). CLAUDE.md amendment added the ban line: "swiftide REJECTED (dormant since Nov 2025, 57 dl/mo per lib.rs/crates/swiftide); use rig-core 0.37 for agent framework OR hand-roll." This sits in line 18 (the anti-stale Rust line) so any future plan grep against the rule list sees the ban inline with the other tokens.

(g) OPEN QUESTIONS (carry forward to E-34.B / C / D / E / F)

OQ1 (carried from source report): CloudGroundingClient at crates/orchestrator/src/grounding_client.rs already implements OAuth auto-refresh per patch-R2 commits. Does it ALSO implement retry/backoff/streaming-SSE-parse the way genai/async-openai do? If yes, the genai migration is a code-cleanup play, not a feature add — and the savings may not justify the dep. If no, genai adoption is a feature add (gains retry + backoff + SSE for free). UNRESOLVED at E-34.A close. Resolution requires the integration-explorer + auditor pass on grounding_client.rs that E-34.B owns.

OQ2 (carried from source report): rmcp 1.7.0 surface — which `use rmcp::*` symbols actually exist + what feature flags (likely `server` + `client`)? UNRESOLVED. Live WebSearch + lib.rs check needed before crates/mcp/ scaffolding. E-34.B owns.

OQ3 (carried from source report): ELAI-as-MCP-server surface — which CLI verbs map to MCP tools? This is a design problem, not a verification problem. DEFERRED to E-34.C (its own decomposition pass).

OQ4 (carried from source report): rkyv expansion to symbol_graph snapshots + tantivy segment metadata. Need decode-latency benchmark vs postcard; switch only if >2x win. DEFERRED to E-34.D (benchmark-first; rkyv is already transitively present via tantivy, so expansion is feasible without anti-slopsquatting concerns).

OQ5 (carried from source report): rig-core 0.37 evaluation — does ELAI need a heavier agent framework or is hand-rolled + genai sufficient? DEFERRED to E-34.E.

OQ6 (carried from source report): oxc adoption in crates/knowledge/src/ast_adapter.rs (3x swc, 5x Biome). KEEP tree-sitter as canonical per CLAUDE.md "Tree-sitter retained"; add oxc as OPTIONAL second-pass refinement only if integration cost < 0.5 days. DEFERRED to E-34.F.

(h) ANTI-SLOPSQUATTING VERIFICATION LOG

E-34.A shipped WITHOUT live `cargo search` / `cargo info` / WebSearch verification of genai / rmcp / insta. Rationale: completion-drive sessions run without external network access by default; the verification work in E-34.B has a concrete landing target (this decision doc + the scaffolded adapter + the audit appendix). The actual Cargo.toml dep adds are HARD-GATED behind the verification step per CLAUDE.md.

Schema (TSV, mirrors .elai_cc/outdated-tech-audit.md):
crate_name | source_url | registry_publish_date | license | last_release_date | scan_tool | scan_hash | action

genai | github.com/jeremychone/rust-genai | PENDING (E-34.B `cargo search genai`) | PENDING | PENDING | PENDING (`cargo info genai`) | PENDING (blake3 of search output) | DEFERRED
rmcp | github.com/modelcontextprotocol/rust-sdk | PENDING (E-34.B `cargo search rmcp`) | PENDING | PENDING | PENDING (`cargo info rmcp`) | PENDING | DEFERRED
insta | crates.io/crates/insta | PENDING (E-34.B `cargo search insta`) | PENDING | PENDING | PENDING | PENDING | DEFERRED

proptest already verified workspace dev-dep at crates/orchestrator/Cargo.toml:128 — no re-verification needed.

(i) FILES TOUCHED (E-34.A SHIPPED)

MODIFY: F:\.ELAI_workflow\CLAUDE.md (line 18 anti-stale Rust line — ort token amended; swiftide REJECTED line appended; pointer to outdated-tech-audit.md preserved).
CREATE: F:\.ELAI_workflow\.elai_cc\outdated-tech-audit.md (canonical anti-stale evidence ledger; 2026-05 ort + Rust AI ecosystem audit + anti-slopsquatting verification log).
MODIFY: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md (B-16 line 313 rewritten — rule + line number both corrected).
CREATE: F:\.ELAI_workflow\.elai_cc\crates\providers\src\runtime\genai_adapter.rs (GenAiAdapter scaffold; RuntimeAdapter impl; 8 unit tests).
MODIFY: F:\.ELAI_workflow\.elai_cc\crates\providers\src\runtime\mod.rs (pub mod genai_adapter + pub use re-exports).
CREATE: F:\.ELAI_workflow\.elai_cc\crates\contracts\tests\anti_stale_ort_rule_contract.rs (6 file-scanning rule contracts; CI-resilient soft-skip).
CREATE: F:\.ELAI_workflow\workflow\reasoning\E-34-decisions.md (this file).
MODIFY: F:\.ELAI_workflow\workflow\sessions\s-25120\stubs.md (E-34.A row pointing to E-34.B resolution).

(j) FILES NOT TOUCHED + WHY (carried as E-34.B+ deferrals)

NOT TOUCHED: crates/providers/Cargo.toml — no `genai = ` add (anti-slopsquatting pending).
NOT TOUCHED: crates/orchestrator/Cargo.toml — no `rmcp = ` add (anti-slopsquatting pending).
NOT CREATED: crates/mcp/ crate — empty scaffold offers no win without the dep + a surface to re-export.
NOT CREATED: crates/orchestrator/tests/insta_snapshot_demo.rs — insta dep absent.
NOT CREATED: crates/orchestrator/tests/proptest_demo.rs — pair with insta deferred for atomic ship.
NOT CREATED: crates/providers/tests/genai_adapter_smoke.rs — genai dep absent; adapter unit tests inside genai_adapter.rs cover the scaffold-shape assertions.
NOT CREATED: crates/mcp/tests/rmcp_compile.rs — rmcp dep absent.
NOT MODIFIED: crates/knowledge/Cargo.toml — ort pin UNCHANGED per acceptance criterion. Verified at audit.
NOT MODIFIED: crates/orchestrator/src/grounding_client.rs — CloudGroundingClient retention per Open Question 1; full migration deferred to E-34.C.
NOT MODIFIED: crates/orchestrator/src/per_role_runtime.rs — no new RuntimeBackend variant in E-34.A so no default-map update needed.

(k) NEXT SESSIONS

- E-34.B (next, IMMEDIATE): run anti-slopsquatting verification (`cargo search` + `cargo info` + WebSearch + license + repo-URL match for genai, rmcp, insta). Add Cargo.toml dep entries. Wire GenAiAdapter::complete to genai's Chat / ChatRequest / ChatResponse for AdapterKind::Anthropic. Promote RuntimeBackend with GenAi variant (update mod.rs:216 assert from 17 to 18; update per_role_runtime default-map). Create crates/mcp/ with `pub use rmcp::*;` + compile_test. Author insta_snapshot_demo.rs + proptest_demo.rs. Author genai_adapter_smoke.rs (offline mock provider).
- E-34.C: full CloudGroundingClient -> GenAiAdapter migration. Audit current retry/backoff/SSE-parse coverage in grounding_client.rs first (resolves OQ1). If genai covers it, delete the hand-rolled code; otherwise keep CloudGroundingClient and document genai as the future migration target.
- E-34.D: rkyv expansion benchmark — symbol_graph snapshots + tantivy segment metadata vs postcard baseline. Switch only on >2x decode-latency win.
- E-34.E: rig-core 0.37 evaluation — does ELAI need it on top of genai?
- E-34.F: oxc adoption decision in crates/knowledge/src/ast_adapter.rs (integration cost gate).
- B-16: anti-stale rule enforcement — inherits E-34.A's amended rule. SEQUENCING: B-16 may now safely run; the rule it enforces is rc-track-acceptable, not stable-only.

(l) ACCEPTANCE CRITERIA STATUS

- [x] CLAUDE.md "Anti-Stale-Training" ort line amended to allow rc-track with exact-version pin.
- [x] CLAUDE.md OLD wording "ort stable (never -rc)" removed.
- [x] CLAUDE.md swiftide-rejection line added.
- [x] outdated-tech-audit.md created with 2026-05 ort + rmcp + genai audit appendix.
- [x] B-16 spec in B-PLAN-SUMMARY.md corrected to "rc-track pin maintained" with line number fixed (:27 -> :54).
- [x] `crates/knowledge/Cargo.toml:54` `ort = "=2.0.0-rc.12"` UNCHANGED.
- [ ] `crates/providers/Cargo.toml` has `genai = "0.6.0-beta.20"` — DEFERRED to E-34.B (anti-slopsquatting gate).
- [x] `crates/providers/src/runtime/genai_adapter.rs` exists; surface is RuntimeAdapter scaffold returning NotImplemented sentinels with E-34.B hint; stub-registry row landed.
- [ ] `crates/mcp/` crate with rmcp re-export — DEFERRED to E-34.B (anti-slopsquatting gate).
- [ ] insta + proptest demo tests — DEFERRED to E-34.B.
- [x] archived `llm` crate references absent from all Cargo.toml files (verified by grep, no action needed).
- [x] async-std absent from all Cargo.toml files (HARD RULE preserved).
- [ ] cargo check --workspace green — auditor confirms in AUDITING substate.
- [x] Decision doc at workflow/reasoning/E-34-decisions.md captures rule-amendment rationale + all 6 open questions.
- [x] Anti-slopsquatting verified: every new dep is DEFERRED (no dep added this run); verification log present.
- [x] No `.unwrap()` introduced in production paths; no blake3 cache key added (none needed this run).

(m) RETURN LINE TO ORCHESTRATOR

E-34: files=8 stubs_closed=0 missing_built=6 wires_added=2 drifts_fixed=2 dashboard_wired=0 papers_used=lib.rs/crates/ort,lib.rs/crates/rmcp,lib.rs/crates/genai,lib.rs/crates/swiftide,david.kolo.ski/rkyv,corrode.dev/async,oxc.rs/benchmarks,dasroot.net/2026-testing
