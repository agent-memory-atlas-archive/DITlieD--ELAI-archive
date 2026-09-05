# H-Plan Index — the EFFICIENCY + CANVAS + TRUST + UX/MEMORY + GROUNDED-REASONING + DGM-VERIFICATION + REVERSIBILITY + CONTEXT-LIFECYCLE + REVIEW-ANCHORING + EXTERNAL-STEALS side, wiring-enforced (54 sessions; Tracks 1-4 + Track 6 + Track 7 + Track 8 + Track 9 + Track 10 + Track 11 authored; Track 5 = odysseus, reserved)
Date: 2026-06-03 (Track 1/2); 2026-06-04 (Track 3 — TRUST-HARDENING added)
Source synthesis:
  workflow/reasoning/ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md (the four-pillar engineering
    direction + the L1..L8 efficiency levers + the F-E1..F-E9 wire-dark fix list = Track 1)
  .elai_cc/crates/dashboard-v2/CANVAS-WORKBENCH-DESIGN.md (the canvas-workbench that replaces the
    orbit, its C1..C6 build plan + the §15-21 spawn_view reply-channel = Track 2)
  D:\.REGRET_workflow\memory-bank\plans\elai-trust-hardening-design.md (the T-1..T-13 trust-hardening
    design: deterministic gates for trusting an AUTONOMOUS trading-strategy search/build loop = Track 3)
Grounding map (W-7 source, re-grepped by authors + validator):
  docs/thefinalglance/H-plan/_grounding/H-GROUNDING-MAP.md (Track 1/2 — 17 grounding streams: 12
    code-state readers + 5 web verifiers, run wf_6b48bc93-104, 2026-06-03).
  docs/thefinalglance/H-plan/_grounding/H-TRUST-GROUNDING-MAP.md (Track 3 — the trust substrate map:
    TS-1..TS-8 substrate-shipped corrections + NN-1..NN-13 net-new confirmations + TD-1..TD-5 deltas +
    the TRADING-LANE TIERING block + the H-17..H-29 symbol tables, live elai.map greps 2026-06-04).
  docs/thefinalglance/H-plan/_grounding/H-UX-MEMORY-GROUNDING-MAP.md (Track 4 — UX + USER-MEMORY: the
    crates/memory substrate map + QOL anchors + odysseus prior-art + the confirm-at-authoring list; research
    run wf_c8950d72-45e + author run wf_02516dd9-e82, live elai.map greps + crates/memory read 2026-06-04).
  docs/thefinalglance/H-plan/_grounding/H-REASONING-GROUNDING-MAP.md (Track 6 — GROUNDED-REASONING: RS-1..RS-11
    shipped reasoning/verification substrate + RG-39..RG-42 gaps + RD-1..RD-4, live elai.map greps 2026-06-04).
  docs/thefinalglance/H-plan/_grounding/H-VERIFICATION-GROUNDING-MAP.md (Track 7 — DGM-VERIFICATION: DG-1..DG-7
    shipped DGM-loop substrate + PC-1..PC-8 prior-plan composition + VD-1..VD-5 + SCHEMA + CITATIONS, live
    crates/orchestrator greps 2026-06-05).
Predecessors: B-plan, C-plan, D-plan, E-plan, F-plan, G-plan. H COMPOSES-WITH them (GD-1); restates none.
Track 3 NOTE: the trust-hardening design calls itself a future "T-plan"; per the directing instruction it is
  authored HERE as H-plan TRACK 3 (H-17..H-29). The doctrinal fit is exact — the design's D1 = wiring R1/R2/R4,
  D3 = gate-freeze R8, D5 = verify-the-verifiers — so Track 3 inherits the same fail-closed off-agent-artifact
  done-gate the F/G/H Track-1/2 plans use (W-1..W-10 + GD-1..GD-5), plus the TD-1..TD-5 deltas below.

Predecessor reading (mandatory before any H-N execution):
  docs/thefinalglance/H-plan/_grounding/H-GROUNDING-MAP.md (THE grounded symbol map — read your
    session's slice + CANONICAL CONSTANTS + DESIGN-DOC CORRECTIONS + WEB VERIFICATION before coding)
  docs/thefinalglance/F-plan/F-0_INDEX.md (the W-1..W-10 wiring doctrine this plan INHERITS verbatim)
  docs/thefinalglance/G-plan/G-0_INDEX.md (the GD-1..GD-5 deltas H also inherits where applicable)
  This file (H-0_INDEX.md), then H-SUMMARY_INDEX.md, then the relevant H-N detail file.
  Track-1 only: workflow/reasoning/ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10-§16.
  Track-2 only: .elai_cc/crates/dashboard-v2/CANVAS-WORKBENCH-DESIGN.md (+ DESIGN.md for pixels).

## Why H-plan exists (read this before anything else)

The F-plan and G-plan built the TRUST spine (WRITE-side gates + READ/EVIDENCE/MEASUREMENT). H-plan
is the two remaining "new/improvement" bodies of work the trust spine sits underneath:

TRACK 1 — EFFICIENCY (H-1..H-9). The engineering-direction record (2026-06-03) reframed the thesis:
ELAI is a deterministic verification-and-orchestration harness wrapped around a CLOUD flagship that is
provably CHEAPER on the SAME model than Claude Code / Cursor / Codex. The cost program is TWO orthogonal
stacks — INPUT (prompt caching + cache-stable layout + masking + tool-search + batch) and OUTPUT (diff/
FIM/streaming, the 5x-billed lever ELAI most neglected). Almost every win is ACTIVATION of wire-dark
substrate, not invention (the grounding map confirms it per session). This track catches ELAI up to
caching parity and then beats heuristic editors on the ONE axis a deterministic + journaled harness can:
a byte-stable, ASSERTED, reproducible cache prefix (which a heuristic agent structurally cannot guarantee).

TRACK 2 — CANVAS-WORKBENCH (H-10..H-16). The orbit (3D galaxy, watch-not-work) is replaced by a 2D
generative spatial work-surface the model drives and the user points at: one canvas, typed nodes/edges
from real relationships, instanced five ways (CODE-GRAPH / PLAN / RESEARCH / ARCHITECTURE / DESIGN),
plus the §15-21 reframe — the canvas as the model's general-purpose visual reply channel (spawn_view).
A canvas alone is now table-stakes (Anthropic/OpenAI/Vercel/tldraw/MCP all ship one); the MOAT is the
journaled, reproducible, taint-tagged op-stream UNDERNEATH it — i.e. the same trust spine F/G already
bet on, applied to a new surface. That is why H-15 COMPOSES-WITH G-1 (taint) + G-2 (RRA), restating
neither.

TRACK 3 — TRUST-HARDENING (H-17..H-29). The NORTH STAR: when ELAI later runs an AUTONOMOUS system that
searches for / builds trading strategies, the operator must TRUST both the LLM-WRITTEN code AND the
AUTONOMOUSLY-DISCOVERED strategy — and that trust must be enforced by DETERMINISTIC ELAI mechanisms
(rules-engine predicates, gates, runtime verifiers reading off-agent artifacts), NOT by .md rules an LLM may
ignore. The structural threat (design PART 0) is the SEARCH ITSELF, not only a lying LLM: an optimizer that
maximises a financial metric ships overfit/leaked/fabricated edge EVEN IF EVERY AGENT IS HONEST (the max of N
noisy Sharpe ratios is upward-biased; an in-sample Sharpe > 1.0 appears after ~7 pure-noise trials — Bailey &
Lopez de Prado). Three trust-breakers, closed in priority order: TB-1 selection-bias-under-search (the keystone
gap — H-17 trial ledger + H-18 DSR/PBO + H-19 holdout budget), TB-2 look-ahead/leakage ("compiles + tests pass"
is structurally blind to it — H-20 leakage taint + H-21 runtime differential + H-22 trading nulls), TB-3
self-evaluator capture (a self-mod optimizer routes itself to PASS — H-27 verify-the-verifiers + H-28
reward-tampering, with H-23/H-24/H-25 reproducibility+parity+anti-fabrication and H-26 dynamic value verification
and H-29 deployment drift in between). THE TRACK-3 CENTRAL FINDING (grounded 2026-06-04 vs live elai.map): the
fail-closed off-agent-artifact substrate is ALREADY SHIPPED (rules engine, SSB WAL, taint primitive, RRA +
determinism, mutation + metamorphic, DGM guards + LockedKeysAuditor, F-4 stdout_hash, seccodeplt/SAFEDELTA,
model_drift) — the G-SUMMARY's "ABSENT" on RRA/etc is STALE; the finance/search-specific GATES (trial ledger,
Deflated-Sharpe, leakage predicate, trading nulls, etc.) are the genuine BUILD. So Track 3 is BUILD-ON-SHIPPED,
not build-from-nothing (see H-TRUST-GROUNDING-MAP TS-1..TS-8 + NN-1..NN-13).

TRACK 4 — UX + USER-MEMORY (H-30..H-38). The instrument-first body of work (grounded 2026-06-04 via a
15-lens internet sweep + a read of the shipped crates/memory substrate; research run wf_c8950d72-45e +
author run wf_02516dd9-e82). ELAI engineers every LLM-problem but ships almost no user-facing UX and ZERO
durable USER memory. Track 4 closes that under the SAME wiring done-gate: H-30 is an INSTRUMENT-FIRST keystone
that must MEASURE a non-stale, token-cheap memory (a stale_fact_reached_prompt ratio + a belief-revision eval)
BEFORE H-31/H-32 are trusted; H-31 + H-32 ACTIVATE the already-shipped but wire-dark crates/memory bi-temporal
substrate (soft_close / audit_conflicts / FactType::Preference / valid_from_ms-valid_to_ms / mem0_dispatcher /
get_relevant_facts — PROD-WIRED on the WRITE/contradiction side, dark on the capture + retrieval-into-prompt
side) into a two-tier user memory (Tier-B preferences + Tier-A code-grounded validate-before-use). The staleness
fix is at WRITE time (bi-temporal supersession; retrieval HARD-FILTERS superseded rows, NEVER narrates
[OUTDATED] = the measured Graphiti +68%-token trap; DELETE banned for replay), contradiction-detection runs on
the CLOUD on WRITES ONLY (spare the 7B), and the AUDITOR/CRITIC/ADVERSARY roles get ZERO user-memory injection
(Personalization-Trap arXiv:2510.09905) to protect the trust product. H-33..H-38 are host-side governance + 5
"free-win" QOL surfaces (memory hub, MCP defer-load, /rewind over the journal, completion notifications,
plan-approval gate, cost-confirm) that ride existing substrate at ~zero local-token cost. odysseus
(docs/.source/odysseus-main, MIT) is the reimplement-reference for the memory + tool-index patterns
(docs/.source/odysseus-INDEX.md + odysseus_py.map = the map). Explicitly NOT a personal agent: work-facts only,
never a persona. Track 5 (the 6 odysseus steals + minor refinements) is SEQUENCED AFTER Track 4.

TRACK 6 — GROUNDED-REASONING HARDENING (H-39..H-42). Authored 2026-06-04 from the Hallucination-Stations research
thread (the YouTube explainer AIYQp1n51ZI + arXiv:2507.07505 ingested, then a 3-angle deep-research sweep over the
formal transformer-expressivity / hallucination-inevitability / verification-vs-generation literature; interface maps
in workflow/papers/, symbol map _grounding/H-REASONING-GROUNDING-MAP.md). The paper/video claim a hard O(N^2*d)
per-pass ceiling forces hallucination, verifiers share it (peer-verification "unsound"), and CoT "buys nothing". The
rigorous literature REFUTES the central CoT claim (poly CoT = EXACTLY P, 2310.07923; T steps simulate a size-T circuit,
2402.12875; GD provably learns NC1 with CoT, 2511.07378 — the bound is on the single PASS ~TC0, NOT the autoregressive
loop) and shows the COMMON hallucinations are low-complexity, sustained by EVAL INCENTIVES rewarding confident guessing
over abstention (OpenAI 2509.04664). VERDICT (anti-sycophantic): ELAI's trust spine is ALREADY strongly aligned — D-10
select_grounding_mode routes sound-checker-first (grounding_mode.rs:157) + firewalls the auditor; D-11/D-16/D-17/E-6/E-19
are SHIPPED. Track 6 is FOUR small ACTIVATION/EXTENSION refinements + instruments on shipped code, NOT a redesign: H-39
abstention-aware promotion+eval reward (the KEYSTONE — the one real cross-subsystem misalignment), H-40 decorrelation-keyed
verifier retention (Weaver 2506.18203), H-41 the live checker-coverage + checker-gap instrument (Kambhampati 2402.08115 +
the Xu 2401.11817 Table-3 taxonomy), H-42 serial-depth-scaled reasoning budget + the depth-not-width guard (2503.01544 /
TC0). The 3 inevitability papers (2509.04664 / 2401.11817 / 2502.12187) were evaluated for standalone plans and DECLINED —
they GROUND H-39 + reinforce existing doctrine (Xu folded into H-41, Suzuki into GD-4). Track 6 inherits W-1..W-10 +
GD-1/2/4/5 + HD-1..HD-5 and ADDS RD-1..RD-4 (below). NUMBERING: Track 5 (odysseus) stays RESERVED + UNAUTHORED; Track 6's
files take H-39..H-42 and Track 7's take H-43..H-48, so Track 5 when authored starts at H-55 (collision-safe; Track 8 REVERSIBILITY took H-49, Track 9 CONTEXT-LIFECYCLE took H-50/H-51, Track 10 REVIEW-ANCHORING took H-52, Track 11 EXTERNAL-STEALS took H-53/H-54).

TRACK 7 — DGM SELF-IMPROVEMENT VERIFICATION HARDENING (H-43..H-48). Authored 2026-06-05 from the self-verification
dossier (workflow/reasoning/ELAI-SELF-VERIFICATION-RESEARCH-DESIGN-2026-06-05.md — 6 background research agents, every
arXiv id + repo WebFetch-verified; 3 keystone cites re-confirmed via WebSearch this session) + the E-19 SE-Agent L4
relocation. The META-FINDING the whole track rotates around (Self-Verification-Prevents-Collapse NeurIPS X5Hk8aMs6w +
Solver-Verifier Gap 2507.00075 + Skalse 2209.13085 + Co-rewarding 2508.00410): A RECURSIVE SELF-IMPROVEMENT LOOP
WITHOUT A REAL, UN-GAMEABLE, STRONGER-THAN-GENERATOR VERIFICATION FILTER PROVABLY COLLAPSES; WITH ONE, IT PROVABLY
CONVERGES. ELAI's DGM-as-shipped is the DIVERGENT branch because its verification FILTER is two placeholders, grep-
confirmed LIVE this session: derived_alpha (darwin_admit_loop.rs:186) is literally blake3(config_bytes) normalized to
[0,1] — a function of candidate BYTES, not PERFORMANCE (GAP-1, P0); score_candidate_for_traps (trap_tasks.rs:123) is a
keyword substring grep the code self-flags "unsound" (GAP-2, P0); promotion tau is "UNCONFIRMED carries forward"
(promotion_gate.rs, GAP-3); the BAI bandit is scaffold-only on a placeholder reward (GAP-4); the eval trust == the eval
placeholder (GAP-5). Track 7 replaces the FILTER: H-43 execution-grounded fitness (CodeRM 2501.01054 + CodeT
2207.10397 — the KEYSTONE; a failing-code config must score LOW), H-44 behavioral trap detection with MEASURED recall
(ImpossibleBench 2510.20270 + Isomorphic-Perturbation 2604.15149 + CodeHalu 2405.00253 + TRACE 2601.20103 recall), H-45
worst-case + lineage-diverse promotion scoring + PRMBench-calibrated tau (Coste 2310.02743 + Eisenstein 2312.09244 +
Co-rewarding 2508.00410), H-46 HGM clade-metaproductivity parent selection (2510.21614), H-47 the DGM eval firewall
(date-gate LiveCodeBench 2403.07974 + template-regen GSM-Symbolic 2410.05229 + Thresholdout 1506.02629 binding —
COMPOSING D-6/E-54/H-19, NOT restating), H-48 the BAI bandit ACTIVATED on the real reward (Discounted-TS 2305.10718 +
ExIt 2509.04575 — COMPOSING D-5/D-29). THE TRACK-7 CENTRAL FINDING (grounded 2026-06-05): most of the dossier's PART D
is ALREADY PLANNED (D-2 PPM, D-5/D-29 BAI, D-6 held-out store, D-13 adversary, E-6 multi-agent, E-53 Verus proof gate,
E-54 corpus) — so Track 7 is replacing TWO PLACEHOLDERS + a few compositional upgrades, the genuine net-new being the
execution-grounded fitness + the behavioral detector. E-53 (Verus/Kani) is COMPOSE-NOT-AUTHOR (the narrow CSQ-Critical
proof ceiling, VD-5; VERINA 4.9% / CLEVER 0.6% = never routine); SE-Agent recombination stays E-19 Task 11 (H-43's
fitness makes it runnable). Track 7 inherits W-1..W-10 + GD-1/2/4/5 and ADDS VD-1..VD-5 (below). Grounded symbol map
(W-7): _grounding/H-VERIFICATION-GROUNDING-MAP.md (DG-1..DG-7 shipped substrate + PC-1..PC-8 prior-plan composition +
VD-1..VD-5 + SCHEMA + CITATIONS, live greps 2026-06-05).

TRACK 8 — REVERSIBILITY / TIME-TRAVEL (H-49). Authored 2026-06-05 from the undo/redo design (docs/thefinalglance/
UNDO-REDO-TIMETRAVEL-DESIGN-2026-06-05.md; prior art Cursor / Cline-Roo / Aider / undo-tree WebSearch-verified, the ELAI
substrate subagent-greped this session). The user-need: every LLM file edit reversible AND re-appliable (redo) at ANY node,
never losing a timeline, PLUS a lossless paged transcript so a long session's START is always reachable (the Claude-Code
"can only see the tail" complaint). This is the follow-on H-35 EXPLICITLY DEFERS (H-35:32 branch-and-fork keeping both
timelines + redo + cloud-model code-restore). THE TRACK-8 FINDING (grounded 2026-06-05 vs live source): the substrate is
~70% shipped but fragmented + wire-dark — the file-edit pre-image is ALREADY captured at file_ops.rs:272/300 then DISCARDED;
the byte-exact restore engine (edit_checkpoints.rs, test-proven) is gated OFF + path-heuristic + full-copy; the SessionDag is
a real parent_id DAG; SSB FileWrite is metadata-only with NO emitter; and there is NO redo anywhere. Track 8 is ONE session:
a content-addressed (blake3 + zstd, dedup) object store WRAPPING edit_checkpoints, an undo-tree HEAD over the SessionDag
(undo=back, redo=forward-edge, undo-then-edit=fork so nothing is lost), a divergence guard that snapshots-before-restore so a
user's manual edits are NEVER clobbered (the Cursor footgun / the Aider commit-user-work-first fix), the 3 user-selectable
grains (edit/turn/babystep), CAS-default + shadow-git-option, and the lossless paged transcript (compose H-6 recover_stubs +
E-29). Snapshot-restore (NOT replay) is load-bearing — model-agnostic, works on the production CLOUD lane where H-35's replay
is only VerifyDivergence. It COMPOSES H-35 / E-25 / H-6 / E-29 / C-23 / E-44 / G-2, restating none (GD-1). Inherits
W-1..W-10 + GD-1/2/4/5 + HD-1..HD-5 and ADDS RV-1..RV-4 (below).

TRACK 9 — CONTEXT LIFECYCLE (H-50..H-51). Authored 2026-06-05 from the context-management + handoff design
(docs/thefinalglance/CONTEXT-LIFECYCLE-REGIME-AWARE-AND-HANDOFF-DESIGN-2026-06-05.md; prior art Letta sleep-time /
Mem0 / A-MEM / Anthropic context-engineering / vLLM-APC-vs-transformers WebSearch-verified, ELAI substrate subagent-
greped this session). THE TRACK-9 FINDING (grounded 2026-06-05 vs live source): cloud and local need OPPOSITE context
policies and ELAI runs ONE cloud-shaped policy for both — so a long LOCAL session is effectively UNPROTECTED
(auto-compact is a FLAT 100K at conversation.rs:22, the 85% gate reads a 64K-non-opus window at cli_consts.rs:20
ignoring ELAI_LOCAL_MAX_CONTEXT, and recover_stubs re-inflates the trim to ~0 net at session_dag.rs:1551), it dies at a
hard context_window_blocked instead of unloading; and LOCAL prompt caching is a NO-OP because the sidecar's own
PrefixKVCache (server.py:1819) is bypassed on the streaming path the live client uses (runtime_client.rs:547). Cloud
caching, by contrast, is fully live (H-1 split-brain REFUTED). Track 9 fixes this: H-50 makes in-session management
REGIME-AWARE (ONE window source of truth + window-derived thresholds; LOCAL unload/load via the SHIPPED ladder/masking +
keep-stubs-unloaded + selective reload; CLOUD keep-the-cache-stable-prefix), H-51 makes cross-session continuation
token-cheap + semi-automatic (a sleep-time grounded topic-digest auto-authored from the journal, auto-DETECT + one-tap
confirm NOT blind continue, tiered load ~20-27K -> ~1-3K on local). It COMPOSES H-1/H-2/H-6/H-35/H-49 + the user's live
.claude/rules/handoff-protocol.md, restating none (GD-1). Inherits W-1..W-10 + GD-1/2/4/5 + HD-1..HD-5 and ADDS CL-1..CL-4 (below).

TRACK 10 — REVIEW-ANCHORING (H-52). Authored 2026-06-05 from an external-codebase STEAL exercise: three sources were
fetched, the two code repos cloned to docs/.source + mapped + explored (databow = an ADBC/Arrow data-engineering CLI,
NO ELAI fit; anthropics/defending-code-reference-harness = architectural patterns Track 7 ALREADY covers — execution-
grounded fitness, fresh-adversary re-attack, off-agent verification; alibaba/open-code-review = the ONE net-new steal).
The steal: open-code-review's DETERMINISTIC snippet->hunk comment-anchoring resolver — the model emits a VERBATIM code
snippet and NEVER a line number, deterministic code computes the hunk/line, 0,0 = can't-anchor -> file-fallback, never a
guessed inline line. This is the exact primitive G-5's reviewer cockpit asks for in its OWN Open-Question #2 (the hunk_id
join / wrong-granularity risk) and the same primitive E-41's cross-family PR reviewer needs to anchor comments without
trusting a model line number — NEITHER plan specifies it. H-52 is the off-agent-artifact doctrine pushed down into the
review-finding SCHEMA: a finding carries {snippet, vouch-handle} and is STRUCTURALLY UNABLE to assert WHERE it lands, so
it cannot fabricate it. It COMPOSES-WITH G-5 (the scoring consumer that closes its own OQ#2) + C-7 (the hunk_id space) +
E-41/C-40 (the DEFERRED PR-comment consumer) + F-4 (a vouch to anchor), restating none (GD-1). open-code-review's "smart
bundling" + "reflection module" are grep-verified VAPORWARE (excluded); its 26%-vs-15.5% F1 is NOT same-model-isolated
(directional only, HD-1). One session; inherits W-1..W-10 + GD-1/2/4/5 + HD-1..HD-5; ADDS no new delta family.

TRACK 11 — EXTERNAL-SYSTEM RESEARCH STEALS (H-53, H-54). Authored 2026-06-06 from a research exercise on two
external SYSTEMS (not codebases): Perplexity AI (oracle openness research, workflow/sessions/oracle/
2026-06-06-perplexity-ai-openness.md) and AlphaProof Nexus (DeepMind, arXiv:2605.22763, the proof corpus cloned to
docs/.source/alphaproof-nexus-results + mapped + explored). Two net-new steals, each a cant-lie hardening of an
already-strong ELAI surface, each grounded in ELAI's OWN substrate (the external system supplies the SHAPE, not the
guarantee). H-53 (citation pre-binding): Perplexity's documented answer stage embeds numbered ranked evidence
anchors INTO the prompt BEFORE generation and constrains synthesis to cite-as-you-generate ("citations predetermined,
not retrofitted") — ELAI's answer path is DETECT-AFTER today (E-48 GhostCite hardening + Phase 2B verify run AFTER the
model emits freeform [N]). H-53 moves citation assignment to a first-class PRE-GENERATION stage so a ghost/mis-attributed
citation is STRUCTURALLY impossible on the answer path; it COMPOSES E-48 (verify becomes a closed-set membership check),
H-52 (the same deterministic verbatim->anchor resolver), E-35/C-26 (the constrained-decode Tier-2), D-16 (refusal), 3E
(the slot). The Perplexity-SPECIFIC mechanism is analyst-reported (tier-2/3, MEDIUM, GD-5) — the steal rests on ELAI's
D-11 constructive-grounding + E-48, never a Perplexity number. H-54 (obligation-closure escape-hatch gate): AlphaProof
documents the EXACT gate-gaming ELAI's H-44 names as its placeholder weakness (relocate the unproven `sorry` into a
renamed helper lemma; a hallucinated false "known-result" lemma — both keyword-FREE at the top statement), and ships the
deterministic fix ELAI lacks: inspect the WHOLE dependency closure, a `sorry`/`admit`/`axiom`/`assume` ANYWHERE
disqualifies (the corpus confirms it — 128 sorries confined to 2 LABELLED files, 69 fully closed). H-54 ports that as the
cheapest DETERMINISTIC FIRST leg of trap detection on the E-53 formal-verification lane and the deterministic core H-44's
keyword grep was a stand-in for; it is the wiring R7 insight applied to a proof DAG (admitted != proved, as test-root !=
production-reachable). AlphaProof is also external GROUNDING for H-43 (compiler-as-fitness, not bytes) + TD-3 (the LLM
critic is a search heuristic, the compiler is the gate) + E-53's HARD-AVOID of routine Lean (~2.5-9% pass, $/problem).
Both inherit W-1..W-10 + GD-1/2/4/5 + HD-1..HD-5 (and H-54 the Track-7 VD-1..VD-5); ADD no new delta family. Track 5
(odysseus) reservation bumps to start at H-55. Reasoning digests: workflow/reasoning/
PERPLEXITY-CITATION-SYNTHESIS-STEAL-2026-06-06.md + ALPHAPROOF-DEPENDENCY-CLOSURE-STEAL-2026-06-06.md.

THE CENTRAL FINDING (grounded 2026-06-03, spot-checked vs source): most of Track 1 is wire-dark
ACTIVATION (the main coding-agent loop ALREADY prompt-caches; only the RuntimeAdapter split-brain + the
hit-rate monitor are dark — see DESIGN-DOC CORRECTIONS C1), and the canvas-store isolation gate already
PASSES (no Svelte runes store imports three.js), so the orbit can be retired cleanly. H-plan's single
job: light up the dark efficiency substrate and stand up the canvas, under the SAME hard, EXECUTED wiring
done-gate the F/G plans use, so none of it ships wire-dark again.

## Status legend
- PLANNED — file written, work not started (all 16 H-N at generation)
- IN PROGRESS — active session
- DONE — passes the FULL done-gate (wiring_contract_check.py --run-trigger PASS; cargo-green is
  necessary but NOT sufficient)
- DEFERRED — RunPod / external dependency gated (named per H-N)

================================================================================
THE WIRING DOCTRINE — INHERITED W-1..W-10 (canon: F-0_INDEX.md) + GD-1/GD-2/GD-4/GD-5 (G-0) + H-DELTAS
================================================================================

Every H-N INHERITS the F-plan doctrine W-1..W-10 VERBATIM (do not re-litigate; F-0_INDEX.md is canon):
  W-1  no-wire-dark done-gate: DONE only when wiring_contract_check.py --bn-file <H-N>.md --run-trigger
       returns PASS (produces exist as DEF, consumes resolve to CALL/REF, reachable_from reached by BFS,
       stub_ban clean, AND observable_effect actually RAN + emitted the off-agent artifact). cargo-green != DONE.
  W-2  no default-off env gate as the terminal state (kill-switch DISABLE only; default ON; observable_effect
       runs with NO env override). [Track-1 caveat: a flag that flips dark->live, e.g. ELAI_FOCUS_TUNNEL_EXECUTE
       or ELAI_E25_COMPACTION_LADDER_ENABLE, must be promoted to default-ON or registered in
       wire-dark-allowlist.toml with a promotion plan; default-off is not a terminal state — W-2 == R9.]
  W-3  reachable from a REAL production entrypoint (CLI subcommand / FSM transition / Tauri command / IPC
       dispatch / hook); a #[cfg(test)]-only caller is wire-dark.
  W-4  frontend_pair mandatory for any dashboard-surfaceable signal; cross_lang_diff.py PASS.
  W-5  observable_effect is a LIVE end-to-end trigger, not a unit test; RunPod-gated work ships a
       deterministic fixture harness exercising the REAL path + a separately-gated live checkbox.
  W-6  >=1 ACCEPTANCE item verified THROUGH the dashboard (dashboard-smoker), or N/A with justification.
  W-7  no hallucinated integration points: every consumes_symbol / reachable_from anchor / MODIFY target
       grep-confirmed in elai.map + H-GROUNDING-MAP.md AT AUTHORING TIME; the validator re-greps.
  W-8  produces/consumes symmetry: every produced symbol has an in-plan caller.
  W-9  new SSB action_types flow the action_type golden lock (9-tuple lockstep; Pattern-2 single-variant-
       with-Kind where it applies); SCHEMA bump named from the LIVE baseline (207/484), never hard-coded.
  W-10 tiered-gate: any axis needing network/subprocess/RunPod at gate time is SPLIT — the in-process Tier-1
       is the fail-closed done-gate; the network/subprocess Tier-2 is separately gated and MUST NOT advertise
       fail-closed if it is fail-open.

INHERITED GD rules (canon: G-0_INDEX.md) that apply to H verbatim:
  GD-1  COMPOSE-NOT-RESTATE. Every H-N either fills a new efficiency/canvas gap OR COMPOSES-WITH an
        existing F/G/B/C/D/E artifact by CONSUMING its symbol/verdict. Re-implementing or re-stating a
        prior-plan artifact (apply_anthropic_cache_markers, try_edit_tiers, RouteDecisionEvent, CloudSpend,
        media_validator Tier2, G-1 taint_flow, G-2 RRA, the 9-tuple SSB lockstep) is a P0 incident logged
        to gotchas.md. Each H-N NAMES the prior artifact it composes with.
  GD-2  REPRODUCIBLE-METRIC. Any number an H-N emits (cache hit-rate, $/resolved-task, token ratio, DSR,
        canvas-spawn rate, over-block rate) MUST be recomputable from the SSB event stream under a pinned
        ELAI_SEED. A number that cannot be re-derived from the journal is wire-dark by construction.
  GD-4  FP-FRICTION RELEASE GATE. No H-N gate flips default-ON until its false-positive / over-block / over-
        escalation rate is MEASURED and bounded on >=3 real runs (routing over-escalation H-7; dead-code
        false orphans H-10; taint over-block H-13/H-15; design-mode writeback corruption H-14).
  GD-5  NO-FABRICATED-GUARANTEE. A fail-open / RunPod-pending / cloud-opaque path NEVER advertises a
        guarantee it does not deliver (cloud cache = measured-hit-rate-not-promise; aborted-stream billing =
        UNVERIFIED until confirmed; cloud reproducibility = action-stream-not-tokens).

H-DELTA rules (additional, specific to the efficiency/UI nature of this plan — all BLOCKING):
  HD-1  MEASURED-NOT-MULTIPLIED. Every efficiency number is reported at the MODEL-HARNESS-CONFIG level
        (Harness-Bench arXiv:2605.27922 — capability/cost vary per pairing) and is the MEASURED value on
        ELAI's own runs (hit-rate, $/resolved-task delta), NEVER a literature figure quoted as ELAI's and
        NEVER the PRODUCT of several levers multiplied into a headline (the exact sin ELAI-EFFICIENCY §11
        forbids: input-side levers overlap, output is a separate 5x stack; the combined cut is bounded by
        the larger lever on a turn, not the product). Tag every borrowed number [WEAK]/[DESIGN] per its
        ledger entry. The honest claim is one reproducible $/resolved-task tuple, not a bigger number.
  HD-2  CACHE-PRESERVING. No H-N may bust the byte-stable cache prefix. Volatile content (tool results,
        SSB rows, RAG chunks) lives strictly AFTER the last breakpoint; a change that lands content before
        a breakpoint is a cache-bust = a TEST FAILURE (debug_assert the cached byte-prefix is identical
        turn-to-turn — ELAI's allocator + ELAI_SEED can GUARANTEE this, a heuristic agent cannot). Masking/
        compaction (H-6) DROPS TRAILING observations, never REWRITES the prefix (LLM re-summarization busts
        the cache; deterministic masking preserves it — ELAI-EFFICIENCY §10.6). The 20-block lookback means
        long tool loops need a SECOND tail breakpoint.
  HD-3  OUTPUT-IS-5X / DECODING-IS-SACRED. Output bills 5x input and caching never touches output, so an
        output-token win (diff/FIM, H-3) counts 5x and must NOT be conflated with a cache win (a token
        saved on output is worth 5x one saved on cached input). Corollary: do NOT force-JSON / constrained-
        decode the coder/planner/reasoning roles (arXiv:2408.02442 — constrained decoding degrades
        reasoning). Constrained/grammar-guided decoding is admissible ONLY on pure op-emission / pure-data
        roles (canvas ops H-13, the 5-class diagnoser), per-mode grammars with a CI compile-time ceiling
        and a non-silent grammar_unavailable fallback (CANVAS §7).
  HD-4  OFF-AGENT-CANVAS-GATE. Every model-driving / canvas done-gate (H-13/H-14/H-15) is AUDITOR-driven
        against a task authored BEFORE the slice with the gate fileset frozen (wiring R4/R8; CANVAS §13/§20
        in-boundary risk). The load-bearing proof is the off-agent SSB row + the rendered node observed by
        the auditor, NEVER the implementing agent's stdout and NEVER its own unit test. A gate the
        implementing agent can satisfy from inside its own write-boundary is VOID; the gate FAILS if the
        dispatcher is wired to nothing in production even when a unit path passes.
  HD-5  SUBSTRATE-HONESTY. A rendered picture is not proof the thing it depicts is correct (CANVAS §4/§19).
        The DKG static call graph cannot see dynamic dispatch (Tauri invoke() strings, dyn Trait, the SSB
        bus, the CanvasDispatcher itself); the entrypoints map re-attaches dynamic-dispatch-only nodes
        before orphan detection, and the runtime coverage_count is the AUTHORITATIVE backstop. Topology-
        only-suspected-dead renders VISUALLY DISTINCT from coverage-confirmed-dead. A model-drawn diagram
        is valid-not-verified until the media_validator VLM (or an executed artifact) checks its CONTENT,
        not its syntax. Never launder a static guess or a model drawing as ground truth.

TD-DELTA rules (TRACK 3 TRUST-HARDENING only, H-17..H-29; additional, all BLOCKING; canon: H-TRUST-GROUNDING-MAP.md):
  TD-1  PERSISTED-N-KEYSTONE. Every significance/edge number deflates by a persisted, append-only, agent-
        unresettable cumulative trial count N (per (data_window, objective)). A reported Sharpe/edge whose config
        has ZERO matching trial rows is a HARD FAIL (trial_count_missing), never SKIP. (design D2; without N an
        HONEST searcher still ships overfit edge — H-17 is the keystone H-18/H-22/H-25/H-29 all read.)
  TD-2  FROZEN-EVALUATOR. The backtest harness, metric code, gates, thresholds AND the H-17 trial ledger ALL live
        in the gate-manifest LOCKED(mutable=0) set the self-mod agent cannot read or mutate (reuse R8
        gate_freeze_check.py + the rules-engine mutable=0 tier + AllowedConfigKey + LockedKeysAuditor). (design D3,
        Everitt current-RF.) Highest-leverage single safety move.
  TD-3  LLM-IS-T3-ADVISORY-ONLY. LLM/auditor judgment is a SOFT T3 backstop that advises a human; it is NEVER the
        load-bearing promotion gate (design D4; BadScientist accepts fabricated results 82%). A deterministic 3a
        BLOCK can never be overridden by a probabilistic 3b green.
  TD-4  VERIFY-THE-VERIFIERS. Each PROPOSED->ACTIVE gate revision must (a) kill 100% of a FROZEN out-of-band mutant
        corpus targeting the gate's OWN logic, (b) satisfy its declared metamorphic relations INCLUDING a mandatory
        null_band_sidedness MR, (c) reject every frozen trap; any escape blocks promotion + increments a persisted
        escaped-mutant counter (design D5; the REGRET symmetric-vs-one-sided-band incident — H-27 enforces this).
  TD-5  HONEST-RESIDUE. Name the irreducible undecidable residue (design PART 5) and route it to human Layer-3 / T3
        advisory; never advertise a guarantee the mechanism cannot deliver. Cross-hardware FP bitwise (physics),
        P(Y|X) concept drift before labels (undecidable), target-derived-feature leakage (judgment), genuine-vs-lucky
        search artifact (necessary not sufficient), self-mod SAFETY (Rice/Schmidhuber) all stay human-gated.
        Extends GD-5. The live autonomous-trading lane is W-10-tiered (fixture harness now; live run a gated checkbox).

RD-DELTA rules (TRACK 6 GROUNDED-REASONING HARDENING only, H-39..H-42; additional, all BLOCKING; canon: H-REASONING-GROUNDING-MAP.md):
  RD-1  ABSTENTION-CREDITED REWARD. Any reward an autonomous loop OPTIMIZES (promotion two_gate_promote, BAI, GEPA, the
        bench scorers) MUST credit a correct abstention and penalize a confident-wrong strictly MORE than an abstention
        (explicit-confidence-target binary grading, penalty=t/(1-t); OpenAI 2509.04664 Observation 1). A reward that scores
        confident-wrong == abstain is a hallucination-BREEDING signal = FAIL. The abstain bit is READ from the shipped
        conformal RefusalDecision (D-16), never re-derived (GD-1). [H-39]
  RD-2  DECORRELATION-OVER-ACCURACY. A verifier/role is retained on its conditional error-DECORRELATION (unique catch given
        ground truth), NOT standalone accuracy. Pruning a decorrelated low-accuracy verifier on a marginal-lift flag is a
        Weaver-class error (2506.18203). The honest ceiling is the ORACLE verifier (Pass@K) — an ensemble approaches, never
        exceeds it; never advertise "catches everything" (extends GD-5). [H-40]
  RD-3  SOUND-CHECKER-FIRST + COVERAGE-IS-THE-LEVER. A claim with an available sound constructive checker is NEVER graded by
        an LLM verifier instead (already enforced, grounding_mode.rs:157). The LIVE Mode-A coverage ratio + the checker-gap
        backlog are the measured investment signal (Kambhampati 2402.08115); NP-complete/undecidable claim classes (Xu
        2401.11817 Table 3) route to a sound solver or refuse, never an LLM grade. [H-41]
  RD-4  DEPTH-NOT-WIDTH. Reasoning budget scales with a sub-task's estimated SERIAL depth, never with context-window size.
        A context-window increase is NEVER accepted as the fix for a depth-class reasoning failure: a single forward pass
        is ~TC0; the autoregressive CoT loop is the serial-compute escape with lower-bounded step counts (2503.01544 /
        2502.02393 / 2310.07923). Access != compute-depth. [H-42]

VD-DELTA rules (TRACK 7 DGM-VERIFICATION HARDENING only, H-43..H-48; additional, all BLOCKING; canon: H-VERIFICATION-GROUNDING-MAP.md):
  VD-1  FITNESS-IS-EXECUTION-GROUNDED-NOT-BYTES. Any signal the self-improvement loop OPTIMIZES (derived_alpha, the
        promotion reward, the bandit arm) MUST be a function of the candidate's MEASURED EXECUTION behavior (pass-rate
        over generated tests / dual-agreement mass), NEVER its bytes (hash) or its own confidence. A config producing
        FAILING code MUST score LOW. A byte-hash or self-reported fitness defaulted-ON is a FAIL (the divergent branch;
        Self-Verification-Prevents-Collapse + Skalse 2209.13085 unhackable-proxy). [H-43]
  VD-2  DETECTOR-IS-BEHAVIORAL-WITH-MEASURED-RECALL. Cheat/trap detection MUST be behavioral (execution, spec-test
        conflict, isomorphic perturbation), NOT keyword/substring grep; AND its recall MUST be MEASURED on a frozen
        out-of-band contrastive corpus (TRACE 2601.20103) and reported (GD-2). A keyword-free hack that passes is a FAIL;
        an unmeasured detector advertises a guarantee it cannot prove (extends GD-5). [H-44]
  VD-3  SCORE-WORST-CASE-NOT-MEAN + LINEAGE-DIVERSE. Promotion scores on the WORST-CASE / variance-penalized ensemble
        verdict (Coste 2310.02743 WCO/UWO), judges diversified by PRETRAINING lineage not seeds (Eisenstein 2312.09244);
        cross-view disagreement BLOCKS (Co-rewarding 2508.00410 40-pt self-judge collapse). Promotion on the candidate's
        own mean alpha is a FAIL. tau is CALIBRATED on PRMBench (2501.03124), never a carried-forward UNCONFIRMED
        constant at default-ON. [H-45]
  VD-4  STRONGER-THAN-GENERATOR VERIFIER. The loop's gain is bounded by the verifier-minus-generator capability gap
        (Solver-Verifier Gap 2507.00075); the verifier MUST be execution- or proof-grounded and the generator + cross-
        family verifier MUST run on DIFFERENT pretraining families (Co-rewarding collapse; the abliterated 7B may not
        verify its own family). A verifier weaker-than-or-equal-to the generator is not a valid gate. [cross-cutting: H-43/45/46/48]
  VD-5  HONEST-UNDECIDABLE-RESIDUE. Name the irreducible residue and route it to human/proof Layer-3; never claim
        immunity. Proof gates rarely pass (VERINA 4.9% / CLEVER 0.6% — narrow CSQ-Critical slice only, E-53). A strong
        self-improver can SANDBAG its eval (eval-awareness 2505.23836 AUC 0.83 / 2510.20487 >80%) — mitigate via
        indistinguishability (H-47), do not cure. Keep the whole self-improvement loop DARK/allowlisted (W-2 kill-switch) —
        SAFER, not a sales headline. Extends GD-5 / TD-5. [all Track-7 H-N]

RV-DELTA rules (TRACK 8 REVERSIBILITY / TIME-TRAVEL only, H-49; additional, all BLOCKING; canon: UNDO-REDO-TIMETRAVEL-DESIGN-2026-06-05.md):
  RV-1  NEVER-CLOBBER-THE-HUMAN. Before any working-tree materialize (undo/redo/jump), if a target path's on-disk blake3
        != the HEAD-expected hash, the engine MUST FIRST snapshot the current bytes as a fork (tag auto:pre-restore) and
        only then restore. A restore that overwrites un-snapshotted human edits is a HARD FAIL (the Cursor AI-edits-only
        footgun; the Aider commit-user-work-first fix). The false-fork rate is GD-4-bounded on >=3 runs before default-ON.
  RV-2  LOSSLESS-STORE / LOSSY-VIEW. The CAS object store + the transcript are append-only; trimming, stubbing and
        compaction are VIEW projections for the model window, NEVER the system of record. A handoff that drops or stubs a
        message without it being recoverable via recover_stubs is a FAIL. GC deletes blobs/nodes ONLY by reachability and
        NEVER below the session start. ("see the start of a big session" is a storage invariant first, a paged viewer second.)
  RV-3  REDO-IS-A-FORWARD-EDGE-NOT-A-STACK. Undo/redo are HEAD moves over the append-only undo-tree; undo-then-new-edit
        FORKS (the prior child survives as a sibling). A redo implemented as a destructible stack that a new edit silently
        clears is a FAIL (the classic lost-redo bug). "At any node" means any tree node is addressable, not just the tail.
  RV-4  HONEST-IRREVERSIBILITY. File edits undo soundly (both cloud and local — snapshot-restore is model-agnostic).
        External already-emitted side effects (network / delete / spend / push / deploy / sent-message) are NOT undone;
        they are SURFACED from the E-44 saga compensation log as a flagged warning, never auto-reverted and never
        advertised as undoable. Extends GD-5 / TD-5.

CL-DELTA rules (TRACK 9 CONTEXT LIFECYCLE only, H-50..H-51; additional, all BLOCKING; canon: CONTEXT-LIFECYCLE-REGIME-AWARE-AND-HANDOFF-DESIGN-2026-06-05.md):
  CL-1  WINDOW-TRUTH. ONE model-window source of truth per backend (local = ELAI_LOCAL_MAX_CONTEXT / the
        BudgetPolicy::Local ceiling; cloud = the cloud ceiling), and every trim/handoff threshold is a FRACTION of that
        real window. A flat constant (the 100K auto-compact), a wrong default (64K-non-opus), or two windows that
        disagree is a HARD FAIL — it is why local is unprotected today.
  CL-2  REGIME-OPPOSITE, BACKEND-SELECTED. Cloud keeps a big cache-stable prefix and compacts late (amortize the 1.25x
        write over many 0.1x reads); local minimizes the prefix and unloads early (no cache, no room). The policy is
        chosen by the live BackendMode, never hardcoded. Applying the cloud policy to local (today's bug) or vice versa
        is a FAIL.
  CL-3  UNLOAD-STAYS-UNLOADED (local). On local, offloaded content stays offloaded; reload is SELECTIVE / on-demand,
        never an unconditional recover_stubs re-inflate that nets the trim to ~0 (the current bug). The masked tokens
        stay re-fetchable from SSB (lossless view; aligns RV-2 / H-6).
  CL-4  NO-FABRICATED-CACHE. Never advertise a cache hit a path cannot deliver. The cache-read fraction is MEASURED from
        the journal PER REGIME (cloud > 0; local = 0 until the streaming PrefixKVCache bypass is fixed). A "caching on"
        claim on a path whose journal shows cache_read=0 is a FAIL. Extends GD-2 / GD-5.

DONE DEFINITION (FSM may not reach COMPLETE without all): (1) cargo check --workspace GREEN, (2) auditor
  PASS (sole test runner), (3) wiring_contract_check.py --run-trigger PASS, (4) cross_lang_diff PASS if a
  frontend_pair is present, (5) the W-6 dashboard E2E item (or justified N/A), (6) the GD-4 FP/over-block
  bound recorded where the H-N defaults a gate ON, (7) memory-writer ran.

================================================================================
THE 54 SESSIONS (Tracks 1-4 + Track 6 + Track 7 + Track 8 + Track 9 + Track 10 + Track 11 authored; Track 5 = odysseus, reserved/unauthored)
================================================================================

TRACK 1 — EFFICIENCY (activation of wire-dark substrate; F-E1..F-E9 reformulated as wiring-enforced H-N)
  H-N   title                                            lever / nature           composes-with        effort wave
  H-1   prompt caching on the production path            CHEAPER / ACTIVATION     B-27/C-5, E-39       M      1
        (close the RuntimeAdapter cache split-brain + activate the dark hit-rate monitor; the main loop
         already caches via apply_anthropic_cache_markers — see CORRECTION C1)
  H-2   cache-stable layout + breakpoints + 1h TTL       CHEAPER / BUILD          B-10/Phase-3E        M      1
        (feed real slot bytes to update_breakpoints, consume bp.valid to PLACE markers on the cloud path,
         tail breakpoint, byte-prefix debug_assert — the existing mechanism is inert + Local-only)
  H-3   output-token stack (diff/FIM/streaming)          CHEAPER / BUILD+ACTIVATE B-28, B-20           L      1
        (relax ModeSelector Rule 1 + thread a BufferEditor so DIFF emits on cloud; one-line streaming-
         validator activation at runtime_builder.rs:306; StreamingEarlyTerminated SSB producer)
  H-4   Tool Search Tool + advanced-tool-use beta        CHEAPER / BUILD          B-37/E-18, E-40      M      1
        (send advanced-tool-use-2025-11-20 on the OAuth branch too; tool catalog behind the search tool)
  H-5   Batch API for non-interactive lanes              CHEAPER / BUILD          B-42, B-7, E-8, E-9  M      1
        (a batch client; route Cosmos refresh / research / GEPA / bench at 50% + 1h TTL)
  H-6   deterministic observation masking + mem read-back CHEAPER / ACTIVATION    E-25, F-5            M      1
        (activate adaptive_reduce as default long-session manager; un-gate the E-25 ladder; wire
         recover_stubs + get_relevant_facts read-backs — prod uses a hardcoded placeholder today)
  H-7   calibrated routing (architect/editor/verifier)   SMARTER-ORCH / BUILD     B-37D, E-31          L      1
        (wire PerRoleRuntime/RuntimeOrchestrator into dispatch on top of the live cap-enforcement leg;
         replace the hardcoded layer_failed:1; emit RouteDecisionEvent — a calibrated router, not 7B self-conf)
  H-8   persist OracleAnswerCache (+ idempotent grounding) CHEAPER / BUILD         C-30                S      1
        (disk-back the in-process oracle cache so /research|/ask answers survive restart; PromptCache
         already persists — do NOT re-implement it)
  H-9   cost cockpit dashboard                           PROVABLE / ACTIVATE+BUILD C-5, C-25, E-49     M      1
        (attach .with_ssb_writer to the cloud client so CloudSpend rows land; CloudPanel stops rendering
         zeros; per-response input/output/cached breakdown + cache-expiry timer + $/resolved-task live)

TRACK 2 — CANVAS-WORKBENCH (the new product surface; replaces the orbit; CANVAS-WORKBENCH-DESIGN C1..C6 + §15-21)
  H-N   title                                            lever / nature           composes-with        effort wave
  H-10  canvas shell + CODE-GRAPH (read-only)            STEERING / BUILD         B-29, B-33, B-34     L      2
        (B0 worst-case DKG measurement -> renderer pick; render the symbol call-graph already on canvas;
         materialise the producer-dark PageRank table; dead-code island + entrypoints map + coverage backstop)
  H-11  live SSB layer + file explorer + scoped fs        STEERING / BUILD        C-4, B-29            M      2
        (CanvasActivityEvent overlay + ghost reconciliation; FileExplorer + the BUILD of the fs surface —
         tauri-plugin-fs + scoped capability; activate the orphaned list_projects/create_project)
  H-12  PLAN + RESEARCH modes + shared chrome             STEERING / BUILD(port)  plan/research trees  M      2
        (port OrbitMode1/3 to flat frames; mode bar; single-consumer SSB handover at the default switch)
  H-13  model drives the canvas (ops + dispatcher + SSB)  GENERATIVE / BUILD      H-4 pattern, F-3p1   L      2
        (canvas_* tools via the E-40 3-arm pattern; CanvasDispatcher mode-gating + sanitizeAction;
         ONE Pattern-2 CanvasOpApplied; ARCHITECTURE mode; media_validator Tier2 reuse)
  H-14  APP/WEBSITE DESIGN mode (web-first, RN deferred)  GENERATIVE / BUILD(net) recast, @onlook/parser XL     2
        (dev-preview proxy; data-oid opaque-id tagging mirroring @onlook/parser ids.ts; two-tier recast
         writeback — riskiest mode, gated on a recast prove-it spike; see CORRECTIONS C-design)
  H-15  spawn_view reply channel + taint-gated deixis     GENERATIVE / BUILD      G-1, G-2, H-13       L      2
        (the spawn_view verb + comparison-board view; inbound CanvasUiAction::Select gated by G-1 taint;
         snapshot/delta reconciliation per tab; ONE Pattern-2 CanvasViewEvent — the trust-spine moat)
  H-16  retire the orbit                                  CLEANUP / DELETE         (scene/* + orbit)    S      2
        (delete scene/three-* + StarCanvas + OrbitCanvas + the three/troika deps AFTER a full release of
         Canvas-as-default; canvas-store isolation gate already PASSES — clean removal)

TRACK 3 — TRUST-HARDENING (the autonomous-trading-trust gates; T-1..T-13 from the trust-hardening design; BUILD-ON-SHIPPED-substrate)
  H-N   title (T-N)                                       cluster / threat          composes-with         effort wave
  H-17  persisted untamperable trial ledger (T-1)         A SELECTION-BIAS / TB-1   SSB WAL, gate-freeze   S-M    3  [KEYSTONE]
        (append-only strategy_trials + register_trial choke-point + trial_count read API; N agent-unresettable + frozen)
  H-18  Deflated-Sharpe / PBO / MinBTL gate (T-2)         A SELECTION-BIAS / TB-1   H-17, rules COMPOSITE  M      3  [#1 net-new gate]
        (LOCKED COMPOSITE: block promotion unless DSR+PBO+MinBTL clear, N from H-17; numpy/scipy sidecar Tier-2; fail-closed if N absent)
  H-19  reusable-holdout query budget (T-3)               A SELECTION-BIAS / TB-1   H-17, MetricGate       M      3
        (DP fresh-holdout budget on the OOS set; after budget spent -> refuse OOS queries; deployment-trust half)
  H-20  leakage static-analysis COMPOSITE (T-4)           B LEAKAGE / TB-2          taint primitive, G-1   M-L    3
        (no fit-site reaches both train+test sinks; .shift AST; SCHEMA split-boundary; G-1 live-taint prereq flagged; runtime backstop = H-21)
  H-21  runtime look-ahead / causality differential (T-5) B LEAKAGE / TB-2          DriftComparator        M      3  [un-gameable backstop]
        (run the pipeline on full vs future-truncated data; a column that changes uses the future; frozen runner Tier-2)
  H-22  trading metamorphic null COMPOSITE (T-6)          C PERTURBATION / TB-1+2   metamorphic_testing.rs L      3
        (future-shuffle/time-reversal/zero-drift-null named relations + permutation null-band + DSR-vs-null; the edge must collapse to noise)
  H-23  same-seed re-run hash-equality gate (T-7)         D REPRODUCIBILITY         RRA, determinism.rs    M      3
        (re-invoke the real entrypoint under recorded env+seed; assert DECLARED-METRIC-OUTPUT blake3 equality; seed-coverage AST + env-pin)
  H-24  sim-to-real parity gate (T-8)                     D PARITY                  replay_resume, RRA     M      3
        (A: ban `if live:`/`if backtest:` + single-entrypoint SCHEMA; C: replay==golden hash; B behavioral parity DEFERRED Tier-2)
  H-25  result manifest + re-derivation verifier (T-9)    E ANTI-FABRICATION        F-4 stdout_hash, RRA   M      3
        (bind each reported NUMBER to provenance + RE-PARSE it; harness frozen + monkeypatch AST ban; extends F-4 test-PASS to result-honest)
  H-26  shadow-exec + financial-invariant MR library (T-10) E DYNAMIC-VERIFICATION  metamorphic+mutation   M-L    3
        (two-impl ULP divergence + put-call-parity/strike-monotonicity MR pack + naked-float-== ban; Daikon-mined = PROPOSED-must-kill-a-mutant)
  H-27  gate self-validation (T-11)                       F META-LAYER / TB-3       mutation+metamorphic+trap L   3  [verify-the-verifiers]
        (each gate must kill its own frozen mutant corpus + satisfy null_band_sidedness MR + reject traps before ACTIVE; the GBM-band fix)
  H-28  reward-tampering gate + harness-frozen (T-12)     F META-LAYER / TB-3       blake3 tripwire, two_gate S    3  [self-mod safety]
        (freeze evaluator+harness+thresholds+H-17 ledger; HARD-BLOCK on hash mismatch + FLAG implausible score delta with hash intact)
  H-29  LOCKED fail-closed drift state machine (T-13)     G DEPLOYMENT-TRUST        model_drift, canary, H-18 M    3
        (4-axis PSI in/out + schema-fingerprint + 14d deviation + 90d TTL warning->drift->halt on the resume FSM edge; missing evidence = HALT)

TRACK 4 — UX + USER-MEMORY (instrument-first; activate wire-dark crates/memory + host-side free-win QOL; grounded 2026-06-04)
  H-N   title                                            lever / nature              composes-with         effort wave
  H-30  staleness / intent-drift INSTRUMENT              PROVABLE / BUILD            seccodeplt_runner     M      4  [KEYSTONE; gates H-31/H-32]
        (stale_fact_reached_prompt ratio + BeliefShift-style conflict-resolution eval; --broken-retrieval arm MUST scream)
  H-31  user-decision bi-temporal ledger (Tier-B)        STEERING/TRUST / ACTIVATION crates/memory, H-37   M-L    4  depends H-30
        (activate soft_close/get_relevant_facts/dispatch into /remember capture + retrieval-into-prompt; auditor firewall)
  H-32  code-grounded fact memory (Tier-A)               TRUST / BUILD-on-DKG        H-31, DKG             M      4  depends H-31
        (validate-citation-before-use; validity-axis != recency; 28d unused TTL; fail-closed on Unknown citation)
  H-33  memory hub + governance + provenance             STEERING/TRUST / ACTIVATION H-31, F-12 drawer     M      4  depends H-31
        (de-stub the parsed `elai memory` verbs; list/forget/pin; which-prefs-fired + /continue resume-summary)
  H-34  MCP defer-loading + Tool-Search emulation        CHEAPER / BUILD             C-35 lazy cache       M      4  (distinct from H-4)
        (MEASURE per-turn MCP-schema tokens FIRST, then defer-load the live STDIO lane; odysseus tool_index reimpl)
  H-35  /rewind over the SSB journal                     STEERING / BUILD-UI         H-33, G-2 RRA         M      4  depends H-33
        (conversation/code/both restore over the seeded journal; no second shadow-git; re-verify_rra post-rewind)
  H-36  completion / needs-input notifications           STEERING / BUILD            H-35                  S      4  depends H-35
        (OSC9 + BEL + OS-notify FROM an SSB row, on completion AND needs-input; dev-event only, NOT engagement)
  H-37  plan-approval gate at PLANNING->TESTING_FIRST    STEERING / BUILD-on-pregate H-36, B-22 gate       S-M    4  depends H-36 (soft)
        (activate run_interactive_gate on the compliance pre-gate edge; approve/redirect/reject -> plan_decision_event)
  H-38  cost-confirm on TaskStop / pod_stop              SAFETY/STEERING / BUILD     H-9, H-37, orch T1    S      4  depends H-9
        (surface cost-so-far + unsaved before a billable/unsaved stop; $0 saved stop passes frictionless)

TRACK 6 — GROUNDED-REASONING HARDENING (refinements + instruments on the SHIPPED reasoning/verification substrate; hallucination-limits-derived)
  H-N   title                                            lever / nature              composes-with            effort wave
  H-39  abstention-aware promotion + eval scoring        PROVABLE/TRUST / BUILD       D-16/D-17, D-5, E-49/G-6  M     6  [KEYSTONE]
        (AbstentionAwareScore correct=+1/abstain=0/confident-wrong=-t/(1-t); feed two_gate_promote + coding_metrics +
         the bench scorers; read the abstain bit from conformal RefusalDecision — the reward the self-mod loop climbs stops rewarding guessing)
  H-40  decorrelation-keyed verifier/role retention      TRUST / EXTENSION            E-6 iso_budget_harness    S-M   6
        (add a unique-catch/decorrelation axis to IsoBudgetReport; prune only on low-lift AND low-unique-catch — Weaver:
         a decorrelated below-accuracy verifier is RETAINED; reuse the shipped iso_budget_measured row, no new variant)
  H-41  live checker-coverage + checker-gap INSTRUMENT    PROVABLE / BUILD             D-10/D-11/D-21, D-13      M     6
        (the A>B preference is ALREADY shipped at grounding_mode.rs:157 — INSTRUMENT it: a live Mode-A coverage ratio +
         a +CheckerGapDetected backlog classified by the Xu Table-3 complexity taxon; ranks where a sound checker buys the most coverage)
  H-42  serial-depth reasoning budget + depth-not-width   SMARTER-ORCH/TRUST / EXTENSION Phase-2C GRC, E-19      M     6
        (estimate_serial_depth -> scale_grc_budget bounded by a LOCKED ceiling; classify DepthBound vs AccessBound so a
         context lift is never the fix for a depth failure — formally grounds 'smarter orchestration not a bigger model', H-7)

TRACK 7 — DGM SELF-IMPROVEMENT VERIFICATION HARDENING (replace the two placeholder filters + compositional upgrades; the convergence precondition)
  H-N   title                                            lever / nature              composes-with             effort wave
  H-43  execution-grounded DGM fitness                   TRUST/PROVABLE / BUILD       D-2, E-19, E-53, safedelta M-L   7  [KEYSTONE; GAP-1 P0]
        (replace derived_alpha=blake3(config_bytes) with CodeRM pass-rate + CodeT dual-agreement; a failing-code config
         scores LOW; the off-agent fitness E-19 Task 11 + H-39's 'correct' bit wait on; FitnessEvaluated row)
  H-44  behavioral trap detection + measured recall      TRUST / BUILD                D-24, D-13, E-54          M-L   7  [GAP-2 P0]
        (replace score_candidate_for_traps keyword-grep with ImpossibleBench conflict + Isomorphic-Perturbation +
         CodeHalu execution; MEASURE recall on TRACE; cross-view gate; TrapBehaviorDetected row)
  H-45  worst-case + lineage-diverse promotion scoring   TRUST / BUILD(extend)        D-5, D-13, promotion_gate M     7  [GAP-3; depends H-43]
        (score worst-case/variance over lineage-diverse views, not the candidate's mean; disagreement BLOCKS;
         PRMBench-calibrated LOCKED tau replacing the UNCONFIRMED constant; PromotionScoreAggregated row)
  H-46  HGM clade-metaproductivity parent selection      SMARTER-ORCH / BUILD         config_archive, D-5       M     7  [depends H-43]
        (select_parent keyed on the descendant clade's aggregated measured deltas, not the immediate sigmoid score;
         the Metaproductivity-Performance Mismatch fix; no-lineage fallback; CladeSelected row)
  H-47  DGM eval firewall (date-gate + template-regen)   PROVABLE / BUILD(compose)    D-6, E-54, H-19           M     7  [GAP-5; depends H-43]
        (date-gate held-out to the generation clock + GSM-Symbolic template-regen at score-time + bind H-19 Thresholdout
         budget; COMPOSES D-6/E-54/H-19, restates none; indistinguishability probe; EvalFirewallEvent row)
  H-48  wire the BAI bandit on the real reward           SMARTER-ORCH / ACTIVATION    D-5, D-29, H-43, H-45     M     7  [GAP-4 P2; LAST]
        (activate the scaffold Discounted-TS + Successive-Rejects on H-45's aggregated execution reward; ExIt curriculum;
         COMPOSES D-5/D-29, reuses their telemetry where possible; +BanditArmSampled only if no emitter)

TRACK 8 — REVERSIBILITY / TIME-TRAVEL (reversible LLM edits + lossless history; the follow-on H-35 defers; BUILD-on-shipped)
  H-N   title                                            lever / nature                   composes-with             effort wave
  H-49  undo/redo/branching file time-travel + lossless  STEERING/TRUST / BUILD-on-shipped H-35,E-25,H-6,E-29,C-23   L      8  depends H-35+E-25+H-6
        history (CAS object store WRAPPING edit_checkpoints + an undo-tree HEAD over the SessionDag; redo + branch-on-
         divergence; the RV-1 divergence guard; 3 grains edit/turn/babystep; CAS default + shadow-git option; paged
         virtual-scroll transcript via recover_stubs+E-29; delivers H-35's DEFERRED branch-fork + redo + cloud code-restore)

TRACK 9 — CONTEXT LIFECYCLE (regime-aware in-session management + token-cheap cross-session handoff; BUILD-on-shipped)
  H-N   title                                            lever / nature                   composes-with             effort wave
  H-50  regime-aware in-session context management       CHEAPER/STEERING / BUILD-on-shipped H-1/H-2,H-6,H-35/H-49   M-L    9  [the local-unprotected fix]
        (ONE window source of truth + window-derived trim thresholds; LOCAL unload/load = route the SHIPPED ladder/
         masking into the interactive turn + keep stubs unloaded + selective reload; CLOUD keep-cache-stable-prefix;
         optional Tier-2 unbypass of the sidecar PrefixKVCache; kills the flat-100K / 64K-non-opus / two-windows bug)
  H-51  sleep-time handoff: digest + auto-detect + tiered CHEAPER/STEERING / BUILD-on-shipped H-50,H-6,H-35,the-rule  M      9  depends H-50
        (sleep-time grounded TOPIC-digest auto-authored from the SSB journal; auto-DETECT + one-tap confirm NOT blind
         continue; tiered load ~20-27K -> ~1-3K on local; wires the live .claude/rules/handoff-protocol.md into the
         runtime; raw transcript lazy via H-50 selective recover_stubs)

TRACK 10 — REVIEW-ANCHORING (deterministic snippet->hunk review-finding anchoring for the cockpit; stolen from alibaba/open-code-review; BUILD)
  H-N   title                                            lever / nature              composes-with             effort wave
  H-52  deterministic snippet->hunk finding-anchoring    VERIFIED-BEFORE-SEEN / BUILD G-5, C-7, E-41, F-4         M     10  [closes G-5 Open-Question #2]
        + line-less finding schema (the model emits a verbatim snippet, NEVER a line; deterministic code computes the
         hunk/line; 0,0 = can't-anchor -> file-fallback, never a guessed inline line; duplicate-snippet -> Ambiguous
         guard + hunk-proximity tiebreak the source lacks; ONE FindingAnchored SSB row; G-5 score_review_surface
         consumes it to join vouches at the right hunk; RE-LOCATION LLM retry + the E-41 GitHub-PR leg DEFERRED Tier-2;
         "smart bundling"/"reflection module" are excluded vaporware — grep-verified absent from the source)

TRACK 11 — EXTERNAL-SYSTEM RESEARCH STEALS (two cant-lie hardenings from researching Perplexity + AlphaProof; BUILD; grounded on ELAI's own substrate)
  H-N   title                                            lever / nature              composes-with             effort wave
  H-53  citation pre-binding + constrained synthesis     VERIFIED-BEFORE-SEEN / BUILD E-48, H-52, E-35/C-26, D-16 M-L  11  [ghost citations made structurally impossible]
        (freeze the ranked evidence as numbered anchors injected BEFORE generation; the answer's [N] resolve ONLY to
         pre-bound anchors -> E-48 verify becomes a closed-set membership check, an ungroundable claim refuses (D-16),
         never an invented cite; Tier-2 constrained-decode grammar makes an out-of-set [N] unemittable; from Perplexity's
         answer stage — the mechanism is MEDIUM/analyst-reported, the steal rests on ELAI's D-11/E-48, GD-5)
  H-54  obligation-closure escape-hatch gate             TRUST/PROVABLE / BUILD       E-53, H-44                 M    11  [the relocated-sorry / hallucinated-known-result defense]
        (walk the FULL dependency closure of a discharged goal; a sorry/admit/axiom/assume ANYWHERE disqualifies, not
         the top statement; per-verifier hatch map Verus/Kani/Dafny/Z3/Lean; fail-closed + a REGISTERED allowlist for
         declared-open obligations; the deterministic FIRST leg H-44's keyword grep was a stand-in for; from AlphaProof's
         documented gate-gaming + whole-closure defense; R7 applied to a proof DAG — admitted != proved)

Each H-N's "Existing state" block names the prior-plan artifacts + the wire-dark ELAI symbols it composes
with (grep-confirmed in H-GROUNDING-MAP.md at authoring; the author re-greps to confirm — W-7).

================================================================================
CANONICAL CONSTANTS + SCHEMA_VERSION BUMP SITES (LIVE baseline 2026-06-03; NEVER hard-code; re-read at edit)
================================================================================
SCHEMA_VERSION = 207  (crates/orchestrator/src/ssb.rs:2909). TOTAL_VARIANTS = 484
  (crates/orchestrator/tests/action_type_lock.rs:549). AUTHORITATIVE, read from current source.
  STALE — do NOT use as baseline: CLAUDE.md header 191/464; F-plan memory note 205/482.
  LATENT DRIFT to fix: e45_cross_language_drift_contract.rs:294 still pins 482 (will FAIL on re-run);
  the FIRST H-N that bumps SCHEMA also bumps this line to keep e45 green.

Each H-N that adds an SSB action_type bumps from the LIVE baseline (207 -> 207+N; 484 -> 484+N), updating
BOTH ssb.rs:2909 (SCHEMA_VERSION) AND action_type_lock.rs:549 (+ALL_VARIANTS array). Treat 207/484 as the
2026-06-03 snapshot — concurrent G-plan work may move it; re-read the live integer at edit time (W-9).

  H-1   conditional: +cache_efficiency_sampled (Pattern-2) for the hit-rate off-agent artifact, OR emit
        via an existing row — author decides; if a new variant, baseline+1. (Prefer reuse; minimise bumps.)
  H-3   conditional: StreamingEarlyTerminated currently has NO production emitter (only tracing::warn). If
        the variant already exists, ACTIVATE the emitter (no bump); if absent, +streaming_early_terminated baseline+1.
  H-5   +batch_lifecycle (Pattern-2; Kind: Submitted / Completed / Expired). baseline+1.
  H-6   NO new action_type — reuse the existing ContextReduction payload (ssb.rs:10134, write-active);
        H-6 activates its PRODUCER on the live read path. (GD-1 compose.)
  H-7   NO new action_type — RouteDecisionEvent already exists wire-dark (ssb.rs:8153, Pattern-2 43rd, no
        emitter). H-7 wires the PRODUCER (Ssb::write_payload) + promotes the gate. (GD-1 compose with E-31.)
  H-9   NO new action_type — CloudSpend already exists, dark (grounding_client.rs:922). H-9 attaches the
        SSB writer so the row lands. (GD-1 compose with C-5.) cost_per_task advisory optional.
  H-11  +canvas_activity_event (the live spatial overlay; carries file_path/symbol_name/mode fallback for
        ghost-node reconciliation). baseline+1. H-11 owns the LIVE-LAYER overlay transport (it ships before
        H-13 in Wave 2, so the overlay variant cannot belong to H-13).
  H-13  +canvas_op_applied (Pattern-2; 10-arm Kind: NodeCreated/EdgeConnected/NodeDeleted/EdgeDeleted/
        AnnotationAdded/LabelEdited/GroupCreated/NodeForked/ScreenshotCaptured/LayoutChanged). baseline+1.
        H-13 owns the model-driven OP-APPLICATION transport.
  H-15  +canvas_view_event (Pattern-2; Kind: ViewSpawned / UiAction / StateCheckpoint). baseline+1.
        DECISION (resolves CANVAS §18 "pick one, do not maintain two"): there are exactly THREE distinct
        canvas SSB transports, one per concern — H-11 owns the LIVE OVERLAY (CanvasActivityEvent), H-13 owns
        model OP-APPLICATION (CanvasOpApplied), H-15 owns the REPLY-CHANNEL meta-events (CanvasViewEvent:
        spawn_view + inbound deixis + snapshot/delta). None subsumes another; none is duplicated.

  TRACK 3 (each adds ONE off-agent-artifact action_type; Pattern-2 single-variant-with-Kind preferred; baseline+1
  EACH, re-read the LIVE integer at edit — Track 3 runs in WAVE 3 after Track 1/2 + G-plan, so it WILL be past 207;
  the FIRST Track-3 H-N to bump SCHEMA also bumps the stale e45_cross_language_drift_contract.rs:294 pin to live):
    H-17  +TrialRegistered (the trial ledger off-agent row). baseline+1.
    H-18  +DeflationGateEvaluated (the {DSR,PBO,N} promotion triple). baseline+1.
    H-19  +HoldoutBudgetEvent (Pattern-2; Kind: QuerySpent / Exhausted / Provisioned). baseline+1.
    H-20  +LeakageBlocked (Pattern-2; Kind: FitReachesBothSinks / NormalizationAcrossBoundary / FullSeriesPCA / MissingShift). baseline+1.
    H-21  +LookaheadDifferential (Pattern-2; Kind: Causal / FutureLeak). baseline+1.
    H-22  +TradingNullTest (Pattern-2; Kind: FutureShuffleSurvived / PermutationNullFail / CostLawViolated / SeedNondeterministic);
          per-relation failures REUSE the existing MrViolated (no new variant for those — minimise bumps). baseline+1.
    H-23  +RerunHashEquality (Pattern-2; Kind: Equal / Divergent). baseline+1.
    H-24  +SimRealParity (Pattern-2; Kind: ModeDivergentBranch / ReplayGoldenMismatch / SingleEntrypointMissing). baseline+1.
    H-25  +ResultManifestVerified (Pattern-2; Kind: ReDerived / FabricationDetected / MonkeypatchBlocked). baseline+1.
    H-26  +DifferentialDivergence (Pattern-2; Kind: ShadowDiverged / InvariantViolated / NakedFloatEq); failures REUSE MrViolated. baseline+1.
    H-27  +GateSelfValidation (Pattern-2; Kind: Passed / MutantEscaped / MrFailed / TrapAccepted). baseline+1.
    H-28  +RewardTampering (Pattern-2; Kind: HarnessHashMismatch / ImplausibleScoreDelta / StructuralSelfModRefused);
          the hash-mismatch leg REUSES the shipped ArchiveVerifyResult tripwire (no new variant for it). baseline+1.
    H-29  +DriftInterlock (Pattern-2; Kind: Warning / Drift / Halt); REUSE the shipped model_drift_suspected signal. baseline+1.

  TRACK 4 (UX + USER-MEMORY; LIVE BASELINE MOVED — re-confirmed 2026-06-04: SCHEMA_VERSION=220 at ssb.rs:2990,
  TOTAL_VARIANTS=497 at action_type_lock.rs:624; the 207/484 above is the 2026-06-03 snapshot and is now STALE.
  Each H-N bumps from the LIVE 220/497, SHIP-ORDER-CUMULATIVE; the e45_cross_language_drift_contract variant pin
  is still stale (~482) and the FIRST Track-4 bump must reconcile it):
    H-30  +StalenessEvalMeasured (Pattern-2; the off-agent eval row). baseline+1.
    H-31  +UserDecisionSuperseded (Pattern-2; Kind: Superseded / Captured / ProposalSurfaced). baseline+1.
    H-32  +fact_validation_failed (Pattern-2; read-side validity gate). baseline+1. (prefer folding into H-31's flag; no second flag)
    H-33  +MemoryGovernanceEvent (Pattern-2; Kind: Forgotten / Pinned / Unpinned). baseline+1.
    H-34  +mcp_tool_deferred AND +mcp_schema_budget_measured (TWO variants). baseline+2.
    H-35  +SessionRewound (Pattern-2). baseline+1.
    H-36  +agent_notification_emitted (Pattern-2). baseline+1.
    H-37  +plan_decision_event (Pattern-2; Kind: Approve / Redirect / Reject / AutoAdvance; payload mirrors GateDecisionPayload). baseline+1.
    H-38  +StopConfirmationEvent (Pattern-2; Kind: ConfirmedProceed / RejectedHeld / PassedNoFriction). baseline+1.
  Net Track-4 SCHEMA delta if all ship = +10 variants (220->230, 497->507); confirm the live integer at each edit (W-9).

  TRACK 6 (GROUNDED-REASONING; LIVE baseline 220/497 same as Track 4; ship-order-cumulative; confirm the live integer at
  edit; the FIRST Track-6 H-N to bump SCHEMA reconciles the stale e45_cross_language_drift_contract variant pin ~482):
    H-39  +AbstentionScored (Pattern-2; the abstention-adjusted promotion/eval reward row). baseline+1.
    H-40  NO new action_type — EXTEND IsoBudgetReport + reuse the shipped iso_budget_measured row (add unique_catch +
          decorrelation fields to its payload). GD-1 compose, minimise bumps.
    H-41  +CheckerGapDetected (Pattern-2; the claim-class-fell-to-Mode-B/refusal-for-lack-of-checker signal); the live
          Mode-A coverage RATIO is a roll-up over the shipped grounding-mode + CheckerResultEmitted rows (no variant). baseline+1.
    H-42  +ReasoningBudgetScaled (Pattern-2; the serial-depth-scaled budget + DepthBound/AccessBound classification). baseline+1.
  Net Track-6 SCHEMA delta if all ship = +3 variants (H-40 reuses the shipped row).

  TRACK 7 (DGM-VERIFICATION; LIVE BASELINE MOVED — re-confirmed 2026-06-05: SCHEMA_VERSION=224 at ssb.rs:3010,
  TOTAL_VARIANTS=501 at action_type_lock.rs:645; the 207/484 and 220/497 above are EARLIER snapshots and are now STALE.
  Each H-N bumps from the LIVE 224/501, SHIP-ORDER-CUMULATIVE; the FIRST Track-7 H-N to bump SCHEMA also reconciles the
  stale e45_cross_language_drift_contract.rs:294 pin (~482); re-read the live integer at each edit — W-9):
    H-43  +FitnessEvaluated (Pattern-2; Kind: ExecutionPassRate / DualAgreement / FitnessRegression). baseline+1.
    H-44  +TrapBehaviorDetected (Pattern-2; Kind: ImpossiblePassExploit / IsomorphMismatch / ExecutionHallucination /
          KeywordFallback); the recall roll-up rides this variant's payload (no 2nd variant). baseline+1.
    H-45  +PromotionScoreAggregated (Pattern-2; Kind: WorstCaseScored / VariancePenalized / LineageDiverseDisagree). baseline+1.
    H-46  +CladeSelected (Pattern-2; Kind: CladeMetaproductivity / ImmediateScoreFallback). baseline+1.
    H-47  +EvalFirewallEvent (Pattern-2; Kind: PreUpdateItemRejected / TemplateRegenerated / IndistinguishabilityProbe);
          the holdout-budget leg REUSES H-19's HoldoutBudgetEvent (no new variant — GD-1). baseline+1.
    H-48  NO new action_type (default) — REUSE D-5's BAI promotion telemetry; +BanditArmSampled only if darwinian_bai
          has no reusable emitter (conditional baseline+1; prefer reuse). author decides at edit.
  Net Track-7 SCHEMA delta if H-43..H-47 ship = +5 variants (224->229, 501->506); H-48 conditional. Confirm the live integer at each edit (W-9).

  TRACK 8 (REVERSIBILITY; LIVE baseline 224/501 as of 2026-06-05, SAME snapshot as Track 7; re-read the live integer at edit
  — Track-7 H-43..H-47 may move it +5 if they ship first; the FIRST Track-8 bump reconciles the stale
  e45_cross_language_drift_contract pin if no prior track has):
    H-49  +EditHistoryEvent (Pattern-2; Kind: SnapshotCaptured / Deduped / Redo / Branched / Jumped / TooLargeSkipped) for the
          file-CAS capture + forward/branch navigation; REUSE H-35's SessionRewound for the undo/restore beacon (GD-1, no 2nd
          variant). baseline+1.
  Net Track-8 SCHEMA delta = +1 variant (the undo beacon reuses H-35's SessionRewound). Confirm the live integer at edit (W-9).

  TRACK 9 (CONTEXT LIFECYCLE; LIVE baseline 224/501 as of 2026-06-05, SAME snapshot as Track 7/8; re-read the live
  integer at edit — Track-7/8 + H-50 variants may move it; the FIRST Track-9 bump reconciles the stale
  e45_cross_language_drift_contract pin if no prior track has):
    H-50  REUSE the existing ContextReduction payload for unload events (GD-1, no new variant); optional +1
          WindowPolicyApplied (Pattern-2) ONLY if a distinct off-agent regime-decision row is needed (prefer reuse).
    H-51  +HandoffLifecycleEvent (Pattern-2; Kind: DigestWritten / ResumeOffered / ResumeConfirmed / TieredLoad). baseline+1.
  Net Track-9 SCHEMA delta = +1 variant (H-50 reuses ContextReduction; H-51 adds one). Confirm the live integer at edit (W-9).

  TRACK 10 (REVIEW-ANCHORING; LIVE baseline 224/501 as of 2026-06-05, SAME snapshot as Track 7/8/9; re-read the live
  integer at edit — Track-7/8/9 variants may move it; the FIRST Track-10 bump reconciles the stale
  e45_cross_language_drift_contract pin if no prior track has):
    H-52  +FindingAnchored (Pattern-2; Kind: Anchored / FileFallback / Ambiguous / Unanchorable / Relocated). baseline+1.
  Net Track-10 SCHEMA delta = +1 variant. Confirm the live integer at edit (W-9).

  TRACK 11 (EXTERNAL-STEALS; LIVE baseline 224/501 as of 2026-06-05, SAME snapshot as Track 7/8/9/10; re-read the live
  integer at edit — prior-track variants may move it; the FIRST Track-11 bump reconciles the stale
  e45_cross_language_drift_contract pin if no prior track has):
    H-53  PREFER REUSE — extend E-48's citation_verified payload (+anchor_id +pre_bound) for the closed-set verify leg
          (GD-1, no new variant); +CitationAnchorsBound (Pattern-2) ONLY if a distinct pre-binding off-agent row is
          required (author decides at edit; prefer reuse to minimise bumps; conditional baseline+1).
    H-54  PREFER COMPOSE — add a ClosureEscapeHatch Kind arm to H-44's TrapBehaviorDetected if H-44 has shipped (GD-1, no
          new variant); +ObligationClosureScanned (Pattern-2; Kind: ClosureClean / EscapeHatchAnywhere / RegisteredException)
          ONLY if standalone (conditional baseline+1).
  Net Track-11 SCHEMA delta = 0..+2 variants (both prefer reuse/compose; confirm the live integer at edit, W-9).

================================================================================
SEQUENCING (cheap-and-grounded first; the headline number + the moat last)
================================================================================
  WAVE 1 (efficiency activation; low risk, high leverage, mostly no new deps):
    H-1 -> H-2 -> H-9 FIRST (caching + cache-stable layout + cost VISIBILITY — prove the measured hit-rate
      and $/task actually move; H-9 is the instrument that makes H-1/H-2 falsifiable). Then H-3 + H-5 in
      parallel (output stack + batch, disjoint files). Then H-6 + H-4. Then H-7 (the heaviest — calibrated
      routing on the live cap-enforcement leg). H-8 last (small, isolated persistence).
  WAVE 2 (the canvas; the new product surface):
    H-10 FIRST and BLOCKING (the C1.B0 worst-case DKG measurement gates the renderer choice; nothing
      renders until the number is known). Then H-11 + H-12 (live layer + explorer; PLAN/RESEARCH parity)
      reach orbit feature-parity. Then H-13 (model drives the canvas). Then H-15 (spawn_view reply channel
      — needs H-13's op transport + COMPOSES-WITH G-1 taint + G-2 RRA). Then H-14 (DESIGN mode — riskiest,
      gated on a recast prove-it spike). Then H-16 LAST (retire the orbit only after Canvas-as-default
      ships a full release with no regressions; single-consumer SSB handover at the switch).
  WAVE 3 (TRUST-HARDENING; the autonomous-trading-trust gates; per the design's PART 6 dependency order):
    H-17 FIRST and BLOCKING (the trial ledger keystone — nothing else is trustworthy without persisted N).
      Then H-18 (DSR/PBO gate, reads N) — H-17->H-18 is the keystone pair. Then CLUSTER B/C in parallel after
      H-17: H-20 (leakage taint) + H-21 (runtime differential — the un-gameable backstop) + H-22 (trading nulls).
      Then CLUSTER D/E in parallel: H-23 (re-run hash) + H-24 (sim-real parity) + H-25 (result manifest) + H-26
      (shadow exec). Then CLUSTER F (the meta-layer capstone): H-27 (verify-the-verifiers) + H-28 (reward-tampering)
      — these depend on the gates existing (they validate H-18/H-20/H-21/H-22/H-26 and freeze the harness+H-17 ledger).
      Then CLUSTER G: H-19 (holdout budget) + H-29 (drift FSM, depends H-18). H-29 LAST (gates the live-trading resume edge).
      Note: WAVE 3 presumes ELAI's autonomous trading-search lane; the live lane is W-10-tiered (fixture harness
      now, live run a separate gated checkbox), so WAVE 3 ships the gate spine without a live-trading dependency.
  WAVE 4 (UX + USER-MEMORY; instrument-first; mostly activation + host-side, low risk):
    H-30 FIRST and BLOCKING (the staleness instrument — H-31/H-32 may not ship until it reports recognition AND
      application on the supersession path). Then H-31 (activate the bi-temporal ledger into capture+retrieval) ->
      H-32 (code-grounded validate-before-use) -> H-33 (memory hub + governance). Then the 5 free-wins, mostly
      parallel (disjoint files): H-34 (MCP defer-load) + H-35 (/rewind) -> H-36 (notifications) -> H-37 (plan gate)
      -> H-38 (cost-confirm; depends H-9 spend ledger). All host-side / journal-read = ~zero local-token cost.

  WAVE 6 (GROUNDED-REASONING HARDENING; refinements/instruments on shipped substrate; low risk, parallel-safe):
    H-39 FIRST (the keystone — the abstention-aware reward the whole self-improvement loop climbs; everything else is a
      verdict that feeds it). Then H-40 + H-41 + H-42 in parallel (disjoint files): H-40 decorrelation retention on E-6,
      H-41 the checker-coverage instrument on D-10/D-11, H-42 depth-scaled budget on Phase-2C GRC. All in-process, no new
      deps, no RunPod axis. Each defaults ON with a kill-switch; the FP/over-flag rate is bounded on >=3 runs (GD-4) where
      it changes a decision (H-40 retention, H-42 scaling). Inherits W/GD/HD + ADDS RD-1..RD-4.

  WAVE 7 (DGM-VERIFICATION HARDENING; replace the two placeholder filters; the loop's convergence precondition):
    H-43 FIRST and BLOCKING (the execution-grounded fitness keystone — every other Track-7 step optimizes/aggregates/
      selects-on a signal that is meaningless until the byte-hash is replaced; a regret-bounded bandit or a clade over a
      hash is theatre, VD-1). H-44 in PARALLEL with H-43 (disjoint file: trap_tasks vs execution_fitness) — the behavioral
      detector with measured recall. Then, AFTER H-43: H-45 (worst-case + lineage-diverse promotion scoring + PRMBench tau),
      H-46 (clade-metaproductivity selection), H-47 (the eval firewall — date-gate + template-regen + H-19 budget binding,
      composing D-6/E-54/H-19) in parallel (disjoint files). H-48 LAST (the BAI bandit ACTIVATED on H-45's aggregated real
      reward — depends H-43 + H-45; the stub_ban forbids a bandit on a byte-hash reward). All behind the existing
      cfg(feature="darwinian-evolve")+ELAI_DARWINIAN_LOOP_ENABLE kill-switch (allowlisted, W-2). W-10-tiered: every H-N
      ships a deterministic frozen-fixture done-gate now; the live multi-generation bench/regret run is a separately-gated
      checkbox (RUNPOD-PENDING per the dossier PART F — 1+2+5 only become MEANINGFUL once the Tier-2 real bench exists).
      Inherits W-1..W-10 + GD-1/2/4/5 and ADDS VD-1..VD-5. E-53 (Verus/Kani proof gate) is COMPOSE-NOT-AUTHOR — the narrow
      CSQ-Critical ceiling on top (VD-5), never routine.

  WAVE 8 (REVERSIBILITY / TIME-TRAVEL; one session, BUILD-on-shipped, low risk; runs AFTER Wave 4's H-35/H-6):
    H-49 (the whole feature) — depends H-35 (/rewind + SessionRewound) + E-25 (edit_checkpoints engine) + H-6 (recover_stubs
      read-back). Build order WITHIN H-49: (1) CAS object store + file_manifest column + RestoreEngine trait (CAS default,
      shadow-git stub); (2) the capture hook from file_ops.original_file (default-ON, kill-switch ELAI_TIMETRAVEL_DISABLE,
      replacing the dark ELAI_E25_EDIT_CHECKPOINTS_ENABLE heuristic); (3) HEAD undo/redo + branch-on-divergence + the RV-1
      divergence guard + the 3 grains; (4) the lossless-transcript precondition + recover_stubs wiring (with H-6) + the E-29
      paged virtual-scroll viewer (replacing TranscriptDrawer ENTRY_LIMIT=50); (5) the C-23 BranchTree timeline + the
      Restore-Files-Only dashboard surface. Each sub-step ships producer+consumer+smoke together (R6). Inherits W-1..W-10 +
      GD-1/2/4/5 + HD-1..HD-5 and ADDS RV-1..RV-4. GD-4: the divergence-guard false-fork rate is bounded on >=3 runs before default-ON.

  WAVE 9 (CONTEXT LIFECYCLE; runs after Wave 1's H-1/H-2 + Wave 4's H-6/H-35; low-medium risk, mostly wiring):
    H-50 FIRST (the local-unprotected fix is load-bearing): ONE window source of truth + window-derived thresholds, then
      the LOCAL unload/load policy (route the shipped ladder + H-6 masking into the interactive turn, keep stubs unloaded,
      make recover_stubs selective), leaving the cloud caching path intact; the optional sidecar PrefixKVCache unbypass is
      a Tier-2 spike. Then H-51 (depends H-50's selective recovery): the sleep-time grounded topic-digest auto-authored
      from the journal + auto-DETECT + one-tap confirm + tiered load. Each sub-step ships producer+consumer+smoke (R6).
      Inherits W-1..W-10 + GD-1/2/4/5 + HD-1..HD-5 and ADDS CL-1..CL-4. GD-4: the local false-drop rate (context still
      needed) is bounded on >=3 runs before default-ON.

  WAVE 10 (REVIEW-ANCHORING; one session, BUILD; runs AFTER G-5's cockpit + C-7's hunk_id exist; low risk, in-process):
    H-52 (the whole feature) — the deterministic snippet->hunk resolver + the line-less finding schema. Build order:
      (1) review_anchor.rs (normalize -> match a consecutive run against the hunk NEW-then-OLD side -> hunk-proximity
      tiebreak -> duplicate->Ambiguous guard -> Unanchorable->file-fallback), REUSING ELAI's existing diff parser (W-7);
      (2) the ReviewFinding line-less schema (no model-settable line/verdict); (3) the FindingAnchored SSB variant
      (baseline+1); (4) the additive G-5 score_review_surface wire (consume FindingAnchored, join at hunk granularity —
      closes OQ#2); (5) the cockpit render (Anchored-inline vs FileFallback-band). DEFER the RE-LOCATION LLM retry + the
      E-41 GitHub-PR leg (Tier-2). Inherits W-1..W-10 + GD-1/2/4/5 + HD-1..HD-5. GD-4: the over-fallback (Ambiguous +
      FileFallback) rate is bounded on >=3 runs before default-ON. Depends G-5 (cockpit) + C-7 (hunk_id); composes
      E-41/C-40 (the deferred PR leg).

  WAVE 11 (EXTERNAL-STEALS; two independent cant-lie hardenings; parallel-safe — disjoint file scope):
    H-53 + H-54 run in PARALLEL (no inter-dependency; H-53 touches the answer/citation path, H-54 the verifier lane).
    H-53 (citation pre-binding): build order — (1) citation_prebind.rs (EvidenceAnchorSet + rank-stable ids under
      ELAI_SEED + freeze step); (2) the Evidence context slot AFTER the last cache breakpoint (HD-2; coordinate with H-50
      per-backend window budget); (3) the closed-set verify (extend E-48 CitationVerifier + citation_verified); (4) the
      D-16 refusal on an ungroundable claim. DEFER the E-35/C-26 constrained-decode grammar (Tier-2; marker positions only,
      HD-3) + the local-window tuning. Depends E-48 + E-35/C-26; composes H-52/D-16/3E. GD-4: the over-refusal rate is
      bounded on >=3 runs (cross-check D-17 healthy band) before default-ON.
    H-54 (obligation-closure scan): build order — (1) verifier/closure_scan.rs (BFS over the verifier-exposed obligation
      DAG + the per-verifier HatchKind map + the registered-exception check), REUSING each E-53 verifier's obligation
      exposure (W-7); (2) the obligation-closure-allowlist.toml (R8-frozen via gate_freeze_check.py); (3) wire it as the
      cheapest FIRST leg of the E-53 lane (+ H-44's deterministic first leg, demoting the keyword grep to KeywordFallback);
      (4) the off-agent row (compose H-44 TrapBehaviorDetected, or ObligationClosureScanned standalone). Depends E-53;
      composes H-44 + the R5/R7/R8 doctrine + Track-7 VD-1..VD-5. GD-4: the false-block rate is bounded on >=3 runs.
    Inherits W-1..W-10 + GD-1/2/4/5 + HD-1..HD-5 (H-54 also Track-7 VD-1..VD-5); ADDS no new delta family.

THE THREE HEADLINE MOVES:
  (Track 1) H-1 + H-2 + H-9 together = the MEASURED efficiency number (cache-read fraction + $/resolved-task
    tuple, reproducible from the journal under ELAI_SEED) — the believe-the-number control that pairs with
    G-6 SAFEDELTA and is the SAME measurement discipline as the trust number (ELAI-EFFICIENCY §14).
  (Track 2) H-15 (spawn_view on the journaled + reproducible + taint-tagged op-stream) = the canvas MOAT
    consumer products (Artifacts / ChatGPT Canvas / MCP Apps / tldraw) structurally lack, because it rides
    G-1 taint + G-2 RRA. The canvas is table-stakes; the op-stream underneath is the differentiator.
  (Track 3) H-17 + H-18 + H-27 + H-28 together = the AUTONOMY-TRUST spine: a persisted-N trial ledger (H-17)
    deflating every reported edge (H-18 DSR/PBO), a frozen self-validated evaluator the optimizer cannot game
    (H-27 verify-the-verifiers + H-28 reward-tampering). This is what lets ELAI run an AUTONOMOUS strategy
    search/build loop the operator can TRUST — the deterministic answer to "the search itself ships overfit
    edge even if every agent is honest" (TB-1/TB-3). The same off-agent-artifact + fail-closed discipline as
    the F/G trust gates, applied to the finance/autonomous-search domain.

================================================================================
HARD RULES INHERITED FROM CLAUDE.md + F-0/G-0 (apply to every H-N)
================================================================================
- blake3 only (never blake2/sha2 for non-RFC uses).
- No .unwrap()/.expect() in production paths (test cfg + main entrypoint allowed).
- Anti-slopsquatting: verify name+repo+version+date+license BEFORE any dep. Per-H-N dep flags (from the
  2026-06-03 web verification — BINDING): H-10 pin sigma@3.0.3 (v4 is alpha-only = BLOCK) + cytoscape@>=3.31
  (3.34.0 latest, WebGL renderer PROVISIONAL — graceful canvas fallback) + cytoscape-layout-utilities@1.1.1
  (real but STALE 2021 — flag for staleness review, registered before use); H-11 tauri-plugin-fs (verify the
  current 2.x version via cargo, do NOT assume); H-14 recast@0.23.11 (MIT, verified) + MIRROR @onlook/parser
  ids.ts in-repo, do NOT depend on @onlook/babel-plugin-react (phantom/legacy, 403, absent from current repo = BLOCK).
- One change at a time; the auditor is the sole test runner; the main LLM never runs tests.
- Grounded-by-default: every external-behaviour claim cites the synthesis doc + its primary source; the
  CARRIED WEB LANDMINES below are BINDING.

================================================================================
CARRIED WEB LANDMINES (verified 2026-06-03 — DO NOT RE-INTRODUCE the design-doc value; use the corrected one)
================================================================================
- 1-HOUR CACHE TTL needs NO beta header (CORRECTED). It is GA via "ttl":"1h" on the cache_control marker.
  extended-cache-ttl-2025-04-11 is the HISTORICAL header — the gate was REMOVED, not renamed. H-2 MUST NOT
  send it. (verify:caching-api)
- MIN CACHEABLE PREFIX is per-MODEL-ID, not a Sonnet-vs-Opus tier (CORRECTED). 1024: Opus 4.8, Sonnet 4.6/4.5;
  4096: Mythos, Opus 4.7/4.6/4.5, Haiku 4.5; 2048: Haiku 3.5. Opus 4.8 = 1024 (NOT 4096). H-1/H-2 key min-prefix
  on the EXACT model id, never "Opus=4096". (verify:caching-api)
- CACHE BREAKEVEN is on the FIRST read for 5-min TTL (1.25x+0.1x<2.0x), but the SECOND read for 1-hour TTL
  (2.0x write). Flag the 1-hour exception. (verify:caching-api)
- TOOL SEARCH reduction is ~77K -> 8.7K (85%), NOT ~72K (CORRECTED). Umbrella beta = advanced-tool-use-2025-11-20;
  a second tool-search-scoped header tool-search-tool-2025-10-19 also exists. The search tool itself must NEVER
  be deferred. A custom ANTHROPIC_BASE_URL DISABLES server-side deferral — ELAI implements its own. (verify:tool-search)
- BATCH API: 50% all models, /v1/messages/batches, 24h window / 29-day retention / 100K-or-256MB cap; supports
  tools+system+thinking; EXCLUDES streaming + Fast mode + max_tokens:0 + Threads; pair with 1h cache TTL.
  All CONFIRMED; no batch code exists today (H-5 is pure BUILD). (verify:batch-api)
- sigma v4 does NOT exist as a stable release (alpha-only) — pin sigma@3.0.3 (H-10). (verify:canvas-deps)
- ONLOOK is IFRAME-based, NOT its own renderer (REFUTED). The H-14 "proxied-iframe is novel" premise must
  re-ground on the SANDBOX/PROXY axis (ELAI proxies an arbitrary foreign app vs Onlook's CodeSandbox-SDK
  container over penpal), not renderer-vs-iframe. (verify:design-deps)
- ONLOOK data-oid is an OPAQUE GENERATED id (createOid), NOT a sourcemap-style location (CORRECTED). Store the
  id->{file,range,scope} map SERVER-SIDE keyed by the opaque id; do NOT encode line/col in the DOM attribute. (verify:design-deps)
- @onlook/babel-plugin-react is legacy/unfetchable (403), absent from the current repo (CORRECTED). Mirror
  @onlook/parser ids.ts (@babel/standalone) in-repo; do NOT depend on the phantom package. (verify:design-deps)

================================================================================
CARRIED DESIGN-DOC CORRECTIONS (activation-vs-build reality, from H-GROUNDING-MAP §DESIGN-DOC CORRECTIONS)
================================================================================
These flip whether an H-N is ACTIVATION (wire a dark symbol) or BUILD (create absent code). Authors MUST
read the full corrections C1..C17 in H-GROUNDING-MAP.md. The load-bearing ones:
- C1 (H-1): the main coding-agent loop ALREADY caches (handle_infer_request -> send_message ->
  apply_anthropic_cache_markers). H-1 is NOT "add caching to a markerless path" — it is "close the
  RuntimeAdapter::complete split-brain + activate the dark hit-rate monitor." The efficiency doc's "zero
  markers on the production path" is WRONG.
- C3/C5/C6 (H-2/H-3): the breakpoint-stability mechanism is computed-but-inert (bp.valid -> eprintln only),
  fed empty slices for slots 2-7, Local-branch-only. Streaming early-abort is dark on ALL paths (premise
  "sidecar-only" FALSE) — one-line activation at runtime_builder.rs:306. DIFF never emits in prod (editor=None).
- C7 (H-6): adaptive_reduce + recover_stubs + get_relevant_facts are real-but-never-called; prod substitutes
  a hardcoded "[stubbed — content archived in SSB]" placeholder. H-6 = ACTIVATE the read-backs.
- C10 (H-9): the cloud spend ledger is dark END-TO-END (.with_ssb_writer never called); CloudPanel renders
  ALL-ZEROS. H-9 = attach the writer + build the absent token-tracker UI.
- C11 (H-10, INVERTED): the symbol CALL graph IS already on canvas (build_symbol_view). PageRank is the
  producer-dark piece (dkg_nodes_pagerank table never materialised); dkg_edges carries NO call edges.
- C12 (H-12/H-16): the canvas-store isolation gate PASSES (no store imports three.js) — orbit retire is clean.
- C17 (H-13/H-15): the E-37 trust spine (Provenance/allow_tool_with_origin) is wire-dark END-TO-END; G-1
  taint_flow + G-2 RRA + canvas screenshot are ABSENT; but F-3p1 CommittedToolSet is LIVE and the
  media_validator Tier2 consumer is shipped. Taint-gated Select reuses capability_flow::taint_text (partial).

================================================================================
SESSION LOG (populated as H-Ns ship)
================================================================================
(Empty at generation. Each shipped H-N adds a row: context-compiler -> integration-explorer ->
spec-grounder -> executor -> auditor -> fixer (if BLOCKs) -> wiring-contract-validator (RUN_TRIGGER PASS
REQUIRED) -> dashboard-smoker (if frontend_pair) -> memory-writer -> COMMITTING.)

================================================================================
NEXT STEP
================================================================================
Start WAVE 1: H-1 (caching split-brain + hit-rate monitor) -> H-2 (cache-stable layout) -> H-9 (cost
visibility) to land the measured efficiency number, then the rest of Wave 1. WAVE 2 opens with H-10's
blocking B0 DKG measurement. WAVE 3 (TRUST-HARDENING) opens with H-17 (the trial-ledger keystone, BLOCKING)
-> H-18 (DSR/PBO), then the leakage + perturbation + reproducibility + meta-layer + drift clusters. Respect
W-1..W-10 + GD-1/2/4/5 + HD-1..HD-5 on Track 1/2, and ADDITIONALLY TD-1..TD-5 on every Track-3 (H-17..H-29) session.
  TRACK 4 (H-30..H-38, UX + USER-MEMORY) is authored 2026-06-04 and runs as WAVE 4, opening with H-30 (the
  staleness instrument, BLOCKING) -> H-31/H-32 (activate the two-tier user memory) -> H-33 governance -> the
  H-34..H-38 free-win QOL. Track 5 (the 6 odysseus steals + minor refinements, referencing docs/.source/odysseus-main)
  is SEQUENCED AFTER Track 4 per the approved instrument-first ordering.

  TRACK 6 (H-39..H-42, GROUNDED-REASONING HARDENING) is authored 2026-06-04 and runs as WAVE 6, opening with H-39 (the
  abstention-aware reward keystone, BLOCKING) -> H-40 + H-41 + H-42 in parallel. It inherits W-1..W-10 + GD-1/2/4/5 +
  HD-1..HD-5 and ADDS RD-1..RD-4. The finding is mostly CONFIRMATION (ELAI's trust spine already aligns with the rigorous
  literature); Track 6 is four small refinements + instruments, not a redesign.

  TRACK 7 (H-43..H-48, DGM SELF-IMPROVEMENT VERIFICATION HARDENING) is authored 2026-06-05 and runs as WAVE 7, opening
  with H-43 (the execution-grounded fitness keystone, BLOCKING — replaces the byte-hash derived_alpha) + H-44 in parallel
  (the behavioral trap detector replacing the keyword grep), then H-45/H-46/H-47 in parallel after H-43, then H-48 LAST.
  It inherits W-1..W-10 + GD-1/2/4/5 and ADDS VD-1..VD-5. The META-FINDING: a self-improvement loop without a real,
  un-gameable, stronger-than-generator verification filter PROVABLY collapses; Track 7 replaces ELAI's two placeholder
  filters (hash-fitness + keyword-traps) and upgrades the scoring/selection/eval to the convergent regime. Most of the
  dossier's PART D is already planned (D-2/D-5/D-6/D-13/E-6/E-53/E-54) — Track 7 is two placeholder replacements + a few
  compositional upgrades; E-53 (Verus) is compose-not-author. Track 5 (odysseus — the 6 odysseus steals + refinements,
  referencing docs/.source/odysseus-main) remains reserved + unauthored and, since Track 6 took H-39..H-42 and Track 7
  took H-43..H-48, Track 8 took H-49, Track 9 took H-50/H-51, Track 10 took H-52, and Track 11 took H-53/H-54, will
  start at H-55 (collision-safe).

  TRACK 8 (H-49, REVERSIBILITY / TIME-TRAVEL) is authored 2026-06-05 and runs as WAVE 8 (after Wave 4's H-35/H-6): the
  reversible-edit feature — a content-addressed object store wrapping E-25 edit_checkpoints, an undo-tree HEAD over the
  SessionDag (undo/redo/branch-on-divergence at any node, nothing lost), the RV-1 never-clobber-the-human divergence guard,
  the 3 user-selectable grains, and a lossless paged transcript (compose H-6 recover_stubs + E-29). It delivers H-35's three
  DEFERRED items (branch-fork keeping both timelines, redo, cloud-model code-restore via snapshot-not-replay). Inherits
  W/GD/HD and ADDS RV-1..RV-4. Design: docs/thefinalglance/UNDO-REDO-TIMETRAVEL-DESIGN-2026-06-05.md.

  TRACK 9 (H-50..H-51, CONTEXT LIFECYCLE) is authored 2026-06-05 and runs as WAVE 9 (after H-1/H-2 + H-6/H-35): the
  regime-aware context fix — H-50 makes in-session management backend-aware (one window truth + window-derived thresholds;
  LOCAL unload/load vs CLOUD cache-stable-prefix), closing the grounded gap that a long LOCAL session is unprotected today
  (flat-100K auto-compact + 64K-non-opus gate + recover_stubs re-inflate = dies at context_window_blocked) and that LOCAL
  caching is a no-op (sidecar PrefixKVCache bypassed on the streaming path); H-51 makes cross-session continuation
  token-cheap + semi-automatic (sleep-time grounded topic-digest + auto-detect + one-tap confirm + tiered load ~20-27K ->
  ~1-3K), wiring the user's live handoff-protocol.md into the runtime. Inherits W/GD/HD and ADDS CL-1..CL-4. Design:
  docs/thefinalglance/CONTEXT-LIFECYCLE-REGIME-AWARE-AND-HANDOFF-DESIGN-2026-06-05.md.

  TRACK 10 (H-52, REVIEW-ANCHORING) is authored 2026-06-05 from an external-STEAL exercise (databow + Anthropic's
  defending-code-reference-harness + alibaba/open-code-review fetched/cloned to docs/.source + mapped + explored) and
  runs as WAVE 10 (after G-5's cockpit + C-7's hunk_id exist): the deterministic snippet->hunk finding-anchoring resolver
  + the line-less finding schema, stolen from open-code-review's resolver.go — the ONE net-new steal of the three
  (databow has no ELAI fit; the Anthropic harness's patterns are already covered by Track 7). It closes G-5's own
  Open-Question #2 (the hunk_id join) and gives E-41's PR reviewer a deterministic anchor instead of a trusted model
  line; G-5's score_review_surface CONSUMES the FindingAnchored row. Inherits W/GD/HD; ADDS no new delta family. Detail:
  docs/thefinalglance/H-plan/H-52_deterministic_finding_anchoring_cockpit.md.

  TRACK 11 (H-53, H-54, EXTERNAL-SYSTEM RESEARCH STEALS) is authored 2026-06-06 from researching two external SYSTEMS
  (Perplexity AI via the oracle openness run + AlphaProof Nexus via the WebFetch-verified paper arXiv:2605.22763 + the
  proof corpus cloned to docs/.source/alphaproof-nexus-results) and runs as WAVE 11 (H-53 + H-54 parallel, disjoint
  files). H-53 (citation pre-binding + constrained synthesis) steals Perplexity's answer-stage SHAPE: freeze the ranked
  evidence as numbered anchors injected BEFORE generation so every [N] resolves only to a pre-bound anchor — moving ELAI's
  answer path from DETECT-AFTER (E-48 GhostCite hardening post-hoc) to PREVENT-BEFORE (ghost citations structurally
  impossible); composes E-48 (verify -> closed-set membership) + H-52 (the same deterministic verbatim->anchor resolver) +
  E-35/C-26 (Tier-2 grammar) + D-16 (refusal) + 3E (slot). The Perplexity-specific mechanism is analyst-reported (MEDIUM,
  GD-5) — the steal rests on ELAI's D-11/E-48, never a Perplexity number. H-54 (obligation-closure escape-hatch gate)
  steals AlphaProof's documented gate-gaming defense: a generator relocates the unproven `sorry` into a renamed helper
  lemma or hallucinates a false "known-result" lemma (both keyword-free at the top), so inspect the WHOLE dependency
  closure — a sorry/admit/axiom/assume ANYWHERE disqualifies; it is the deterministic FIRST leg of trap detection on the
  E-53 verifier lane (Verus/Kani/Dafny/Z3/Lean) and the deterministic core H-44's keyword grep (GAP-2 P0) was a stand-in
  for, the wiring R7 insight applied to a proof DAG (admitted != proved). AlphaProof is also external grounding for H-43
  (compiler-as-fitness not bytes) + TD-3 (LLM critic = search heuristic, compiler = gate) + E-53's HARD-AVOID of routine
  Lean (~2.5-9% pass, $/problem). Both inherit W/GD/HD (H-54 also Track-7 VD-1..VD-5); ADD no new delta family. Track 5
  (odysseus) reservation bumps to start at H-55. Details: docs/thefinalglance/H-plan/H-53_citation_prebind_constrained_synthesis.md
  + H-54_obligation_closure_escape_hatch_gate.md. Reasoning digests: workflow/reasoning/
  PERPLEXITY-CITATION-SYNTHESIS-STEAL-2026-06-06.md + ALPHAPROOF-DEPENDENCY-CLOSURE-STEAL-2026-06-06.md.

End of H-0_INDEX.md.
