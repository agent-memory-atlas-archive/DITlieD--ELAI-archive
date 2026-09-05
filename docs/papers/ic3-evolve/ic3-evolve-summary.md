IC3-Evolve: Proof-/Witness-Gated Offline LLM-Driven Heuristic Evolution for IC3 Hardware Model Checking

Source: https://arxiv.org/abs/2604.03232
Authors: Mingkai Miao, Guangyu Hu, Ziyi Yang, Hongce Zhang
Date: January 18, 2026
Category: cs.AI

---

Abstract

IC3-Evolve is an automated framework that uses LLMs to evolve the heuristics of IC3, a hardware safety verification algorithm. The system proposes code modifications to an IC3 implementation while enforcing rigorous correctness: SAFE runs must emit a certificate (inductive invariant) independently checked by certifaiger, and UNSAFE runs must emit a replayable counterexample trace validated by aigsim. The evolved tool operates standalone with zero LLM inference at runtime. On both public HWMCC benchmarks and industrial verification instances, the approach discovers practical heuristic improvements that outperform the HWMCC'24/25 winner.

---

1. Problem Statement

IC3 (Incremental Construction of Inductive Clauses for Indubitable Correctness) is the dominant algorithm for hardware safety verification. Its performance depends heavily on heuristic decisions (worklist ordering, clause generalization, push propagation) that are hand-tuned by experts. Prior approaches either embed ML models at runtime (adding inference overhead and soundness risk) or use LLMs in-the-loop during verification (requiring LLM availability at runtime).

IC3-Evolve eliminates both problems: the LLM is used offline to evolve the source code of the checker itself, producing a standalone binary with zero runtime ML dependency.

![Figure 1: Comparison of approaches](images/fig1-approaches-comparison.png)
Figure 1: Three paradigms compared. (a) Learning-augmented IC3 embeds ML models at runtime. (b) LLM-in-the-loop calls the LLM during verification. (c) IC3-Evolve uses the LLM offline to evolve source code; the deployed binary is standalone.

---

2. IC3 Algorithm Background

IC3 maintains a frame sequence F_0, ..., F_k where each frame over-approximates the set of reachable states at that depth. The algorithm works on transition systems (I(x), T(x,y,x'), P(x)) where I is the initial states, T is the transition relation, and P is the safety property.

Algorithm 1: IC3 Overview

    Input: (I(x), T(x,y,x'), P(x))
    Output: SAFE or UNSAFE

    F_0 <- I; F_1 <- P; k <- 1; Q <- empty
    if I AND NOT(P) is SAT or I AND T AND NOT(P') is SAT then
      return UNSAFE
    end if

    while true do
      while F_k AND T AND NOT(P') is SAT with witness cube s do
        if not BlockOne(s, k) then
          return UNSAFE
        end if
      end while
      PushClauses(k)
      if exists i < k: F_i = F_{i+1} then
        return SAFE    // fixpoint reached
      end if
      k <- k + 1; F_k <- P
    end while

    Function BlockOne(s, k):
      insert (s, k) into Q
      return BlockProofObligations(Q)

Algorithm 2: IC3 Key Subroutines

    Function BlockProofObligations(Q):
      while Q is not empty do
        (s, i) <- Pop(Q)           // PO handling heuristic: which obligation to process
        if F_i AND s is UNSAT then continue
        if i = 0 then return false // counterexample found
        if F_{i-1} AND T AND s' is SAT then
          p <- PredGen(GetWitness(), i-1, s)   // predecessor generalization
          insert (s, i) into Q
          insert (p, i-1) into Q
        else
          c <- IndGen(s, i)        // inductive generalization
          for j = 1 to i do
            F_j <- F_j AND c
          end for
        end if
      end while
      return true

    Function PredGen(p, j, s):
      return MinimizePred(F_j, T, p, s)

    Function IndGen(s, i):
      return RelInductiveMinimize(I, F_{i-1}, T, NOT(s))

    Function PushClauses(k):
      for i = 1 to k-1 do
        foreach c in F_i \ F_{i+1} do
          if F_i AND T AND NOT(c') is UNSAT then
            F_{i+1} <- F_{i+1} AND c
          end if
        end foreach
      end for

Key insight: every subroutine contains heuristic choices (Pop ordering, literal dropping in IndGen, assumption ordering in PredGen, push ordering in PushClauses) that dramatically affect performance but are hard to optimize by hand.

---

3. System Architecture

![Figure 2: IC3-Evolve Framework](images/fig2-framework-overview.png)
Figure 2: Framework overview. Compass&Jump selects slot(s); a programmer agent proposes a slot-restricted patch with hypothesis; an evaluator agent validates via proof/witness gates and performance metrics; accepted patches are promoted.

The framework orchestrates two LLM agents (programmer and evaluator) in a closed loop:

3.1 Heuristic Slots

The code is partitioned into editable "slots" — logically coherent regions that correspond to specific heuristic decisions:

Table 1: Heuristic Slots

| Slot         | Scope (editable functions & metrics)                                                         |
|------------- |----------------------------------------------------------------------------------------------|
| PO handling  | BlockProofObligations: worklist order, stale-PO skip, requeue/budget. Metrics: PO depth, SAT/PO, requeue rate |
| Ind. gen.    | IndGen: literal dropping, UNSAT cores, CTG/MIC budgets. Metrics: core size, lemma yield, generalization SAT calls |
| Pred. gen.   | PredGen: assumption order, predecessor selection, cooldown. Metrics: chain length, predecessor SAT calls, depth |
| Push/prop.   | PushClauses: push order, caching, subsumption. Metrics: push success, redundancy, frame stabilization |
| Cross-slot   | Small multi-slot changes and synergies. Metrics: overall PAR2 and bucketed regressions        |

3.2 Slot-Restricted Patch Space

Each iteration restricts edits to one slot (Compass mode) or a small number of slots (Jump mode). Constraints:
- Max 80 added lines per patch
- Max 3 modified files
- Must compile with existing Makefile
- Edits restricted to IC3*, Model*, main.cpp
- Forbidden directories: aiger/, minisat/, scorr_aig/, test/
- Return codes preserved: 0 = SAFE, 1 = UNSAFE
- All SAT solver calls remain unmodified

3.3 Two-Agent Iteration Protocol

1. Compass&Jump selector chooses the slot scope and guidance moves
2. Programmer agent receives: slot focus, current code snippets, metrics, logs, knowledge pack, MoveSet guidance
3. Programmer proposes a patch with explicit hypothesis and fallback plan
4. Evaluator agent applies the patch, builds the code, runs hard correctness checks
5. Tests on evolution workload (100 HWMCC instances, 1800s timeout per instance)
6. Evaluator diagnoses results, returns ranked MoveSet for next iteration guidance

3.4 Knowledge Packs

Curated per-slot references containing:
- Relevant code excerpts from the current implementation
- Academic paper snippets (IC3 original, PDR, generalization techniques)
- Implementation notes and known pitfalls

Retrieved by relevance to the current slot focus, length-capped to fit context budget.

---

4. Proof-/Witness-Gated Validation

This is the core safety mechanism. Every proposed patch must pass hard correctness gates BEFORE performance metrics are even considered.

Gate 1 — SAFE Certificate Validation:
  - For every instance returning SAFE, the evolved checker must emit an inductive invariant
  - The invariant is independently validated by certifaiger
  - If validation fails, the patch is rejected regardless of performance

Gate 2 — UNSAFE Witness Validation:
  - For every instance returning UNSAFE, the evolved checker must emit a counterexample trace
  - The trace is replayed by aigsim against the original circuit
  - If replay fails (trace doesn't reach a bad state), the patch is rejected

Gate 3 — Consistency:
  - Return codes must match expected results
  - No missing artifacts (invariants or traces)
  - No mismatched SAFE/UNSAFE classifications vs. known ground truth

Gate 4 — Regression Budget:
  - Performance must not regress beyond a configurable threshold
  - Measured by PAR2 score (penalized average runtime, 2x timeout for unsolved)

Promotion: A patch is promoted to the current best ONLY if all hard gates pass AND performance improves.

Critical property: This separates optimization from soundness. The LLM is free to propose any heuristic change, but the proof/witness gates are an independent, non-LLM verification layer that prevents unsound modifications from ever being deployed.

---

5. Compass & Jump Search Policy

Algorithm 3: Compass & Jump

    Input: slots S; MoveSet M; progress history H;
           jump probability p_jump; jump size J in {2, 3}
    Output: allowed slots A subset of S; guidance moves G; updated p_jump

    if M = empty then
      sample s ~ Uniform(S)
      return ({s}, empty, p_jump)
    end if

    p_jump <- AdjustJump(p_jump, H)
    M <- RankByScore(M)

    if Rand() < p_jump then
      G <- TopDistinctSlots(M, J)     // Jump: pick top J moves from distinct slots
    else
      m* <- SelectBest(M, H)          // Compass: single best move
      G <- {m*}
    end if

    A <- {m.slot | m in G}
    return (A, G, p_jump)

Scoring formula:

    score(m) = w_c * conf(m) - w_r * risk(m) - w_k * cost(m)

where m = (slot, direction, confidence, risk, cost)

The jump probability p_jump adapts based on progress history: it increases when single-slot exploration stalls and decreases after successful single-slot improvements. This balances exploitation (deepening a promising slot) with exploration (discovering cross-slot synergies).

---

6. Prompt Assembly and Context Management

Algorithm 4: Prompt Assembly with Bounded Context

    Input: diff, metrics, logs, KB, slot focus
    Output: assembled prompt

    1. Assemble core fields (slot focus, diff summary, key metrics)
    2. Attach local code snippets for target functions (budgeted)
    3. Attach logs for top-k% slow cases (clip by lines/chars)
    4. Attach slot-relevant KB snippets (cap length)

    if prompt exceeds budget or agent output missing then
      Drop KB; shorten logs/metrics/moveset; retry (slim prompt)
    end if

Hierarchical prompt budgeting assigns explicit token limits to:
- Code snippets (highest priority)
- Metrics and statistics
- Log excerpts (filtered to worst-performing instances)
- Knowledge base retrieval (lowest priority, dropped first)

Automatic slim-prompt fallback omits the knowledge base when context overflows, ensuring robust operation across long evolution runs (200+ iterations).

---

7. Experimental Setup

Starting point: IC3ref (Bradley 2013) — lightweight reference implementation
Evolution budget: 200 iterations (100 structured sweep + 100 Compass&Jump)
Evolution workload: 100 public HWMCC instances (Set A)
Evaluation workload: 100 unseen HWMCC instances (Set B) + 302 industrial instances
LLM: GPT-5-Codex
SAT backend: CaDiCaL 2.0
Timeout: 1800s per instance
Metric: PAR2 (penalized average runtime; unsolved = 2 * timeout)
Format: AIGER (standard hardware model checking format)

---

8. Results

Table 2: Main Results

| Solver          | HWMCC Set A        |          |        |          | HWMCC Set B        |          |        |          | Industrial (302)   |          |        |          |
|-----------------|-------|---------|-------|---------|-------|---------|-------|---------|-------|---------|-------|---------|
|                 | #S/#T | #UNSAFE | #SAFE | PAR2(s) | #S/#T | #UNSAFE | #SAFE | PAR2(s) | #S/#T | #UNSAFE | #SAFE | PAR2(s) |
| IC3ref          | 75/100 | 21     | 54    | 1050.61 | 75/100 | 26     | 49    | 1068.33 | 187/302 | 39    | 148   | 1403.31 |
| IC3ref-CaDiCaL  | 85/100 | 22     | 63    | 718.87  | 85/100 | 28     | 57    | 732.28  | 198/302 | 38    | 160   | 1302.13 |
| ABC             | 86/100 | 22     | 64    | 665.59  | 88/100 | 26     | 62    | 618.59  | 185/302 | 36    | 149   | 1417.50 |
| rIC3-CaDiCaL    | 86/100 | 21     | 65    | 639.61  | 89/100 | 27     | 62    | 524.35  | 182/302 | 36    | 146   | 1449.78 |
| IC3-Evolve      | 92/100 | 24     | 68    | 490.67  | 94/100 | 28     | 66    | 464.56  | 225/302 | 36    | 189   | 1044.26 |

Key takeaways:
- IC3-Evolve solves 94/100 unseen HWMCC instances vs. 89 for the HWMCC'24/25 winner (rIC3)
- PAR2 reduction of 11.4% over rIC3 on unseen public benchmarks (464.56 vs 524.35)
- On industrial benchmarks: 225/302 solved vs 182 for rIC3 (23.6% more instances), PAR2 reduced by 28%
- Gains transfer to unseen instances — not overfit to evolution workload

![Figure 3: Evolution trace](images/fig3-evolution-trace.png)
Figure 3: Representative evolution trace on the HWMCC optimization workload. Bars show attempted challengers (green = accepted, red = rejected). Line shows best-so-far PAR2. Progress arrives as punctuated jumps interspersed with rejected candidates.

---

9. Case Study: Push Propagation Evolution (Rounds r000-r013)

Table 4: Push-Propagation Milestones

| Round              | PAR2 (s)  | Timeouts |
|--------------------|-----------|----------|
| r000 (baseline)    | 1050.61   | 25       |
| r002 (Stage 1)     | 1050.42   | 25       |
| r004 (Stage 2)     | 983.73    | 22       |
| r013 (Stage 3)     | 943.07    | 21       |

Stage 1 (r002) — Instrumentation:
  Gate simplification applied only after observing actual pushing behavior. Progress monitoring counters introduced. Minimal PAR2 change but establishes the measurement infrastructure.

Stage 2 (r004) — Gating:
  Skip repeatedly stalled frames using stallStreak tracking. If a frame fails to push clauses for N consecutive attempts, skip it temporarily. PAR2 drops 6.4% (1050.61 -> 983.73), timeouts drop from 25 to 22.

Stage 3 (r013) — Adaptive Budgeting:
  Reallocate push effort from stalled frames to productive ones. Frames with high push success rates get more budget; stalled frames get less. Final PAR2: 943.07 (10.2% total improvement over baseline).

Autonomous discovery: The LLM independently discovered the instrument -> gate -> adapt progression, a well-known engineering pattern, without being told to follow it.

---

10. Ablation Studies

Table 3: Single-Slot Isolation and Naive Composition

| Variant         | Set A              | Set B              |
|-----------------|--------------------|--------------------|
|                 | #Solved / PAR2(s)  | #Solved / PAR2(s)  |
| Baseline        | 75 / 1050.61       | 75 / 1068.33       |
| PO-only         | 78 / 995.35        | 75 / 1093.42       |
| PredGen-only    | 79 / 954.36        | 78 / 970.38        |
| IndGen-only     | 78 / 927.80        | 84 / 803.73        |
| Push-only       | 76 / 1016.05       | 75 / 1139.22       |
| Naive-Compose   | 78 / 944.86        | 85 / 775.19        |
| IC3-Evolve      | 92 / 490.67        | 94 / 464.56        |

Key findings:
- Single-slot evolution yields unstable, non-transferable gains (PO-only and Push-only actually regress on Set B)
- Naive composition (merging all single-slot patches) performs much worse than integrated evolution
- Cross-slot synergies are essential: IC3-Evolve's integrated approach discovers interactions that isolated optimization cannot

10.1 No-Gate Counterfactual

Rounds r093-r094 proposed a clause-cleaning heuristic showing +9 solved instances on the evolution set. However, it failed independent witness validation — the heuristic was producing incorrect UNSAFE classifications that happened to improve performance metrics. The proof/witness gate caught and rejected this unsound "improvement."

This demonstrates that performance improvement alone is insufficient; independent verification gates are necessary to prevent unsound optimizations.

---

11. Context Management Details

Hierarchical prompt budget allocation:
1. Slot focus and patch context (highest priority)
2. Code snippets for target functions (budgeted by lines)
3. Metrics and statistics (structured, compact)
4. Execution logs for worst-performing instances (top-k%, clipped)
5. Knowledge base snippets (lowest priority, dropped first in slim mode)

When context exceeds the model's budget or the agent fails to produce structured output, the system falls back to a slim prompt that omits the knowledge base, shortens logs/metrics/moveset, and retries. This ensures robustness across 200+ iteration runs.

---

12. Baselines and Tools

| Tool/Solver     | Description                                           |
|-----------------|-------------------------------------------------------|
| IC3ref          | Bradley's 2013 reference IC3 implementation           |
| IC3ref-CaDiCaL  | IC3ref with CaDiCaL 2.0 SAT backend (stronger solver)|
| ABC             | Industrial-strength verification toolchain            |
| rIC3-CaDiCaL    | HWMCC'24/25 competition winner with CaDiCaL           |
| certifaiger     | Independent proof certificate validator               |
| aigsim          | AIGER counterexample trace replay utility             |
| CaDiCaL 2.0     | State-of-the-art SAT solver                          |
| GPT-5-Codex     | LLM used for code evolution (offline only)            |

---

13. Key References

- Bradley (2011): IC3 — original algorithm
- Een et al. (2011): PDR (Property Directed Reachability) — equivalent formulation
- Su et al. (2024): Lemma prediction in IC3 generalization
- Biere et al. (2024): CaDiCaL SAT solver
- Yu et al. (2021): Proof certification in hardware model checking
- HWMCC: https://hwmcc.github.io/ — Hardware Model Checking Competition

---

14. How This Could Be Useful to ELAI

14.1 Zero-Overhead Rule Generation (Phase 3F)

The paper's core thesis is using LLMs to evolve heuristics offline so the deployed system carries zero ML inference overhead. This aligns exactly with the design of the elai_cc Rules Engine. While the auditor agent might discover violation patterns, those patterns are meant to be solidified into deterministic AST or GREP rules that execute in <1ms.

The slot-restricted patching approach can be adapted to strictly control how the system auto-generates contract tests and transitions rules from PROPOSED to ACTIVE. By constraining where and how the LLM can propose rule changes, you reduce the search space and prevent it from breaking the core Predicate DSL interpreter.

14.2 Proof-Gated Self-Modification (Phase 4H)

IC3-Evolve does not trust the LLM; it gates every proposed patch behind an independently verifiable certificate (inductive invariant) or a replayable counterexample trace. This is directly applicable to the Layer 2 log-driven config proposals and staged evaluations.

The self-modification agent currently relies on metrics and SWE-bench grading. This can be strengthened by integrating a proof-gated requirement: any config variant or structural change proposed by the self-mod agent must first output a replayable trace that perfectly matches the Deterministic Simulation Testing (DST) reference model. If the simulation diverges, the patch is rejected before it even reaches the config archive.

14.3 Application to Quantitative Trading and Agentic Empire

The architecture — offline heuristic evolution protected by deterministic gating — maps directly onto quantitative trading algorithms and automated SaaS production pipelines.

Instead of having an LLM actively make live trading or infrastructure decisions, use the LLM offline to evolve the mathematical heuristics of trading algorithms. The "witness gate" would be a strict, deterministic backtest against historical financial data. If the evolved heuristic passes the backtest with mathematical proof of profitability and risk compliance, the static compiled code is deployed. This keeps the live environment fast, deterministic, and immune to runtime hallucinations.

Key pattern: LLM generates candidates offline -> independent deterministic gate validates -> only proven artifacts deploy to production.
