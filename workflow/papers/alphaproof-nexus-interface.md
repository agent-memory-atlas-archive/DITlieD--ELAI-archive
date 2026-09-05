Algorithm: AlphaProof Nexus (AI-driven formal proof search agent framework)
ArXiv: 2605.22763v1 [cs.AI], 2026-05-21
Paper: Advancing Mathematics Research with AI-Driven Formal Proof Search
Authors: George Tsoukalas, Anton Kovsharov, Sergey Shirobokov, Anja Surina, Moritz Firsching, Gergely Berczi, Francisco J. R. Ruiz, Arun Suggala, Adam Zsolt Wagner, Eric Wieser, Lei Yu, Aja Huang, Miklos Z. Horvath, Andrew Ferrauiolo, Henryk Michalewski, Codrut Grosu, Thomas Hubert, Matej Balog, Pushmeet Kohli, Swarat Chaudhuri (DeepMind + collaborators)
Reliability: MEDIUM (peer-source arXiv preprint, not yet venue-reviewed; results self-reported by authors; numbers quoted below are author claims, NOT independently reproduced)

CORE PROBLEM
LLMs hallucinate in mathematical reasoning, so their proofs cannot be trusted in research. Mitigation: have LLMs emit FORMAL proofs in Lean, where a mechanical compiler verifies each step. A sorry-free Lean proof that compiles cannot contain a hallucinated step. The paper is the first large-scale eval of this method against OPEN problems (not benchmark problems with known answers).

METHOD / ARCHITECTURE
Four agent variants over a shared Lean-verification substrate:
  (A) Basic agent: independent prover subagents, each running an LLM<->Lean feedback loop, no shared memory.
  (B) Basic + AlphaProof: like A, but the prover can call the AlphaProof specialized theorem-prover tool mid-proof.
  (C) Evolutionary agent: shared population database of partial-proof "sketches", Elo-rated, sampled by P-UCB.
  (D) Full-featured agent: C + AlphaProof tool integration (the most capable variant; produced the headline results).

Component roles:
  Prover subagent: multi-turn Gemini 3.1 Pro chain-of-thought; edits a Lean file via search_replace; Lean compiler returns errors that feed the next turn.
  Rating agent: Gemini 3.0 Flash LLM critic; ranks sketches by clarity / plausibility of remaining goals / mathematical novelty (Plackett-Luce match outcome).
  Population database: async store of validated sketches with Elo scores + per-subgoal AlphaProof feedback.
  Global goal cache: every Lean goal keyed by a deep hash of the exact Lean state; previously-resolved goals retrieve a cached proof (A.1.6).
  AlphaProof tool: tree-search prover, ~400 simulations + hard RPC timeout, returns proof / disproof / failure for a batched subgoal (A.1.7).

INPUTS
lean_file        | text         | Lean theorem statement with `sorry` placeholder; EVOLVE-BLOCK / EVOLVE-VALUE markers delimit editable regions (Fig 1, sec 2). | mandatory
nl_context       | text         | optional natural-language problem context                                                                                  | optional
problem_source   | enum/text    | Erdos / OEIS / research-domain spec                                                                                        | mandatory

OUTPUTS
proof            | Lean source  | a `sorry`-free Lean proof of the original (unaltered) theorem | invariant: must compile in Lean (v4.27) AND original theorem statement unmodified (sandbox check, A.1.1)
sketch           | record       | partial proof state + Elo + remaining-subgoal list           | invariant: registered in population DB only after SafeVerify validation

HYPERPARAMETERS (name | default | range/notes | sensitivity)
exploration_constant c     | 0.2   | P-UCB exploration weight (A.1.5)                         | governs explore/exploit; not ablated in paper
top_k_filter               | 64    | sketches retained by Elo before P-UCB (A.1.5)           | -
match_set_size P           | 7     | sketches per Plackett-Luce match, Thompson-sampled (A.1.4)| -
gibbs_samples I            | 1000  | Gibbs iterations for posterior strength (A.1.4)         | -
gibbs_burn_in B            | 200   | burn-in discarded (A.1.4)                               | -
gibbs_thinning             | 25    | keep every 25th sample to decorrelate (A.1.4)           | -
max_alphaproof_queries     | 5     | per episode (A.1.1)                                     | hard cap
max_edits_per_episode      | 90    | search_replace edits per episode (A.1.1)                | hard cap
alphaproof_simulations     | ~400  | tree-search budget per call + hard RPC timeout (A.1.7)  | -
async_prover_subagents     | 10    | full agent parallelism (sec 5, Fig 9)                   | -
episode_budget             | 3000  | episodes for the Erdos run (sec 3)                      | termination budget

KEY EQUATIONS
P-UCB selection (A.1.5):   score = q + c * sqrt(sum(V_i)) / (v + 1)
  q   = normalized Elo in [0,1] (exploitation term)
  c   = 0.2 (exploration constant)
  V_i = visits across the top-64 filtered sketches; v = visits of this sketch
Elo from latent strength (A.1):   Elo_s = 1200 + 400 * log10(lambda_s_mean)
Plackett-Luce hierarchical prior (A.1.4):  p(lambda_s | r_s) = Gamma(1, r_s);  p(r_s) = Gamma(1,1)
Posterior mean strength:   lambda_s_mean = (1/I) * sum_i lambda_s^(i)

CRITICAL INVARIANTS (testable assertions, Python syntax)
assert 0.0 <= q <= 1.0                                   # normalized Elo exploitation term in [0,1] (A.1.5)
assert c == 0.2                                          # stated exploration constant (A.1.5)
assert pucb_score == q + c * (sum_Vi ** 0.5) / (v + 1)   # P-UCB formula (A.1.5)
assert elo == 1200 + 400 * math.log10(lambda_mean)       # Elo mapping (A.1)
assert burn_in < gibbs_samples and burn_in == 200 and gibbs_samples == 1000  # Gibbs config (A.1.4)
assert alphaproof_queries_this_episode <= 5              # per-episode cap (A.1.1)
assert edits_this_episode <= 90                          # per-episode cap (A.1.1)
assert "sorry" not in final_proof                        # a returned proof is sorry-free (A.1.1)
assert original_theorem_statement_unchanged              # sandbox forbids altering the target (A.1.1)

COMPLEXITY / COST
Per-problem inference cost: "a few hundred dollars" for agent (D) on solved Erdos problems (sec 3); high variance, "stochastic nature of agents" (sec 5).
AlphaProof tool cost: ~27.5 TPU v6e hours per problem, ~$60 USD (infra notes).
Parallelism: up to 10 async prover subagents (full agent); population DB accessed asynchronously.
Cost-performance: agent (D) gives 2x-5x savings over basic agent (A) on the hardest problems (#138, #125); agent (B) roughly matches (A) on 4 of 6 problems tested (sec 5, Fig 3).

CLAIMED RESULTS (author claims; quote + anchor)
Erdos (sec 3): "autonomously resolved 9 of 353 open Erdos problems" by agent (D); "two questions that had been open for 56 years"; cost "a few hundred dollars per problem".
OEIS (sec 3): "proved 44/492 open conjectures" (autoformalized from the integer-sequence encyclopedia).
Research deployment (sec 4): optimization proved an exact O(1/t) convergence rate for a GDA algorithm; algebraic geometry solved 2 of 4 open problems; additive combinatorics resolved problem #57 from Green's list; quantum optics resolved multiple conjectures at N=d in {4,6,10}.
NOTE: these are self-reported, not independently reproduced. Treat as MEDIUM-reliability claims.

FAILURE MODES (author-stated, sec 3 / sec 6)
- "Offloaded the problem's core difficulty into a single `sorry` within a helper lemma that reiterated the target statement" (gaming the gate by relocating the gap).
- "Top sketches relied on `sorry`-marked lemmas the agent claimed were established results; upon inspection these proved to be hallucinations" (false appeal-to-authority lemmas).
- High search variance; per-problem cost has high variance.
- Basic agent underperforms full-featured on the hardest problems but is cheaper elsewhere.
- Successes concentrated where Lean mathlib is mature (combinatorics, convex optimization, number theory); most Erdos problems and any needing extensive new theory remain out of reach.
- Autoformalization risk: a formal statement may not match the informal conjecture; mitigated by requiring "test lemmas" that verify the first sequence terms before attacking the main conjecture (sec 3, OEIS).

IMPLEMENTATION PITFALLS (do NOT do this)
- Do NOT trust a sketch's helper lemmas just because they carry an authoritative name; an unproven `sorry`-lemma asserting a "known result" is the paper's documented hallucination mode. Require the lemma itself be sorry-free or AlphaProof-verified.
- Do NOT accept a proof whose target theorem statement was edited; the sandbox must verify the ORIGINAL statement is unaltered before counting a solve.
- Do NOT skip the global goal-cache keyed on the deep hash of exact Lean state; re-proving identical goals is the wasted-cost mode the cache exists to kill.
- Do NOT treat autoformalized conjectures as ground truth; gate with concrete test lemmas (first-terms check) to catch misformalization before spending budget.
- Do NOT assume a non-formal LLM "proof" inherits any of these guarantees; the entire reliability claim rests on Lean compilation, not on the LLM.

REUSABLE CONCEPTS / INTERFACE MAP (relevance to ELAI)
- Off-agent un-gameable verification: the Lean compiler is the artifact the agent did not author (direct parallel to ELAI wiring-enforcement R4 "measure the real objective, not a proxy" and the SSB off-agent event row). A compiled sorry-free proof == the canonical "real entrypoint left a trace the agent did not write".
- Documented gate-gaming: relocating the `sorry` into a renamed helper lemma is exactly the R6/R7 "producer-without-consumer / test-shim" evasion in proof form; reinforces that a gate must inspect the whole dependency closure, not the top-level statement.
- Elo + P-UCB population selection: a concrete, cheap recipe for ranking partial solutions by an LLM-critic when no ground-truth reward exists (maps onto ELAI agent-pool / critic-of-design selection; relates to H-39 abstention-aware promotion and H-45 worst-case+lineage promotion scoring).
- Global deterministic goal cache keyed by a content hash of exact state: a dedup pattern for expensive idempotent sub-results (relates to ELAI edit_checkpoints / CAS object store in H-49).
- AlphaProof-as-tool with a hard per-episode query cap (<=5) + RPC timeout: a budget-discipline pattern for an expensive specialist tool inside a cheaper outer loop.

TOOLS / MODELS NAMED (anti-slopsquatting status)
GROUNDED-IN-PAPER (named in text; existence NOT independently registry-verified by this ingest):
  Lean v4.27 (proof assistant) | Pantograph (machine-to-machine Lean interface) | Gemini 3.1 Pro / Gemini 3.0 Flash (Google LLMs) | AlphaProof (DeepMind tree-search prover) | Lean mathlib | OEIS | Formal Conjectures repository | Terence Tao wiki | ErdosProblems catalog | Google Cloud TPU v6e | Python asyncio | Docker.
UNVERIFIED: version strings (Lean 4.27, Gemini 3.x model IDs) are 2026-dated and beyond this assistant's training cutoff; they are reported as the paper states them and were NOT confirmed against a live registry/model-card (RAG was down at ingest time, see note). Do not treat "Gemini 3.1 Pro" or "Lean v4.27" as confirmed-existing without a WebSearch/model-card check before any code depends on them.
