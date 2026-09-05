# I-34: CRANE hybrid + grammar pinning — free thinking tokens, pinned grammars, post-parse re-validation

- Track 18 — DETERMINISTIC-OVER-PROMPT HARDENING | lever TRUST-CANT-LIE | grounding STRONG | nature BUILD | effort M | WAVE 1 | IC-18

- Premise: the shipped crane_adaptive flag (IS-12, providers/src/decoding.rs:73-82) has no runtime leg.
  Constrained decoding without CRANE-style free thinking degrades reasoning 10-15pp (structure-snowballing
  arXiv:2604.06066). XGrammar-class engines emit silently non-compliant outputs (38 silent failures in
  JSONSchemaBench). Grammars are an attack surface (CDA: EnumAttack 96.2% ASR). All three problems have
  deterministic fixes that compose the shipped substrate.

## Steal/build
Three hardening layers on the IS-12 substrate:

(1) CRANE-style grammar augmentation on the local sidecar lane: thinking tokens are PERMITTED between
structural commitments (the model reasons freely, then commits to a structured token at commit points).
Reflection/self-correction loops get constraints on FINAL output only (structure-snowballing defense:
tight grammars on intermediate reasoning steps are the measured cause — IC-18 binding). The crane_adaptive
flag gets the runtime that IS-12 notes is absent. C-37 re-evaluation pilot consumes this surface.

(2) Grammar pinning as a control-plane artifact: every grammar file is blake3-pinned in a manifest (the
same MCP-schema pinning pattern from E-24). Third-party grammars are NEVER loaded. Enum-bearing schemas
from untrusted sources are REJECTED at load. The pin manifest is R8-frozen (the running system may not
modify its own grammar manifest). GrammarGateEvent(PinViolation) is emitted on any tamper.

(3) Post-parse schema re-validation on every constrained output: after the engine yields a token sequence,
the raw output is re-parsed against the schema. Silent non-compliance (the engine thinks it complied but
the output fails re-validation) is caught and routed to D-16 refusal instead of silently corrupting
downstream. The --trust-mask broken arm (which skips re-validation) is the fixture's negative case.

Loading a third-party grammar is the EXCLUDED alternative.
Trusting the mask output without post-parse re-validation is the EXCLUDED alternative (landmine #7).

## Existing ground truth
- IS-12: providers/src/decoding.rs DecodingMode::Grammar{ebnf_path, crane_adaptive} :73-82 — crane_adaptive FLAG ships; CRANE runtime absent (grounding map, grep-confirmed)
- PLAN-asserted: E-35/E-18 XGrammar substrate (constrained decoding pipeline); executor re-greps per W-7
- PLAN-asserted: C-26 DecodingMode and the config surface; executor re-greps per W-7
- PLAN-asserted: C-37 re-evaluation pilot (the CRANE consumer); executor re-greps per W-7
- PLAN-asserted: E-24 blake3 pin precedent (MCP schema pattern to mirror); executor re-greps per W-7
- IC-18 (grounding map): CRANE posture, structure-snowballing defense, post-parse mandatory
- Landmine #7 (I-0 §CARRIED WEB LANDMINES): XGrammar silent non-compliance; post-parse is mandatory
- Landmine #8 (I-0 §CARRIED WEB LANDMINES): CDA attack surface; grammars are pinned control-plane artifacts

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- providers/src/decoding.rs: CRANE runtime leg (thinking-token gate between commit points)
- Grammar pin manifest + load-time blake3 verification + GrammarGateEvent(PinViolation) on tamper
- Post-parse schema re-validation: re_validate_output() called after every constrained generation
- GrammarGateEvent SSB rows (CranePassed / PinViolation / PostParseRejected)
- Fixture: mutated grammar file (-> PinViolation), seeded silent-non-compliant output (-> PostParseRejected), reasoning-heavy CRANE fixture (pass-rate >= FreeText arm — measured per ID-5)
- Grammar-pin status in the decode panel (frontend_pair)
- decision doc: workflow/reasoning/I-34-decisions.md

DEFER (Tier-2 / gated):
- Python sidecar mask layer CRANE integration — requires sidecar boot (separate session, hardware-heavy flag per CLAUDE.md)
- Cross-language verifier-pack for grammar compliance — noted Tier-2 in I-17

## Integration
- MODIFY providers/src/decoding.rs: implement crane_adaptive runtime (the IS-12 flag finally gets a body); add re_validate_output()
- new: grammar store with pin manifest (~80 LOC), R8-frozen (gate_freeze_check.py covers it)
- MODIFY Python sidecar mask layer — Tier-2, see above
- seam to D-16 refusal: PostParseRejected routes to the existing refusal path (GD-1 compose)
- seam to C-37 re-evaluation pilot: the pilot calls into the CRANE runtime leg
- frontend_pair: grammar-pin status chip in the existing decode panel (Svelte + Tauri read); cross_lang_diff.py PASS (W-4)

## WIRING_CONTRACT
produces_symbols:
  - providers/src/decoding.rs::crane_adaptive runtime (the missing leg of IS-12)
  - providers/src/decoding.rs::re_validate_output() (post-parse schema re-validation)
  - grammar pin manifest + load-time verification (new module, path TBD at executor re-grep — W-7)
  - ssb action_type::GrammarGateEvent (Kind: CranePassed / PinViolation / PostParseRejected) — baseline+1 from LIVE 221/532 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9)
consumes_symbols:
  - providers/src/decoding.rs::DecodingMode::Grammar{ebnf_path, crane_adaptive} :73-82 (IS-12 — the flag the runtime legs into; GD-1)
  - E-35/E-18 XGrammar substrate (the constrained-decoding pipeline; GD-1)
  - C-26 DecodingMode config (PLAN-asserted; GD-1)
  - E-24 blake3 pin pattern (the manifest format to mirror; GD-1)
  - D-16 refusal path (PostParseRejected route; GD-1)
reachable_from:
  - DecodingMode::Grammar -> crane_adaptive runtime -> GrammarGateEvent(CranePassed) SSB row  [PRODUCTION local-decode surface]
  - grammar manifest load -> blake3 verify -> GrammarGateEvent(PinViolation) on tamper  [PRODUCTION decode-init surface]
  - post-parse re_validate_output -> GrammarGateEvent(PostParseRejected) -> D-16 refusal  [PRODUCTION output-validation surface]
  - test callers in crates/contracts/tests/ do NOT satisfy R7
stub_ban:
  - NO loading a third-party grammar (the pin manifest is the sole authority)
  - NO trusting the mask output without calling re_validate_output() (silent non-compliance is the measured failure mode — landmine #7)
  - NO constraints on intermediate reasoning steps (CRANE-style: free tokens between commit points, constraint on FINAL output only)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2 (blake3 only per the pin manifest)
frontend_pair:
  grammar-pin status chip in the existing decode panel — Svelte chip + Tauri read; cross_lang_diff.py PASS required (W-4).
observable_effect:
  trigger: "elai run --crane-fixture --seed 42 (reasoning-heavy fixture: CRANE mode vs FreeText arm + mutated grammar + seeded non-compliant output)"
  expect_ssb: "GrammarGateEvent rows: CranePassed for the compliant fixture (reasoning pass-rate field >= FreeText arm — measured per ID-5), PinViolation for the mutated grammar file, PostParseRejected for the seeded silently-non-compliant output (all under ELAI_SEED, GD-2)"
  expect_artifact: "the --trust-mask broken arm (skips re_validate_output) ships the bad output and FAILS its contract test (R2 — missing evidence is hard fail); grammar-pin status renders in the decode panel (cross_lang_diff.py PASS)"

## Grounding
- IS-12 (grounding map): crane_adaptive flag ships, runtime absent — confirms BUILD scope
- IC-18 (grounding map): CRANE posture, structure-snowballing, post-parse mandatory
- Landmine #7 (I-0): XGrammar 38 silent failures in JSONSchemaBench — post-parse is mandatory
- Landmine #8 (I-0): CDA EnumAttack 96.2% ASR, DictAttack 75.8-99.5% — grammars are control-plane artifacts
- Constrained-decoding dossier: IC-18 source; structure-snowballing arXiv:2604.06066 WebSearch-confirm before headline
- CRANE arXiv:2502.09061 — WebSearch-confirm before headline
- XGrammar arXiv:2411.15100, JSONSchemaBench arXiv:2501.10868 — WebSearch-confirm before headline
- SSB: +GrammarGateEvent (Kind: CranePassed/PinViolation/PostParseRejected) — baseline+1 from LIVE 221/532 as of 2026-06-10; re-read at edit (W-9)

## COMPOSES-WITH
- IS-12 providers/src/decoding.rs::DecodingMode::Grammar — the flag the runtime legs into; do NOT re-implement the decoding pipeline
- E-35/E-18 XGrammar substrate — compose the mask layer, do NOT re-implement
- E-24 blake3 pin pattern — mirror the MCP-schema pinning approach, do NOT re-implement the pin protocol
- D-16 refusal path — the PostParseRejected destination; compose, do NOT fork
- C-37 re-evaluation pilot — the CRANE runtime consumer; C-37 composes I-34, not the reverse

## J-15 AMENDMENT — llguidance Rust-side grammar compile+validate+pin leg (source-cited)
Grounding: determinism triage ledger item 3
(workflow/reasoning/steal-determinism-repos-2026-06-11.md); guidance-ai/llguidance crates.io
anti-slopsquat verification 2026-06-11 (v1.7.6, MIT, ~525K total downloads, Moskal/MS origin);
detllm-source-interface.md (the blake3 pin-in-same-stack-as-RRA pattern, workflow/papers/).

### The steal: orchestrator-side ParserFactory (Option A)
Add `llguidance` (crates.io v1.7.6, MIT) as an orchestrator-side dep for Rust-native grammar
compile + validate + blake3-pin at PLAN TIME, not inference time. The `ParserFactory` compiles
the grammar spec into a validated parser; malformed grammars fail FAST at plan time (before any
inference call), and the grammar blake3 pin is computed in the same hash stack as the RRA
(G-2), so a grammar drift between plan and execution is detectable on `replay --verify`.

### Architecture decision (Option A only; Option B REJECTED)
- Option A (STEAL): llguidance on the orchestrator side for compile+validate+pin; the sidecar's
  XGrammar is UNCHANGED for inference-time token masking. The pin is a plan-time artifact; the
  mask is a runtime artifact. Clean separation, no sidecar inference-loop redesign.
- Option B (REJECTED): full Rust-side masking via llguidance's own token-level constraints.
  REJECTED because it requires a sidecar inference-loop redesign (the sidecar's
  LogitsProcessor chain is Python-side; routing grammar constraints through it from Rust would
  duplicate XGrammar's masking and break the E-35 substrate contract). HD-3 also bans
  constrained decoding on reasoning/coder/planner roles — the Rust leg is compile+validate only.

### Compiled-state serialization note
llguidance's compiled parser state is NOT serializable (README-verified). This is NOT a problem:
recompile from the grammar spec each boot. Compilation is cheap (sub-second for JSON-schema
grammars per the llguidance README benchmarks). The serializable artifact is the grammar SPEC +
its blake3 pin, not the compiled parser. The RRA carries the spec + pin; the executor
recompiles on load.

### NEW DEP — user confirmation required (house rule)
`llguidance` is a NEW Cargo dependency. Per the house rule (no new deps without explicit user
confirmation), the I-34 executor MUST re-verify the crates.io anti-slopsquat snapshot at
execution time (never trust this recorded snapshot) and obtain user confirmation before adding
the dep. Verification snapshot (2026-06-11, re-verify at edit): crates.io API confirms
`llguidance` v1.7.6, MIT license, ~525K total downloads, maintained by the guidance-ai org
(Moskal/MS). xgrammar-rs was considered as the alternative and REJECTED: C++ binding (not pure
Rust), ~5.6K downloads (93x fewer than llguidance), and duplicates the sidecar's XGrammar
surface rather than complementing it.

End of I-34.
