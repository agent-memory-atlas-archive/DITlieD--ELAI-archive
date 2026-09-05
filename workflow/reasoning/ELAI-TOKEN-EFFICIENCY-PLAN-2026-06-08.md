ELAI TOKEN-EFFICIENCY IMPROVEMENT PLAN
Generated 2026-06-08. Synthesis of 5 GPT deep-research surveys + Opus synthesis agent + Grok-heavy
manual cross-model reasoning, grounded against ELAI's own wire-dark substrate (H-plan Track 1).
Research files: workflow/research/token-efficiency/01..05 + GROK-PROMPT.md.
Premise: ELAI already BUILT most of the token-efficiency substrate; it is WIRE-DARK. The win is
ACTIVATION (wiring an existing dark symbol + a test), not invention. Every number below is the
measured figure from the cited source; "vendor" = vendor-internal, not independent.

================================================================================
THE ONE-LINE THESIS
================================================================================
Activate caching + output + tool-search this week (the three levers that hit EVERY cloud request,
measured at 78.5% / 61-95% / 85%), instrument them with the cost cockpit FIRST so the gains are
falsifiable, then compound with routing + masking + batch. ELAI's off-agent-artifact verification
spine already dodges the worst multi-agent traps the research warns about.

================================================================================
GROUNDING POSTURE (measured vs vendor)
================================================================================
STRONGEST (independent academic, trust as targets): arXiv:2601.06007 (caching), arXiv:2606.05304
(PACT), arXiv:2601.14470 (Tokenomics), arXiv:2509.23586 (AgentDiet), arXiv:2605.00914 (Cost of
Consensus), SWE-Edit, JSONSchemaBench, A-RAG, OpenDev ACC, LocAgent/KGCompass, ICSE-2025 code
calibration, IFIM, CodeFast.
VENDOR (directional, not proof): Anthropic Tool Search 85%, Anthropic contextual-retrieval 67%,
Cursor self-summary/fast-apply, Relace 47%, Voyage reranker.
DERIVED (arithmetic, not a published benchmark): FIM ~95% output cut; "kill at token 50 not 500".
HD-1 house rule: every efficiency number is MEASURED per config on ELAI's own stack, never multiplied
across levers. The percentages below are upper-bound expectations, not promises.

================================================================================
WAVE 0 - INSTRUMENT FIRST (do before any activation)
================================================================================
W0. H-9 cost cockpit / spend ledger - ACTIVATION
  Why first: the spend SSB emit is gated behind an unattached writer, so CloudPanel renders zeros and
  the cache hit-rate monitor has no caller. Until the cache-read / cache-creation ratio is live you
  CANNOT tell whether H-1 actually fired. HD-1 makes every later % unfalsifiable without this.
  Do: attach .with_ssb_writer on the cloud client (the TemplateCrystallizer pattern already does this
  correctly); surface per-response input/output/CACHED breakdown + $/task + cache-hit-rate.
  Measured target: instrument only (0 direct saving); it is the believe-the-number prerequisite.
  H-plan map: H-9. Effort: S, pure activation.

================================================================================
WAVE 1 - PER-REQUEST CLOUD LEVERS (biggest $, every request)
================================================================================
W1a. H-1 + H-2 prompt caching full activation - ACTIVATION
  Gap: the RuntimeAdapter path hardcodes cache_control=None and ships ZERO markers for its secondary
  callers (hook-prompt, focus-tunnel, judge) despite supports_caching=true; the 2 breakpoints + blake3
  byte-stability flag are inert (flag consumed by an eprintln), fed empty slices, reachable only under
  the Local budget policy. No 1h TTL.
  Do: unify marker logic onto the RuntimeAdapter path; feed real slot bytes; place markers on a
  verified-stable prefix only (consume bp.valid); add a SECOND tail breakpoint; opt into ttl:"1h"
  (NO beta header); enforce the HD-2 debug_assert that the cached byte-prefix is identical turn-to-turn
  (a cache-bust is a test failure).
  Measured (arXiv:2601.06007, 500+ sessions): system-prompt-only caching = 78.5% cost cut + 22.9% TTFT
  on Sonnet 4.5; 88% cost cut at 50K prompt. Anthropic mechanics: hierarchy tools->system->messages,
  exact-prefix, 4 breakpoints, read=0.1x / 5min-write=1.25x / 1h-write=2x.
  HAZARD GUARD: min cacheable prefix is 1,024 tokens on Sonnet 4.6 / Opus 4.8; below it TTFT REGRESSES
  10-18% - guard the first turns. Custom ANTHROPIC_BASE_URL keeps caching but disables server-side
  features. JSON key-order must be stable (serialization bug = silent cache miss).
  H-plan map: H-1, H-2. Effort: M, pure activation.

W1b. H-4 Tool Search Tool + advanced-tool-use beta on OAuth path - ACTIVATION
  Gap: no advanced-tool-use header; full 16-tool catalog loaded into every request; the OAuth
  production path emits only oauth-2025-04-20 so any with_beta is dead. C-30 dkg_* tools are
  catalog-only (no convert/dispatch arm).
  Do: join advanced-tool-use-2025-11-20 onto the OAuth beta header; move the catalog behind
  tool_search_tool_regex_20251119 gated on catalog size; fix the dkg_* tools via the E-40 3-arm
  pattern. Because a custom base_url disables server-side deferral, keep ELAI's own ToolSearch
  emulation.
  Measured (Anthropic vendor): ~77K -> 8.7K tool-def tokens, 85% reduction, 95% window preserved.
  H-plan map: H-4. Effort: M, pure activation.

================================================================================
WAVE 2 - OUTPUT-TOKEN STACK (output bills 5x; caching never touches it)
================================================================================
W2. H-3 diff/FIM emission + streaming early-abort - MOSTLY ACTIVATION
  Gap: FocusTunnel DIFF never emits in prod (editor_fs_override hardcoded None -> always whole-file
  STRUCTURED); FIM double-gated dark; streaming early-abort validator hardcoded None on ALL paths.
  Do: relax ModeSelector Rule 1 + thread a real BufferEditor so DIFF emits diffs on cloud (REUSE
  B-20 try_edit_tiers, shipped); construct Some(validator) at runtime_builder + call on_chunk every
  ~50 tokens, writing the StreamingEarlyTerminated SSB row; route LOCALIZED single-span edits on the
  sidecar to a BASE/FIM path.
  Measured: AdaEdit -61% output + -52% latency at best edit quality (academic); CodeFast early-abort
  -63% to -77% output, Pass@1 unchanged (academic); FIM ~95% output cut on localized edits (derived);
  plain diff only -10.7% output and -2.2pp completion (Aider) - so prefer AdaEdit-style/FIM over naive
  diff where the acceptance metric is sensitive.
  HAZARD GUARD (load-bearing): FIM on the INSTRUCT model COLLAPSES (Qwen2.5-Coder base 91% -> instruct
  21.8% on IHumanEval, IFIM paper). Route FIM to the BASE model only, or an IFIM adapter (recovers
  95.8%). Never force-JSON the coder role (HD-3). Speculative decoding is LATENCY not billed-tokens
  and is unavailable on Anthropic - skip on cloud, consider only on the self-hosted sidecar.
  H-plan map: H-3. Effort: L, activation + small validator wiring.

================================================================================
WAVE 3 - ORCHESTRATION (compound the above; 2 net-new builds here)
================================================================================
W3a. H-7 calibrated architect / editor / verifier routing - ACTIVATION + 1 BUILD
  Gap: per-role RuntimeOrchestrator + PerRoleRuntime built but UNWIRED; RouteDecisionEvent has no
  emitter; escalation hardcoded layer_failed=1.
  Do: wire architect (cloud flagship plans) + editor (local 7B emits cheaply-checkable edits) +
  verifier (the auditor in a separate context) on the live CloudModeRouter cap leg; emit
  RouteDecisionEvent; replace layer_failed=1 with a VERIFIER-GATED signal (off-agent check FIRST,
  calibrated log-prob SECOND, never self-confidence).
  NET-NEW BUILD (sequence before trusting any log-prob): recalibrate Platt scaling on the EXACT
  NF4-abliterated serving stack using verifier pass/fail labels. A BF16 threshold is invalid on NF4;
  abliteration removes refusal-as-uncertainty; self-verbalized confidence is unusable (Qwen-2.5-3B
  reports confidence on every prediction).
  Measured: SWE-Edit viewer+editor +2.1pp resolve at -17.9% cost (academic); Aider architect/editor
  lifts score even same-model (Sonnet 77.4->80.5%); token log-prob AUROC 0.7-0.83 OOD vs supervised
  routers; ICSE-2025 Platt scaling code-gen ECE 0.32 -> 0.03. Keep candidate pool 3-10 models.
  HAZARD GUARD: do NOT add model variants before verifier-gating is solid; the existing 1 local + 1-2
  cloud tiers already capture most routing gains (RouterEval).
  H-plan map: H-7. Effort: L, activation + calibration build.

W3b. H-6 deterministic observation masking + memory read-back - ACTIVATION
  Gap: adaptive_reduce, recover_stubs (real blake3 SSB recovery), get_relevant_facts have zero prod
  callers; prod substitutes a hardcoded "[stubbed]" placeholder.
  Do: make adaptive_reduce the default long-session manager; un-gate the E-25 ladder (cheapest-first);
  wire recover_stubs + get_relevant_facts into the live read paths; reuse the existing ContextReduction
  SSB payload (no new action_type).
  Measured: AgentDiet trajectory GC -39.9% to -59.7% input tokens, total cost -21.1% to -35.9% at
  parity (academic); OpenDev ACC -54% peak observation context, per-tool summarization 30K -> <100
  tokens (academic).
  HAZARD GUARD (the #1 interaction with Wave 1): summarization DESTROYS the cache prefix; deterministic
  tool-result clearing with a meaningful clear_at_least threshold PRESERVES it. Mask trailing
  observations WITHOUT busting the breakpoint; the HD-2 byte-stable assert must still hold; keep
  thinking:"all" to maximize cache reuse. Evict TOOL OUTPUTS first, never user intent or recent
  working memory.
  H-plan map: H-6. Effort: M, pure activation.

W3c. PACT structured action-state handoffs - NET-NEW BUILD (not in H-plan)
  Why: ELAI spawns fresh GC'd subagent contexts (correct) but the controller<->subagent handoff passes
  richer-than-needed content. PACT transforms each handoff into a compact public record: action-taken /
  next-action / grounded task-state / reusable-artifact pointer - private reasoning stays off the
  shared channel.
  Measured (arXiv:2606.05304): -38.7% tokens average; -47% tokens/resolved on SWE-agent; OpenHands
  19.4% -> 23.0% resolved while cutting tokens/resolved. Pairs with bounded review budgets (Agent
  Contracts: 50K/3-iter cut review tokens -90% with no significant success drop) to cap the verification
  sink (Tokenomics: Code Review = 59.4% of all tokens).
  H-plan map: NEW (propose as H-Track-1 addendum). Effort: M, build on the existing spawn protocol.

================================================================================
WAVE 4 - LANES + RETRIEVAL (free multipliers + refinement)
================================================================================
W4a. H-5 Batch API for non-interactive lanes - BUILD (small)
  Gap: no Anthropic Batch API anywhere; Cosmos refresh, web-research fan-out, bench/eval all pay 2x.
  Do: add a batch client (POST /v1/messages/batches) and route the non-interactive lanes; keep the
  interactive /task + chat REPL synchronous. RunPod-gated lanes ship a deterministic fixture harness +
  a separately-gated live checkbox (W-10).
  Measured: flat 50% off, zero quality cost.
  H-plan map: H-5. Effort: M, build (no batch transport exists).

W4b. H-8 persist OracleAnswerCache + retrieval tightening - ACTIVATION + refinement
  Gap: OracleAnswerCache is blake3-keyed tier-TTL but in-process only (lost on exit).
  Do: disk-back it for IDEMPOTENT grounding answers only (never context-dependent codegen - staleness
  is a correctness hazard); PromptCache already persists to disk, do not re-implement. Then tighten
  retrieval to snippet-first / signatures-before-bodies / reread-dedup, and expose precise LSP/SCIP
  symbol tools (ELAI already has tree-sitter + PageRank + SCIP map).
  Measured: A-RAG progressive disclosure ~49% fewer retrieved tokens + higher accuracy on multi-hop
  (academic) - BUT naive-agentic retrieval BALLOONS to 56K tokens, so the discipline (snippet-first +
  dedup) is what saves, not "give the agent retrieval tools"; LSP-as-retriever 5-34x fewer tokens than
  grep/read (OSS measured); LocAgent ~86% cost reduction (academic). Code-minification: source >90% of
  repair-prompt tokens, -42% input by stripping comments/docstrings/whitespace - NOT dedent (the outlier
  that hurts). Found-in-the-Middle: place key snippets at prompt EDGES.
  H-plan map: H-8 + new retrieval refinement. Effort: S build + refinement.

================================================================================
CROSS-CUTTING (decisions that hold across all waves)
================================================================================
1. ELAI ALREADY WINS THE MULTI-AGENT CONTRADICTION. Cost of Consensus (debate 2.1-3.4x tokens for
   equal/worse accuracy) and single-agent-with-skills (-54% vs MAS) condemn LLM DEBATE. ELAI does not
   debate: it spawns fresh GC'd contexts and verifies with OFF-AGENT artifacts (SSB rows, blake3,
   coverage, executable tests). That is exactly what the research endorses (deterministic gates >>
   LLM-as-judge; Compiled AI 57x fewer tokens at 1,000 tx). Do NOT add LLM critic loops; keep pushing
   recurrent checks into deterministic gates. Multi-agent is justified ONLY for breadth-parallel
   discovery / independent audit branches, never tightly-coupled edit-review-edit loops.
2. AGENTS.md / context-file discipline. The literature contradicts itself (-16.58% output one study,
   +20% cost / -3% perf another for LLM-generated broad files). Resolution: keep memory-bank /
   context-package / handoff files SPARSE, high-signal, operational (entry points, invariants, artifact
   pointers, stop conditions). Broad repo-overview dumps are pure haulage cost. Treat the blackboard as
   an interface contract, not a repo summary.
3. MEASURE BEFORE MULTIPLY (HD-1). W0 lands first; after each wave, record the real per-config delta
   from the SSB journal under ELAI_SEED. Never advertise a stacked product of percentages.
4. THE ONE INTERACTION TO WATch ACROSS WAVES: Wave 3b masking vs Wave 1 caching. Deterministic clearing
   with clear_at_least preserves the prefix; summarization or threshold-free clearing busts it and
   re-pays the 1.25x/2x write. The HD-2 byte-stable assert is the guard that makes this safe.

================================================================================
SUGGESTED SEQUENCING (by interaction-dependency, not just ROI)
================================================================================
Week 1: W0 (instrument) -> W1a (caching) -> W1b (tool search). Biggest measured step-change in cloud
  cost since launch; all three hit every request and W0 proves they fired.
Week 2: W2 (output stack) - the 5x-billed lever; FIM only on the base sidecar model.
Week 3: W3a (routing + NF4 Platt recalibration) -> W3b (masking, cache-safe) -> W3c (PACT handoffs).
Week 4: W4a (batch) + W4b (oracle persist + retrieval tightening).
Each item is a vertical slice (producer + its in-prod consumer + an off-agent smoke that proves the
SSB row landed), per the wiring-enforcement R1-R9 done-gate. cargo-green alone is not done.

================================================================================
WHAT THE RESEARCH GOT (slightly) WRONG FOR ELAI'S STACK
================================================================================
- The literature treats small models as "weaker". ELAI's architecture (fresh contexts + off-agent
  verification + deterministic gates) makes the NF4 7B a STRONGER editor than generic small-model
  papers assume. Route every file-local / verifier-checkable edit to base/FIM Qwen first; reserve
  cloud for planning + repeated verifier failure.
- Several headline numbers are vendor-internal (Cursor fast-apply 9x, Voyage reranker, Relace 47%).
  Use them as experiment prompts, not targets. Trust the arXiv / Aider / SWE-Edit numbers as targets.
- Abliterated-model + NF4 calibration for code is genuinely UNDER-STUDIED; ELAI's own pass/fail
  calibration set (from its verifiers) will matter more than any current paper. Build it in W3a.

Sources: workflow/research/token-efficiency/01-prompt-caching-context.md, 02-output-token-reduction.md,
03-multi-agent-context-cost.md, 04-model-routing-cascades.md, 05-retrieval-context-selection.md;
GROK-PROMPT.md (Grok-heavy manual reasoning); Opus synthesis agent (in-session). H-plan mapping:
docs/thefinalglance/H-plan/H-SUMMARY_INDEX.md Track 1 (H-1..H-9).
End of plan.
