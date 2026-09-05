spec-eval: is ELAI already better than Hermes Agent and Kilo Code?
date: 2026-07-10
mode: AS-SPECIFIED (spec-eval skill). ELAI judged on its full spec (docs/thefinalglance/ELAI-SPEC-OVERVIEW.md, Phases 0-4M + letters B..P). Comparators judged on their source-grounded overviews: HERMES-SPEC-OVERVIEW.md (NousResearch hermes-agent, HEAD 111544d, cloned 2026-07-10) and KILOCODE-SPEC-OVERVIEW.md (Kilo-Org/kilocode, HEAD 1426dca, cloned 2026-07-10). Shipped-vs-planned never feeds the verdict; a readiness note sits at the end, clearly separated.

VERDICT (headline)

"Better" is not scalar across these three; they are different products sharing one doctrine (model = swappable cog, capability at the deterministic/authored edges). Axis by axis:

- On ELAI's declared wedge, trust-evidenced autonomous execution, ELAI's design is categorically ahead of both. Not incrementally: neither comparator has ANY member of the class {off-agent evidence, fail-closed gates, frozen evaluator, byte-replayable runs}. This is a design-class gap, not a feature-count gap.
- On reach as a personal agent (channels, plugins, learn-from-experience liberality), Hermes is the better design. ELAI's spec does not attempt a 20-platform gateway and its O/P letters arrive at the personal-agent grain late and gated.
- On distribution topology and upstream leverage, Kilo is the better design. One `kilo serve` engine + SDK-generated thin clients (VS Code/JetBrains/Zed/web), and a fork-merge discipline that keeps absorbing upstream opencode improvements. ELAI's Phase-0 fork diverged permanently and receives nothing from its upstream.

So: is ELAI already better than Hermes/Kilo? As a coding harness judged on verification, reproducibility, retrieval depth, and determinism, yes, by design and by a wide margin. As a personal-assistant reach product, no (Hermes). As a distribution/ecosystem topology, no (Kilo). The wedge axis is the one ELAI chose to compete on, and on that axis neither comparator is even attempting the same property class.

AXIS 1 — VERIFICATION / TRUST (ELAI's wedge) — ELAI wins, design-class gap

ELAI (spec): off-agent evidence (gates measure artifacts the generator did not author: event rows, coverage hits, hash-bound test outcomes), fail-closed (missing evidence = FAIL, gate frozen and read-only to the run it judges), F-letter write-gates (vuln/hallucinated-dep/secret/malicious-skill blocking pre-disk), test-integrity hash binding (a fabricated PASS is detectable), G-letter taint propagation (untrusted-content -> destructive-sink blocked deterministically), RRA (single portable artifact: action stream + seed + hardware pin + gate-verdict hashes, `elai replay --verify` re-derives it; G-SUMMARY_INDEX.md:38-55), SAFEDELTA gate-ON/OFF ablation vs competitor harnesses (G-6, its own PASS hash-bound via F-4), I-letter tri-layer anti-gaming stack (hard veto -> calibrated scorer -> adversary) and evidence-ranked search (selection by receipts, never an LLM judge).

Hermes (source-grounded): `verification_evidence.py` + `verification_stop.py` collect evidence and can gate a turn's completion; guardrails/file_safety/secret_scope/redact annotate or gate tool calls. Its own overview states the divergence: "Hermes's is evidence-collecting, not frozen-evaluator." Nothing fail-closed, no replay contract anywhere (its cron explicitly "has no journal/replay contract"), the agent's own loop decides done.

Kilo (source-grounded): permission gates on tool calls (user consent), a Review agent (prompt-shaped, surfaces issues), LSP as edit-verification backend. Developer-in-the-loop, advisory. Modes (Code/Plan/Ask/Debug/Review) are prompt-shaped personas; ELAI's FSM states carry evidence gates on transitions.

Judgment: a Review agent and a soft verification-stop are both in-boundary — the generating system evaluates itself. ELAI's whole spine is built on the measured failure of exactly that (in-boundary artifacts gamed; wiring-enforcement research). Neither comparator can make the claim ELAI's design makes: "this run's claims are checkable by an artifact the model did not author, and the whole run re-derives byte-identically." That claim is the product. Design-class win, ELAI.

AXIS 2 — DETERMINISM / REPRODUCIBILITY — ELAI wins, uncontested

ELAI: entropy quarantine from Phase 0 (all time/randomness through injected ClockProvider/EntropySource, seeded runs byte-identical), SSB append-only journal, DST (simulation testing against an infallible reference), RRA replay, journal-completeness substrate (J: prove every side-effect produced a row), SSB hash chain + formal verification of the critical core (Kani/TLA+).
Hermes: synchronous loop, SQLite session store, no determinism contract at all.
Kilo: session persistence/export/portability, snapshots, git worktrees — state capture, not reproducibility; no seeded-replay concept.
Neither comparator attempts this property. Uncontested.

AXIS 3 — RETRIEVAL / CODE INTELLIGENCE — ELAI wins on depth

ELAI: Phase-1 layered cascade with per-layer fallback — tree-sitter DKG (typed symbol graph) -> tantivy BM25 -> dual-model ONNX vectors with circuit breakers -> PageRank token-budgeted repo map -> CatRAG/A-RAG agentic assembly -> cross-encoder reranker, plus 3K hardening (AST-aware chunking, HyDE, tier routing) and B-letter symbol-level impact analysis / 13+ language DKG.
Kilo: LanceDB vector store + tree-sitter parse + embeddings, worker-based indexing, plus LSP. A solid single-layer semantic index; no graded degradation, no lexical layer, no importance ranking named.
Hermes: SQLite FTS5 over session history + repo-context injection (AGENTS.md-aware working dir). Not a code-retrieval engine.
ELAI's design is the deepest and, notably, degrades gracefully (vector failure -> BM25-only) where Kilo's is a single modality. Win, ELAI.

AXIS 4 — LEARNING / MEMORY — split; opposite bets, both defensible; ELAI's is safer, Hermes's is faster

Hermes: the differentiator by intent. Skills authored from experience ("solved a hard problem -> write a reusable skill"), a curator daemon GC-ing agent-authored skills by usage decay (never deletes, pin-exempt, snapshot-backed), pluggable durable memory (8 backends behind a closed ABC). Liberal minting, cheap accumulation, decay as quality control.
ELAI: verified case graph (J) — a case node is minted ONLY on a proven fix, reuse sits behind an execution-grounded filter (default-off); two-tier user memory (H) with bi-temporal preferences + code-grounded validate-before-use facts + an auditor-objectivity firewall; K-18 measures whether case-reuse actually compounds (recurrence hit-rate bench).
Judgment: Hermes accumulates capability faster and cheaper but its quality control is usage-decay, not correctness — a wrong-but-frequently-used skill survives. ELAI accumulates slower but every reused artifact carries proof, and K-18 exists to measure whether the compounding is real rather than asserted. For a coding agent whose output ships, ELAI's bet is the right one; for a personal assistant where a stale skill costs little, Hermes's is. On-wedge: ELAI. Off-wedge: Hermes.

AXIS 5 — REACH / SURFACES / ECOSYSTEM — Hermes wins

Hermes: one core across CLI, TUI, Electron desktop, and a ~20-platform messaging gateway (telegram/discord/slack/whatsapp/signal/matrix/email/sms/...); plugin surfaces with 5 discovery systems; cron scheduled autonomy with multi-platform delivery; ACP for IDEs; MCP client + shipped servers; a skills marketplace posture (agentskills.io-compatible).
Kilo: VS Code (flagship), JetBrains, Zed, web console, CLI/TUI — all thin clients of one server.
ELAI: Tauri dashboard + CLI + canvas-workbench + ELAI-as-MCP-server control plane + voice front door (J) + O/P personal-research/personal-world grains. Deliberate, but nowhere near Hermes's channel breadth, and the spec does not aim there.
Honest call: Hermes's breadth-first edge expansion is a genuinely better design FOR ITS GOAL (a personal agent that meets you where you are). ELAI's counter is that every ELAI surface routes through the same journaled/gated spine (voice replays like typed chat; canvas ops are journaled + taint-tagged) — depth per surface vs count of surfaces. But on the axis itself, Hermes wins.

AXIS 6 — TOPOLOGY / DISTRIBUTION / UPSTREAM LEVERAGE — Kilo wins

Kilo: one-backend/many-thin-clients over HTTP+SSE with an SDK auto-generated from server endpoints — a capability added to the core surfaces across all products with zero per-client reimplementation. And the fork-merge discipline (`kilocode_change` markers, CI-enforced quarantine) means Kilo continuously absorbs upstream opencode improvements while keeping its diff small.
ELAI: one-frontend/many-backends (4M RuntimeAdapter: sidecar, cloud, vendored CLIs) — the mirror image, and the right shape for ELAI's goal (the harness is the product, the engine is swappable). But ELAI's Phase-0 fork diverged permanently: it gets zero ongoing leverage from its upstream, and carries the whole maintenance surface alone. Kilo's fork hygiene is a directly better mechanism and is correctly flagged in the Kilo overview as a steal target for ELAI.
M-letter (signed offline entitlement, org policy packs, open evidence standard) is a distribution design Kilo lacks entirely, and zero-telemetry local rollups vs Kilo's PostHog+OTel is a real differentiator for the security-sensitive buyer. But on plain developer-distribution topology today: Kilo's design is cleaner. Win, Kilo.

AXIS 7 — EFFICIENCY / COST — ELAI's design is stronger; Hermes's is simpler

Hermes: prompt-cache preservation as a sacred invariant shapes everything (strict role alternation, skills injected as user messages, compression as the only sanctioned cache-breaker) — an elegant, cheap, load-bearing discipline. Honest per-model pricing/credits tracking. Kilo: 500+ models zero-markup gateway, FIM/autocomplete lane.
ELAI: the same invariant made STRUCTURAL rather than disciplinary — L-T3 context-core inversion (immutable transcript + pure fold-view projection makes cache-stability a property of the data model, not of everyone's good behavior; L-SUMMARY_INDEX L-T3, measured ~89-93% provider cache-read in the studied prior art), plus H-letter output-token stack (diff/FIM + streaming early-abort), Batch API lanes, observation masking, ledger-trained routing on verified outcomes (I), J token ledger + BYOK + free-API rotation, K-2 $/verified-task bench. ELAI also invented Hermes's invariant independently and then went one level deeper (structural vs disciplinary). Design win, ELAI — with the caveat that Hermes's version costs ~nothing to build and ELAI's is a whole subsystem.

AXIS 8 — SANDBOXING / ISOLATION — split

Kilo: bubblewrap (Linux) + seatbelt (macOS) kernel-level FS/network confinement — concrete, OS-enforced, the strongest single isolation mechanism named across all three specs. Hermes: pluggable terminal environments (docker/ssh/modal/daytona/singularity) — delegation to external isolators. ELAI: 4A process isolation where agents hold NO network/LLM/tools (everything proxied over a bounded 15-message IPC) + COW workspace isolation (I) + Firecracker exploration (E).
ELAI's 4A is the deeper ARCHITECTURAL cut (the agent structurally cannot reach gate code or the network — capability removal, not confinement), but Kilo's kernel-level confinement is the stronger enforcement mechanism for what an agent CAN reach. Ideal design = both; each spec holds half. Split.

WHERE THE COMPARATORS' DESIGNS ARE GENUINELY BETTER (anti-sycophancy ledger)

1. Kilo's fork-merge hygiene (`kilocode_change` + CI guards + upstream absorption) — strictly better than ELAI's diverged fork on maintenance economics. Steal candidate.
2. Hermes's Footprint Ladder as an ADMISSION mechanism — ELAI's N-letter anti-fork rule is the same idea but arrived AFTER fragmentation happened (N exists because four claim types / ten routers / six intensity deciders had already forked). Hermes had the admission rule from the start; prevention beats N's cure. ELAI's composition validator (N, runs at plan-authoring time) is the stronger mechanized version — but it was needed because the discipline wasn't structural earlier.
3. Kilo's SDK-generated-from-server client model — eliminates per-client drift by construction. ELAI's dashboard/CLI/canvas have no equivalent single-source client contract in the spec.
4. Hermes's derive-everything-from-one-COMMAND_REGISTRY (CLI dispatch, gateway menus, autocomplete, help all derived) — same "one source of truth, many projections" move ELAI makes for evidence; ELAI's spec doesn't state it for its command surface.
5. Simplicity as a feature: both comparators fit in a developer's head. ELAI's B..P corpus is enormous; its own K-4 gate-tax instrument exists because the trust spine has a real, measurable friction cost the comparators simply don't pay. If the gate-tax ceilings can't be met, the wedge's affordability claim weakens — this is ELAI's largest design risk, and it is correctly instrumented rather than denied.

DESIGN WEAKNESSES IN THE COMPARATORS (so the ledger cuts both ways)

- Hermes: no journal/replay contract anywhere; verification is in-boundary; skill minting has no correctness gate (decay ≠ proof); breadth multiplies attack surface with only a guardrail layer (its dep-pinning policy exists because it was bitten twice — litellm, Mini Shai-Hulud).
- Kilo: centralized telemetry (PostHog) is a hard disqualifier for the security-sensitive segment ELAI targets; verification is entirely developer-attention-bound (the human is the gate, which does not scale to autonomy); no determinism story means a Kilo run is unauditable after the fact.
- Neither can serve the buyer who asks "prove this agent didn't fabricate its test results" — that buyer is ELAI's whole thesis.

VERDICT RESTATED

ELAI as specified is already the better DESIGN on the axes it chose: verification/trust (design-class gap — the property set {off-agent, fail-closed, frozen, replayable} has zero members in either comparator), determinism (uncontested), retrieval depth, cost architecture, and safe-compounding memory. Hermes is the better design for personal-agent reach; Kilo is the better design for distribution topology and upstream leverage; both are better on held-in-one-head simplicity. These are different races. On the shared ground where all three ARE the same product (an agentic coding harness), ELAI's spec is the strongest of the three — provided the K-4 gate-tax and K-2 $/verified-task numbers land inside their ceilings, because affordability is the one axis where the trust spine could lose to the comparators' zero-overhead posture.

IMPLEMENTATION-READINESS NOTE (sequencing facts, NOT part of the verdict; index-derived, confirm against live code)

Hermes and Kilo are shipped, used products today (Kilo: 10k+ stars, flagship VS Code extension; Hermes: ~2,600 py files, ~17k tests). ELAI's comparative claims above describe the spec; per the indexes, large parts are shipped (Phases 0-3 substrate, wiring/gate stack, RRA/attestation per I-plan notes, token ledger J-28, gate-tax runner K-4 with ceilings pending a real journal) and others are at plan (much of L/N/O/P, SAFEDELTA number not yet landed). The trust-spine proof numbers (PS-1..PS-6) are the bridge from "better by design" to "better, measured" — that is a timing gap, not a design gap.

grounding: ELAI-SPEC-OVERVIEW.md (whole); HERMES-SPEC-OVERVIEW.md + KILOCODE-SPEC-OVERVIEW.md (both source-grounded, HEAD-pinned 2026-07-10); G-SUMMARY_INDEX.md:38-123 (RRA/SAFEDELTA); L-SUMMARY_INDEX.md L-T3 (context-core, ~89-93% cache-read prior art); decision recorded to mcp_think topic spec-eval-hermes-kilo.

--------------------------------------------------------------------------------

ADDENDUM 2026-07-10 (same session) — user follow-up: "could ELAI lose by being expensive?" and "I want ELAI best overall, should we improve the losing axes?"

CLARIFICATION 1 — the "expensive" risk is gate friction, not dollar price.

Every ELAI task pays a tax the comparators do not: the tokens + wall-clock of gates, audits, journaling, and verification stacked on the actual work. Hermes and Kilo pay zero gate-tax and deliver zero verification. The risk is not "ELAI's API calls cost more"; it is that if the trust-spine overhead per task is too heavy, users feel it as slowness/cost and churn to a zero-overhead harness even though nothing there is verified. This risk is named and instrumented in ELAI's own spec: K-4 measures the composed gate-tax as two labeled numbers (coding_task_gate_tax + audit_gate_tax) each bound to a locked ceiling with a fail-closed breach exit code (shipped at commit d1eb94a, ceilings pending a real elai-run journal); K-2 measures $/verified-task, the honest comparison metric, where ELAI's design intends to WIN because a competitor's "cheap" task that ships an unverified bug is not cheap. The mitigation also already exists in spec: K's risk-tiered gating policy + P's graduated ceremony (low-risk work runs gates observe-only — the spine scales DOWN). What is missing is only the measured number proving the tax fits the ceiling. That is PS-2/PS-4, already top of the proof-spine order. Conclusion: cost is the ONE axis where ELAI can actually lose to the comparators, the spec knows it, and the fix is measurement work already sequenced first — not new design.

CLARIFICATION 2 — "best overall" means closing some axes and deliberately NOT closing others.

The losing axes split into two classes; copying the competitor is only right for one of them.

Worth closing (cheap, on-doctrine):
- Fork hygiene (steal Kilo's kilocode_change markers + CI guards). ELAI's Phase-0 fork diverged irreversibly — nothing to retrofit there. But 4M vendors coding-agent CLIs with live, fast-moving upstreams; applying Kilo's marker-quarantine discipline to those vendored runtimes captures the maintenance-economics win nearly free.
- One-registry surface contract (Kilo's SDK-generated clients / Hermes's derive-everything-from-COMMAND_REGISTRY). ELAI is most of the way there: ts-rs bindings with a drift gate, and the J-letter ELAI-as-MCP-server control plane is already the one-server/thin-clients shape. Formalizing "every surface derives from one registry" is an N-letter-style consolidation, not new build.
- The cost axis itself: land PS-2 (K-4 gate-tax vs ceiling) + PS-4 (K-2 $/verified-task). This is the entire "don't lose on cost" work and is already sequenced first.

Deliberately NOT closing (opposite bets that ARE the product):
- Hermes's ~20-platform messaging gateway: a huge permanent maintenance surface, feeds no PS number, and the control plane already makes any future channel a thin adapter over the same journaled spine (a Telegram adapter later is small; a gateway track now is off-wedge).
- Hermes's liberal skill-minting and Kilo's centralized telemetry (PostHog): ELAI inverted both on purpose — correctness-gated case reuse instead of usage-decay skills, zero-telemetry local rollups (M) instead of centralized analytics. These inversions are load-bearing for the security-sensitive buyer.

Sequencing caveat (proof-spine PSR-2): the two "worth closing" steals feed no PS number directly — they are spine-exempt discipline/distribution units requiring explicit user ack before authoring. Recommended order regardless: PS-2/PS-4 first (kills the only losable axis), then the two cheap steals.

Decisions recorded to mcp_think topic spec-eval-hermes-kilo (2 entries).

OUTCOME 2026-07-10b (paper-to-plan Phase 5, main-LLM in-band): the ADDENDUM's two "worth closing" distribution steals authored into NEW LETTER Q as Q-9 (distribution hygiene, Track DISTRIBUTION, two units in one file). UNIT A = Kilo fork-hygiene (elai_change markers + CI quarantine) applied to the 4M VENDORED CLIs — NOT the diverged Phase-0 fork (the addendum is explicit that fork is unretrofittable). UNIT B = the one-registry surface contract (Kilo SDK-gen / Hermes COMMAND_REGISTRY) formalized by extending the ts-rs/cross_lang_diff drift gate to command-surface symmetry. Both PROOF_SPINE: none (distribution), owner-delegated PSR-2 ack 2026-07-10; sequenced AFTER the proof spine per the addendum order (PS-2/PS-4 first). The cost-axis work the addendum ranks first (PS-2 K-4 gate-tax + PS-4 K-2 $/verified-task) is NOT re-authored here — it is existing sequenced spine work; Q-3's advisor arm + Q-2's wake-gate + Q-7's weak-lane bench are the Q units that FEED PS-4. Q-9 PASSES plan_gate_check. The "deliberately NOT closing" axes (Hermes gateway, liberal skill-minting, centralized telemetry) authored as NOTHING, by design. No commit (docs gitignored).
