steal-pillar-sota-2026-06-12 — verdict ledger for the 8-pillar SOTA recheck + Fallow multi-language + Top-50 bad-behaviors + skills-repos triage.

Source corpus: 12 Perplexity deep-research reports (one PE-executable decoy excluded) + 2 GitHub skills repos + 1 video transcript (matthadnoff) + 1 user research doc (Top 50 Bad Behaviors of LLMs). Digests at workflow/reasoning/pillar-sota-digests-2026-06-11/. Verification: 6 sonnet deep-dives (P5/P1/P2/P0/Fallow + 1 reused) + 3 haiku confirm/refute (P6/P7/P4) + 3 opus compressors + 1 opus skills reader + main-LLM direct WebFetch on 3 P7 papers. ~1.0M subagent tokens.

LIVE SSB baseline (main-LLM grep 2026-06-12, W-9): SCHEMA_VERSION = 231 (crates/orchestrator/src/ssb.rs:3084) / TOTAL_VARIANTS = 542 (crates/orchestrator/tests/action_type_lock.rs:671). The feature map's 229/540 and ALL-PLANS 226/537 are BOTH stale by 2-5. Re-grep at edit.

Expected-outcome check (handoff prediction was "a few HARDENs, maybe one REPLACE"): HELD. Most report findings independently re-derived the existing roadmap (confirmations). NEW grounded survivors = 9 plan units J-16..J-24. ONE REPLACE-class item (P5 serving engine) handled ADDITIVELY (no removal of a shipped component without user confirmation; CLAUDE.md). Several report claims REFUTED as hallucinations.

================================================================================
TIER 1 — ENTER-PIPELINE (became new J plan units, grounded)
================================================================================
J-16 BLAST-RADIUS / ANTI-REGRESSION GATE [KEYSTONE — the user's #1 priority]. The "small fix breaks
  something else" class. GROUNDED GAP (verified): there is NO published regression-test-selection or
  over-editing benchmark for LLM agent edits (verify-p4-context agent, 4 search iterations, zero
  named technique). ELAI has the substrate (B-29 symbol-level impact API impact_radius/call_flow,
  DKG, codegraph MCP, impact_radius.py, impact-mapper agent) but it is a PRE-EDIT ADVISORY for the
  main LLM, not a GATE. Top-50 cluster #14 (regression bugs, no dep map), #15 (over-editing, 1-line
  ask -> 200-line diff), #16 (silent deletion), #29 (unrequested improvements), #31 (symptom-not-
  root-cause). Grounds: AgentDebug arXiv:2509.25370 (failure isolation +24% all-correct, VERIFIED).
J-17 PLAN-LEVEL COMPOSITIONAL SAFETY (SIF) + sandbox config-audit. GROUNDED NEW GATE (verify-p2):
  per-step evidence gates structurally cannot catch a goal fragmented into jointly-benign subtasks
  (SIF arXiv:2604.08608 VERIFIED, AAAI 2026 Symposium; 71%/0%-FP on N=14 proof-of-concept — weak-ish
  sample, real mechanism). Our G-1 data-plane taint is a DIFFERENT layer (data flow, not plan
  composition). Fix = a narrow pre-dispatch compositional policy check over the decomposition DAG at
  the concepts->phases boundary. Sandbox: SandboxEscapeBench (arXiv:2603.02277, ICML 2026 Oral,
  VERIFIED) shows the real escape vector is MISCONFIG not kernel zero-day -> Priority-1 = remote
  sidecar config audit; Priority-2 = gVisor on the Linux remote. Grounds: SAFEFLOW arXiv:2506.07564,
  MAScope arXiv:2603.04469, Containment arXiv:2604.23425 (all VERIFIED).
J-18 P1 RETRIEVAL HARDENING. (1) CodeRankEmbed INT8 ONNX replacing the GraphCodeBERT-era code-
  embedding slot: VERIFIED dim=768 EXACT match to our HNSW slot (config.json hidden_size=768), MIT,
  137M, 8192 ctx claimed (max_trained_positions=2048 — test chunking), CoIR 60.1. Export ONNX
  ourselves from nomic-ai/CodeRankEmbed (skip the 162-dl/mo community quant). (2) Serena LSP 4th
  channel (github.com/oraios/serena, 25.2k stars MIT headless, VERIFIED) OR direct rust-analyzer
  stdio. (3) agentic-grep post-retrieval repair (DCI arXiv:2605.05242 VERIFIED). Extends E-17.
J-19 P5 vLLM-V1 ALTERNATIVE BACKEND (ADDITIVE, not a removal). The lone REPLACE verdict, handled as
  an additive RuntimeBackend (we have 11 + RuntimeAdapter trait, B-37) behind a flag. vLLM V1
  VERIFIED: native /v1/messages (zero Rust client change), AWQ+Marlin on SM 8.6 (RTX 3090),
  VLLM_BATCH_INVARIANT beta at SM>=8.0, cross-request prefix-cache KV reuse (the HF sidecar's missing
  win), XGrammar built-in. The serving-engine SWAP-OUT decision stays with the user (J-19 does not
  remove HF). Logits-chain portability documented (DoLa NOT-portable, already --no-dola live).
J-20 SELF-IMPROVEMENT BANDIT + SECURITY-INVARIANT PROMOTION GATE. Non-stationary Thompson (DS-TS
  arXiv:2305.10718, TS-CD arXiv:2009.02791 — both VERIFIED STRONG) replacing plain TS in the
  promotion path. Mandatory security-invariant suite per promotion (Misevolve arXiv:2509.26354
  VERIFIED: refusal 99.4->54.4%, ASR 0.6->20.6% after memory evolution). Cross-family verifier is
  CONFIRMATION of VD-4/H-45/D-13 (When-Does-Verification arXiv:2512.02304 VERIFIED). New file
  SUPERSEDING the plain-TS slices of D-5/D-29/H-48 (never amends them — see DECLINES note).
J-21 MEMORY SEMANTIC-VALIDATION + POISONING HARDENING. Citation-valid != semantically-valid: a
  refactor preserves a file:line while invalidating the stored claim. Add commit-aware / state-level
  validation (GEM state-trajectory model, "Is Agent Memory a Database?" arXiv:2605.26252 VERIFIED) +
  retrieval-time authorization + provenance-linked raw-evidence tier + TTL->activation-decay. Grounds
  (CONFIRMED REAL by main-LLM WebFetch): A-MEM arXiv:2502.12110 (NeurIPS 2025, write-time linking),
  Poison-Once/MPBench arXiv:2606.04329 (4 write channels, 9 vulns, prompt-injection defenses do NOT
  transfer to memory poisoning). Extends H-31/H-32/E-5. CITE PATTERNS NOT PLATFORMS (see DECLINES).
J-22 JOURNAL PAYLOAD-SCHEMA LOCK + writer-isolation contract. The single genuinely-new P0-journal
  item (verify-p0): serde #[serde(deny_unknown_fields)] + insta/snapshot on SSB payload STRUCTS —
  the golden-lock checks action_type NAMES but not payload FIELD shapes (a field rename/add slips
  through). Near-zero cost, closes a real drift hole. Plus a writer-isolation contract test asserting
  the SSB single-writer invariant (PunkGo's insight; we already have the structural property).
  Batched RFC-6962 Merkle checkpoint = noted as a RIDER on I-18, not re-authored here.
J-23 MULTI-LANGUAGE REVIEW-GATE CATALOG (Fallow-shaped) + frontend design/a11y gate. Fallow is
  JS/TS-ONLY (VERIFIED — must not be called multi-language). Shape (verify-fallow): a two-tier
  deterministic Layer-1 review backend — Tier-1 sarif_ingest (Semgrep CE / PMD-CPD / Psalm / detekt,
  all SARIF-emitting, VERIFIED), Tier-2 arch_gate TOML (dependency-cruiser / Knip / Deptrac /
  Periphery JSON+exit-code). Gates ELAI's OWN code first (Rust: cargo-modules/udeps/clippy; Py:
  vulture/ruff; TS: knip/dependency-cruiser). CodeQL LICENSE-BLOCKED for closed-source (note). Folds
  the de-AI-UI deterministic subset (design-token-drift check + axe-core a11y-tree gate). Extends
  VibeGate F-1/F-2. Shaped like the J-9 hardware catalog (data-driven, probe-once).
J-24 NON-PROMPT STEERING ACTIVATION + context-frugal tools/MCP. Answers the user's two questions as
  a real unit. (1) Steering without prompts IS possible and partly built: /v1/steering/* endpoints +
  ActAdd activation steering exist in the sidecar (phase-3J, DARK via --no-steering). Activate +
  MEASURE steering vectors as a behavior/personality lever that costs ZERO context tokens (vs role-
  prompt tokens). (2) Tools/MCP context cost: H-4 Tool Search Tool (77K->8.7K), H-34 MCP defer-
  loading, E-22 lazy MCP schema already plan the infra lever (load tool schemas on demand). J-24
  activates the steering leg + threads the deterministic P4 observation-size pre-assessment (BACM
  arXiv:2604.01664 VERIFIED, the NONE/SELECTIVE/ALL budget check — deterministic, allowed).

================================================================================
TIER 2 — MECHANISM-RIDERS (confirmation / already-planned; recorded, NOT new units)
================================================================================
- Cross-family secondary verifier on promotion = ALREADY VD-4 / H-45 / D-13. (When-Does-Verification
  2512.02304 confirms.) J-20 references, does not rebuild.
- Merkle checkpoint + Sigstore/Rekor anchor + in-toto/SLSA + RFC-3161 TSA = RIDERS on I-18/G-2/G-8
  (verify-p0 verdicts: RIDER-ON-EXISTING ×4, ALREADY-PLANNED ×2). Only the payload-schema-lock (J-22)
  is genuinely new.
- Writer isolation as a separate process = our I-1 directive-supervisor; for the JOURNAL specifically
  the delta is one contract test (J-22), not a rebuild.
- AWQ-Marlin replacing NF4 = E-27 (planned, checkpoint never built); J-19 carries it on the vLLM leg.
- Firecracker / WASM-WASI tool sandbox = E-15 (built) / forward-looking; J-17 ranks gVisor first for
  the actual deployment, Firecracker Q3-2026, WASI 1.0 ~2027.
- madsim DST = shipped (phase-3I); turmoil/mad-turmoil/Antithesis are alternatives, no port.
- Mem0/Zep/Letta swap = the P7 report ITSELF says don't (provenance collapse, poisoning, local-cost).
- Temporal durable execution = the P2 report self-rejects for local (needs external server).
- Frontier dual-vendor escalation pool (Opus 4.8 / GPT-5.5, API-not-browser) = refinement of E-31 /
  H-7 / I-49; the model IDs are unverified June-2026 hypotheticals (transcribed, not grounded).
  Recorded as a routing note; NOT a unit (rides E-31/I-49). If executed, verify model availability.
- SWE-Pruner (arXiv:2601.16746 VERIFIED, github yanami4/s-pruner, 0.6B NEURAL line-skimmer): selects
  lines without rewriting, runs below cache boundary, local. TENSION with IC-3 (deterministic
  compression only) / HD-2 (cache). Recorded as DEFERRED / opt-in-measured, NOT a unit — the
  deterministic BACM size pre-assessment (J-24) is the clean part; the neural skimmer needs a gated
  measured trial before it touches the context path.
- Skills-repos deterministic subset: the git-guardrails hook, cursor 1000-line-growth blocker, TS
  exhaustive-switch never-default, no-inline-imports, any/unknown/cast flagging, [DEBUG-*] leftover
  detection -> these are VibeGate F-2 / J-23 gate seeds. The taste rubric ("prefer boring maintainable
  over hacky/magical") is irreducibly LLM-judgment -> auditor/critic ADVISORY lens only (TD-3, I-32).

================================================================================
TIER 3 — REFUTED / HALLUCINATED (reports got these wrong; never act on them)
================================================================================
- "Qwen3-Coder 14B" (claimed ~58% SWE-Bench at ~9GB Q4): DOES NOT EXIST (verify-p5; QwenLM/Qwen3-
  Coder repo has no 14B dense Coder; smallest is 30B-A3B MoE). Report hallucination.
- "MAControl" (claimed Microsoft plan-level-security paper): DOES NOT EXIST as a paper name (verify-
  p2; the real paper is arXiv:2602.11416 PRUDENTIA "Optimizing Agent Planning for Security and
  Autonomy"). Report invented the label.
- "Retrieval tools degrade coding agents" framing of arXiv:2603.20432: REFUTED (verify-p1; the actual
  paper "Coding Agents are Effective Long-Context Processors" argues the OPPOSITE — tool+filesystem
  use beats semantic retrieval by 17.3%). The report mis-cited its own source's direction.
- 7 P7 memory PLATFORMS with ZERO source attribution (TierMem, MemIR, Eywa, RIG/SPADE, Prometheus,
  MemMachine, SimpleMem): UNVERIFIABLE (verify-p7 stalled; the platform names could not be confirmed).
  TREATED AS PATTERN-ONLY. J-21 cites the CONFIRMED arXiv mechanism papers (A-MEM 2502.12110,
  Is-Agent-Memory-a-DB/GEM 2605.26252, Poison-Once/MPBench 2606.04329) and the design PATTERN, never
  a platform name. Do NOT re-propose these 7 as platforms.
- JAIR SW-TS + IEEE ISTAS 2025 "Security Degradation" (37.6% figure): NOT-FOUND via public search
  (venues real, specific cites unconfirmed). Used as WEAK secondary grounding only; J-20's bandit leg
  rests on the VERIFIED DS-TS/TS-CD papers, the security-suite threshold is ELAI-calibrated not the
  37.6% literal.

================================================================================
ANSWERS TO THE USER'S DIRECT QUESTIONS (recorded for the handoff)
================================================================================
Q-skills-vs-harness: ELAI does NOT depend on prompts/skills the way mattpocock/skills (125,249 stars)
  + cursor team-kit do — both are PURE PROSE injected on a trigger word; exactly ONE deterministic
  artifact across 48 skills (a git-block bash hook). ELAI's enforcement (FSM gates, rules-engine in
  Rust, VibeGate, wiring gates, hooks) runs OUTSIDE the context window and cannot be ignored. Nuance:
  ELAI still uses prompts to STEER generation; you cannot steer what code a model writes purely from
  outside, only REJECT from outside. Split = enforcement-in-harness, steering-minimal-in-prompt.
Q-steer-without-prompts: PARTLY POSSIBLE AND PARTLY BUILT. Activation steering (ActAdd vectors via the
  sidecar /v1/steering endpoints, DARK) shapes behavior/personality with ZERO context tokens. Also:
  constrained decoding (op-emission only, HD-3), and finetune (deferred, E-47/E-55). -> J-24 activates
  the steering leg. "Are ELAI rules minimal and measured?" — minimal yes (rules are DATA in Rust, 0
  tokens, fire 100%); MEASURED not yet (SAFEDELTA/benchmarks unrun, G-6 BENCH-PENDING). Honest.
Q-tools-MCP-context: ELAI supports tools + MCP (C-39 bridge, MCP hardening E-24/G-4). They DO cost
  context, but the infra lever exists: H-4 Tool Search Tool (77K->8.7K tool-def), H-34 MCP defer-
  loading, E-22 lazy MCP schema — load schemas on demand, not all upfront. Fundamental floor: the
  model needs SOME schema in context to call a tool; the win is lazy/deferred + RAG-over-tools, not
  zero. -> J-24 threads the activation note.
Q-Matt-video-infra: his handoff skill = 5 lines of prose, lossy markdown to temp dir, manual, no
  enforcement. ELAI's infra equivalents: session DAG branch/merge + 85% handoff trigger (B-9/3H),
  H-51 journal-DERIVED digest (not recollection), E-25 ladder + H-6 deterministic masking (not
  summarize-and-pray), H-49 lossless time-travel. His "sediment layers degrade" complaint validates
  ELAI's lossless-journal-projection design (RV-2). What he does by hand, ELAI does as infra.

GROUNDING / VERIFICATION: all arXiv IDs WebSearch/WebFetch-verified or flagged WEAK/REFUTED above;
live SSB 231/542 grepped 2026-06-12; CodeRankEmbed 768d confirmed via config.json; vLLM /v1/messages
+ batch-invariant + AWQ-Marlin-on-SM8.6 confirmed via vLLM docs; Fallow JS/TS-only confirmed via repo.
Nothing committed (docs/workflow gitignored; zero .elai_cc code changes this session).
OUTCOME: 9 new units J-16..J-24 authored; J-plan now 24 sessions. Most reports = confirmation; 1
REPLACE handled additively (J-19); 4 report claims refuted. Realistic-outcome prediction held.
