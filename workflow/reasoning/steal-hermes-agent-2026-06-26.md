HERMES-AGENT (NousResearch) vs ELAI — DOES ELAI LACK ANYTHING IT HAS?
2026-06-26. Mode: spec-eval AS-SPECIFIED (ELAI planned/half-wired counted as DONE for
the design verdict; build-status is a separate readiness note). Grounded: hermes claims
from the code-grounded map at docs/.source/hermes-agent-INDEX.md (clone commit e3db1ef,
MIT, v0.17.0, ~28K Python LOC, active); ELAI claims from docs/thefinalglance/ELAI-FEATURE-MAP.md.

ONE-LINE FRAMING
hermes-agent and ELAI are different SHAPES, not competitors on one axis. hermes = BROAD,
pragmatic, ships-everything personal-agent runtime with a self-improving skill loop and
ADVISORY safety. ELAI = NARROW, deep, trust-first dev harness with FAIL-CLOSED gates,
where the equivalent capabilities are designed stronger but not all shipped. So the
honest question splits in two: (1) where is hermes's BREADTH a real ELAI gap, and (2)
where hermes overlaps ELAI, who has the better DESIGN.

================================================================================
A. FEATURE-BY-FEATURE VERDICTS (hermes capability -> ELAI status)
================================================================================

SELF-IMPROVING SKILL LOOP (background review forks aux model post-turn to propose
SKILL.md edits; Curator daemon lifecycle; closed learning loop) [hermes: SHIPPED]
-> ELAI ALREADY-PLANNED, AND ELAI's DESIGN IS STRONGER. ELAI's I-12 evidence-gated
playbook curator (promote on >=k SSB-VERIFIED outcomes, export to CLAUDE.md/AGENTS.md/
skills), J-10 insight-backprop tree memory, J-96/J-97 verified-case-graph episodic->
skill->rule ladder, all bound by K-19 self-improvement-fitness doctrine (execution-
grounded fitness only). Crucially: hermes's loop is ASYNC and UNVERIFIED ("unverified by
a gate", per its own limitations) — that is EXACTLY the DGM-without-a-real-filter pattern
ELAI proved provably collapses (H-43..H-48 / K-19). So this is NOT a steal; it is ELAI's
thesis validated by a live system that ships the ungated version. READINESS NOTE: hermes
ships it today; ELAI's gated version is planned/partial.

CROSS-SESSION RECALL (FTS5 keyword search over past session messages, exposed as a
mid-task `session_search` tool) [hermes: SHIPPED] -> ELAI ALREADY-PLANNED, STRONGER.
BM25 (tantivy) >> FTS5; plus OracleAnswerCache, A-MEM/MemGPT infinite-context (E-1),
H-51 handoff digest, and J-97 gated case-graph reuse (the mid-task "recall a proven past
solution" tool). Covered and deeper. No steal.

MULTI-MODEL FAN-OUT (MoA: /moa fans out to N reference models, aggregates) [hermes:
SHIPPED] -> ELAI ALREADY-SHIPPED, STRONGER. B-39 chat-reflect (TRIAGE->REASONER N=4->
AUDITOR REFINE->SYNTHESIZE) + D-13 cross-FAMILY adversary (different pretraining family,
the diversity hermes's same-pool reference models lack) + E-6 MAST. No steal.

PROVIDER/TRANSPORT BREADTH (200+ providers via OpenRouter; 4 transports chat_completions/
anthropic_messages/bedrock_converse/codex_responses) [hermes: SHIPPED] -> ELAI COVERED.
B-37 11-backend multi-runtime + J-30 BYOK + J-54 free-pool rotation + bedrock/azure
direct lanes + J-64 vendor-CLI bridges. OpenRouter DYNAMIC registry explicitly
ALREADY-REJECTED 3x (I-51 static models.toml). No steal.

VERIFICATION-EVIDENCE LEDGER (2nd SQLite DB tracks which terminal cmds count as post-edit
verification; nudge if model stops without one) [hermes: SHIPPED, advisory] -> ELAI
MASSIVELY AHEAD. This is the seed of ELAI's whole spine, done as a soft nudge. ELAI =
fail-closed wiring-enforcement + off-agent auditor + DAIRA evidence gates + SSB evidence
rows + production_caller_check + R10 integration smoke. hermes "verify-on-stop is advisory
only, does not auto-run tests"; ELAI's is the load-bearing gate. No steal.

INJECTION DEFENSE (3-scope regex threat library; WARNS on tool-result injection, BLOCKS on
memory/skill writes; invisible-unicode + C2-Brainworm signatures) [hermes: SHIPPED] ->
ELAI AHEAD (structural vs signature). G-1 taint-flow + E-37 perimeter + J-101
transformation-surviving taint binding + J-102 zero-width/homoglyph frontmatter scan +
K-13 OWASP mapping. hermes "does NOT block tool result delivery — warns only" = weaker.
ONE cheap rider worth noting: hermes ships a CURATED known-attack-SIGNATURE set (C2
Brainworm patterns); ELAI's defense is structural (taint), which can be paraphrased past a
signature but also catches novel attacks. A curated signature set as a CHEAP added layer
under the structural one is a LOW-value optional rider, not a gap.

SUBAGENT DELEGATION (`delegate_task`: isolated child agents, no parent history, blocked-
tool list) [hermes: SHIPPED] -> ELAI AHEAD. orchestrate/auto-orchestrate + Tier-C + agent_
pool + scoped context + dispatch-preamble scope contract + J-80 worktree isolation + FSM
evidence gates + handoff mediation. No steal.

CONTEXT MANAGEMENT (compress at ~75%, session-split chain, pluggable ContextEngine ABC)
[hermes: SHIPPED] -> ELAI's BUILT-IN IS STRONGER; ELAI LACKS THE PLUGGABILITY. ELAI:
context_policy regime-aware (cloud vs local opposite policies), 3E 8-slot allocator, J-34
measured-knee calibration, H-49/H-50/H-51, compaction ladder. The ONE thing hermes has
that ELAI does not: a third-party ContextEngine PLUGIN ABC. For ELAI (closed coherent
trust harness) plugin extensibility is off-doctrine and low value — note, not adopt.

LAYERED SYSTEM PROMPT (SOUL.md/USER.md/MEMORY.md/AGENTS.md tiers: stable/context/volatile)
[hermes: SHIPPED] -> ELAI COVERED, STRONGER. 3E typed slots + cache-stable prefix + TZ-1
volatile-after-cache-breakpoint + measured slot ordering (J-34) + user memory E-5/H-30..33.

TRAJECTORY COMPRESSOR (standalone JSONL pipeline: agent runs -> fine-tuning training data)
[hermes: SHIPPED] -> ELAI ALREADY-PLANNED. I-30 recovery-trajectory SFT exhaust (J-50
extends), C-49 experience distillation, GRPO flywheel, SFT-before-GRPO. GPU-deferred.
hermes ships a concrete tool for it (maturity note). No design steal.

CRONJOB TOOL (scheduled agent runs) [hermes: SHIPPED] -> ELAI ALREADY-REJECTED (R-5, I-53
declined; J-100 watchdog is step/progress-based, not a scheduler). Deliberate decline.

================================================================================
B. WHERE ELAI GENUINELY LACKS SOMETHING (the honest list, ranked)
================================================================================

GAP-1 [MEDIUM, the one with real technical merit] PLUGGABLE REMOTE / SANDBOXED EXECUTION
BACKENDS for the shell tool. hermes runs its terminal tool across SIX backends:
local/Docker/SSH/Singularity/Modal/Daytona. ELAI's execution is LOCAL Tier-C process
isolation only (plus remote sidecar = inference, RunPod = GPU — neither is a shell-exec
sandbox). The capability ELAI lacks: route an agent's CODE EXECUTION into a containerized
or remote ephemeral environment. WHY IT MATTERS FOR ELAI: safe execution of UNTRUSTED
generated code (composes directly with J-63 local-claudeception + Tier-C + the RunPod
pod-lifecycle already wired) — "run the app ELAI just wrote in a throwaway Docker/Daytona
box, not on the user's host." This is a real EXTENSION shaped like B-37/RuntimeOrchestrator
but for the TERMINAL tool instead of inference: an ExecutionBackend router behind the shell
tool {Local(Tier-C) | Docker | Remote-pod}. Honest caveat: it leans away from ELAI's local-
first posture, so it is opt-in, and the sound version reuses Tier-C's seccomp/cap-drop as
the local rung. NOT planned anywhere in B..K — genuine whitespace.

GAP-2 [LOW-MEDIUM, cheap robustness] STREAM-INTERRUPTION DISCRIMINATION. hermes distinguishes
a NETWORK break from a LENGTH truncation on the model stream (PARTIAL_STREAM_STUB_ID
sentinel) and applies a different retry/continuation strategy per case. ELAI has circuit
breakers + retry + DEGRADED modes but (per the feature map) no evidence of this specific
truncation-vs-disconnect discrimination + per-case continuation prompt. Cheap, robustness-
only, fits ELAI's reliability bar. A small rider on the runtime_client streaming path, not
a track.

GAP-3 [LOW-MEDIUM, scoped to one ELAI surface] COMPUTER-USE + BROWSER-AUTOMATION TOOL SUITE.
hermes ships 10 browser tools + computer_use + vision + image-gen + TTS. ELAI has VLM
perception (J-1/J-8) and the canvas app-preview proxy but no general browser-automation or
computer-use agent tools. Relevant ONLY to the canvas APP/WEBSITE design mode (H-14, where
driving a real browser to verify a generated UI would compose) — elsewhere it is off-
positioning. Note for H-14, not a general adoption.

GAP-4 [LOW / off-positioning] MULTI-CHANNEL DELIVERY GATEWAYS (Telegram/Discord/Slack/
WhatsApp/Signal/Matrix). ELAI ships CLI + Tauri dashboard only. This is a personal-
assistant product surface that does not fit ELAI's local dev-trust-harness wedge. Real
absence, but DECLINE by positioning, not a deficiency. (If ELAI ever wants a remote
"notify/respond from my phone" surface, the H-36 notification seam is where it would
attach — one channel, not six.)

================================================================================
C. THE READINESS ASYMMETRY (separate from the design verdict)
================================================================================
On the OVERLAP set (self-improvement loop, cross-session recall, multi-model fan-out,
context management, trajectory->training), ELAI's designs are stronger (gated, grounded,
execution-verified) but hermes SHIPS them today and ELAI's are planned/partial. That is a
maturity gap, not a design gap — but it is real and worth stating plainly: hermes is a
working demonstration that the BROAD version of these features is buildable and useful now,
while ELAI's bet is that the GATED version is worth the extra build. The one place hermes's
shipped behaviour actively WARNS against ELAI's plan being over-built is nowhere — its own
limitations (unverified skill loop, advisory verification, no mandatory sandbox) are
exactly the corners ELAI's trust spine refuses to cut.

================================================================================
D. BOTTOM LINE
================================================================================
Does ELAI lack anything hermes-agent has? Yes, four things, only ONE of which is a real
design-merit gap rather than a positioning choice:
  - GAP-1 pluggable remote/sandboxed EXECUTION backends for the shell tool [MEDIUM, adopt
    as an EXTENSION — it composes cleanly with Tier-C + J-63 + RunPod and fills a genuine
    safe-untrusted-execution hole].
  - GAP-2 stream-interruption discrimination [LOW-MEDIUM, cheap robustness rider].
  - GAP-3 browser/computer-use tools [LOW-MEDIUM, only for canvas H-14].
  - GAP-4 multi-channel gateways + cronjob [DECLINE — off-positioning / already-rejected].
Everything else hermes has, ELAI either ships, plans more strongly, or deliberately
rejected. The dominant finding is the inverse of the question: hermes's self-improvement
loop, verification nudge, and injection defense are the UNGATED versions of ELAI's spine —
hermes is, in effect, a live argument FOR ELAI's gated designs, not a source of missing
capability. The only thing worth actually building from this read is GAP-1.
