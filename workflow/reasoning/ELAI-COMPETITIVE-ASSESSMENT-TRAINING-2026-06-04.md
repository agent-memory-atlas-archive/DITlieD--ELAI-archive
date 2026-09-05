ELAI COMPETITIVE ASSESSMENT — PHASE 1 (TRAINING-ONLY, UNGROUNDED)
Date: 2026-06-04
Author: Claude Opus 4.8 (1M)
Status: reasoning from training data + the read plan summaries ONLY. No web search yet.
        Every external-ecosystem claim below is "From training (unsearched)" unless it
        derives from a file I read this session. Phase 2 (separate file) grounds these.

WHAT WAS READ THIS SESSION (the evidence base for this file)
- phase-index-MASTER.txt (architecture spine, phases 0-4M)
- B/C/D/E/F/G/H plan summary indexes (the full roadmap, ~250 sessions of design)
These are design specs. Per the user's instruction I treat ELAI as if fully shipped.
In reality most of C/D/E/F/G/H is PLANNED, not shipped — see "Reality discount" at end.

================================================================================
1. WHAT ELAI ACTUALLY IS (so the comparison is fair)
================================================================================

ELAI is a Rust-native autonomous coding agent + harness, forked from a Rust port of a
Claude-Code-class CLI ("claw-code-rust-port"), then rebuilt into a 9-crate workspace with
a Tauri 2 / Svelte 5 desktop dashboard. Its defining shape is not "an IDE plugin" and not
"a chat wrapper" — it is an orchestration substrate that drives an LLM (local quantized OR
cloud) through a deterministic, gated, evidence-producing workflow.

Five pillars, in order of how differentiated they are:

(A) TRUST / VERIFICATION SPINE — the most distinctive thing.
    - SSB (Shared State Board): append-only SQLite WAL event journal; single-writer; every
      action_type typed; the system's source of truth. Determinism via ELAI_SEED +
      ClockProvider/EntropySource quarantine => byte-identical runs.
    - Wiring enforcement (F/G/H doctrine): fail-closed gates that measure an artifact the
      agent did NOT author (a runtime SSB row, an LLVM coverage hit, a span). "Compiles +
      unit tests pass" is explicitly rejected as a definition of done. Producer-without-
      consumer is a build break. The gate is frozen and read-only to the run it evaluates.
    - Reproducible-Run Artifact (G-2): a portable bundle of the action stream + seed +
      model pin + per-gate blake3 verdict hashes; `elai replay --verify` re-runs and
      reports PASS or first-divergence. In-toto signed attestation (G-8). Write-time
      secret/PII redaction (G-9).
    - Security-at-write-time: VibeGate (F-1) fans a file write across code-vuln + secret +
      hallucinated-dep + malicious-skill axes and BLOCKS fail-closed; CWE rulesets; a
      per-iteration security-monotonicity gate (F-10); taint propagation (G-1).
    - SAFEDELTA (G-6): a gate-ON-vs-gate-OFF ablation benchmark measuring residual insecure
      diffs reaching a human, on git-history-stripped held-out tasks, vs competitor harnesses.

(B) KNOWLEDGE ENGINE — genuinely deep retrieval.
    tree-sitter DKG (SQLite dep graph, 13+ languages) + BM25 (tantivy) + dual ONNX vectors
    (GraphCodeBERT 768d code + Granite 384d text) + PageRank repo-map (aider-derived) +
    CatRAG + A-RAG + cross-encoder rerank + HyDE + symbol-level impact analysis (BFS
    reverse-call, hop-banded) + entry-point detection + a SCIP-enriched code map. This is
    deeper than the codebase indexing most coding agents ship.

(C) REASONING ENGINE (GRC) — Grounded Reasoning Chain.
    A state machine forcing observe->hypothesize->plan->act->verify with mandatory grounding
    citations, per-topic budgets, per-agent configs, CSQ (counterfactual safety questions),
    6+ verification methods (FILE/SOURCE/TEST/WEB/API/REGISTRY/ARTIFACT/COSMOS/VISUAL).

(D) ORCHESTRATION / SELF-IMPROVEMENT.
    9-state FSM with HITL gating + autonomous mode; agent pool with checkpoint/crash-recovery/
    Erlang-style supervision/anti-spiral; BSP supersteps; rules engine (rules as DATA in
    SQLite, self-discovering, three-tier lifecycle); hierarchical plan management; session
    handoff DAG. DGM-class self-improvement (Darwin-Godel) but BOUNDED: typed reasoning-
    template archive, BAI Thompson-sampling promotion, 5 orthogonal tripwires, cross-domain
    canary, cryptographic archive, L3 frozen surfaces. Two-Gate guardrail against reward-hacking.

(E) DEPLOYMENT FLEXIBILITY.
    Per-role runtime routing across 11 backends (local sidecar, vLLM, SGLang, Ollama-deprecated,
    MLX, llama.cpp, cloud Anthropic/DeepSeek). Local lane = Qwen2.5-Coder-7B abliterated (NF4/AWQ);
    cloud lane = Sonnet 4.6 workhorse + Opus confirmation-gated. Hybrid escalation.

================================================================================
2. IS ELAI GOOD? (two-sided, anti-sycophantic)
================================================================================

THE CASE THAT IT IS GOOD (architecture + research rigor):
- The trust spine is, on paper, best-in-class. I cannot name a shipping coding agent that
  treats reproducibility, off-agent-artifact gating, and a security-delta benchmark as
  first-class. This is a real, defensible idea, not a feature-list bullet.
- Engineering discipline is unusually high: Rust, determinism proofs, anti-stale dependency
  policy, no-unwrap-in-prod, fail-closed-by-default, blake3 everywhere, single-writer event
  log. This is how you build something auditable, which is the whole pitch.
- The design is heavily research-grounded (hundreds of arxiv citations across D/E). The team
  clearly reads the literature and ports real findings (CaMeL, RTBAS, FIDES, DGM, GEPA,
  conformal refusal, Deflated-Sharpe for the trading track, etc.).
- The knowledge engine is deeper than most. Symbol-level impact analysis + PageRank repo-map
  + multi-ranker RRF + reranker is a serious retrieval stack.

THE CASE FOR SKEPTICISM (the things that actually decide whether it wins):
- It does not own a model. The quality lane is Anthropic's cloud Sonnet/Opus — the SAME brains
  Claude Code uses. So ELAI's coding *capability* ceiling is Claude's ceiling. It cannot
  out-reason Claude Code on a hard task, because it is Claude underneath. This is the single
  most important fact and it caps the entire "is it better at coding" question at "no, it's
  the same, possibly with more overhead."
- The local model is weak. Qwen2.5-Coder-7B-abliterated at NF4 is far below frontier. The
  local-first story is a cost/privacy/offline play, not a quality play. (Memory already
  records: "model is stale, need a real measured number before release.")
- Enormous wire-dark surface. The F/G/H plans exist precisely because ~40-45% of shipped work
  was built-but-not-wired-to-production. Much of the sophistication is latent, not active. A
  capability that never runs in production is not a capability.
- Solo project / bus factor. (Memory: "solo-rug-pull" named as a root contradiction.) Every
  competitor below has a funded team; ELAI's durability and support story is a real buyer risk.
- Complexity is itself a risk. ~250 planned sessions, 9 crates, an FSM, a rules engine, a self-
  improvement loop, a custom dashboard, a Python sidecar. The surface area to keep correct and
  the cognitive load to operate it are large. Sophistication that the user cannot feel is cost.
- "Consider it done" is doing heavy lifting. In reality B is mostly shipped, but C/D/E/F/G/H
  are largely PLANNED. The assessment of the *design* is strong; the assessment of the *product
  today* must be discounted hard.

VERDICT (training-only): As an ARCHITECTURE and a STATEMENT OF INTENT about trustworthy
autonomous coding, ELAI is genuinely good — arguably ahead of the field on the trust/verification
axis. As a PRODUCT competing on coding capability, it is structurally capped at parity with
Claude Code (because it is Claude underneath) and behind on maturity/UX/team. Its only honest
path to "better than X" is on axes Claude Code/Cursor/Windsurf do not contest: provable/auditable/
reproducible execution, write-time security, deterministic replay, and measured cost-per-task.

================================================================================
3. DOES ELAI HAVE COMPETITORS THAT DO THE SAME THING?
================================================================================

Decompose "the same thing" by axis, because no single competitor overlaps on all of ELAI:

AXIS 1 — Agentic coding harness on cloud models (THE crowded space).
  From training (unsearched): Claude Code, Cursor, Windsurf (Codeium), Aider, Cline (formerly
  Claude Dev), Continue.dev, GitHub Copilot (agent/workspace), OpenAI Codex CLI, Devin
  (Cognition), Replit Agent, Zed's agent, Roo Code, Amazon Q Developer, Google Jules/Gemini CLI.
  This is fully saturated. ELAI is one of many here, and not the most polished or known.

AXIS 2 — Local-first / offline coding agent.
  From training: Aider (supports local via Ollama/litellm), Continue.dev (local models),
  Tabby (self-hosted completion), Cline with local endpoints, Ollama-centric setups, Twinny,
  Tabnine (on-prem). ELAI overlaps but its local lane is weak and its differentiators are
  elsewhere. None of these combine local-first with ELAI's trust spine.

AXIS 3 — Deep code RAG / knowledge graph.
  From training: Cursor's codebase index, Sourcegraph Cody, Continue, Augment Code,
  claude-context (the thing E-22 benchmarks against), Aider's repo-map (ELAI's PageRank
  derives from it). Several do strong retrieval; none publish a symbol-graph + impact-analysis
  + reproducible retrieval trace the way ELAI's design does, but the *capability* overlaps.

AXIS 4 — Trust / verification / reproducibility / off-agent gating (ELAI's wedge).
  From training: I cannot name a mainstream coding-agent PRODUCT that ships this. The closest
  analogues are research/eval tooling, not in-the-loop product features:
    - SWE-bench / SWE-agent (academic eval harnesses, not a dev product)
    - AgentDojo (prompt-injection eval), not a shipping gate in a coding agent
    - in-toto / SLSA / Sigstore (supply-chain attestation — general, not coding-agent-native)
    - Semgrep/Snyk/Socket (security scanners — separate tools, post-hoc, not write-time fail-closed)
    - CodeGate (tried write-time security; per F-plan it is DEAD)
  So on the trust axis ELAI is close to first-in-category as a productized, integrated thing.
  This is the real moat candidate.

AXIS 5 — Self-improving agent (DGM-class).
  From training: Sakana AI's Darwin-Godel Machine is research; AlphaEvolve/FunSearch are
  research; no shipping coding product runs a guarded recursive self-improvement loop in
  production. ELAI productizing a bounded/guarded version is unusual but also the riskiest,
  least-proven, easiest-to-distrust part.

AXIS 6 — Cost-efficiency on the same cloud model (H-plan Track 1).
  From training: routers like RouteLLM, AutoMix, and "architect/editor" splits (Aider's
  architect mode is the canonical one) exist; prompt caching is an Anthropic platform feature
  every harness can use. So ELAI is NOT alone on cost — but combining caching + diff/FIM output
  + batch API + calibrated routing + a per-task cost cockpit, all measured and journaled, is a
  coherent, defensible package most harnesses assemble only partially.

CONCLUSION: There is no single "ELAI competitor." There are crowded competitors on axes 1-3
(where ELAI is a follower), and near-empty space on axis 4 (trust/reproducibility, where ELAI
leads) and axis 5 (self-improvement, where almost nobody ships). The strategy implication is
obvious and matches the existing market memory: do NOT headline axes 1-3; headline axis 4.

================================================================================
4. HOW GOOD IS ELAI vs CLAUDE CODE, CURSOR, WINDSURF? (scored, training-only)
================================================================================

Scoring 1-5 (5 best). "ELAI" = the fully-shipped design, cloud lane, as specified.

  Dimension                      | Claude Code | Cursor | Windsurf | ELAI (design)
  ------------------------------ | ----------- | ------ | -------- | -------------
  Raw coding capability          |     5       |   5    |    5     |   5 (== Claude; same model)
  Local/offline capability       |     1       |   2    |    2     |   3 (weak model, but real)
  Codebase retrieval depth       |     4       |   5    |    4     |   4
  UX / polish / IDE integration  |     4       |   5    |    5     |   2 (custom dashboard, solo)
  Ecosystem / community / support|     5       |   5    |    4     |   1 (solo, no community)
  Maturity / stability / shipped |     5       |   5    |    5     |   2 (mostly planned)
  Verification / trust / proof   |     2       |   2    |    2     |   5 (the wedge)
  Reproducibility / determinism  |     1       |   1    |    1     |   5 (unique)
  Write-time security            |     2       |   2    |    2     |   5 (unique)
  Cost transparency + efficiency |     3       |   3    |    3     |   4 (if H-Track1 ships)
  Autonomy / self-improvement    |     3       |   3    |    3     |   4 (bounded DGM, unproven)

HOW TO READ THE TABLE:
- On the axes buyers currently shop for (capability, UX, retrieval, maturity, ecosystem),
  Claude Code / Cursor / Windsurf win clearly. ELAI is a solo follower there and cannot lead.
- On the axes ELAI invented its identity around (trust, reproducibility, write-time security,
  cost-proof), ELAI is alone at 5 and the incumbents sit at ~2. This is the ENTIRE basis for a
  defensible position. If those axes don't matter to a buyer, ELAI has no story vs Claude Code.
- "Raw coding capability == Claude" is the trap: ELAI can match but never exceed Claude Code on
  the cloud lane, because it IS Claude. Any "smarter" claim must be about ORCHESTRATION
  (architect/editor split, verification catching errors Claude Code ships, fewer wasted tokens),
  never about the model.

WHO ELAI BEATS, HONESTLY:
- It beats Claude Code/Cursor/Windsurf for a buyer who must PROVE what the agent did
  (regulated, security-sensitive, agency-delivering-client-work, audit-required, air-gapped).
- It beats them on $/resolved-task IF H-Track-1 (caching/diff/batch/routing) ships and measures.
- It does not beat them for the median developer who wants fast, polished, capable autocomplete-
  to-agent in their IDE today. There it is strictly behind.

================================================================================
5. THE STRATEGIC CORE (what the comparison implies)
================================================================================

The capability ceiling is rented from Anthropic; therefore competing on capability is a loss.
The only winnable game is "same model, but you can TRUST and REPRODUCE and AUDIT and CHEAPLY-RUN
what it did." That is exactly the trust spine + H-Track-1 cost story. Everything else (deep RAG,
self-improvement, multi-backend) is supporting cast, not the headline.

Two root contradictions remain unsolved by architecture alone (both already in market memory):
  (1) Anthropic-dependency: the product's quality is a competitor's API. Mitigation is the local
      lane + multi-backend, but the local model is too weak today to be the quality story.
  (2) Solo / bus-factor: an auditable-trust product whose vendor might vanish is a hard enterprise
      sell. Mitigation candidates: open-source the VERIFIER only (keep engine closed), escrow.

================================================================================
6. REALITY DISCOUNT (do not let "consider it done" hide this)
================================================================================
- B-plan: largely SHIPPED. The MVP wiring, dashboard V2, multi-runtime, idea-framing, cosmos,
  per-level audits exist.
- C/D/E: largely PLANNED. The verification backfill, self-improvement substrate, and the 55-
  session research sweep are designs, mostly unbuilt.
- F/G: the trust gates — partly built (F-1/F-2a/F-4 etc. shipped per memory), much PLANNED,
  several RUNPOD-PENDING (no measured numbers yet).
- H: authored 2026-06-03/04, entirely PLANNED.
- The single most load-bearing missing thing is A MEASURED NUMBER: SAFEDELTA / insecure-rate /
  $-per-task / DSR — the plans repeatedly defer the headline until a real measurement exists.
  Until then, every "better than X" claim is ungrounded conjecture.

================================================================================
7. OPEN QUESTIONS TO GROUND IN PHASE 2 (web search)
================================================================================
Q1. 2026 competitive landscape: are Claude Code / Cursor / Windsurf still the frontier set, and
    has any of them added reproducibility / write-time security / off-agent verification since
    my training cutoff? (If a competitor shipped ELAI's wedge, the moat is gone.)
Q2. Does any shipping coding agent now do deterministic replay / reproducible-run artifacts /
    in-toto attestation of agent actions?
Q3. Is anyone productizing guarded recursive self-improvement (DGM-class) in a coding tool?
Q4. Write-time/in-loop security gating: state of CodeGate (dead?), Semgrep-in-agent, Socket,
    and any new "secure-by-construction codegen" products in 2026.
Q5. Is the "architect/editor split + caching + batch" cost story still novel, or now table-stakes
    (e.g., did Aider/Cursor/Claude Code ship equivalent cost tooling)?
Q6. Reality check on the local model: is Qwen2.5-Coder-7B still a reasonable local-lane choice in
    mid-2026, or has the local-coding-model frontier moved (Qwen3-Coder, DeepSeek-Coder-V3, etc.)?
Q7. EU AI Act / provenance (in-toto, SLSA, C2PA-for-code) status in 2026 — does the attestation
    angle have a real regulatory tailwind or is it speculative?

End of Phase 1 (training-only) file.
