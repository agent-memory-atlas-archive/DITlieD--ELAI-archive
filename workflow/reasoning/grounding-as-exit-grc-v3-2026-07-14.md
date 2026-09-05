# Grounding-as-exit — the GRC composition ruling (2026-07-14)

problem: the corpus holds every piece of a grounded reasoning loop, but the pieces were never bound
under one stated doctrine, and the harness CLAUDE.md still described the retired GRC v1 (mcp_think as
mandatory reasoning channel, per-topic call budgets, blockquote echoes). The user's 2026-07-14 direction
(sourced from an external FSM-reasoning writeup, fsm_reasoning_without_thinking_effort.md) named the
missing frame: reasoning depth should be governed by GROUNDING STATE, not by an effort level or call
budget — think cheaply first, keep working until the driving claims ground or you honestly abstain,
and only then act.

THE DOCTRINE (one paragraph)
A reasoning process exits into action when, and only when, the claims that the next mutation depends on
are grounded per the grounded-by-default ladder — or it exits into a credited abstention/escalation when
grounding is not reachable inside sane bounds. Between iterations the loop must add NEW EVIDENCE
(a retrieval, a grep, a test run, a search result, an executed probe), never another pass of rumination
over the same context. The cheap first pass is always allowed; escalation is earned by an ungrounded
load-bearing claim, not scheduled by task class. Abstention is a terminal with positive credit, not a
failure state. Grounding verdicts come from deterministic checkers, never from the model's own
self-claimed grounding field.

WHY THIS IS A COMPOSITION RULING, NOT A NEW UNIT (N-plan anti-fork)
Every mechanism already exists in spec; this note binds them under the frame and forbids authoring a
duplicate organ:
- J-83 GRC-v3 BeliefState carrier: per-step RECEIPT GATE — a step commits iff its routed receipt passes,
  else it abstains; only committed (grounded) entries become state. This IS "reason until grounded" at
  step grain.
- J-84 2-axis reasoning-effort classifier: deterministic cheap-first tiering (serial depth -> parallel
  width -> escalate-or-abstain) over H-42 serial-depth budgets + I-26 dynamic thinking budgets. This IS
  "a first always-allowed think, more only when needed".
- J-85 overthinking guard: when stuck, the next budget unit is more INFORMATION (retrieve/escalate/
  abstain), never more reasoning. This IS the new-evidence-per-iteration law.
- J-86 receipt-gated candidate ladder: receipt > vote over independent samples > never LLM self-eval.
- D-10 orchestrator-owned mode router (the LLM has no input to selection) + the constructive-checker
  registry: the grounding verdict source, off-agent by construction.
- D-16 conformal refusal + D-17 honest-refusal SLA + H-39 abstention-aware promotion (+1 / 0 / -t/(1-t)):
  the credited honest exit and its anti-gaming band.
- Phase-2 GRC plan-grounding sweep: the act-grain gate (no phase transition on an ungrounded claim).
- I-8 voluntary abort + dead-end hashing: termination guarantee against re-entering the same dead end.
- L-22/L-23/L-24: the explore lane where the loop runs on conjectures (falsifier-carrying candidates,
  criticism maps, computed status folds) without touching the proven-only paths.

FAILURE MODES THE COMPOSITION MUST HOLD AGAINST (from the 2026-07-14 spec-eval)
1. Non-termination: "until grounded" with no honest exit loops forever -> H-39 credited abstention +
   I-8 dead-end hashing are mandatory parts of the loop, not options.
2. Goodhart on the gate: self-claimed grounding decorations -> the exit verdict is only ever a
   deterministic checker's (D-10 registry; L-5 do-test for causal claims), fail-closed.
3. Typicality filter (creativity): a grounding exit fused with selection silently kills unusual ideas
   (the measured 13-27 pick-loss mechanism) -> axes never fuse (novelty-engine D4); grounding is a hard
   floor, novelty a separate axis, profiles select over survivors.

THE MEASURED BOUNDARY (what the evidence permits, 2026-07 creativity benches)
Grounding pressure never hurt creativity in any bench (NOVEL-CREDIBLE's grounded floor won novelty
22-14; the 3.5x anchor receipt is the line's only large win). What lost, three times, was structure
INSIDE generation (B-0 ProbeUtility, B-3 archive-conditioned rotation, §6 parallel context-blind
operator calls, Q-10 relational packets). Therefore: the loop's divergence step stays ONE free
self-coordinating call; grounding applies at COMMIT/EXIT; and the one untested link — feeding the
grounding-failure kill-reasons back into a single revision call (the loop-back) — is exactly P-25
FAMILY B (TRIAGE_REVISE vs SHUFFLED_REVISE), which FALSIFIES the creativity instantiation of this
doctrine instead of shipping it on faith. A FAMILY B result either way does not touch the doctrine's
verify/act legs, which stand on their own receipts.

HARNESS EDITION (applied 2026-07-14)
CLAUDE.md GRC paragraph rewritten to v2 reality: native thinking is the reasoning channel; mcp_think is
an opt-in decision ledger at real forks only; budgets/blockquote echoes retired; grounding-as-exit stated
as the mutation precondition. CSQ / Reasoning-After-Action / Anti-Spiral rewired from "call mcp_think to
reason" to "reason natively, ledger the fork".

CONSUMERS
- CLAUDE.md (harness edition, this date)
- docs/thefinalglance/P-plan/P-25 (FAMILY B = the creativity falsifier of the loop-back leg)
- the GRC-v3 wave files J-83..J-86 when they execute (this note is their stated doctrine line;
  execution order unchanged — this note authorizes no build)

Decision ledger: mcp_think topic 'grounded-reasoning-loop-design', 2026-07-14 (searchable via
mcp_search_context). Receipts: ELAI-SPEC-OVERVIEW.md (read in full 2026-07-14), J-SUMMARY_INDEX
1845-1917, D-SUMMARY_INDEX D-10/D-13/D-16/D-17, ALL-PLANS-MASTER-SUMMARY:182 (H-39..H-42),
novelty-engine-design-2026-07-12.md §6/§8/§10, research-coupled-creativity handoff (b0v3 REFUTED).
