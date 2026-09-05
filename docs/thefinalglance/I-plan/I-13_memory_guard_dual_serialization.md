# I-13: memory guard + dual serialization

- Track 14 — CACHE-INVARIANT ECONOMICS MIDDLEWARE | lever SECURITY-EFFICIENCY | grounding STRONG | nature BUILD | effort M | WAVE 2 | IC-10
- Premise: ELAI's memory write path has no deterministic scan at write time — invisible-Unicode injection,
  multi-byte tricks, and secret patterns can enter the Recall/Archival store unchecked.
  Context efficiency suffers from the other direction: tool results enter context at full size even when a
  condensed form suffices, and there is no mechanism for the model to expand a specific range on demand.
  Three orthogonal fixes shipped together: write-time guard, dual serialization, and TraceQuery expansion.

## Steal/build
IC-10 binding correction: the write-time scan must COMPOSE the shipped H-31/H-32 auditor-objectivity firewall
+ E-37 PromptArmor + memory-crate MINJA defenses; it never re-filters what they already cover.
Net-new: (a) write-time deterministic scan on ALL Recall/Archival/playbook memory writes (injection patterns,
invisible-Unicode/multi-byte tricks, secrets) — regex + aho-corasick first; a YARA engine is a NEW DEP
requiring anti-slopsquat verification and explicit user approval (IC-10) before it is added; (b)
OpenHands-style dual serialization: to_ui_dict (full payload, dashboard) vs to_llm_content (condensed,
context-assembly path) on every tool result; (c) TraceQuery tool so the model can expand exact line ranges
from disk on demand, preventing condensation starvation when the condensed form loses needed detail.
The FP rate of the write-time guard is bounded on >=3 real runs before default-ON (GD-4).
G-9 redaction is the write-time precedent; IS-11 taint/provenance is the safety-crate composition seam.

## Existing ground truth
- IS-11: capability_flow.rs taint_text:234; crates/safety/src/provenance.rs:131 Provenance (dedicated safety crate) — the composition seam for I-13's write-time guard (PLAN-asserted; executor re-greps per W-7)
- H-31/H-32: auditor-objectivity firewall in the memory crate (PLAN-asserted; executor re-greps per W-7); I-13 composes these — never re-filters
- E-37 PromptArmor: data-plane classify (shipped — compose, never re-implement)
- G-9 redaction: write-time precedent (PLAN-asserted; executor re-greps per W-7)
- H-6 masking: different layer (H-6 drops turns; I-13 scans writes) — compose without conflict
- IC-10 (I-GROUNDING-MAP.md): YARA = new dep requiring anti-slopsquat + user approval; regex/aho-corasick first is the REQUIRED order

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- memory write-time scan: regex + aho-corasick patterns for invisible-Unicode, injection markers, secret patterns at the mem0_dispatcher/store write choke-points; MemoryGuardEvent(PoisonBlocked) SSB row with pattern class named
- dual serialization: to_ui_dict + to_llm_content trait on tool results (ToolResult enum extension in tools/tool_result.rs); crates/tools adapter sites updated; journal shows the token delta
- TraceQuery tool added via the 3-arm catalog pattern (IS-14): expands exact line ranges from disk on demand; MemoryGuardEvent(TraceQueryExpanded) row
- +MemoryGuardEvent (Kind: PoisonBlocked/DualSerialized/TraceQueryExpanded) in ssb.rs / action_type_lock.rs
- FP-rate fixture: >=3 real runs on a clean corpus before guard is default-ON; GD-4 bounds it

DEFER (Tier-2 / gated):
- YARA engine integration (new dep — anti-slopsquat + user approval required first; add only after approval and only if regex/aho-corasick FP rate is insufficient after measurement)
- automated secret-pattern library updates from a versioned threat feed (governance needed; note as follow-on)

## Integration
Scan insertion: memory write choke-points — mem0_dispatcher.rs + store.rs write seam (PLAN-asserted; executor re-greps per W-7)
Dual serialization: crates/tools/src/tool_result.rs (to_ui_dict / to_llm_content trait) + adapter call sites
TraceQuery: production_agent_rpc.rs build_tool_catalog (IS-14: the 3-arm pattern); crates/tools/src/trace_query.rs (~80 LOC)
Safety crate composition: crates/safety/src/provenance.rs (IS-11) — I-13 registers its scanner as a write-side provenance hook
frontend_pair: MemoryFactsDrawer guard badges — pattern class + token-delta visible to the user (PLAN-asserted; executor re-greps per W-7)

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/memory_guard.rs::MemoryWriteScanner (regex/aho-corasick write-time scan)
  - crates/tools/src/tool_result.rs::to_ui_dict + to_llm_content (dual serialization trait)
  - crates/tools/src/trace_query.rs::TraceQuery (on-demand range expansion tool)
  - +MemoryGuardEvent (Kind: PoisonBlocked/DualSerialized/TraceQueryExpanded) in ssb.rs / action_type_lock.rs
consumes_symbols:
  - crates/safety/src/provenance.rs::Provenance (IS-11: the safety-crate hook registration point) — GD-1
  - H-31/H-32 memory-crate firewall (compose; never re-filter what they cover) — GD-1
  - E-37 PromptArmor data-plane classify (compose; I-13's scan covers write-time only) — GD-1
  - G-9 redaction write-time gate (compose pattern; never re-implement) — GD-1
  - production_agent_rpc.rs::build_tool_catalog (IS-14: 3-arm pattern for TraceQuery registration) — GD-1
reachable_from:
  - mem0_dispatcher.rs write path -> MemoryWriteScanner::scan()  [PRODUCTION memory write path]
  - tool_result assembly -> to_llm_content()  [PRODUCTION context-assembly path]
  - model tool call -> TraceQuery::expand()  [PRODUCTION tool invocation path]
  - (test callers in contracts/ or #[cfg(test)] do NOT satisfy R7 production reachability)
stub_ban:
  - NO YARA engine added without anti-slopsquat verification + explicit user approval (IC-10; regex/aho-corasick ships first)
  - NO re-filtering of patterns already covered by H-31/H-32 or E-37 (compose only)
  - NO tool result entering context without passing through to_llm_content (dual-serialization invariant)
  - NO silent-swallow on a MemoryGuardEvent(PoisonBlocked) — the write must be REFUSED, not silently dropped
  - NO guard default-ON before GD-4 FP-rate bound is established on >=3 real runs
  - NO .unwrap()/.expect() on production scan or serialization paths
  - NO blake2/sha2 (blake3 only)
frontend_pair:
  MemoryFactsDrawer guard badges — required; PoisonBlocked events and token-delta from dual-serialization are user-visible security + efficiency signals (W-4/W-6).
observable_effect:
  trigger: "cargo run -p elai-cli -- run --task 'summarise this file' --emit-ssb  (with a fixture file containing invisible-Unicode injection pattern)"
  expect_ssb: "MemoryGuardEvent{kind: PoisonBlocked, pattern_class: 'invisible_unicode'} on the seeded malicious memory write; MemoryGuardEvent{kind: DualSerialized, token_delta: N} showing context-vs-dashboard size difference on a 500-file directory listing; MemoryGuardEvent{kind: TraceQueryExpanded, byte_range: ...} on a model TraceQuery call"
  expect_artifact: "the malicious write is REFUSED (no store entry, PoisonBlocked row with pattern class named); the 500-file listing reaches the dashboard complete but the journal shows condensed context form; a --no-guard broken arm passes the injection write and FAILS the fixture check (R2/R1)"

## Grounding
- IS-11 (capability_flow.rs taint_text:234; crates/safety/src/provenance.rs:131), IS-14 (build_tool_catalog:657 3-arm pattern)
- IC-10 (I-GROUNDING-MAP.md: compose H-31/H-32/E-37/MINJA first; YARA = new dep, user-approval required; regex/aho-corasick first)
- H-31/H-32 firewall, E-37 PromptArmor, G-9 redaction (all PLAN-asserted; executor re-greps)
- GD-4 (G-0_INDEX.md: FP/over-block rate bounded on >=3 real runs before default-ON)
- SSB: +MemoryGuardEvent (Kind: PoisonBlocked/DualSerialized/TraceQueryExpanded) — baseline+1 from LIVE 221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9)

## COMPOSES-WITH
H-31/H-32 auditor-objectivity firewall in the memory crate (never re-filter; add write-time scope only)
E-37 PromptArmor (data-plane classify; I-13 covers the write-time path it does not)
G-9 redaction write-time gate (pattern precedent)
IS-11 crates/safety/src/provenance.rs (the hook registration seam)
IS-14 production_agent_rpc.rs build_tool_catalog (the 3-arm TraceQuery registration pattern)
H-6 masking (different layer — never conflict; H-6 drops turns, I-13 scans writes)

End of I-13.
