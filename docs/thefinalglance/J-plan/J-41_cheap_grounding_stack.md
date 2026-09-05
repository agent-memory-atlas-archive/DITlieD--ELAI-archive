# J-41 cheap_grounding_stack

- Track J | lever grounding-determinism + local-economy | grounding arXiv:2404.10774 + arXiv:2506.20384 + research.perplexity.ai (closed-set citation) | nature build (activate F-12 OnnxNliJudge + extend H-53/E-35) | effort M | WAVE DD-3 | depends F-12, H-53, E-35, J-40
- Source-grounding must run WITHOUT a frontier LLM (local-lane + cost), and the user's token-hash instinct is right in spirit but wrong at granularity: token-level membership is vacuous (function/common tokens are in every source) and forbids legitimate synthesis. The sound form is a three-layer stack: a cheap deterministic EXTRACTIVE first pass (span fingerprinting), then a SMALL claim-level entailment model only on what the first pass flags, then CLOSED-SET citation binding so fabricated ("ghost") citations are structurally impossible. Activates F-12's empty OnnxNliJudge slot and extends H-53 + E-35 to a grammar-constrained closed-set citation decode.

## Steal/build
1. EXP-2 (OBSERVE-ONLY, step 1, no default-ON until passed): build 100 (source-snippet, generated-claim) pairs FROM THE ELAI CODEBASE (50 entailed / 50 not), run a candidate small grounding model (MiniCheck-FT5 770M, or Paladin-mini 3.8B), measure balanced accuracy. <75% = the model does NOT transfer to the code-claim domain without fine-tune (LLM-AggreFact is general text; code-domain transfer is the #1 residual risk). This decides whether the model slot is usable as-is.
2. Three-tier extractive fingerprint as a cheap FIRST PASS (never the sole gate): substring (exact, Rabin-Karp/KMP, full confidence) -> fuzzy (>=0.8 token-overlap / edit-distance) -> semantic (>=0.85 cosine). Token-level VACUOUS; sentence / n-gram>=5 meaningful; plagiarism floor 8-10 consecutive words. Flags fabrication candidates (claims with ZERO supporting span) for tier 2; passes verbatim-supported claims cheaply.
3. Claim-level entailment via the F-12 OnnxNliJudge slot: drop a small grounding model (MiniCheck-FT5 / Paladin-mini ONNX export) into `<root>/.elai_cc/models/onnx/` so the already-compiled `OnnxNliJudge` activates. ADAPTER NEEDED: `OnnxNliJudge` expects 3-class NLI (label_order [NliLabel;3]) but MiniCheck/Paladin are BINARY grounding (supported/unsupported) -- either add a `BinaryGroundingJudge` sibling implementing the `NliJudge` trait or map binary->NLI labels. Claim decomposition (fine-grained beats sentence-level by 10-15% coverage) via a small T5/BART extractor.
4. Closed-set citation binding (hard form) via a new decoding mode: add `DecodingMode::ClosedSet { allowed_ids }` that feeds the H-53 `EvidenceAnchorSet` id list as the grammar/token allowlist at decode time (invalid completions -> logit -inf), making ghost citations STRUCTURALLY impossible, not merely discouraged. Out-of-set citation -> D-16 conformal refusal (abstain), the documented quality cost of enforcement under partial evidence.
EXCLUDED: token-level hash membership (vacuous); generative claim rewriting; using a small NLI model as the SOLE gate on synthesized output (false-negatives dominate on paraphrase -- the extractive pass is necessary-not-sufficient, the entailment model handles paraphrase); RAG-HAT-style forced citation on partial evidence WITHOUT an abstention path (constraint amplifies confident fabrication -- abstention is mandatory, D-16).

## Existing ground truth
(grep-confirmed 2026-06-16 via grounding agent; re-grep at edit, W-7)
- `NliJudge` trait at `crates/memory/src/semantic_contradiction.rs:98` (`classify(premise, hypothesis) -> NliVerdict`); `DeterministicNliJudge` (Tier-1 lexical) WIRED; `OnnxNliJudge` at `crates/memory/src/nli_onnx.rs:60` (session/tokenizer/`label_order: [NliLabel;3]`), loads `<root>/.elai_cc/models/onnx/DEFAULT_NLI_MODEL_FILENAME`, returns Err (fail-safe SKIP) when the file is absent. CODE EXISTS + compiles; the .onnx model file is ABSENT (Tier-2 model staging).
- `EvidenceAnchorSet` at `crates/orchestrator/src/citation_prebind.rs:106`; `freeze_anchors(ranked, seed)` at `:192`; `check_citation_in_set(anchor_set, id, cited_excerpt)` at `:308`. WIRED; prod entry `prebind_verify_citations` in `citation_verifier.rs`. SSB `CitationAnchorsBound` (229) already exists.
- `ConstrainedDecodingConfig` at `crates/providers/src/decoding.rs:122`; `DecodingMode::Grammar { ebnf_path, crane_adaptive }` at `:155`; `default_decoding_for(RoleDecoding::Executor)` -> Grammar; `RoleDecoding` enum at `:394`. Seam: add a `ClosedSet { allowed_ids }` arm + attach in `default_decoding_for` gated on a non-empty `EvidenceAnchorSet`.
- GROUNDING VERIFICATION (phase-2B): 6 methods incl SOURCE_CHECK (read_file + regex) -- the substring/source first-pass already has a home.
- E-54 n-gram contamination filters (9/13-gram) -- in-tree span-fingerprint precedent.

## Integration
The three tiers compose as a pipeline: extractive fingerprint (cheap, deterministic, GROUNDING VERIFICATION SOURCE_CHECK + a new span matcher) -> OnnxNliJudge binary grounding (only on flagged claims) -> closed-set citation decode (H-53 anchors as the E-35 grammar allowlist) -> D-16 refusal on out-of-set / unsupported. Consumes J-40's ranked compressed spans as the evidence set (the harness-fetched, fingerprinted corpus). Reuses the shipped CitationAnchorsBound SSB row + GROUNDING VERIFICATION verdicts -- net 0 new action types. Honest residue: EXP-2 may show the general grounding model fails on code claims ("this fn is pure", "idempotent") -> then the slot needs a code-domain fine-tune before default-ON (recorded, not faked).

## WIRING_CONTRACT
produces_symbols: [ExtractiveGroundingPass, BinaryGroundingJudge (or NliLabel adapter), DecodingMode::ClosedSet, bind_closed_set_decode]
consumes_symbols: [NliJudge, OnnxNliJudge, EvidenceAnchorSet, check_citation_in_set, ConstrainedDecodingConfig, default_decoding_for, GROUNDING VERIFICATION SOURCE_CHECK, RefusalDecision (D-16)]
reachable_from: [the research/synthesis path in research_pipeline.rs (J-40 hands spans in); prebind_verify_citations production entry; the decode role-config call site]
stub_ban: [no token-level membership check, no SKIP-as-pass on missing model (missing model = explicit SKIP verdict surfaced, NOT a silent grounded), no forced citation without an abstention branch]
observable_effect: a citation cited outside the frozen EvidenceAnchorSet is REFUSED at decode (logit -inf) or post-hoc rejected -> a D-16 refusal / CitationAnchorsBound row in the journal; an unsupported claim flagged by tier 1 with no tier-2 entailment is logged as ungrounded
SSB: REUSE CitationAnchorsBound (229) + GROUNDING VERIFICATION verdict rows -- net 0 new action types. (If an extractive-pass telemetry row is wanted, prefer reuse; only add Pattern-2 if genuinely new, bumped from LIVE 268/577 at impl, W-9.)

## Grounding
- [MiniCheck: Efficient Fact-Checking of LLMs on Grounding Documents, arXiv:2404.10774](https://arxiv.org/abs/2404.10774) -- MiniCheck-FT5 770M matches GPT-4 grounding accuracy at 400x lower cost; Bespoke-MiniCheck-7B SOTA, >500 docs/min on one A6000; trained to recognize cross-sentence synthesis (raw DeBERTa-MNLI does not). VERIFIED 2026-06-16.
- [Paladin-mini: A Compact and Efficient Grounding Model, arXiv:2506.20384](https://arxiv.org/abs/2506.20384) -- 3.8B grounding model (Phi-4-mini); BINARY grounding framing (claim-supported-by-doc), NOT 3-class NLI -- the adapter reason. VERIFIED.
- [Perplexity Research: Query-Aware Context Compression](https://research.perplexity.ai/articles/query-aware-context-compression-for-better-snippets) + closed-set/graph-constrained citation (KG-Trie, invalid completions -> -inf): ghost citations structurally impossible. T1 vendor + the legal-AI KG-Trie pattern; design rationale. VERIFIED (sonnet oracle).
- NLI + lexical-match flag ~ GPT-4o 89.9% on long-form QA at orders-of-magnitude fewer params; DistilBERT-125M serviceable -- the cheap baseline (maps to the shipped DeterministicNliJudge Tier-1 + a lexical flag). (Survey-reported; confirm model/number before any headline claim.)
- Field-wide ghost-citation rates: free-gen 14-95%; partial-evidence 48%; unsupported-evidence 65% -- why closed-set binding matters. (Survey-reported, design rationale only.)

## COMPOSES-WITH
- F-12: activates the OnnxNliJudge slot (with the binary-grounding adapter).
- H-53: EvidenceAnchorSet is the closed set; reuses freeze_anchors + check_citation_in_set + CitationAnchorsBound.
- E-35: the constrained-decoding substrate (XGrammar); new ClosedSet decode mode.
- D-16: abstention/refusal on out-of-set or unsupported claims.
- J-40: supplies the harness-fetched, compressed, ranked spans as the evidence set.
- ELAI-DETERMINISM-DOCTRINE: the canonical "nothing the model asserts is trusted without a deterministic, model-independent check" application (grounding-side).

## Open questions / risks
- EXP-2 outcome gates the entailment tier: code-claim domain transfer is unproven; <75% balanced accuracy = fine-tune before default-ON, do not ship the general model as a gate on code claims.
- 3-class NLI vs binary grounding mismatch -- the adapter (BinaryGroundingJudge sibling or label mapping) is a numbered deliverable, not an afterthought.
- Compression (J-40) dropping a true claim's supporting span -> false ungrounded; tune J-40's span budget on recall, and let tier-2 entailment catch paraphrase the extractive tier misses.
- ClosedSet decode requires the provider/sidecar to support a token allowlist at decode (XGrammar/grammar path); confirm the local sidecar exposes it (E-35 substrate is PARTIAL).

## Next
The DD-wave trio (J-39/J-40/J-41) closes after EXP-1 + EXP-2 pass and each leg's observable_effect is gate-verified; promotion to default-ON follows the GD >=3-run FP-bound rule.
