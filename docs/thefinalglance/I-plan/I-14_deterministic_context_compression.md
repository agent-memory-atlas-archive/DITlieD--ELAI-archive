# I-14: deterministic context compression

- Track 14 — CACHE-INVARIANT ECONOMICS MIDDLEWARE | lever CHEAPER | grounding STRONG | nature BUILD | effort M | WAVE 2 | ID-3 | IC-2/IC-3
- Premise: context compression today has no middleware layer — full compiler panics, full tracebacks, and
  large AST dumps enter the model verbatim, burning tokens and breaking cache prefix stability.
  The corrected compression stack puts deterministic Tier-1 first (LSP-slice + tree-sitter AST fold),
  demotes neural compression to a gated Tier-2, bans cipher substitution, and journals every transform
  as a reversible receipt so replay reconstructs what the model saw exactly.

## Steal/build
IC-2 binding correction: LLMLingua-2-first (the source-doc design) is DEMOTED. An XLM-R-class encoder costs
local VRAM and is neural where deterministic cuts exist. Corrected Tier-1 = headless-LSP slicing (extract
exactly the AST node/declaration/failing line a compiler error references — the error already has a location)
+ tree-sitter AST folding (bodies of irrelevant functions -> "/* elided */", signatures/types intact;
BPE semantic priors are preserved because the signature tokens stay). LLMLingua-2-class token classification
is gated Tier-2 for prose-only payloads with a VRAM budget check gate.
IC-3 binding correction: n-gram abbreviation / cipher symbol substitution for code identifiers is KILLED —
cipher tokens break BPE semantic priors ("neural poison", critique §4-1). Numeric quantization for tabular
payloads survives as a narrow Tier-2 option.
IC-4 (transitively via H-2 bp.valid): every compression must preserve the blake3 prefix stability invariant —
a Tier-1 transform on the volatile tail must not touch the frozen prefix region (HD-2).
ID-3 receipt: every compression landing in context gets a ContextTransformReceipt SSB row (Kind: LspSlice /
AstFold / Tier2Compressed) whose reverse pointer re-expands to the original bytes via blake3 hash match.
EXCLUDED alternatives: n-gram cipher/abbreviation substitution (killed — IC-3); LLMLingua-2 as Tier-1
(demoted — IC-2); un-journaled transforms (banned — ID-3).

## Existing ground truth
- IN-6: NO compression middleware (no LLMLingua/compressor/api_middleware module) — I-14 is a genuine BUILD (grep-confirmed absent)
- E-52 LSP verifier lane: the rust-analyzer headless substrate (PLAN-asserted; I-14 shares the client, never re-implements) — executor re-greps per W-7
- knowledge tree-sitter grammars: the folding substrate (PLAN-asserted; executor re-greps per W-7)
- IS-6: context_budget.rs update_breakpoints:1507 (blake3 prefix stability) — the HD-2 invariant I-14 must not violate
- H-6 masking: different layer (H-6 drops turns; I-14 compresses payloads) — compose without conflict
- ID-3 (I-0_INDEX.md): JOURNALED-MIDDLEWARE; every middleware transform lands a reversible receipt; un-journaled transform is banned

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- crates/orchestrator/src/context_transform.rs (~250 LOC): ContextTransformer with two Tier-1 strategies — LspSliceTransform (headless rust-analyzer, error-location-to-AST-node extraction) and AstFoldTransform (tree-sitter; function bodies -> "/* elided */", keep signatures/types)
- ID-3 receipt emission: ContextTransformReceipt SSB row (Kind: LspSlice/AstFold/Tier2Compressed) with reverse pointer (blake3 hash of original bytes) and before/after token counts
- +ContextTransformReceipt (Kind: LspSlice/AstFold/Tier2Compressed) in ssb.rs / action_type_lock.rs
- integration hook at the tool-result -> context assembly seam (before context_budget.rs adaptive_reduce)
- fixture: seeded 10K-token compiler panic -> <=500-token slice + receipt with blake3 round-trip check

DEFER (Tier-2 / gated):
- LLMLingua-2-class encoder (new dep — anti-slopsquat + user approval; VRAM budget check gate; prose-only payloads only — IC-2)
- tree-sitter grammars for Python/TypeScript beyond Rust (requires grammar file acquisition; note as a follow-on)

## Integration
New module: crates/orchestrator/src/context_transform.rs (~250 LOC)
Hook point: tool-result -> context-assembly seam (before context_budget.rs adaptive_reduce; after the tool result is finalised)
Shared client: E-52 rust-analyzer headless client (compose; never re-implement the LSP wire protocol)
Shared grammars: knowledge crate tree-sitter grammar files (compose; never duplicate)
frontend_pair: per-transform receipt rows in the Transcript drawer (PLAN-asserted; executor re-greps per W-7)

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/context_transform.rs::ContextTransformer (Tier-1 + Tier-2-gated pipeline)
  - crates/orchestrator/src/context_transform.rs::LspSliceTransform (error-location -> AST-node extraction)
  - crates/orchestrator/src/context_transform.rs::AstFoldTransform (tree-sitter function-body elision)
  - +ContextTransformReceipt (Kind: LspSlice/AstFold/Tier2Compressed) in ssb.rs / action_type_lock.rs — ID-3 receipt
consumes_symbols:
  - E-52 LSP verifier lane rust-analyzer client (shared headless client — never re-implement) — GD-1
  - knowledge::tree_sitter grammars (shared grammar files — never duplicate) — GD-1
  - context_budget.rs::update_breakpoints (IS-6: the bp.valid / prefix-stability invariant to not violate) — GD-1
  - H-6 masking (different layer — compose without conflict; H-6 drops turns, I-14 compresses payloads) — GD-1
reachable_from:
  - tool_result finalisation -> ContextTransformer::transform()  [PRODUCTION context-assembly path]
  - (test callers in contracts/ or #[cfg(test)] do NOT satisfy R7 production reachability)
stub_ban:
  - NO cipher/abbreviation substitution for code identifiers (IC-3: neural poison — banned unconditionally)
  - NO un-journaled transform (every transform emits a ContextTransformReceipt — missing receipt = FAIL per ID-3)
  - NO LLMLingua-2 as Tier-1 (IC-2: demoted to gated Tier-2; Tier-1 is deterministic-only)
  - NO Tier-2 attempt without a VRAM budget check (fail-closed: refuse if headroom insufficient)
  - NO transform that touches the blake3-frozen prefix region (HD-2 invariant; Tier-1 operates on volatile-tail content only)
  - NO re-implementation of E-52 LSP client (compose the shipped headless client)
  - NO .unwrap()/.expect() on production transform or receipt paths
  - NO blake2/sha2 for the reverse-pointer hash (blake3 only)
frontend_pair:
  Per-transform receipt rows in the Transcript drawer — required; the before/after token delta and the reversibility link are user-visible audit signals (W-4/W-6).
observable_effect:
  trigger: "cargo run -p elai-cli -- run --task 'fix the compile error in src/main.rs' --emit-ssb  (with a fixture file producing a 10K-token compiler panic)"
  expect_ssb: "ContextTransformReceipt{kind: LspSlice, before_tokens: ~10000, after_tokens: <=500, reverse_hash: blake3(...)} for the compiler-error payload; receipt row is present (ID-3 check: zero receipts while transforms ran = FAIL per R1)"
  expect_artifact: "the journaled request shows <=500-token slice entering context (not the full 10K panic); the receipt's reverse pointer re-expands to the original bytes (blake3 match on the fixture); pass-rate on a fixed fixture set is unchanged vs uncompressed (measured against the fixture baseline — ID-5); a Tier-2 attempt without sufficient VRAM headroom is REFUSED and logged; an un-journaled-transform broken arm (transform runs but no receipt emitted) is caught by the post-transform receipt-presence check and FAILS (R1/ID-3)"

## Grounding
- IN-6 (compression middleware absent confirmed), IS-6 (context_budget.rs update_breakpoints:1507)
- IC-2 (LLMLingua-2 demoted to Tier-2; deterministic Tier-1 first — ADECO critique §1/§3/§4-1)
- IC-3 (n-gram cipher/abbreviation killed — neural poison; numeric quantization narrow Tier-2 only — critique §4-1)
- ID-3 (I-0_INDEX.md: JOURNALED-MIDDLEWARE; reversible receipt required; replay must reconstruct what model saw)
- HD-2 (H-0_INDEX.md: nothing busts the byte-stable cache prefix; a cache-bust is a test failure)
- E-52 LSP verifier lane, knowledge tree-sitter grammars (PLAN-asserted; executor re-greps)
- LLMLingua-2 (MSFT): WebSearch-confirm availability + license + current version before adding as Tier-2 dep (anti-slopsquat; only after user approval)
- SSB: +ContextTransformReceipt (Kind: LspSlice/AstFold/Tier2Compressed) — baseline+1 from LIVE 221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9)
FULL-FORM EXPANSION REQUIRED before EXECUTING (I-0 reading order rule — effort M; multiple cross-crate seams).

## COMPOSES-WITH
E-52 LSP verifier lane (rust-analyzer headless client — shared; never re-implement the LSP wire protocol)
knowledge crate tree-sitter grammars (shared grammar files)
context_budget.rs update_breakpoints / H-2 (the prefix-stability invariant; I-14 must not violate HD-2)
H-6 masking (different layer — co-exist, do not conflict)
ID-3 JOURNALED-MIDDLEWARE doctrine (the receipt pattern I-14 implements)

End of I-14.
